import { execFile } from 'node:child_process'
import { readFile, writeFile } from 'node:fs/promises'
import { promisify } from 'node:util'
import path from 'node:path'
import process from 'node:process'

const execFileAsync = promisify(execFile)
const cwd = process.cwd()

const packageJsonPath = path.join(cwd, 'package.json')
const cargoTomlPath = path.join(cwd, 'src-tauri', 'Cargo.toml')

function parseArgs(argv) {
  const options = {
    remote: 'origin',
    branch: '',
    message: '',
    dryRun: false
  }

  const positionals = []

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i]

    if (arg === '--dry-run') {
      options.dryRun = true
      continue
    }

    if (arg === '--remote') {
      options.remote = argv[i + 1] || ''
      i += 1
      continue
    }

    if (arg.startsWith('--remote=')) {
      options.remote = arg.slice('--remote='.length)
      continue
    }

    if (arg === '--branch') {
      options.branch = argv[i + 1] || ''
      i += 1
      continue
    }

    if (arg.startsWith('--branch=')) {
      options.branch = arg.slice('--branch='.length)
      continue
    }

    if (arg === '--message' || arg === '-m') {
      options.message = argv[i + 1] || ''
      i += 1
      continue
    }

    if (arg.startsWith('--message=')) {
      options.message = arg.slice('--message='.length)
      continue
    }

    if (arg === '--help' || arg === '-h') {
      printHelp()
      process.exit(0)
    }

    positionals.push(arg)
  }

  if (!options.message && positionals.length > 0) {
    options.message = positionals.join(' ')
  }

  if (!options.remote) {
    throw new Error('`--remote` cannot be empty')
  }

  return options
}

function printHelp() {
  console.log(`
Usage:
  npm run release:github -- [commit message]
  npm run release:github -- --message "feat: xxx"

Options:
  --remote <name>   Git remote name, default: origin
  --branch <name>   Push branch name, default: current branch
  --message, -m     Commit message, default: chore: release vX.Y.Z
  --dry-run         Only print the steps, do not modify git or files

What it does:
  1. Read the current version from package.json
  2. Auto increment patch version by 1
  3. Sync version to src-tauri/Cargo.toml
  4. git add -A
  5. git commit
  6. git tag -a vX.Y.Z
  7. git push <remote> <branch>
  8. git push <remote> vX.Y.Z
`)
}

async function runGit(args, options = {}) {
  const { stdout, stderr } = await execFileAsync('git', args, {
    cwd,
    maxBuffer: 1024 * 1024 * 10,
    ...options
  })
  return { stdout: stdout.trim(), stderr: stderr.trim() }
}

function parseVersion(input) {
  const match = /^v?(\d+)\.(\d+)\.(\d+)$/.exec(input.trim())
  if (!match) {
    throw new Error(`Unsupported version format: ${input}`)
  }

  return {
    major: Number(match[1]),
    minor: Number(match[2]),
    patch: Number(match[3])
  }
}

function compareVersion(a, b) {
  if (a.major !== b.major) {
    return a.major - b.major
  }
  if (a.minor !== b.minor) {
    return a.minor - b.minor
  }
  return a.patch - b.patch
}

function formatVersion(version) {
  return `${version.major}.${version.minor}.${version.patch}`
}

function bumpPatch(version) {
  return {
    major: version.major,
    minor: version.minor,
    patch: version.patch + 1
  }
}

async function getCurrentBranch() {
  const { stdout } = await runGit(['branch', '--show-current'])
  if (!stdout) {
    throw new Error('Unable to detect current branch')
  }
  return stdout
}

async function getLatestTag() {
  const { stdout } = await runGit(['tag', '--sort=-v:refname'])
  const firstLine = stdout.split('\n').find(Boolean)
  return firstLine || ''
}

async function getLatestRemoteTag(remote) {
  const { stdout } = await runGit(['ls-remote', '--tags', remote])
  const tagNames = stdout
    .split('\n')
    .map((line) => line.trim())
    .filter(Boolean)
    .map((line) => line.split('\t')[1] || '')
    .filter((ref) => ref.startsWith('refs/tags/'))
    .map((ref) => ref.replace(/^refs\/tags\//, '').replace(/\^\{\}$/, ''))
    .filter((tag, index, all) => /^v\d+\.\d+\.\d+$/.test(tag) && all.indexOf(tag) === index)

  if (tagNames.length === 0) {
    return ''
  }

  return tagNames.sort((left, right) => compareVersion(parseVersion(right), parseVersion(left)))[0]
}

async function ensureTagDoesNotExist(tagName, remote) {
  const { stdout } = await runGit(['tag', '--list', tagName])
  if (stdout === tagName) {
    throw new Error(`Tag already exists locally: ${tagName}`)
  }

  const remoteRefs = await runGit(['ls-remote', '--tags', remote, tagName, `${tagName}^{}`])
  if (remoteRefs.stdout) {
    throw new Error(`Tag already exists on remote ${remote}: ${tagName}`)
  }
}

async function readVersions() {
  const packageJsonRaw = await readFile(packageJsonPath, 'utf8')
  const packageJson = JSON.parse(packageJsonRaw)
  const cargoTomlRaw = await readFile(cargoTomlPath, 'utf8')

  if (typeof packageJson.version !== 'string') {
    throw new Error('package.json version is missing')
  }

  const cargoVersionMatch = cargoTomlRaw.match(/^version\s*=\s*"([^"]+)"/m)
  if (!cargoVersionMatch) {
    throw new Error('src-tauri/Cargo.toml version is missing')
  }

  return {
    packageJson,
    packageJsonRaw,
    cargoTomlRaw,
    packageVersion: packageJson.version,
    cargoVersion: cargoVersionMatch[1]
  }
}

async function writeVersions(nextVersion) {
  const { packageJson, cargoTomlRaw } = await readVersions()

  packageJson.version = nextVersion

  const nextPackageJson = `${JSON.stringify(packageJson, null, 2)}\n`
  const nextCargoToml = cargoTomlRaw.replace(
    /^version\s*=\s*"([^"]+)"/m,
    `version = "${nextVersion}"`
  )

  await writeFile(packageJsonPath, nextPackageJson, 'utf8')
  await writeFile(cargoTomlPath, nextCargoToml, 'utf8')
}

async function main() {
  const options = parseArgs(process.argv.slice(2))

  const branch = options.branch || await getCurrentBranch()
  const remote = options.remote

  const { packageVersion, cargoVersion } = await readVersions()

  if (packageVersion !== cargoVersion) {
    throw new Error(
      `Version mismatch: package.json=${packageVersion}, src-tauri/Cargo.toml=${cargoVersion}`
    )
  }

  const latestTag = await getLatestTag()
  const latestRemoteTag = await getLatestRemoteTag(remote)
  const baseVersion = parseVersion(packageVersion)

  let releaseBase = baseVersion
  if (latestTag) {
    const tagVersion = parseVersion(latestTag)
    if (compareVersion(tagVersion, baseVersion) > 0) {
      releaseBase = tagVersion
    }
  }
  if (latestRemoteTag) {
    const remoteTagVersion = parseVersion(latestRemoteTag)
    if (compareVersion(remoteTagVersion, releaseBase) > 0) {
      releaseBase = remoteTagVersion
    }
  }

  const nextVersion = formatVersion(bumpPatch(releaseBase))
  const tagName = `v${nextVersion}`
  const commitMessage = options.message || `chore: release ${tagName}`

  await ensureTagDoesNotExist(tagName, remote)

  console.log(`Remote: ${remote}`)
  console.log(`Branch: ${branch}`)
  console.log(`Current version: ${packageVersion}`)
  console.log(`Latest local tag: ${latestTag || 'none'}`)
  console.log(`Latest remote tag: ${latestRemoteTag || 'none'}`)
  console.log(`Next version: ${nextVersion}`)
  console.log(`Commit message: ${commitMessage}`)

  if (options.dryRun) {
    console.log('Dry run enabled, no files or git history were changed.')
    return
  }

  await writeVersions(nextVersion)

  console.log('Staging files...')
  await runGit(['add', '-A'])

  console.log('Creating commit...')
  await runGit(['commit', '-m', commitMessage])

  console.log(`Creating tag ${tagName}...`)
  await runGit(['tag', '-a', tagName, '-m', tagName])

  console.log(`Pushing branch ${branch} to ${remote}...`)
  await runGit(['push', remote, branch])

  console.log(`Pushing tag ${tagName} to ${remote}...`)
  await runGit(['push', remote, tagName])

  console.log(`Release pushed successfully: ${tagName}`)
}

main().catch(async (error) => {
  console.error(error instanceof Error ? error.message : String(error))
  process.exit(1)
})

import { readFile, readdir } from 'node:fs/promises'
import path from 'node:path'
import process from 'node:process'

const API_BASE = 'https://gitee.com/api/v5'
const owner = process.env.GITEE_RELEASE_OWNER || 'chengccn1'
const repo = process.env.GITEE_RELEASE_REPO || 'rust-switchhost'
const accessToken = process.env.GITEE_RELEASE_TOKEN || process.env.GITEE_ACCESS_TOKEN
const tagName = process.env.GITEE_RELEASE_TAG
const artifactDir = path.resolve(
  process.cwd(),
  process.env.GITEE_RELEASE_ARTIFACT_DIR || 'src-tauri/target/release/bundle'
)

if (!accessToken) {
  throw new Error('Missing GITEE_RELEASE_TOKEN or GITEE_ACCESS_TOKEN environment variable')
}

if (!tagName) {
  throw new Error('Missing GITEE_RELEASE_TAG environment variable')
}

const version = tagName.replace(/^v/, '')
const releaseName = `Rust SwitchHost ${tagName}`
const releaseBody = [
  'Gitee 国内镜像自动发布。',
  '',
  `版本：${tagName}`,
  '',
  '说明：',
  '- GitHub Release 仍然是在线升级与签名产物的主发布源',
  '- Gitee Release 用于国内镜像下载'
].join('\n')

const uploadableExtensions = [
  '.AppImage',
  '.deb',
  '.rpm',
  '.tar.gz',
  '.tar.xz',
  '.zip',
  '.msi',
  '.exe',
  '.dmg'
]

async function walk(dir) {
  const entries = await readdir(dir, { withFileTypes: true })
  const files = await Promise.all(entries.map(async (entry) => {
    const fullPath = path.join(dir, entry.name)
    if (entry.isDirectory()) {
      return walk(fullPath)
    }
    return [fullPath]
  }))
  return files.flat()
}

function shouldUpload(filePath) {
  return uploadableExtensions.some((ext) => filePath.endsWith(ext))
}

async function requestJson(url, init = {}) {
  const response = await fetch(url, init)
  if (!response.ok) {
    throw new Error(`Request failed ${response.status}: ${await response.text()}`)
  }
  return response.json()
}

async function listReleases() {
  const url = new URL(`${API_BASE}/repos/${owner}/${repo}/releases`)
  url.searchParams.set('access_token', accessToken)
  url.searchParams.set('per_page', '100')
  return requestJson(url)
}

async function createRelease() {
  const body = new URLSearchParams({
    access_token: accessToken,
    tag_name: tagName,
    name: releaseName,
    body: releaseBody,
    target_commitish: 'main',
    prerelease: 'false'
  })

  return requestJson(`${API_BASE}/repos/${owner}/${repo}/releases`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/x-www-form-urlencoded'
    },
    body
  })
}

async function updateRelease(releaseId) {
  const body = new URLSearchParams({
    access_token: accessToken,
    tag_name: tagName,
    name: releaseName,
    body: releaseBody,
    target_commitish: 'main',
    prerelease: 'false'
  })

  return requestJson(`${API_BASE}/repos/${owner}/${repo}/releases/${releaseId}`, {
    method: 'PATCH',
    headers: {
      'Content-Type': 'application/x-www-form-urlencoded'
    },
    body
  })
}

async function ensureRelease() {
  const releases = await listReleases()
  const existing = releases.find((release) => release.tag_name === tagName)
  if (existing) {
    return updateRelease(existing.id)
  }
  return createRelease()
}

async function listAssets(releaseId) {
  const url = new URL(`${API_BASE}/repos/${owner}/${repo}/releases/${releaseId}/attach_files`)
  url.searchParams.set('access_token', accessToken)
  return requestJson(url)
}

async function uploadAsset(releaseId, filePath) {
  const fileBuffer = await readFile(filePath)
  const form = new FormData()
  form.append('access_token', accessToken)
  form.append('file', new Blob([fileBuffer]), path.basename(filePath))

  const response = await fetch(`${API_BASE}/repos/${owner}/${repo}/releases/${releaseId}/attach_files`, {
    method: 'POST',
    body: form
  })

  if (!response.ok) {
    throw new Error(`Failed to upload ${path.basename(filePath)}: ${response.status} ${await response.text()}`)
  }
}

async function main() {
  const release = await ensureRelease()
  const allFiles = await walk(artifactDir)
  const artifacts = allFiles.filter(shouldUpload)

  if (artifacts.length === 0) {
    throw new Error(`No release artifacts found in ${artifactDir}`)
  }

  const uploadedAssets = await listAssets(release.id)
  const uploadedNames = new Set(uploadedAssets.map((asset) => asset.name))

  for (const artifact of artifacts) {
    const fileName = path.basename(artifact)
    if (uploadedNames.has(fileName)) {
      console.log(`Skip existing asset: ${fileName}`)
      continue
    }

    console.log(`Uploading asset: ${fileName}`)
    await uploadAsset(release.id, artifact)
  }

  console.log(`Gitee release ${tagName} synced successfully`)
  console.log(`Mirror version: ${version}`)
}

await main()

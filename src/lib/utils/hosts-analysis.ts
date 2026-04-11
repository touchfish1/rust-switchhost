import type {
  HostsAffectedDomain,
  HostsConflictGroup,
  HostsContentStats,
  HostsDiffLine,
  HostsDiffSummary,
  Scheme
} from '$lib/types'

type ParsedHostsEntry = {
  domain: string
  ip: string
  schemeName: string
}

function stripInlineComment(line: string) {
  const commentIndex = line.indexOf('#')
  return commentIndex >= 0 ? line.slice(0, commentIndex) : line
}

function parseHostsEntries(content: string, schemeName: string): ParsedHostsEntry[] {
  const entries: ParsedHostsEntry[] = []

  for (const rawLine of content.split('\n')) {
    const contentPart = stripInlineComment(rawLine).trim()
    if (!contentPart) continue

    const tokens = contentPart.split(/\s+/).filter(Boolean)
    if (tokens.length < 2) continue

    const [ip, ...domains] = tokens
    for (const domain of domains) {
      entries.push({
        domain: domain.toLowerCase(),
        ip,
        schemeName
      })
    }
  }

  return entries
}

export function buildMergedHostsContent(schemes: Scheme[]) {
  if (schemes.length === 0) return ''

  return schemes
    .map((scheme) => `# Group: ${scheme.name}\n${scheme.content.trim()}`)
    .join('\n\n')
}

export function summarizeHostsContent(content: string): HostsContentStats {
  let lineCount = 0
  let hostEntryCount = 0
  let commentCount = 0

  for (const line of content.split('\n')) {
    if (!line && content.length === 0) continue

    lineCount += 1
    const trimmed = line.trim()
    if (!trimmed) continue

    if (trimmed.startsWith('#')) {
      commentCount += 1
      continue
    }

    const tokens = stripInlineComment(line).trim().split(/\s+/).filter(Boolean)
    if (tokens.length >= 2) {
      hostEntryCount += tokens.length - 1
    }
  }

  return {
    lineCount,
    hostEntryCount,
    commentCount
  }
}

export function summarizeHostsDiff(currentContent: string, nextContent: string): HostsDiffSummary {
  const currentLines = currentContent ? currentContent.split('\n') : []
  const nextLines = nextContent ? nextContent.split('\n') : []
  const currentCounts = new Map<string, number>()
  const nextCounts = new Map<string, number>()

  for (const line of currentLines) {
    currentCounts.set(line, (currentCounts.get(line) || 0) + 1)
  }

  for (const line of nextLines) {
    nextCounts.set(line, (nextCounts.get(line) || 0) + 1)
  }

  let unchangedLines = 0

  for (const [line, currentCount] of currentCounts.entries()) {
    unchangedLines += Math.min(currentCount, nextCounts.get(line) || 0)
  }

  return {
    addedLines: Math.max(0, nextLines.length - unchangedLines),
    removedLines: Math.max(0, currentLines.length - unchangedLines),
    unchangedLines
  }
}

export function collectHostsDiffLines(currentContent: string, nextContent: string): HostsDiffLine[] {
  const currentLines = currentContent ? currentContent.split('\n') : []
  const nextLines = nextContent ? nextContent.split('\n') : []
  const currentCounts = new Map<string, number>()
  const nextCounts = new Map<string, number>()

  for (const line of currentLines) {
    currentCounts.set(line, (currentCounts.get(line) || 0) + 1)
  }

  for (const line of nextLines) {
    nextCounts.set(line, (nextCounts.get(line) || 0) + 1)
  }

  const diffLines: HostsDiffLine[] = []

  for (const line of currentLines) {
    const remaining = nextCounts.get(line) || 0
    if (remaining > 0) {
      nextCounts.set(line, remaining - 1)
    } else if (line.trim()) {
      diffLines.push({ kind: 'removed', value: line })
    }
  }

  for (const line of nextLines) {
    const remaining = currentCounts.get(line) || 0
    if (remaining > 0) {
      currentCounts.set(line, remaining - 1)
    } else if (line.trim()) {
      diffLines.push({ kind: 'added', value: line })
    }
  }

  return diffLines.slice(0, 120)
}

export function collectAffectedDomains(currentContent: string, nextContent: string): HostsAffectedDomain[] {
  const currentMap = buildDomainMap(currentContent)
  const nextMap = buildDomainMap(nextContent)
  const domains = new Set<string>([...currentMap.keys(), ...nextMap.keys()])
  const affectedDomains: HostsAffectedDomain[] = []

  for (const domain of domains) {
    const currentIps = currentMap.get(domain) || []
    const nextIps = nextMap.get(domain) || []
    const currentSignature = currentIps.join('|')
    const nextSignature = nextIps.join('|')

    if (!currentSignature && nextSignature) {
      affectedDomains.push({ domain, change: 'added' })
    } else if (currentSignature && !nextSignature) {
      affectedDomains.push({ domain, change: 'removed' })
    } else if (currentSignature !== nextSignature) {
      affectedDomains.push({ domain, change: 'updated' })
    }
  }

  return affectedDomains
    .sort((left, right) => left.domain.localeCompare(right.domain))
    .slice(0, 40)
}

export function detectHostsConflicts(schemes: Scheme[]): HostsConflictGroup[] {
  const entries = schemes.flatMap((scheme) => parseHostsEntries(scheme.content, scheme.name))
  const byDomain = new Map<string, Map<string, Set<string>>>()
  const lastAppliedEntryByDomain = new Map<string, ParsedHostsEntry>()

  for (const entry of entries) {
    if (!byDomain.has(entry.domain)) {
      byDomain.set(entry.domain, new Map())
    }

    const domainMappings = byDomain.get(entry.domain)!
    if (!domainMappings.has(entry.ip)) {
      domainMappings.set(entry.ip, new Set())
    }

    domainMappings.get(entry.ip)!.add(entry.schemeName)
    lastAppliedEntryByDomain.set(entry.domain, entry)
  }

  return Array.from(byDomain.entries())
    .filter(([, mappings]) => mappings.size > 1)
    .map(([domain, mappings]) => ({
      domain,
      effectiveIp: lastAppliedEntryByDomain.get(domain)?.ip || '',
      winningSchemeName: lastAppliedEntryByDomain.get(domain)?.schemeName || '',
      mappings: Array.from(mappings.entries())
        .map(([ip, schemeNames]) => ({
          ip,
          schemeNames: Array.from(schemeNames).sort((left, right) => left.localeCompare(right, 'zh-CN'))
        }))
        .sort((left, right) => left.ip.localeCompare(right.ip))
    }))
    .sort((left, right) => left.domain.localeCompare(right.domain))
}

function buildDomainMap(content: string) {
  const domainMap = new Map<string, string[]>()

  for (const rawLine of content.split('\n')) {
    const contentPart = stripInlineComment(rawLine).trim()
    if (!contentPart) continue

    const tokens = contentPart.split(/\s+/).filter(Boolean)
    if (tokens.length < 2) continue

    const [ip, ...domains] = tokens
    for (const domain of domains) {
      const normalized = domain.toLowerCase()
      if (!domainMap.has(normalized)) {
        domainMap.set(normalized, [])
      }
      domainMap.get(normalized)!.push(ip)
    }
  }

  for (const [domain, ips] of domainMap.entries()) {
    domainMap.set(domain, ips.sort())
  }

  return domainMap
}

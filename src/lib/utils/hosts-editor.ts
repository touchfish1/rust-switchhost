export interface HostsValidationIssue {
  line: number
  message: string
}

export interface HostsContentAnalysis {
  ruleCount: number
  commentCount: number
  issues: HostsValidationIssue[]
}

export function isValidIP(ip: string): boolean {
  const ipv4Regex = /^(\d{1,3}\.){3}\d{1,3}$/
  const ipv6Regex = /^([0-9a-fA-F]{0,4}:){2,7}[0-9a-fA-F]{0,4}$/

  if (ipv4Regex.test(ip)) {
    const parts = ip.split('.')
    return parts.every((part) => {
      const num = parseInt(part, 10)
      return num >= 0 && num <= 255
    })
  }

  return ipv6Regex.test(ip) || ip === '::1'
}

export function isValidHostname(hostname: string): boolean {
  if (!hostname || hostname.length > 253) return false
  if (hostname === 'localhost') return true

  const labels = hostname.split('.')
  return labels.every((label) => {
    if (!label || label.length > 63) return false
    return /^[A-Za-z0-9_](?:[A-Za-z0-9_-]*[A-Za-z0-9_])?$/.test(label)
  })
}

export function analyzeHostsContent(content: string): HostsContentAnalysis {
  const issues: HostsValidationIssue[] = []
  let ruleCount = 0
  let commentCount = 0

  content.split('\n').forEach((line, index) => {
    const trimmed = line.trim()
    if (!trimmed) return

    if (trimmed.startsWith('#')) {
      commentCount += 1
      return
    }

    const commentIndex = line.indexOf('#')
    const contentPart = commentIndex >= 0 ? line.slice(0, commentIndex) : line
    const tokens = contentPart.trim().split(/\s+/).filter(Boolean)

    if (tokens.length === 0) return

    const [ip, ...domains] = tokens

    if (!isValidIP(ip)) {
      issues.push({ line: index + 1, message: `第 ${index + 1} 行 IP 格式无效` })
    }

    if (domains.length === 0) {
      issues.push({ line: index + 1, message: `第 ${index + 1} 行缺少域名` })
      return
    }

    const invalidDomains = domains.filter((domain) => !isValidHostname(domain))
    if (invalidDomains.length > 0) {
      issues.push({
        line: index + 1,
        message: `第 ${index + 1} 行域名格式无效：${invalidDomains.join(', ')}`
      })
    }

    ruleCount += domains.length
  })

  return {
    ruleCount,
    commentCount,
    issues
  }
}

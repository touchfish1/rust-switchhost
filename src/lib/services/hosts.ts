import { invoke } from '@tauri-apps/api/core'
import type { DnsFlushResult, HostsPermissionInfo } from '$lib/types'

export function checkHostsPermission() {
  return invoke<HostsPermissionInfo>('check_hosts_permission')
}

export function getHostsContent() {
  return invoke<string>('get_hosts_content')
}

export function flushDnsCache() {
  return invoke<DnsFlushResult>('flush_dns_cache')
}

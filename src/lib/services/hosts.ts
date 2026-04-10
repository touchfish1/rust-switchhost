import { invoke } from '@tauri-apps/api/core'
import type { DnsFlushResult, HostsBackupEntry, HostsPermissionInfo } from '$lib/types'

export function checkHostsPermission() {
  return invoke<HostsPermissionInfo>('check_hosts_permission')
}

export function getHostsContent() {
  return invoke<string>('get_hosts_content')
}

export function flushDnsCache() {
  return invoke<DnsFlushResult>('flush_dns_cache')
}

export function listHostsBackups() {
  return invoke<HostsBackupEntry[]>('list_hosts_backups')
}

export function getHostsBackupContent(path: string) {
  return invoke<string>('get_hosts_backup_content', { path })
}

export function restoreHostsBackup(path: string) {
  return invoke<string>('restore_hosts_backup', { path })
}

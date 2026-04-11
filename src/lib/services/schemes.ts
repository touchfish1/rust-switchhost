import { invoke } from '@tauri-apps/api/core'
import type { Scheme, SyncLogEntry } from '$lib/types'

export function getAllSchemes() {
  return invoke<Scheme[]>('get_all_schemes')
}

export function createScheme(name: string, content: string) {
  return invoke<Scheme>('create_scheme', { name, content })
}

export function updateScheme(id: string, name: string, content: string) {
  return invoke<Scheme>('update_scheme', { id, name, content })
}

export function deleteScheme(id: string) {
  return invoke<void>('delete_scheme', { id })
}

export function setSchemeEnabled(id: string, enabled: boolean) {
  return invoke<Scheme[]>('set_scheme_enabled', { id, enabled })
}

export function exportSchemes(path: string) {
  return invoke<void>('export_schemes', { path })
}

export function importSchemes(path: string) {
  return invoke<Scheme[]>('import_schemes', { path })
}

export function updateSchemeRemoteConfig(
  id: string,
  remoteUrl: string,
  autoSyncEnabled: boolean,
  syncIntervalMinutes: number | null
) {
  return invoke<Scheme>('update_scheme_remote_config', {
    id,
    remoteUrl,
    autoSyncEnabled,
    syncIntervalMinutes
  })
}

export function syncRemoteScheme(id: string, trigger: 'manual' | 'scheduled' = 'manual') {
  return invoke<Scheme>('sync_remote_scheme', { id, trigger })
}

export function getSchemeSyncLogs(id: string) {
  return invoke<SyncLogEntry[]>('get_scheme_sync_logs', { id })
}

export function fetchRemoteHosts(url: string) {
  return invoke<string>('fetch_remote_hosts', { url })
}

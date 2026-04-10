import type { Update } from '@tauri-apps/plugin-updater'

export interface Scheme {
  id: string
  name: string
  content: string
  remote_url?: string | null
  auto_sync_enabled?: boolean
  sync_interval_minutes?: number | null
  last_synced_at?: string | null
  last_sync_error?: string | null
  sync_status?: string
  last_sync_message?: string | null
  next_retry_at?: string | null
  consecutive_failures?: number
  enabled: boolean
  created_at: string
  updated_at: string
}

export interface UpdateInfo {
  current_version: string
  latest_version: string
  has_update: boolean
  release_name: string
  published_at: string
  body: string
  html_url: string
  download_url: string | null
}

export interface HostsPermissionInfo {
  has_permission: boolean
  hosts_path: string
  platform: string
  message: string
}

export interface DnsFlushResult {
  success: boolean
  platform: string
  message: string
}

export interface SyncLogEntry {
  timestamp: string
  status: string
  trigger: string
  message: string
}

export type UpdaterHandle = Update | null

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
  guidance_title?: string | null
  guidance_steps: string[]
  commands: string[]
}

export interface DnsFlushResult {
  success: boolean
  platform: string
  message: string
}

export interface DnsLookupResult {
  domain: string
  success: boolean
  addresses: string[]
  message: string
}

export interface HostsBackupEntry {
  filename: string
  path: string
  created_at: string
  size_bytes: number
  line_count: number
  host_entry_count: number
  comment_count: number
}

export interface SyncLogEntry {
  timestamp: string
  status: string
  trigger: string
  message: string
}

export interface SchemeTemplate {
  id: string
  name: string
  description: string
  content: string
  source?: 'builtin' | 'custom'
}

export interface HostsConflictMapping {
  ip: string
  schemeNames: string[]
}

export interface HostsConflictGroup {
  domain: string
  mappings: HostsConflictMapping[]
  effectiveIp: string
  winningSchemeName: string
}

export interface HostsContentStats {
  lineCount: number
  hostEntryCount: number
  commentCount: number
}

export interface HostsDiffSummary {
  addedLines: number
  removedLines: number
  unchangedLines: number
}

export interface HostsDiffLine {
  kind: 'added' | 'removed'
  value: string
}

export interface HostsAffectedDomain {
  domain: string
  change: 'added' | 'removed' | 'updated'
  isConflict: boolean
}

export interface RemoteSyncPreviewState {
  diff: HostsDiffSummary
  diffLines: HostsDiffLine[]
  affectedDomains: HostsAffectedDomain[]
}

export type UpdaterHandle = Update | null

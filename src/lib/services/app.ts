import { getVersion } from '@tauri-apps/api/app'
import { invoke } from '@tauri-apps/api/core'

export function getAppVersion() {
  return getVersion()
}

export function restartApp() {
  return invoke<void>('restart_app')
}

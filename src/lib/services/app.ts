import { getVersion } from '@tauri-apps/api/app'
import { invoke } from '@tauri-apps/api/core'

export type TrayMetricsWindowState = {
  x: number
  y: number
}

export function getAppVersion() {
  return getVersion()
}

export function restartApp() {
  return invoke<void>('restart_app')
}

export function getTrayMetricsWindowState() {
  return invoke<TrayMetricsWindowState | null>('get_tray_metrics_window_state')
}

export function setTrayMetricsWindowState(state: TrayMetricsWindowState) {
  return invoke<void>('set_tray_metrics_window_state', { state })
}

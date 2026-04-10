import { invoke } from '@tauri-apps/api/core'
import { check } from '@tauri-apps/plugin-updater'
import type { UpdateInfo, UpdaterHandle } from '$lib/types'

export async function checkForUpdates() {
  const [releaseInfo, updaterUpdate] = await Promise.all([
    invoke<UpdateInfo>('check_for_updates'),
    check()
  ])

  return {
    releaseInfo,
    updaterUpdate: updaterUpdate as UpdaterHandle
  }
}

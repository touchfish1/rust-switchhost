import { derived, writable } from 'svelte/store'
import type { UpdateInfo, UpdaterHandle } from '$lib/types'

interface UpdaterState {
  updateInfo: UpdateInfo | null
  availableUpdate: UpdaterHandle
  isInstallingUpdate: boolean
  updateProgressText: string
  updateProgressValue: number | null
  showUpdateModal: boolean
}

const initialState: UpdaterState = {
  updateInfo: null,
  availableUpdate: null,
  isInstallingUpdate: false,
  updateProgressText: '',
  updateProgressValue: null,
  showUpdateModal: false
}

function createUpdaterStore() {
  const { subscribe, set, update } = writable<UpdaterState>(initialState)

  return {
    subscribe,
    setUpdateResult(updateInfo: UpdateInfo, availableUpdate: UpdaterHandle) {
      update((state) => ({
        ...state,
        updateInfo,
        availableUpdate
      }))
    },
    setShowUpdateModal(showUpdateModal: boolean) {
      update((state) => ({ ...state, showUpdateModal }))
    },
    startInstall(progressText = '正在准备下载更新...') {
      update((state) => ({
        ...state,
        isInstallingUpdate: true,
        updateProgressText: progressText,
        updateProgressValue: 0
      }))
    },
    finishInstall() {
      update((state) => ({
        ...state,
        isInstallingUpdate: false
      }))
    },
    setProgress(updateProgressText: string, updateProgressValue: number | null = null) {
      update((state) => ({
        ...state,
        updateProgressText,
        updateProgressValue: updateProgressValue === null ? state.updateProgressValue : statefulClamp(updateProgressValue)
      }))
    },
    clearProgress() {
      update((state) => ({ ...state, updateProgressText: '', updateProgressValue: null }))
    },
    reset() {
      set(initialState)
    }
  }
}

function statefulClamp(value: number | null) {
  if (value === null) return null
  return Math.max(0, Math.min(100, value))
}

export const updater = createUpdaterStore()
export const hasPendingUpdate = derived(updater, ($updater) =>
  Boolean($updater.updateInfo?.has_update || $updater.availableUpdate)
)

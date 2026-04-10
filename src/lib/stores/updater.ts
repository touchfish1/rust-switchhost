import { derived, writable } from 'svelte/store'
import type { UpdateInfo, UpdaterHandle } from '$lib/types'

interface UpdaterState {
  updateInfo: UpdateInfo | null
  availableUpdate: UpdaterHandle
  isInstallingUpdate: boolean
  updateProgressText: string
  showUpdateModal: boolean
}

const initialState: UpdaterState = {
  updateInfo: null,
  availableUpdate: null,
  isInstallingUpdate: false,
  updateProgressText: '',
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
        updateProgressText: progressText
      }))
    },
    finishInstall() {
      update((state) => ({
        ...state,
        isInstallingUpdate: false
      }))
    },
    setProgress(updateProgressText: string) {
      update((state) => ({ ...state, updateProgressText }))
    },
    clearProgress() {
      update((state) => ({ ...state, updateProgressText: '' }))
    },
    reset() {
      set(initialState)
    }
  }
}

export const updater = createUpdaterStore()
export const hasPendingUpdate = derived(updater, ($updater) =>
  Boolean($updater.updateInfo?.has_update || $updater.availableUpdate)
)

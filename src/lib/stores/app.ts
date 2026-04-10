import { writable } from 'svelte/store'
import type { HostsPermissionInfo } from '$lib/types'

function createFlagStore() {
  const { subscribe, update } = writable<Record<string, boolean>>({})

  return {
    subscribe,
    start(key: string) {
      update((flags) => ({ ...flags, [key]: true }))
    },
    stop(key: string) {
      update((flags) => ({ ...flags, [key]: false }))
    },
    set(key: string, value: boolean) {
      update((flags) => ({ ...flags, [key]: value }))
    }
  }
}

export const appVersion = writable('')
export const appError = writable<string | null>(null)
export const hostsPermissionInfo = writable<HostsPermissionInfo | null>(null)
export const loadingFlags = createFlagStore()

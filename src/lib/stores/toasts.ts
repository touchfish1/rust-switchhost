import { writable } from 'svelte/store'

export type ToastKind = 'success' | 'error' | 'warning' | 'info'

export interface ToastItem {
  id: number
  kind: ToastKind
  message: string
  duration: number
}

function createToastStore() {
  const { subscribe, update } = writable<ToastItem[]>([])
  let nextId = 1

  return {
    subscribe,
    push(message: string, kind: ToastKind = 'info', duration = 2200) {
      const id = nextId++
      update((items) => [...items, { id, kind, message, duration }])
      return id
    },
    remove(id: number) {
      update((items) => items.filter((item) => item.id !== id))
    }
  }
}

export const toasts = createToastStore()

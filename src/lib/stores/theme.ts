import { writable } from 'svelte/store'

const STORAGE_KEY = 'theme'

function resolveInitialTheme() {
  if (typeof window === 'undefined') {
    return false
  }

  const savedTheme = localStorage.getItem(STORAGE_KEY)
  if (savedTheme === 'dark') return true
  if (savedTheme === 'light') return false

  return window.matchMedia('(prefers-color-scheme: dark)').matches
}

function applyTheme(isDark: boolean) {
  if (typeof document === 'undefined') {
    return
  }

  document.documentElement.classList.toggle('dark', isDark)
  localStorage.setItem(STORAGE_KEY, isDark ? 'dark' : 'light')
}

function createThemeStore() {
  const { subscribe, set, update } = writable(false)

  return {
    subscribe,
    initialize() {
      const isDark = resolveInitialTheme()
      applyTheme(isDark)
      set(isDark)
    },
    setTheme(isDark: boolean) {
      applyTheme(isDark)
      set(isDark)
    },
    toggle() {
      update((current) => {
        const next = !current
        applyTheme(next)
        return next
      })
    }
  }
}

export const theme = createThemeStore()

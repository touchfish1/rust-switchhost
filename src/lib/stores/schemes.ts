import { derived, get, writable } from 'svelte/store'
import type { Scheme } from '$lib/types'

export const schemes = writable<Scheme[]>([])
export const activeSchemeId = writable<string | null>(null)
export const editorContent = writable('')

export const activeScheme = derived(
  [schemes, activeSchemeId],
  ([$schemes, $activeSchemeId]) =>
    $activeSchemeId ? $schemes.find((scheme) => scheme.id === $activeSchemeId) || null : null
)

export function setSchemes(nextSchemes: Scheme[], preferredId?: string | null) {
  schemes.set(nextSchemes)

  if (nextSchemes.length === 0) {
    activeSchemeId.set(null)
    editorContent.set('')
    return
  }

  const currentId = preferredId ?? get(activeSchemeId)
  const preservedId = currentId && nextSchemes.some((scheme) => scheme.id === currentId)
    ? currentId
    : nextSchemes[0].id

  activeSchemeId.set(preservedId)
  editorContent.set(nextSchemes.find((scheme) => scheme.id === preservedId)?.content || '')
}

export function selectScheme(id: string) {
  activeSchemeId.set(id)
  editorContent.set(get(schemes).find((scheme) => scheme.id === id)?.content || '')
}

export function setEditorContent(content: string) {
  editorContent.set(content)
}

export function applyUpdatedScheme(updatedScheme: Scheme) {
  schemes.update((items) => items.map((scheme) => (scheme.id === updatedScheme.id ? updatedScheme : scheme)))

  if (get(activeSchemeId) === updatedScheme.id) {
    editorContent.set(updatedScheme.content)
  }
}

export function upsertScheme(nextScheme: Scheme) {
  schemes.update((items) => {
    const exists = items.some((scheme) => scheme.id === nextScheme.id)
    return exists
      ? items.map((scheme) => (scheme.id === nextScheme.id ? nextScheme : scheme))
      : [...items, nextScheme]
  })
  activeSchemeId.set(nextScheme.id)
  editorContent.set(nextScheme.content)
}

export function removeScheme(id: string) {
  const currentSchemes = get(schemes)
  const filtered = currentSchemes.filter((scheme) => scheme.id !== id)
  schemes.set(filtered)

  if (get(activeSchemeId) !== id) {
    return
  }

  const nextActive = filtered[0] || null
  activeSchemeId.set(nextActive?.id || null)
  editorContent.set(nextActive?.content || '')
}

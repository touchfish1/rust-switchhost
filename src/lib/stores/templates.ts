import { derived, writable } from 'svelte/store'
import type { SchemeTemplate } from '$lib/types'
import { builtinSchemeTemplates } from '$lib/data/templates'

const STORAGE_KEY = 'rust-switchhost-custom-templates'

function createTemplateStore() {
  const customTemplates = writable<SchemeTemplate[]>([])

  return {
    subscribe: customTemplates.subscribe,
    initialize() {
      if (typeof localStorage === 'undefined') return

      try {
        const saved = localStorage.getItem(STORAGE_KEY)
        if (!saved) return
        const parsed = JSON.parse(saved) as SchemeTemplate[]
        customTemplates.set(
          parsed.map((template) => ({ ...template, source: 'custom' }))
        )
      } catch (error) {
        console.error('Failed to restore custom templates:', error)
        customTemplates.set([])
      }
    },
    saveTemplate(template: Omit<SchemeTemplate, 'id' | 'source'>) {
      customTemplates.update((items) => {
        const nextName = makeUniqueTemplateName(items, template.name.trim() || '自定义模板')
        const nextItems = [
          {
            id: crypto.randomUUID(),
            source: 'custom' as const,
            name: nextName,
            description: template.description.trim() || '由当前方案保存的自定义模板',
            content: template.content
          },
          ...items
        ]
        persistTemplates(nextItems)
        return nextItems
      })
    },
    deleteTemplate(id: string) {
      customTemplates.update((items) => {
        const nextItems = items.filter((template) => template.id !== id)
        persistTemplates(nextItems)
        return nextItems
      })
    }
  }
}

function persistTemplates(templates: SchemeTemplate[]) {
  if (typeof localStorage === 'undefined') return
  localStorage.setItem(STORAGE_KEY, JSON.stringify(templates))
}

function makeUniqueTemplateName(templates: SchemeTemplate[], baseName: string) {
  if (!templates.some((template) => template.name === baseName)) {
    return baseName
  }

  let index = 2
  while (templates.some((template) => template.name === `${baseName} ${index}`)) {
    index += 1
  }

  return `${baseName} ${index}`
}

export const customTemplates = createTemplateStore()
export const schemeTemplates = derived(customTemplates, ($customTemplates) => [
  ...builtinSchemeTemplates,
  ...$customTemplates
])

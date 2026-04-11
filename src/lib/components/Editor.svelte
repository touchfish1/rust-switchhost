<svelte:options runes={true} />

<script lang="ts">
  import { onMount, onDestroy } from 'svelte'
  import { EditorView, lineNumbers, highlightActiveLine, highlightActiveLineGutter, Decoration, ViewPlugin, ViewUpdate, keymap } from '@codemirror/view'
  import { EditorState, Compartment, RangeSetBuilder, RangeSet, Prec } from '@codemirror/state'
  import { history } from '@codemirror/commands'
  import { isValidHostname, isValidIP } from '$lib/utils/hosts-editor'

  type DecorationSet = RangeSet<Decoration>

  type EditorProps = {
    content?: string
    readOnly?: boolean
    summaryText?: string
    tips?: string[]
    issues?: string[]
    onChange?: (detail: { content: string }) => void
  }

  let {
    content = '',
    readOnly = false,
    summaryText = '',
    tips = [],
    issues = [],
    onChange = () => {}
  }: EditorProps = $props()

  let editorContainer = $state<HTMLDivElement | undefined>(undefined)
  let view = $state<EditorView | null>(null)
  let themeCompartment = new Compartment()
  let readOnlyCompartment = new Compartment()
  let fontSize = $state(14)
  let isDarkMode = $state(false)
  
  const ipDecoration = Decoration.mark({ class: 'cm-ip' })
  const invalidIpDecoration = Decoration.mark({ class: 'cm-ip-invalid' })
  const domainDecoration = Decoration.mark({ class: 'cm-domain' })
  const invalidDomainDecoration = Decoration.mark({ class: 'cm-domain-invalid' })
  const commentDecoration = Decoration.mark({ class: 'cm-comment' })
  
  function hostsHighlighter(view: EditorView): DecorationSet {
    const builder = new RangeSetBuilder<Decoration>()
    const text = view.state.doc.toString()
    const lines = text.split('\n')
    let pos = 0
    
    for (const line of lines) {
      const trimmed = line.trim()
      
      if (trimmed.startsWith('#')) {
        builder.add(pos, pos + line.length, commentDecoration)
      } else if (trimmed) {
        const commentIndex = line.indexOf('#')
        const contentPart = commentIndex >= 0 ? line.slice(0, commentIndex) : line
        const contentTrimmed = contentPart.trim()

        if (commentIndex >= 0) {
          builder.add(pos + commentIndex, pos + line.length, commentDecoration)
        }

        if (contentTrimmed) {
          const tokenRegex = /\S+/g
          const tokens = Array.from(contentPart.matchAll(tokenRegex))

          if (tokens.length > 0) {
            const [ipMatch, ...hostMatches] = tokens
            const ip = ipMatch[0]
            const ipStart = pos + ipMatch.index!
            builder.add(
              ipStart,
              ipStart + ip.length,
              isValidIP(ip) ? ipDecoration : invalidIpDecoration
            )

            for (const match of hostMatches) {
              const hostname = match[0]
              const hostStart = pos + match.index!
              builder.add(
                hostStart,
                hostStart + hostname.length,
                isValidHostname(hostname) ? domainDecoration : invalidDomainDecoration
              )
            }

            if (hostMatches.length === 0) {
              builder.add(ipStart, ipStart + ip.length, invalidIpDecoration)
            }
          }
        }
      }
      
      pos += line.length + 1
    }
    
    return builder.finish()
  }
  
  const highlightPlugin = ViewPlugin.fromClass(class {
    decorations: DecorationSet
    
    constructor(view: EditorView) {
      this.decorations = hostsHighlighter(view)
    }
    
    update(update: ViewUpdate) {
      if (update.docChanged || update.viewportChanged) {
        this.decorations = hostsHighlighter(update.view)
      }
    }
  }, {
    decorations: v => v.decorations
  })
  
  function createLightTheme(size: number) {
    return EditorView.theme({
      '&': {
        backgroundColor: 'var(--editor-bg, #ffffff)',
        color: 'var(--text-primary, #213547)',
        height: '100%'
      },
      '.cm-content': {
        caretColor: 'var(--text-primary, #213547)',
        fontFamily: 'var(--font-family)',
        fontSize: `${size}px`,
        lineHeight: '1.6',
        padding: '0 16px'
      },
      '.cm-cursor': {
        borderLeftColor: 'var(--text-primary, #213547)'
      },
      '.cm-gutters': {
        backgroundColor: 'var(--line-numbers-bg, #fafafa)',
        color: 'var(--text-secondary, #8c8c8c)',
        border: 'none',
        borderRight: '1px solid var(--border-color, #e0e0e0)',
        fontFamily: 'var(--font-family)'
      },
      '.cm-activeLineGutter': {
        backgroundColor: 'var(--hover-bg, #e6f7ff)'
      },
      '.cm-activeLine': {
        backgroundColor: 'var(--hover-bg, #e6f7ff)'
      },
      '.cm-selectionMatch': {
        backgroundColor: '#e6f7ff'
      },
      '.cm-ip': {
        color: 'var(--syntax-ip, #52c41a)',
        fontWeight: '500'
      },
      '.cm-ip-invalid': {
        color: 'var(--syntax-error, #ff4d4f)',
        fontWeight: '500',
        textDecoration: 'underline wavy'
      },
      '.cm-domain': {
        color: 'var(--syntax-domain, #1890ff)',
        fontWeight: '500'
      },
      '.cm-domain-invalid': {
        color: 'var(--syntax-error, #ff4d4f)',
        textDecoration: 'underline wavy'
      },
      '.cm-comment': {
        color: 'var(--syntax-comment, #8c8c8c)',
        fontStyle: 'italic'
      }
    }, { dark: false })
  }
  
  function createDarkTheme(size: number) {
    return EditorView.theme({
      '&': {
        backgroundColor: 'var(--editor-bg, #1e1e1e)',
        color: 'var(--text-primary, #f0f0f0)',
        height: '100%'
      },
      '.cm-content': {
        caretColor: 'var(--text-primary, #f0f0f0)',
        fontFamily: 'var(--font-family)',
        fontSize: `${size}px`,
        lineHeight: '1.6',
        padding: '0 16px'
      },
      '.cm-cursor': {
        borderLeftColor: 'var(--text-primary, #f0f0f0)'
      },
      '.cm-gutters': {
        backgroundColor: 'var(--line-numbers-bg, #252526)',
        color: 'var(--text-secondary, #a0a0a0)',
        border: 'none',
        borderRight: '1px solid var(--border-color, #3a3a3a)',
        fontFamily: 'var(--font-family)'
      },
      '.cm-activeLineGutter': {
        backgroundColor: 'var(--hover-bg, #2a2d2e)'
      },
      '.cm-activeLine': {
        backgroundColor: 'var(--hover-bg, #2a2d2e)'
      },
      '.cm-selectionMatch': {
        backgroundColor: '#264f78'
      },
      '.cm-ip': {
        color: 'var(--syntax-ip, #4ec9b0)',
        fontWeight: '500'
      },
      '.cm-ip-invalid': {
        color: 'var(--syntax-error, #ff4d4f)',
        fontWeight: '500',
        textDecoration: 'underline wavy'
      },
      '.cm-domain': {
        color: 'var(--syntax-domain, #569cd6)',
        fontWeight: '500'
      },
      '.cm-domain-invalid': {
        color: 'var(--syntax-error, #ff6b6b)',
        textDecoration: 'underline wavy'
      },
      '.cm-comment': {
        color: 'var(--syntax-comment, #6a9955)',
        fontStyle: 'italic'
      }
    }, { dark: true })
  }
  
  function getTheme() {
    return isDarkMode ? createDarkTheme(fontSize) : createLightTheme(fontSize)
  }
  
  function updateFontSize(delta: number) {
    const newSize = Math.max(10, Math.min(32, fontSize + delta))
    if (newSize !== fontSize) {
      fontSize = newSize
      localStorage.setItem('editor-font-size', String(fontSize))
      applyTheme()
    }
  }
  
  function applyTheme() {
    if (!view) return
    view.dispatch({
      effects: themeCompartment.reconfigure(getTheme())
    })
  }
  
  function createEditor() {
    if (!editorContainer) return
    
    const savedFontSize = localStorage.getItem('editor-font-size')
    if (savedFontSize) {
      fontSize = parseInt(savedFontSize) || 14
    }
    
    const updateListener = EditorView.updateListener.of((update) => {
      if (update.docChanged) {
        content = update.state.doc.toString()
        onChange({ content })
      }
    })
    
    const fontSizeKeymap = Prec.highest(keymap.of([
      {
        key: 'Mod--',
        run: () => { updateFontSize(-1); return true }
      },
      {
        key: 'Mod-Minus',
        run: () => { updateFontSize(-1); return true }
      },
      {
        key: 'Mod-=',
        run: () => { updateFontSize(1); return true }
      },
      {
        key: 'Mod-Plus',
        run: () => { updateFontSize(1); return true }
      },
      {
        key: 'Mod-0',
        run: () => { 
          fontSize = 14
          localStorage.setItem('editor-font-size', '14')
          applyTheme()
          return true 
        }
      }
    ]))
    
    const state = EditorState.create({
      doc: content,
      extensions: [
        lineNumbers(),
        highlightActiveLine(),
        highlightActiveLineGutter(),
        history(),
        EditorView.lineWrapping,
        highlightPlugin,
        fontSizeKeymap,
        themeCompartment.of(getTheme()),
        readOnlyCompartment.of(EditorState.readOnly.of(readOnly)),
        updateListener,
        EditorView.theme({
          '&': { height: '100%' },
          '.cm-scroller': { overflow: 'auto' }
        })
      ]
    })
    
    view = new EditorView({
      state,
      parent: editorContainer
    })
  }
  
  function updateTheme(dark: boolean) {
    isDarkMode = dark
    applyTheme()
  }
  
  function updateReadOnly(newReadOnly: boolean) {
    if (!view) return
    
    view.dispatch({
      effects: readOnlyCompartment.reconfigure(EditorState.readOnly.of(newReadOnly))
    })
  }
  
  onMount(() => {
    createEditor()
    
    const observer = new MutationObserver(() => {
      const dark = document.documentElement.classList.contains('dark')
      updateTheme(dark)
    })
    
    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ['class']
    })
    
    const dark = document.documentElement.classList.contains('dark')
    updateTheme(dark)
    
    return () => observer.disconnect()
  })
  
  onDestroy(() => {
    if (view) {
      view.destroy()
    }
  })

  $effect(() => {
    if (view && content !== view.state.doc.toString()) {
      view.dispatch({
        changes: {
          from: 0,
          to: view.state.doc.length,
          insert: content
        }
      })
    }
  })

  $effect(() => {
    if (view) {
      updateReadOnly(readOnly)
    }
  })
</script>

<div class="editor-shell">
  {#if !readOnly && (summaryText || tips.length > 0 || issues.length > 0)}
    <div class="editor-guidance">
      {#if summaryText}
        <div class="editor-guidance-summary">{summaryText}</div>
      {/if}
      {#if tips.length > 0}
        <div class="editor-guidance-row">
          {#each tips as tip}
            <span class="guidance-chip">{tip}</span>
          {/each}
        </div>
      {/if}
      {#if issues.length > 0}
        <div class="editor-issues">
          {#each issues as issue}
            <div class="editor-issue">{issue}</div>
          {/each}
        </div>
      {/if}
    </div>
  {/if}

  <div class="editor-container" bind:this={editorContainer}>
    <div class="font-size-hint">
      Ctrl + +/- 调节字号 | 当前: {fontSize}px
    </div>
  </div>
</div>

<style>
  .editor-shell {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }

  .editor-guidance {
    padding: 14px 18px;
    border-bottom: 1px solid var(--border-color, #e0e0e0);
    background: color-mix(in srgb, var(--sidebar-bg, #f5f5f5) 72%, var(--editor-bg, #ffffff) 28%);
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .editor-guidance-summary {
    font-size: 13px;
    color: var(--text-primary, #213547);
    font-weight: 600;
  }

  .editor-guidance-row {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .guidance-chip {
    display: inline-flex;
    align-items: center;
    min-height: 24px;
    padding: 0 10px;
    border-radius: 999px;
    border: 1px solid var(--border-color, #e0e0e0);
    background: var(--editor-bg, #ffffff);
    color: var(--text-secondary, #8c8c8c);
    font-size: 12px;
  }

  .editor-issues {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .editor-issue {
    font-size: 12px;
    color: var(--danger-color, #ff4d4f);
    line-height: 1.5;
  }

  .editor-container {
    flex: 1;
    height: 100%;
    overflow: hidden;
    position: relative;
  }
  
  .font-size-hint {
    position: absolute;
    bottom: 8px;
    right: 16px;
    font-size: 11px;
    color: var(--text-secondary, #8c8c8c);
    opacity: 0.6;
    pointer-events: none;
    z-index: 10;
    font-family: var(--font-family);
  }
</style>

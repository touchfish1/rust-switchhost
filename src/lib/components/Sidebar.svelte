<script lang="ts">
  import { createEventDispatcher, onMount, onDestroy } from 'svelte'
  
  export let schemes: any[] = []
  export let activeSchemeId: string | null = null
  export let width = 320
  
  const dispatch = createEventDispatcher()
  
  let editingId: string | null = null
  let editingName = ''
  let editInput: HTMLInputElement | null = null
  let isResizing = false
  let startX = 0
  let startWidth = width
  
  function selectScheme(id: string) {
    if (editingId === id) return
    dispatch('select', { id })
  }
  
  function createNewScheme() {
    dispatch('create')
  }

  function handleResizeMove(event: MouseEvent) {
    if (!isResizing) return
    const nextWidth = Math.min(520, Math.max(280, startWidth + event.clientX - startX))
    dispatch('resize', { width: nextWidth })
  }

  function stopResize() {
    isResizing = false
    document.body.style.cursor = ''
    document.body.style.userSelect = ''
    window.removeEventListener('mousemove', handleResizeMove)
    window.removeEventListener('mouseup', stopResize)
  }

  function startResize(event: MouseEvent) {
    event.preventDefault()
    event.stopPropagation()
    isResizing = true
    startX = event.clientX
    startWidth = width
    document.body.style.cursor = 'col-resize'
    document.body.style.userSelect = 'none'
    window.addEventListener('mousemove', handleResizeMove)
    window.addEventListener('mouseup', stopResize)
  }

  function importSchemes(event: Event) {
    event.stopPropagation()
    dispatch('import')
  }

  function exportSchemes(event: Event) {
    event.stopPropagation()
    dispatch('export')
  }
  
  function deleteScheme(id: string, event: Event) {
    event.stopPropagation()
    dispatch('delete', { id })
  }
  
  function toggleScheme(id: string, event: Event) {
    event.stopPropagation()
    const target = event.currentTarget as HTMLInputElement
    dispatch('toggle', { id, enabled: target.checked })
  }
  
  function startEdit(id: string, name: string, event: Event) {
    event.stopPropagation()
    editingId = id
    editingName = name
    setTimeout(() => {
      editInput?.focus()
      editInput?.select()
    }, 0)
  }

  function startEditByDoubleClick(id: string, name: string, event: MouseEvent) {
    startEdit(id, name, event)
  }
  
  function saveEdit(id: string) {
    const trimmedName = editingName.trim()
    if (trimmedName) {
      dispatch('rename', { id, name: trimmedName })
    }
    editingId = null
    editingName = ''
  }
  
  function cancelEdit() {
    editingId = null
    editingName = ''
  }
  
  function handleEditKeydown(event: KeyboardEvent) {
    if (event.key === 'Enter') {
      event.preventDefault()
      if (editingId) saveEdit(editingId)
    } else if (event.key === 'Escape') {
      cancelEdit()
    }
  }
  
  function handleKeydown(event: KeyboardEvent) {
    if (event.ctrlKey || event.metaKey) {
      if (event.key === 'n') {
        event.preventDefault()
        createNewScheme()
      }
    }
  }
  
  onMount(() => {
    window.addEventListener('keydown', handleKeydown)
  })
  
  onDestroy(() => {
    window.removeEventListener('keydown', handleKeydown)
    stopResize()
  })
</script>

<div class="sidebar" style={`width: ${width}px;`}>
  <div class="sidebar-header">
    <div class="header-copy">
      <h2>分组列表</h2>
      <span class="header-subtitle">勾选后立即生效，可同时启用多个分组</span>
    </div>
    <div class="header-actions">
      <div class="toolbar-group">
        <button 
          class="btn-new" 
          on:click={createNewScheme} 
          title="新建分组 (Ctrl+N)"
        >
          <svg viewBox="0 0 1024 1024" width="16" height="16" fill="currentColor">
            <path d="M512 64C264.6 64 64 264.6 64 512s200.6 448 448 448 448-200.6 448-448S759.4 64 512 64z m192 472c0 4.4-3.6 8-8 8H544v152c0 4.4-3.6 8-8 8h-48c-4.4 0-8-3.6-8-8V544H328c-4.4 0-8-3.6-8-8v-48c0-4.4 3.6-8 8-8h152V328c0-4.4 3.6-8 8-8h48c4.4 0 8 3.6 8 8v152h152c4.4 0 8 3.6 8 8v48z"/>
          </svg>
        </button>
      </div>

      <div class="toolbar-group toolbar-group-muted">
        <button
          class="btn-toolbar"
          on:click={importSchemes}
          title="导入分组"
          aria-label="导入分组"
        >
          <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d="M12 3v12"/>
            <path d="m7 10 5 5 5-5"/>
            <path d="M5 21h14"/>
          </svg>
        </button>
        <button
          class="btn-toolbar"
          on:click={exportSchemes}
          title="导出分组"
          aria-label="导出分组"
        >
          <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d="M12 21V9"/>
            <path d="m17 14-5-5-5 5"/>
            <path d="M5 3h14"/>
          </svg>
        </button>
      </div>
    </div>
  </div>
  
  <div class="scheme-list">
    {#if schemes.length === 0}
      <div class="empty-state">
        <div class="empty-icon">📋</div>
        <p>暂无分组</p>
        <button class="btn-create" on:click={createNewScheme}>
          创建第一个分组
        </button>
      </div>
    {:else}
      {#each schemes as scheme (scheme.id)}
        <div
          class="scheme-item"
          class:active={scheme.id === activeSchemeId}
          class:editing={editingId === scheme.id}
          on:click={() => selectScheme(scheme.id)}
          on:dblclick={(e) => startEditByDoubleClick(scheme.id, scheme.name, e)}
          on:keydown={(e) => e.key === 'Enter' && selectScheme(scheme.id)}
          role="button"
          tabindex="0"
        >
          <div class="scheme-info">
            {#if editingId === scheme.id}
              <input
                bind:this={editInput}
                type="text"
                class="edit-input"
                bind:value={editingName}
                on:keydown={handleEditKeydown}
                on:blur={() => saveEdit(scheme.id)}
                on:click|stopPropagation
              />
            {:else}
              <span class="scheme-name" title="双击编辑名称">{scheme.name}</span>
              <span class="scheme-date">
                {new Date(scheme.updated_at).toLocaleDateString()}
              </span>
            {/if}
          </div>
          
          <div class="scheme-actions">
            {#if editingId !== scheme.id}
              <label class="switch" title="启用/禁用分组">
                <input 
                  type="checkbox" 
                  checked={scheme.enabled}
                  on:change={(e) => toggleScheme(scheme.id, e)}
                />
                <span class="slider"></span>
              </label>
              
              <button
                class="btn-edit"
                on:click={(e) => startEdit(scheme.id, scheme.name, e)}
                title="编辑名称"
                aria-label="编辑名称"
              >
                <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                  <path d="M12 20h9"/>
                  <path d="M16.5 3.5a2.12 2.12 0 1 1 3 3L7 19l-4 1 1-4Z"/>
                </svg>
              </button>
              
              <button
                class="btn-delete"
                on:click={(e) => deleteScheme(scheme.id, e)}
                title="删除分组"
                aria-label="删除分组"
              >
                <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                  <path d="M3 6h18"/>
                  <path d="M8 6V4h8v2"/>
                  <path d="M19 6l-1 14H6L5 6"/>
                  <path d="M10 11v6"/>
                  <path d="M14 11v6"/>
                </svg>
              </button>
            {/if}
          </div>
        </div>
      {/each}
    {/if}
  </div>
  
  <div class="sidebar-footer">
    <div class="hint">
      <span class="hint-text">Ctrl+N 新建分组 | 双击名称可重命名</span>
      <span class="hint-text">勾选后立即生效，并可同时启用多个分组</span>
    </div>
  </div>

  <button
    class="resize-handle"
    on:mousedown={startResize}
    type="button"
    aria-label="调整分组列表宽度"
  ></button>
</div>

<style>
  .sidebar {
    background: var(--sidebar-bg, #f5f5f5);
    border-right: 1px solid var(--border-color, #e0e0e0);
    display: flex;
    flex-direction: column;
    height: 100%;
    flex-shrink: 0;
    position: relative;
    min-width: 280px;
    max-width: 520px;
  }
  
  .sidebar-header {
    padding: 16px;
    border-bottom: 1px solid var(--border-color, #e0e0e0);
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 16px;
    background: var(--editor-bg, #ffffff);
  }

  .header-copy {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  
  .sidebar-header h2 {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
    color: var(--text-primary, #213547);
  }

  .header-subtitle {
    font-size: 11px;
    color: var(--text-secondary, #8c8c8c);
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 14px;
    margin-left: auto;
    padding-left: 12px;
  }

  .toolbar-group {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .toolbar-group-muted {
    padding-left: 14px;
    border-left: 1px solid var(--border-color, #e0e0e0);
  }
  
  .btn-new {
    margin-left: auto;
    width: 32px;
    height: 32px;
    border-radius: 6px;
    border: none;
    background: var(--primary-color, #1890ff);
    color: white;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s;
    padding: 0;
  }
  
  .btn-new svg {
    width: 18px;
    height: 18px;
  }
  
  .btn-new:hover {
    background: var(--primary-hover, #40a9ff);
    transform: scale(1.05);
  }
  
  .btn-new:active {
    transform: scale(0.95);
  }

  .btn-toolbar {
    width: 32px;
    height: 32px;
    border-radius: 6px;
    border: 1px solid var(--border-color, #e0e0e0);
    background: transparent;
    color: var(--text-primary, #213547);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s;
    padding: 0;
  }

  .btn-toolbar:hover {
    color: var(--primary-color, #1890ff);
    border-color: var(--primary-color, #1890ff);
    background: var(--hover-bg, #e6f7ff);
  }
  
  .scheme-list {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
  }
  
  .empty-state {
    text-align: center;
    padding: 60px 20px;
    color: var(--text-secondary, #8c8c8c);
  }
  
  .empty-icon {
    font-size: 48px;
    margin-bottom: 16px;
    opacity: 0.5;
  }
  
  .empty-state p {
    margin: 0 0 16px 0;
    font-size: 14px;
  }
  
  .btn-create {
    padding: 10px 20px;
    border-radius: 6px;
    border: 1px solid var(--primary-color, #1890ff);
    background: transparent;
    color: var(--primary-color, #1890ff);
    cursor: pointer;
    transition: all 0.2s;
    font-size: 14px;
  }
  
  .btn-create:hover {
    background: var(--primary-color, #1890ff);
    color: white;
    transform: translateY(-1px);
  }
  
  .scheme-item {
    padding: 12px;
    margin-bottom: 4px;
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.2s;
    display: flex;
    justify-content: space-between;
    align-items: center;
    position: relative;
  }
  
  .scheme-item:hover {
    background: var(--hover-bg, #e6f7ff);
    transform: translateX(2px);
  }
  
  .scheme-item:focus {
    outline: 2px solid var(--primary-color, #1890ff);
    outline-offset: -2px;
  }
  
  .scheme-item.active {
    background: var(--primary-color, #1890ff);
    color: white;
    box-shadow: 0 2px 8px rgba(24, 144, 255, 0.3);
  }
  
  .scheme-item.active .scheme-date {
    color: rgba(255, 255, 255, 0.8);
  }
  
  .scheme-item.editing {
    background: var(--hover-bg, #e6f7ff);
    transform: translateX(2px);
  }
  
  .scheme-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 4px;
    overflow: hidden;
  }
  
  .scheme-name {
    font-weight: 500;
    font-size: 14px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    cursor: text;
  }
  
  .scheme-date {
    font-size: 12px;
    color: var(--text-secondary, #8c8c8c);
  }
  
  .edit-input {
    width: 100%;
    padding: 4px 8px;
    border: 1px solid var(--primary-color, #1890ff);
    border-radius: 4px;
    font-size: 14px;
    font-weight: 500;
    background: white;
    color: var(--text-primary, #213547);
    outline: none;
    box-sizing: border-box;
  }
  
  .scheme-actions {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  
  .switch {
    position: relative;
    display: inline-block;
    width: 36px;
    height: 20px;
    flex-shrink: 0;
  }
  
  .switch input {
    opacity: 0;
    width: 0;
    height: 0;
  }
  
  .slider {
    position: absolute;
    cursor: pointer;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: #ccc;
    transition: 0.3s;
    border-radius: 20px;
  }
  
  .slider:before {
    position: absolute;
    content: "";
    height: 14px;
    width: 14px;
    left: 3px;
    bottom: 3px;
    background-color: white;
    transition: 0.3s;
    border-radius: 50%;
  }
  
  input:checked + .slider {
    background-color: var(--success-color, #52c41a);
  }
  
  input:checked + .slider:before {
    transform: translateX(16px);
  }
  
  .scheme-item.active .slider {
    background-color: rgba(255, 255, 255, 0.3);
  }
  
  .scheme-item.active input:checked + .slider {
    background-color: #73d13d;
  }
  
  .btn-edit,
  .btn-delete {
    width: 28px;
    height: 28px;
    border-radius: 4px;
    border: none;
    background: transparent;
    color: var(--btn-icon-color, #666666);
    cursor: pointer;
    opacity: 0.6;
    transition: all 0.2s;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0;
  }
  
  .btn-edit svg,
  .btn-delete svg {
    width: 16px;
    height: 16px;
  }
  
  .scheme-item:hover .btn-edit,
  .scheme-item:hover .btn-delete {
    opacity: 1;
    background: rgba(0, 0, 0, 0.1);
  }
  
  :global(html.dark) .btn-edit,
  :global(html.dark) .btn-delete {
    opacity: 0.7;
  }
  
  :global(html.dark) .scheme-item:hover .btn-edit,
  :global(html.dark) .scheme-item:hover .btn-delete {
    background: rgba(255, 255, 255, 0.15);
  }
  
  .btn-edit:hover {
    background: var(--primary-color, #1890ff) !important;
    color: white !important;
    transform: scale(1.1);
  }
  
  .btn-delete:hover {
    background: var(--danger-color, #ff4d4f) !important;
    color: white !important;
    transform: scale(1.1);
  }
  
  .scheme-item.active .btn-edit,
  .scheme-item.active .btn-delete {
    color: rgba(255, 255, 255, 0.9);
    opacity: 0.7;
  }
  
  .scheme-item.active:hover .btn-edit,
  .scheme-item.active:hover .btn-delete {
    background: rgba(255, 255, 255, 0.2);
    opacity: 1;
  }
  
  .scheme-item.active .btn-edit:hover,
  .scheme-item.active .btn-delete:hover {
    background: rgba(255, 255, 255, 0.35) !important;
    color: white !important;
  }
  
  .sidebar-footer {
    padding: 12px 16px;
    border-top: 1px solid var(--border-color, #e0e0e0);
    background: var(--editor-bg, #ffffff);
  }
  
  .hint-text {
    display: block;
    font-size: 12px;
    color: var(--text-secondary, #8c8c8c);
    opacity: 0.7;
  }

  .resize-handle {
    position: absolute;
    top: 0;
    right: -3px;
    width: 6px;
    height: 100%;
    cursor: col-resize;
    z-index: 2;
    border: none;
    padding: 0;
    background: transparent;
  }

  .resize-handle::after {
    content: '';
    position: absolute;
    top: 0;
    left: 2px;
    width: 2px;
    height: 100%;
    background: transparent;
    transition: background 0.2s ease;
  }

  .resize-handle:hover::after {
    background: var(--primary-color, #1890ff);
  }
</style>

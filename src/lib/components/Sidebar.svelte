<script lang="ts">
  import { createEventDispatcher, onMount, onDestroy } from 'svelte'
  
  export let schemes: any[] = []
  export let activeSchemeId: string | null = null
  
  const dispatch = createEventDispatcher()
  
  let editingId: string | null = null
  let editingName = ''
  let editInput: HTMLInputElement | null = null
  
  function selectScheme(id: string) {
    if (editingId === id) return
    dispatch('select', { id })
  }
  
  function createNewScheme() {
    dispatch('create')
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
  })
</script>

<div class="sidebar">
  <div class="sidebar-header">
    <div class="header-copy">
      <h2>分组列表</h2>
      <span class="header-subtitle">可同时启用多个分组</span>
    </div>
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
              <span class="scheme-name">{scheme.name}</span>
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
                <svg viewBox="0 0 1024 1024" width="14" height="14" fill="currentColor">
                  <path d="M257.7 752c2.2 0 4-0.5 5.9-1.6l152.8-87.7c2.9-1.7 5-4.4 5.7-7.6l21.8-97c0.7-3.2 0-6.5-1.9-9.2l-53.7-74.6c-1.9-2.6-4.9-4.2-8.1-4.5l-99.2-9.4c-3.3-0.3-6.5 0.8-8.9 3l-72.4 62.3c-2.4 2.1-3.8 5.2-3.7 8.4l2.4 105.8c0.1 3.3 1.6 6.3 4.2 8.3l71.7 57.2c2.1 1.7 4.7 2.6 7.4 2.6z m4.6-23.6L204.9 680l-2.1-92.9 63.5-54.7 86.6 8.2 46.9 65.1-19 84.4-118.3 67.9z"/>
                  <path d="M880 836H144c-17.7 0-32 14.3-32 32v0c0 4.4 3.6 8 8 8h784c4.4 0 8-3.6 8-8v0c0-17.7-14.3-32-32-32z"/>
                  <path d="M878.3 254.4l-54.7-54.7c-12.5-12.5-32.8-12.5-45.3 0L383.2 594.9l100 100 395.1-395.1c12.5-12.5 12.5-32.8 0-45.4z m-45.3 5.3L448.2 644.5l-50-50 384.8-384.8 50 50z"/>
                </svg>
              </button>
              
              <button
                class="btn-delete"
                on:click={(e) => deleteScheme(scheme.id, e)}
                title="删除分组"
                aria-label="删除分组"
              >
                <svg viewBox="0 0 1024 1024" width="14" height="14" fill="currentColor">
                  <path d="M360 184h-8c4.4 0 8-3.6 8-8v8h304v-8c0 4.4 3.6 8 8 8h-8v72h72v-80c0-35.3-28.7-64-64-64H352c-35.3 0-64 28.7-64 64v80h72v-72z"/>
                  <path d="M864 256H160c-17.7 0-32 14.3-32 32v32c0 4.4 3.6 8 8 8h60.4l24.7 523c1.6 34.1 29.8 61 63.9 61h454c34.2 0 62.3-26.8 63.9-61l24.7-523H888c4.4 0 8-3.6 8-8v-32c0-17.7-14.3-32-32-32z m-528 432c-17.7 0-32-14.3-32-32V416c0-17.7 14.3-32 32-32s32 14.3 32 32v240c0 17.7-14.3 32-32 32z m176 0c-17.7 0-32-14.3-32-32V416c0-17.7 14.3-32 32-32s32 14.3 32 32v240c0 17.7-14.3 32-32 32z m176 0c-17.7 0-32-14.3-32-32V416c0-17.7 14.3-32 32-32s32 14.3 32 32v240c0 17.7-14.3 32-32 32z"/>
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
      <span class="hint-text">Ctrl+N 新建分组 | 双击编辑名称</span>
      <span class="hint-text">勾选后可与其他分组一起应用</span>
    </div>
  </div>
</div>

<style>
  .sidebar {
    width: 280px;
    background: var(--sidebar-bg, #f5f5f5);
    border-right: 1px solid var(--border-color, #e0e0e0);
    display: flex;
    flex-direction: column;
    height: 100%;
  }
  
  .sidebar-header {
    padding: 16px;
    border-bottom: 1px solid var(--border-color, #e0e0e0);
    display: flex;
    justify-content: space-between;
    align-items: center;
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
  
  .btn-new {
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
</style>

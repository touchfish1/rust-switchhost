<script lang="ts">
  import { onMount } from 'svelte'
  import { invoke } from '@tauri-apps/api/core'
  import Sidebar from './components/Sidebar.svelte'
  import Editor from './components/Editor.svelte'
  import ThemeToggle from './components/ThemeToggle.svelte'
  import Modal from './components/Modal.svelte'
  
  interface Scheme {
    id: string
    name: string
    content: string
    enabled: boolean
    created_at: string
    updated_at: string
  }
  
  let schemes: Scheme[] = []
  let activeSchemeId: string | null = null
  let activeScheme: Scheme | null = null
  let editorContent: string = ''
  let isLoading = false
  let error: string | null = null
  let isDarkMode = false
  
  let showCreateModal = false
  let showDeleteModal = false
  let showCurrentHostsModal = false
  let deleteTargetId: string | null = null
  let newSchemeName = ''
  let currentHostsContent = ''
  
  onMount(async () => {
    const savedTheme = localStorage.getItem('theme')
    isDarkMode = savedTheme === 'dark' || 
      (!savedTheme && window.matchMedia('(prefers-color-scheme: dark)').matches)
    updateTheme()
    
    await loadSchemes()
  })
  
  function updateTheme() {
    if (isDarkMode) {
      document.documentElement.classList.add('dark')
    } else {
      document.documentElement.classList.remove('dark')
    }
    localStorage.setItem('theme', isDarkMode ? 'dark' : 'light')
  }
  
  function handleThemeToggle(event: CustomEvent) {
    isDarkMode = event.detail.isDark
    updateTheme()
  }
  
  async function loadSchemes() {
    try {
      isLoading = true
      error = null
      schemes = await invoke('get_all_schemes')
      if (schemes.length > 0 && !activeSchemeId) {
        activeSchemeId = schemes[0].id
        activeScheme = schemes[0]
        editorContent = activeScheme.content
      }
    } catch (e) {
      error = `加载分组失败: ${e}`
      console.error('Failed to load schemes:', e)
    } finally {
      isLoading = false
    }
  }
  
  async function handleSelectScheme(id: string) {
    activeSchemeId = id
    activeScheme = schemes.find(s => s.id === id) || null
    editorContent = activeScheme?.content || ''
  }

  async function openCurrentHostsModal() {
    try {
      isLoading = true
      error = null
      currentHostsContent = await invoke('get_hosts_content')
      showCurrentHostsModal = true
    } catch (e) {
      error = `读取当前 Hosts 失败: ${e}`
      console.error('Failed to get current hosts content:', e)
    } finally {
      isLoading = false
    }
  }
  
  function openCreateModal() {
    newSchemeName = ''
    showCreateModal = true
  }
  
  async function handleCreateConfirm(event: CustomEvent) {
    const name = event.detail.value?.trim()
    if (!name) return
    
    try {
      isLoading = true
      error = null
      const newScheme = await invoke('create_scheme', {
        name,
        content: '# 新的 hosts 配置\n127.0.0.1 localhost\n'
      })
      
      schemes = [...schemes, newScheme]
      activeSchemeId = newScheme.id
      activeScheme = newScheme
      editorContent = newScheme.content
    } catch (e) {
      error = `创建分组失败: ${e}`
      console.error('Failed to create scheme:', e)
    } finally {
      isLoading = false
    }
  }
  
  function openDeleteModal(id: string) {
    deleteTargetId = id
    showDeleteModal = true
  }
  
  async function handleDeleteConfirm() {
    if (!deleteTargetId) return
    
    try {
      isLoading = true
      error = null
      await invoke('delete_scheme', { id: deleteTargetId })
      
      schemes = schemes.filter(s => s.id !== deleteTargetId)
      if (activeSchemeId === deleteTargetId) {
        activeSchemeId = schemes.length > 0 ? schemes[0].id : null
        activeScheme = activeSchemeId ? schemes[0] : null
        editorContent = activeScheme?.content || ''
      }
    } catch (e) {
      error = `删除分组失败: ${e}`
      console.error('Failed to delete scheme:', e)
    } finally {
      isLoading = false
      deleteTargetId = null
    }
  }
  
  async function handleContentChange(event: CustomEvent) {
    editorContent = event.detail.content
    
    if (activeSchemeId) {
      try {
        const updated = await invoke('update_scheme', {
          id: activeSchemeId,
          name: activeScheme?.name || '未命名',
          content: editorContent
        })
        
        schemes = schemes.map(s => s.id === updated.id ? updated : s)
        activeScheme = updated
      } catch (e) {
        console.error('Failed to update scheme:', e)
      }
    }
  }
  
  async function handleRename(event: CustomEvent) {
    const { id, name } = event.detail
    try {
      isLoading = true
      error = null
      const updated = await invoke('update_scheme', {
        id,
        name,
        content: schemes.find(s => s.id === id)?.content || ''
      })
      
      schemes = schemes.map(s => s.id === updated.id ? updated : s)
      if (activeSchemeId === id) {
        activeScheme = updated
      }
    } catch (e) {
      error = `重命名失败: ${e}`
      console.error('Failed to rename scheme:', e)
    } finally {
      isLoading = false
    }
  }

  async function handleToggleScheme(event: CustomEvent) {
    const { id, enabled } = event.detail

    try {
      isLoading = true
      error = null
      schemes = await invoke('set_scheme_enabled', { id, enabled })

      activeSchemeId = id

      activeScheme = activeSchemeId
        ? schemes.find((scheme) => scheme.id === activeSchemeId) || null
        : null

      if (activeScheme) {
        editorContent = activeScheme.content
      }

      showSuccessToast(enabled ? '分组已启用并生效' : '分组已停用并生效')
    } catch (e) {
      error = `${enabled ? '启用' : '禁用'}分组失败: ${e}`
      console.error('Failed to toggle scheme:', e)
    } finally {
      isLoading = false
    }
  }
  
  function showSuccessToast(message: string) {
    const toast = document.createElement('div')
    toast.className = 'toast success'
    toast.textContent = message
    document.body.appendChild(toast)
    
    setTimeout(() => {
      toast.classList.add('show')
    }, 10)
    
    setTimeout(() => {
      toast.classList.remove('show')
      setTimeout(() => toast.remove(), 300)
    }, 2000)
  }
</script>

<div class="app" class:dark={isDarkMode}>
  <div class="header">
    <h1>🔧 Rust SwitchHost</h1>
    <div class="header-actions">
      <button class="btn-secondary" on:click={openCurrentHostsModal} disabled={isLoading}>
        查看当前 Hosts
      </button>
      <ThemeToggle isDark={isDarkMode} on:toggle={handleThemeToggle} />
    </div>
  </div>
  
  {#if error}
    <div class="error-banner">
      {error}
      <button on:click={() => error = null}>×</button>
    </div>
  {/if}
  
  <div class="main">
    <Sidebar
      {schemes}
      {activeSchemeId}
      on:select={(e) => handleSelectScheme(e.detail.id)}
      on:create={openCreateModal}
      on:delete={(e) => openDeleteModal(e.detail.id)}
      on:rename={handleRename}
      on:toggle={handleToggleScheme}
    />
    
    <div class="content">
      {#if activeScheme}
        <div class="editor-header">
          <h2>{activeScheme.name}</h2>
          <span class="scheme-meta">
            分组内容编辑中 | 创建于 {new Date(activeScheme.created_at).toLocaleString()}
          </span>
        </div>
        
        <Editor
          content={editorContent}
          on:change={handleContentChange}
        />
      {:else}
        <div class="empty-state">
          <h2>欢迎使用 Rust SwitchHost</h2>
          <p>请从左侧选择一个分组，或创建新分组开始使用</p>
          <button class="btn-primary" on:click={openCreateModal}>
            创建第一个分组
          </button>
        </div>
      {/if}
    </div>
  </div>
  
  {#if isLoading}
    <div class="loading-overlay">
      <div class="spinner"></div>
    </div>
  {/if}
  
  {#if showCreateModal}
    <Modal
      title="创建新分组"
      confirmText="创建"
      inputValue={newSchemeName}
      on:confirm={handleCreateConfirm}
      on:cancel={() => showCreateModal = false}
      on:close={() => showCreateModal = false}
    />
  {/if}
  
  {#if showDeleteModal}
    <Modal
      title="删除分组"
      confirmText="删除"
      cancelText="取消"
      type="danger"
      on:confirm={handleDeleteConfirm}
      on:cancel={() => { showDeleteModal = false; deleteTargetId = null; }}
      on:close={() => { showDeleteModal = false; deleteTargetId = null; }}
    >
      <p class="confirm-text">确定要删除分组「{schemes.find(s => s.id === deleteTargetId)?.name || ''}」吗？</p>
      <p class="confirm-warning">此操作不可撤销。</p>
    </Modal>
  {/if}

  {#if showCurrentHostsModal}
    <div
      class="hosts-modal-overlay"
      on:click|self={() => showCurrentHostsModal = false}
      on:keydown={(e) => e.key === 'Escape' && (showCurrentHostsModal = false)}
      role="dialog"
      aria-modal="true"
      aria-label="当前 Hosts 文件"
      tabindex="0"
    >
      <div class="hosts-modal" role="document">
        <div class="hosts-modal-header">
          <div>
            <h3>当前 Hosts 文件</h3>
            <p>这里显示的是系统当前实际 hosts 内容</p>
          </div>
          <button
            class="hosts-close-btn"
            on:click={() => showCurrentHostsModal = false}
            aria-label="关闭"
          >
            ×
          </button>
        </div>

        <div class="hosts-modal-body">
          <Editor content={currentHostsContent} readOnly={true} />
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  :global(:root) {
    --primary-color: #1890ff;
    --primary-hover: #40a9ff;
    --danger-color: #ff4d4f;
    --text-primary: #213547;
    --text-secondary: #8c8c8c;
    --border-color: #e0e0e0;
    --sidebar-bg: #f5f5f5;
    --editor-bg: #ffffff;
    --line-numbers-bg: #fafafa;
    --hover-bg: #e6f7ff;
    --btn-icon-color: #666666;
    
    --syntax-ip: #52c41a;
    --syntax-domain: #1890ff;
    --syntax-comment: #8c8c8c;
    --syntax-error: #ff4d4f;
  }
  
  :global(:root.dark) {
    --text-primary: #f0f0f0;
    --text-secondary: #a0a0a0;
    --border-color: #3a3a3a;
    --sidebar-bg: #252526;
    --editor-bg: #1e1e1e;
    --line-numbers-bg: #252526;
    --hover-bg: #2a2d2e;
    --btn-icon-color: #d0d0d0;
    
    --syntax-ip: #4ec9b0;
    --syntax-domain: #569cd6;
    --syntax-comment: #6a9955;
  }
  
  :global(.toast) {
    position: fixed;
    top: 80px;
    left: 50%;
    transform: translateX(-50%) translateY(-20px);
    padding: 12px 24px;
    border-radius: 8px;
    font-size: 14px;
    font-weight: 500;
    z-index: 3000;
    opacity: 0;
    transition: all 0.3s ease;
    font-family: 'Microsoft YaHei', 'PingFang SC', sans-serif;
  }
  
  :global(.toast.success) {
    background: #f6ffed;
    color: #52c41a;
    border: 1px solid #b7eb8f;
    box-shadow: 0 4px 12px rgba(82, 196, 26, 0.2);
  }
  
  :global(.toast.show) {
    opacity: 1;
    transform: translateX(-50%) translateY(0);
  }
  
  :global(.dark .toast.success) {
    background: #162312;
    color: #95de64;
    border-color: #3d5c2e;
  }
  
  .app {
    height: 100vh;
    display: flex;
    flex-direction: column;
    background: var(--editor-bg);
    color: var(--text-primary);
    transition: background-color 0.3s, color 0.3s;
    font-family: 'Microsoft YaHei', 'PingFang SC', sans-serif;
  }
  
  .header {
    height: 60px;
    padding: 0 24px;
    border-bottom: 1px solid var(--border-color);
    display: flex;
    align-items: center;
    justify-content: space-between;
    background: var(--editor-bg);
  }
  
  .header h1 {
    margin: 0;
    font-size: 20px;
    font-weight: 600;
    color: var(--text-primary);
  }
  
  .header-actions {
    display: flex;
    gap: 12px;
    align-items: center;
  }
  
  .btn-primary {
    padding: 8px 16px;
    border-radius: 6px;
    border: none;
    background: var(--primary-color);
    color: white;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: background-color 0.2s;
  }
  
  .btn-primary:hover:not(:disabled) {
    background: var(--primary-hover);
  }
  
  .btn-primary:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .btn-secondary {
    padding: 8px 16px;
    border-radius: 6px;
    border: 1px solid var(--border-color);
    background: transparent;
    color: var(--text-primary);
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
  }

  .btn-secondary:hover:not(:disabled) {
    border-color: var(--primary-color);
    color: var(--primary-color);
    background: var(--hover-bg);
  }

  .btn-secondary:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
  
  .error-banner {
    padding: 12px 24px;
    background: #fff2f0;
    border-bottom: 1px solid #ffccc7;
    color: var(--danger-color);
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  
  .dark .error-banner {
    background: #2a1f1f;
    border-bottom-color: #5a3030;
  }
  
  .error-banner button {
    background: none;
    border: none;
    font-size: 20px;
    cursor: pointer;
    color: var(--danger-color);
  }
  
  .main {
    flex: 1;
    display: flex;
    overflow: hidden;
  }
  
  .content {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  
  .editor-header {
    padding: 16px 24px;
    border-bottom: 1px solid var(--border-color);
    display: flex;
    justify-content: space-between;
    align-items: center;
    background: var(--editor-bg);
  }
  
  .editor-header h2 {
    margin: 0;
    font-size: 18px;
    font-weight: 600;
    color: var(--text-primary);
  }
  
  .scheme-meta {
    font-size: 13px;
    color: var(--text-secondary);
  }
  
  .empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    color: var(--text-secondary);
  }
  
  .empty-state h2 {
    margin: 0 0 12px 0;
    font-size: 24px;
    color: var(--text-primary);
  }
  
  .empty-state p {
    margin: 0 0 24px 0;
    font-size: 16px;
  }
  
  .loading-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(255, 255, 255, 0.8);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }
  
  .dark .loading-overlay {
    background: rgba(30, 30, 30, 0.8);
  }
  
  .spinner {
    width: 40px;
    height: 40px;
    border: 3px solid var(--border-color);
    border-top-color: var(--primary-color);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }
  
  @keyframes spin {
    to { transform: rotate(360deg); }
  }
  
  .confirm-text {
    margin: 0 0 8px 0;
    font-size: 15px;
    color: var(--text-primary, #213547);
  }
  
  .confirm-warning {
    margin: 0;
    font-size: 13px;
    color: var(--danger-color, #ff4d4f);
  }

  .hosts-modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.45);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 2100;
    padding: 24px;
  }

  .hosts-modal {
    width: min(1100px, 100%);
    height: min(760px, calc(100vh - 48px));
    background: var(--editor-bg);
    border-radius: 12px;
    box-shadow: 0 18px 60px rgba(0, 0, 0, 0.25);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .hosts-modal-header {
    padding: 16px 20px;
    border-bottom: 1px solid var(--border-color);
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 16px;
  }

  .hosts-modal-header h3 {
    margin: 0 0 4px 0;
    font-size: 18px;
    color: var(--text-primary);
  }

  .hosts-modal-header p {
    margin: 0;
    font-size: 13px;
    color: var(--text-secondary);
  }

  .hosts-close-btn {
    width: 36px;
    height: 36px;
    border: none;
    border-radius: 8px;
    background: transparent;
    color: var(--text-secondary);
    font-size: 24px;
    line-height: 1;
    cursor: pointer;
    transition: all 0.2s;
  }

  .hosts-close-btn:hover {
    background: var(--hover-bg);
    color: var(--text-primary);
  }

  .hosts-modal-body {
    flex: 1;
    min-height: 0;
  }
</style>

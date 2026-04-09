<script lang="ts">
  import { onDestroy, onMount } from 'svelte'
  import { getVersion } from '@tauri-apps/api/app'
  import { invoke } from '@tauri-apps/api/core'
  import { open as openDialog, save } from '@tauri-apps/plugin-dialog'
  import { open } from '@tauri-apps/plugin-shell'
  import { check, type DownloadEvent, type Update } from '@tauri-apps/plugin-updater'
  import Sidebar from './components/Sidebar.svelte'
  import Editor from './components/Editor.svelte'
  import ThemeToggle from './components/ThemeToggle.svelte'
  import Modal from './components/Modal.svelte'
  import CreateSchemeModal from './components/CreateSchemeModal.svelte'
  
  interface Scheme {
    id: string
    name: string
    content: string
    remote_url?: string | null
    auto_sync_enabled?: boolean
    sync_interval_minutes?: number | null
    last_synced_at?: string | null
    last_sync_error?: string | null
    enabled: boolean
    created_at: string
    updated_at: string
  }

  interface UpdateInfo {
    current_version: string
    latest_version: string
    has_update: boolean
    release_name: string
    published_at: string
    body: string
    html_url: string
    download_url: string | null
  }

  interface HostsPermissionInfo {
    has_permission: boolean
    hosts_path: string
    platform: string
    message: string
  }
  
  let schemes: Scheme[] = []
  let activeSchemeId: string | null = null
  let activeScheme: Scheme | null = null
  let editorContent: string = ''
  let isLoading = false
  let error: string | null = null
  let isDarkMode = false
  let appVersion = ''
  let hostsPermissionInfo: HostsPermissionInfo | null = null
  
  let showCreateModal = false
  let showDeleteModal = false
  let showCurrentHostsModal = false
  let showUpdateModal = false
  let deleteTargetId: string | null = null
  let currentHostsContent = ''
  let updateInfo: UpdateInfo | null = null
  let availableUpdate: Update | null = null
  let isInstallingUpdate = false
  let updateProgressText = ''
  let hasPendingUpdate = false
  let updateCheckTimer: ReturnType<typeof setInterval> | null = null
  let remoteSyncTimer: ReturnType<typeof setInterval> | null = null
  let isSyncingRemoteScheme = false
  let isCreatingScheme = false
  let sidebarWidth = 320
  const syncingSchemeIds = new Set<string>()
  
  onMount(async () => {
    const savedTheme = localStorage.getItem('theme')
    const savedSidebarWidth = localStorage.getItem('sidebar-width')
    isDarkMode = savedTheme === 'dark' || 
      (!savedTheme && window.matchMedia('(prefers-color-scheme: dark)').matches)
    sidebarWidth = savedSidebarWidth ? Math.min(520, Math.max(280, Number(savedSidebarWidth) || 320)) : 320
    updateTheme()

    await Promise.all([
      loadAppVersion(),
      loadSchemes(),
      checkHostsPermission()
    ])

    await checkForUpdatesSilently()
    updateCheckTimer = setInterval(() => {
      void checkForUpdatesSilently()
    }, 15 * 60 * 1000)

    remoteSyncTimer = setInterval(() => {
      void runScheduledRemoteSync()
    }, 60 * 1000)
  })

  onDestroy(() => {
    if (updateCheckTimer) {
      clearInterval(updateCheckTimer)
      updateCheckTimer = null
    }

    if (remoteSyncTimer) {
      clearInterval(remoteSyncTimer)
      remoteSyncTimer = null
    }
  })

  async function loadAppVersion() {
    try {
      appVersion = await getVersion()
    } catch (e) {
      console.error('Failed to load app version:', e)
      appVersion = ''
    }
  }
  
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

  function applyUpdatedScheme(updatedScheme: Scheme) {
    schemes = schemes.map((scheme) => (scheme.id === updatedScheme.id ? updatedScheme : scheme))

    if (activeSchemeId === updatedScheme.id) {
      activeScheme = updatedScheme
      editorContent = updatedScheme.content
    }
  }

  async function checkHostsPermission() {
    try {
      hostsPermissionInfo = await invoke<HostsPermissionInfo>('check_hosts_permission')
    } catch (e) {
      console.error('Failed to check hosts permission:', e)
      hostsPermissionInfo = null
    }
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

  async function performUpdateCheck(options: { silent: boolean }) {
    const { silent } = options

    try {
      if (!silent) {
        isLoading = true
        error = null
      }

      const [releaseInfo, updaterUpdate] = await Promise.all([
        invoke<UpdateInfo>('check_for_updates'),
        check()
      ])

      updateInfo = releaseInfo
      availableUpdate = updaterUpdate
      hasPendingUpdate = !!(releaseInfo.has_update || updaterUpdate)

      if (!silent) {
        showUpdateModal = true
      }

      if (!silent && updateInfo && !updateInfo.has_update) {
        showSuccessToast(`当前已是最新版本 ${updateInfo.current_version}`)
      }
    } catch (e) {
      if (!silent) {
        error = `检查更新失败: ${e}`
      }
      console.error(`Failed to ${silent ? 'silently c' : 'c'}heck for updates:`, e)
    } finally {
      if (!silent) {
        isLoading = false
      }
    }
  }

  async function checkForUpdatesSilently() {
    await performUpdateCheck({ silent: true })
  }

  async function handleCheckUpdates() {
    await performUpdateCheck({ silent: false })
  }

  async function handleInstallUpdate() {
    if (!availableUpdate) {
      if (updateInfo?.download_url) {
        await openUpdateUrl(updateInfo.download_url)
      } else if (updateInfo?.html_url) {
        await openUpdateUrl(updateInfo.html_url)
      }
      return
    }

    try {
      isInstallingUpdate = true
      error = null
      updateProgressText = '正在准备下载更新...'

      await availableUpdate.downloadAndInstall((event: DownloadEvent) => {
        if (event.event === 'Started') {
          const total = event.data.contentLength
          updateProgressText = total
            ? `开始下载更新，总大小 ${(total / 1024 / 1024).toFixed(2)} MB`
            : '开始下载更新'
        } else if (event.event === 'Progress') {
          updateProgressText = `正在下载更新，已接收 ${(event.data.chunkLength / 1024).toFixed(1)} KB 数据块`
        } else if (event.event === 'Finished') {
          updateProgressText = '下载完成，正在安装更新...'
        }
      })

      updateProgressText = '安装完成，应用即将重启...'
      hasPendingUpdate = false
      await invoke('restart_app')
    } catch (e) {
      error = `安装更新失败: ${e}`
      console.error('Failed to install update:', e)
      updateProgressText = ''
    } finally {
      isInstallingUpdate = false
    }
  }

  async function openUpdateUrl(url: string) {
    try {
      await open(url)
    } catch (e) {
      console.error('Failed to open external url:', e)
      window.open(url, '_blank', 'noopener,noreferrer')
    }
  }

  function formatPublishedAt(value: string) {
    if (!value) return '未知'

    try {
      return new Date(value).toLocaleString()
    } catch {
      return value
    }
  }
  
  function openCreateModal() {
    showCreateModal = true
  }
  
  async function handleCreateConfirm(event: CustomEvent) {
    const name = event.detail.name?.trim()
    const type = event.detail.type as 'local' | 'remote'
    const remoteUrl = event.detail.remoteUrl?.trim() || ''
    const autoSyncEnabled = Boolean(event.detail.autoSyncEnabled)
    const syncIntervalInput = event.detail.syncIntervalMinutes as string
    const syncIntervalMinutes = syncIntervalInput ? Number(syncIntervalInput) : null

    if (!name) return
    if (type === 'remote' && !remoteUrl) {
      error = '远程 URL 分组必须填写远程地址'
      return
    }
    if (type === 'remote' && autoSyncEnabled && (!syncIntervalMinutes || syncIntervalMinutes <= 0)) {
      error = '启用定时同步时，请填写大于 0 的同步间隔'
      return
    }
    
    try {
      isCreatingScheme = true
      isLoading = true
      error = null
      let newScheme = await invoke<Scheme>('create_scheme', {
        name,
        content: type === 'remote'
          ? '# 远程 URL 分组\n# 首次同步后会自动填充内容\n'
          : '# 新的 hosts 配置\n127.0.0.1 localhost\n'
      })

      if (type === 'remote') {
        newScheme = await invoke<Scheme>('update_scheme_remote_config', {
          id: newScheme.id,
          remoteUrl,
          autoSyncEnabled,
          syncIntervalMinutes
        })
      }

      if (schemes.some((scheme) => scheme.id === newScheme.id)) {
        schemes = schemes.map((scheme) => (scheme.id === newScheme.id ? newScheme : scheme))
      } else {
        schemes = [...schemes, newScheme]
      }

      activeSchemeId = newScheme.id
      activeScheme = newScheme
      editorContent = newScheme.content
      showCreateModal = false

      if (type === 'remote') {
        try {
          const syncedScheme = await invoke<Scheme>('sync_remote_scheme', { id: newScheme.id })
          applyUpdatedScheme(syncedScheme)
          showSuccessToast('远程 URL 分组已创建并完成首次同步')
        } catch (syncError) {
          error = `远程分组已创建，但首次同步失败: ${syncError}`
          console.error('Failed to sync new remote scheme:', syncError)
        }
      }
    } catch (e) {
      error = `创建分组失败: ${e}`
      console.error('Failed to create scheme:', e)
    } finally {
      isCreatingScheme = false
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

  async function syncSchemeById(id: string, silent = false) {
    if (syncingSchemeIds.has(id)) return

    try {
      syncingSchemeIds.add(id)
      if (!silent) {
        isSyncingRemoteScheme = true
        isLoading = true
        error = null
      }

      const updated = await invoke<Scheme>('sync_remote_scheme', { id })
      applyUpdatedScheme(updated)

      if (!silent) {
        showSuccessToast('远程分组同步成功')
      }
    } catch (e) {
      if (!silent) {
        error = `同步远程分组失败: ${e}`
      }
      console.error('Failed to sync remote scheme:', e)
      await loadSchemes()
    } finally {
      syncingSchemeIds.delete(id)
      if (!silent) {
        isSyncingRemoteScheme = false
        isLoading = false
      }
    }
  }

  async function handleSyncActiveScheme() {
    if (!activeSchemeId) return
    await syncSchemeById(activeSchemeId, false)
  }

  async function runScheduledRemoteSync() {
    const now = Date.now()

    for (const scheme of schemes) {
      const remoteUrl = scheme.remote_url?.trim()
      const interval = scheme.sync_interval_minutes
      if (!scheme.auto_sync_enabled || !remoteUrl || !interval || interval <= 0) {
        continue
      }

      const lastSyncedAt = scheme.last_synced_at ? new Date(scheme.last_synced_at).getTime() : 0
      const due = lastSyncedAt === 0 || now - lastSyncedAt >= interval * 60 * 1000

      if (due) {
        await syncSchemeById(scheme.id, true)
      }
    }
  }

  async function handleExportSchemes() {
    try {
      const exportPath = await save({
        title: '导出分组',
        defaultPath: `rust-switchhost-schemes-${appVersion || 'backup'}.json`,
        filters: [
          {
            name: 'JSON',
            extensions: ['json']
          }
        ]
      })

      if (!exportPath) return

      isLoading = true
      error = null
      await invoke('export_schemes', { path: exportPath })
      showSuccessToast('分组已导出')
    } catch (e) {
      error = `导出分组失败: ${e}`
      console.error('Failed to export schemes:', e)
    } finally {
      isLoading = false
    }
  }

  async function handleImportSchemes() {
    try {
      const importPath = await openDialog({
        title: '导入分组',
        multiple: false,
        directory: false,
        filters: [
          {
            name: 'JSON',
            extensions: ['json']
          }
        ]
      })

      if (!importPath || Array.isArray(importPath)) return

      isLoading = true
      error = null
      schemes = await invoke('import_schemes', { path: importPath })
      activeScheme = activeSchemeId
        ? schemes.find((scheme) => scheme.id === activeSchemeId) || schemes[0] || null
        : schemes[0] || null
      activeSchemeId = activeScheme?.id || null
      editorContent = activeScheme?.content || ''
      showSuccessToast('分组已导入，导入项默认未启用')
    } catch (e) {
      error = `导入分组失败: ${e}`
      console.error('Failed to import schemes:', e)
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

  function handleSidebarResize(event: CustomEvent) {
    sidebarWidth = event.detail.width
    localStorage.setItem('sidebar-width', String(sidebarWidth))
  }
</script>

<div class="app" class:dark={isDarkMode}>
  <div class="header">
    <div class="header-brand">
      <h1>🔧 Rust SwitchHost</h1>
      {#if appVersion}
        <span class="app-version">v{appVersion}</span>
      {/if}
    </div>
    <div class="header-actions">
      <button
        class="btn-secondary update-trigger"
        class:has-notification={hasPendingUpdate}
        on:click={handleCheckUpdates}
        disabled={isLoading}
      >
        检查更新
      </button>
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

  {#if hostsPermissionInfo && !hostsPermissionInfo.has_permission}
    <div class="permission-banner">
      <div>
        <strong>Hosts 权限不足</strong>
        <span>{hostsPermissionInfo.message}</span>
      </div>
    </div>
  {/if}
  
  <div class="main">
    <Sidebar
      {schemes}
      {activeSchemeId}
      width={sidebarWidth}
      on:select={(e) => handleSelectScheme(e.detail.id)}
      on:create={openCreateModal}
      on:import={handleImportSchemes}
      on:export={handleExportSchemes}
      on:delete={(e) => openDeleteModal(e.detail.id)}
      on:rename={handleRename}
      on:toggle={handleToggleScheme}
      on:resize={handleSidebarResize}
    />
    
    <div class="content">
      {#if activeScheme}
        <div class="editor-header">
          <div class="editor-title">
            <h2>{activeScheme.name}</h2>
            <span class="scheme-meta">
              分组内容编辑中 | 创建于 {new Date(activeScheme.created_at).toLocaleString()}
            </span>
            {#if activeScheme.remote_url}
              <span class="remote-meta">
                远程源已配置
                {#if activeScheme.auto_sync_enabled && activeScheme.sync_interval_minutes}
                  · 每 {activeScheme.sync_interval_minutes} 分钟同步
                {/if}
              </span>
            {/if}
          </div>
          <div class="editor-actions">
            {#if activeScheme.remote_url}
              <button class="btn-secondary" on:click={handleSyncActiveScheme} disabled={isSyncingRemoteScheme || isLoading}>
                {isSyncingRemoteScheme ? '同步中...' : '立即同步'}
              </button>
            {/if}
          </div>
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
    <CreateSchemeModal
      isOpen={showCreateModal}
      isSubmitting={isCreatingScheme}
      on:confirm={handleCreateConfirm}
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

  {#if showUpdateModal && updateInfo}
    <div
      class="hosts-modal-overlay"
      on:click|self={() => showUpdateModal = false}
      on:keydown={(e) => e.key === 'Escape' && (showUpdateModal = false)}
      role="dialog"
      aria-modal="true"
      aria-label="检查更新"
      tabindex="0"
    >
      <div class="update-modal" role="document">
        <div class="hosts-modal-header">
          <div>
            <h3>在线升级</h3>
            <p>
              当前版本 {updateInfo.current_version} · 最新版本 {updateInfo.latest_version}
            </p>
          </div>
          <button
            class="hosts-close-btn"
            on:click={() => showUpdateModal = false}
            aria-label="关闭"
          >
            ×
          </button>
        </div>

        <div class="update-modal-body">
          <div class:status-card={true} class:update-available={updateInfo.has_update}>
            <strong>{updateInfo.has_update ? '发现新版本' : '当前已是最新版本'}</strong>
            <span>发布时间：{formatPublishedAt(updateInfo.published_at)}</span>
          </div>

          <div class="update-meta">
            <div class="update-meta-row">
              <span>版本标题</span>
              <strong>{updateInfo.release_name}</strong>
            </div>
            <div class="update-meta-row">
              <span>一键升级</span>
              <strong>{availableUpdate ? '可直接下载安装' : '当前仅可跳转下载'}</strong>
            </div>
            <div class="update-meta-row">
              <span>发布页</span>
              <button class="link-btn" on:click={() => openUpdateUrl(updateInfo.html_url)}>
                打开 GitHub Release
              </button>
            </div>
            {#if updateInfo.download_url}
              <div class="update-meta-row">
                <span>推荐下载</span>
                <button class="link-btn" on:click={() => openUpdateUrl(updateInfo.download_url!)}>
                  打开当前系统下载链接
                </button>
              </div>
            {/if}
          </div>

          <div class="release-notes">
            <h4>发布说明</h4>
            <pre>{updateInfo.body || '暂无发布说明'}</pre>
          </div>

          {#if updateInfo.has_update}
            <div class="update-actions">
              <button
                class="btn-secondary"
                on:click={() => openUpdateUrl(updateInfo.html_url)}
                disabled={isInstallingUpdate}
              >
                查看发布页
              </button>
              <button
                class="btn-primary"
                on:click={handleInstallUpdate}
                disabled={isInstallingUpdate}
              >
                {isInstallingUpdate
                  ? '升级中...'
                  : availableUpdate
                    ? '一键下载安装并重启'
                    : '打开下载链接'}
              </button>
            </div>
          {/if}

          {#if updateProgressText}
            <div class="update-progress">
              {updateProgressText}
            </div>
          {/if}
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

  .header-brand {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 0;
  }

  .app-version {
    display: inline-flex;
    align-items: center;
    height: 26px;
    padding: 0 10px;
    border-radius: 999px;
    background: var(--hover-bg);
    border: 1px solid var(--border-color);
    color: var(--text-secondary);
    font-size: 12px;
    font-weight: 600;
    line-height: 1;
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

  .update-trigger {
    position: relative;
  }

  .update-trigger.has-notification::after {
    content: '';
    position: absolute;
    top: 6px;
    right: 6px;
    width: 8px;
    height: 8px;
    border-radius: 999px;
    background: #ff4d4f;
    box-shadow: 0 0 0 2px var(--editor-bg);
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

  .permission-banner {
    padding: 12px 24px;
    background: #fffbe6;
    border-bottom: 1px solid #ffe58f;
    color: #ad6800;
    display: flex;
    align-items: center;
  }

  .permission-banner div {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .permission-banner strong {
    font-size: 14px;
  }

  .permission-banner span {
    font-size: 13px;
  }

  .dark .permission-banner {
    background: #2b2615;
    border-bottom-color: #6b5b18;
    color: #ffd666;
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

  .editor-title {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
  }

  .editor-actions {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  
  .scheme-meta {
    font-size: 13px;
    color: var(--text-secondary);
  }

  .remote-meta {
    font-size: 12px;
    color: var(--primary-color);
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

  .update-modal {
    width: min(760px, 100%);
    max-height: min(760px, calc(100vh - 48px));
    background: var(--editor-bg);
    border-radius: 12px;
    box-shadow: 0 18px 60px rgba(0, 0, 0, 0.25);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .update-modal-body {
    padding: 20px;
    overflow: auto;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .status-card {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 14px 16px;
    border-radius: 10px;
    background: var(--hover-bg);
    border: 1px solid var(--border-color);
  }

  .status-card strong {
    font-size: 15px;
    color: var(--text-primary);
  }

  .status-card span {
    font-size: 13px;
    color: var(--text-secondary);
  }

  .status-card.update-available {
    border-color: #91d5ff;
    background: rgba(24, 144, 255, 0.08);
  }

  .update-meta {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .update-meta-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 12px 0;
    border-bottom: 1px solid var(--border-color);
  }

  .update-meta-row span {
    font-size: 13px;
    color: var(--text-secondary);
  }

  .update-meta-row strong {
    font-size: 14px;
    color: var(--text-primary);
    text-align: right;
  }

  .link-btn {
    border: none;
    background: transparent;
    color: var(--primary-color);
    cursor: pointer;
    font-size: 14px;
    font-weight: 600;
    padding: 0;
  }

  .link-btn:hover {
    color: var(--primary-hover);
    text-decoration: underline;
  }

  .release-notes h4 {
    margin: 0 0 10px 0;
    color: var(--text-primary);
    font-size: 15px;
  }

  .release-notes pre {
    margin: 0;
    white-space: pre-wrap;
    word-break: break-word;
    font-family: 'Consolas', 'Monaco', monospace;
    font-size: 13px;
    line-height: 1.6;
    color: var(--text-primary);
    background: var(--hover-bg);
    border: 1px solid var(--border-color);
    border-radius: 10px;
    padding: 14px 16px;
  }

  .update-actions {
    display: flex;
    justify-content: flex-end;
    gap: 12px;
  }

  .update-progress {
    padding: 12px 14px;
    border-radius: 10px;
    background: var(--hover-bg);
    border: 1px solid var(--border-color);
    color: var(--text-primary);
    font-size: 13px;
  }
</style>

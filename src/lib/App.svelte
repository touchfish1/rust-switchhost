<script lang="ts">
  import { onDestroy, onMount } from 'svelte'
  import { get } from 'svelte/store'
  import { listen, type UnlistenFn } from '@tauri-apps/api/event'
  import { open as openDialog, save } from '@tauri-apps/plugin-dialog'
  import { open } from '@tauri-apps/plugin-shell'
  import { type DownloadEvent } from '@tauri-apps/plugin-updater'
  import Sidebar from './components/Sidebar.svelte'
  import Editor from './components/Editor.svelte'
  import ThemeToggle from './components/ThemeToggle.svelte'
  import Modal from './components/Modal.svelte'
  import CreateSchemeModal from './components/CreateSchemeModal.svelte'
  import BackupHistoryModal from './components/BackupHistoryModal.svelte'
  import DnsDiagnosticModal from './components/DnsDiagnosticModal.svelte'
  import SyncLogModal from './components/SyncLogModal.svelte'
  import CurrentHostsModal from './components/CurrentHostsModal.svelte'
  import UpdateModal from './components/UpdateModal.svelte'
  import Toast from './components/Toast.svelte'
  import { getSchemeTemplateContent } from '$lib/data/templates'
  import { getAppVersion, restartApp } from '$lib/services/app'
  import {
    checkHostsPermission as fetchHostsPermission,
    flushDnsCache,
    getHostsBackupContent,
    getHostsContent,
    listHostsBackups,
    resolveDomain,
    restoreHostsBackup
  } from '$lib/services/hosts'
  import {
    getAllSchemes,
    createScheme as createSchemeRequest,
    deleteScheme as deleteSchemeRequest,
    exportSchemes as exportSchemesRequest,
    getSchemeSyncLogs,
    importSchemes as importSchemesRequest,
    setSchemeEnabled as setSchemeEnabledRequest,
    syncRemoteScheme,
    updateScheme as updateSchemeRequest,
    updateSchemeRemoteConfig as updateSchemeRemoteConfigRequest
  } from '$lib/services/schemes'
  import { checkForUpdates } from '$lib/services/updater'
  import type { DnsLookupResult, HostsBackupEntry, Scheme, SyncLogEntry } from '$lib/types'
  import { appError, appVersion, hostsPermissionInfo, loadingFlags } from '$lib/stores/app'
  import {
    activeScheme as activeSchemeStore,
    activeSchemeId as activeSchemeIdStore,
    applyUpdatedScheme as applyUpdatedSchemeStore,
    editorContent as editorContentStore,
    removeScheme,
    schemes as schemesStore,
    selectScheme,
    setSchemes,
    upsertScheme
  } from '$lib/stores/schemes'
  import { customTemplates, schemeTemplates } from '$lib/stores/templates'
  import { theme } from '$lib/stores/theme'
  import { toasts } from '$lib/stores/toasts'
  import { hasPendingUpdate, updater } from '$lib/stores/updater'

  let showCreateModal = false
  let showDeleteModal = false
  let showCurrentHostsModal = false
  let showBackupHistoryModal = false
  let showDnsDiagnosticModal = false
  let showSyncLogModal = false
  let createModalMode: 'create' | 'edit-remote' = 'create'
  let remoteEditTarget: Scheme | null = null
  let deleteTargetId: string | null = null
  let currentHostsContent = ''
  let isFlushingDns = false
  let updateCheckTimer: ReturnType<typeof setInterval> | null = null
  let isSyncingRemoteScheme = false
  let isCreatingScheme = false
  let isImportingSchemes = false
  let isExportingSchemes = false
  let isOpeningCurrentHosts = false
  let isOpeningBackupHistory = false
  let isResolvingDns = false
  let isLoadingBackupContent = false
  let isRestoringBackup = false
  let sidebarWidth = 320
  let syncLogs: SyncLogEntry[] = []
  let backupEntries: HostsBackupEntry[] = []
  let selectedBackupPath = ''
  let selectedBackupContent = ''
  let diagnosticDomain = ''
  let dnsLookupResult: DnsLookupResult | null = null
  let syncEventUnlisten: UnlistenFn | null = null
  const syncingSchemeIds = new Set<string>()

  onMount(async () => {
    const savedSidebarWidth = localStorage.getItem('sidebar-width')
    theme.initialize()
    customTemplates.initialize()
    sidebarWidth = savedSidebarWidth ? Math.min(520, Math.max(280, Number(savedSidebarWidth) || 320)) : 320
    loadingFlags.start('initial')

    try {
      await Promise.all([
        loadAppVersion(),
        loadSchemes(),
        checkHostsPermission()
      ])

      await checkForUpdatesSilently()
      updateCheckTimer = setInterval(() => {
        void checkForUpdatesSilently()
      }, 15 * 60 * 1000)
      syncEventUnlisten = await listen('schemes-changed', async () => {
        await loadSchemes()
        if (showSyncLogModal && get(activeSchemeIdStore)) {
          await loadSyncLogs(get(activeSchemeIdStore)!)
        }
      })
    } finally {
      loadingFlags.stop('initial')
    }
  })

  onDestroy(() => {
    if (updateCheckTimer) {
      clearInterval(updateCheckTimer)
      updateCheckTimer = null
    }

    if (syncEventUnlisten) {
      syncEventUnlisten()
      syncEventUnlisten = null
    }
  })

  async function loadAppVersion() {
    try {
      appVersion.set(await getAppVersion())
    } catch (e) {
      console.error('Failed to load app version:', e)
      appVersion.set('')
    }
  }

  function handleThemeToggle(detail: { isDark: boolean }) {
    theme.setTheme(detail.isDark)
  }

  async function loadSchemes() {
    try {
      appError.set(null)
      const loadedSchemes = await getAllSchemes()
      setSchemes(loadedSchemes)
    } catch (e) {
      appError.set(`加载分组失败: ${e}`)
      console.error('Failed to load schemes:', e)
    }
  }

  function handleSelectScheme(id: string) {
    selectScheme(id)
  }

  async function checkHostsPermission() {
    try {
      hostsPermissionInfo.set(await fetchHostsPermission())
    } catch (e) {
      console.error('Failed to check hosts permission:', e)
      hostsPermissionInfo.set(null)
    }
  }

  async function openCurrentHostsModal() {
    try {
      isOpeningCurrentHosts = true
      loadingFlags.start('currentHosts')
      appError.set(null)
      currentHostsContent = await getHostsContent()
      showCurrentHostsModal = true
    } catch (e) {
      appError.set(`读取当前 Hosts 失败: ${e}`)
      console.error('Failed to get current hosts content:', e)
    } finally {
      isOpeningCurrentHosts = false
      loadingFlags.stop('currentHosts')
    }
  }

  async function openBackupHistoryModal() {
    try {
      isOpeningBackupHistory = true
      appError.set(null)
      backupEntries = await listHostsBackups()
      selectedBackupPath = backupEntries[0]?.path || ''
      selectedBackupContent = ''
      showBackupHistoryModal = true

      if (selectedBackupPath) {
        await handleSelectBackup(selectedBackupPath)
      }
    } catch (e) {
      appError.set(`读取备份历史失败: ${e}`)
      console.error('Failed to load hosts backups:', e)
    } finally {
      isOpeningBackupHistory = false
    }
  }

  async function handleFlushDns() {
    try {
      isFlushingDns = true
      appError.set(null)
      const result = await flushDnsCache()
      if (result.success) {
        showToast(result.message, 'success')
      } else {
        appError.set(result.message)
      }
    } catch (e) {
      appError.set(`刷新 DNS 缓存失败: ${e}`)
      console.error('Failed to flush DNS cache:', e)
    } finally {
      isFlushingDns = false
    }
  }

  function openDnsDiagnosticModal() {
    showDnsDiagnosticModal = true
    dnsLookupResult = null
  }

  async function handleResolveDomain() {
    try {
      isResolvingDns = true
      appError.set(null)
      dnsLookupResult = await resolveDomain(diagnosticDomain)
    } catch (e) {
      appError.set(`DNS 诊断失败: ${e}`)
      console.error('Failed to resolve domain:', e)
    } finally {
      isResolvingDns = false
    }
  }

  async function handleSelectBackup(path: string) {
    if (!path) return

    try {
      isLoadingBackupContent = true
      selectedBackupPath = path
      selectedBackupContent = await getHostsBackupContent(path)
    } catch (e) {
      appError.set(`读取备份内容失败: ${e}`)
      console.error('Failed to load backup content:', e)
      selectedBackupContent = ''
    } finally {
      isLoadingBackupContent = false
    }
  }

  async function handleRestoreBackup(path: string) {
    if (!path) return

    try {
      isRestoringBackup = true
      appError.set(null)
      const message = await restoreHostsBackup(path)
      await Promise.all([
        loadSchemes(),
        checkHostsPermission()
      ])
      if (showCurrentHostsModal) {
        currentHostsContent = await getHostsContent()
      }
      showToast(message, 'success')
      showBackupHistoryModal = false
    } catch (e) {
      appError.set(`恢复备份失败: ${e}`)
      console.error('Failed to restore hosts backup:', e)
    } finally {
      isRestoringBackup = false
    }
  }

  async function performUpdateCheck(options: { silent: boolean }) {
    const { silent } = options

    try {
      if (!silent) {
        loadingFlags.start('updateCheck')
        appError.set(null)
      }

      const { releaseInfo, updaterUpdate } = await checkForUpdates()
      updater.setUpdateResult(releaseInfo, updaterUpdate)

      if (!silent) {
        updater.setShowUpdateModal(true)
      }

      if (!silent && !releaseInfo.has_update) {
        showToast(`当前已是最新版本 ${releaseInfo.current_version}`, 'success')
      }
    } catch (e) {
      if (!silent) {
        appError.set(`检查更新失败: ${e}`)
      }
      console.error(`Failed to ${silent ? 'silently c' : 'c'}heck for updates:`, e)
    } finally {
      if (!silent) {
        loadingFlags.stop('updateCheck')
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
    const currentUpdater = get(updater)

    if (!currentUpdater.availableUpdate) {
      if (currentUpdater.updateInfo?.download_url) {
        await openUpdateUrl(currentUpdater.updateInfo.download_url)
      } else if (currentUpdater.updateInfo?.html_url) {
        await openUpdateUrl(currentUpdater.updateInfo.html_url)
      }
      return
    }

    try {
      updater.startInstall()
      appError.set(null)

      await currentUpdater.availableUpdate.downloadAndInstall((event: DownloadEvent) => {
        if (event.event === 'Started') {
          const total = event.data.contentLength
          updater.setProgress(total
            ? `开始下载更新，总大小 ${(total / 1024 / 1024).toFixed(2)} MB`
            : '开始下载更新')
        } else if (event.event === 'Progress') {
          updater.setProgress(`正在下载更新，已接收 ${(event.data.chunkLength / 1024).toFixed(1)} KB 数据块`)
        } else if (event.event === 'Finished') {
          updater.setProgress('下载完成，正在安装更新...')
        }
      })

      updater.setProgress('安装完成，应用即将重启...')
      await restartApp()
    } catch (e) {
      appError.set(`安装更新失败: ${e}`)
      console.error('Failed to install update:', e)
      updater.clearProgress()
    } finally {
      updater.finishInstall()
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
    createModalMode = 'create'
    remoteEditTarget = null
    showCreateModal = true
  }

  function openRemoteEditModal(id: string) {
    const target = get(schemesStore).find((scheme) => scheme.id === id && scheme.remote_url) || null
    if (!target) return
    createModalMode = 'edit-remote'
    remoteEditTarget = target
    showCreateModal = true
  }
  
  async function handleCreateConfirm(detail: {
    name: string
    type: 'local' | 'remote'
    remoteUrl: string
    autoSyncEnabled: boolean
    syncIntervalMinutes: string
    templateId: string | null
  }) {
    const name = detail.name?.trim()
    const type = detail.type
    const remoteUrl = detail.remoteUrl?.trim() || ''
    const autoSyncEnabled = Boolean(detail.autoSyncEnabled)
    const syncIntervalInput = detail.syncIntervalMinutes
    const syncIntervalMinutes = syncIntervalInput ? Number(syncIntervalInput) : null
    const templateContent = getSchemeTemplateContent(detail.templateId)
    const currentSchemes = get(schemesStore)

    if (!name) return
    if (type === 'remote' && !remoteUrl) {
      appError.set('远程 URL 分组必须填写远程地址')
      return
    }
    if (type === 'remote' && autoSyncEnabled && (!syncIntervalMinutes || syncIntervalMinutes <= 0)) {
      appError.set('启用定时同步时，请填写大于 0 的同步间隔')
      return
    }
    
    try {
      isCreatingScheme = true
      loadingFlags.start('create')
      appError.set(null)

      if (createModalMode === 'edit-remote' && remoteEditTarget) {
        const renamedScheme = await updateSchemeRequest(
          remoteEditTarget.id,
          name,
          currentSchemes.find((scheme) => scheme.id === remoteEditTarget.id)?.content || ''
        )

        const updatedRemoteScheme = await updateSchemeRemoteConfigRequest(
          remoteEditTarget.id,
          remoteUrl,
          autoSyncEnabled,
          syncIntervalMinutes
        )

        applyUpdatedSchemeStore({
          ...updatedRemoteScheme,
          name: renamedScheme.name
        })
        showCreateModal = false
        remoteEditTarget = null
        showToast('远程分组配置已更新', 'success')
        return
      }

      let newScheme = await createSchemeRequest(
        name,
        type === 'remote'
          ? '# 远程 URL 分组\n# 首次同步后会自动填充内容\n'
          : templateContent || '# 新的 hosts 配置\n127.0.0.1 localhost\n'
      )

      if (type === 'remote') {
        newScheme = await updateSchemeRemoteConfigRequest(
          newScheme.id,
          remoteUrl,
          autoSyncEnabled,
          syncIntervalMinutes
        )
      }

      upsertScheme(newScheme)
      showCreateModal = false

      if (type === 'remote') {
        try {
          const syncedScheme = await syncRemoteScheme(newScheme.id, 'manual')
          applyUpdatedSchemeStore(syncedScheme)
          showToast('远程 URL 分组已创建并完成首次同步', 'success')
        } catch (syncError) {
          appError.set(`远程分组已创建，但首次同步失败: ${syncError}`)
          console.error('Failed to sync new remote scheme:', syncError)
        }
      }
    } catch (e) {
      appError.set(`创建分组失败: ${e}`)
      console.error('Failed to create scheme:', e)
    } finally {
      isCreatingScheme = false
      loadingFlags.stop('create')
    }
  }

  function handleSaveCurrentSchemeAsTemplate() {
    const currentScheme = get(activeSchemeStore)
    if (!currentScheme) return

    customTemplates.saveTemplate({
      name: currentScheme.name,
      description: `从分组「${currentScheme.name}」保存`,
      content: get(editorContentStore)
    })
    showToast('已保存为自定义模板，可在新建分组时直接复用', 'success')
  }

  function handleDeleteTemplate(id: string) {
    customTemplates.deleteTemplate(id)
    showToast('自定义模板已删除', 'success')
  }
  
  function openDeleteModal(id: string) {
    deleteTargetId = id
    showDeleteModal = true
  }
  
  async function handleDeleteConfirm() {
    if (!deleteTargetId) return
    
    try {
      loadingFlags.start('delete')
      appError.set(null)
      await deleteSchemeRequest(deleteTargetId)
      removeScheme(deleteTargetId)
      showDeleteModal = false
      showToast('分组已删除', 'success')
    } catch (e) {
      appError.set(`删除分组失败: ${e}`)
      console.error('Failed to delete scheme:', e)
    } finally {
      loadingFlags.stop('delete')
      deleteTargetId = null
    }
  }
  
  async function handleContentChange(detail: { content: string }) {
    const nextContent = detail.content
    setEditorContent(nextContent)
    const currentActiveSchemeId = get(activeSchemeIdStore)
    const currentActiveScheme = get(activeSchemeStore)

    if (currentActiveSchemeId) {
      try {
        const updated = await updateSchemeRequest(currentActiveSchemeId, currentActiveScheme?.name || '未命名', nextContent)
        applyUpdatedSchemeStore(updated)
      } catch (e) {
        console.error('Failed to update scheme:', e)
      }
    }
  }
  
  async function handleRename(detail: { id: string; name: string }) {
    const { id, name } = detail
    try {
      loadingFlags.start('rename')
      appError.set(null)
      const updated = await updateSchemeRequest(id, name, get(schemesStore).find((scheme) => scheme.id === id)?.content || '')
      applyUpdatedSchemeStore(updated)
    } catch (e) {
      appError.set(`重命名失败: ${e}`)
      console.error('Failed to rename scheme:', e)
    } finally {
      loadingFlags.stop('rename')
    }
  }

  async function handleToggleScheme(detail: { id: string; enabled: boolean }) {
    const { id, enabled } = detail

    try {
      loadingFlags.start('toggle')
      appError.set(null)
      const nextSchemes = await setSchemeEnabledRequest(id, enabled)
      setSchemes(nextSchemes, id)
      showToast(enabled ? '分组已启用并生效' : '分组已停用并生效', 'success')
    } catch (e) {
      appError.set(`${enabled ? '启用' : '禁用'}分组失败: ${e}`)
      console.error('Failed to toggle scheme:', e)
    } finally {
      loadingFlags.stop('toggle')
    }
  }

  async function syncSchemeById(id: string, silent = false) {
    if (syncingSchemeIds.has(id)) return

    try {
      syncingSchemeIds.add(id)
      if (!silent) {
        isSyncingRemoteScheme = true
        loadingFlags.start('sync')
        appError.set(null)
      }

      const updated = await syncRemoteScheme(id, silent ? 'scheduled' : 'manual')
      applyUpdatedSchemeStore(updated)

      if (!silent) {
        showToast('远程分组同步成功', 'success')
      }
    } catch (e) {
      if (!silent) {
        appError.set(`同步远程分组失败: ${e}`)
      }
      console.error('Failed to sync remote scheme:', e)
      await loadSchemes()
    } finally {
      syncingSchemeIds.delete(id)
      if (!silent) {
        isSyncingRemoteScheme = false
        loadingFlags.stop('sync')
      }
    }
  }

  async function handleSyncActiveScheme() {
    const currentActiveSchemeId = get(activeSchemeIdStore)
    if (!currentActiveSchemeId) return
    await syncSchemeById(currentActiveSchemeId, false)
  }

  function openSyncLogModal() {
    const currentActiveScheme = get(activeSchemeStore)
    if (!currentActiveScheme?.remote_url) return
    void loadSyncLogs(currentActiveScheme.id)
    showSyncLogModal = true
  }

  async function loadSyncLogs(id: string) {
    try {
      syncLogs = await getSchemeSyncLogs(id)
    } catch (e) {
      console.error('Failed to load sync logs:', e)
      syncLogs = []
    }
  }

  async function handleExportSchemes() {
    try {
      const exportPath = await save({
        title: '导出分组',
        defaultPath: `rust-switchhost-schemes-${get(appVersion) || 'backup'}.json`,
        filters: [
          {
            name: 'JSON',
            extensions: ['json']
          }
        ]
      })

      if (!exportPath) return

      isExportingSchemes = true
      loadingFlags.start('export')
      appError.set(null)
      await exportSchemesRequest(exportPath)
      showToast('分组已导出', 'success')
    } catch (e) {
      appError.set(`导出分组失败: ${e}`)
      console.error('Failed to export schemes:', e)
    } finally {
      isExportingSchemes = false
      loadingFlags.stop('export')
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

      isImportingSchemes = true
      loadingFlags.start('import')
      appError.set(null)
      setSchemes(await importSchemesRequest(importPath))
      showToast('分组已导入，导入项默认未启用', 'success')
    } catch (e) {
      appError.set(`导入分组失败: ${e}`)
      console.error('Failed to import schemes:', e)
    } finally {
      isImportingSchemes = false
      loadingFlags.stop('import')
    }
  }

  function showToast(message: string, kind: 'success' | 'error' | 'warning' | 'info' = 'info') {
    toasts.push(message, kind)
  }

  function handleSidebarResize(detail: { width: number }) {
    sidebarWidth = detail.width
    localStorage.setItem('sidebar-width', String(sidebarWidth))
  }

  function formatSyncStatus(status?: string) {
    switch (status) {
      case 'syncing':
        return '同步中'
      case 'success':
        return '同步成功'
      case 'error':
        return '同步失败'
      default:
        return '待同步'
    }
  }
</script>

<div class="app" class:dark={$theme}>
  <div class="header">
    <div class="header-brand">
      <h1>🔧 Rust SwitchHost</h1>
      {#if $appVersion}
        <span class="app-version">v{$appVersion}</span>
      {/if}
    </div>
    <div class="header-actions">
      <button
        class="btn-secondary update-trigger"
        class:has-notification={$hasPendingUpdate}
        on:click={handleCheckUpdates}
        disabled={$loadingFlags.updateCheck}
      >
        {$loadingFlags.updateCheck ? '检查中...' : '检查更新'}
      </button>
      <button class="btn-secondary" on:click={handleFlushDns} disabled={isFlushingDns}>
        {isFlushingDns ? '刷新 DNS 中...' : '刷新 DNS'}
      </button>
      <button class="btn-secondary" on:click={openCurrentHostsModal} disabled={isOpeningCurrentHosts}>
        {isOpeningCurrentHosts ? '读取中...' : '查看当前 Hosts'}
      </button>
      <button class="btn-secondary" on:click={openBackupHistoryModal} disabled={isOpeningBackupHistory}>
        {isOpeningBackupHistory ? '读取备份中...' : '备份恢复'}
      </button>
      <button class="btn-secondary" on:click={openDnsDiagnosticModal}>
        DNS 诊断
      </button>
      <ThemeToggle isDark={$theme} onToggle={handleThemeToggle} />
    </div>
  </div>
  
  {#if $appError}
    <div class="error-banner">
      {$appError}
      <button on:click={() => appError.set(null)}>×</button>
    </div>
  {/if}

  {#if $hostsPermissionInfo && !$hostsPermissionInfo.has_permission}
    <div class="permission-banner">
      <div>
        <strong>Hosts 权限不足</strong>
        <span>{$hostsPermissionInfo.message}</span>
      </div>
    </div>
  {/if}
  
  <div class="main">
    <Sidebar
      schemes={$schemesStore}
      activeSchemeId={$activeSchemeIdStore}
      width={sidebarWidth}
      onSelect={(detail) => handleSelectScheme(detail.id)}
      onCreate={openCreateModal}
      onImport={handleImportSchemes}
      onExport={handleExportSchemes}
      onDelete={(detail) => openDeleteModal(detail.id)}
      onEditRemote={(detail) => openRemoteEditModal(detail.id)}
      onRename={handleRename}
      onToggle={handleToggleScheme}
      onResize={handleSidebarResize}
    />
    
    <div class="content">
      {#if $loadingFlags.initial && $schemesStore.length === 0}
        <div class="initial-loading">
          <div class="spinner"></div>
          <p>正在加载分组与权限信息...</p>
        </div>
      {:else if $activeSchemeStore}
        <div class="editor-header">
          <div class="editor-title">
            <h2>{$activeSchemeStore.name}</h2>
            <span class="scheme-meta">
              分组内容编辑中 | 创建于 {new Date($activeSchemeStore.created_at).toLocaleString()}
            </span>
            {#if $activeSchemeStore.remote_url}
              <span class="remote-meta">
                远程源已配置
                {#if $activeSchemeStore.auto_sync_enabled && $activeSchemeStore.sync_interval_minutes}
                  · 每 {$activeSchemeStore.sync_interval_minutes} 分钟同步
                {/if}
              </span>
              <div class="sync-status-row">
                <span class={`sync-badge ${$activeSchemeStore.sync_status || 'idle'}`}>
                  {formatSyncStatus($activeSchemeStore.sync_status)}
                </span>
                {#if $activeSchemeStore.last_sync_message}
                  <span class="sync-message">{$activeSchemeStore.last_sync_message}</span>
                {/if}
                {#if $activeSchemeStore.next_retry_at && $activeSchemeStore.sync_status === 'error'}
                  <span class="sync-next-retry">
                    下次重试：{new Date($activeSchemeStore.next_retry_at).toLocaleString()}
                  </span>
                {/if}
              </div>
            {/if}
          </div>
          <div class="editor-actions">
            <button class="btn-secondary" on:click={handleSaveCurrentSchemeAsTemplate}>
              保存为模板
            </button>
            {#if $activeSchemeStore.remote_url}
              <button class="btn-secondary" on:click={openSyncLogModal} disabled={isSyncingRemoteScheme}>
                同步日志
              </button>
              {#if $activeSchemeStore.sync_status === 'error'}
                <button class="btn-secondary" on:click={handleSyncActiveScheme} disabled={isSyncingRemoteScheme}>
                  重试同步
                </button>
              {/if}
              <button class="btn-secondary" on:click={handleSyncActiveScheme} disabled={isSyncingRemoteScheme}>
                {isSyncingRemoteScheme ? '同步中...' : '立即同步'}
              </button>
            {/if}
          </div>
        </div>
        
        <Editor
          content={$editorContentStore}
          onChange={handleContentChange}
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

  {#if showCreateModal}
    <CreateSchemeModal
      isOpen={showCreateModal}
      isSubmitting={isCreatingScheme}
      mode={createModalMode}
      initialName={remoteEditTarget?.name || ''}
      initialType={remoteEditTarget ? 'remote' : 'local'}
      initialRemoteUrl={remoteEditTarget?.remote_url || ''}
      initialAutoSyncEnabled={remoteEditTarget?.auto_sync_enabled || false}
      initialSyncIntervalMinutes={remoteEditTarget?.sync_interval_minutes ? String(remoteEditTarget.sync_interval_minutes) : '15'}
      templates={$schemeTemplates}
      onDeleteTemplate={handleDeleteTemplate}
      onConfirm={handleCreateConfirm}
      onClose={() => {
        showCreateModal = false
        remoteEditTarget = null
        createModalMode = 'create'
      }}
    />
  {/if}
  
  {#if showDeleteModal}
    <Modal
      title="删除分组"
      confirmText="删除"
      cancelText="取消"
      type="danger"
      onConfirm={handleDeleteConfirm}
      onCancel={() => { showDeleteModal = false; deleteTargetId = null }}
      onClose={() => { showDeleteModal = false; deleteTargetId = null }}
    >
      <p class="confirm-text">确定要删除分组「{$schemesStore.find((scheme) => scheme.id === deleteTargetId)?.name || ''}」吗？</p>
      <p class="confirm-warning">此操作不可撤销。</p>
    </Modal>
  {/if}

  <CurrentHostsModal
    isOpen={showCurrentHostsModal}
    content={currentHostsContent}
    onClose={() => { showCurrentHostsModal = false }}
  />

  <BackupHistoryModal
    isOpen={showBackupHistoryModal}
    backups={backupEntries}
    {selectedBackupPath}
    {selectedBackupContent}
    isLoadingContent={isLoadingBackupContent}
    isRestoring={isRestoringBackup}
    onClose={() => { showBackupHistoryModal = false }}
    onSelectBackup={handleSelectBackup}
    onRestoreBackup={handleRestoreBackup}
  />

  <DnsDiagnosticModal
    isOpen={showDnsDiagnosticModal}
    domain={diagnosticDomain}
    lookupResult={dnsLookupResult}
    isResolving={isResolvingDns}
    onClose={() => { showDnsDiagnosticModal = false }}
    onDomainChange={(value) => { diagnosticDomain = value }}
    onResolve={handleResolveDomain}
  />

  {#if $updater.updateInfo}
    <UpdateModal
      isOpen={$updater.showUpdateModal}
      updateInfo={$updater.updateInfo}
      availableUpdate={$updater.availableUpdate}
      isInstallingUpdate={$updater.isInstallingUpdate}
      updateProgressText={$updater.updateProgressText}
      {formatPublishedAt}
      onClose={() => { updater.setShowUpdateModal(false) }}
      onOpenUrl={openUpdateUrl}
      onInstall={handleInstallUpdate}
    />
  {/if}

  {#if showSyncLogModal && $activeSchemeStore}
    <SyncLogModal
      isOpen={showSyncLogModal}
      schemeName={$activeSchemeStore.name}
      logs={syncLogs}
      onClose={() => { showSyncLogModal = false }}
    />
  {/if}

  <Toast />
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
  
  .app {
    height: 100vh;
    display: flex;
    flex-direction: column;
    background: var(--editor-bg);
    color: var(--text-primary);
    transition: background-color 0.3s, color 0.3s;
    font-family: var(--font-family);
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

  .sync-status-row {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    align-items: center;
  }

  .sync-badge {
    display: inline-flex;
    align-items: center;
    height: 24px;
    padding: 0 10px;
    border-radius: 999px;
    font-size: 12px;
    font-weight: 600;
    border: 1px solid var(--border-color);
  }

  .sync-badge.idle {
    color: var(--text-secondary);
    background: var(--hover-bg);
  }

  .sync-badge.syncing {
    color: #1677ff;
    background: rgba(22, 119, 255, 0.12);
    border-color: rgba(22, 119, 255, 0.3);
  }

  .sync-badge.success {
    color: #389e0d;
    background: rgba(82, 196, 26, 0.12);
    border-color: rgba(82, 196, 26, 0.3);
  }

  .sync-badge.error {
    color: #cf1322;
    background: rgba(255, 77, 79, 0.12);
    border-color: rgba(255, 77, 79, 0.3);
  }

  .sync-message,
  .sync-next-retry {
    font-size: 12px;
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

  .initial-loading {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 14px;
    color: var(--text-secondary);
  }

  .initial-loading p {
    margin: 0;
    font-size: 14px;
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

</style>

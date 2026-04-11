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
  import MergedHostsPreviewModal from './components/MergedHostsPreviewModal.svelte'
  import RemoteSyncPreviewModal from './components/RemoteSyncPreviewModal.svelte'
  import QuickStartGuideModal from './components/QuickStartGuideModal.svelte'
import Toast from './components/Toast.svelte'
import { builtinSchemeTemplates, getSchemeTemplateContent } from '$lib/data/templates'
  import {
    buildMergedHostsContent,
    collectAffectedDomains,
    collectHostsDiffLines,
    detectHostsConflicts,
    summarizeHostsContent,
    summarizeHostsDiff
  } from '$lib/utils/hosts-analysis'
  import { analyzeHostsContent } from '$lib/utils/hosts-editor'
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
    fetchRemoteHosts as fetchRemoteHostsRequest,
    syncRemoteScheme,
    updateScheme as updateSchemeRequest,
    updateSchemeRemoteConfig as updateSchemeRemoteConfigRequest
  } from '$lib/services/schemes'
  import { checkForUpdates } from '$lib/services/updater'
  import type {
    DnsLookupResult,
    HostsBackupEntry,
    HostsValidationIssue,
    HostsConflictGroup,
    HostsContentStats,
    HostsAffectedDomain,
    HostsDiffLine,
    HostsDiffSummary,
    Scheme,
    SyncLogEntry,
    WriteResultSummary
  } from '$lib/types'
  import { appError, appVersion, hostsPermissionInfo, loadingFlags } from '$lib/stores/app'
  import {
    activeScheme as activeSchemeStore,
    activeSchemeId as activeSchemeIdStore,
    applyUpdatedScheme as applyUpdatedSchemeStore,
    editorContent as editorContentStore,
    removeScheme,
    schemes as schemesStore,
    selectScheme,
    setEditorContent,
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
  let showMergedPreviewModal = false
  let showQuickStartGuide = false
  let showRestoreBackupConfirm = false
  let showEmptySchemeConfirm = false
  let createModalMode: 'create' | 'edit-remote' = 'create'
  let createModalInitialType: 'local' | 'remote' = 'local'
  let remoteEditTarget: Scheme | null = null
  let deleteTargetId: string | null = null
  let pendingRestoreBackupPath = ''
  let pendingEnableSchemeId: string | null = null
  let currentHostsContent = ''
  let mergedPreviewCurrentContent = ''
  let isFlushingDns = false
  let updateCheckTimer: ReturnType<typeof setInterval> | null = null
  let isSyncingRemoteScheme = false
  let isCreatingScheme = false
  let isImportingSchemes = false
  let isExportingSchemes = false
  let isOpeningMergedPreview = false
  let isOpeningCurrentHosts = false
  let isOpeningBackupHistory = false
  let isResolvingDns = false
  let isLoadingBackupContent = false
  let isRestoringBackup = false
  let sidebarWidth = 320
  let syncLogs: SyncLogEntry[] = []
  let writeResultSummary: WriteResultSummary | null = null
  let backupEntries: HostsBackupEntry[] = []
  let selectedBackupPath = ''
  let selectedBackupContent = ''
  let diagnosticDomain = ''
  let dnsLookupResult: DnsLookupResult | null = null
  let syncEventUnlisten: UnlistenFn | null = null
  const syncingSchemeIds = new Set<string>()
  let schemesForAnalysis: Scheme[] = []
  let activeSchemeSnapshot: Scheme | null = null
  let enabledSchemesForAnalysis: Scheme[] = []
  let previewSourceSchemes: Scheme[] = []
  let previewMergedHostsContent = ''
  let previewScopeLabel = ''
  let previewNote = ''
  let previewCurrentStats: HostsContentStats = { lineCount: 0, hostEntryCount: 0, commentCount: 0 }
  let previewMergedStats: HostsContentStats = { lineCount: 0, hostEntryCount: 0, commentCount: 0 }
  let previewDiffSummary: HostsDiffSummary = { addedLines: 0, removedLines: 0, unchangedLines: 0 }
  let previewDiffLines: HostsDiffLine[] = []
  let mergedPreviewConflicts: HostsConflictGroup[] = []
  let activeSchemeConflicts: HostsConflictGroup[] = []
  let deleteTargetScheme: Scheme | null = null
  let activeEditorRuleCount = 0
  let activeEditorCommentCount = 0
  let activeEditorIssues: HostsValidationIssue[] = []
  let showRemoteSyncPreviewModal = false
  let isLoadingRemoteSyncPreview = false
  let isApplyingRemoteSyncPreview = false
  let remoteSyncPreviewScheme: Scheme | null = null
  let remoteSyncPreviewContent = ''
  let remoteSyncPreviewDiff: HostsDiffSummary = { addedLines: 0, removedLines: 0, unchangedLines: 0 }
  let remoteSyncPreviewAffectedDomains: HostsAffectedDomain[] = []
  let remoteSyncPreviewDiffLines: HostsDiffLine[] = []
  let showWriteResultDetails = false
  let writeResultSearch = ''
  const QUICK_START_STORAGE_KEY = 'quick-start-guide-dismissed-v1'
  const editorTips = [
    '格式：IP 域名1 域名2',
    '注释请以 # 开头',
    '启用后只会写入软件托管区块'
  ]

  $: schemesForAnalysis = $schemesStore.map((scheme) =>
    scheme.id === $activeSchemeIdStore ? { ...scheme, content: $editorContentStore } : scheme
  )
  $: activeSchemeSnapshot = schemesForAnalysis.find((scheme) => scheme.id === $activeSchemeIdStore) || null
  $: enabledSchemesForAnalysis = schemesForAnalysis.filter((scheme) => scheme.enabled)
  $: previewSourceSchemes = enabledSchemesForAnalysis.length > 0
    ? enabledSchemesForAnalysis
    : activeSchemeSnapshot
      ? [activeSchemeSnapshot]
      : []
  $: previewMergedHostsContent = buildMergedHostsContent(previewSourceSchemes)
  $: mergedPreviewConflicts = detectHostsConflicts(previewSourceSchemes)
  $: activeSchemeConflicts = activeSchemeSnapshot ? detectHostsConflicts([activeSchemeSnapshot]) : []
  $: previewScopeLabel = enabledSchemesForAnalysis.length > 0
    ? `当前已启用的 ${enabledSchemesForAnalysis.length} 个分组`
    : activeSchemeSnapshot
      ? `当前选中的分组「${activeSchemeSnapshot.name}」`
      : '暂无可预览的分组'
  $: previewNote = enabledSchemesForAnalysis.length > 0
    ? ''
    : activeSchemeSnapshot
      ? '当前还没有启用中的分组，下面展示的是当前选中分组的预计结果。'
      : '请先创建或启用分组后再进行预览。'
  $: previewCurrentStats = summarizeHostsContent(mergedPreviewCurrentContent)
  $: previewMergedStats = summarizeHostsContent(previewMergedHostsContent)
  $: previewDiffSummary = summarizeHostsDiff(mergedPreviewCurrentContent, previewMergedHostsContent)
  $: previewDiffLines = collectHostsDiffLines(mergedPreviewCurrentContent, previewMergedHostsContent)
  $: deleteTargetScheme = $schemesStore.find((scheme) => scheme.id === deleteTargetId) || null
  $: {
    const analysis = analyzeHostsContent($editorContentStore)
    activeEditorRuleCount = analysis.ruleCount
    activeEditorCommentCount = analysis.commentCount
    activeEditorIssues = analysis.issues.slice(0, 3)
  }

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

      if (localStorage.getItem(QUICK_START_STORAGE_KEY) !== '1') {
        showQuickStartGuide = true
      }

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
        await copyUpdateDiagnostic(result.message)
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

  function openQuickStartGuide() {
    showQuickStartGuide = true
  }

  function closeQuickStartGuide() {
    showQuickStartGuide = false
    localStorage.setItem(QUICK_START_STORAGE_KEY, '1')
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

  function handleRestoreBackup(path: string) {
    if (!path) return
    pendingRestoreBackupPath = path
    showRestoreBackupConfirm = true
  }

  async function confirmRestoreBackup() {
    const path = pendingRestoreBackupPath
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
      showRestoreBackupConfirm = false
      pendingRestoreBackupPath = ''
    }
  }

  function hasEffectiveHostsRules(content: string) {
    return analyzeHostsContent(content).ruleCount > 0
  }

  async function performToggleScheme(id: string, enabled: boolean) {
    try {
      loadingFlags.start('toggle')
      appError.set(null)
      const beforeContent = await getHostsContent()
      const nextSchemes = await setSchemeEnabledRequest(id, enabled)
      const enabledSchemes = nextSchemes.filter((scheme) => scheme.enabled)
      const conflicts = detectHostsConflicts(enabledSchemes)
      setSchemes(nextSchemes, id)
      const afterContent = await getHostsContent()
      writeResultSummary = {
        title: enabled ? '分组已启用并写入系统 Hosts' : '分组已停用并重新写入系统 Hosts',
        description: enabled ? '下面是这次实际写入带来的变更摘要。' : '下面是停用后重新计算得到的变更摘要。',
        diff: summarizeHostsDiff(beforeContent, afterContent),
        diffLines: collectHostsDiffLines(beforeContent, afterContent),
        affectedDomains: collectAffectedDomains(beforeContent, afterContent),
        timestamp: new Date().toISOString()
      }
      writeResultSearch = ''
      showWriteResultDetails = false
      showToast(enabled ? '分组已启用并生效' : '分组已停用并生效', 'success')
      if (enabled && conflicts.length > 0) {
        showToast(`检测到 ${conflicts.length} 个域名冲突，建议查看“合并预览”`, 'warning', 3200)
      }
    } catch (e) {
      appError.set(`${enabled ? '启用' : '禁用'}分组失败: ${e}`)
      console.error('Failed to toggle scheme:', e)
    } finally {
      loadingFlags.stop('toggle')
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
      let downloadedBytes = 0
      let totalDownloadBytes: number | null = null

      await currentUpdater.availableUpdate.downloadAndInstall((event: DownloadEvent) => {
        if (event.event === 'Started') {
          totalDownloadBytes = typeof event.data.contentLength === 'number'
            ? event.data.contentLength
            : null
          downloadedBytes = 0
          updater.setProgress(
            totalDownloadBytes && totalDownloadBytes > 0
              ? `开始下载更新，已下载 0 MB / ${(totalDownloadBytes / 1024 / 1024).toFixed(2)} MB`
              : '开始下载更新，正在连接下载源...',
            0
          )
        } else if (event.event === 'Progress') {
          downloadedBytes += event.data.chunkLength
          if (totalDownloadBytes && totalDownloadBytes > 0) {
            const progress = (downloadedBytes / totalDownloadBytes) * 100
            updater.setProgress(
              `正在下载更新，已下载 ${(downloadedBytes / 1024 / 1024).toFixed(2)} MB / ${(totalDownloadBytes / 1024 / 1024).toFixed(2)} MB`,
              progress
            )
          } else {
            updater.setProgress(
              `正在下载更新，已接收 ${(downloadedBytes / 1024 / 1024).toFixed(2)} MB`,
              downloadedBytes > 0 ? 0 : null
            )
          }
        } else if (event.event === 'Finished') {
          updater.setProgress('下载完成，正在安装更新...', 100)
        }
      })

      updater.setProgress('安装完成，应用即将重启...', 100)
      await restartApp()
    } catch (e) {
      const message = `安装更新失败: ${e}`
      appError.set(message)
      console.error('Failed to install update:', e)
      updater.clearProgress()
      await copyUpdateDiagnostic(message)
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
  
  function openCreateModal(initialType: 'local' | 'remote' = 'local') {
    createModalMode = 'create'
    createModalInitialType = initialType
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
    const duplicatedScheme = currentSchemes.find((scheme) =>
      scheme.id !== remoteEditTarget?.id &&
      scheme.name.trim().toLowerCase() === name?.toLowerCase()
    )

    if (!name) return
    if (duplicatedScheme) {
      appError.set(`已存在同名分组「${duplicatedScheme.name}」`)
      return
    }
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
      createModalInitialType = 'local'

      if (type === 'remote') {
        try {
          const syncedScheme = await syncRemoteScheme(newScheme.id, 'manual')
          applyUpdatedSchemeStore(syncedScheme)
          showToast('远程 URL 分组已创建并完成首次同步', 'success')
        } catch (syncError) {
          appError.set(`远程分组已创建，但首次同步失败: ${syncError}`)
          console.error('Failed to sync new remote scheme:', syncError)
        }
      } else {
        showToast('本地分组已创建', 'success')
      }
    } catch (e) {
      appError.set(`创建分组失败: ${e}`)
      console.error('Failed to create scheme:', e)
    } finally {
      isCreatingScheme = false
      loadingFlags.stop('create')
    }
  }

  function getSuggestedDemoSchemeName() {
    const baseName = '示例分组'
    const existingNames = new Set(get(schemesStore).map((scheme) => scheme.name.trim().toLowerCase()))
    if (!existingNames.has(baseName.toLowerCase())) return baseName

    let index = 2
    while (existingNames.has(`${baseName} ${index}`.toLowerCase())) {
      index += 1
    }
    return `${baseName} ${index}`
  }

  async function handleCreateExampleScheme() {
    try {
      loadingFlags.start('create')
      appError.set(null)
      const template = builtinSchemeTemplates.find((item) => item.id === 'example-group')
      const created = await createSchemeRequest(
        getSuggestedDemoSchemeName(),
        template?.content || '# 示例分组\n127.0.0.1 demo.local.test\n'
      )
      upsertScheme(created)
      selectScheme(created.id)
      showToast('示例分组已创建，可以直接启用或先查看合并预览', 'success')
    } catch (e) {
      appError.set(`创建示例分组失败: ${e}`)
      console.error('Failed to create example scheme:', e)
    } finally {
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

  async function openMergedPreviewModal() {
    if (previewSourceSchemes.length === 0) return

    try {
      isOpeningMergedPreview = true
      appError.set(null)
      mergedPreviewCurrentContent = await getHostsContent()
      showMergedPreviewModal = true
    } catch (e) {
      appError.set(`读取当前 Hosts 失败: ${e}`)
      console.error('Failed to get hosts content for preview:', e)
    } finally {
      isOpeningMergedPreview = false
    }
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

    if (enabled) {
      const targetScheme = schemesForAnalysis.find((scheme) => scheme.id === id)
      if (targetScheme && !hasEffectiveHostsRules(targetScheme.content)) {
        pendingEnableSchemeId = id
        showEmptySchemeConfirm = true
        return
      }
    }

    await performToggleScheme(id, enabled)
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
      const beforeContent = silent ? '' : await getHostsContent()

      const updated = await syncRemoteScheme(id, silent ? 'scheduled' : 'manual')
      applyUpdatedSchemeStore(updated)

      if (!silent) {
        if (updated.enabled) {
          const afterContent = await getHostsContent()
          writeResultSummary = {
            title: '远程分组已同步并写入系统 Hosts',
            description: '下面是这次同步后实际写入带来的变更摘要。',
            diff: summarizeHostsDiff(beforeContent, afterContent),
            diffLines: collectHostsDiffLines(beforeContent, afterContent),
            affectedDomains: collectAffectedDomains(beforeContent, afterContent),
            timestamp: new Date().toISOString()
          }
          writeResultSearch = ''
          showWriteResultDetails = false
        }
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
    const currentScheme = get(activeSchemeStore)
    if (!currentScheme?.id || !currentScheme.remote_url) return

    try {
      isLoadingRemoteSyncPreview = true
      appError.set(null)
      remoteSyncPreviewScheme = currentScheme
      remoteSyncPreviewContent = await fetchRemoteHostsRequest(currentScheme.remote_url)
      remoteSyncPreviewDiff = summarizeHostsDiff(currentScheme.content, remoteSyncPreviewContent)
      remoteSyncPreviewDiffLines = collectHostsDiffLines(currentScheme.content, remoteSyncPreviewContent)
      remoteSyncPreviewAffectedDomains = collectAffectedDomains(currentScheme.content, remoteSyncPreviewContent)
      showRemoteSyncPreviewModal = true
    } catch (e) {
      appError.set(`拉取远程预览失败: ${e}`)
      console.error('Failed to fetch remote preview:', e)
    } finally {
      isLoadingRemoteSyncPreview = false
    }
  }

  async function confirmRemoteSyncPreview() {
    if (!remoteSyncPreviewScheme) return

    try {
      isApplyingRemoteSyncPreview = true
      showRemoteSyncPreviewModal = true
      await syncSchemeById(remoteSyncPreviewScheme.id, false)
    } finally {
      isApplyingRemoteSyncPreview = false
      showRemoteSyncPreviewModal = false
      remoteSyncPreviewScheme = null
      remoteSyncPreviewContent = ''
      remoteSyncPreviewDiff = { addedLines: 0, removedLines: 0, unchangedLines: 0 }
      remoteSyncPreviewDiffLines = []
      remoteSyncPreviewAffectedDomains = []
    }
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

  function showToast(
    message: string,
    kind: 'success' | 'error' | 'warning' | 'info' = 'info',
    duration?: number
  ) {
    toasts.push(message, kind, duration)
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

  async function copyUpdateDiagnostic(message: string) {
    const payload = [
      `应用版本: ${get(appVersion) || 'unknown'}`,
      `平台: ${navigator.platform || 'unknown'}`,
      `时间: ${new Date().toLocaleString()}`,
      `错误: ${message}`
    ].join('\n')

    try {
      await navigator.clipboard.writeText(payload)
      showToast('已复制诊断信息，可直接反馈给开发者', 'info')
    } catch (error) {
      console.error('Failed to copy diagnostic info:', error)
    }
  }

  function formatLastSync(value?: string | null) {
    if (!value) return '还没有同步记录'
    return new Date(value).toLocaleString()
  }

  function getSyncFailureSuggestion(message?: string | null) {
    const normalized = (message || '').toLowerCase()
    if (!normalized) return ''
    if (normalized.includes('timeout') || normalized.includes('timed out')) {
      return '建议检查网络连通性，或稍后重试同步。'
    }
    if (normalized.includes('dns') || normalized.includes('resolve') || normalized.includes('name or service not known')) {
      return '建议检查远程地址是否可访问，或先确认当前 DNS/代理配置。'
    }
    if (normalized.includes('404') || normalized.includes('403') || normalized.includes('401')) {
      return '建议确认远程 URL 是否正确、资源是否仍存在，以及访问权限是否正常。'
    }
    if (normalized.includes('ssl') || normalized.includes('certificate') || normalized.includes('tls')) {
      return '建议检查证书有效性，必要时改用可信 HTTPS 源。'
    }
    return '建议先查看同步日志，再根据错误信息检查网络、地址和权限配置。'
  }

  function getSyncOverviewTone(status?: string) {
    switch (status) {
      case 'success':
        return 'success'
      case 'error':
        return 'warning'
      case 'syncing':
        return 'info'
      default:
        return 'neutral'
    }
  }

  function matchesWriteResultSearch(value: string) {
    const keyword = writeResultSearch.trim().toLowerCase()
    if (!keyword) return true
    return value.toLowerCase().includes(keyword)
  }

  async function copyWriteResultDiff() {
    if (!writeResultSummary) return

    const content = writeResultSummary.diffLines
      .filter((item) => matchesWriteResultSearch(item.value))
      .map((item) => `${item.kind === 'added' ? '+' : '-'} ${item.value}`)
      .join('\n')

    if (!content) {
      showToast('当前筛选条件下没有可复制的 diff 内容', 'warning')
      return
    }

    try {
      await navigator.clipboard.writeText(content)
      showToast('写入 diff 已复制到剪贴板', 'success')
    } catch (error) {
      console.error('Failed to copy write result diff:', error)
      showToast('复制 diff 失败，请检查系统剪贴板权限', 'error')
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
      <button class="btn-secondary" on:click={openQuickStartGuide}>
        使用引导
      </button>
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

  {#if writeResultSummary}
    <div class="write-result-banner">
      <div>
        <strong>{writeResultSummary.title}</strong>
        <span>{writeResultSummary.description}</span>
        <small>
          新增 {writeResultSummary.diff.addedLines} 行 · 移除 {writeResultSummary.diff.removedLines} 行 · 保留 {writeResultSummary.diff.unchangedLines} 行
          · {new Date(writeResultSummary.timestamp).toLocaleTimeString()}
        </small>
        {#if writeResultSummary.affectedDomains.length > 0}
          <input
            class="write-result-search"
            type="text"
            bind:value={writeResultSearch}
            placeholder="按域名筛选这次写入结果"
          />
          <div class="write-result-domains">
            {#each writeResultSummary.affectedDomains.filter((item) => matchesWriteResultSearch(item.domain)) as item}
              <span class={`write-domain-chip ${item.change}`}>
                {item.domain} · {item.change === 'added' ? '新增' : item.change === 'removed' ? '移除' : '更新'}
              </span>
            {/each}
          </div>
        {/if}
        <button class="write-result-toggle" on:click={copyWriteResultDiff}>
          复制当前 diff
        </button>
        <button class="write-result-toggle" on:click={() => { showWriteResultDetails = !showWriteResultDetails }}>
          {showWriteResultDetails ? '收起详细 diff' : '展开详细 diff'}
        </button>
        {#if showWriteResultDetails && writeResultSummary.diffLines.length > 0}
          <pre class="write-result-diff">
{writeResultSummary.diffLines
  .filter((item) => matchesWriteResultSearch(item.value))
  .map((item) => `${item.kind === 'added' ? '+' : '-'} ${item.value}`)
  .join('\n')}
          </pre>
        {/if}
      </div>
      <button on:click={() => { writeResultSummary = null }}>×</button>
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
            <div class="analysis-row">
              <span class="analysis-chip">
                {enabledSchemesForAnalysis.length > 0
                  ? `已启用 ${enabledSchemesForAnalysis.length} 个分组`
                  : '当前未启用分组'}
              </span>
              {#if mergedPreviewConflicts.length > 0}
                <span class="analysis-chip warning">
                  合并冲突 {mergedPreviewConflicts.length} 个域名
                </span>
              {:else if previewSourceSchemes.length > 0}
                <span class="analysis-chip success">
                  {enabledSchemesForAnalysis.length > 0 ? '合并结果未发现冲突' : '当前分组未发现冲突'}
                </span>
              {/if}
              {#if activeSchemeConflicts.length > 0 && !$activeSchemeStore.enabled}
                <span class="analysis-chip warning">
                  当前分组内存在 {activeSchemeConflicts.length} 个冲突域名
                </span>
              {/if}
            </div>
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
              {#if $activeSchemeStore.sync_status === 'error' && $activeSchemeStore.last_sync_error}
                <div class="sync-suggestion">
                  建议处理：{getSyncFailureSuggestion($activeSchemeStore.last_sync_error)}
                </div>
              {/if}
              <div class="sync-overview-grid">
                <div class={`sync-overview-card ${getSyncOverviewTone($activeSchemeStore.sync_status)}`}>
                  <span>同步状态</span>
                  <strong>{formatSyncStatus($activeSchemeStore.sync_status)}</strong>
                </div>
                <div class="sync-overview-card neutral">
                  <span>上次同步</span>
                  <strong>{formatLastSync($activeSchemeStore.last_synced_at)}</strong>
                </div>
                <div class={`sync-overview-card ${($activeSchemeStore.consecutive_failures || 0) > 0 ? 'warning' : 'success'}`}>
                  <span>连续失败</span>
                  <strong>{($activeSchemeStore.consecutive_failures || 0) > 0 ? `${$activeSchemeStore.consecutive_failures} 次` : '0 次'}</strong>
                </div>
                <div class="sync-overview-card neutral">
                  <span>同步方式</span>
                  <strong>
                    {$activeSchemeStore.auto_sync_enabled && $activeSchemeStore.sync_interval_minutes
                      ? `自动 · ${$activeSchemeStore.sync_interval_minutes} 分钟`
                      : '手动触发'}
                  </strong>
                </div>
              </div>
            {/if}
          </div>
          <div class="editor-actions">
            <button
              class="btn-secondary"
              on:click={openMergedPreviewModal}
              disabled={isOpeningMergedPreview || previewSourceSchemes.length === 0}
            >
              {isOpeningMergedPreview ? '生成预览中...' : '合并预览'}
            </button>
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
          summaryText={`当前分组包含 ${activeEditorRuleCount} 条规则，${activeEditorCommentCount} 条注释${activeEditorIssues.length > 0 ? `，待修正 ${activeEditorIssues.length} 处提示` : ''}`}
          tips={editorTips}
          issues={activeEditorIssues}
          onChange={handleContentChange}
        />
      {:else}
        <div class="empty-state">
          <h2>欢迎使用 Rust SwitchHost</h2>
          <p>请从左侧选择一个分组，或先创建本地/远程分组开始使用</p>
          <div class="empty-actions">
            <button class="btn-primary" on:click={() => openCreateModal('local')}>
              新建本地分组
            </button>
            <button class="btn-secondary" on:click={() => openCreateModal('remote')}>
              新建远程分组
            </button>
            <button class="btn-secondary" on:click={handleCreateExampleScheme}>
              创建示例分组
            </button>
            <button class="btn-secondary" on:click={openQuickStartGuide}>
              查看使用引导
            </button>
          </div>
          <span class="empty-hint">本地分组适合手动维护，远程分组适合团队共享与定时同步。</span>
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
      initialType={remoteEditTarget ? 'remote' : createModalInitialType}
      initialRemoteUrl={remoteEditTarget?.remote_url || ''}
      initialAutoSyncEnabled={remoteEditTarget ? Boolean(remoteEditTarget.auto_sync_enabled) : createModalInitialType === 'remote'}
      initialSyncIntervalMinutes={remoteEditTarget?.sync_interval_minutes ? String(remoteEditTarget.sync_interval_minutes) : '15'}
      existingSchemes={$schemesStore}
      editingSchemeId={remoteEditTarget?.id || null}
      templates={$schemeTemplates}
      onDeleteTemplate={handleDeleteTemplate}
      onConfirm={handleCreateConfirm}
      onClose={() => {
        showCreateModal = false
        remoteEditTarget = null
        createModalMode = 'create'
        createModalInitialType = 'local'
      }}
    />
  {/if}

  {#if showRestoreBackupConfirm}
    <Modal
      title="恢复备份版本"
      confirmText="恢复并覆盖当前托管区块"
      cancelText="取消"
      type="danger"
      onConfirm={confirmRestoreBackup}
      onCancel={() => { showRestoreBackupConfirm = false; pendingRestoreBackupPath = '' }}
      onClose={() => { showRestoreBackupConfirm = false; pendingRestoreBackupPath = '' }}
    >
      <p class="confirm-text">确定要恢复当前选中的 Hosts 备份吗？</p>
      <p class="confirm-meta">恢复后会立即写入系统 Hosts，并以备份内容替换当前软件托管区块状态。</p>
      <p class="confirm-warning">建议先确认备份预览内容与时间点，再执行恢复。</p>
    </Modal>
  {/if}

  {#if showEmptySchemeConfirm}
    <Modal
      title="启用空规则分组"
      confirmText="仍然启用"
      cancelText="返回检查"
      type="danger"
      onConfirm={() => {
        const id = pendingEnableSchemeId
        showEmptySchemeConfirm = false
        pendingEnableSchemeId = null
        if (id) void performToggleScheme(id, true)
      }}
      onCancel={() => { showEmptySchemeConfirm = false; pendingEnableSchemeId = null }}
      onClose={() => { showEmptySchemeConfirm = false; pendingEnableSchemeId = null }}
    >
      <p class="confirm-text">这个分组还没有有效的 Hosts 规则，启用后通常不会带来实际映射变化。</p>
      <p class="confirm-meta">如果你只是想先占位或稍后再填写内容，可以继续启用；否则建议先补上规则再生效。</p>
      <p class="confirm-warning">继续启用不会修改系统原始 Hosts 内容，但可能让你误以为分组已经产生效果。</p>
    </Modal>
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
      <p class="confirm-text">确定要删除分组「{deleteTargetScheme?.name || ''}」吗？</p>
      {#if deleteTargetScheme}
        <p class="confirm-meta">
          类型：{deleteTargetScheme.remote_url ? '远程分组' : '本地分组'} · 状态：{deleteTargetScheme.enabled ? '已启用' : '未启用'}
        </p>
        {#if deleteTargetScheme.enabled}
          <p class="confirm-meta">
            删除后会立即重新计算剩余启用分组，并把新的合并结果写入系统 Hosts。
          </p>
        {/if}
      {/if}
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
      updateProgressValue={$updater.updateProgressValue}
      {formatPublishedAt}
      onClose={() => { updater.setShowUpdateModal(false) }}
      onOpenUrl={openUpdateUrl}
      onInstall={handleInstallUpdate}
      onCopyDiagnostic={() => copyUpdateDiagnostic($appError || '请结合当前升级状态反馈问题')}
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

  <MergedHostsPreviewModal
    isOpen={showMergedPreviewModal}
    currentContent={mergedPreviewCurrentContent}
    mergedContent={previewMergedHostsContent}
    {previewScopeLabel}
    note={previewNote}
    sourceSchemes={previewSourceSchemes}
    currentStats={previewCurrentStats}
    mergedStats={previewMergedStats}
    diffSummary={previewDiffSummary}
    diffLines={previewDiffLines}
    conflicts={mergedPreviewConflicts}
    onClose={() => { showMergedPreviewModal = false }}
  />

  <RemoteSyncPreviewModal
    isOpen={showRemoteSyncPreviewModal}
    schemeName={remoteSyncPreviewScheme?.name || ''}
    remoteUrl={remoteSyncPreviewScheme?.remote_url || ''}
    currentContent={remoteSyncPreviewScheme?.content || ''}
    remoteContent={remoteSyncPreviewContent}
    diffSummary={remoteSyncPreviewDiff}
    diffLines={remoteSyncPreviewDiffLines}
    affectedDomains={remoteSyncPreviewAffectedDomains}
    isLoading={isLoadingRemoteSyncPreview}
    isApplying={isApplyingRemoteSyncPreview}
    onClose={() => {
      if (isApplyingRemoteSyncPreview) return
      showRemoteSyncPreviewModal = false
      remoteSyncPreviewScheme = null
      remoteSyncPreviewContent = ''
      remoteSyncPreviewDiff = { addedLines: 0, removedLines: 0, unchangedLines: 0 }
      remoteSyncPreviewAffectedDomains = []
    }}
    onConfirm={confirmRemoteSyncPreview}
  />

  <QuickStartGuideModal
    isOpen={showQuickStartGuide}
    onClose={closeQuickStartGuide}
  />

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

  .write-result-banner {
    padding: 12px 24px;
    background: rgba(82, 196, 26, 0.1);
    border-bottom: 1px solid rgba(82, 196, 26, 0.28);
    color: #237804;
    display: flex;
    justify-content: space-between;
    gap: 16px;
    align-items: flex-start;
  }

  .write-result-banner div {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .write-result-banner strong,
  .write-result-banner span,
  .write-result-banner small {
    line-height: 1.5;
  }

  .write-result-banner span,
  .write-result-banner small {
    font-size: 13px;
  }

  .write-result-banner button {
    border: none;
    background: transparent;
    color: inherit;
    font-size: 20px;
    cursor: pointer;
  }

  .write-result-domains {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-top: 4px;
  }

  .write-result-search {
    width: min(360px, 100%);
    padding: 10px 12px;
    border-radius: 10px;
    border: 1px solid rgba(35, 120, 4, 0.18);
    background: rgba(255, 255, 255, 0.75);
    color: var(--text-primary);
    font-size: 13px;
    margin-top: 6px;
  }

  .write-domain-chip {
    display: inline-flex;
    align-items: center;
    min-height: 24px;
    padding: 0 10px;
    border-radius: 999px;
    font-size: 12px;
    border: 1px solid rgba(35, 120, 4, 0.18);
    background: rgba(255, 255, 255, 0.55);
    color: #237804;
  }

  .write-domain-chip.removed {
    color: #cf1322;
    border-color: rgba(255, 77, 79, 0.24);
    background: rgba(255, 77, 79, 0.08);
  }

  .write-domain-chip.updated {
    color: #0958d9;
    border-color: rgba(24, 144, 255, 0.24);
    background: rgba(24, 144, 255, 0.08);
  }

  .write-result-toggle {
    align-self: flex-start;
    padding: 0;
    font-size: 13px !important;
    font-weight: 600;
    text-decoration: underline;
  }

  .write-result-diff {
    margin: 0;
    padding: 12px 14px;
    border-radius: 10px;
    border: 1px solid rgba(35, 120, 4, 0.18);
    background: rgba(255, 255, 255, 0.65);
    color: #1f1f1f;
    font-size: 12px;
    line-height: 1.6;
    white-space: pre-wrap;
    word-break: break-word;
    max-height: 220px;
    overflow: auto;
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

  .analysis-row {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    align-items: center;
  }

  .analysis-chip {
    display: inline-flex;
    align-items: center;
    min-height: 24px;
    padding: 0 10px;
    border-radius: 999px;
    font-size: 12px;
    font-weight: 600;
    border: 1px solid var(--border-color);
    background: var(--sidebar-bg);
    color: var(--text-secondary);
  }

  .analysis-chip.success {
    color: #389e0d;
    background: rgba(82, 196, 26, 0.12);
    border-color: rgba(82, 196, 26, 0.3);
  }

  .analysis-chip.warning {
    color: #cf1322;
    background: rgba(255, 77, 79, 0.12);
    border-color: rgba(255, 77, 79, 0.3);
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

  .sync-overview-grid {
    margin-top: 8px;
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 10px;
  }

  .sync-overview-card {
    padding: 10px 12px;
    border-radius: 12px;
    border: 1px solid var(--border-color);
    background: var(--sidebar-bg);
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-width: 0;
  }

  .sync-overview-card span {
    font-size: 12px;
    color: var(--text-secondary);
  }

  .sync-overview-card strong {
    font-size: 13px;
    color: var(--text-primary);
    line-height: 1.5;
    word-break: break-word;
  }

  .sync-overview-card.success {
    border-color: rgba(82, 196, 26, 0.3);
    background: rgba(82, 196, 26, 0.08);
  }

  .sync-overview-card.warning {
    border-color: rgba(255, 77, 79, 0.3);
    background: rgba(255, 77, 79, 0.08);
  }

  .sync-overview-card.info {
    border-color: rgba(24, 144, 255, 0.3);
    background: rgba(24, 144, 255, 0.08);
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

  .sync-suggestion {
    margin-top: 4px;
    font-size: 12px;
    color: #ad6800;
    line-height: 1.6;
    padding: 8px 10px;
    border-radius: 10px;
    background: rgba(250, 173, 20, 0.12);
    border: 1px solid rgba(250, 173, 20, 0.24);
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

  .empty-actions {
    display: flex;
    gap: 12px;
    margin-bottom: 14px;
  }

  .empty-hint {
    font-size: 13px;
    color: var(--text-secondary);
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

  .confirm-meta {
    margin: 0 0 8px 0;
    font-size: 13px;
    line-height: 1.6;
    color: var(--text-secondary);
  }
  
  .confirm-warning {
    margin: 0;
    font-size: 13px;
    color: var(--danger-color, #ff4d4f);
  }

  @media (max-width: 1080px) {
    .editor-header {
      flex-direction: column;
      align-items: flex-start;
      gap: 12px;
    }

    .editor-actions {
      flex-wrap: wrap;
    }

    .sync-overview-grid {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
  }

  @media (max-width: 720px) {
    .header {
      height: auto;
      padding: 16px;
      flex-direction: column;
      align-items: stretch;
      gap: 12px;
    }

    .header-actions,
    .empty-actions {
      flex-wrap: wrap;
    }

    .empty-actions {
      width: 100%;
      max-width: 320px;
      flex-direction: column;
    }

    .sync-overview-grid {
      grid-template-columns: 1fr;
    }
  }

</style>

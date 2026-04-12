<script lang="ts">
  import { onDestroy, onMount } from 'svelte'
  import { listen, type UnlistenFn } from '@tauri-apps/api/event'
  import { Window, getCurrentWindow } from '@tauri-apps/api/window'
  import {
    getTrayMetricsWindowState,
    setTrayMetricsWindowState,
    type TrayMetricsWindowState
  } from '$lib/services/app'

  type MetricsSnapshot = {
    cpu_usage: number
    used_memory: number
    total_memory: number
    download_bytes_per_sec: number
    upload_bytes_per_sec: number
  }

  type WidgetMode = 'compact' | 'detailed'

  const HISTORY_LIMIT = 24
  const MODE_STORAGE_KEY = 'tray-metrics-mode'
  const currentWindow = getCurrentWindow()
  const isLinux = navigator.userAgent.toLowerCase().includes('linux')

  let metrics: MetricsSnapshot = {
    cpu_usage: 0,
    used_memory: 0,
    total_memory: 0,
    download_bytes_per_sec: 0,
    upload_bytes_per_sec: 0
  }

  let focusedUnlisten: UnlistenFn | null = null
  let metricsUnlisten: UnlistenFn | null = null
  let movedUnlisten: UnlistenFn | null = null
  let persistPositionTimer: ReturnType<typeof setTimeout> | null = null
  let lastUpdated = '等待数据...'
  let widgetMode: WidgetMode = 'detailed'
  let cpuHistory: number[] = []
  let memoryHistory: number[] = []
  let downloadHistory: number[] = []
  let uploadHistory: number[] = []
  let pendingPositionState: TrayMetricsWindowState | null = null

  $: memoryPercent = metrics.total_memory > 0
    ? Math.round((metrics.used_memory / metrics.total_memory) * 100)
    : 0

  $: cpuValue = `${Math.round(metrics.cpu_usage)}%`
  $: memoryValue = `${formatBytes(metrics.used_memory)} / ${formatBytes(metrics.total_memory)}`
  $: downloadValue = formatRate(metrics.download_bytes_per_sec)
  $: uploadValue = formatRate(metrics.upload_bytes_per_sec)
  $: cpuSparkline = buildSparkline(cpuHistory, 100)
  $: memorySparkline = buildSparkline(memoryHistory, 100)
  $: downloadSparkline = buildSparkline(downloadHistory, Math.max(...downloadHistory, 1))
  $: uploadSparkline = buildSparkline(uploadHistory, Math.max(...uploadHistory, 1))

  onMount(async () => {
    document.body.classList.add('tray-metrics-window')
    widgetMode = loadWidgetMode()

    const savedPosition = await getTrayMetricsWindowState().catch(() => null)
    if (savedPosition) {
      pendingPositionState = savedPosition
    }

    metricsUnlisten = await listen<MetricsSnapshot>('metrics-updated', (event) => {
      const snapshot = event.payload
      const nextMemoryPercent = snapshot.total_memory > 0
        ? Math.round((snapshot.used_memory / snapshot.total_memory) * 100)
        : 0

      metrics = snapshot
      cpuHistory = pushMetricSample(cpuHistory, Math.round(snapshot.cpu_usage))
      memoryHistory = pushMetricSample(memoryHistory, nextMemoryPercent)
      downloadHistory = pushMetricSample(downloadHistory, snapshot.download_bytes_per_sec)
      uploadHistory = pushMetricSample(uploadHistory, snapshot.upload_bytes_per_sec)
      lastUpdated = formatTime(new Date())
    })

    movedUnlisten = await currentWindow.onMoved(({ payload }) => {
      pendingPositionState = {
        x: Math.round(payload.x),
        y: Math.round(payload.y)
      }
      queuePersistPosition()
    })

    focusedUnlisten = await currentWindow.onFocusChanged(({ payload: focused }) => {
      if (!focused && !isLinux) {
        void currentWindow.hide()
      }
    })
  })

  onDestroy(() => {
    document.body.classList.remove('tray-metrics-window')

    if (metricsUnlisten) {
      metricsUnlisten()
      metricsUnlisten = null
    }

    if (movedUnlisten) {
      movedUnlisten()
      movedUnlisten = null
    }

    if (focusedUnlisten) {
      focusedUnlisten()
      focusedUnlisten = null
    }

    if (persistPositionTimer) {
      clearTimeout(persistPositionTimer)
      persistPositionTimer = null
    }
  })

  async function openMainWindow() {
    const mainWindow = await Window.getByLabel('main')
    if (mainWindow) {
      await mainWindow.show()
      await mainWindow.setFocus()
    }

    await currentWindow.hide()
  }

  function setWidgetMode(mode: WidgetMode) {
    widgetMode = mode
    localStorage.setItem(MODE_STORAGE_KEY, mode)
  }

  function loadWidgetMode(): WidgetMode {
    const savedMode = localStorage.getItem(MODE_STORAGE_KEY)
    return savedMode === 'compact' ? 'compact' : 'detailed'
  }

  function queuePersistPosition() {
    if (persistPositionTimer) {
      clearTimeout(persistPositionTimer)
    }

    persistPositionTimer = setTimeout(() => {
      if (pendingPositionState) {
        void setTrayMetricsWindowState(pendingPositionState)
      }
    }, 120)
  }

  function pushMetricSample(samples: number[], value: number) {
    return [...samples.slice(-(HISTORY_LIMIT - 1)), value]
  }

  function buildSparkline(samples: number[], maxValue: number) {
    if (samples.length === 0) {
      return ''
    }

    const width = 100
    const height = 32
    const safeMax = maxValue <= 0 ? 1 : maxValue

    return samples
      .map((value, index) => {
        const x = samples.length === 1 ? width / 2 : (index / (samples.length - 1)) * width
        const y = height - (Math.min(value, safeMax) / safeMax) * height
        return `${x.toFixed(2)},${y.toFixed(2)}`
      })
      .join(' ')
  }

  function formatRate(bytesPerSecond: number) {
    return `${formatBytes(bytesPerSecond)}/s`
  }

  function formatBytes(bytes: number) {
    const units = ['B', 'KB', 'MB', 'GB', 'TB']
    let unitIndex = 0
    let value = bytes

    while (value >= 1024 && unitIndex < units.length - 1) {
      value /= 1024
      unitIndex += 1
    }

    if (value >= 100 || unitIndex === 0) {
      return `${value.toFixed(0)} ${units[unitIndex]}`
    }

    if (value >= 10) {
      return `${value.toFixed(1)} ${units[unitIndex]}`
    }

    return `${value.toFixed(2)} ${units[unitIndex]}`
  }

  function formatTime(date: Date) {
    return new Intl.DateTimeFormat('zh-CN', {
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit'
    }).format(date)
  }
</script>

<section class="widget-shell" data-tauri-drag-region>
  <div class="widget-card" class:compact={widgetMode === 'compact'}>
    <header class="widget-header">
      <div class="widget-title" data-tauri-drag-region>
        <p class="eyebrow">Tray Monitor</p>
        <h1>系统状态</h1>
        {#if widgetMode === 'detailed'}
          <small>可拖拽，位置自动记忆</small>
        {/if}
      </div>

      <div class="widget-actions">
        <div class="mode-toggle">
          <button
            class:active={widgetMode === 'compact'}
            type="button"
            on:click={() => setWidgetMode('compact')}
          >
            紧凑
          </button>
          <button
            class:active={widgetMode === 'detailed'}
            type="button"
            on:click={() => setWidgetMode('detailed')}
          >
            详细
          </button>
        </div>

        <button class="open-main" type="button" on:click={openMainWindow}>
          主界面
        </button>
      </div>
    </header>

    <div class="metrics-grid">
      <article class="metric-card cpu">
        <span class="metric-label">CPU</span>
        <strong>{cpuValue}</strong>
        <div class="sparkline-wrap">
          <svg viewBox="0 0 100 32" aria-hidden="true">
            <polyline points={cpuSparkline} />
          </svg>
        </div>
        {#if widgetMode === 'detailed'}
          <small>最近 {cpuHistory.length || 0} 次采样</small>
        {/if}
      </article>

      <article class="metric-card memory">
        <span class="metric-label">内存</span>
        <strong>{memoryPercent}%</strong>
        <div class="sparkline-wrap">
          <svg viewBox="0 0 100 32" aria-hidden="true">
            <polyline points={memorySparkline} />
          </svg>
        </div>
        <small>{memoryValue}</small>
      </article>

      <article class="metric-card network">
        <span class="metric-label">下载</span>
        <strong>{downloadValue}</strong>
        <div class="sparkline-wrap">
          <svg viewBox="0 0 100 32" aria-hidden="true">
            <polyline points={downloadSparkline} />
          </svg>
        </div>
        {#if widgetMode === 'detailed'}
          <small>实时吞吐趋势</small>
        {/if}
      </article>

      <article class="metric-card upload">
        <span class="metric-label">上传</span>
        <strong>{uploadValue}</strong>
        <div class="sparkline-wrap">
          <svg viewBox="0 0 100 32" aria-hidden="true">
            <polyline points={uploadSparkline} />
          </svg>
        </div>
        {#if widgetMode === 'detailed'}
          <small>实时吞吐趋势</small>
        {/if}
      </article>
    </div>

    <footer class="widget-footer">
      <span>
        {#if isLinux}
          Linux 下左键先看托盘菜单
        {:else if widgetMode === 'compact'}
          失焦自动收起
        {:else}
          拖拽后会记忆位置，失焦自动收起
        {/if}
      </span>
      <span>{lastUpdated}</span>
    </footer>
  </div>
</section>

<style>
  :global(body.tray-metrics-window) {
    min-width: 0;
    min-height: 0;
    background: transparent;
    overflow: hidden;
  }

  .widget-shell {
    min-height: 100vh;
    padding: 10px;
    background:
      radial-gradient(circle at top left, rgba(54, 99, 181, 0.18), transparent 42%),
      radial-gradient(circle at bottom right, rgba(25, 168, 136, 0.18), transparent 38%),
      transparent;
  }

  .widget-card {
    height: calc(100vh - 20px);
    padding: 16px;
    border-radius: 22px;
    color: #e8f0ff;
    background:
      linear-gradient(145deg, rgba(13, 23, 41, 0.96), rgba(9, 16, 31, 0.92)),
      #0b1426;
    box-shadow: 0 20px 45px rgba(4, 10, 23, 0.38);
    border: 1px solid rgba(159, 190, 255, 0.14);
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .widget-card.compact {
    gap: 12px;
  }

  .widget-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 12px;
  }

  .widget-title {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .eyebrow {
    margin: 0 0 4px;
    font-size: 11px;
    letter-spacing: 0.18em;
    text-transform: uppercase;
    color: rgba(183, 206, 255, 0.66);
  }

  h1 {
    margin: 0;
    font-size: 24px;
    line-height: 1;
  }

  .widget-title small {
    color: rgba(215, 228, 255, 0.68);
    font-size: 12px;
  }

  .widget-actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .mode-toggle {
    display: inline-flex;
    padding: 3px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.08);
  }

  .mode-toggle button,
  .open-main {
    border: none;
    border-radius: 999px;
    padding: 8px 12px;
    font-size: 12px;
    cursor: pointer;
  }

  .mode-toggle button {
    padding: 6px 10px;
    color: rgba(220, 232, 255, 0.82);
    background: transparent;
  }

  .mode-toggle button.active {
    color: #081221;
    background: linear-gradient(135deg, #89f7fe, #66a6ff);
  }

  .open-main {
    color: #0f1b31;
    background: linear-gradient(135deg, #f6d365, #fda085);
  }

  .metrics-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 10px;
    flex: 1;
  }

  .compact .metrics-grid {
    gap: 8px;
  }

  .metric-card {
    padding: 14px;
    border-radius: 18px;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.08);
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    gap: 8px;
  }

  .compact .metric-card {
    padding: 12px;
    gap: 6px;
  }

  .metric-card strong {
    font-size: 24px;
    line-height: 1.05;
    font-weight: 700;
  }

  .compact .metric-card strong {
    font-size: 21px;
  }

  .metric-card small {
    color: rgba(215, 228, 255, 0.68);
    font-size: 12px;
  }

  .metric-label {
    font-size: 12px;
    color: rgba(196, 214, 255, 0.78);
  }

  .sparkline-wrap {
    height: 32px;
    margin-top: auto;
  }

  .sparkline-wrap svg {
    width: 100%;
    height: 100%;
    overflow: visible;
  }

  .sparkline-wrap polyline {
    fill: none;
    stroke: rgba(255, 255, 255, 0.95);
    stroke-width: 2.5;
    stroke-linecap: round;
    stroke-linejoin: round;
  }

  .cpu {
    background: linear-gradient(180deg, rgba(80, 150, 255, 0.16), rgba(255, 255, 255, 0.05));
  }

  .memory {
    background: linear-gradient(180deg, rgba(0, 201, 167, 0.16), rgba(255, 255, 255, 0.05));
  }

  .network {
    background: linear-gradient(180deg, rgba(255, 177, 66, 0.16), rgba(255, 255, 255, 0.05));
  }

  .upload {
    background: linear-gradient(180deg, rgba(246, 114, 128, 0.16), rgba(255, 255, 255, 0.05));
  }

  .widget-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    font-size: 11px;
    color: rgba(193, 207, 234, 0.62);
  }
</style>

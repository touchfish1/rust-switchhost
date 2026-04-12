<script lang="ts">
  import { onDestroy, onMount } from 'svelte'
  import { listen, type UnlistenFn } from '@tauri-apps/api/event'
  import { Window } from '@tauri-apps/api/window'
  import { getCurrentWindow } from '@tauri-apps/api/window'

  type MetricsSnapshot = {
    cpu_usage: number
    used_memory: number
    total_memory: number
    download_bytes_per_sec: number
    upload_bytes_per_sec: number
  }

  const currentWindow = getCurrentWindow()

  let metrics: MetricsSnapshot = {
    cpu_usage: 0,
    used_memory: 0,
    total_memory: 0,
    download_bytes_per_sec: 0,
    upload_bytes_per_sec: 0
  }

  let focusedUnlisten: UnlistenFn | null = null
  let metricsUnlisten: UnlistenFn | null = null
  let lastUpdated = '等待数据...'

  $: memoryPercent = metrics.total_memory > 0
    ? Math.round((metrics.used_memory / metrics.total_memory) * 100)
    : 0

  $: cpuValue = `${Math.round(metrics.cpu_usage)}%`
  $: memoryValue = `${formatBytes(metrics.used_memory)} / ${formatBytes(metrics.total_memory)}`
  $: downloadValue = formatRate(metrics.download_bytes_per_sec)
  $: uploadValue = formatRate(metrics.upload_bytes_per_sec)

  onMount(async () => {
    document.body.classList.add('tray-metrics-window')

    metricsUnlisten = await listen<MetricsSnapshot>('metrics-updated', (event) => {
      metrics = event.payload
      lastUpdated = formatTime(new Date())
    })

    focusedUnlisten = await currentWindow.onFocusChanged(({ payload: focused }) => {
      if (!focused) {
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

    if (focusedUnlisten) {
      focusedUnlisten()
      focusedUnlisten = null
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
  <div class="widget-card">
    <header class="widget-header" data-tauri-drag-region>
      <div>
        <p class="eyebrow">Tray Monitor</p>
        <h1>系统状态</h1>
      </div>
      <button class="open-main" type="button" on:click={openMainWindow}>
        主界面
      </button>
    </header>

    <div class="metrics-grid">
      <article class="metric-card cpu">
        <span class="metric-label">CPU</span>
        <strong>{cpuValue}</strong>
      </article>

      <article class="metric-card memory">
        <span class="metric-label">内存</span>
        <strong>{memoryPercent}%</strong>
        <small>{memoryValue}</small>
      </article>

      <article class="metric-card network">
        <span class="metric-label">下载</span>
        <strong>{downloadValue}</strong>
      </article>

      <article class="metric-card upload">
        <span class="metric-label">上传</span>
        <strong>{uploadValue}</strong>
      </article>
    </div>

    <footer class="widget-footer">
      <span>靠近托盘显示，失焦自动收起</span>
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

  .widget-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 12px;
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

  .open-main {
    border: none;
    border-radius: 999px;
    padding: 8px 12px;
    font-size: 12px;
    color: #0f1b31;
    background: linear-gradient(135deg, #f6d365, #fda085);
    cursor: pointer;
  }

  .metrics-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 10px;
    flex: 1;
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

  .metric-card strong {
    font-size: 24px;
    line-height: 1.05;
    font-weight: 700;
  }

  .metric-card small {
    color: rgba(215, 228, 255, 0.68);
    font-size: 12px;
  }

  .metric-label {
    font-size: 12px;
    color: rgba(196, 214, 255, 0.78);
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
    justify-content: space-between;
    gap: 12px;
    font-size: 11px;
    color: rgba(193, 207, 234, 0.62);
  }
</style>

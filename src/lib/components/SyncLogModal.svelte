<svelte:options runes={true} />

<script lang="ts">
  import type { SyncLogEntry } from '$lib/types'

  type SyncLogModalProps = {
    isOpen?: boolean
    schemeName?: string
    logs?: SyncLogEntry[]
    onClose?: (() => void) | undefined
  }

  let {
    isOpen = false,
    schemeName = '',
    logs = [],
    onClose = undefined
  }: SyncLogModalProps = $props()
  let filter = $state<'all' | 'success' | 'error'>('all')
  let copiedMessage = $state('')

  const filteredLogs = $derived(
    filter === 'all' ? logs : logs.filter((log) => log.status === filter)
  )

  function closeModal() {
    onClose?.()
  }

  function formatTrigger(trigger: string) {
    if (trigger === 'scheduled') return '定时同步'
    if (trigger === 'manual') return '手动同步'
    return trigger
  }

  async function copyLog(log: SyncLogEntry) {
    const payload = [
      `时间: ${new Date(log.timestamp).toLocaleString()}`,
      `状态: ${log.status}`,
      `触发: ${formatTrigger(log.trigger)}`,
      `消息: ${log.message}`
    ].join('\n')

    await navigator.clipboard.writeText(payload)
    copiedMessage = '已复制该条日志'
    setTimeout(() => {
      copiedMessage = ''
    }, 1800)
  }
</script>

{#if isOpen}
  <div
    class="modal-overlay"
    onclick={(event) => event.target === event.currentTarget && closeModal()}
    onkeydown={(event) => event.key === 'Escape' && closeModal()}
    role="dialog"
    aria-modal="true"
    aria-label="同步日志"
    tabindex="0"
  >
    <div class="modal" role="document">
      <div class="modal-header">
        <div>
          <h3>同步日志</h3>
          <p>{schemeName}</p>
        </div>
        <button class="close-btn" onclick={closeModal} aria-label="关闭">×</button>
      </div>

      <div class="modal-body">
        <div class="toolbar">
          <div class="filter-group">
            <button class:selected={filter === 'all'} onclick={() => (filter = 'all')}>全部</button>
            <button class:selected={filter === 'success'} onclick={() => (filter = 'success')}>成功</button>
            <button class:selected={filter === 'error'} onclick={() => (filter = 'error')}>失败</button>
          </div>
          {#if copiedMessage}
            <span class="copied-hint">{copiedMessage}</span>
          {/if}
        </div>

        {#if filteredLogs.length === 0}
          <div class="empty-state">暂无同步日志</div>
        {:else}
          {#each filteredLogs as log}
            <div class="log-item">
              <div class="log-head">
                <span class="log-time">{new Date(log.timestamp).toLocaleString()}</span>
                <div class="log-head-actions">
                  <span class:status-success={log.status === 'success'} class:status-error={log.status === 'error'} class="log-status">
                    {log.status === 'success' ? '成功' : '失败'}
                  </span>
                  <button class="copy-btn" onclick={() => copyLog(log)}>复制</button>
                </div>
              </div>
              <div class="log-trigger">{formatTrigger(log.trigger)}</div>
              <div class="log-message">{log.message}</div>
            </div>
          {/each}
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.45);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 2200;
    padding: 24px;
  }

  .modal {
    width: min(720px, 100%);
    max-height: min(760px, calc(100vh - 48px));
    background: var(--editor-bg, #ffffff);
    border-radius: 12px;
    box-shadow: 0 18px 60px rgba(0, 0, 0, 0.25);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .modal-header {
    padding: 16px 20px;
    border-bottom: 1px solid var(--border-color, #e0e0e0);
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 16px;
  }

  .modal-header h3 {
    margin: 0 0 4px 0;
    font-size: 18px;
    color: var(--text-primary, #213547);
  }

  .modal-header p {
    margin: 0;
    font-size: 13px;
    color: var(--text-secondary, #8c8c8c);
  }

  .close-btn {
    width: 36px;
    height: 36px;
    border: none;
    border-radius: 8px;
    background: transparent;
    color: var(--text-secondary, #8c8c8c);
    font-size: 24px;
    cursor: pointer;
  }

  .close-btn:hover {
    background: var(--hover-bg, #f0f0f0);
  }

  .modal-body {
    padding: 20px;
    overflow: auto;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .toolbar {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    align-items: center;
    flex-wrap: wrap;
  }

  .filter-group {
    display: flex;
    gap: 8px;
  }

  .filter-group button,
  .copy-btn {
    border: 1px solid var(--border-color, #e0e0e0);
    background: transparent;
    color: var(--text-primary, #213547);
    border-radius: 999px;
    padding: 6px 12px;
    font-size: 12px;
    cursor: pointer;
  }

  .filter-group button.selected {
    border-color: var(--primary-color, #1890ff);
    color: var(--primary-color, #1890ff);
    background: rgba(24, 144, 255, 0.08);
  }

  .copied-hint {
    font-size: 12px;
    color: #389e0d;
  }

  .empty-state {
    padding: 40px 20px;
    text-align: center;
    color: var(--text-secondary, #8c8c8c);
  }

  .log-item {
    padding: 14px 16px;
    border: 1px solid var(--border-color, #e0e0e0);
    border-radius: 10px;
    background: var(--hover-bg, #f8fbff);
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .log-head {
    display: flex;
    justify-content: space-between;
    gap: 16px;
    align-items: center;
  }

  .log-head-actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .log-time {
    font-size: 13px;
    color: var(--text-secondary, #8c8c8c);
  }

  .log-status {
    font-size: 12px;
    font-weight: 600;
  }

  .status-success {
    color: #52c41a;
  }

  .status-error {
    color: #ff4d4f;
  }

  .log-trigger {
    font-size: 12px;
    color: var(--primary-color, #1890ff);
    font-weight: 600;
  }

  .log-message {
    font-size: 14px;
    color: var(--text-primary, #213547);
    line-height: 1.6;
    word-break: break-word;
  }
</style>

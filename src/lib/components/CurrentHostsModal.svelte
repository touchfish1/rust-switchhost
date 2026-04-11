<script lang="ts">
  import Editor from './Editor.svelte'
  import { getManagedHostsBlockRange } from '$lib/utils/hosts-editor'
  import { toasts } from '$lib/stores/toasts'

  export let isOpen = false
  export let content = ''
  export let onClose: () => void

  $: managedBlockRange = getManagedHostsBlockRange(content)
  $: summaryText = managedBlockRange
    ? `已检测到软件托管区块，共 ${managedBlockRange.lineCount} 行，下面已高亮显示。`
    : '当前 Hosts 中还没有软件托管区块，说明尚未通过本软件写入规则。'

  async function copyCurrentHostsContent() {
    if (!content) {
      toasts.push('当前 Hosts 内容为空，暂时没有可复制的文本', 'warning')
      return
    }

    try {
      await navigator.clipboard.writeText(content)
      toasts.push('当前 Hosts 内容已复制到剪贴板', 'success')
    } catch (error) {
      console.error('Failed to copy current hosts content:', error)
      toasts.push('复制当前 Hosts 内容失败，请检查系统剪贴板权限', 'error')
    }
  }
</script>

{#if isOpen}
  <div
    class="hosts-modal-overlay"
    on:click|self={onClose}
    on:keydown={(event) => event.key === 'Escape' && onClose()}
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
        <div class="hosts-modal-actions">
          <button class="hosts-copy-btn" on:click={copyCurrentHostsContent} disabled={!content}>
            复制当前内容
          </button>
          <button class="hosts-close-btn" on:click={onClose} aria-label="关闭">×</button>
        </div>
      </div>

      <div class="hosts-modal-body">
        <div class="hosts-hint">
          {summaryText}
        </div>
        <Editor content={content} readOnly={true} managedBlockRange={managedBlockRange} />
      </div>
    </div>
  </div>
{/if}

<style>
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

  .hosts-modal-actions {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .hosts-copy-btn {
    height: 36px;
    padding: 0 14px;
    border-radius: 8px;
    border: 1px solid var(--border-color);
    background: var(--editor-bg);
    color: var(--text-primary);
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
  }

  .hosts-copy-btn:disabled {
    opacity: 0.65;
    cursor: not-allowed;
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
    display: flex;
    flex-direction: column;
  }

  .hosts-hint {
    padding: 12px 20px;
    border-bottom: 1px solid var(--border-color);
    background: rgba(24, 144, 255, 0.08);
    color: var(--text-primary);
    font-size: 13px;
    line-height: 1.6;
  }
</style>

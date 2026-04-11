<script lang="ts">
  import Editor from './Editor.svelte'
  import type { HostsAffectedDomain, HostsDiffSummary } from '$lib/types'
  import { toasts } from '$lib/stores/toasts'

  export let isOpen = false
  export let schemeName = ''
  export let remoteUrl = ''
  export let currentContent = ''
  export let remoteContent = ''
  export let diffSummary: HostsDiffSummary = { addedLines: 0, removedLines: 0, unchangedLines: 0 }
  export let affectedDomains: HostsAffectedDomain[] = []
  export let isLoading = false
  export let isApplying = false
  export let onClose: () => void
  export let onConfirm: () => void | Promise<void>

  $: conflictDomains = affectedDomains.filter((item) => item.isConflict)

  async function copyConflictDomains() {
    const content = conflictDomains.map((item) => item.domain).join('\n')
    if (!content) return

    try {
      await navigator.clipboard.writeText(content)
      toasts.push('冲突域名已复制到剪贴板', 'success')
    } catch (error) {
      console.error('Failed to copy conflict domains:', error)
      toasts.push('复制冲突域名失败，请检查系统剪贴板权限', 'error')
    }
  }
</script>

{#if isOpen}
  <div
    class="sync-preview-overlay"
    on:click|self={onClose}
    on:keydown={(event) => event.key === 'Escape' && !isApplying && onClose()}
    role="dialog"
    aria-modal="true"
    aria-label="同步前预览"
    tabindex="0"
  >
    <div class="sync-preview-modal" role="document">
      <div class="sync-preview-header">
        <div>
          <h3>同步前预览</h3>
          <p>{schemeName} · {remoteUrl}</p>
        </div>
        <button class="close-btn" on:click={onClose} aria-label="关闭" disabled={isApplying}>×</button>
      </div>

      <div class="sync-preview-body">
        {#if isLoading}
          <div class="loading-state">正在拉取远程内容并生成预览...</div>
        {:else}
          <div class="summary-grid">
            <div class="summary-card">
              <span>新增行数</span>
              <strong>{diffSummary.addedLines}</strong>
            </div>
            <div class="summary-card">
              <span>移除行数</span>
              <strong>{diffSummary.removedLines}</strong>
            </div>
            <div class="summary-card">
              <span>保留行数</span>
              <strong>{diffSummary.unchangedLines}</strong>
            </div>
          </div>

          {#if affectedDomains.length > 0}
            <div class="affected-box">
              <div class="affected-head">
                <strong>受影响域名</strong>
                {#if conflictDomains.length > 0}
                  <button class="copy-conflicts" on:click={copyConflictDomains}>
                    复制冲突域名
                  </button>
                {/if}
              </div>
              {#if conflictDomains.length > 0}
                <div class="conflict-note">
                  检测到 {conflictDomains.length} 个冲突域名：这些域名当前已有映射，远程同步后会改成新的 IP。
                </div>
              {/if}
              <div class="affected-list">
                {#each affectedDomains as item}
                  <span class={`affected-chip ${item.change}`} class:conflict={item.isConflict}>
                    {item.domain} · {item.change === 'added' ? '新增' : item.change === 'removed' ? '移除' : '更新'}
                  </span>
                {/each}
              </div>
            </div>
          {/if}

          <div class="preview-grid">
            <section class="preview-pane">
              <div class="preview-head">
                <strong>当前分组内容</strong>
              </div>
              <div class="preview-editor">
                <Editor content={currentContent} readOnly={true} />
              </div>
            </section>

            <section class="preview-pane">
              <div class="preview-head">
                <strong>远程最新内容</strong>
              </div>
              <div class="preview-editor">
                <Editor content={remoteContent} readOnly={true} />
              </div>
            </section>
          </div>
        {/if}
      </div>

      <div class="sync-preview-footer">
        <button class="btn-secondary" on:click={onClose} disabled={isApplying}>取消</button>
        <button class="btn-primary" on:click={onConfirm} disabled={isLoading || isApplying}>
          {isApplying ? '同步中...' : '确认同步并应用'}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .sync-preview-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.45);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 2250;
    padding: 24px;
  }

  .sync-preview-modal {
    width: min(1240px, 100%);
    height: min(820px, calc(100vh - 48px));
    background: var(--editor-bg);
    border-radius: 14px;
    box-shadow: 0 18px 60px rgba(0, 0, 0, 0.25);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .sync-preview-header,
  .sync-preview-footer {
    padding: 16px 20px;
    border-bottom: 1px solid var(--border-color);
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
  }

  .sync-preview-footer {
    border-bottom: none;
    border-top: 1px solid var(--border-color);
    justify-content: flex-end;
  }

  .sync-preview-header h3 {
    margin: 0 0 4px 0;
    font-size: 18px;
    color: var(--text-primary);
  }

  .sync-preview-header p {
    margin: 0;
    font-size: 12px;
    color: var(--text-secondary);
    word-break: break-all;
  }

  .sync-preview-body {
    flex: 1;
    min-height: 0;
    padding: 18px 20px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .summary-grid {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 12px;
  }

  .summary-card {
    padding: 14px 16px;
    border-radius: 12px;
    border: 1px solid var(--border-color);
    background: var(--sidebar-bg);
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .summary-card span {
    font-size: 12px;
    color: var(--text-secondary);
  }

  .summary-card strong {
    font-size: 20px;
    color: var(--text-primary);
  }

  .preview-grid {
    flex: 1;
    min-height: 0;
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 14px;
  }

  .affected-box {
    padding: 14px 16px;
    border-radius: 12px;
    border: 1px solid var(--border-color);
    background: var(--sidebar-bg);
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .affected-box strong {
    color: var(--text-primary);
    font-size: 13px;
  }

  .affected-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }

  .copy-conflicts {
    border: 1px solid var(--border-color);
    background: transparent;
    color: var(--text-primary);
    border-radius: 999px;
    padding: 6px 10px;
    font-size: 12px;
    cursor: pointer;
  }

  .conflict-note {
    font-size: 12px;
    color: #cf1322;
    line-height: 1.6;
  }

  .affected-list {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .affected-chip {
    display: inline-flex;
    align-items: center;
    min-height: 24px;
    padding: 0 10px;
    border-radius: 999px;
    font-size: 12px;
    border: 1px solid var(--border-color);
    color: var(--text-primary);
    background: var(--editor-bg);
  }

  .affected-chip.added {
    color: #389e0d;
    border-color: rgba(82, 196, 26, 0.28);
    background: rgba(82, 196, 26, 0.08);
  }

  .affected-chip.removed {
    color: #cf1322;
    border-color: rgba(255, 77, 79, 0.28);
    background: rgba(255, 77, 79, 0.08);
  }

  .affected-chip.updated {
    color: #0958d9;
    border-color: rgba(24, 144, 255, 0.28);
    background: rgba(24, 144, 255, 0.08);
  }

  .affected-chip.conflict {
    box-shadow: inset 0 0 0 1px rgba(255, 77, 79, 0.28);
  }

  .preview-pane {
    min-height: 0;
    display: flex;
    flex-direction: column;
    border: 1px solid var(--border-color);
    border-radius: 12px;
    overflow: hidden;
  }

  .preview-head {
    padding: 12px 14px;
    border-bottom: 1px solid var(--border-color);
    background: var(--sidebar-bg);
    color: var(--text-primary);
  }

  .preview-editor {
    flex: 1;
    min-height: 0;
  }

  .loading-state {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-secondary);
    font-size: 14px;
  }

  .close-btn,
  .btn-secondary,
  .btn-primary {
    cursor: pointer;
  }

  .close-btn {
    width: 36px;
    height: 36px;
    border: none;
    border-radius: 8px;
    background: transparent;
    color: var(--text-secondary);
    font-size: 24px;
  }

  .btn-secondary,
  .btn-primary {
    padding: 10px 16px;
    border-radius: 10px;
    font-size: 14px;
    font-weight: 600;
    border: 1px solid var(--border-color);
  }

  .btn-secondary {
    background: transparent;
    color: var(--text-primary);
  }

  .btn-primary {
    border-color: var(--primary-color);
    background: var(--primary-color);
    color: #fff;
  }

  .btn-secondary:disabled,
  .btn-primary:disabled,
  .close-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  @media (max-width: 900px) {
    .summary-grid,
    .preview-grid {
      grid-template-columns: 1fr;
    }
  }
</style>

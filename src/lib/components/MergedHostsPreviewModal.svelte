<script lang="ts">
  import Editor from './Editor.svelte'
  import type { HostsConflictGroup, HostsContentStats, HostsDiffSummary, Scheme } from '$lib/types'
  import { toasts } from '$lib/stores/toasts'

  export let isOpen = false
  export let currentContent = ''
  export let mergedContent = ''
  export let previewScopeLabel = ''
  export let note = ''
  export let sourceSchemes: Scheme[] = []
  export let currentStats: HostsContentStats
  export let mergedStats: HostsContentStats
  export let diffSummary: HostsDiffSummary
  export let conflicts: HostsConflictGroup[] = []
  export let onClose: () => void

  async function copyMergedContent() {
    try {
      await navigator.clipboard.writeText(mergedContent)
      toasts.push('合并结果已复制到剪贴板', 'success')
    } catch (error) {
      console.error('Failed to copy merged hosts content:', error)
      toasts.push('复制失败，请检查系统剪贴板权限', 'error')
    }
  }
</script>

{#if isOpen}
  <div
    class="preview-overlay"
    on:click|self={onClose}
    on:keydown={(event) => event.key === 'Escape' && onClose()}
    role="dialog"
    aria-modal="true"
    aria-label="合并预览"
    tabindex="0"
  >
    <div class="preview-modal" role="document">
      <div class="preview-header">
        <div>
          <h3>合并预览</h3>
          <p>{previewScopeLabel || '查看即将写入系统 Hosts 的合并结果'}</p>
        </div>
        <div class="preview-header-actions">
          <button class="preview-copy" type="button" on:click={copyMergedContent} disabled={!mergedContent}>
            复制合并结果
          </button>
          <button class="preview-close" type="button" on:click={onClose} aria-label="关闭">×</button>
        </div>
      </div>

      <div class="preview-summary">
        <div class="summary-card">
          <strong>当前 Hosts</strong>
          <span>{currentStats.lineCount} 行 / {currentStats.hostEntryCount} 条映射</span>
        </div>
        <div class="summary-card">
          <strong>预计生效</strong>
          <span>{mergedStats.lineCount} 行 / {mergedStats.hostEntryCount} 条映射</span>
        </div>
        <div class="summary-card">
          <strong>变化统计</strong>
          <span>+{diffSummary.addedLines} / -{diffSummary.removedLines} / ={diffSummary.unchangedLines}</span>
        </div>
        <div class="summary-card" class:warning={conflicts.length > 0}>
          <strong>冲突域名</strong>
          <span>{conflicts.length} 个</span>
        </div>
      </div>

      {#if note}
        <div class="preview-note">{note}</div>
      {/if}

      {#if sourceSchemes.length > 0}
        <div class="source-box">
          <strong>当前参与合并的分组</strong>
          <div class="source-list">
            {#each sourceSchemes as scheme}
              <span class="source-chip">{scheme.name}</span>
            {/each}
          </div>
        </div>
      {/if}

      {#if conflicts.length > 0}
        <div class="conflict-box">
          <div class="conflict-head">
            <strong>检测到域名冲突</strong>
            <span>同一个域名在不同分组中指向了不同 IP。下面会直接标出当前合并顺序下的最终生效结果。</span>
          </div>
          <div class="conflict-list">
            {#each conflicts as conflict}
              <div class="conflict-item">
                <div class="conflict-domain-row">
                  <strong>{conflict.domain}</strong>
                  <span class="conflict-result">
                    最终生效：{conflict.effectiveIp} · {conflict.winningSchemeName}
                  </span>
                </div>
                <div class="conflict-mappings">
                  {#each conflict.mappings as mapping}
                    <span class:selected={mapping.ip === conflict.effectiveIp}>
                      {mapping.ip} · {mapping.schemeNames.join(' / ')}
                    </span>
                  {/each}
                </div>
              </div>
            {/each}
          </div>
        </div>
      {/if}

      <div class="preview-body">
        <section class="preview-panel">
          <div class="preview-panel-head">
            <h4>当前系统 Hosts</h4>
            <span>用于对比真实文件内容</span>
          </div>
          <div class="preview-editor">
            <Editor content={currentContent} readOnly={true} />
          </div>
        </section>

        <section class="preview-panel">
          <div class="preview-panel-head">
            <h4>预计合并结果</h4>
            <span>这是启用分组最终写入的内容</span>
          </div>
          <div class="preview-editor">
            <Editor content={mergedContent} readOnly={true} />
          </div>
        </section>
      </div>
    </div>
  </div>
{/if}

<style>
  .preview-overlay {
    position: fixed;
    inset: 0;
    padding: 24px;
    background: rgba(0, 0, 0, 0.45);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 2300;
  }

  .preview-modal {
    width: min(1280px, 100%);
    height: min(860px, calc(100vh - 48px));
    background: var(--editor-bg);
    border-radius: 14px;
    box-shadow: 0 24px 80px rgba(0, 0, 0, 0.28);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .preview-header {
    padding: 18px 22px;
    border-bottom: 1px solid var(--border-color);
    display: flex;
    justify-content: space-between;
    gap: 16px;
    align-items: flex-start;
  }

  .preview-header h3 {
    margin: 0 0 4px;
    color: var(--text-primary);
    font-size: 18px;
  }

  .preview-header p {
    margin: 0;
    color: var(--text-secondary);
    font-size: 13px;
  }

  .preview-header-actions {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .preview-copy {
    height: 36px;
    padding: 0 14px;
    border-radius: 8px;
    border: 1px solid var(--border-color);
    background: var(--sidebar-bg);
    color: var(--text-primary);
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
  }

  .preview-copy:hover:not(:disabled) {
    border-color: var(--primary-color);
    color: var(--primary-color);
    background: var(--hover-bg);
  }

  .preview-copy:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .preview-close {
    width: 36px;
    height: 36px;
    border: none;
    border-radius: 8px;
    background: transparent;
    color: var(--text-secondary);
    font-size: 24px;
    cursor: pointer;
  }

  .preview-close:hover {
    background: var(--hover-bg);
    color: var(--text-primary);
  }

  .preview-summary {
    padding: 18px 22px 0;
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 12px;
  }

  .summary-card {
    padding: 12px 14px;
    border-radius: 12px;
    border: 1px solid var(--border-color);
    background: var(--sidebar-bg);
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .summary-card strong {
    font-size: 13px;
    color: var(--text-primary);
  }

  .summary-card span {
    font-size: 12px;
    color: var(--text-secondary);
  }

  .summary-card.warning {
    border-color: rgba(255, 77, 79, 0.35);
    background: rgba(255, 77, 79, 0.08);
  }

  .preview-note {
    margin: 14px 22px 0;
    padding: 10px 12px;
    border-radius: 10px;
    border: 1px solid var(--border-color);
    background: var(--hover-bg);
    color: var(--text-secondary);
    font-size: 13px;
  }

  .source-box {
    margin: 14px 22px 0;
    padding: 12px 14px;
    border-radius: 12px;
    border: 1px solid var(--border-color);
    background: var(--sidebar-bg);
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .source-box strong {
    font-size: 13px;
    color: var(--text-primary);
  }

  .source-list {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .source-chip {
    display: inline-flex;
    align-items: center;
    min-height: 28px;
    padding: 0 12px;
    border-radius: 999px;
    background: var(--editor-bg);
    border: 1px solid var(--border-color);
    color: var(--text-secondary);
    font-size: 12px;
    font-weight: 600;
  }

  .conflict-box {
    margin: 14px 22px 0;
    padding: 14px;
    border-radius: 12px;
    border: 1px solid rgba(255, 77, 79, 0.28);
    background: rgba(255, 77, 79, 0.08);
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .conflict-head {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .conflict-head strong {
    color: #cf1322;
    font-size: 14px;
  }

  .conflict-head span {
    color: var(--text-secondary);
    font-size: 12px;
    line-height: 1.6;
  }

  .conflict-list {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 10px;
    overflow: auto;
  }

  .conflict-item {
    padding: 12px;
    border-radius: 10px;
    background: var(--editor-bg);
    border: 1px solid rgba(255, 77, 79, 0.16);
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .conflict-domain-row {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .conflict-item strong {
    color: var(--text-primary);
    font-size: 13px;
  }

  .conflict-result {
    color: #cf1322;
    font-size: 12px;
    font-weight: 600;
  }

  .conflict-mappings {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .conflict-mappings span {
    color: var(--text-secondary);
    font-size: 12px;
    word-break: break-word;
  }

  .conflict-mappings span.selected {
    color: #cf1322;
    font-weight: 600;
  }

  .preview-body {
    flex: 1;
    min-height: 0;
    padding: 18px 22px 22px;
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 16px;
  }

  .preview-panel {
    min-height: 0;
    display: flex;
    flex-direction: column;
    border: 1px solid var(--border-color);
    border-radius: 12px;
    overflow: hidden;
    background: var(--sidebar-bg);
  }

  .preview-panel-head {
    padding: 14px 16px;
    border-bottom: 1px solid var(--border-color);
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .preview-panel-head h4 {
    margin: 0;
    font-size: 14px;
    color: var(--text-primary);
  }

  .preview-panel-head span {
    font-size: 12px;
    color: var(--text-secondary);
  }

  .preview-editor {
    flex: 1;
    min-height: 0;
    background: var(--editor-bg);
  }

  @media (max-width: 1100px) {
    .preview-summary,
    .preview-body,
    .conflict-list {
      grid-template-columns: 1fr;
    }
  }

  @media (max-width: 720px) {
    .preview-header {
      flex-direction: column;
    }

    .preview-header-actions {
      width: 100%;
      justify-content: space-between;
    }
  }
</style>

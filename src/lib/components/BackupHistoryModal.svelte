<script lang="ts">
  import type { HostsBackupEntry } from '$lib/types'

  export let isOpen = false
  export let backups: HostsBackupEntry[] = []
  export let selectedBackupPath = ''
  export let selectedBackupContent = ''
  export let isLoadingContent = false
  export let isRestoring = false
  export let onClose: () => void
  export let onSelectBackup: (path: string) => void | Promise<void>
  export let onRestoreBackup: (path: string) => void | Promise<void>

  function handleClose() {
    onClose()
  }

  function formatSize(bytes: number) {
    if (bytes < 1024) return `${bytes} B`
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
    return `${(bytes / 1024 / 1024).toFixed(2)} MB`
  }
</script>

{#if isOpen}
  <div
    class="backup-modal-overlay"
    on:click|self={handleClose}
    on:keydown={(event) => event.key === 'Escape' && handleClose()}
    role="dialog"
    aria-modal="true"
    aria-label="Hosts 备份恢复"
    tabindex="0"
  >
    <div class="backup-modal" role="document">
      <div class="backup-modal-header">
        <div>
          <h3>备份恢复</h3>
          <p>查看自动备份的 hosts 历史版本，并在需要时一键恢复。</p>
        </div>
        <button class="close-btn" on:click={handleClose} aria-label="关闭">×</button>
      </div>

      <div class="backup-modal-body">
        <aside class="backup-list">
          <div class="backup-list-head">
            <strong>最近备份</strong>
            <span>{backups.length} 条</span>
          </div>

          {#if backups.length === 0}
            <div class="empty-state">暂时还没有可恢复的备份</div>
          {:else}
            {#each backups as backup (backup.path)}
              <button
                class="backup-item"
                class:selected={selectedBackupPath === backup.path}
                on:click={() => onSelectBackup(backup.path)}
                disabled={isRestoring}
              >
                <strong>{backup.filename}</strong>
                <span>{new Date(backup.created_at).toLocaleString()}</span>
                <div class="backup-meta">
                  <small>{backup.host_entry_count} 条规则</small>
                  <small>{backup.comment_count} 条注释</small>
                  <small>{backup.line_count} 行</small>
                </div>
                <small>{formatSize(backup.size_bytes)}</small>
              </button>
            {/each}
          {/if}
        </aside>

        <section class="backup-preview">
          <div class="backup-preview-head">
            <div>
              <strong>内容预览</strong>
              <span>
                {#if selectedBackupPath}
                  已选择一个备份版本
                {:else}
                  请选择左侧备份查看内容
                {/if}
              </span>
            </div>

            <button
              class="restore-btn"
              on:click={() => onRestoreBackup(selectedBackupPath)}
              disabled={!selectedBackupPath || isLoadingContent || isRestoring}
            >
              {isRestoring ? '恢复中...' : '恢复这个版本'}
            </button>
          </div>

          {#if isLoadingContent}
            <div class="preview-loading">正在读取备份内容...</div>
          {:else if selectedBackupContent}
            <pre>{selectedBackupContent}</pre>
          {:else}
            <div class="preview-placeholder">选择一个备份后，这里会显示对应的 hosts 内容。</div>
          {/if}
        </section>
      </div>
    </div>
  </div>
{/if}

<style>
  .backup-modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.45);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 2200;
    padding: 24px;
  }

  .backup-modal {
    width: min(1180px, 100%);
    height: min(760px, calc(100vh - 48px));
    background: var(--editor-bg);
    border-radius: 14px;
    box-shadow: 0 18px 60px rgba(0, 0, 0, 0.25);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .backup-modal-header {
    padding: 16px 20px;
    border-bottom: 1px solid var(--border-color);
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 16px;
  }

  .backup-modal-header h3 {
    margin: 0 0 4px 0;
    font-size: 18px;
    color: var(--text-primary);
  }

  .backup-modal-header p {
    margin: 0;
    font-size: 13px;
    color: var(--text-secondary);
  }

  .close-btn {
    width: 36px;
    height: 36px;
    border: none;
    border-radius: 8px;
    background: transparent;
    color: var(--text-secondary);
    font-size: 24px;
    line-height: 1;
    cursor: pointer;
  }

  .close-btn:hover {
    background: var(--hover-bg);
    color: var(--text-primary);
  }

  .backup-modal-body {
    flex: 1;
    min-height: 0;
    display: grid;
    grid-template-columns: 320px 1fr;
  }

  .backup-list {
    border-right: 1px solid var(--border-color);
    padding: 18px;
    overflow: auto;
    display: flex;
    flex-direction: column;
    gap: 12px;
    background: var(--sidebar-bg);
  }

  .backup-list-head,
  .backup-preview-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }

  .backup-list-head strong,
  .backup-preview-head strong {
    color: var(--text-primary);
    font-size: 14px;
  }

  .backup-list-head span,
  .backup-preview-head span {
    color: var(--text-secondary);
    font-size: 12px;
  }

  .backup-item {
    border: 1px solid var(--border-color);
    background: var(--editor-bg);
    color: var(--text-primary);
    border-radius: 12px;
    padding: 14px;
    display: flex;
    flex-direction: column;
    gap: 6px;
    text-align: left;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .backup-item:hover,
  .backup-item.selected {
    border-color: var(--primary-color);
    background: var(--hover-bg);
  }

  .backup-item strong {
    font-size: 13px;
  }

  .backup-item span,
  .backup-item small {
    color: var(--text-secondary);
    font-size: 12px;
  }

  .backup-meta {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .backup-preview {
    padding: 18px;
    display: flex;
    flex-direction: column;
    gap: 14px;
    min-width: 0;
  }

  .restore-btn {
    padding: 9px 14px;
    border: none;
    border-radius: 8px;
    background: var(--primary-color);
    color: #fff;
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
  }

  .restore-btn:disabled,
  .backup-item:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .backup-preview pre,
  .preview-loading,
  .preview-placeholder,
  .empty-state {
    margin: 0;
    border-radius: 12px;
    border: 1px solid var(--border-color);
    background: var(--sidebar-bg);
    color: var(--text-primary);
    padding: 16px;
    font-size: 13px;
    line-height: 1.65;
  }

  .backup-preview pre {
    flex: 1;
    min-height: 0;
    overflow: auto;
    white-space: pre-wrap;
    word-break: break-word;
    font-family: var(--font-family-mono);
    background: var(--editor-bg);
  }

  .preview-loading,
  .preview-placeholder,
  .empty-state {
    color: var(--text-secondary);
  }

  @media (max-width: 860px) {
    .backup-modal-body {
      grid-template-columns: 1fr;
    }

    .backup-list {
      border-right: none;
      border-bottom: 1px solid var(--border-color);
      max-height: 260px;
    }

    .backup-preview-head {
      align-items: flex-start;
      flex-direction: column;
    }
  }
</style>

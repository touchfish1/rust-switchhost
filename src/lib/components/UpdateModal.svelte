<script lang="ts">
  import type { UpdateInfo, UpdaterHandle } from '$lib/types'
  import { toasts } from '$lib/stores/toasts'

  export let isOpen = false
  export let updateInfo: UpdateInfo
  export let availableUpdate: UpdaterHandle = null
  export let isInstallingUpdate = false
  export let updateProgressText = ''
  export let updateProgressValue: number | null = null
  export let formatPublishedAt: (value: string) => string
  export let onClose: () => void
  export let onOpenUrl: (url: string) => void | Promise<void>
  export let onInstall: () => void | Promise<void>
  export let onCopyDiagnostic: () => void | Promise<void>

  async function copyReleaseInfo() {
    const payload = [
      `当前版本: ${updateInfo.current_version}`,
      `最新版本: ${updateInfo.latest_version}`,
      `版本标题: ${updateInfo.release_name}`,
      `发布时间: ${formatPublishedAt(updateInfo.published_at)}`,
      `发布页: ${updateInfo.html_url}`,
      updateInfo.download_url ? `推荐下载: ${updateInfo.download_url}` : '',
      '',
      '发布说明:',
      updateInfo.body || '暂无发布说明'
    ]
      .filter(Boolean)
      .join('\n')

    try {
      await navigator.clipboard.writeText(payload)
      toasts.push('版本信息已复制到剪贴板', 'success')
    } catch (error) {
      console.error('Failed to copy release info:', error)
      toasts.push('复制版本信息失败，请检查系统剪贴板权限', 'error')
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
    aria-label="检查更新"
    tabindex="0"
  >
    <div class="update-modal" role="document">
      <div class="hosts-modal-header">
        <div>
          <h3>在线升级</h3>
          <p>当前版本 {updateInfo.current_version} · 最新版本 {updateInfo.latest_version}</p>
        </div>
        <button class="hosts-close-btn" on:click={onClose} aria-label="关闭">×</button>
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
            <div class="meta-action-group">
              <button class="link-btn" on:click={copyReleaseInfo}>
                复制版本信息
              </button>
              <button class="link-btn" on:click={() => onOpenUrl(updateInfo.html_url)}>
                打开 GitHub Release
              </button>
            </div>
          </div>
          {#if updateInfo.download_url}
            <div class="update-meta-row">
              <span>推荐下载</span>
              <button class="link-btn" on:click={() => onOpenUrl(updateInfo.download_url!)}>
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
              on:click={() => onOpenUrl(updateInfo.html_url)}
              disabled={isInstallingUpdate}
            >
              查看发布页
            </button>
            <button
              class="btn-secondary"
              on:click={onCopyDiagnostic}
              disabled={isInstallingUpdate}
            >
              复制诊断信息
            </button>
            <button class="btn-primary" on:click={onInstall} disabled={isInstallingUpdate}>
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
            {#if updateProgressValue !== null}
              <div class="update-progress-head">
                <strong>下载进度</strong>
                <span>{Math.round(updateProgressValue)}%</span>
              </div>
              <div class="update-progress-bar" aria-hidden="true">
                <div
                  class="update-progress-fill"
                  style={`width: ${Math.max(0, Math.min(100, updateProgressValue))}%`}
                ></div>
              </div>
            {/if}
            <div class="update-progress-text">{updateProgressText}</div>
          </div>
        {/if}
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

  .meta-action-group {
    display: flex;
    align-items: center;
    gap: 14px;
    flex-wrap: wrap;
    justify-content: flex-end;
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
    font-family: var(--font-family-mono);
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
    padding: 14px;
    border-radius: 10px;
    background: var(--hover-bg);
    border: 1px solid var(--border-color);
    color: var(--text-primary);
    font-size: 13px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .update-progress-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }

  .update-progress-head strong,
  .update-progress-head span {
    color: var(--text-primary);
  }

  .update-progress-head span {
    font-weight: 700;
  }

  .update-progress-bar {
    height: 10px;
    border-radius: 999px;
    background: rgba(22, 119, 255, 0.12);
    overflow: hidden;
  }

  .update-progress-fill {
    height: 100%;
    border-radius: inherit;
    background: linear-gradient(90deg, #1677ff 0%, #36cfc9 100%);
    transition: width 0.2s ease;
  }

  .update-progress-text {
    line-height: 1.6;
    word-break: break-word;
  }

  @media (max-width: 640px) {
    .update-meta-row,
    .update-actions {
      flex-direction: column;
      align-items: flex-start;
    }

    .meta-action-group {
      justify-content: flex-start;
    }
  }
</style>

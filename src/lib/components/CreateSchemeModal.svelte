<script lang="ts">
  import { createEventDispatcher } from 'svelte'

  export let isOpen = false
  export let isSubmitting = false

  const dispatch = createEventDispatcher()

  let name = ''
  let schemeType: 'local' | 'remote' = 'local'
  let remoteUrl = ''
  let autoSyncEnabled = true
  let syncIntervalMinutes = '15'

  $: if (isOpen) {
    name = ''
    schemeType = 'local'
    remoteUrl = ''
    autoSyncEnabled = true
    syncIntervalMinutes = '15'
  }

  function handleClose() {
    dispatch('close')
  }

  function handleConfirm() {
    dispatch('confirm', {
      name: name.trim(),
      type: schemeType,
      remoteUrl: remoteUrl.trim(),
      autoSyncEnabled,
      syncIntervalMinutes: syncIntervalMinutes.trim()
    })
  }
</script>

{#if isOpen}
  <div
    class="modal-overlay"
    on:click|self={handleClose}
    on:keydown={(event) => event.key === 'Escape' && handleClose()}
    role="dialog"
    aria-modal="true"
    aria-label="创建新分组"
    tabindex="0"
  >
    <div class="modal" role="document">
      <div class="modal-header">
        <div>
          <h3>创建新分组</h3>
          <p>支持创建普通分组或远程 URL 分组</p>
        </div>
        <button class="close-btn" on:click={handleClose} aria-label="关闭">×</button>
      </div>

      <div class="modal-body">
        <label class="form-group">
          <span>分组名称</span>
          <input
            type="text"
            bind:value={name}
            placeholder="例如：开发环境 / 公司代理 / GitHub 镜像"
            disabled={isSubmitting}
          />
        </label>

        <div class="form-group">
          <span>分组类型</span>
          <div class="type-grid">
            <button
              class="type-card"
              class:selected={schemeType === 'local'}
              type="button"
              on:click={() => (schemeType = 'local')}
              disabled={isSubmitting}
            >
              <strong>本地分组</strong>
              <small>手动编辑 hosts 内容</small>
            </button>

            <button
              class="type-card"
              class:selected={schemeType === 'remote'}
              type="button"
              on:click={() => (schemeType = 'remote')}
              disabled={isSubmitting}
            >
              <strong>远程 URL</strong>
              <small>首次创建后自动拉取并可定时同步</small>
            </button>
          </div>
        </div>

        {#if schemeType === 'remote'}
          <label class="form-group">
            <span>远程 URL</span>
            <input
              type="url"
              bind:value={remoteUrl}
              placeholder="https://example.com/hosts.txt"
              disabled={isSubmitting}
            />
          </label>

          <label class="checkbox-row">
            <input type="checkbox" bind:checked={autoSyncEnabled} disabled={isSubmitting} />
            <span>启用定时同步</span>
          </label>

          <label class="form-group">
            <span>同步间隔（分钟）</span>
            <input
              type="number"
              min="1"
              step="1"
              bind:value={syncIntervalMinutes}
              placeholder="例如 15"
              disabled={isSubmitting || !autoSyncEnabled}
            />
          </label>

          <div class="tip-box">
            创建后会先保存远程配置，再立即拉取一次内容。分组启用时，后续同步成功会自动应用到系统 Hosts。
          </div>
        {/if}
      </div>

      <div class="modal-footer">
        <button class="btn btn-default" type="button" on:click={handleClose} disabled={isSubmitting}>
          取消
        </button>
        <button class="btn btn-primary" type="button" on:click={handleConfirm} disabled={isSubmitting}>
          {isSubmitting ? '创建中...' : '创建分组'}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 2200;
    padding: 24px;
  }

  .modal {
    width: min(620px, 100%);
    background: var(--editor-bg, #ffffff);
    border-radius: 14px;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.25);
    overflow: hidden;
  }

  .modal-header {
    padding: 18px 22px;
    border-bottom: 1px solid var(--border-color, #e0e0e0);
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 16px;
  }

  .modal-header h3 {
    margin: 0 0 4px 0;
    color: var(--text-primary, #213547);
    font-size: 18px;
  }

  .modal-header p {
    margin: 0;
    color: var(--text-secondary, #8c8c8c);
    font-size: 13px;
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
    padding: 22px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .form-group span,
  .checkbox-row span {
    color: var(--text-primary, #213547);
    font-size: 14px;
    font-weight: 500;
  }

  .form-group input {
    width: 100%;
    box-sizing: border-box;
    padding: 10px 12px;
    border: 1px solid var(--border-color, #e0e0e0);
    border-radius: 8px;
    background: var(--editor-bg, #ffffff);
    color: var(--text-primary, #213547);
    font-size: 14px;
  }

  .form-group input:focus {
    outline: none;
    border-color: var(--primary-color, #1890ff);
  }

  .type-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 12px;
  }

  .type-card {
    padding: 14px 16px;
    border-radius: 12px;
    border: 1px solid var(--border-color, #e0e0e0);
    background: var(--editor-bg, #ffffff);
    color: var(--text-primary, #213547);
    cursor: pointer;
    display: flex;
    flex-direction: column;
    gap: 6px;
    text-align: left;
    transition: all 0.2s ease;
  }

  .type-card strong {
    font-size: 15px;
  }

  .type-card small {
    font-size: 12px;
    color: var(--text-secondary, #8c8c8c);
  }

  .type-card:hover,
  .type-card.selected {
    border-color: var(--primary-color, #1890ff);
    background: var(--hover-bg, #e6f7ff);
  }

  .checkbox-row {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .tip-box {
    padding: 12px 14px;
    border-radius: 10px;
    background: var(--hover-bg, #e6f7ff);
    border: 1px solid var(--border-color, #e0e0e0);
    color: var(--text-secondary, #8c8c8c);
    font-size: 13px;
    line-height: 1.6;
  }

  .modal-footer {
    padding: 16px 22px 20px;
    display: flex;
    justify-content: flex-end;
    gap: 12px;
    border-top: 1px solid var(--border-color, #e0e0e0);
    background: var(--sidebar-bg, #f5f5f5);
  }

  .btn {
    padding: 10px 16px;
    border-radius: 8px;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    border: none;
  }

  .btn-default {
    background: var(--editor-bg, #ffffff);
    border: 1px solid var(--border-color, #e0e0e0);
    color: var(--text-primary, #213547);
  }

  .btn-primary {
    background: var(--primary-color, #1890ff);
    color: white;
  }

  .btn:disabled,
  .type-card:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  @media (max-width: 640px) {
    .type-grid {
      grid-template-columns: 1fr;
    }
  }
</style>

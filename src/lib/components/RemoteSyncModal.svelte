<script lang="ts">
  import { createEventDispatcher } from 'svelte'

  export let isOpen = false
  export let schemeName = ''
  export let remoteUrl = ''
  export let autoSyncEnabled = false
  export let syncIntervalMinutes = ''
  export let lastSyncedAt = ''
  export let lastSyncError = ''
  export let isSaving = false
  export let isSyncing = false

  const dispatch = createEventDispatcher()

  let localUrl = ''
  let localAutoSyncEnabled = false
  let localInterval = ''

  $: if (isOpen) {
    localUrl = remoteUrl
    localAutoSyncEnabled = autoSyncEnabled
    localInterval = syncIntervalMinutes
  }

  function handleClose() {
    dispatch('close')
  }

  function handleSave() {
    dispatch('save', {
      remoteUrl: localUrl.trim(),
      autoSyncEnabled: localAutoSyncEnabled,
      syncIntervalMinutes: localInterval.trim()
    })
  }

  function handleSyncNow() {
    dispatch('sync')
  }
</script>

{#if isOpen}
  <div
    class="modal-overlay"
    on:click|self={handleClose}
    on:keydown={(event) => event.key === 'Escape' && handleClose()}
    role="dialog"
    aria-modal="true"
    aria-label="远程同步设置"
    tabindex="0"
  >
    <div class="modal" role="document">
      <div class="modal-header">
        <div>
          <h3>远程同步设置</h3>
          <p>{schemeName || '当前分组'}</p>
        </div>
        <button class="close-btn" on:click={handleClose} aria-label="关闭">×</button>
      </div>

      <div class="modal-body">
        <label class="form-group">
          <span>远程 URL</span>
          <input
            type="url"
            bind:value={localUrl}
            placeholder="https://example.com/hosts"
            disabled={isSaving || isSyncing}
          />
        </label>

        <label class="checkbox-row">
          <input
            type="checkbox"
            bind:checked={localAutoSyncEnabled}
            disabled={isSaving || isSyncing || !localUrl.trim()}
          />
          <span>启用定时同步</span>
        </label>

        <label class="form-group">
          <span>同步间隔（分钟）</span>
          <input
            type="number"
            min="1"
            step="1"
            bind:value={localInterval}
            placeholder="例如 15"
            disabled={isSaving || isSyncing || !localAutoSyncEnabled}
          />
        </label>

        <div class="info-card">
          <span>说明：每个远程 URL 对应一个分组。分组已启用时，远程同步成功后会立即应用到系统 Hosts。</span>
        </div>

        {#if lastSyncedAt}
          <div class="status-line">
            最近同步时间：{lastSyncedAt}
          </div>
        {/if}

        {#if lastSyncError}
          <div class="error-message">
            最近同步失败：{lastSyncError}
          </div>
        {/if}
      </div>

      <div class="modal-footer">
        <button
          class="btn btn-default"
          on:click={handleSyncNow}
          disabled={isSaving || isSyncing || !localUrl.trim()}
        >
          {isSyncing ? '同步中...' : '立即同步'}
        </button>
        <button class="btn btn-default" on:click={handleClose} disabled={isSaving || isSyncing}>
          取消
        </button>
        <button class="btn btn-primary" on:click={handleSave} disabled={isSaving || isSyncing}>
          {isSaving ? '保存中...' : '保存设置'}
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
    width: min(560px, 100%);
    background: var(--editor-bg, #ffffff);
    border-radius: 12px;
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
    gap: 14px;
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

  .form-group input[type='url'],
  .form-group input[type='number'] {
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

  .checkbox-row {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .info-card {
    padding: 12px 14px;
    border-radius: 10px;
    background: var(--hover-bg, #e6f7ff);
    border: 1px solid var(--border-color, #e0e0e0);
    color: var(--text-secondary, #8c8c8c);
    font-size: 13px;
    line-height: 1.6;
  }

  .status-line {
    color: var(--text-secondary, #8c8c8c);
    font-size: 13px;
  }

  .error-message {
    padding: 12px 14px;
    border-radius: 10px;
    background: #fff2f0;
    border: 1px solid #ffccc7;
    color: var(--danger-color, #ff4d4f);
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

  .btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
</style>

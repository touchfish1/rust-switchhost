<script lang="ts">
  import { createEventDispatcher } from 'svelte'
  
  export let isOpen: boolean = false
  
  const dispatch = createEventDispatcher()
  
  let url: string = ''
  let isLoading: boolean = false
  let error: string | null = null
  
  async function handleFetch() {
    if (!url.trim()) {
      error = '请输入 URL'
      return
    }
    
    try {
      isLoading = true
      error = null
      dispatch('fetch', { url })
    } catch (e) {
      error = `获取失败: ${e}`
    } finally {
      isLoading = false
    }
  }
  
  function handleClose() {
    dispatch('close')
  }
</script>

{#if isOpen}
  <div class="modal-overlay" on:click={handleClose}>
    <div class="modal" on:click|e| e.stopPropagation()>
      <div class="modal-header">
        <h3>从远程 URL 获取 hosts</h3>
        <button class="close-btn" on:click={handleClose}>×</button>
      </div>
      
      <div class="modal-body">
        <div class="form-group">
          <label for="url">URL 地址</label>
          <input
            type="url"
            id="url"
            bind:value={url}
            placeholder="https://example.com/hosts"
            disabled={isLoading}
          />
          <p class="hint">
            输入包含 hosts 内容的 URL，例如 GitHub Gist 或其他在线 hosts 文件
          </p>
        </div>
        
        {#if error}
          <div class="error-message">
            {error}
          </div>
        {/if}
      </div>
      
      <div class="modal-footer">
        <button
          class="btn-secondary"
          on:click={handleClose}
          disabled={isLoading}
        >
          取消
        </button>
        <button
          class="btn-primary"
          on:click={handleFetch}
          disabled={isLoading || !url.trim()}
        >
          {isLoading ? '获取中...' : '获取'}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 2000;
  }
  
  .modal {
    background: var(--editor-bg, #ffffff);
    border-radius: 8px;
    box-shadow: 0 4px 24px rgba(0, 0, 0, 0.15);
    width: 90%;
    max-width: 500px;
    overflow: hidden;
  }
  
  .modal-header {
    padding: 16px 20px;
    border-bottom: 1px solid var(--border-color, #e0e0e0);
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  
  .modal-header h3 {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
    color: var(--text-primary, #213547);
  }
  
  .close-btn {
    width: 32px;
    height: 32px;
    border: none;
    background: transparent;
    font-size: 24px;
    cursor: pointer;
    color: var(--text-secondary, #8c8c8c);
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 4px;
    transition: background-color 0.2s;
  }
  
  .close-btn:hover {
    background: var(--hover-bg, #f0f0f0);
  }
  
  .modal-body {
    padding: 20px;
  }
  
  .form-group {
    margin-bottom: 16px;
  }
  
  .form-group label {
    display: block;
    margin-bottom: 8px;
    font-size: 14px;
    font-weight: 500;
    color: var(--text-primary, #213547);
  }
  
  .form-group input {
    width: 100%;
    padding: 10px 12px;
    border: 1px solid var(--border-color, #e0e0e0);
    border-radius: 6px;
    font-size: 14px;
    background: var(--editor-bg, #ffffff);
    color: var(--text-primary, #213547);
    transition: border-color 0.2s;
  }
  
  .form-group input:focus {
    outline: none;
    border-color: var(--primary-color, #1890ff);
  }
  
  .form-group input:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
  
  .hint {
    margin: 8px 0 0 0;
    font-size: 12px;
    color: var(--text-secondary, #8c8c8c);
  }
  
  .error-message {
    padding: 12px;
    background: #fff2f0;
    border: 1px solid #ffccc7;
    border-radius: 6px;
    color: var(--danger-color, #ff4d4f);
    font-size: 13px;
  }
  
  .modal-footer {
    padding: 16px 20px;
    border-top: 1px solid var(--border-color, #e0e0e0);
    display: flex;
    justify-content: flex-end;
    gap: 12px;
  }
  
  .btn-secondary {
    padding: 8px 16px;
    border-radius: 6px;
    border: 1px solid var(--border-color, #e0e0e0);
    background: transparent;
    color: var(--text-primary, #213547);
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
  }
  
  .btn-secondary:hover:not(:disabled) {
    background: var(--hover-bg, #f0f0f0);
  }
  
  .btn-secondary:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
  
  .btn-primary {
    padding: 8px 16px;
    border-radius: 6px;
    border: none;
    background: var(--primary-color, #1890ff);
    color: white;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: background-color 0.2s;
  }
  
  .btn-primary:hover:not(:disabled) {
    background: var(--primary-hover, #40a9ff);
  }
  
  .btn-primary:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
</style>

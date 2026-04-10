<script lang="ts">
  import { onMount } from 'svelte'

  type ModalProps = {
    title?: string
    confirmText?: string
    cancelText?: string
    showCancel?: boolean
    type?: 'default' | 'danger'
    inputValue?: string
    onConfirm?: (detail: { value: string }) => void
    onCancel?: () => void
    onClose?: () => void
  }

  export let title: ModalProps['title'] = ''
  export let confirmText: ModalProps['confirmText'] = '确定'
  export let cancelText: ModalProps['cancelText'] = '取消'
  export let showCancel: ModalProps['showCancel'] = true
  export let type: ModalProps['type'] = 'default'
  export let inputValue: ModalProps['inputValue'] = ''
  export let onConfirm: NonNullable<ModalProps['onConfirm']> = () => {}
  export let onCancel: NonNullable<ModalProps['onCancel']> = () => {}
  export let onClose: NonNullable<ModalProps['onClose']> = () => {}

  let showModal = false
  let localInputValue = inputValue

  onMount(() => {
    showModal = true
  })

  function handleConfirm() {
    onConfirm({ value: localInputValue })
    closeModal()
  }

  function handleCancel() {
    onCancel()
    closeModal()
  }

  function closeModal() {
    showModal = false
    setTimeout(() => {
      onClose()
    }, 200)
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      handleCancel()
    } else if (e.key === 'Enter' && !showCancel) {
      handleConfirm()
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if showModal}
  <div
    class="modal-overlay"
    onclick={(e) => e.target === e.currentTarget && handleCancel()}
    onkeydown={(e) => e.key === 'Escape' && handleCancel()}
    role="dialog"
    aria-modal="true"
    tabindex="0"
  >
    <div class="modal-container" class:danger={type === 'danger'} role="document">
      <div class="modal-header">
        <h3>{title}</h3>
        <button class="close-btn" onclick={handleCancel} aria-label="关闭">×</button>
      </div>
      
      <div class="modal-body">
        <slot>
          {#if type === 'default'}
            <input
              type="text"
              placeholder="请输入..."
              class="modal-input"
              bind:value={localInputValue}
              onkeydown={(e) => e.key === 'Enter' && handleConfirm()}
            />
          {:else}
            <p class="confirm-text">此操作不可撤销，确定要继续吗？</p>
          {/if}
        </slot>
      </div>
      
      <div class="modal-footer">
        {#if showCancel}
          <button class="btn btn-default" onclick={handleCancel}>
            {cancelText}
          </button>
        {/if}
        <button 
          class="btn" 
          class:btn-primary={type === 'default'}
          class:btn-danger={type === 'danger'}
          onclick={handleConfirm}
        >
          {confirmText}
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
    animation: fadeIn 0.2s ease;
  }
  
  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }
  
  .modal-container {
    background: var(--editor-bg, #ffffff);
    border-radius: 12px;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
    min-width: 360px;
    max-width: 480px;
    animation: slideIn 0.2s ease;
    overflow: hidden;
  }
  
  .modal-container.danger {
    border-top: 3px solid var(--danger-color, #ff4d4f);
  }
  
  @keyframes slideIn {
    from {
      opacity: 0;
      transform: translateY(-20px) scale(0.95);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }
  
  .modal-header {
    padding: 20px 24px 16px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid var(--border-color, #e0e0e0);
  }
  
  .modal-header h3 {
    margin: 0;
    font-size: 18px;
    font-weight: 600;
    color: var(--text-primary, #213547);
  }
  
  .close-btn {
    background: none;
    border: none;
    font-size: 24px;
    color: var(--text-secondary, #8c8c8c);
    cursor: pointer;
    padding: 0;
    line-height: 1;
    transition: color 0.2s;
  }
  
  .close-btn:hover {
    color: var(--text-primary, #213547);
  }
  
  .modal-body {
    padding: 24px;
  }
  
  .modal-input {
    width: 100%;
    padding: 12px 16px;
    border: 1px solid var(--border-color, #e0e0e0);
    border-radius: 8px;
    font-size: 14px;
    color: var(--text-primary, #213547);
    background: var(--editor-bg, #ffffff);
    transition: border-color 0.2s, box-shadow 0.2s;
    box-sizing: border-box;
  }
  
  .modal-input:focus {
    outline: none;
    border-color: var(--primary-color, #1890ff);
    box-shadow: 0 0 0 3px rgba(24, 144, 255, 0.1);
  }
  
  .confirm-text {
    margin: 0;
    font-size: 15px;
    color: var(--text-primary, #213547);
    line-height: 1.6;
  }
  
  .modal-footer {
    padding: 16px 24px 20px;
    display: flex;
    justify-content: flex-end;
    gap: 12px;
    background: var(--sidebar-bg, #f5f5f5);
  }
  
  .btn {
    padding: 10px 20px;
    border-radius: 8px;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
    border: none;
  }
  
  .btn-default {
    background: var(--editor-bg, #ffffff);
    color: var(--text-primary, #213547);
    border: 1px solid var(--border-color, #e0e0e0);
  }
  
  .btn-default:hover {
    border-color: var(--primary-color, #1890ff);
    color: var(--primary-color, #1890ff);
  }
  
  .btn-primary {
    background: var(--primary-color, #1890ff);
    color: white;
  }
  
  .btn-primary:hover {
    background: var(--primary-hover, #40a9ff);
  }
  
  .btn-danger {
    background: var(--danger-color, #ff4d4f);
    color: white;
  }
  
  .btn-danger:hover {
    background: #ff7875;
  }
</style>

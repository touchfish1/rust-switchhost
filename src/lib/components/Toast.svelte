<script lang="ts">
  import { fly, fade } from 'svelte/transition'
  import { toasts } from '$lib/stores/toasts'

  const iconMap = {
    success: '✓',
    error: '!',
    warning: '!',
    info: 'i'
  } as const

  function autoclose(_node: HTMLElement, params: { id: number; duration: number }) {
    const timer = window.setTimeout(() => {
      toasts.remove(params.id)
    }, params.duration)

    return {
      destroy() {
        window.clearTimeout(timer)
      }
    }
  }
</script>

<div class="toast-stack" aria-live="polite" aria-atomic="true">
  {#each $toasts as toast (toast.id)}
    <div
      class={`toast ${toast.kind}`}
      in:fly={{ y: -12, duration: 180 }}
      out:fade={{ duration: 160 }}
      use:autoclose={{ id: toast.id, duration: toast.duration }}
      role="status"
    >
      <span class="toast-icon" aria-hidden="true">{iconMap[toast.kind]}</span>
      <span class="toast-message">{toast.message}</span>
      <button class="toast-close" type="button" on:click={() => toasts.remove(toast.id)} aria-label="关闭通知">
        ×
      </button>
    </div>
  {/each}
</div>

<style>
  .toast-stack {
    position: fixed;
    top: 78px;
    right: 20px;
    z-index: 3200;
    display: flex;
    flex-direction: column;
    gap: 12px;
    pointer-events: none;
  }

  .toast {
    min-width: 280px;
    max-width: min(420px, calc(100vw - 32px));
    padding: 12px 14px;
    border-radius: 12px;
    border: 1px solid var(--border-color, #e0e0e0);
    box-shadow: 0 14px 34px rgba(15, 23, 42, 0.14);
    background: var(--editor-bg, #ffffff);
    color: var(--text-primary, #213547);
    display: grid;
    grid-template-columns: 20px minmax(0, 1fr) 20px;
    gap: 10px;
    align-items: start;
    pointer-events: auto;
  }

  .toast.success {
    border-color: rgba(82, 196, 26, 0.3);
    background: #f6ffed;
  }

  .toast.error {
    border-color: rgba(255, 77, 79, 0.28);
    background: #fff2f0;
  }

  .toast.warning {
    border-color: rgba(250, 173, 20, 0.28);
    background: #fffbe6;
  }

  .toast.info {
    border-color: rgba(24, 144, 255, 0.24);
    background: #e6f4ff;
  }

  :global(.dark) .toast {
    box-shadow: 0 16px 36px rgba(0, 0, 0, 0.35);
  }

  :global(.dark) .toast.success {
    background: #162312;
    border-color: #3d5c2e;
  }

  :global(.dark) .toast.error {
    background: #2a1f1f;
    border-color: #5a3030;
  }

  :global(.dark) .toast.warning {
    background: #2b2615;
    border-color: #6b5b18;
  }

  :global(.dark) .toast.info {
    background: #152535;
    border-color: #245a8d;
  }

  .toast-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    border-radius: 999px;
    font-size: 12px;
    font-weight: 700;
    color: white;
    background: var(--primary-color, #1890ff);
  }

  .toast.success .toast-icon {
    background: #52c41a;
  }

  .toast.error .toast-icon {
    background: #ff4d4f;
  }

  .toast.warning .toast-icon {
    background: #faad14;
    color: #4a2f00;
  }

  .toast-message {
    font-size: 14px;
    line-height: 1.5;
    word-break: break-word;
  }

  .toast-close {
    border: none;
    background: transparent;
    color: var(--text-secondary, #8c8c8c);
    cursor: pointer;
    font-size: 18px;
    line-height: 1;
    padding: 0;
  }

  @media (max-width: 640px) {
    .toast-stack {
      left: 12px;
      right: 12px;
      top: 72px;
    }

    .toast {
      min-width: 0;
      max-width: none;
    }
  }
</style>

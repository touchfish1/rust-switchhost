<script lang="ts">
  import type { DnsLookupResult } from '$lib/types'

  export let isOpen = false
  export let domain = ''
  export let lookupResult: DnsLookupResult | null = null
  export let isResolving = false
  export let onClose: () => void
  export let onDomainChange: (value: string) => void
  export let onResolve: () => void | Promise<void>

  function handleClose() {
    onClose()
  }

  function handleSubmit(event: Event) {
    event.preventDefault()
    onResolve()
  }
</script>

{#if isOpen}
  <div
    class="diagnostic-overlay"
    on:click|self={handleClose}
    on:keydown={(event) => event.key === 'Escape' && handleClose()}
    role="dialog"
    aria-modal="true"
    aria-label="DNS 诊断"
    tabindex="0"
  >
    <div class="diagnostic-modal" role="document">
      <div class="diagnostic-header">
        <div>
          <h3>DNS 诊断</h3>
          <p>输入域名，快速查看当前系统实际解析结果，辅助判断 hosts 是否已生效。</p>
        </div>
        <button class="close-btn" on:click={handleClose} aria-label="关闭">×</button>
      </div>

      <div class="diagnostic-body">
        <form class="lookup-form" on:submit={handleSubmit}>
          <input
            type="text"
            value={domain}
            on:input={(event) => onDomainChange((event.currentTarget as HTMLInputElement).value)}
            placeholder="例如 api.local.test 或 github.com"
            disabled={isResolving}
          />
          <button type="submit" class="resolve-btn" disabled={!domain.trim() || isResolving}>
            {isResolving ? '解析中...' : '开始诊断'}
          </button>
        </form>

        {#if lookupResult}
          <div class="result-card" class:error={!lookupResult.success}>
            <div class="result-head">
              <strong>{lookupResult.success ? '解析成功' : '解析失败'}</strong>
              <span>{lookupResult.domain}</span>
            </div>

            <p>{lookupResult.message}</p>

            {#if lookupResult.addresses.length > 0}
              <div class="address-list">
                {#each lookupResult.addresses as address}
                  <code>{address}</code>
                {/each}
              </div>
            {/if}
          </div>
        {:else}
          <div class="placeholder">
            输入一个域名后开始诊断，这里会显示当前系统 DNS 解析结果。
          </div>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .diagnostic-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.45);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 2200;
    padding: 24px;
  }

  .diagnostic-modal {
    width: min(760px, 100%);
    background: var(--editor-bg);
    border-radius: 14px;
    box-shadow: 0 18px 60px rgba(0, 0, 0, 0.25);
    overflow: hidden;
  }

  .diagnostic-header {
    padding: 16px 20px;
    border-bottom: 1px solid var(--border-color);
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 16px;
  }

  .diagnostic-header h3 {
    margin: 0 0 4px 0;
    font-size: 18px;
    color: var(--text-primary);
  }

  .diagnostic-header p {
    margin: 0;
    font-size: 13px;
    color: var(--text-secondary);
    line-height: 1.6;
  }

  .close-btn {
    width: 36px;
    height: 36px;
    border: none;
    border-radius: 8px;
    background: transparent;
    color: var(--text-secondary);
    font-size: 24px;
    cursor: pointer;
  }

  .close-btn:hover {
    background: var(--hover-bg);
    color: var(--text-primary);
  }

  .diagnostic-body {
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .lookup-form {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 12px;
  }

  .lookup-form input {
    width: 100%;
    box-sizing: border-box;
    padding: 12px 14px;
    border: 1px solid var(--border-color);
    border-radius: 10px;
    background: var(--editor-bg);
    color: var(--text-primary);
    font-size: 14px;
  }

  .lookup-form input:focus {
    outline: none;
    border-color: var(--primary-color);
  }

  .resolve-btn {
    padding: 0 16px;
    border: none;
    border-radius: 10px;
    background: var(--primary-color);
    color: #fff;
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
  }

  .resolve-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .result-card,
  .placeholder {
    border-radius: 12px;
    border: 1px solid var(--border-color);
    background: var(--sidebar-bg);
    padding: 16px;
  }

  .result-card.error {
    border-color: rgba(255, 77, 79, 0.3);
    background: rgba(255, 77, 79, 0.06);
  }

  .result-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    margin-bottom: 10px;
  }

  .result-head strong {
    color: var(--text-primary);
    font-size: 14px;
  }

  .result-head span,
  .result-card p,
  .placeholder {
    color: var(--text-secondary);
    font-size: 13px;
    line-height: 1.6;
  }

  .result-card p {
    margin: 0 0 12px 0;
  }

  .address-list {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .address-list code {
    padding: 6px 10px;
    border-radius: 999px;
    background: var(--editor-bg);
    border: 1px solid var(--border-color);
    color: var(--text-primary);
    font-size: 12px;
  }

  @media (max-width: 640px) {
    .lookup-form {
      grid-template-columns: 1fr;
    }

    .result-head {
      flex-direction: column;
      align-items: flex-start;
    }
  }
</style>

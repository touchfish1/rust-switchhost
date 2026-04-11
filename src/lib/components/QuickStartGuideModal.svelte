<script lang="ts">
  export let isOpen = false
  export let onClose: () => void
  export let onCreateLocal: () => void | Promise<void> = () => {}
  export let onCreateRemote: () => void | Promise<void> = () => {}
  export let onCreateExample: () => void | Promise<void> = () => {}

  const steps = [
    {
      title: '1. 创建分组',
      description: '先新建一个本地或远程分组。空白分组也可以先创建，后续再补规则。'
    },
    {
      title: '2. 编写或同步规则',
      description: '本地分组适合手动编辑，远程分组适合团队共享和定时同步。'
    },
    {
      title: '3. 启用并预览',
      description: '勾选分组后会立即生效，建议先看一次“合并预览”确认最终写入结果。'
    }
  ]
</script>

{#if isOpen}
  <div
    class="guide-overlay"
    on:click|self={onClose}
    on:keydown={(event) => event.key === 'Escape' && onClose()}
    role="dialog"
    aria-modal="true"
    aria-label="首次使用引导"
    tabindex="0"
  >
    <div class="guide-modal" role="document">
      <div class="guide-header">
        <div>
          <h3>首次使用引导</h3>
          <p>3 分钟内就能完成一次安全可控的 Hosts 切换。</p>
        </div>
        <button class="close-btn" on:click={onClose} aria-label="关闭">×</button>
      </div>

      <div class="guide-body">
        <div class="guide-safety">
          <strong>使用前你可以放心这 2 件事</strong>
          <span>软件只维护自己追加的托管区块，不会覆盖系统原始 Hosts 内容。</span>
          <span>Linux 下写入时可能弹出权限确认，这是正常行为。</span>
        </div>

        <div class="guide-steps">
          {#each steps as step}
            <div class="guide-step">
              <strong>{step.title}</strong>
              <p>{step.description}</p>
            </div>
          {/each}
        </div>

        <div class="guide-tips">
          <span>提示：双击左侧分组名称可重命名，`Ctrl+N` 可快速新建。</span>
          <span>如果启用多个分组，越靠后的规则越可能成为最终生效结果。</span>
        </div>

        <div class="guide-actions-box">
          <strong>可以直接从这里开始</strong>
          <div class="guide-actions">
            <button class="btn-secondary" on:click={onCreateLocal}>新建本地分组</button>
            <button class="btn-secondary" on:click={onCreateRemote}>新建远程分组</button>
            <button class="btn-secondary" on:click={onCreateExample}>创建示例分组</button>
          </div>
        </div>
      </div>

      <div class="guide-footer">
        <button class="btn-primary" on:click={onClose}>开始使用</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .guide-overlay {
    position: fixed;
    inset: 0;
    z-index: 2300;
    background: rgba(0, 0, 0, 0.48);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 24px;
  }

  .guide-modal {
    width: min(680px, 100%);
    background: var(--editor-bg, #ffffff);
    border-radius: 16px;
    box-shadow: 0 24px 70px rgba(0, 0, 0, 0.3);
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .guide-header,
  .guide-footer {
    padding: 18px 22px;
    border-bottom: 1px solid var(--border-color, #e0e0e0);
  }

  .guide-footer {
    border-bottom: none;
    border-top: 1px solid var(--border-color, #e0e0e0);
    display: flex;
    justify-content: flex-end;
  }

  .guide-header {
    display: flex;
    justify-content: space-between;
    gap: 16px;
  }

  .guide-header h3 {
    margin: 0 0 6px 0;
    font-size: 20px;
    color: var(--text-primary, #213547);
  }

  .guide-header p {
    margin: 0;
    font-size: 13px;
    color: var(--text-secondary, #8c8c8c);
  }

  .guide-body {
    padding: 22px;
    display: flex;
    flex-direction: column;
    gap: 18px;
  }

  .guide-safety {
    padding: 16px;
    border-radius: 14px;
    background: linear-gradient(135deg, rgba(24, 144, 255, 0.12), rgba(54, 207, 201, 0.08));
    border: 1px solid rgba(24, 144, 255, 0.2);
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .guide-safety strong,
  .guide-step strong {
    color: var(--text-primary, #213547);
  }

  .guide-safety span,
  .guide-step p,
  .guide-tips span {
    color: var(--text-secondary, #8c8c8c);
    font-size: 13px;
    line-height: 1.6;
  }

  .guide-steps {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 12px;
  }

  .guide-step {
    padding: 16px;
    border-radius: 14px;
    border: 1px solid var(--border-color, #e0e0e0);
    background: var(--sidebar-bg, #f5f5f5);
  }

  .guide-step p {
    margin: 8px 0 0 0;
  }

  .guide-tips {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .guide-actions-box {
    padding: 16px;
    border-radius: 14px;
    border: 1px solid var(--border-color, #e0e0e0);
    background: var(--sidebar-bg, #f5f5f5);
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .guide-actions-box strong {
    color: var(--text-primary, #213547);
  }

  .guide-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
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
    background: var(--hover-bg, #e6f7ff);
    color: var(--text-primary, #213547);
  }

  .btn-primary {
    padding: 10px 18px;
    border-radius: 10px;
    border: none;
    background: var(--primary-color, #1890ff);
    color: #fff;
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
  }

  .btn-primary:hover {
    background: var(--primary-hover, #40a9ff);
  }

  .btn-secondary {
    padding: 10px 16px;
    border-radius: 10px;
    border: 1px solid var(--border-color, #e0e0e0);
    background: var(--editor-bg, #ffffff);
    color: var(--text-primary, #213547);
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
  }

  .btn-secondary:hover {
    border-color: var(--primary-color, #1890ff);
    color: var(--primary-color, #1890ff);
  }

  @media (max-width: 820px) {
    .guide-steps {
      grid-template-columns: 1fr;
    }

    .guide-actions {
      flex-direction: column;
    }
  }
</style>

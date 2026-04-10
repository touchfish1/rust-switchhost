<svelte:options runes={true} />

<script lang="ts">
  import type { SchemeTemplate } from '$lib/types'

  type ConfirmPayload = {
    name: string
    type: 'local' | 'remote'
    remoteUrl: string
    autoSyncEnabled: boolean
    syncIntervalMinutes: string
    templateId: string | null
  }

  type CreateSchemeModalProps = {
    isOpen?: boolean
    isSubmitting?: boolean
    mode?: 'create' | 'edit-remote'
    initialName?: string
    initialType?: 'local' | 'remote'
    initialRemoteUrl?: string
    initialAutoSyncEnabled?: boolean
    initialSyncIntervalMinutes?: string
    templates?: SchemeTemplate[]
    onDeleteTemplate?: (id: string) => void
    onClose?: () => void
    onConfirm?: (payload: ConfirmPayload) => void
  }

  let {
    isOpen = false,
    isSubmitting = false,
    mode = 'create',
    initialName = '',
    initialType = 'local',
    initialRemoteUrl = '',
    initialAutoSyncEnabled = true,
    initialSyncIntervalMinutes = '15',
    templates = [],
    onDeleteTemplate = () => {},
    onClose = () => {},
    onConfirm = () => {}
  }: CreateSchemeModalProps = $props()

  let name = $state('')
  let schemeType = $state<'local' | 'remote'>('local')
  let remoteUrl = $state('')
  let autoSyncEnabled = $state(true)
  let syncIntervalMinutes = $state('15')
  let selectedTemplateId = $state<string | null>('blank')
  let wasOpen = $state(false)

  const selectedTemplate = $derived(
    templates.find((template) => template.id === selectedTemplateId) ?? null
  )

  $effect(() => {
    if (isOpen && !wasOpen) {
      name = initialName
      schemeType = initialType
      remoteUrl = initialRemoteUrl
      autoSyncEnabled = initialAutoSyncEnabled
      syncIntervalMinutes = initialSyncIntervalMinutes
      selectedTemplateId = templates[0]?.id ?? null
      wasOpen = true
    } else if (!isOpen && wasOpen) {
      wasOpen = false
    }
  })

  function handleClose() {
    onClose()
  }

  function handleConfirm() {
    onConfirm({
      name: name.trim(),
      type: schemeType,
      remoteUrl: remoteUrl.trim(),
      autoSyncEnabled,
      syncIntervalMinutes: syncIntervalMinutes.trim(),
      templateId: schemeType === 'local' ? selectedTemplateId : null
    })
  }
</script>

{#if isOpen}
  <div
    class="modal-overlay"
    onclick={(event) => event.target === event.currentTarget && handleClose()}
    onkeydown={(event) => event.key === 'Escape' && handleClose()}
    role="dialog"
    aria-modal="true"
    aria-label={mode === 'edit-remote' ? '编辑远程分组' : '创建新分组'}
    tabindex="0"
  >
    <div class="modal" role="document">
      <div class="modal-header">
        <div>
          <h3>{mode === 'edit-remote' ? '编辑远程分组' : '创建新分组'}</h3>
          <p>{mode === 'edit-remote' ? '修改远程 URL、同步开关和同步间隔' : '支持创建普通分组或远程 URL 分组'}</p>
        </div>
        <button class="close-btn" onclick={handleClose} aria-label="关闭">×</button>
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

        {#if mode === 'create'}
          <div class="form-group">
            <span>分组类型</span>
            <div class="type-grid">
              <button
                class="type-card"
                class:selected={schemeType === 'local'}
                type="button"
                onclick={() => (schemeType = 'local')}
                disabled={isSubmitting}
              >
                <strong>本地分组</strong>
                <small>手动编辑 hosts 内容</small>
              </button>

              <button
                class="type-card"
                class:selected={schemeType === 'remote'}
                type="button"
                onclick={() => (schemeType = 'remote')}
                disabled={isSubmitting}
              >
                <strong>远程 URL</strong>
                <small>首次创建后自动拉取并可定时同步</small>
              </button>
            </div>
          </div>
        {/if}

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
        {:else if mode === 'create'}
          <div class="form-group">
            <span>快速模板</span>
            <div class="template-grid">
              {#each templates as template (template.id)}
                <button
                  class="template-card"
                  class:selected={selectedTemplateId === template.id}
                  type="button"
                  onclick={() => (selectedTemplateId = template.id)}
                  disabled={isSubmitting}
                >
                  <strong>{template.name}</strong>
                  <small>{template.description}</small>
                </button>
              {/each}
            </div>
          </div>

          {#if selectedTemplate}
            <div class="template-preview">
              <div class="template-preview-head">
                <div class="template-preview-copy">
                  <strong>模板预览</strong>
                  <span>{selectedTemplate.name}</span>
                </div>
                {#if selectedTemplate.source === 'custom'}
                  <button
                    class="template-delete-btn"
                    type="button"
                    onclick={() => {
                      onDeleteTemplate(selectedTemplate.id)
                      selectedTemplateId = templates.find((template) => template.id !== selectedTemplate.id)?.id ?? null
                    }}
                    disabled={isSubmitting}
                  >
                    删除模板
                  </button>
                {/if}
              </div>
              <pre>{selectedTemplate.content}</pre>
            </div>
          {/if}
        {/if}
      </div>

      <div class="modal-footer">
        <button class="btn btn-default" type="button" onclick={handleClose} disabled={isSubmitting}>
          取消
        </button>
        <button class="btn btn-primary" type="button" onclick={handleConfirm} disabled={isSubmitting}>
          {isSubmitting ? (mode === 'edit-remote' ? '保存中...' : '创建中...') : (mode === 'edit-remote' ? '保存修改' : '创建分组')}
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

  .template-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 12px;
  }

  .template-card {
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

  .template-card strong {
    font-size: 14px;
  }

  .template-card small {
    font-size: 12px;
    line-height: 1.5;
    color: var(--text-secondary, #8c8c8c);
  }

  .template-card:hover,
  .template-card.selected {
    border-color: var(--primary-color, #1890ff);
    background: var(--hover-bg, #e6f7ff);
  }

  .template-preview {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 14px;
    border-radius: 12px;
    border: 1px solid var(--border-color, #e0e0e0);
    background: var(--sidebar-bg, #f5f5f5);
  }

  .template-preview-head {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    align-items: center;
  }

  .template-preview-head strong {
    font-size: 14px;
    color: var(--text-primary, #213547);
  }

  .template-preview-copy {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .template-preview-head span {
    font-size: 12px;
    color: var(--text-secondary, #8c8c8c);
  }

  .template-delete-btn {
    padding: 7px 12px;
    border-radius: 8px;
    border: 1px solid rgba(255, 77, 79, 0.28);
    background: rgba(255, 77, 79, 0.08);
    color: #cf1322;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
  }

  .template-delete-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .template-preview pre {
    margin: 0;
    max-height: 220px;
    overflow: auto;
    padding: 12px 14px;
    border-radius: 10px;
    background: var(--editor-bg, #ffffff);
    border: 1px solid var(--border-color, #e0e0e0);
    color: var(--text-primary, #213547);
    font-family: var(--font-family-mono);
    font-size: 12px;
    line-height: 1.6;
    white-space: pre-wrap;
    word-break: break-word;
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

    .template-grid {
      grid-template-columns: 1fr;
    }

    .template-preview-head {
      flex-direction: column;
      align-items: flex-start;
    }
  }
</style>

import type { SchemeTemplate } from '$lib/types'

export const builtinSchemeTemplates: SchemeTemplate[] = [
  {
    id: 'blank',
    source: 'builtin',
    name: '空白模板',
    description: '从最小 hosts 注释开始，自由填写自己的规则。',
    content: '# 新的 hosts 配置\n# 在下方填写你的 hosts 规则\n'
  },
  {
    id: 'local-dev',
    source: 'builtin',
    name: '本地开发',
    description: '适合本地站点联调，预置 localhost 和常见本地域名写法。',
    content: [
      '# 本地开发模板',
      '127.0.0.1 localhost',
      '127.0.0.1 api.local.test',
      '127.0.0.1 admin.local.test',
      '',
      '# 如果你的前端和接口分别跑在不同端口，可继续扩展域名映射'
    ].join('\n')
  },
  {
    id: 'example-group',
    source: 'builtin',
    name: '示例分组',
    description: '一键体验完整流程，适合首次熟悉启用、预览和回滚操作。',
    content: [
      '# 示例分组',
      '127.0.0.1 demo.local.test',
      '127.0.0.1 api.demo.local.test',
      '',
      '# 你可以先启用体验，再按需替换为自己的域名'
    ].join('\n')
  },
  {
    id: 'staging',
    source: 'builtin',
    name: '测试环境',
    description: '适合将多个业务域名统一指向测试机或预发布环境。',
    content: [
      '# 测试环境模板',
      '10.0.0.10 api.staging.internal',
      '10.0.0.10 web.staging.internal',
      '10.0.0.10 assets.staging.internal',
      '',
      '# 请按实际测试环境 IP 修改'
    ].join('\n')
  },
  {
    id: 'cdn-debug',
    source: 'builtin',
    name: '静态资源调试',
    description: '用于将站点和静态资源域名切到指定机器，排查 CDN 或缓存问题。',
    content: [
      '# 静态资源调试模板',
      '203.0.113.10 www.example.com',
      '203.0.113.10 static.example.com',
      '203.0.113.10 cdn.example.com',
      '',
      '# 请将示例 IP 和域名替换为你的线上或灰度节点'
    ].join('\n')
  }
]

export function getSchemeTemplateContent(templateId: string | null | undefined) {
  return builtinSchemeTemplates.find((template) => template.id === templateId)?.content
}

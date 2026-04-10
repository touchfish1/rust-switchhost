# Rust SwitchHost 项目优化计划

> **版本**：v1.0 | **日期**：2026-04-10 | **项目版本**：v0.0.23
> **仓库**：https://github.com/touchfish1/rust-switchhost

---

## 一、项目现状概述

Rust SwitchHost 是一个基于 Tauri 2.x + Svelte 5 + Rust 的跨平台 Hosts 文件管理工具，整体代码质量良好，但存在安全性、架构、代码质量、UI/UX 四个方面的优化空间。

### 当前项目规模

| 指标 | 数值 |
|------|------|
| 总代码行数 | ~8,017 行 |
| Rust 后端 | 1,642 行（11 个 .rs 文件） |
| Svelte 前端 | 3,905 行（8 个组件） |
| Tauri IPC 命令 | 19 个 |
| 单元测试 | 仅 5 个 |

### 核心问题概览

| 方向 | 问题数 | 🔴 高优先级 | 🟡 中优先级 | 🟢 低优先级 |
|------|--------|:-----------:|:-----------:|:-----------:|
| 安全性修复 | 5 | 3 | 2 | 0 |
| 前端架构重构 | 6 | 1 | 3 | 2 |
| 后端代码质量 | 5 | 2 | 2 | 1 |
| UI/UX 改进 | 4 | 0 | 2 | 2 |
| **合计** | **20** | **6** | **9** | **5** |

---

## 二、安全性修复

### 任务 S1：设置 CSP 内容安全策略 🔴

- **问题**：`src-tauri/tauri.conf.json` 第 24-26 行，`"csp": null` 完全禁用了 CSP
- **风险**：页面可加载任意来源的脚本和资源，存在 XSS 攻击面
- **修改方案**：
  1. 在 `tauri.conf.json` 中设置合理的 CSP 策略
  2. 推荐值：
     ```
     default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' asset: https://asset.localhost; connect-src 'self' https://github.com https://api.github.com https://gitee.com
     ```
  3. 根据实际远程 URL 同步需求，可能需要扩展 `connect-src`
- **涉及文件**：`src-tauri/tauri.conf.json`
- **预估工时**：0.5h
- **验证方式**：启动应用，检查 DevTools Console 无 CSP 违规警告

---

### 任务 S2：save_config 错误传播 🔴

- **问题**：`src-tauri/src/schemes/manager.rs` 第 413-427 行，`save_config` 在 `fs::write` 失败时返回 `Ok(())`
- **风险**：用户以为操作成功，但配置未持久化，存在数据丢失风险
- **当前代码**：
  ```rust
  match fs::write(&self.config_path, &content) {
      Ok(_) => Ok(()),
      Err(e) => {
          eprintln!("Warning: Failed to save config: {}", e);
          Ok(())   // ← 吞掉了错误！
      }
  }
  ```
- **修改方案**：
  1. 移除 `match` 中的 `Err` 分支，改为 `fs::write(&self.config_path, &content)?`
  2. 上层调用方已有 `.map_err()` 处理，无需额外修改
  3. 前端已有 `try/catch` 包裹 `invoke` 调用，错误会自动展示
- **涉及文件**：`src-tauri/src/schemes/manager.rs`（第 421-424 行）
- **预估工时**：0.5h
- **验证方式**：模拟写入权限失败场景，确认前端收到错误提示

---

### 任务 S3：创建/更新方案时验证 hosts 内容 🔴

- **问题**：`src-tauri/src/commands/schemes.rs` 第 24-46 行，`create_scheme` 和 `update_scheme` 命令未调用 `validate_hosts_content`
- **风险**：用户可保存无效的 hosts 内容，仅在启用时才报错
- **修改方案**：
  1. 在 `create_scheme` 命令中，调用 `manager.create_scheme` 前先调用 `validate_hosts_content(&content)`
  2. 在 `update_scheme` 命令中，同理添加验证
  3. 验证失败时返回明确的中文错误信息
- **涉及文件**：`src-tauri/src/commands/schemes.rs`（第 24-46 行）
- **预估工时**：0.5h
- **验证方式**：尝试创建包含无效 IP 的方案，确认被拒绝

---

### 任务 S4：添加 SSRF 防护 🟡

- **问题**：`src-tauri/src/validation.rs` 第 5-22 行，`validate_remote_url` 仅检查格式，不检查内网地址
- **风险**：攻击者可指定 `http://127.0.0.1`、`http://169.254.169.254`（云元数据）等地址
- **修改方案**：
  1. 在 `validation.rs` 中新增 `is_private_url(url: &str) -> bool` 函数
  2. 解析 URL 的 hostname，检查是否为：
     - 回环地址：`127.0.0.0/8`、`::1`
     - 私有网段：`10.0.0.0/8`、`172.16.0.0/12`、`192.168.0.0/16`
     - 链路本地：`169.254.0.0/16`
  3. 在 `validate_remote_url` 中调用该检查
  4. 添加对应的单元测试
- **涉及文件**：`src-tauri/src/validation.rs`
- **预估工时**：1h
- **验证方式**：尝试添加 `http://127.0.0.1/hosts` 远程 URL，确认被拒绝

---

### 任务 S5：移除未使用的 thiserror/anyhow 依赖 🟡

- **问题**：`src-tauri/Cargo.toml` 声明了 `thiserror = "2"` 和 `anyhow = "1"`，但整个 `src-tauri/src/` 中无任何引用
- **风险**：增加编译时间和二进制体积
- **修改方案**：从 `Cargo.toml` 的 `[dependencies]` 中移除 `thiserror` 和 `anyhow`
- **涉及文件**：`src-tauri/Cargo.toml`
- **预估工时**：0.2h
- **验证方式**：`cargo build` 编译通过

---

## 三、前端架构重构

### 任务 F1：拆分 App.svelte 状态管理 🔴

- **问题**：`src/lib/App.svelte` 的 script 部分有 **693 行**，包含 **31 个状态变量**和 20+ 个处理函数
- **状态变量清单**（第 58-89 行）：

  | 变量名 | 类型 | 用途 |
  |--------|------|------|
  | `schemes` | `Scheme[]` | 全部分组数据 |
  | `activeSchemeId` | `string \| null` | 当前选中 ID |
  | `activeScheme` | `Scheme \| null` | 当前选中对象 |
  | `editorContent` | `string` | 编辑器内容 |
  | `isLoading` | `boolean` | 全局加载状态 |
  | `error` | `string \| null` | 全局错误 |
  | `isDarkMode` | `boolean` | 主题状态 |
  | `appVersion` | `string` | 应用版本 |
  | `hostsPermissionInfo` | `HostsPermissionInfo \| null` | 权限信息 |
  | `showCreateModal` | `boolean` | 创建弹窗 |
  | `showDeleteModal` | `boolean` | 删除弹窗 |
  | `showCurrentHostsModal` | `boolean` | Hosts 查看弹窗 |
  | `showUpdateModal` | `boolean` | 更新弹窗 |
  | `showSyncLogModal` | `boolean` | 同步日志弹窗 |
  | `createModalMode` | `'create' \| 'edit-remote'` | 弹窗模式 |
  | `remoteEditTarget` | `Scheme \| null` | 远程编辑目标 |
  | `deleteTargetId` | `string \| null` | 删除目标 |
  | `currentHostsContent` | `string` | 当前 hosts 内容 |
  | `updateInfo` | `UpdateInfo \| null` | 更新信息 |
  | `availableUpdate` | `Update \| null` | 可用更新 |
  | `isInstallingUpdate` | `boolean` | 安装中 |
  | `isFlushingDns` | `boolean` | DNS 刷新中 |
  | `updateProgressText` | `string` | 更新进度 |
  | `hasPendingUpdate` | `boolean` | 有待安装更新 |
  | `updateCheckTimer` | `ReturnType<typeof setInterval> \| null` | 检查定时器 |
  | `isSyncingRemoteScheme` | `boolean` | 远程同步中 |
  | `isCreatingScheme` | `boolean` | 创建中 |
  | `sidebarWidth` | `number` | 侧边栏宽度 |
  | `syncLogs` | `Array<...>` | 同步日志 |
  | `syncEventUnlisten` | `UnlistenFn \| null` | 事件取消监听 |
  | `syncingSchemeIds` | `Set<string>` | 同步中的 ID 集合 |

- **修改方案**：
  1. 创建 `src/lib/stores/` 目录
  2. `src/lib/stores/schemes.ts` — 抽取 `schemes`、`activeSchemeId`、`activeScheme`、`editorContent` 及 CRUD 函数
  3. `src/lib/stores/theme.ts` — 抽取 `isDarkMode` 及切换逻辑
  4. `src/lib/stores/app.ts` — 抽取 `isLoading`、`error`、`appVersion` 等应用级状态
  5. `src/lib/stores/updater.ts` — 抽取 `updateInfo`、`availableUpdate`、`isInstallingUpdate` 等更新状态
  6. 使用 Svelte 5 的 writable stores 或 `$state` runes 管理状态
- **涉及文件**：新建 `src/lib/stores/*.ts`，修改 `src/lib/App.svelte`
- **预估工时**：4h
- **验证方式**：所有功能正常运行，App.svelte script 部分缩减到 200 行以内

---

### 任务 F2：迁移到 Svelte 5 Runes 🟡

- **问题**：6 个子组件使用 `createEventDispatcher`，5 处使用 `$:` 响应式语句，0 处使用 Svelte 5 runes
- **当前使用情况**：

  **createEventDispatcher（6 个文件）**：

  | 文件 | import 行 | 赋值行 |
  |------|-----------|--------|
  | `CreateSchemeModal.svelte` | 2 | 13 |
  | `RemoteSyncModal.svelte` | 2 | 14 |
  | `Modal.svelte` | 2 | 12 |
  | `Sidebar.svelte` | 2 | 8 |
  | `Editor.svelte` | 3 | 14 |
  | `ThemeToggle.svelte` | 2 | 6 |

  **$:` 响应式语句（5 处）**：

  | 文件 | 行号 | 用途 |
  |------|------|------|
  | `CreateSchemeModal.svelte` | 22, 31 | 弹窗打开/关闭时重置表单 |
  | `RemoteSyncModal.svelte` | 20 | 弹窗打开时加载数据 |
  | `Editor.svelte` | 369, 379 | 内容同步和主题切换 |

- **修改方案**：
  1. 将 `createEventDispatcher` 替换为回调函数 props（如 `onClose`、`onChange`）
  2. 将 `$:` 响应式语句替换为 `$derived()` 或 `$effect()`
  3. 将 `let` 状态变量替换为 `$state()`
- **涉及文件**：`src/lib/components/*.svelte`（6 个文件）
- **预估工时**：3h
- **验证方式**：所有组件交互正常，无 console 警告

---

### 任务 F3：统一类型定义 🟡

- **问题**：`Scheme`、`UpdateInfo`、`HostsPermissionInfo`、`DnsFlushResult` 等接口全部内联在 `App.svelte` 第 16-56 行
- **修改方案**：
  1. 创建 `src/lib/types/index.ts`
  2. 将所有接口定义迁移到该文件
  3. 在各组件中统一 `import`
- **涉及文件**：新建 `src/lib/types/index.ts`，修改 `App.svelte` 及引用这些类型的组件
- **预估工时**：1h
- **验证方式**：TypeScript 编译无错误

---

### 任务 F4：清理遗留死代码 🟡

- **问题**：`src/lib/components/RemoteSyncModal.svelte`（287 行）存在但无任何文件 import 它
- **修改方案**：直接删除 `RemoteSyncModal.svelte` 文件
- **涉及文件**：删除 `src/lib/components/RemoteSyncModal.svelte`
- **预估工时**：0.2h
- **验证方式**：应用编译和运行正常

---

### 任务 F5：抽取内联弹窗为独立组件 🟢

- **问题**：`App.svelte` 模板中有 2 个大型内联弹窗：
  - 第 857-887 行：Hosts 查看弹窗（~31 行模板）
  - 第 889-983 行：在线升级弹窗（~95 行模板）
  - 对应 CSS 约 185 行（第 1392-1576 行）
- **修改方案**：
  1. 创建 `src/lib/components/CurrentHostsModal.svelte`
  2. 创建 `src/lib/components/UpdateModal.svelte`
  3. 将模板和样式从 `App.svelte` 迁移到新组件
  4. 通过 props 和事件与父组件通信
- **涉及文件**：新建 2 个组件文件，修改 `App.svelte`
- **预估工时**：2h
- **验证方式**：弹窗功能正常，App.svelte 模板部分显著缩减

---

### 任务 F6：引入 Service 层封装 Tauri 调用 🟢

- **问题**：`App.svelte` 中直接调用 `invoke()` 约 15 处，业务逻辑与 IPC 调用耦合
- **修改方案**：
  1. `src/lib/services/schemes.ts` — 封装所有方案相关 `invoke` 调用
  2. `src/lib/services/hosts.ts` — 封装 hosts 文件操作 `invoke` 调用
  3. `src/lib/services/updater.ts` — 封装更新检查 `invoke` 调用
  4. 每个函数添加类型标注和错误处理
- **涉及文件**：新建 `src/lib/services/*.ts`，修改 `App.svelte`
- **预估工时**：2h
- **验证方式**：所有 `invoke` 调用正常工作

---

## 四、后端代码质量

### 任务 B1：统一错误处理体系 🔴

- **问题**：项目声明了 `thiserror` 和 `anyhow` 但未使用，所有错误通过 `io::Error` 和 `Result<_, String>` 处理
- **修改方案**：
  1. 在 `Cargo.toml` 中保留 `thiserror`（移除 `anyhow`，桌面应用不需要）
  2. 创建 `src-tauri/src/error.rs`，定义统一错误枚举：
     ```rust
     #[derive(Debug, thiserror::Error)]
     pub enum AppError {
         #[error("IO error: {0}")]
         Io(#[from] std::io::Error),
         #[error("Validation failed: {0}")]
         Validation(String),
         #[error("Scheme not found: {0}")]
         SchemeNotFound(String),
         #[error("Network error: {0}")]
         Network(String),
         #[error("Config error: {0}")]
         Config(String),
     }
     ```
  3. 为 `AppError` 实现 `Into<InvokeError>` 或手动转换为 `String`
  4. 逐步替换 `commands/` 和 `manager.rs` 中的手动错误处理
- **涉及文件**：新建 `src-tauri/src/error.rs`，修改 `commands/*.rs`、`manager.rs`、`hosts/mod.rs`
- **预估工时**：3h
- **验证方式**：`cargo build` 编译通过，所有命令的错误返回格式不变

---

### 任务 B2：tray.rs unwrap 替换 🔴

- **问题**：`src-tauri/src/tray.rs` 中有 5 处 `unwrap()`，`lib.rs` 中有 2 处

  | 文件 | 行号 | 代码 | 风险 |
  |------|------|------|------|
  | `tray.rs` | 16 | `.icon(app.default_window_icon().unwrap().clone())` | 中 — 图标缺失时崩溃 |
  | `tray.rs` | 28 | `window.show().unwrap()` | 低 |
  | `tray.rs` | 29 | `window.set_focus().unwrap()` | 低 |
  | `tray.rs` | 39 | `window.show().unwrap()` | 低 |
  | `tray.rs` | 40 | `window.set_focus().unwrap()` | 低 |
  | `lib.rs` | 26 | `app.get_webview_window("main").unwrap()` | 低 |
  | `lib.rs` | 37 | `window.hide().unwrap()` | 低 |

- **修改方案**：
  1. 第 16 行：使用 `if let Some(icon) = app.default_window_icon()` 或提供默认图标
  2. 第 28-29、39-40 行：使用 `if let Ok(_) = window.show()` 静默处理
  3. `lib.rs` 第 26、37 行同理处理
- **涉及文件**：`src-tauri/src/tray.rs`、`src-tauri/src/lib.rs`
- **预估工时**：0.5h
- **验证方式**：`cargo build` 编译通过，应用启动正常

---

### 任务 B3：添加核心模块单元测试 🟡

- **问题**：整个项目仅 5 个单元测试，核心功能无覆盖

  | 文件 | 测试内容 |
  |------|---------|
  | `manager.rs` | `calculates_retry_delay_with_backoff`（1 个） |
  | `validation.rs` | `validates_remote_urls`、`validates_hosts_content`（2 个） |
  | `updates.rs` | 版本解析相关（2 个） |

- **修改方案**：
  1. 为 `schemes/manager.rs` 添加测试：`create_scheme`、`update_scheme`、`delete_scheme`、`switch_scheme`、`apply_active_schemes`
  2. 为 `hosts/mod.rs` 添加测试：`read_hosts_file`、`backup_hosts_file`（使用临时文件）
  3. 为 `commands/schemes.rs` 的 `perform_remote_sync` 添加集成测试（mock HTTP）
  4. 目标：测试覆盖从 5 个增加到 **20+ 个**
- **涉及文件**：`src-tauri/src/schemes/manager.rs`、`src-tauri/src/hosts/mod.rs`
- **预估工时**：4h
- **验证方式**：`cargo test` 全部通过

---

### 任务 B4：清理 active_scheme_ids 旧字段 🟡

- **问题**：`schemes/mod.rs` 第 77 行的 `active_scheme_ids` 旧字段仍在 `manager.rs` 的 6 处被读写

  | 文件 | 行号 | 操作 |
  |------|------|------|
  | `manager.rs` | 174 | `delete_scheme` 中清理 |
  | `manager.rs` | 201 | `switch_scheme` 中同步写入 |
  | `manager.rs` | 224 | `set_scheme_enabled`（启用）中同步 |
  | `manager.rs` | 229 | `set_scheme_enabled`（禁用）中同步 |
  | `manager.rs` | 474 | `migrate_legacy_platform_state` 中检查 |
  | `manager.rs` | 479 | 迁移逻辑 |

- **修改方案**：
  1. 保留字段定义（加 `#[serde(default)]` 保持向后兼容）
  2. 移除 `manager.rs` 中所有对 `active_scheme_ids` 的写入操作
  3. 简化 `migrate_legacy_platform_state` 迁移逻辑
  4. 添加版本号检查，当 `config.version >= "2.0"` 时跳过旧字段迁移
- **涉及文件**：`src-tauri/src/schemes/manager.rs`
- **预估工时**：1h
- **验证方式**：`cargo test` 通过，升级后旧配置文件正常加载

---

### 任务 B5：后台同步间隔可配置化 🟢

- **问题**：`lib.rs` 第 83 行硬编码 30 秒轮询间隔
  ```rust
  tokio::time::sleep(std::time::Duration::from_secs(30)).await;
  ```
- **修改方案**：
  1. 在 `lib.rs` 中定义常量 `const BACKGROUND_SYNC_INTERVAL_SECS: u64 = 30`
  2. 或从 `tauri.conf.json` 读取配置
  3. 考虑根据最短 `sync_interval_minutes` 动态调整
- **涉及文件**：`src-tauri/src/lib.rs`
- **预估工时**：0.5h
- **验证方式**：`cargo build` 通过

---

## 五、UI/UX 改进

### 任务 U1：改进 Toast 通知系统 🟡

- **问题**：`App.svelte` 第 660-674 行，`showSuccessToast` 使用纯 DOM 操作，非组件化，无暗色模式适配
- **当前代码**：
  ```typescript
  function showSuccessToast(message: string) {
      const toast = document.createElement('div')
      toast.className = 'toast success'
      toast.textContent = message
      document.body.appendChild(toast)
      setTimeout(() => { toast.classList.add('show') }, 10)
      setTimeout(() => {
          toast.classList.remove('show')
          setTimeout(() => toast.remove(), 300)
      }, 2000)
  }
  ```
  调用位置共 8 处（DNS 刷新、版本检查、远程同步、分组操作、导入导出等）
- **修改方案**：
  1. 创建 `src/lib/components/Toast.svelte` 声明式组件
  2. 支持多种类型：`success`、`error`、`warning`、`info`
  3. 支持暗色模式样式
  4. 支持堆叠显示多个 toast
  5. 使用 Svelte transition 实现进入/退出动画
  6. 在 `App.svelte` 中用 toast 状态数组替代 DOM 操作
- **涉及文件**：新建 `Toast.svelte`，修改 `App.svelte`
- **预估工时**：2h
- **验证方式**：各操作触发 toast 正常显示，暗色模式下样式正确

---

### 任务 U2：细化 Loading 状态 🟡

- **问题**：`App.svelte` 第 817-821 行，全局单一 `isLoading` 全屏遮罩，粒度过粗
  ```html
  {#if isLoading}
      <div class="loading-overlay">
          <div class="spinner"></div>
      </div>
  {/if}
  ```
- **修改方案**：
  1. 保留全局 `isLoading` 用于初始加载
  2. 为各操作添加局部 loading 状态（`isCreatingScheme`、`isDeletingScheme`、`isImporting`、`isExporting` 等）
  3. 在对应按钮上显示 spinner + 禁用态
  4. 移除全屏遮罩，改为按钮级或区域级 loading
- **涉及文件**：`src/lib/App.svelte`、`src/lib/components/Sidebar.svelte`
- **预估工时**：2h
- **验证方式**：操作时不再出现全屏遮罩，按钮显示 loading 态

---

### 任务 U3：统一字体定义 🟢

- **问题**：5 处字体定义互相矛盾

  | 位置 | 行号 | 字体栈 |
  |------|------|--------|
  | `index.html` | 9 | `-apple-system, BlinkMacSystemFont, 'Segoe UI', 'Microsoft YaHei', 'PingFang SC', 'SimSun', sans-serif` |
  | `src/app.css` | 2 | `Inter, system-ui, Avenir, Helvetica, Arial, sans-serif` |
  | `src/App.svelte` | 13 | `-apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', 'Oxygen', 'Ubuntu', ...` |
  | `src/lib/App.svelte` | 1043, 1071 | `'Microsoft YaHei', 'PingFang SC', sans-serif` |
  | `src/lib/components/Editor.svelte` | 407 | `'Microsoft YaHei', 'PingFang SC', sans-serif` |

- **修改方案**：
  1. 在 `src/app.css` 的 `:root` 中定义统一的 CSS 变量：`--font-family`
  2. 值设为：`'Microsoft YaHei', 'PingFang SC', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif`
  3. 删除其他 4 处的字体定义，统一引用 `var(--font-family)`
- **涉及文件**：`src/app.css`、`src/App.svelte`、`src/lib/App.svelte`、`src/lib/components/Editor.svelte`、`index.html`
- **预估工时**：0.5h
- **验证方式**：各平台字体显示一致

---

### 任务 U4：统一组件事件风格 🟢

- **问题**：`SyncLogModal` 使用回调 prop `onClose`，其他 6 个组件使用 `createEventDispatcher`
- **当前代码**（`SyncLogModal.svelte` 第 11 行）：
  ```typescript
  export let onClose: () => void
  ```
- **修改方案**：
  1. 将 `SyncLogModal` 的 `onClose` 回调 prop 改为 `createEventDispatcher` 派发 `close` 事件
  2. 更新 `App.svelte` 第 990 行的调用方式
  3. 或者在 F2（迁移到 Runes）中统一改为回调 props 风格
- **涉及文件**：`src/lib/components/SyncLogModal.svelte`、`src/lib/App.svelte`
- **预估工时**：0.5h
- **验证方式**：同步日志弹窗正常打开和关闭

---

## 六、执行路线图

```
第一阶段：安全与稳定性（~5h）
  S1 → S2 → S3 → B2 → S5
  目标：消除安全隐患和数据丢失风险

第二阶段：后端质量提升（~8h）
  B1 → B4 → B3 → S4 → B5
  目标：建立统一错误处理体系，增加测试覆盖

第三阶段：前端架构重构（~12h）
  F3 → F4 → F1 → F6 → F2 → F5
  目标：拆分上帝组件，建立清晰的代码架构

第四阶段：UI/UX 打磨（~5h）
  U1 → U2 → U3 → U4
  目标：提升用户体验和代码一致性
```

**总预估工时：约 30 小时**

---

## 七、风险评估

| 风险 | 概率 | 影响 | 缓解措施 |
|------|:----:|:----:|---------|
| F1 拆分状态管理导致功能回归 | 中 | 高 | 每拆分一个 store 后立即手动测试全部功能 |
| B1 统一错误处理破坏前端错误展示 | 低 | 中 | 保持 `Result<_, String>` 返回格式不变 |
| F2 Runes 迁移引入兼容性问题 | 低 | 中 | Svelte 5 向后兼容，逐步迁移 |
| S1 设置 CSP 导致资源加载失败 | 中 | 中 | 先在开发环境测试，逐步收紧策略 |
| B3 添加测试发现隐藏 bug | 高 | 低 | 记录 bug 但不阻塞计划执行 |

---

## 八、验收标准

| # | 标准 | 验证方式 |
|---|------|---------|
| 1 | `cargo build` 零错误零警告 | 编译输出检查 |
| 2 | `cargo test` 全部通过（目标 20+ 测试） | 测试输出检查 |
| 3 | `npm run build` 前端构建无错误 | 构建输出检查 |
| 4 | 所有 19 个 Tauri IPC 命令功能正常 | 手动功能测试 |
| 5 | CSP 策略生效且无违规 | DevTools Console 检查 |
| 6 | `App.svelte` script 部分不超过 200 行 | 代码行数统计 |
| 7 | 无死代码（`RemoteSyncModal` 已删除） | 全局搜索确认 |
| 8 | 字体定义统一为 1 处 | 全局搜索确认 |
| 9 | Toast 组件化且支持暗色模式 | 手动 UI 测试 |
| 10 | 无全屏 loading 遮罩（初始加载除外） | 手动 UI 测试 |

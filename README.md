# Rust SwitchHost

一个跨平台的 Hosts 文件管理和快速切换工具，使用 Rust + Tauri 2.x + Svelte 构建。

## ✨ 特性

- 🚀 **轻量高效**：基于 Tauri 2.x，应用体积小，内存占用低
- 🎨 **现代 UI**：使用 Svelte 5 构建，响应式设计
- 📝 **语法高亮**：支持 IP、域名、注释的语法高亮显示
- 🔄 **快速切换**：一键切换不同的 hosts 配置方案
- 💾 **自动备份**：修改前自动备份原始配置
- 🎯 **系统托盘**：托盘图标快速访问和切换

## 🛠️ 技术栈

### 后端
- **Rust** - 系统级编程语言
- **Tauri 2.x** - 跨平台桌面应用框架
- **serde** - 序列化/反序列化
- **tokio** - 异步运行时

### 前端
- **Svelte 5** - 前端框架
- **TypeScript** - 类型安全
- **Vite** - 构建工具

## 📦 安装与运行

### 前置要求

- Rust 1.70+
- Node.js 18+
- npm 或 pnpm

### 开发模式

```bash
# 安装依赖
npm install

# 启动开发服务器
npm run tauri dev
```

### 构建生产版本

```bash
# 构建应用
npm run tauri build
```

构建产物位于 `src-tauri/target/release/bundle/` 目录。

### GitHub Release 自动发布

项目已包含 GitHub Actions 发布工作流：

- 工作流文件：`.github/workflows/release.yml`
- 触发方式：推送 `v*` 格式的 tag，例如 `v0.0.4`
- 发布结果：自动构建 Windows / macOS / Linux 安装包，并上传到对应的 GitHub Release
- 默认 Linux runner：GitHub-hosted `ubuntu-22.04`
- 额外 Linux 发行版：支持通过 self-hosted runner 扩展，例如 Rocky Linux

示例发布流程：

```bash
git tag v0.0.4
git push githost v0.0.4
```

也可以在 GitHub Actions 页面手动触发 `Release` 工作流，并填写 `release_tag`。

### Rocky Linux 等其他 Linux 发行版

GitHub-hosted Linux runner 默认只有 Ubuntu，因此像 Rocky Linux、AlmaLinux 这类发行版，需要使用 self-hosted runner。

当前工作流已经预留了 Rocky Linux 发布 job，启用方式如下：

1. 准备一台 Rocky Linux 机器，并安装 GitHub Actions self-hosted runner
2. 给该 runner 打上标签：`self-hosted`, `linux`, `x64`, `rocky-linux`
3. 在 GitHub 仓库变量中设置 `ENABLE_SELF_HOSTED_LINUX=true`

启用后，发布流程会额外在 Rocky Linux runner 上构建并把产物上传到同一个 GitHub Release。

### Gitee 国内镜像发布

项目已新增 Gitee Go 流水线配置文件：

- `/.workflow/gitee-release.yml`

用途：

- 监听 Gitee 仓库中的 `v*` tag
- 在 Gitee 流水线中构建 Linux 安装包
- 自动创建或更新 Gitee Release
- 将构建产物作为国内镜像附件上传到 Gitee Release

建议定位：

- GitHub Release：主发布源，同时用于应用内在线升级
- Gitee Release：国内镜像下载源

Gitee 侧需要额外配置的环境变量：

- `GITEE_RELEASE_TOKEN`
- `GITEE_RELEASE_OWNER`
- `GITEE_RELEASE_REPO`

推荐值：

- `GITEE_RELEASE_OWNER=chengccn1`
- `GITEE_RELEASE_REPO=rust-switchhost`

说明：

- `GITEE_RELEASE_TOKEN` 需要使用 Gitee 个人访问令牌，并至少具备当前仓库的发布权限
- 当前 Gitee 流水线默认构建 Linux 产物，适合作为国内镜像发布
- 在线升级仍然继续使用 GitHub Release 与签名产物

### 在线升级配置

项目已经接入 Tauri Updater，可实现应用内一键下载安装并自动重启升级。

要让 GitHub Release 产出可用于在线升级的安装包与 `latest.json`，还需要在仓库 Secrets 中配置：

- `TAURI_SIGNING_PRIVATE_KEY`
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`

说明：

- 公钥已经写入 [src-tauri/tauri.conf.json](/d:/opensource/rust-switchhost/src-tauri/tauri.conf.json)
- 私钥不要提交到仓库，建议仅保存在本地或 CI Secrets 中
- Release 工作流发布成功后，应用内“检查更新”即可执行“一键下载安装并重启”

## 📁 项目结构

```
rust-switchhost/
├── src/                    # 前端代码
│   ├── lib/               # Svelte 组件
│   ├── app.css            # 全局样式
│   └── main.ts            # 入口文件
├── src-tauri/             # Rust 后端
│   ├── src/
│   │   ├── commands/      # Tauri 命令
│   │   ├── hosts/         # hosts 文件处理
│   │   ├── schemes/       # 方案管理
│   │   └── lib.rs         # 主入口
│   ├── Cargo.toml         # Rust 依赖
│   └── tauri.conf.json    # Tauri 配置
├── package.json           # Node.js 依赖
└── PLAN.md               # 开发计划
```

## 🎯 核心功能

### 已完成 ✅

- [x] 项目初始化和配置
- [x] Hosts 文件解析器
- [x] 方案管理模块（CRUD）
- [x] 文件读写和备份机制
- [x] 权限错误处理
- [x] 基础 UI 框架

## 🔧 开发指南

### 添加新的 Tauri 命令

1. 在 `src-tauri/src/commands/` 中定义命令函数
2. 在 `src-tauri/src/lib.rs` 的 `invoke_handler` 中注册
3. 在前端使用 `invoke()` 调用

### 修改 hosts 文件路径

编辑 `src-tauri/src/hosts/mod.rs` 中的 `get_hosts_path()` 函数。

## ⚠️ 注意事项

### 权限要求

- **Windows**: 需要管理员权限修改 `C:\Windows\System32\drivers\etc\hosts`
- **macOS/Linux**: 需要 `sudo` 权限修改 `/etc/hosts`

### 首次运行

应用会尝试创建配置目录：
- Windows: `%APPDATA%\rust-switchhost\`
- macOS: `~/Library/Application Support/rust-switchhost/`
- Linux: `~/.config/rust-switchhost/`

## 📝 开发日志

### 2026-04-07

- ✅ 完成项目初始化
- ✅ 实现 hosts 文件解析和写入
- ✅ 实现方案管理核心功能
- ✅ 配置文件自动备份
- ✅ 权限错误优雅处理
- ✅ Rust 后端编译通过

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

## 📄 许可证

MIT License

## 🙏 致谢

- [Tauri](https://tauri.app/) - 跨平台桌面应用框架
- [Svelte](https://svelte.dev/) - 前端框架
- [SwitchHosts](https://github.com/oldj/SwitchHosts) - 灵感来源

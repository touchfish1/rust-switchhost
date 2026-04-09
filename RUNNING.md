# 🚀 Rust SwitchHost 运行指南

## 📋 前置要求

### 必需软件
- **Rust**: 1.70+ 版本
- **Node.js**: 18+ 版本
- **npm**: 9+ 版本

### 检查环境
```bash
# 检查 Rust 版本
rustc --version
cargo --version

# 检查 Node.js 版本
node --version
npm --version
```

## 🛠️ 安装步骤

### 1. 安装依赖
```bash
# 进入项目目录
cd d:\opensource\rust-switchhost

# 安装 Node.js 依赖
npm install
```

### 2. 首次编译（可选）
```bash
# 编译 Rust 后端（检查是否有错误）
cd src-tauri
cargo build
cd ..
```

## 🎮 运行方式

### 方式一：开发模式（推荐）

```bash
# 启动开发服务器（带热重载）
npm run tauri dev
```

**特点**：
- ✅ 自动打开应用窗口
- ✅ 前端热重载（修改代码自动刷新）
- ✅ 后端自动重新编译
- ✅ 开发者工具自动打开

**首次运行**：
- Rust 编译需要 2-5 分钟
- 后续启动只需几秒钟

### 方式二：仅前端开发

```bash
# 只启动前端开发服务器
npm run dev
```

访问：http://localhost:1420

### 方式三：生产构建

```bash
# 构建生产版本
npm run tauri build
```

**输出位置**：
- Windows: `src-tauri/target/release/bundle/msi/`
- macOS: `src-tauri/target/release/bundle/dmg/`
- Linux: `src-tauri/target/release/bundle/deb/`

## 🐛 常见问题

### 1. 编译错误

**问题**：Rust 编译失败
```bash
# 清理并重新编译
cd src-tauri
cargo clean
cargo build
```

**问题**：依赖下载慢
```bash
# 使用国内镜像（可选）
# 编辑 ~/.cargo/config
[source.crates-io]
replace-with = 'ustc'

[source.ustc]
registry = "https://mirrors.ustc.edu.cn/crates.io-index"
```

### 2. 权限问题

**Windows**：
- 需要管理员权限修改 hosts 文件
- 右键 → 以管理员身份运行

**macOS/Linux**：
- 修改 `/etc/hosts` 需要额外权限
- 不要直接用 `sudo` 启动图形界面应用，否则在 Ubuntu 中可能出现 `unable to get session bus`
- 应保持应用在当前桌面用户会话中运行，再通过 `pkexec` / polkit 或系统权限配置完成写入

### 3. 沙箱环境限制

**问题**：在 Trae IDE 中运行失败
```
Failed to create config directory: 拒绝访问
```

**解决方案**：
- 这是沙箱安全限制，正常现象
- 在非沙箱环境中运行即可
- 代码已实现优雅降级

### 4. 端口占用

**问题**：端口 1420 被占用
```bash
# Windows
netstat -ano | findstr :1420
taskkill /PID <进程ID> /F

# macOS/Linux
lsof -i :1420
kill -9 <PID>
```

### 5. WebView 问题

**Windows**：
- 需要安装 WebView2 运行时
- Windows 10/11 通常已预装

**Linux**：
```bash
# Ubuntu/Debian
sudo apt install libwebkit2gtk-4.0-dev

# Fedora
sudo dnf install webkit2gtk3-devel
```

## 📂 项目结构

```
rust-switchhost/
├── src/                    # 前端代码
│   ├── lib/
│   │   ├── App.svelte     # 主应用
│   │   └── components/    # 组件
│   └── main.ts
├── src-tauri/             # Rust 后端
│   ├── src/
│   │   ├── commands/      # API 命令
│   │   ├── hosts/         # hosts 处理
│   │   ├── schemes/       # 方案管理
│   │   └── tray.rs        # 系统托盘
│   └── tauri.conf.json    # 配置
└── package.json
```

## 🎯 开发工作流

### 推荐流程

1. **启动开发服务器**
   ```bash
   npm run tauri dev
   ```

2. **修改代码**
   - 前端：修改 `src/` 下的文件
   - 后端：修改 `src-tauri/src/` 下的文件

3. **查看效果**
   - 前端修改：自动刷新
   - 后端修改：自动重新编译

4. **测试功能**
   - 创建方案
   - 编辑 hosts
   - 切换主题
   - 测试托盘

### 调试技巧

**前端调试**：
- 开发模式下自动打开 DevTools
- 按 F12 或 Ctrl+Shift+I

**后端调试**：
```rust
// 在代码中添加日志
println!("Debug info: {:?}", data);
eprintln!("Error: {}", error);
```

**查看日志**：
- 终端输出
- 浏览器控制台

## 📦 构建发布

### Windows

```bash
# 构建 MSI 安装包
npm run tauri build

# 输出
src-tauri/target/release/bundle/msi/Rust SwitchHost_0.1.0_x64.msi
```

### macOS

```bash
# 构建 DMG
npm run tauri build

# 输出
src-tauri/target/release/bundle/dmg/Rust SwitchHost_0.1.0_x64.dmg
```

### Linux

```bash
# 构建 DEB/RPM
npm run tauri build

# 输出
src-tauri/target/release/bundle/deb/rust-switchhost_0.1.0_amd64.deb
```

## 🔧 高级配置

### 自定义端口

编辑 `vite.config.ts`：
```typescript
server: {
  port: 3000,  // 修改端口
}
```

### 修改窗口大小

编辑 `src-tauri/tauri.conf.json`：
```json
{
  "app": {
    "windows": [{
      "width": 1200,
      "height": 800
    }]
  }
}
```

### 添加新功能

1. **后端**：在 `src-tauri/src/commands/` 添加命令
2. **前端**：在 `src/lib/` 调用 `invoke()`

## 📚 相关文档

- [Tauri 官方文档](https://tauri.app/)
- [Svelte 文档](https://svelte.dev/)
- [Rust 文档](https://doc.rust-lang.org/)

## 💡 提示

1. **首次运行**：编译时间较长，请耐心等待
2. **开发模式**：推荐使用，支持热重载
3. **权限问题**：以管理员身份运行
4. **沙箱限制**：在非沙箱环境测试

## 🆘 获取帮助

遇到问题？检查以下步骤：

1. ✅ 检查 Rust 和 Node.js 版本
2. ✅ 运行 `npm install` 安装依赖
3. ✅ 检查端口是否被占用
4. ✅ 查看终端错误信息
5. ✅ 检查浏览器控制台

---

**快速启动命令**：
```bash
npm install && npm run tauri dev
```

祝开发顺利！🎉

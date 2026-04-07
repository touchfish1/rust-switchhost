# Rust SwitchHost 应用开发计划

## 一、项目概述

开发一个跨平台的 Hosts 文件管理和快速切换工具，使用 Rust + Tauri 技术栈，实现轻量、高性能的桌面应用。

## 二、技术选型

### 2.1 核心框架
- **Tauri 2.x** - 推荐方案
  - 优势：
    - 极小的应用体积（2.5-10MB vs Electron 80-120MB）
    - 低内存占用（30-40MB vs Electron 100-400MB）
    - 使用系统原生 WebView（macOS: WebKit, Windows: WebView2, Linux: WebKitGTK）
    - 活跃的社区支持（104,959 GitHub Stars）
    - 支持跨平台（Windows、macOS、Linux、iOS、Android）
  - 适用性：非常适合 SwitchHost 这类工具型应用

### 2.2 前端技术栈
- **Svelte** 或 **React** + TypeScript
  - Svelte 优势：编译时优化，运行时更轻量
  - React 优势：生态成熟，组件库丰富
- **UI 组件库**：
  - Svelte: skeleton、shadcn-svelte
  - React: Ant Design、shadcn/ui
- **代码编辑器**：Monaco Editor 或 CodeMirror（语法高亮）

### 2.3 后端技术栈
- **Rust** 核心库
  - `serde` - 序列化/反序列化
  - `tokio` - 异步运行时
  - `dirs` - 跨平台目录路径
  - `notify` - 文件系统监听

## 三、功能规划

### 3.1 核心功能（MVP）

#### 1. Hosts 文件管理
- 读取系统 hosts 文件
  - Windows: `C:\Windows\System32\drivers\etc\hosts`
  - macOS/Linux: `/etc/hosts`
- 写入 hosts 文件（需要管理员权限）
- 自动备份机制
- 文件格式解析和验证

#### 2. 方案管理
- 创建多个 hosts 配置方案
- 方案的增删改查
- 方案快速切换
- 多选模式（组合多个方案）
- 方案启用/禁用状态管理

#### 3. 编辑器功能
- 语法高亮（IP、域名、注释）
- 行号显示
- 快速注释/取消注释（Ctrl+/ 或 Cmd+/）
- 实时语法检查
- 撤销/重做

#### 4. 系统托盘
- 托盘图标显示
- 右键菜单快速切换方案
- 左键点击显示主窗口
- 托盘菜单项：
  - 当前方案列表
  - 显示/隐藏主窗口
  - 退出应用

### 3.2 进阶功能

#### 5. 远程 Hosts
- 从 URL 获取远程 hosts 配置
- 定时自动同步
- 本地缓存

#### 6. 导入导出
- 导出所有方案为 JSON/YAML
- 导入配置文件
- 与团队成员共享配置

#### 7. 用户体验
- 深色/浅色主题切换
- 多语言支持（中文、英文）
- 系统启动自启
- DNS 缓存刷新提示

## 四、技术实现方案

### 4.1 项目结构
```
rust-switchhost/
├── src-tauri/              # Rust 后端
│   ├── src/
│   │   ├── main.rs         # 入口文件
│   │   ├── lib.rs          # 库文件
│   │   ├── hosts/          # hosts 文件操作模块
│   │   │   ├── mod.rs
│   │   │   ├── parser.rs   # 解析器
│   │   │   ├── writer.rs   # 写入器
│   │   │   └── backup.rs   # 备份管理
│   │   ├── schemes/        # 方案管理模块
│   │   │   ├── mod.rs
│   │   │   └── manager.rs
│   │   ├── tray/           # 系统托盘模块
│   │   │   ├── mod.rs
│   │   │   └── menu.rs
│   │   └── commands/       # Tauri 命令
│   │       ├── mod.rs
│   │       ├── hosts.rs
│   │       └── schemes.rs
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                    # 前端代码
│   ├── lib/
│   ├── components/
│   ├── stores/
│   └── app.tsx/svelte
├── package.json
└── README.md
```

### 4.2 关键技术点

#### 1. 管理员权限处理

**Windows 方案：**
```rust
// 检测是否有管理员权限
fn is_elevated() -> bool {
    // 使用 Windows API 检测
}

// 请求管理员权限（需要重新启动进程）
fn request_elevation() {
    // 使用 ShellExecuteW 以管理员身份重新运行
}
```

**macOS/Linux 方案：**
```rust
// 使用 sudo 提示
// 或使用 polkit (Linux)
```

**推荐方案：**
- 首次运行时检测权限
- 提示用户以管理员身份运行
- 保存配置到用户目录，只在切换时需要权限

#### 2. Hosts 文件解析

```rust
pub struct HostEntry {
    pub ip: String,
    pub domain: String,
    pub comment: Option<String>,
    pub enabled: bool,
}

pub fn parse_hosts(content: &str) -> Vec<HostEntry> {
    // 解析 hosts 文件格式
    // 支持 # 注释
    // 支持 IP 域名 格式
}
```

#### 3. 系统托盘实现（Tauri 2.x）

```rust
use tauri::{
    tray::TrayIconBuilder,
    menu::{Menu, MenuItem},
    Manager,
};

fn setup_tray(app: &AppHandle) {
    let menu = Menu::new(app);
    let schemes = get_schemes();
    
    for scheme in schemes {
        let item = MenuItem::new(app, &scheme.name, true, None);
        menu.append(&item);
    }
    
    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap())
        .menu(&menu)
        .on_menu_event(|app, event| {
            // 处理菜单点击事件
            switch_scheme(&event.id);
        })
        .build(app);
}
```

#### 4. 文件监听

```rust
use notify::{Watcher, RecursiveMode, watcher};

fn watch_hosts_file(path: &Path) {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = watcher(tx, Duration::from_secs(2)).unwrap();
    
    watcher.watch(path, RecursiveMode::NonRecursive).unwrap();
    
    // 监听文件变化，通知前端更新
}
```

### 4.3 数据存储

**方案配置文件格式（JSON）：**
```json
{
  "version": "1.0",
  "schemes": [
    {
      "id": "uuid-1",
      "name": "开发环境",
      "content": "127.0.0.1 localhost\n192.168.1.100 dev.example.com",
      "enabled": true,
      "created_at": "2025-01-01T00:00:00Z",
      "updated_at": "2025-01-01T00:00:00Z"
    }
  ],
  "active_schemes": ["uuid-1"]
}
```

**存储位置：**
- Windows: `%APPDATA%/rust-switchhost/`
- macOS: `~/Library/Application Support/rust-switchhost/`
- Linux: `~/.config/rust-switchhost/`

## 五、UI 设计方案

### 5.1 主界面布局

```
┌─────────────────────────────────────────────────────────┐
│  Rust SwitchHost                    ─ □ ✕              │
├───────────────┬─────────────────────────────────────────┤
│               │                                         │
│  方案列表      │         编辑器区域                       │
│               │                                         │
│  ☑ 开发环境    │  1  127.0.0.1  localhost               │
│  ☐ 测试环境    │  2  192.168.1.100  dev.example.com     │
│  ☐ 生产环境    │  3  # 注释行                            │
│  ☐ 远程配置    │  4  10.0.0.1  test.example.com         │
│               │                                         │
│  [+ 新建方案]  │                                         │
│               │                                         │
├───────────────┴─────────────────────────────────────────┤
│  状态: 已启用 "开发环境" | 最后更新: 2025-01-01 12:00    │
└─────────────────────────────────────────────────────────┘
```

### 5.2 系统托盘菜单

```
┌─────────────────────┐
│ ● 开发环境          │  ← 当前激活的方案
├─────────────────────┤
│ ○ 测试环境          │
│ ○ 生产环境          │
│ ○ 远程配置          │
├─────────────────────┤
│ 显示主窗口          │
│ 退出                │
└─────────────────────┘
```

### 5.3 配色方案

**浅色主题：**
- 背景: #FFFFFF
- 侧边栏: #F5F5F5
- 主色调: #1890FF (蓝色)
- IP 地址: #52C41A (绿色)
- 域名: #1890FF (蓝色)
- 注释: #8C8C8C (灰色)

**深色主题：**
- 背景: #1E1E1E
- 侧边栏: #252526
- 主色调: #1890FF
- IP 地址: #4EC9B0
- 域名: #569CD6
- 注释: #6A9955

## 六、开发计划

### 阶段一：项目初始化（1-2 天）
- [ ] 使用 Tauri CLI 创建项目
- [ ] 配置前端框架（Svelte/React）
- [ ] 配置 Tauri 权限和设置
- [ ] 设置开发环境

### 阶段二：核心功能开发（3-5 天）
- [ ] 实现 hosts 文件解析器
- [ ] 实现方案管理模块
- [ ] 实现文件读写（含权限处理）
- [ ] 实现备份恢复机制

### 阶段三：UI 开发（3-4 天）
- [ ] 实现主界面布局
- [ ] 实现方案列表组件
- [ ] 集成代码编辑器（Monaco/CodeMirror）
- [ ] 实现语法高亮
- [ ] 实现深色/浅色主题

### 阶段四：系统托盘（1-2 天）
- [ ] 实现托盘图标
- [ ] 实现托盘菜单
- [ ] 实现快速切换功能

### 阶段五：进阶功能（2-3 天）
- [ ] 实现远程 hosts 同步
- [ ] 实现导入导出
- [ ] 实现多语言支持
- [ ] 实现开机自启

### 阶段六：测试与优化（2-3 天）
- [ ] 跨平台测试（Windows、macOS、Linux）
- [ ] 性能优化
- [ ] 错误处理完善
- [ ] 用户体验优化

### 阶段七：打包发布（1-2 天）
- [ ] 配置打包脚本
- [ ] 生成安装包
- [ ] 编写用户文档

**预计总工期：13-21 天**

## 七、风险与挑战

### 7.1 技术风险
1. **权限问题**
   - 风险：不同操作系统的权限机制不同
   - 解决方案：提供清晰的权限引导，优雅降级

2. **文件锁定**
   - 风险：hosts 文件可能被其他程序锁定
   - 解决方案：重试机制，友好的错误提示

3. **跨平台兼容性**
   - 风险：不同平台的 hosts 文件格式可能有细微差异
   - 解决方案：充分的跨平台测试

### 7.2 用户体验风险
1. **误操作**
   - 风险：用户可能误删重要配置
   - 解决方案：自动备份、确认对话框、撤销功能

2. **网络问题**
   - 风险：远程 hosts 同步失败
   - 解决方案：本地缓存、离线模式、重试机制

## 八、参考资源

### 8.1 技术文档
- [Tauri 官方文档](https://tauri.app/)
- [Tauri 系统托盘指南](https://tauri.app/zh-cn/learn/system-tray/)
- [Rust hosts 文件处理库](https://crates.io/crates/hostcraft-core)

### 8.2 类似项目
- [SwitchHosts (Electron)](https://github.com/oldj/SwitchHosts) - 原版 SwitchHosts
- 功能参考和 UI 设计灵感来源

### 8.3 Rust GUI 框架对比
| 框架 | 优势 | 劣势 | 适用场景 |
|------|------|------|----------|
| Tauri | 轻量、Web 技术栈、活跃社区 | 需要 WebView | 桌面应用首选 |
| Iced | 纯 Rust、类型安全 | 学习曲线较陡 | 纯 Rust 项目 |
| Slint | 声明式 UI、高性能 | 商业许可限制 | 嵌入式/桌面 |
| egui | 即时模式、简单快速 | 功能相对简单 | 工具应用 |

## 九、下一步行动

确认此计划后，将立即开始执行：

1. 初始化 Tauri 项目
2. 选择前端框架（建议 Svelte，更轻量）
3. 实现核心的 hosts 文件解析功能
4. 逐步完成各阶段开发任务

---

**计划制定时间：** 2026-04-07  
**预计开始时间：** 待确认  
**预计完成时间：** 开始后 13-21 天

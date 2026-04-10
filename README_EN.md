# Rust SwitchHost

A cross-platform Hosts file manager and fast switching tool built with Rust, Tauri 2.x, and Svelte 5.

[中文 README](./README.md)

## Features

- Lightweight desktop app built on Tauri 2.x
- Modern Svelte 5 UI with dark mode support
- Hosts syntax highlighting for IPs, domains, and comments
- Fast scheme switching with immediate application
- Automatic backup before hosts changes
- System tray integration
- Remote URL schemes with manual and scheduled sync
- Built-in updater support

## Tech Stack

### Backend

- Rust
- Tauri 2.x
- serde
- tokio

### Frontend

- Svelte 5
- TypeScript
- Vite

## Requirements

- Rust 1.70+
- Node.js 18+
- npm or pnpm

## Development

```bash
npm install
npm run tauri dev
```

## Production Build

```bash
npm run tauri build
```

Build outputs are generated under `src-tauri/target/release/bundle/`.

## Release Workflow

### GitHub Release

The repository includes an automated GitHub Actions release workflow:

- Workflow file: `.github/workflows/release.yml`
- Trigger: push a tag in the format `v*`, for example `v0.0.25`
- Result: builds Windows, macOS, and Linux packages and uploads them to the matching GitHub Release

Example:

```bash
git tag v0.0.25
git push githost v0.0.25
```

You can also trigger the workflow manually from GitHub Actions and provide `release_tag`.

### Gitee Mirror Release

The project also includes a Gitee Go workflow:

- Workflow file: `.workflow/gitee-release.yml`

It can:

- watch `v*` tags from the Gitee repository
- build Linux packages
- create or update a Gitee Release
- upload Linux bundles as mirror assets for users in mainland China

Recommended positioning:

- GitHub Release: primary release source
- Gitee Release: mainland China mirror source

### Updater Signing

Tauri updater artifacts require signing keys in CI:

- `TAURI_SIGNING_PRIVATE_KEY`
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`

The updater public key is already configured in [src-tauri/tauri.conf.json](/d:/opensource/rust-switchhost/src-tauri/tauri.conf.json).

## Project Structure

```text
rust-switchhost/
├── src/                  # frontend code
│   ├── lib/              # Svelte components, stores, services, types
│   ├── app.css           # global styles
│   └── main.ts           # app entry
├── src-tauri/            # Rust backend
│   ├── src/
│   │   ├── commands/     # Tauri IPC commands
│   │   ├── hosts/        # hosts file operations
│   │   ├── schemes/      # scheme management
│   │   └── lib.rs        # backend entry
│   ├── Cargo.toml
│   └── tauri.conf.json
├── package.json
└── README.md
```

## Core Capabilities

- Scheme CRUD management
- Hosts parsing, writing, and backup
- Permission checks and error handling
- Current hosts viewer
- DNS flush actions
- Online update checks and install flow
- Remote sync logs and retry handling

## Development Notes

### Adding a New Tauri Command

1. Add the command in `src-tauri/src/commands/`
2. Register it in `src-tauri/src/lib.rs`
3. Call it from the frontend through `invoke()` or the service layer

### Hosts File Path

Edit `get_hosts_path()` in `src-tauri/src/hosts/mod.rs`.

## Permissions

- Windows: modifying `C:\Windows\System32\drivers\etc\hosts` requires administrator permissions
- macOS/Linux: modifying `/etc/hosts` requires elevated privileges, but the GUI app should not be started with `sudo`; prefer `pkexec`, polkit, or platform-native privilege escalation

## First Run

The app creates its config directory automatically:

- Windows: `%APPDATA%\rust-switchhost\`
- macOS: `~/Library/Application Support/rust-switchhost/`
- Linux: `~/.config/rust-switchhost/`

## Contributing

Issues and pull requests are welcome.

## License

MIT License

## Acknowledgements

- [Tauri](https://tauri.app/)
- [Svelte](https://svelte.dev/)
- [SwitchHosts](https://github.com/oldj/SwitchHosts)

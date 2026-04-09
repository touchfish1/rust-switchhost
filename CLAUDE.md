# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

### Development
```bash
# Start dev server (frontend + Tauri backend)
npm run tauri dev

# Build for production
npm run tauri build

# Frontend only (Vite dev server on port 1420)
npm run dev

# Frontend only build
npm run build
```

### Rust Backend
```bash
# Check Rust code without building
cd src-tauri && cargo check

# Run Rust tests
cd src-tauri && cargo test

# Run a specific test
cd src-tauri && cargo test <test_name>

# Format Rust code
cd src-tauri && cargo fmt

# Lint Rust code
cd src-tauri && cargo clippy
```

## Architecture

This is a **Tauri 2.x** desktop app — Svelte 5 frontend communicating with a Rust backend via `invoke()` calls.

### Rust Backend (`src-tauri/src/`)

- **`lib.rs`** — App entry point: registers `AppState` (wrapping `SchemeManager` in a `Mutex`), sets up the tray, and registers all Tauri command handlers.
- **`hosts/`** — Low-level hosts file I/O:
  - `mod.rs` — Platform-aware path resolution (`C:\Windows\System32\drivers\etc\hosts` on Windows, `/etc/hosts` elsewhere), read/write/backup functions. Every write auto-creates a timestamped backup under the app config dir.
  - `parser.rs` / `writer.rs` — Parse hosts file lines into `HostEntry` structs and serialize back.
- **`schemes/`** — Configuration management:
  - `mod.rs` — `Scheme` and `SchemeConfig` data types (serialized to JSON).
  - `manager.rs` — `SchemeManager` loads/saves `schemes.json` from the platform config dir (`%APPDATA%/rust-switchhost/` on Windows). `switch_scheme()` writes scheme content directly to the system hosts file.
- **`commands/`** — Tauri IPC command handlers:
  - `hosts.rs` — `get_hosts_content`, `write_hosts_content`, `backup_hosts` (note: `backup_hosts` is defined but not registered in the `invoke_handler`)
  - `schemes.rs` — CRUD for schemes + `switch_scheme` + async `fetch_remote_hosts` (via `reqwest`); also defines `AppState`
  - `mod.rs` — exposes a `greet` command (registered but unused in the UI)
- **`tray.rs`** — System tray with "Show Window" and "Quit" menu items.

### Frontend (`src/`)

- **`src/lib/App.svelte`** — Root component: owns all state (schemes list, active scheme, editor content, dark mode). All Tauri `invoke()` calls are made here and passed down as props/events.
- **`src/lib/components/`**:
  - `Sidebar.svelte` — Scheme list with select/create/delete/rename actions
  - `Editor.svelte` — CodeMirror 6 editor with custom hosts syntax highlighting
  - `Modal.svelte` — Reusable modal for create/delete confirmations
  - `ThemeToggle.svelte` — Dark/light mode toggle; theme persisted in `localStorage`
- **`src/main.ts`** / **`src/App.svelte`** — Svelte app mount point.

### Data Flow

```
Frontend invoke() -> Tauri command -> AppState (Mutex<SchemeManager>) -> hosts file / JSON config
```

`AppState` is shared Tauri managed state. All scheme mutations go through `SchemeManager`, which persists to `schemes.json`. `switch_scheme` is the only command that writes to the system hosts file (requiring admin/elevated privileges on Windows).

### Key Dependencies

- **Rust**: `tauri 2`, `serde`/`serde_json`, `dirs 5`, `chrono`, `uuid`, `reqwest 0.12`, `tokio`, `thiserror`, `anyhow`, `tauri-plugin-autostart` (desktop only)
- **Frontend**: Svelte 5, CodeMirror 6 (`@codemirror/*`, `@lezer/*`), `@tauri-apps/api 2`

### Permissions Note

Writing to the system hosts file requires administrator privileges on Windows. The app does not auto-elevate — if the write fails, a Rust `io::Error` propagates back to the frontend as a `String` error.

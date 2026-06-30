# WSL Manager

A lightweight, visual manager for Windows Subsystem for Linux (WSL) instances.

English | [简体中文](./README.zh-CN.md)

Built with **Tauri v2 + React 19 + TypeScript + Rust + Tokio**.

> **Note:** This application is designed for Windows. The Tauri backend calls `wsl.exe` directly, so running the packaged app on other platforms is not supported. Development on Linux/macOS is possible via the built-in mock mode.

---

## Features

- **Instance list** — view all WSL instances with state, WSL version, default marker, distribution name, and WSL 2 VM IP.
- **Start / Stop** — start or terminate individual instances.
- **Shutdown all** — stop every running instance and the WSL 2 VM.
- **Set default** — change the default WSL distribution.
- **Open terminal** — launch Windows Terminal (or the default console host) directly into an instance.
- **Delete instance** — unregister a distribution with a confirmation dialog.
- **Install distribution** — install a new distribution from the Microsoft Store / online source with a custom instance name.
- **Rename instance** — rename an existing instance while preserving its data.
- **WSL version info** — display WSL, kernel, and Windows versions, with fallback to `wsl --status` for older releases.
- **Async operations** — all long-running commands run in the Tokio runtime so the UI stays responsive.

Features planned for future releases:

- Import / export instances
- Switch WSL versions (1 ↔ 2)
- Disk usage display
- System tray / global hotkey

---

## Tech Stack

| Layer | Technology |
| --- | --- |
| Frontend | React 19, TypeScript, Vite |
| Backend | Rust, Tauri v2, Tokio |
| System calls | `wsl.exe`, `wt.exe` via `tokio::process::Command` |
| Styling | Plain CSS |
| Package manager | npm |

---

## Quick Start

### Requirements

- Windows 10 version 1903+ or Windows 11
- WSL enabled (WSL 2 recommended)
- [Rust](https://www.rust-lang.org/tools/install)
- [Node.js](https://nodejs.org/) 18+
- npm

### Install dependencies

```bash
npm install
```

### Run in development

```bash
npm run tauri:dev
```

### Build the release bundle

```bash
npm run tauri:build
```

The installer and executable are placed in `src-tauri/target/release/bundle/`.

### Build on Windows (recommended)

Tauri builds for the host operating system, so generating a Windows `.exe` / `.msi` should be done on Windows:

```powershell
.\scripts\build-windows.ps1
```

`build-windows.bat` is also available for convenience.

---

## Mock Mode

When developing on Linux or macOS, enable mock mode so the Rust backend returns fixture data instead of calling `wsl.exe`:

```bash
# PowerShell
$env:WSL_MANAGER_MOCK = "1"
npm run tauri:dev

# Linux / macOS
WSL_MANAGER_MOCK=1 cargo test --manifest-path src-tauri/Cargo.toml
```

Mock data lives in `src-tauri/fixtures/wsl-list.json`.

> The app does **not** automatically fall back to mock mode on non-Windows systems. If WSL is unavailable and mock mode is disabled, the app reports that WSL is not available.

---

## Project Structure

```
wsl-manager/
├── docs/                   # Requirements, architecture, and acceptance docs
├── public/                 # Static assets
├── scripts/                # Windows build helpers
├── src/                    # Frontend source
│   ├── api.ts              # Tauri command wrappers
│   ├── App.tsx             # Main UI
│   ├── App.css             # Styles
│   ├── main.tsx            # Entry point
│   └── types.ts            # TypeScript types
├── src-tauri/              # Rust + Tauri backend
│   ├── src/
│   │   ├── commands.rs     # Tauri command entry points
│   │   ├── lib.rs          # Application bootstrap
│   │   ├── logger.rs       # Simple file logger
│   │   ├── main.rs         # Binary entry point
│   │   ├── models.rs       # Data models
│   │   └── wsl.rs          # WSL command layer and parsers
│   ├── fixtures/           # Test and mock data
│   ├── Cargo.toml
│   └── tauri.conf.json
├── index.html
├── package.json
├── tsconfig.json
└── vite.config.ts
```

---

## Testing

Run Rust unit tests:

```bash
cd src-tauri && cargo test
```

Run frontend type checks and build:

```bash
npm run build
```

---

## Documentation

- [AGENTS.md](./AGENTS.md) — Coding agent guidelines (Chinese)
- [docs/PRD.md](./docs/PRD.md) — Product requirements
- [docs/ARCHITECTURE.md](./docs/ARCHITECTURE.md) — Architecture design
- [docs/WSL-INTERFACE.md](./docs/WSL-INTERFACE.md) — WSL command interface specification
- [docs/SETUP.md](./docs/SETUP.md) — Development and build guide
- [docs/UI-MOCKUPS.md](./docs/UI-MOCKUPS.md) — UI wireframes
- [docs/ACCEPTANCE.md](./docs/ACCEPTANCE.md) — Acceptance criteria

---

## License

[MIT](./LICENSE)

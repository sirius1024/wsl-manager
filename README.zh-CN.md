# WSL Manager

一个运行在 Windows 上的可视化 WSL（Windows Subsystem for Linux）实例管理工具。

[English](./README.md) | 简体中文

基于 **Tauri v2 + React 19 + TypeScript + Rust + Tokio** 构建。

> **注意：** 本应用面向 Windows 设计，Tauri 后端会直接调用 `wsl.exe`，因此不支持在其他操作系统上运行打包后的应用。在 Linux/macOS 上开发时可使用内置的 mock 模式。

---

## 功能特性

- **实例列表** — 展示所有 WSL 实例的状态、WSL 版本、默认标记、发行版名称以及 WSL2 VM IP。
- **启动 / 停止** — 启动或终止单个实例。
- **全部停止** — 停止所有运行中的实例及 WSL 2 VM。
- **设为默认** — 更改默认 WSL 发行版。
- **打开终端** — 直接打开 Windows Terminal（或默认控制台主机）并进入实例 Shell。
- **删除实例** — 注销发行版，带二次确认。
- **安装发行版** — 从 Microsoft Store / 在线源安装新发行版，支持自定义实例名。
- **重命名实例** — 重命名已有实例并保留数据。
- **WSL 版本信息** — 显示 WSL、内核、Windows 版本，旧版 WSL 会 fallback 到 `wsl --status`。
- **异步操作** — 所有耗时命令在 Tokio 运行时中执行，UI 不卡顿。

后续版本计划支持的功能：

- 导入 / 导出实例
- WSL 版本切换（1 ↔ 2）
- 磁盘占用展示
- 系统托盘 / 全局热键

---

## 技术栈

| 层级 | 技术 |
| --- | --- |
| 前端 | React 19、TypeScript、Vite |
| 后端 | Rust、Tauri v2、Tokio |
| 系统调用 | 通过 `tokio::process::Command` 调用 `wsl.exe`、`wt.exe` |
| 样式 | 原生 CSS |
| 包管理器 | npm |

---

## 快速开始

### 环境要求

- Windows 10 版本 1903+ 或 Windows 11
- 已启用 WSL（建议 WSL 2）
- [Rust](https://www.rust-lang.org/tools/install)
- [Node.js](https://nodejs.org/) 18+
- npm

### 安装依赖

```bash
npm install
```

### 开发运行

```bash
npm run tauri:dev
```

### 构建发布版

```bash
npm run tauri:build
```

安装包和可执行文件会生成在 `src-tauri/target/release/bundle/`。

### 在 Windows 上构建（推荐）

Tauri 会针对当前操作系统构建，因此建议在 Windows 上生成 Windows 安装包：

```powershell
.\scripts\build-windows.ps1
```

也可以使用同目录下的 `build-windows.bat`。

---

## Mock 模式

在 Linux 或 macOS 上开发时，开启 mock 模式后 Rust 后端会返回 fixture 数据，而不是调用 `wsl.exe`：

```bash
# PowerShell
$env:WSL_MANAGER_MOCK = "1"
npm run tauri:dev

# Linux / macOS
WSL_MANAGER_MOCK=1 cargo test --manifest-path src-tauri/Cargo.toml
```

Mock 数据位于 `src-tauri/fixtures/wsl-list.json`。

> 应用**不会**在非 Windows 系统上自动 fallback 到 mock 模式。如果 WSL 不可用且未开启 mock，应用会明确提示 WSL 不可用。

---

## 项目结构

```
wsl-manager/
├── docs/                   # 需求/架构/验收文档
├── public/                 # 静态资源
├── scripts/                # Windows 构建辅助脚本
├── src/                    # 前端源码
│   ├── api.ts              # Tauri 命令调用层
│   ├── App.tsx             # 主界面
│   ├── App.css             # 样式
│   ├── main.tsx            # 入口
│   └── types.ts            # TypeScript 类型
├── src-tauri/              # Rust + Tauri 后端
│   ├── src/
│   │   ├── commands.rs     # Tauri command 入口
│   │   ├── lib.rs          # 应用初始化
│   │   ├── logger.rs       # 简单文件日志
│   │   ├── main.rs         # 二进制入口
│   │   ├── models.rs       # 数据模型
│   │   └── wsl.rs          # WSL 命令封装与解析
│   ├── fixtures/           # 测试/mock 数据
│   ├── Cargo.toml
│   └── tauri.conf.json
├── index.html
├── package.json
├── tsconfig.json
└── vite.config.ts
```

---

## 测试

运行 Rust 单元测试：

```bash
cd src-tauri && cargo test
```

运行前端类型检查与构建：

```bash
npm run build
```

---

## 文档目录

- [AGENTS.md](./AGENTS.md) — Coding Agent 工作规范
- [docs/PRD.md](./docs/PRD.md) — 产品需求
- [docs/ARCHITECTURE.md](./docs/ARCHITECTURE.md) — 架构设计
- [docs/WSL-INTERFACE.md](./docs/WSL-INTERFACE.md) — WSL 交互规范
- [docs/SETUP.md](./docs/SETUP.md) — 开发与构建指南
- [docs/UI-MOCKUPS.md](./docs/UI-MOCKUPS.md) — UI 线框图
- [docs/ACCEPTANCE.md](./docs/ACCEPTANCE.md) — 验收标准

---

## License

[MIT](./LICENSE)

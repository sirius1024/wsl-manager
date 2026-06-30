# 架构设计

## 总体结构

```
┌─────────────────────────────────────┐
│           UI Layer                  │  React 19 + TypeScript
│  (视图、组件、状态管理、用户交互)      │
└──────────────┬──────────────────────┘
               │ Tauri invoke
┌──────────────▼──────────────────────┐
│        Command / Service Layer      │  Rust (Tauri v2) + Tokio
│  (WSL 调用封装、参数校验、错误处理)    │
└──────────────┬──────────────────────┘
               │ tokio::process::Command
┌──────────────▼──────────────────────┐
│         WSL / Windows Layer         │  wsl.exe, wt.exe
│  (系统调用、文件系统)                  │  Windows Terminal
└─────────────────────────────────────┘
```

## 技术栈

- **前端**：React 19 + TypeScript + Vite
- **后端**：Rust + Tauri v2 + Tokio
- **系统调用**：`tokio::process::Command` 异步调用 `wsl.exe` / `wt.exe`
- **UI 样式**：原生 CSS
- **包管理器**：npm

## 模块划分

| 路径 | 职责 |
| --- | --- |
| `src/App.tsx` | 主界面、实例列表、详情面板、操作按钮、创建/重命名弹窗 |
| `src/api.ts` | 封装所有 Tauri `invoke` 调用 |
| `src/types.ts` | TypeScript 类型定义 |
| `src-tauri/src/commands.rs` | Tauri command 入口，暴露给前端 |
| `src-tauri/src/wsl.rs` | WSL 命令封装、输出解析、参数校验、mock 模式、创建/重命名逻辑 |
| `src-tauri/src/models.rs` | `WslInstance`、`WslVersion`、`InstanceState` 数据模型 |
| `src-tauri/src/logger.rs` | 简单的本地文件日志，便于排查 WSL 输出问题 |
| `src-tauri/fixtures/` | 测试与 mock 数据 |

## 数据流

1. 前端通过 `src/api.ts` 调用 Tauri command。
2. Rust `commands.rs` 接收请求并调用 `wsl.rs`。
3. `wsl.rs` 校验参数、异步执行 `wsl.exe`、解析输出。
4. 结果序列化为 JSON 返回前端。
5. React 状态更新，UI 刷新。

## 异步设计

- 所有 Tauri command 均为 `async fn`，运行在 Tokio runtime 上。
- `wsl.exe` 调用使用 `tokio::process::Command`，不会阻塞 UI 主线程。
- 列表刷新、创建实例、启动/停止、重命名等耗时操作均异步执行；前端通过独立的 `refreshing` / `working` 状态控制按钮禁用，避免界面整体卡死。
- `list_instances` 内部使用 `tokio::task::JoinSet` 并行获取每个实例的发行版与 IP。

## 数据模型

```rust
pub struct WslInstance {
    pub name: String,
    pub state: InstanceState,         // Running / Stopped / Unknown
    pub version: u8,
    pub default: bool,
    pub distribution: Option<String>, // from /etc/os-release PRETTY_NAME
    pub ip_address: Option<String>,   // WSL2 VM IP from `hostname -I`
}

pub struct WslVersion {
    pub wsl_version: Option<String>,
    pub kernel_version: Option<String>,
    pub windows_version: Option<String>,
    pub fields: HashMap<String, String>,
    pub raw: String,
}
```

对应 TypeScript 类型见 `src/types.ts`。

## 安全设计

- 所有 WSL 命令均通过白名单函数构造，禁止字符串拼接命令。
- 危险操作（删除、停止全部、重命名）需要前端二次确认 + Rust 层参数校验。
- 发行版名称经过白名单校验：仅允许 `A-Z a-z 0-9 . - _`，且不能以 `-` 开头，防止被解析为命令行选项。
- 创建实例调用 `wsl --install -d <name> --no-launch` spawn 执行，由 WSL 自身触发 UAC，应用不直接提权。
- 默认以普通用户权限运行；需要管理员权限的功能（如版本切换、部分导入/导出场景）在 MVP 中不做。
- Mock 模式通过环境变量 `WSL_MANAGER_MOCK=1` 显式开启，不再自动 fallback，避免在真实环境掩盖问题。

## 配置持久化

- 用户配置后续存储在 `%APPDATA%/com.example.wslmanager/settings.json`。
- 当前 MVP 版本配置项较少，状态保存在 React 内存中。

# WSL 交互规范

所有与 WSL 的交互集中在 Rust `src-tauri/src/wsl.rs` 中实现，前端通过 Tauri command 间接调用。

## 命令白名单（MVP）

| 操作 | 实际命令 | 实现位置 | 备注 |
| --- | --- | --- | --- |
| 列出实例 | `wsl --list --verbose` + `wsl -d <name> -e cat /etc/os-release` + `wsl -d <name> -e hostname -I` | `wsl::list_instances` | 文本解析，并行获取发行版与 IP |
| 启动实例 | `wsl -d <name> -e true` | `wsl::start_instance` | spawn，不阻塞，CREATE_NO_WINDOW |
| 停止实例 | `wsl --terminate <name>` | `wsl::stop_instance` | 异步等待 |
| 停止全部 | `wsl --shutdown` | `wsl::shutdown` | 危险操作，需二次确认 |
| 打开终端 | `wt.exe wsl -d <name>` / `wsl -d <name>` | `wsl::open_terminal` | 优先 Windows Terminal，fallback 默认终端 |
| 设置默认 | `wsl --set-default <name>` | `wsl::set_default_instance` | |
| 删除实例 | `wsl --unregister <name>` | `wsl::delete_instance` | 危险操作，需二次确认 |
| WSL 版本 | `wsl --version` / `wsl --status` | `wsl::get_wsl_version` | 解析为结构化字段，`--status` 作为旧版 fallback |
| 可安装发行版列表 | `wsl --list --online` | `wsl::list_online_distributions` | |
| 创建实例 | `wsl --install -d <distro> [--name <name>] --no-launch` | `wsl::install_distribution` | spawn 执行，由 WSL 触发 UAC/下载 |
| 重命名实例 | `wsl --export` + `wsl --import` + `wsl --unregister` | `wsl::rename_instance` | 数据保留，实例类型变为 imported |

## 命令调用路径

- 通过 `wsl_program()` 统一解析 `wsl.exe` 路径。
- 优先使用 `%SystemRoot%\System32\wsl.exe`。
- 32 位进程 fallback 到 `%SystemRoot%\Sysnative\wsl.exe`。
- 非 Windows 平台返回 `WslError::NotAvailable`。
- 所有命令均通过 `tokio::process::Command` 异步执行；spawn 类命令使用 `CREATE_NO_WINDOW` 避免闪黑框。

## 输出解析策略

1. `wsl --list --verbose` 输出为文本表格。
2. 解析器先尝试通过表头 `NAME` / `VERSION` 定位列边界。
3. 如果表头被本地化，fallback 到 whitespace 切分。
4. 行首 `*` 表示默认实例。
5. 状态优先使用 `STATE` 列值（英文系统上为 `Running` / `Stopped`）。
6. 非英文系统上 `STATE` 可能被本地化，此时用 `wsl --list --running --quiet` 的运行中名称集合做交叉校验，未命中者标记为 `Stopped`。
7. 所有输出统一按 UTF-8 BOM / UTF-16 LE/BE / 严格 UTF-8 自动检测解码；中文 Windows 等 ANSI 代码页 fallback 到 GBK。
8. `wsl --version` 输出按 `Key: Value` 解析为结构化字段；不支持 `--version` 的旧版 WSL fallback 到 `--status`。
9. `wsl --list --online` 输出跳过表头行，取每行第一个 token 作为发行版名称。
10. 错误通过 exit code + stderr 识别。

## 异步执行

- 所有 command 函数均为 `async fn`，使用 `tokio::process::Command` 执行外部进程。
- `list_instances` 内部使用 `tokio::task::JoinSet` 并行获取每个实例的发行版与 IP。
- `install_distribution` / `start_instance` / `open_terminal` 等 spawn 类命令立即返回，由操作系统/WSL 后台执行。

## Mock 模式

满足以下条件时启用 mock：

- 环境变量 `WSL_MANAGER_MOCK=1`

Mock 模式下：

- `list_instances` 返回 `src-tauri/fixtures/wsl-list.json` 中的静态数据。
- `get_wsl_version` 返回 fixture 版本信息。
- 其他命令直接返回成功，不执行真实系统调用。

## 实例详情补充

`list_instances` 在解析完基础列表后，会并行执行以下命令为每个实例补充详情：

- **发行版名称**：`wsl -d <name> -e cat /etc/os-release`，解析 `PRETTY_NAME`。
- **IP 地址**：仅对 `Running` 实例执行 `wsl -d <name> -e hostname -I`，取第一个地址。

这些命令失败时不会导致列表加载失败，对应字段保持 `None`。

## IP 地址说明

WSL 2 使用一个轻量级 VM，所有 WSL 2 发行版共享同一个 VM 网络命名空间。因此：

- `wsl -d <name> -e hostname -I` 返回的是 **WSL 2 VM 的 IP**，不是单个发行版独占的 IP。
- 同一台机器上多个 Running 的 WSL 2 发行版显示的 IP 地址相同，这是预期行为。
- UI 中该字段标注为 **WSL2 VM IP**，避免误解。

## 创建实例

`install_distribution(distro, install_name)` 调用：

```
wsl --install -d <distro> --name <install_name> --no-launch
```

- `distro` 来自 `wsl --list --online`（如 `Ubuntu`、`Debian`）。
- `install_name` 为用户自定义实例名；留空则使用 WSL 默认名称。
- 该命令 spawn 后立即返回，WSL 在后台下载安装，可能触发 UAC。
- 安装进度由 WSL 自身管理，前端不会等待完成；用户可稍后手动刷新列表。

## 重命名实例

WSL 没有原生 rename 命令，`rename_instance(old_name, new_name)` 通过以下步骤实现：

1. 从当前列表中查找旧实例，记录 WSL 版本和默认标记。
2. 停止旧实例：`wsl --terminate <old_name>`（失败不阻断）。
3. 导出到临时 tar：`wsl --export <old_name> <temp.tar>`。
4. 以新名称导入：`wsl --import <new_name> %LOCALAPPDATA%\wsl\<new_name> <temp.tar> --version <version>`。
5. 若旧实例为默认，则设置新实例为默认：`wsl --set-default <new_name>`。
6. 注销旧实例：`wsl --unregister <old_name>`。
7. 清理临时 tar。

注意：

- 数据完整保留。
- 重命名后的实例变为 imported 类型，不再有 Store 发行版的默认用户等配置。
- 这是一个危险操作，前端需要二次确认。
- 临时文件存放在 `%LOCALAPPDATA%\wsl-manager-temp\`。

## 数据模型

```rust
pub struct WslInstance {
    pub name: String,
    pub state: InstanceState,         // Running / Stopped / Unknown
    pub version: u8,
    pub default: bool,
    pub distribution: Option<String>, // e.g. "Ubuntu 22.04 LTS"
    pub ip_address: Option<String>,   // WSL2 VM IP, e.g. "172.21.21.2"
}
```

对应 TypeScript 类型见 `src/types.ts`。

## 参数校验

- 所有接受发行版名称的函数必须先调用 `validate_distro_name`。
- 名称仅允许 `A-Z a-z 0-9 . - _`。
- 名称不能为空，且不能以 `-` 开头。
- 校验失败返回 `WslError::InvalidName`。

## 权限

- 普通权限可执行：列出、启动、停止、停止全部、打开终端、设置默认、删除。
- 创建实例会 spawn `wsl --install`，由 WSL 自身请求管理员权限（UAC）。
- 需要管理员权限的功能（切换 WSL 版本、部分导入/导出场景）在 MVP 中不做。

## 错误处理

| 错误场景 | 预期行为 |
| --- | --- |
| WSL 未安装 | 返回 `WslError::NotAvailable`，前端提示安装 WSL |
| 实例不存在 | 返回 `WslError::InstanceNotFound` / `WslError::CommandFailed`，前端刷新列表 |
| 操作被拒绝 | 提示需要管理员权限 |
| 命令超时 | 当前版本由用户手动重试 |
| 名称不合法 | 返回 `WslError::InvalidName`，前端提示 |
| 输出解析失败 | 返回 `WslError::ParseError`，并记录原始输出到日志 |

## 测试

- Rust 单元测试使用 `src-tauri/fixtures/wsl-list.txt` 与内联样本验证文本解析。
- 使用 `cargo test` 运行。

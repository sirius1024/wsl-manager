# AGENTS.md - Coding Agent 工作规范

本文档面向负责实现本项目的 coding agent，请优先阅读本文档与 `docs/PRD.md`。

## 项目目标

开发一款运行在 Windows 上的可视化 WSL 实例管理器，核心需求见 `docs/PRD.md`。

## 技术栈

- 前端/UI: React 19 + TypeScript + Vite
- 后端/系统调用: Rust + Tauri v2 + Tokio
- 构建工具: npm + Cargo
- 包管理器: npm

## 代码规范

- 代码注释和命名使用英文；提交信息可用英文或中文。
- 遵循项目现有目录结构，新增模块前先阅读 `docs/ARCHITECTURE.md`。
- 所有对 `wsl.exe` 的系统调用必须封装在统一的命令层（见 `docs/WSL-INTERFACE.md`），禁止在 UI 层直接执行命令。
- 对会修改用户数据的操作（删除、导出覆盖、注册表修改、重命名等）必须二次确认。
- 不要自动运行需要管理员权限的命令；如需提权，必须显式提示用户。

## 安全与权限

- 任何 WSL 命令执行前先做参数校验，防止命令注入。
- 不允许在没有用户确认的情况下执行：
  - `wsl --unregister`
  - `wsl --export` 覆盖已有文件
  - 修改注册表或系统配置
- 使用最小权限原则：默认以普通用户运行，需要管理员时再申请。

## 测试要求

- 每个对 WSL 命令的封装函数必须有单元测试（使用 mock 数据或 fixture）。
- UI 组件使用假数据/ fixture 进行独立测试。
- 在提交前至少运行一次完整的 lint + 单元测试。

## 变更流程

1. 阅读相关需求文档。
2. 如需修改架构或接口，先更新 `docs/ARCHITECTURE.md` 或 `docs/WSL-INTERFACE.md`。
3. 实现功能并补充测试。
4. 更新 `README.md` 与 `README.zh-CN.md` 中受影响的说明。
5. 不要自行 `git commit`、`git push` 或修改版本控制历史，除非用户明确要求。

## 有问题时

遇到以下情况请停下来询问用户，不要猜测：

- 需求冲突或缺失
- 需要引入新的系统级权限
- 需要调用文档中未列出的 WSL 命令
- 需要修改产品核心定位或技术栈

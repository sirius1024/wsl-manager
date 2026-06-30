# 开发与构建指南

## 环境要求

- Windows 10 版本 1903+ 或 Windows 11
- 已启用 WSL（建议 WSL 2）
- [Rust](https://www.rust-lang.org/tools/install)
- [Node.js](https://nodejs.org/) 18+
- npm

## 安装依赖

```bash
npm install
```

Rust 依赖会在首次运行 `tauri dev` / `tauri build` 时自动下载。

## 开发运行

```bash
npm run tauri:dev
```

Vite 开发服务器监听 `http://localhost:1420`，Tauri 窗口自动打开。

## 构建发布版

```bash
npm run tauri:build
```

构建产物位于 `src-tauri/target/release/bundle/`。

## 代码检查

```bash
npm run build        # TypeScript 类型检查 + Vite 构建
cd src-tauri && cargo test  # Rust 单元测试
```

完整应用构建（需要 Rust）：

```bash
npm run tauri:build
```

## 目录结构

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

## 调试技巧

- 在非 Windows 环境上开发时，设置 `WSL_MANAGER_MOCK=1` 强制启用 mock 模式。
- 使用 `cargo test` 运行 Rust 单元测试，验证 WSL 输出解析逻辑。
- 日志默认写入应用所在目录的 `wsl-manager.log`，可用于排查 WSL 输出编码或解析问题。
- 使用 Windows Sandbox 或 Hyper-V 虚拟机测试需要管理员权限的功能。

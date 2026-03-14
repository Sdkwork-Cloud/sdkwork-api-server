# 安装准备

本页说明 Windows、Linux、macOS 上的本地环境准备与仓库初始化步骤。

## 必需工具

所有平台都需要：

- Rust stable 与 Cargo
- Node.js 20+
- pnpm 10+

可选：

- PostgreSQL 15+
- Tauri CLI

安装 Tauri CLI：

```bash
cargo install tauri-cli
```

## 平台说明

### Windows

建议：

- 使用 `rustup` 安装 Rust
- Node.js 20+
- PowerShell
- 如果使用 Tauri，确保系统可用 WebView2

### Linux

建议：

- 使用 `rustup` 安装 Rust
- Node.js 20+
- 通过 Corepack 或独立方式安装 pnpm
- 如果使用 Tauri，准备好系统 WebView 相关依赖

### macOS

建议：

- 使用 `rustup` 安装 Rust
- Node.js 20+
- pnpm
- 安装 Xcode Command Line Tools

## 克隆与安装

```bash
git clone https://github.com/Sdkwork-Cloud/sdkwork-api-server.git
cd sdkwork-api-server
```

安装 console 依赖：

```bash
pnpm --dir console install
```

安装 docs 依赖：

```bash
pnpm --dir docs install
```

## 校验工具链

```bash
rustc --version
cargo --version
node --version
pnpm --version
```

如果需要 PostgreSQL：

```bash
psql --version
```

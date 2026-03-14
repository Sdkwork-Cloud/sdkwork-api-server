# Release 构建

本页说明如何构建并启动 release 版本的服务、浏览器前端和可选的 Tauri 桌面包。

## 构建 Rust 服务

```bash
cargo build --release -p admin-api-service -p gateway-service -p portal-api-service
```

输出位于 `target/release/`。

Windows 可执行文件：

- `target/release/admin-api-service.exe`
- `target/release/gateway-service.exe`
- `target/release/portal-api-service.exe`

Linux / macOS 可执行文件：

- `target/release/admin-api-service`
- `target/release/gateway-service`
- `target/release/portal-api-service`

## 启动 Release 二进制

### Windows

```powershell
$env:SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
.\target\release\admin-api-service.exe
```

```powershell
$env:SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
.\target\release\gateway-service.exe
```

```powershell
$env:SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
.\target\release\portal-api-service.exe
```

### Linux 或 macOS

```bash
export SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
./target/release/admin-api-service
```

```bash
export SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
./target/release/gateway-service
```

```bash
export SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
./target/release/portal-api-service
```

## 构建浏览器前端

```bash
pnpm --dir console install
pnpm --dir console build
```

输出目录：

- `console/dist/`

本地预览：

```bash
pnpm --dir console preview
```

## 构建 Tauri 桌面包

```bash
pnpm --dir console tauri:build
```

## Release 校验

```bash
cargo build --release -p admin-api-service -p gateway-service -p portal-api-service
pnpm --dir console build
pnpm --dir docs build
```

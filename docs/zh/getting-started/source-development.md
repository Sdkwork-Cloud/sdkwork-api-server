# 源码运行

本页汇总 Windows、Linux、macOS 上推荐的源码启动流程。

## 默认端口

| 运行面 | 默认绑定地址 | 用途 |
|---|---|---|
| gateway | `127.0.0.1:8080` | OpenAI 兼容 `/v1/*` |
| admin | `127.0.0.1:8081` | 控制平面 |
| portal | `127.0.0.1:8082` | 公共门户 |
| console | `127.0.0.1:5173` | 浏览器与 Tauri 共用前端开发服务 |

## 最快启动方式

### Windows

浏览器模式：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1
```

桌面模式：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1 -Tauri
```

### Linux 或 macOS

浏览器模式：

```bash
node scripts/dev/start-workspace.mjs
```

桌面模式：

```bash
node scripts/dev/start-workspace.mjs --tauri
```

## 分面启动

仅后端：

```bash
node scripts/dev/start-stack.mjs
```

仅浏览器控制台：

```bash
node scripts/dev/start-console.mjs
```

仅桌面控制台：

```bash
node scripts/dev/start-console.mjs --tauri
```

## SQLite 开发

本地默认数据库：

- `sqlite://sdkwork-api-server.db`

默认启动时会自动创建数据库并执行迁移。

## PostgreSQL 开发

```bash
node scripts/dev/start-workspace.mjs \
  --database-url "postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_server"
```

Windows：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1 `
  -DatabaseUrl "postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_server"
```

## 原生命令

```bash
export SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
cargo run -p admin-api-service
```

```bash
export SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
cargo run -p gateway-service
```

```bash
export SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
cargo run -p portal-api-service
```

```bash
pnpm --dir console dev
```

```bash
pnpm --dir console tauri:dev
```

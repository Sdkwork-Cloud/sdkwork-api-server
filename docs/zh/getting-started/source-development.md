# 源码运行

本页说明 Windows、Linux、macOS 上推荐的源码启动方式。

如果你只想看一页就理解每个脚本的职责和完整生命周期，请优先阅读 [脚本生命周期](/zh/getting-started/script-lifecycle)。本页聚焦开发者日常的源码工作流。

## 你需要先知道的两套端口

仓库里存在两套默认端口。

### 托管源码脚本默认端口

这是更新后的源码辅助层默认使用的端口：

| 运行面 | 默认绑定 | 用途 |
|---|---|---|
| gateway | `127.0.0.1:9980` | OpenAI 兼容 `/v1/*` 流量 |
| admin | `127.0.0.1:9981` | 运维控制平面 |
| portal | `127.0.0.1:9982` | 公共认证、dashboard、用量、计费和 API key 生命周期 |
| web host | `0.0.0.0:9983` | Pingora 对外统一交付 admin 和 portal |
| admin Web 应用 | `127.0.0.1:5173` | 独立 admin 浏览器开发服务 |
| portal Web 应用 | `127.0.0.1:5174` | 独立 portal 浏览器开发服务 |

### 服务二进制内建默认端口

如果你直接运行服务二进制，而不通过辅助脚本覆盖，仍然会使用内建默认值：

- gateway：`127.0.0.1:8080`
- admin：`127.0.0.1:8081`
- portal：`127.0.0.1:8082`

## 本地配置根目录

独立服务会从本地 SDKWork 配置根目录读取配置：

- Linux / macOS：`~/.sdkwork/router/`
- Windows：`%USERPROFILE%\\.sdkwork\\router\\`

即使目录为空，服务也能依靠内建默认值启动。

## 选择一种源码启动方式

### 方案 1：托管源码启动

当你需要稳定的运行目录、PID 管理、格式化启动摘要和默认统一浏览器入口时，优先使用这个方案。

Linux / macOS：

```bash
./bin/start-dev.sh
```

Windows：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\start-dev.ps1
```

特点：

- 默认模式是 preview，内置 Pingora Web Host 会成为主要统一浏览器入口
- 运行时状态写入 `artifacts/runtime/dev/`
- 启动日志会打印统一入口、独立服务地址、bootstrap 身份引导和日志文件路径
- 使用 `./bin/stop-dev.sh` 或 `.\bin\stop-dev.ps1` 停止

启动后的主要地址：

- 统一 admin：`http://127.0.0.1:9983/admin/`
- 统一 portal：`http://127.0.0.1:9983/portal/`
- 统一 gateway 健康检查：`http://127.0.0.1:9983/api/v1/health`
- 独立 gateway 健康检查：`http://127.0.0.1:9980/health`
- 独立 admin 健康检查：`http://127.0.0.1:9981/admin/health`
- 独立 portal 健康检查：`http://127.0.0.1:9982/portal/health`

如果你明确想使用独立 Vite 开发服务，而不是统一 Pingora Host：

Linux / macOS：

```bash
./bin/start-dev.sh --browser
```

Windows：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\start-dev.ps1 -Browser
```

### 方案 2：原生源码工作区启动

当你想直接使用原始工作区启动器，并在当前终端里以前台进程控制时，使用这个方案。

Windows：

browser 模式：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1
```

preview 模式：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1 -Preview
```

desktop 模式：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1 -Tauri
```

Linux / macOS：

browser 模式：

```bash
node scripts/dev/start-workspace.mjs
```

preview 模式：

```bash
node scripts/dev/start-workspace.mjs --preview
```

desktop 模式：

```bash
node scripts/dev/start-workspace.mjs --tauri
```

各模式行为：

- browser 模式：
  - 后端位于 `9980`、`9981`、`9982`
  - admin 位于 `http://127.0.0.1:5173/admin/`
  - portal 位于 `http://127.0.0.1:5174/portal/`
- preview 模式：
  - 后端位于 `9980`、`9981`、`9982`
  - 统一 Web Host 位于 `http://127.0.0.1:9983/admin/` 和 `http://127.0.0.1:9983/portal/`
- tauri 模式：
  - 后端位于 `9980`、`9981`、`9982`
  - admin 桌面壳启动，同时 Pingora 继续在 `9983` 提供浏览器访问

原生工作区启动器现在也会打印启动摘要，包含：

- 当前模式
- 前端访问入口
- 独立服务访问入口
- 当前激活的 bootstrap 身份引导

## 分面启动

仅后端服务：

```bash
node scripts/dev/start-stack.mjs
```

仅 admin 应用：

```bash
node scripts/dev/start-admin.mjs
```

仅桌面 admin：

```bash
node scripts/dev/start-admin.mjs --tauri
```

仅 portal 应用：

```bash
node scripts/dev/start-portal.mjs
```

仅统一 Web Host：

```bash
node scripts/dev/start-web.mjs
```

指定对外绑定地址启动统一 Web Host：

```bash
node scripts/dev/start-web.mjs --bind 0.0.0.0:9983
```

Windows 也提供 PowerShell 包装：

- `scripts/dev/start-servers.ps1`
- `scripts/dev/start-workspace.ps1`

## 存储选择

### SQLite 开发

对于原生辅助脚本，如果你不传 `--database-url`，服务会遵循本地配置根目录行为：

- Linux / macOS：`~/.sdkwork/router/sdkwork-api-server.db`
- Windows：`%USERPROFILE%\\.sdkwork\\router\\sdkwork-api-server.db`

对于 `bin/start-dev.*`，托管开发态使用自己独立的可写数据库路径：

- `artifacts/runtime/dev/data/sdkwork-api-router-dev.db`

### PostgreSQL 开发

为 admin、gateway、portal 统一传入共享的 PostgreSQL 连接串：

```bash
node scripts/dev/start-workspace.mjs \
  --database-url "postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_server"
```

Windows：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1 `
  -DatabaseUrl "postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_server"
```

托管源码启动也支持数据库覆盖：

```bash
./bin/start-dev.sh --database-url "postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_server"
```

## 原生源码命令

如果你希望完全绕开辅助脚本，直接单独运行各运行面，可使用：

直接运行 Rust 服务：

```bash
cargo run -p admin-api-service
```

```bash
cargo run -p gateway-service
```

```bash
cargo run -p portal-api-service
```

如果你要显式覆盖本地默认数据库：

```bash
export SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
```

运行 admin 浏览器应用：

```bash
pnpm --dir apps/sdkwork-router-admin dev
```

从源码运行 Tauri：

```bash
pnpm --dir apps/sdkwork-router-admin tauri:dev
```

运行独立 portal 应用：

```bash
pnpm --dir apps/sdkwork-router-portal dev
```

运行 Pingora 公共 Web Host：

```bash
SDKWORK_WEB_BIND=0.0.0.0:9983 cargo run -p router-web-service
```

## 开发身份引导

本地开发流程不再依赖固定内建邮箱和密码。

当前约定是：

- 开发身份来自当前激活的 bootstrap profile
- 本地 `dev` profile 数据位于 `data/identities/dev.json`
- 默认的 `prod` bootstrap profile 不会注入开发身份

gateway 本身没有默认用户名密码。需要使用 portal 签发的 API key 去访问鉴权后的 gateway 接口。

## 推荐校验

在启动前后都推荐执行这些标准校验：

```bash
cargo fmt --all --check
cargo test --workspace -q -j 1
pnpm --dir apps/sdkwork-router-admin typecheck
pnpm --dir apps/sdkwork-router-admin build
pnpm --dir apps/sdkwork-router-portal typecheck
pnpm --dir apps/sdkwork-router-portal build
pnpm --dir docs typecheck
pnpm --dir docs build
```

## 常见注意事项

- 需要稳定单端口入口和托管运行目录时，用 `bin/start-dev.*`
- 需要源码级前台控制时，用 `scripts/dev/start-workspace.*`
- 需要前端热更新时，用 browser 模式；需要对外统一一个浏览器入口时，用 preview 模式
- 如果你的机器上 `998x` 端口仍被占用，请显式覆盖对应 bind 参数或环境变量

## 下一步

- 查看完整脚本职责与生命周期：
  - [脚本生命周期](/zh/getting-started/script-lifecycle)
- 查看编译产物：
  - [编译与打包](/zh/getting-started/build-and-packaging)
- 查看部署导向的发布二进制：
  - [发布构建](/zh/getting-started/release-builds)
- 深入理解系统架构：
  - [软件架构](/zh/architecture/software-architecture)

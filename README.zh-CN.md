# sdkwork-api-server

[English Guide](./README.md)

SDKWork API Server 是一个基于 Rust、Axum、React、pnpm 与 Tauri 的 OpenAI 兼容网关、控制平面和公共自助门户系统。

当前仓库围绕 4 个运行面组织：

- `gateway-service`
  - OpenAI 兼容 `/v1/*` 网关
- `admin-api-service`
  - 面向运营方的 `/admin/*` 控制平面
- `portal-api-service`
  - 面向外部用户的 `/portal/*` 注册、登录、workspace 查询和 API key 自助签发
- `console/`
  - 可被浏览器直接访问、也可被 Tauri 桌面壳承载的 React 前端

## 当前已经实现的能力

后端：

- OpenAI 兼容网关路由，支持 stateful 与 stateless 两种执行路径
- admin API，覆盖 tenants、projects、API keys、channels、proxy providers、credentials、models、routing、usage、billing、extensions
- public portal API：
  - `POST /portal/auth/register`
  - `POST /portal/auth/login`
  - `GET /portal/auth/me`
  - `GET /portal/workspace`
  - `GET /portal/api-keys`
  - `POST /portal/api-keys`
- SQLite 与 PostgreSQL 共用同一套存储抽象
- portal JWT 与 admin JWT 已隔离
- Prometheus metrics 与 HTTP tracing

前端：

- 按 package 拆分的 portal SDK、portal auth、portal dashboard 模块
- 按 package 拆分的 admin 控制台模块
- 浏览器与 Tauri 共享的 hash 路由：
  - `#/portal/register`
  - `#/portal/login`
  - `#/portal/dashboard`
  - `#/admin`

架构：

- controller / interface 层位于 `crates/sdkwork-api-interface-*`
- app / service 层位于 `crates/sdkwork-api-app-*`
- repository / storage 层位于 `crates/sdkwork-api-storage-*`
- React 壳层组合位于 `console/src/`
- 可复用前端业务模块位于 `console/packages/`

## 仓库结构

```text
.
|-- crates/                      # 领域、应用、接口、provider、runtime、storage 等 Rust crate
|-- services/
|   |-- admin-api-service/       # 独立 admin HTTP 服务
|   |-- gateway-service/         # 独立 OpenAI 兼容网关服务
|   `-- portal-api-service/      # 独立 public portal HTTP 服务
|-- console/                     # React + pnpm workspace + 可选 Tauri 桌面壳
|-- scripts/
|   `-- dev/                     # 跨平台启动辅助脚本
|-- docs/                        # 架构、计划与兼容性文档
`-- README.md                    # 英文操作文档
```

## 支持的平台

当前项目面向以下平台运行：

- Windows
- Linux
- macOS

Rust 服务本身是跨平台的。React 控制台可在三种平台上的浏览器中运行。Tauri 桌面模式是可选能力，并且与浏览器共享同一套前端路由，因此桌面启动时浏览器仍然可以同步访问界面。

## 环境要求

必需：

- Rust stable + Cargo
- Node.js 20+
- pnpm 10+

可选：

- PostgreSQL 15+，用于 PostgreSQL 部署
- Tauri CLI，用于桌面开发：
  - `cargo install tauri-cli`

## 默认端口

| 运行面 | 默认绑定地址 | 用途 |
|---|---|---|
| gateway | `127.0.0.1:8080` | OpenAI 兼容 `/v1/*` |
| admin | `127.0.0.1:8081` | 运营控制平面 |
| portal | `127.0.0.1:8082` | 外部用户注册、登录、自助 key 生命周期 |
| console | `127.0.0.1:5173` | 浏览器和 Tauri 共用前端开发服务器 |

## 推荐启动方式

优先使用仓库中的辅助脚本，而不是手动逐条拼命令。

| 工作流 | Windows | Linux / macOS |
|---|---|---|
| 启动后端服务 | `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-servers.ps1` | `node scripts/dev/start-stack.mjs` |
| 启动浏览器控制台 | `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-console.ps1` | `node scripts/dev/start-console.mjs` |
| 启动 Tauri 并保留浏览器访问 | `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-console.ps1 -Tauri` | `node scripts/dev/start-console.mjs --tauri` |
| 预览生产构建 | `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-console.ps1 -Preview` | `node scripts/dev/start-console.mjs --preview` |

说明：

- Windows 后端启动脚本会分别打开 `admin-api-service`、`gateway-service`、`portal-api-service` 的独立 PowerShell 窗口
- Node 脚本是跨平台入口，Windows、Linux、macOS 都可以直接使用
- Tauri dev 模式运行时，浏览器仍然可以访问 `http://127.0.0.1:5173`

## 使用 SQLite 快速启动

这是当前最直接、最完整的本地联调方式。

### Windows

终端 1：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-servers.ps1
```

终端 2：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-console.ps1
```

打开：

- `http://127.0.0.1:5173/#/portal/register`
- `http://127.0.0.1:5173/#/portal/login`
- `http://127.0.0.1:5173/#/portal/dashboard`
- `http://127.0.0.1:5173/#/admin`

### Linux 或 macOS

终端 1：

```bash
node scripts/dev/start-stack.mjs
```

终端 2：

```bash
node scripts/dev/start-console.mjs
```

浏览器访问地址与上面一致。

## Public Portal 自助流程

当后端服务和 console 都已启动后：

1. 打开 `http://127.0.0.1:5173/#/portal/register`
2. 注册一个 portal 用户
3. 注册成功后进入 `#/portal/dashboard`
4. 创建 `live`、`test` 或 `staging` 环境的 gateway API key
5. 立即复制返回的 plaintext key
6. 使用该 key 调用网关

示例：

```bash
curl http://127.0.0.1:8080/v1/models \
  -H "Authorization: Bearer skw_live_your_key_here"
```

注意：portal 列表接口不会再次返回 plaintext key。明文只在创建时返回一次。

## 浏览器与 Tauri 同时使用

如果你希望桌面窗口与浏览器界面同时可用，请使用 Tauri 启动入口。

### Windows

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-console.ps1 -Tauri
```

### Linux 或 macOS

```bash
node scripts/dev/start-console.mjs --tauri
```

原因：

- `tauri dev` 使用 Vite dev server 作为前端来源
- 同一个 Vite 地址依然可以被浏览器直接访问
- portal 注册、登录、dashboard 以及 admin 路由在浏览器和 Tauri 中完全一致

你可以同时打开：

- Tauri 桌面端
- 浏览器中的 `http://127.0.0.1:5173/#/portal/dashboard`

## 使用 PostgreSQL 启动

如果使用 PostgreSQL，只需要让三个服务都指向同一个 PostgreSQL 连接串。

### Windows

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-servers.ps1 `
  -DatabaseUrl "postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_server"
```

### Linux 或 macOS

```bash
node scripts/dev/start-stack.mjs \
  --database-url "postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_server"
```

SQLite 与 PostgreSQL 的迁移都会在启动时自动执行。

## 原生命令回退方式

如果你不想使用辅助脚本，也可以直接执行原生命令。

### Windows PowerShell

Admin：

```powershell
$env:SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
cargo run -p admin-api-service
```

Gateway：

```powershell
$env:SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
cargo run -p gateway-service
```

Portal：

```powershell
$env:SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
cargo run -p portal-api-service
```

浏览器 console：

```powershell
pnpm --dir console install
pnpm --dir console dev
```

Tauri：

```powershell
pnpm --dir console tauri:dev
```

### Linux 或 macOS

Admin：

```bash
export SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
cargo run -p admin-api-service
```

Gateway：

```bash
export SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
cargo run -p gateway-service
```

Portal：

```bash
export SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
cargo run -p portal-api-service
```

浏览器 console：

```bash
pnpm --dir console install
pnpm --dir console dev
```

Tauri：

```bash
pnpm --dir console tauri:dev
```

## 浏览器控制台

安装依赖：

```bash
pnpm --dir console install
```

启动浏览器开发服务器：

```bash
pnpm --dir console dev
```

检查全部 package 类型：

```bash
pnpm --dir console -r typecheck
```

构建生产资源：

```bash
pnpm --dir console build
```

本地预览构建结果：

```bash
pnpm --dir console preview
```

开发服务器默认代理：

- `/admin` -> `http://127.0.0.1:8081`
- `/portal` -> `http://127.0.0.1:8082`
- `/v1` -> `http://127.0.0.1:8080`

如需覆盖代理目标，可使用：

- `SDKWORK_ADMIN_PROXY_TARGET`
- `SDKWORK_PORTAL_PROXY_TARGET`
- `SDKWORK_GATEWAY_PROXY_TARGET`

示例：

```powershell
$env:SDKWORK_ADMIN_PROXY_TARGET="http://127.0.0.1:18081"
$env:SDKWORK_PORTAL_PROXY_TARGET="http://127.0.0.1:18082"
$env:SDKWORK_GATEWAY_PROXY_TARGET="http://127.0.0.1:18080"
pnpm --dir console dev
```

## 服务健康检查与 Metrics

健康检查地址：

- gateway：`http://127.0.0.1:8080/health`
- admin：`http://127.0.0.1:8081/admin/health`
- portal：`http://127.0.0.1:8082/portal/health`

Metrics 地址：

- gateway：`http://127.0.0.1:8080/metrics`
- admin：`http://127.0.0.1:8081/metrics`
- portal：`http://127.0.0.1:8082/metrics`

示例：

```bash
curl http://127.0.0.1:8082/portal/health
curl http://127.0.0.1:8082/metrics
```

## 运行时配置

重要环境变量：

- `SDKWORK_GATEWAY_BIND`
- `SDKWORK_ADMIN_BIND`
- `SDKWORK_PORTAL_BIND`
- `SDKWORK_DATABASE_URL`
- `SDKWORK_ADMIN_JWT_SIGNING_SECRET`
- `SDKWORK_PORTAL_JWT_SIGNING_SECRET`
- `SDKWORK_SECRET_BACKEND`
- `SDKWORK_CREDENTIAL_MASTER_KEY`
- `SDKWORK_SECRET_LOCAL_FILE`
- `SDKWORK_SECRET_KEYRING_SERVICE`
- `SDKWORK_RUNTIME_SNAPSHOT_INTERVAL_SECS`
- `SDKWORK_EXTENSION_PATHS`
- `SDKWORK_EXTENSION_ENABLE_CONNECTOR_EXTENSIONS`
- `SDKWORK_EXTENSION_ENABLE_NATIVE_DYNAMIC_EXTENSIONS`
- `SDKWORK_EXTENSION_TRUSTED_SIGNERS`
- `SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_CONNECTOR_EXTENSIONS`
- `SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_NATIVE_DYNAMIC_EXTENSIONS`

支持的 secret backend：

- `database_encrypted`
- `local_encrypted_file`
- `os_keyring`

## 当前仍然有意不做的部分

当前系统已经可以端到端使用，但下面这些仍然属于后续路线图：

- portal 多成员 workspace 与邀请
- 密码重置与邮件投递
- OAuth / SSO
- MySQL 或 libsql 的独立服务启动支持

## 参考文档

网关兼容性与 API 面覆盖情况：

- `docs/api/compatibility-matrix.md`

运行模式与部署说明：

- `docs/architecture/runtime-modes.md`

## 验证命令

当前项目级验证基线：

```bash
cargo fmt --all --check
cargo test --workspace -q -j 1
pnpm --dir console -r typecheck
pnpm --dir console build
```

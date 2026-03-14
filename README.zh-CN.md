# sdkwork-api-server

[English](./README.md)

SDKWork API Server 是一个基于 Rust、Axum、React、pnpm 与 Tauri 的 OpenAI 兼容网关、控制平面和公共自助门户系统。

当前仓库围绕 3 个 HTTP 边界和 1 个共享控制台构建：

- `gateway-service`：OpenAI 兼容 `/v1/*` 网关
- `admin-api-service`：面向运营方的 `/admin/*` 控制平面
- `portal-api-service`：面向外部用户的 `/portal/*` 注册、登录、workspace 与 API key 自助能力
- `console/`：既可浏览器访问，也可在 Tauri 桌面壳中运行的 React 控制台

## 当前已落地范围

仓库中已经具备：

- OpenAI 兼容网关路由，支持 stateful 与 stateless 两种执行路径
- Admin API：
  - tenants
  - projects
  - API keys
  - channels
  - proxy providers
  - credentials
  - models
  - routing
  - usage
  - billing
  - extensions
- Public Portal API：
  - `POST /portal/auth/register`
  - `POST /portal/auth/login`
  - `GET /portal/auth/me`
  - `GET /portal/workspace`
  - `GET /portal/api-keys`
  - `POST /portal/api-keys`
- SQLite 与 PostgreSQL 共用同一套存储抽象
- 按 package 拆分的 React 控制台模块：
  - portal SDK
  - portal auth
  - portal user dashboard
  - admin 相关视图
- 浏览器与 Tauri 共享的 hash 路由：
  - `#/portal/register`
  - `#/portal/login`
  - `#/portal/dashboard`
  - `#/admin`
- 扩展式 provider 架构，支持 built-in、connector、native dynamic 三类运行时

## 仓库结构

```text
.
|-- crates/                      # 领域、应用、接口、provider、runtime、storage 等 Rust crate
|-- services/
|   |-- admin-api-service/       # 独立 admin HTTP 服务
|   |-- gateway-service/         # 独立 OpenAI 兼容网关服务
|   `-- portal-api-service/      # 独立 public portal HTTP 服务
|-- console/                     # React + pnpm workspace + 可选 Tauri 桌面壳
|-- docs/                        # 架构说明、计划文档、兼容性说明
`-- README.md                    # 英文主文档
```

## 架构摘要

后端采用分层结构：

- `interface/controller`
  - `crates/sdkwork-api-interface-*` 中的 Axum router
- `app/service`
  - `crates/sdkwork-api-app-*` 中的认证、编排、密钥签发、路由决策和业务流程
- `repository/storage`
  - `crates/sdkwork-api-storage-core` 定义统一存储契约
  - SQLite 和 PostgreSQL 分别在独立 crate 中实现

前端遵循 SDKWork package 架构标准：

- `console/src/` 保持轻量，只负责 shell 与 route composition
- 复用模块全部放在 `console/packages/`
- Tauri 特定逻辑只放在 `console/src-tauri/`

## 支持的平台

当前 README 按以下平台提供可执行启动方式：

- Windows PowerShell
- Linux bash
- macOS zsh 或 bash

Rust 服务本身是跨平台的。Web 控制台可在任何支持 Node.js 与 pnpm 的环境中运行。Tauri 桌面壳是可选能力，并且与浏览器共享同一套前端。

## 环境要求

建议最低工具链：

- Rust stable + Cargo
- Node.js 20+
- pnpm 10+

可选：

- PostgreSQL 15+，用于 PostgreSQL 部署
- Tauri CLI，用于桌面开发：
  - `cargo install tauri-cli`

## 默认端口

| 面向对象 | 默认绑定地址 | 作用 |
|---|---|---|
| gateway | `127.0.0.1:8080` | OpenAI 兼容 `/v1/*` |
| admin | `127.0.0.1:8081` | 运营控制平面 |
| portal | `127.0.0.1:8082` | 外部用户注册、登录、自助 key 生命周期 |
| console | `127.0.0.1:5173` | 浏览器与 Tauri 共用前端开发服务器 |

## 使用 SQLite 快速启动

这是本地全链路联调最快的方式。

### Windows PowerShell

打开 4 个终端。

终端 1：

```powershell
$env:SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
cargo run -p admin-api-service
```

终端 2：

```powershell
$env:SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
cargo run -p gateway-service
```

终端 3：

```powershell
$env:SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
cargo run -p portal-api-service
```

终端 4：

```powershell
pnpm --dir console install
pnpm --dir console dev
```

打开：

- `http://127.0.0.1:5173/#/portal/register`
- `http://127.0.0.1:5173/#/portal/login`
- `http://127.0.0.1:5173/#/portal/dashboard`
- `http://127.0.0.1:5173/#/admin`

### Linux 或 macOS

同样打开 4 个终端。

终端 1：

```bash
export SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
cargo run -p admin-api-service
```

终端 2：

```bash
export SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
cargo run -p gateway-service
```

终端 3：

```bash
export SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
cargo run -p portal-api-service
```

终端 4：

```bash
pnpm --dir console install
pnpm --dir console dev
```

浏览器访问地址与上面一致。

## Public Portal 自助流程

当 4 个进程都启动后：

1. 打开 `http://127.0.0.1:5173/#/portal/register`
2. 注册一个 portal 用户
3. 注册成功后进入 `#/portal/dashboard`
4. 在 dashboard 中创建 `live`、`test` 或 `staging` 环境的 gateway API key
5. 立即复制返回的 plaintext key
6. 使用这个 key 调用网关

示例：

```bash
curl http://127.0.0.1:8080/v1/models \
  -H "Authorization: Bearer skw_live_your_key_here"
```

注意：portal 列表接口不会再次返回 plaintext key。明文只会在创建时返回一次。

## 使用 PostgreSQL 启动

使用同样的 3 个服务启动方式，只是把它们都指向同一个 PostgreSQL 数据库。

### Windows PowerShell

```powershell
$env:SDKWORK_DATABASE_URL="postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_server"
cargo run -p admin-api-service
```

在其他终端继续：

```powershell
$env:SDKWORK_DATABASE_URL="postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_server"
cargo run -p gateway-service
```

```powershell
$env:SDKWORK_DATABASE_URL="postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_server"
cargo run -p portal-api-service
```

### Linux 或 macOS

```bash
export SDKWORK_DATABASE_URL="postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_server"
cargo run -p admin-api-service
```

其他终端：

```bash
export SDKWORK_DATABASE_URL="postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_server"
cargo run -p gateway-service
```

```bash
export SDKWORK_DATABASE_URL="postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_server"
cargo run -p portal-api-service
```

SQLite 和 PostgreSQL 的迁移都会在服务启动时自动执行。

## 浏览器控制台启动

安装依赖：

```bash
pnpm --dir console install
```

启动浏览器开发服务器：

```bash
pnpm --dir console dev
```

检查全部 package 的类型：

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

## Tauri 桌面启动

桌面壳位于 `console/src-tauri/`。

### 推荐本地开发流程

终端 1：

```bash
pnpm --dir console install
pnpm --dir console dev
```

终端 2：

```bash
cd console
pnpm tauri:dev
```

这个流程会同时带来：

- Tauri 桌面窗口
- 可被浏览器访问的 Vite 开发服务器
- Portal 与 Admin 共用的 hash 路由

也就是说，桌面开发不会阻断浏览器访问。你可以一边打开桌面端，一边继续在浏览器中访问：

- `http://127.0.0.1:5173/#/portal/dashboard`

## 健康检查与 Metrics

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

重要环境变量包括：

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

## API 面说明

网关兼容性说明见：

- `docs/api/compatibility-matrix.md`

运行模式与部署说明见：

- `docs/architecture/runtime-modes.md`

## 验证命令

当前项目级验证基线：

```bash
cargo fmt --all --check
cargo test --workspace -q -j 1
pnpm --dir console -r typecheck
pnpm --dir console build
```

## 当前边界

当前有意保持的范围：

- 一个 portal 用户默认拥有一个 tenant 和一个 project
- 暂不包含 portal 邀请、多成员 workspace 管理
- 暂不包含密码重置和 OAuth/SSO
- 目前独立服务尚未支持 MySQL 或 libsql

这些都属于后续可叠加能力，不影响当前 gateway + portal + 浏览器/Tauri 联动工作流。

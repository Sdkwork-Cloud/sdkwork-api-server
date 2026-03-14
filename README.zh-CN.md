# sdkwork-api-server

[English](./README.md)

SDKWork API Gateway 是一个基于 Rust、Axum、React、pnpm 和 Tauri 构建的 OpenAI 兼容网关与控制平面工作区。

它面向两种主要运行形态：

- `server` 模式：独立启动 gateway 与 admin 两个 HTTP 服务
- `embedded` 模式：以内嵌运行时的方式被 Tauri 桌面壳或其他 Rust 应用直接调用

当前仓库已经包含：

- OpenAI 兼容的 `/v1/*` 网关接口
- 面向 tenants、projects、API keys、channels、proxy providers、routing、usage、billing、extensions 的管理控制面
- 通过 built-in、connector、native dynamic 三类扩展运行时实现的可插拔 provider 执行体系
- 可配置的本地密钥持久化能力
- 遵循 SDKWork 分层原则的 React 控制台

## 仓库结构

```text
.
├── crates/                     # 领域、应用、接口、存储、运行时、provider 等 Rust crates
├── services/
│   ├── admin-api-service/      # 独立管理 API 服务
│   └── gateway-service/        # 独立 OpenAI 兼容网关服务
├── console/                    # React + pnpm workspace + 可选 Tauri 桌面壳
├── docs/                       # 架构文档与实施计划
└── README.md                   # 英文主文档
```

## 环境要求

建议最低工具链：

- Rust stable + Cargo
- Node.js 20+
- pnpm 10+

按运行方式可选：

- PostgreSQL 15+，用于 PostgreSQL 独立部署
- Tauri CLI，用于桌面模式开发，例如 `cargo install tauri-cli`

本文示例默认使用 PowerShell 环境变量写法，例如 `$env:NAME="value"`。如果你使用 bash 或 zsh，请改成 `export NAME=value`。

## 当前可运行内容

后端：

- `admin-api-service`，默认监听 `127.0.0.1:8081`
- `gateway-service`，默认监听 `127.0.0.1:8080`
- 通过 `sdkwork-api-interface-http::gateway_router_with_stateless_config(...)` 构造的无状态内嵌 `/v1/*` 路由
- gateway 与 admin 两个服务统一暴露 Prometheus 兼容的 `/metrics`
- gateway 与 admin 两个服务统一支持 `x-request-id` 透传与结构化 HTTP tracing

前端：

- `console` Web 开发服务器，支持本地 `/admin` 与 `/v1` 代理
- `console` 的 Vite 生产构建
- `console/src-tauri` 桌面壳，可启动 embedded runtime

持久化：

- SQLite
- PostgreSQL

当前独立服务会对未支持的数据库 URL scheme 直接快速失败，不会静默降级。

## 使用 SQLite 快速启动

这是本地启动全栈的最快路径。

### 1. 启动管理服务

```bash
$env:SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
cargo run -p admin-api-service
```

默认地址：

- `http://127.0.0.1:8081`

### 2. 启动网关服务

打开第二个终端：

```bash
$env:SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
cargo run -p gateway-service
```

默认地址：

- `http://127.0.0.1:8080`

### 3. 启动 Web 控制台

打开第三个终端：

```bash
pnpm --dir console install
pnpm --dir console dev
```

默认地址：

- `http://127.0.0.1:5173`

默认代理：

- `/admin` -> `http://127.0.0.1:8081`
- `/v1` -> `http://127.0.0.1:8080`

### 4. 验证服务

验证 gateway：

```bash
curl http://127.0.0.1:8080/health
```

验证 admin：

```bash
curl http://127.0.0.1:8081/admin/health
```

验证 gateway metrics：

```bash
curl http://127.0.0.1:8080/metrics
```

验证 admin metrics：

```bash
curl http://127.0.0.1:8081/metrics
```

验证请求关联头：

```bash
curl -i -H "x-request-id: demo-request-1" http://127.0.0.1:8080/health
```

两个服务都会保留调用方传入的 `x-request-id`；如果未提供，则自动生成，并同时写回响应头与结构化日志。

## 使用 PostgreSQL 快速启动

为两个独立服务设置相同的 PostgreSQL 连接串即可。

示例：

```bash
$env:SDKWORK_DATABASE_URL="postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_server"
cargo run -p admin-api-service
```

在另一个终端中：

```bash
$env:SDKWORK_DATABASE_URL="postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_server"
cargo run -p gateway-service
```

当前支持的迁移会在启动时自动执行。

## 独立服务启动说明

两个独立二进制都会从环境变量中读取 `StandaloneConfig`。

重要默认值：

- `SDKWORK_GATEWAY_BIND=127.0.0.1:8080`
- `SDKWORK_ADMIN_BIND=127.0.0.1:8081`
- `SDKWORK_DATABASE_URL=sqlite://sdkwork-api-server.db`
- `SDKWORK_SECRET_BACKEND=database_encrypted`
- `SDKWORK_CREDENTIAL_MASTER_KEY=local-dev-master-key`
- `SDKWORK_ADMIN_JWT_SIGNING_SECRET=local-dev-admin-jwt-secret`
- `SDKWORK_RUNTIME_SNAPSHOT_INTERVAL_SECS=0`

推荐启动顺序：

1. 选择数据库 URL
2. 启动 `admin-api-service`
3. 启动 `gateway-service`
4. 通过 admin API 配置 channels、providers、credentials、models、routing
5. 签发 gateway API key
6. 使用该 key 调用 `/v1/*`

两个独立服务在启动时都会初始化统一 tracing，因此每个 HTTP 请求都会带有如下结构化字段：

- `service`
- `request_id`
- `method`
- `route`
- `status`
- `duration_ms`

## Console Web 启动说明

控制台位于 `console/`，是一个 pnpm workspace。

安装依赖：

```bash
pnpm --dir console install
```

启动开发服务器：

```bash
pnpm --dir console dev
```

类型检查：

```bash
pnpm --dir console -r typecheck
```

构建 Web 控制台：

```bash
pnpm --dir console build
```

如果你的后端地址不是默认值，可以覆盖代理目标：

- `SDKWORK_ADMIN_PROXY_TARGET`
- `SDKWORK_GATEWAY_PROXY_TARGET`

示例：

```bash
$env:SDKWORK_ADMIN_PROXY_TARGET="http://127.0.0.1:18081"
$env:SDKWORK_GATEWAY_PROXY_TARGET="http://127.0.0.1:18080"
pnpm --dir console dev
```

## Tauri 内嵌模式启动说明

Tauri 桌面壳位于 `console/src-tauri/`，当前已经具备最小 embedded runtime 引导能力。

推荐流程：

```bash
pnpm --dir console install
pnpm --dir console dev
cd console
cargo tauri dev
```

说明：

- `cargo tauri dev` 需要预先安装 Tauri CLI
- 当前 Tauri 配置依赖 `http://localhost:5173` 上的 Vite 开发服务器
- 桌面壳当前会启动一个临时 embedded runtime，并通过 Tauri command 暴露其 base URL

## 运行时配置

`StandaloneConfig` 当前支持以下环境变量：

- `SDKWORK_GATEWAY_BIND`
- `SDKWORK_ADMIN_BIND`
- `SDKWORK_DATABASE_URL`
- `SDKWORK_EXTENSION_PATHS`
- `SDKWORK_EXTENSION_ENABLE_CONNECTOR_EXTENSIONS`
- `SDKWORK_EXTENSION_ENABLE_NATIVE_DYNAMIC_EXTENSIONS`
- `SDKWORK_EXTENSION_TRUSTED_SIGNERS`
- `SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_CONNECTOR_EXTENSIONS`
- `SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_NATIVE_DYNAMIC_EXTENSIONS`
- `SDKWORK_SECRET_BACKEND`
- `SDKWORK_CREDENTIAL_MASTER_KEY`
- `SDKWORK_ADMIN_JWT_SIGNING_SECRET`
- `SDKWORK_RUNTIME_SNAPSHOT_INTERVAL_SECS`
- `SDKWORK_SECRET_LOCAL_FILE`
- `SDKWORK_SECRET_KEYRING_SERVICE`

支持的 secret backend 标识：

- `database_encrypted`
- `local_encrypted_file`
- `os_keyring`

trusted signer 配置示例：

```text
SDKWORK_EXTENSION_TRUSTED_SIGNERS=sdkwork=<base64-public-key>;partner=<base64-public-key>
```

## 无状态内嵌引导

对于不希望先引入数据库控制面的内嵌或库模式调用方，当前可以直接使用显式的无状态运行时配置。

默认值：

- synthetic tenant ID: `sdkwork-stateless`
- synthetic project ID: `sdkwork-stateless-default`
- 未配置 upstream 时：继续返回 OpenAI 兼容的本地 fallback
- 配置 upstream 后：无状态 router 会把当前已支持的 OpenAI-compatible 数据面转发到单个 upstream runtime

当前无状态 upstream relay 覆盖：

- `/v1/models`
- `/v1/chat/completions`，包含 SSE streaming，以及 list / retrieve / update / delete / messages list
- `/v1/completions`
- `/v1/responses`，包含 SSE streaming，以及 retrieve / delete / cancel / input items / input token counting / compact
- `/v1/embeddings`
- `/v1/files`，包含 list / retrieve / delete / binary content relay
- `/v1/uploads`，包含 part upload / complete / cancel relay
- `/v1/audio/*`，包含 speech 二进制 relay，以及 transcription / translation relay
- `/v1/images/*`，包含 generations / edits / variations relay
- `/v1/moderations` 与 `/v1/realtime/sessions`
- `/v1/assistants`、`/v1/threads`、`/v1/conversations`，以及它们的嵌套资源流
- `/v1/vector_stores`，包含 search / files / file batch flows
- `/v1/batches` 与 `/v1/fine_tuning/jobs`
- `/v1/webhooks`、`/v1/evals`、`/v1/videos`，以及 video content / remix relay

`runtime_key` 支持：

- built-in aliases：`openai`、`openrouter`、`ollama`
- compatibility aliases：`openai-compatible`、`custom-openai`、`openrouter-compatible`、`ollama-compatible`
- 显式扩展 ID，例如 `sdkwork.provider.openai.official`

行为说明：

- 未配置 upstream：走本地兼容 fallback
- 配置了未知 runtime key：回退到本地兼容 fallback
- runtime key 可解析但上游执行失败：返回 `502 Bad Gateway`

最小示例：

```rust
use sdkwork_api_interface_http::{
    gateway_router_with_stateless_config, StatelessGatewayConfig, StatelessGatewayUpstream,
};

let router = gateway_router_with_stateless_config(
    StatelessGatewayConfig::default().with_upstream(
        StatelessGatewayUpstream::from_adapter_kind(
            "custom-openai",
            "https://api.openai.com",
            "sk-upstream-openai",
        ),
    ),
);
```

## 最小上游 relay 配置

当前有状态 relay 路径至少需要：

1. 一个 channel
2. 一个带有 `extension_id`、`adapter_kind`、`base_url` 的 provider
3. 一条上游 credential
4. 一条指向该 provider 的 model catalog entry
5. 一条绑定到目标 tenant / project 的 gateway API key

provider 示例：

```json
{
  "id": "provider-openai-official",
  "channel_id": "openai",
  "extension_id": "sdkwork.provider.openai.official",
  "channel_bindings": [
    { "channel_id": "openai", "is_primary": true }
  ],
  "adapter_kind": "openai",
  "base_url": "https://api.openai.com",
  "display_name": "OpenAI Official"
}
```

credential 示例：

```json
{
  "tenant_id": "tenant-1",
  "provider_id": "provider-openai-official",
  "key_reference": "cred-openai",
  "secret_value": "sk-upstream-openai"
}
```

密钥内容会根据当前 active backend 落到不同存储介质中，而 credential 绑定关系仍然由数据库驱动。

## 验证命令

后端验证：

```bash
cargo fmt --all --check
cargo test --workspace -q -j 1
$env:CARGO_BUILD_JOBS='1'; cargo clippy --workspace --all-targets -- -D warnings
```

前端验证：

```bash
pnpm --dir console -r typecheck
pnpm --dir console build
```

## 当前能力快照

已实现的后端接口包括：

- OpenAI 兼容 `/v1/models`
- `/v1/chat/completions`
- `/v1/completions`
- `/v1/responses`
- `/v1/embeddings`
- chat / responses 的 SSE streaming
- 面向内嵌与库模式的显式 stateless runtime 配置
- 在配置 upstream runtime 时，stateless 模式可对 files、uploads、audio、images、moderations、realtime sessions、assistants、threads、conversations、vector stores、batches、fine-tuning jobs、webhooks、evals、videos 执行 relay

控制面能力包括：

- tenants、projects、gateway API keys
- channels、proxy providers、model catalog entries
- 加密 upstream credentials
- extension installations / instances
- runtime status 可观测性与 provider health snapshots
- 支持 `deterministic_priority`、`weighted_random`、`slo_aware` 的 routing policies
- 管理模拟与真实网关分发的 routing decision logs
- usage records 与 usage summaries
- billing ledger entries、billing summaries、quota policies
- gateway 与 admin 服务统一的 Prometheus 兼容 HTTP metrics
- gateway 与 admin 服务统一的 `x-request-id` 响应透传与结构化 HTTP tracing

最新兼容矩阵请查看：

- [`docs/api/compatibility-matrix.md`](./docs/api/compatibility-matrix.md)

## 当前限制

- 独立服务当前仅支持 SQLite 与 PostgreSQL
- 当前最成熟的 relay 路径仍然是 OpenAI 兼容协议：
  - `openai`
  - `openrouter`
  - `ollama`
- extension hot reload 仍是后续工作
- geo affinity / region-aware routing 仍是后续工作
- stateless 模式当前仍然假设单个 upstream runtime，尚未提供 stateful routing、quota、billing 等语义

## 进一步阅读

- [`README.md`](./README.md)
- [`docs/architecture/runtime-modes.md`](./docs/architecture/runtime-modes.md)
- [`docs/api/compatibility-matrix.md`](./docs/api/compatibility-matrix.md)
- [`docs/plans/`](./docs/plans/)

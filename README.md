# sdkwork-api-server

[中文说明](./README.zh-CN.md)

SDKWork API Gateway is an OpenAI-compatible gateway and control-plane workspace built with Rust, Axum, React, pnpm, and Tauri.

It is designed for two runtime shapes:

- `server` mode: standalone HTTP services for the gateway and admin API
- `embedded` mode: an in-process runtime consumed by the Tauri desktop shell

The repository combines:

- OpenAI-compatible `/v1/*` gateway interfaces
- an admin control plane for tenants, projects, API keys, channels, providers, routing, usage, billing, and extensions
- pluggable provider execution through built-in, connector, and native dynamic extension runtimes
- configurable local secret persistence
- a React console aligned with the SDKWork package architecture

## Repository Layout

```text
.
├── crates/                     # Domain, app, interface, storage, runtime, provider crates
├── services/
│   ├── admin-api-service/      # Standalone admin HTTP service
│   └── gateway-service/        # Standalone OpenAI-compatible gateway HTTP service
├── console/                    # React + pnpm workspace + optional Tauri shell
├── docs/                       # Architecture notes and implementation plans
└── README.zh-CN.md             # Chinese mirror of this operational guide
```

## Prerequisites

Minimum recommended toolchain:

- Rust stable with Cargo
- Node.js 20+
- pnpm 10+

Optional, depending on how you run the project:

- PostgreSQL 15+ for standalone PostgreSQL-backed deployments
- Tauri CLI for desktop development, for example `cargo install tauri-cli`

Shell examples below use PowerShell syntax such as `$env:NAME="value"`. On bash or zsh, replace those assignments with `export NAME=value`.

## What You Can Run Today

Backend:

- `admin-api-service` on `127.0.0.1:8081` by default
- `gateway-service` on `127.0.0.1:8080` by default
- stateless in-process `/v1/*` router construction through `sdkwork-api-interface-http::gateway_router_with_stateless_config(...)`
- Prometheus-compatible `/metrics` endpoints on both services
- shared `x-request-id` propagation and structured HTTP request tracing on both services

Frontend:

- `console` web development server with local proxying to `/admin` and `/v1`
- `console` production build through Vite
- `console/src-tauri` desktop shell that can start an embedded runtime host

Persistence:

- SQLite
- PostgreSQL

Current standalone binaries intentionally fail fast for unsupported URL schemes such as MySQL or libsql.

## Quick Start With SQLite

This is the fastest way to bring up the whole stack locally.

### 1. Start the admin API

```bash
$env:SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
cargo run -p admin-api-service
```

The admin API listens on `http://127.0.0.1:8081` by default.

### 2. Start the gateway API

Open a second terminal:

```bash
$env:SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
cargo run -p gateway-service
```

The gateway listens on `http://127.0.0.1:8080` by default.

### 3. Start the web console

Open a third terminal:

```bash
pnpm --dir console install
pnpm --dir console dev
```

The console dev server runs on `http://127.0.0.1:5173` by default and proxies:

- `/admin` -> `http://127.0.0.1:8081`
- `/v1` -> `http://127.0.0.1:8080`

### 4. Verify the services

Gateway health:

```bash
curl http://127.0.0.1:8080/health
```

Admin health:

```bash
curl http://127.0.0.1:8081/admin/health
```

Gateway metrics:

```bash
curl http://127.0.0.1:8080/metrics
```

Admin metrics:

```bash
curl http://127.0.0.1:8081/metrics
```

Request correlation header example:

```bash
curl -i -H "x-request-id: demo-request-1" http://127.0.0.1:8080/health
```

Both services preserve a caller-supplied `x-request-id` or generate one automatically when the header is missing. The same ID is returned on the response and emitted in the standalone HTTP request logs.

## Quick Start With PostgreSQL

Set `SDKWORK_DATABASE_URL` to a PostgreSQL connection string for both standalone services.

Example:

```bash
$env:SDKWORK_DATABASE_URL="postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_server"
cargo run -p admin-api-service
```

In another terminal:

```bash
$env:SDKWORK_DATABASE_URL="postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_server"
cargo run -p gateway-service
```

The same migrations are applied automatically for the supported dialect.

## Standalone Service Startup

The standalone binaries both read `StandaloneConfig` from environment variables.

Important defaults:

- `SDKWORK_GATEWAY_BIND=127.0.0.1:8080`
- `SDKWORK_ADMIN_BIND=127.0.0.1:8081`
- `SDKWORK_DATABASE_URL=sqlite://sdkwork-api-server.db`
- `SDKWORK_SECRET_BACKEND=database_encrypted`
- `SDKWORK_CREDENTIAL_MASTER_KEY=local-dev-master-key`
- `SDKWORK_ADMIN_JWT_SIGNING_SECRET=local-dev-admin-jwt-secret`
- `SDKWORK_RUNTIME_SNAPSHOT_INTERVAL_SECS=0`

Recommended standalone startup sequence:

1. choose a database URL
2. start `admin-api-service`
3. start `gateway-service`
4. configure channels, providers, credentials, models, and routing through the admin API
5. issue a gateway API key
6. call the `/v1/*` gateway routes with the issued key

Both standalone binaries initialize a shared tracing subscriber at startup, so every HTTP request is logged with `service`, `request_id`, `method`, `route`, `status`, and `duration_ms`.

## Console Web Startup

The console is a pnpm workspace under `console/`.

Install dependencies:

```bash
pnpm --dir console install
```

Run the development server:

```bash
pnpm --dir console dev
```

Typecheck the workspace:

```bash
pnpm --dir console -r typecheck
```

Build the web console:

```bash
pnpm --dir console build
```

The development server is configured for local backend development. Override the proxy targets if needed:

- `SDKWORK_ADMIN_PROXY_TARGET`
- `SDKWORK_GATEWAY_PROXY_TARGET`

Example:

```bash
$env:SDKWORK_ADMIN_PROXY_TARGET="http://127.0.0.1:18081"
$env:SDKWORK_GATEWAY_PROXY_TARGET="http://127.0.0.1:18080"
pnpm --dir console dev
```

## Tauri Embedded Startup

The Tauri shell lives under `console/src-tauri/` and currently exposes a minimal embedded runtime bootstrap.

Recommended flow:

```bash
pnpm --dir console install
pnpm --dir console dev
cd console
cargo tauri dev
```

Notes:

- `cargo tauri dev` requires the Tauri CLI to be installed
- the Tauri config uses the Vite dev server at `http://localhost:5173`
- the desktop shell currently starts an ephemeral embedded runtime and exposes its base URL through a Tauri command

## Runtime Configuration

`StandaloneConfig` currently supports these environment variables:

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

Supported secret backend identifiers:

- `database_encrypted`
- `local_encrypted_file`
- `os_keyring`

Example signer configuration:

```text
SDKWORK_EXTENSION_TRUSTED_SIGNERS=sdkwork=<base64-public-key>;partner=<base64-public-key>
```

## Stateless Library Bootstrap

For embedded or library-hosted use cases that do not want a database-backed control plane yet, the gateway crate now exposes an explicit stateless runtime contract.

Defaults:

- synthetic tenant ID: `sdkwork-stateless`
- synthetic project ID: `sdkwork-stateless-default`
- no configured upstream: OpenAI-compatible local fallback remains active
- configured upstream: the stateless router relays the supported OpenAI-compatible surface to one upstream runtime

Current stateless upstream-relay coverage:

- `/v1/models`
- `/v1/chat/completions`, including SSE streaming plus list, retrieve, update, delete, and messages list
- `/v1/completions`
- `/v1/responses`, including SSE streaming plus retrieve, delete, cancel, input items, input token counting, and compact
- `/v1/embeddings`
- `/v1/files`, including list, retrieve, delete, and binary content relay
- `/v1/uploads`, including part upload, completion, and cancel relay
- `/v1/audio/*`, including speech binary relay plus transcription and translation relay
- `/v1/images/*`, including generations, edits, and variations relay
- `/v1/moderations` and `/v1/realtime/sessions`
- `/v1/assistants`, `/v1/threads`, and `/v1/conversations`, including their nested resource flows
- `/v1/vector_stores`, including search, files, and file batch flows
- `/v1/batches` and `/v1/fine_tuning/jobs`
- `/v1/webhooks`, `/v1/evals`, and `/v1/videos`, including video content and remix relay

Runtime-key resolution accepts:

- built-in aliases such as `openai`, `openrouter`, and `ollama`
- compatibility aliases such as `openai-compatible`, `custom-openai`, `openrouter-compatible`, and `ollama-compatible`
- explicit extension IDs such as `sdkwork.provider.openai.official`

Behavior:

- if no stateless upstream is configured, the router returns compatible local responses
- if a runtime key cannot be resolved, the router falls back locally
- if an explicitly configured upstream resolves but fails during execution, the router returns `502 Bad Gateway`

Minimal example:

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

## Minimal Upstream Relay Setup

The current relay path expects:

1. a channel
2. a provider with `extension_id`, `adapter_kind`, and `base_url`
3. an upstream credential
4. a model catalog entry mapped to that provider
5. a gateway API key for the target tenant and project

Example provider payload:

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

Example credential payload:

```json
{
  "tenant_id": "tenant-1",
  "provider_id": "provider-openai-official",
  "key_reference": "cred-openai",
  "secret_value": "sk-upstream-openai"
}
```

The secret is stored according to the active backend, while the credential binding remains catalog-driven in the database.

## Verification

Backend verification:

```bash
cargo fmt --all --check
cargo test --workspace -q -j 1
$env:CARGO_BUILD_JOBS='1'; cargo clippy --workspace --all-targets -- -D warnings
```

Console verification:

```bash
pnpm --dir console -r typecheck
pnpm --dir console build
```

## Capability Snapshot

Implemented backend surfaces include:

- OpenAI-compatible `/v1/models`
- `/v1/chat/completions`
- `/v1/completions`
- `/v1/responses`
- `/v1/embeddings`
- SSE streaming for chat and responses
- explicit stateless runtime configuration for library and embedded bootstrap use cases
- stateless relay coverage for files, uploads, audio, images, moderations, realtime sessions, assistants, threads, conversations, vector stores, batches, fine-tuning jobs, webhooks, evals, and videos when an upstream runtime is configured

Control-plane features include:

- tenants, projects, and gateway API keys
- channels, proxy providers, and model catalog entries
- encrypted upstream credentials
- extension installations and instances
- runtime status visibility and provider health snapshots
- routing policies with `deterministic_priority`, `weighted_random`, and `slo_aware`
- routing decision logs for admin simulation and real gateway dispatch
- usage records and usage summaries
- billing ledger entries, billing summaries, and quota policies
- Prometheus-compatible HTTP metrics for gateway and admin services
- `x-request-id` response propagation and structured HTTP request tracing for gateway and admin services

For the up-to-date execution-truth matrix, see:

- [`docs/api/compatibility-matrix.md`](./docs/api/compatibility-matrix.md)

## Current Limitations

- standalone binaries currently support SQLite and PostgreSQL only
- the richest relay path today is for OpenAI-compatible provider protocols:
  - `openai`
  - `openrouter`
  - `ollama`
- hot reload for extensions is still future work
- geo affinity and regional routing dimensions are still future work
- stateless mode still assumes a single configured upstream runtime and does not yet provide stateful routing, quota, or billing semantics

## Additional Docs

- [`README.zh-CN.md`](./README.zh-CN.md)
- [`docs/architecture/runtime-modes.md`](./docs/architecture/runtime-modes.md)
- [`docs/api/compatibility-matrix.md`](./docs/api/compatibility-matrix.md)
- [`docs/plans/`](./docs/plans/)

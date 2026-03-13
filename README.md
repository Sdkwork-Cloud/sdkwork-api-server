# sdkwork-api-server

SDKWork API Gateway is a Rust and Tauri oriented OpenAI-compatible gateway workspace.

It is organized as a composable backend plus a pnpm-based console:

- `services/gateway-service`: OpenAI-style `/v1/*` data plane
- `services/admin-api-service`: control plane for tenants, projects, gateway keys, channels, providers, models, routing, usage, and billing
- `crates/*`: domain, application, storage, provider, secret, interface, and runtime layers
- `console/`: React workspace aligned to the SDKWork package architecture
- `console/src-tauri/`: desktop host shell for embedded runtime mode

## What Works Today

Backend:

- Axum-based gateway and admin HTTP services
- OpenAI-compatible contracts for:
  - models
  - chat completions
  - responses
  - embeddings
  - SSE streaming
  - files
  - uploads
  - audio
  - images
  - moderations
  - realtime sessions
  - assistants
  - vector stores
  - batches
  - webhooks
  - evals
- SQLite-backed control plane persistence for:
  - tenants
  - projects
  - gateway API keys
  - channels
  - proxy providers
  - model catalog entries
  - upstream credential references
  - usage records
  - billing ledger entries
- Catalog-backed `/v1/models`
- Real upstream relay for stateful `/v1/chat/completions`:
  - non-stream JSON relay for `adapter_kind = openai`
  - SSE relay for `stream = true` against OpenAI-compatible upstreams
- Real upstream relay for stateful `/v1/responses` and `/v1/embeddings` when provider, model, and credential records are present
- Stub fallback responses for unconfigured providers or unsupported adapter kinds
- Routing simulation API backed by catalog model candidates
- Encrypted upstream credential storage and runtime secret resolution through `credential_master_key`
- Usage and billing telemetry recording from stateful gateway handlers

Console:

- pnpm workspace with typed package boundaries
- typed `sdkwork-api-admin-sdk` client for control-plane APIs
- live dashboard panels for:
  - workspace tenants, projects, and gateway keys
  - channels, providers, and model catalog
  - routing simulation
  - usage and billing telemetry
  - runtime posture

## Current Limitations

The repository is now a working control-plane-first gateway skeleton, but it is not yet a full upstream relay product.

Known gaps:

- upstream relay currently supports OpenAI-compatible adapter kinds:
  - `openai`
  - `openrouter`
  - `ollama`
- provider execution is now registry-based, but only the OpenAI-compatible adapter is registered by default
- only stateful gateway execution paths relay upstream responses; the stateless demo router still emits local stub payloads
- the broader API families beyond models/chat/responses/embeddings/streaming are contract-defined but not yet wired to HTTP handlers or upstream adapters
- routing policies are still placeholder-only; current routing uses catalog candidates plus deterministic fallback
- only SQLite is fully implemented as an active persistence driver; PostgreSQL, MySQL, and libsql remain extension boundaries

## Minimal Upstream Relay Setup

The current relay path expects:

1. a channel
2. a provider with `adapter_kind` and `base_url`
3. an encrypted upstream credential
4. a model catalog entry pointing at that provider

Example provider payload:

```json
{
  "id": "provider-openai-official",
  "channel_id": "openai",
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

`secret_value` is encrypted before being stored in SQLite. The runtime resolves it with the configured `credential_master_key`.

## Runtime Configuration

`StandaloneConfig` now models:

- `database_url`
- inferred storage dialect via `storage_dialect()`
- `secret_backend`
- `credential_master_key`

Supported secret backend strategy identifiers are:

- `database_encrypted`
- `local_encrypted_file`
- `os_keyring`

## Development

Backend verification:

```bash
cargo test --workspace
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
```

Console verification:

```bash
pnpm --dir console install
pnpm --dir console -r typecheck
pnpm --dir console typecheck
pnpm --dir console exec vite build
```

## Architecture Notes

- `Channel` models the upstream ecosystem or vendor family, such as OpenAI, Anthropic, Google, or DeepSeek.
- `ProxyProvider` models a concrete access path under a channel, such as an official endpoint, OpenRouter-style broker, or self-hosted Ollama node.
- The backend is split into domain, application, interface, storage, provider, secret, and runtime crates to preserve controller/service/repository layering without forcing separate deployable processes for every boundary.
- Standalone and embedded runtime modes share the same Rust crates; Tauri integration consumes the same admin and gateway capabilities through the runtime host boundary.
- Stateful gateway execution now uses the catalog, routing, credential, and provider layers together to relay OpenAI-compatible upstream requests while still preserving local stub fallbacks for incomplete configuration.
- Provider dispatch is now routed through `sdkwork-api-provider-core` registry abstractions so new adapter kinds can be added without reworking every gateway relay path.
- `openrouter` and `ollama` are now registered as OpenAI-compatible provider adapters in addition to the direct `openai` adapter.

## Design Docs

- `docs/plans/2026-03-13-sdkwork-api-gateway-design.md`
- `docs/plans/2026-03-13-sdkwork-api-gateway-implementation.md`
- `docs/api/compatibility-matrix.md`
- `docs/architecture/runtime-modes.md`

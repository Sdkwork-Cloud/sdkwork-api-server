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
  - completions
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
- SQLite and PostgreSQL backed control plane persistence for:
  - tenants
  - projects
  - gateway API keys
  - channels
  - proxy providers
  - model catalog entries
  - extension installations
  - extension instances
  - upstream credential references
  - usage records
  - billing ledger entries
- Catalog-backed `/v1/models`
- Real upstream relay for stateful `/v1/chat/completions` and `/v1/completions`:
  - extension-driven JSON relay for providers bound to OpenAI-compatible built-in extensions
  - SSE relay for `stream = true` against OpenAI-compatible upstreams
- Real upstream relay for stateful `/v1/responses` and `/v1/embeddings` when provider, model, and credential records are present
- Stub fallback responses for unconfigured providers or unsupported adapter kinds
- Routing simulation API backed by catalog model candidates
- Built-in extension host with pluggable extension manifests for:
  - `sdkwork.provider.openai.official`
  - `sdkwork.provider.openrouter`
  - `sdkwork.provider.ollama`
- Persisted extension installation and instance config through admin APIs
- Standardized extension package metadata:
  - runtime ID: `sdkwork.provider.*` / `sdkwork.channel.*`
  - distribution name: `sdkwork-provider-*` / `sdkwork-channel-*`
  - Rust crate name: `sdkwork-api-ext-provider-*` / `sdkwork-api-ext-channel-*`
- Configuration-driven extension load planning that merges manifest defaults, installation config, and instance overrides into one runtime plan
- Runtime-selectable upstream credential persistence with three local strategies:
  - `database_encrypted`
  - `local_encrypted_file`
  - `os_keyring`
- Per-credential backend binding and runtime secret resolution through `credential_master_key`
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
- provider execution is now `extension_id`-driven with `adapter_kind` retained as compatibility metadata and protocol hint
- dynamic runtime loading for `native_dynamic` and `connector` extensions is designed but not yet implemented
- only stateful gateway execution paths relay upstream responses; the stateless demo router still emits local stub payloads
- broader API families are now wired as either `relay` or `emulated`; see `docs/api/compatibility-matrix.md` for the execution-truth matrix
- routing policies are still placeholder-only; current routing uses catalog candidates plus deterministic fallback
- SQLite and PostgreSQL are active persistence drivers; MySQL and libsql remain extension boundaries

## Minimal Upstream Relay Setup

The current relay path expects:

1. a channel
2. a provider with `extension_id`, `adapter_kind`, and `base_url`
3. an encrypted upstream credential
4. a model catalog entry pointing at that provider

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

`secret_value` is now persisted according to the active secret backend:

- `database_encrypted`: encrypted envelope stored in SQLite
- `local_encrypted_file`: encrypted envelope stored in a local JSON file
- `os_keyring`: encrypted envelope stored in the operating system keyring

The credential binding itself remains in SQLite so routing and provider resolution stay catalog-driven. Gateway resolution uses the credential record's stored backend kind, so existing credentials remain readable even if the runtime default backend changes later.

## Runtime Configuration

`StandaloneConfig` now models:

- `database_url`
- inferred storage dialect via `storage_dialect()`
- `secret_backend`
- `credential_master_key`
- `secret_local_file`
- `secret_keyring_service`

It can now be loaded from environment variables:

- `SDKWORK_GATEWAY_BIND`
- `SDKWORK_ADMIN_BIND`
- `SDKWORK_DATABASE_URL`
- `SDKWORK_SECRET_BACKEND`
- `SDKWORK_CREDENTIAL_MASTER_KEY`
- `SDKWORK_SECRET_LOCAL_FILE`
- `SDKWORK_SECRET_KEYRING_SERVICE`

Supported secret backend strategy identifiers are:

- `database_encrypted`
- `local_encrypted_file`
- `os_keyring`

Current standalone service binaries fail fast on unsupported storage dialects instead of silently assuming SQLite.

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
- `ProxyProvider` models a concrete access path under one or more channels, such as an official endpoint, OpenRouter-style broker, or self-hosted Ollama node.
- `ProxyProvider.extension_id` is now the runtime execution identity used to resolve a concrete extension package; `adapter_kind` remains useful as compatibility and protocol metadata.
- `ProviderChannelBinding` now allows one provider to bind to multiple channel ecosystems without losing a primary channel for compatibility.
- `ModelCatalogEntry` now carries capability and streaming metadata instead of only `external_name + provider_id`.
- The backend is split into domain, application, interface, storage, provider, secret, and runtime crates to preserve controller/service/repository layering without forcing separate deployable processes for every boundary.
- Standalone and embedded runtime modes share the same Rust crates; Tauri integration consumes the same admin and gateway capabilities through the runtime host boundary.
- Stateful gateway execution now uses the catalog, routing, credential, and provider layers together to relay OpenAI-compatible upstream requests while still preserving local stub fallbacks for incomplete configuration.
- Provider dispatch is now routed through `sdkwork-api-extension-host`, which resolves factories by `extension_id` first and keeps legacy `adapter_kind` aliases as a fallback.
- Extension runtime configuration is split into package metadata, installation state, and mounted instances so one extension can back multiple provider instances.
- Extension load planning now follows a deterministic merge order:
  - manifest metadata defines package identity and default entrypoint/schema references
  - installation state selects runtime and package-level config
  - instance state supplies environment-specific overrides such as `base_url`, `credential_ref`, and traffic weighting
- `openrouter` and `ollama` are registered as built-in OpenAI-compatible provider extensions in addition to the direct `openai` adapter.

## Design Docs

- `docs/plans/2026-03-13-sdkwork-api-gateway-design.md`
- `docs/plans/2026-03-13-sdkwork-api-gateway-implementation.md`
- `docs/api/compatibility-matrix.md`
- `docs/architecture/runtime-modes.md`

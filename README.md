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
- OpenAI-compatible contracts for models, chat completions, responses, embeddings, and SSE streaming
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
- Minimal `/v1/chat/completions`, `/v1/responses`, `/v1/embeddings`, and streaming responses
- Routing simulation API backed by catalog model candidates
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

- `/v1/*` execution is still stubbed at the response generation layer instead of proxying real upstream responses
- upstream secret storage backends exist as crate boundaries, but provider credential encryption and retrieval are not yet wired into runtime request execution
- routing policies are still placeholder-only; current routing uses catalog candidates plus deterministic fallback
- only SQLite is fully implemented as an active persistence driver; PostgreSQL, MySQL, and libsql remain extension boundaries

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

## Design Docs

- `docs/plans/2026-03-13-sdkwork-api-gateway-design.md`
- `docs/plans/2026-03-13-sdkwork-api-gateway-implementation.md`
- `docs/api/compatibility-matrix.md`
- `docs/architecture/runtime-modes.md`

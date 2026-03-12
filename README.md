# sdkwork-api-server

SDKWork API Gateway project.

This repository provides a Rust-based OpenAI-compatible gateway with:

- OpenAI-style `/v1/*` API surface
- multi-tenant control-plane foundations
- channel and proxy-provider abstractions
- storage driver boundaries for SQLite, PostgreSQL, MySQL, and libsql
- Web console workspace built with React and pnpm
- embedded runtime host for Tauri integration

## Current Status

The current implementation includes the first project skeleton and the first working vertical slice of foundations:

- Cargo workspace and service skeleton
- OpenAI contract crates
- canonical gateway contract crate
- domain and storage abstractions
- secret and identity primitives
- admin and gateway HTTP interface skeleton
- `/v1/models`
- `/v1/chat/completions`
- `/v1/responses`
- `/v1/embeddings`
- basic SSE streaming response path
- React console workspace packages
- runtime host crate and initial `src-tauri/` shell

## Development

Backend:

```bash
cargo test --workspace
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
```

Console:

```bash
pnpm --dir console install
pnpm --dir console -r typecheck
pnpm --dir console exec vite build
```

## Design Docs

See:

- `docs/plans/2026-03-13-sdkwork-api-gateway-design.md`
- `docs/plans/2026-03-13-sdkwork-api-gateway-implementation.md`

Additional docs:

- `docs/api/compatibility-matrix.md`
- `docs/architecture/runtime-modes.md`

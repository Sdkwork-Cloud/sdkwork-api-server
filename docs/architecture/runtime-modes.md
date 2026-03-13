# Runtime Modes

## Server Mode

Server mode is the standalone deployment shape.

Characteristics:

- services run as independent binaries
- gateway and admin APIs are exposed over HTTP
- PostgreSQL is the preferred deployment database and is now supported by the shared admin store runtime
- upstream credentials are expected to be managed by a server-side secret backend strategy
- the current repository can run `gateway-service` and `admin-api-service` against either SQLite or PostgreSQL through the same storage abstraction

## Embedded Mode

Embedded mode is the desktop-oriented deployment shape.

Characteristics:

- the runtime is hosted in-process through `sdkwork-api-runtime-host`
- the Tauri shell under `console/src-tauri/` can start or call into the embedded runtime
- loopback binding is the default trust boundary
- SQLite is the preferred local persistence strategy
- OS keyring is the preferred secret backend when available
- the React console packages can target the same admin API surface in both standalone and embedded modes

## Current Implementation State

The current repository includes:

- a minimal `EmbeddedRuntime` abstraction
- a loopback base URL contract
- a Tauri shell scaffold with an initial runtime command
- a live React console that consumes admin APIs for workspace, channel mesh, routing simulation, and telemetry views
- SQLite-backed control-plane persistence for identity, catalog, usage, and billing slices
- PostgreSQL-backed control-plane persistence for the same admin store contract
- encrypted upstream credential persistence and runtime secret resolution
- stateful OpenAI-compatible upstream relay for chat completions, responses, embeddings, and chat SSE when provider configuration is present
- runtime config modeling for storage dialect inference plus secret backend strategy selection
- three live secret persistence strategies:
  - `database_encrypted`
  - `local_encrypted_file`
  - `os_keyring`
- per-credential backend tracking so previously stored secrets can still be resolved after a default backend switch
- environment-driven standalone config loading for bind addresses, database URL, secret backend, credential master key, local secret file path, and keyring service name

The runtime host is still intentionally lightweight, but the core gateway, admin, routing, credential, and provider relay slices now run against the same Rust workspace and can be assembled in-process for embedded mode.

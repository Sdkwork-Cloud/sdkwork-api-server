# Runtime Modes

## Server Mode

Server mode is the standalone deployment shape.

Characteristics:

- services run as independent binaries
- gateway and admin APIs are exposed over HTTP
- PostgreSQL is the preferred deployment database
- upstream credentials are expected to be managed by a server-side secret backend strategy
- the current repository runs `gateway-service` and `admin-api-service` against the same SQLite database by default for local development

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

The runtime host is intentionally lightweight at this stage and will be expanded to assemble gateway, admin, routing, credential, and provider relay execution in-process.

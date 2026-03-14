# Runtime Modes

SDKWork API Server supports both standalone server operation and an embedded desktop-oriented runtime model.

## Server Mode

Server mode is the standalone deployment shape.

Characteristics:

- services run as independent binaries
- gateway, admin, and portal APIs are exposed over HTTP
- PostgreSQL is the preferred deployment database
- upstream credentials are expected to be managed by a server-side secret backend

Choose this mode when:

- you want a browser-accessible shared environment
- you need multiple operators or portal users
- you are deploying behind a reverse proxy or service manager

## Embedded Mode

Embedded mode is the desktop-oriented deployment shape.

Characteristics:

- the runtime can be hosted in-process through the runtime host abstraction
- the Tauri shell can host the same React console as the browser
- loopback binding is the default trust boundary
- SQLite is the preferred local persistence strategy

Choose this mode when:

- you want a desktop-first operator experience
- you are running locally on a single machine
- you want browser and desktop access to share the same frontend routes

## Browser and Tauri Together

In development:

- `pnpm --dir console tauri:dev` uses the Vite dev server
- the same Vite URL stays reachable from a browser
- `start-workspace --tauri` keeps backend services plus the desktop shell in one startup flow

## Deep Dive

For the detailed runtime topology and extension runtime status, read:

- [Detailed Runtime Modes](/architecture/runtime-modes)

# Runtime Modes

SDKWork API Server supports several practical runtime shapes. The important distinction is not only server versus desktop, but also whether you want raw source control or a managed script lifecycle.

## Raw Standalone Service Mode

This is the lowest-level shape.

Characteristics:

- services run as independent binaries
- gateway, admin, and portal APIs are exposed over HTTP
- the binaries keep their built-in defaults unless you override them
- best when you want direct process-level control

Typical entrypoints:

- `cargo run -p gateway-service`
- `cargo run -p admin-api-service`
- `cargo run -p portal-api-service`

Typical default binds:

- gateway: `127.0.0.1:8080`
- admin: `127.0.0.1:8081`
- portal: `127.0.0.1:8082`

## Source Browser Workspace Mode

This is the raw source developer workflow.

Characteristics:

- backend services use the updated helper defaults on `9980`, `9981`, and `9982`
- admin and portal stay on standalone Vite dev servers
- best for frontend iteration and hot reloading

Entry points:

- `node scripts/dev/start-workspace.mjs`
- `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1`

Primary browser URLs:

- admin: `http://127.0.0.1:5173/admin/`
- portal: `http://127.0.0.1:5174/portal/`

## Source Preview Workspace Mode

This is the raw source single-port workflow.

Characteristics:

- backend services stay on `9980`, `9981`, and `9982`
- Pingora serves admin and portal through one browser-visible host
- best when you want browser validation closer to release shape

Entry points:

- `node scripts/dev/start-workspace.mjs --preview`
- `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1 -Preview`

Primary browser URLs:

- unified admin: `http://127.0.0.1:9983/admin/`
- unified portal: `http://127.0.0.1:9983/portal/`

## Source Tauri Workspace Mode

This is the raw source desktop-oriented workflow.

Characteristics:

- backend services stay on `9980`, `9981`, and `9982`
- admin lives in the Tauri desktop shell
- Pingora still exposes browser access through the unified web host

Entry points:

- `node scripts/dev/start-workspace.mjs --tauri`
- `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1 -Tauri`

## Managed Development Mode

This is the recommended scripted development lifecycle.

Characteristics:

- runtime state is managed under `artifacts/runtime/dev/`
- startup and shutdown are PID-driven
- default mode is preview, so one unified browser port is available immediately
- startup logs print unified URLs, direct service URLs, credentials, and log paths

Entry points:

- `./bin/start-dev.sh`
- `powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\start-dev.ps1`

Use this mode when:

- you want one stable command for QA, demos, or repeated local validation
- you want a single browser entrypoint by default
- you want the matching `bin/stop-dev.*` lifecycle

## Managed Release Mode

This is the production-oriented script lifecycle.

Characteristics:

- build, install, start, stop, and service registration are separate phases
- the runtime is installed under a dedicated install home
- `router-product-service` serves `/admin/*`, `/portal/*`, and `/api/*`
- designed for foreground execution under managed scripts or an external service manager

Entry points:

- `./bin/build.sh`
- `./bin/install.sh`
- `./bin/start.sh`
- `./bin/stop.sh`

Windows equivalents:

- `.\bin\build.ps1`
- `.\bin\install.ps1`
- `.\bin\start.ps1`
- `.\bin\stop.ps1`

## Choosing The Right Mode

Choose raw standalone service mode when:

- you need direct control over individual binaries
- you are debugging one service in isolation

Choose source browser workspace mode when:

- you want Vite-based frontend iteration
- you need independent browser dev servers

Choose source preview mode when:

- you want one browser-visible port in a source-tree workflow
- you want browser behavior closer to release shape

Choose managed development mode when:

- you want the easiest repeatable local environment
- you want PID, log, and runtime-home management
- you want startup summaries with URLs and credentials

Choose managed release mode when:

- you are packaging or deploying a server runtime
- you need systemd, launchd, or Windows Task Scheduler integration

## Where To Go Next

- startup and shutdown responsibilities:
  - [Script Lifecycle](/getting-started/script-lifecycle)
- onboarding and local startup:
  - [Source Development](/getting-started/source-development)
- compilation and packaging:
  - [Build and Packaging](/getting-started/build-and-packaging)
- deep architecture and runtime supervision:
  - [Runtime Modes Deep Dive](/architecture/runtime-modes)

---
layout: home

hero:
  name: SDKWork API Server
  text: OpenAI-compatible gateway with admin control plane, public portal, and extension runtime
  tagline: Cross-platform Rust services, browser and Tauri console, SQLite and PostgreSQL persistence, and a pluggable channel/provider architecture.
  actions:
    - theme: brand
      text: Installation Guide
      link: /getting-started/installation
    - theme: alt
      text: Source Development
      link: /getting-started/source-development
    - theme: alt
      text: Release Builds
      link: /getting-started/release-builds

features:
  - title: Cross-platform startup
    details: Run the stack on Windows, Linux, or macOS from source with browser mode, desktop mode, or partial service startup.
  - title: Release-ready runtime
    details: Build release binaries for admin, gateway, and portal services, plus browser assets or Tauri desktop packages.
  - title: Public self-service portal
    details: Ship registration, login, workspace inspection, and self-service API key issuance without exposing operator-only admin APIs.
---

## Start Here

Choose the path that matches what you need right now:

- New to the project:
  - go to [Installation](/getting-started/installation)
- Running from source:
  - go to [Source Development](/getting-started/source-development)
- Building distributable artifacts:
  - go to [Release Builds](/getting-started/release-builds)
- Understanding runtime shape:
  - go to [Runtime Modes](/getting-started/runtime-modes)
- Onboarding end users:
  - go to [Public Portal](/getting-started/public-portal)

## What This Repository Contains

Runtime surfaces:

- `gateway-service`
  - OpenAI-compatible `/v1/*` gateway
- `admin-api-service`
  - operator-only `/admin/*` control plane
- `portal-api-service`
  - public `/portal/*` self-service API
- `console/`
  - React shell for browser and Tauri

Platform support:

- Windows
- Linux
- macOS

Persistence:

- SQLite
- PostgreSQL

Secret backends:

- `database_encrypted`
- `local_encrypted_file`
- `os_keyring`

## Core Operational Entry Points

Full stack from source:

```bash
node scripts/dev/start-workspace.mjs
```

Full stack with Tauri:

```bash
node scripts/dev/start-workspace.mjs --tauri
```

Docs local preview:

```bash
pnpm --dir docs install
pnpm --dir docs dev
```

Console local preview:

```bash
pnpm --dir console install
pnpm --dir console dev
```

# Production Release And Installation Design

**Date:** 2026-04-17

## Goal

Define a professional, production-grade release, installation, configuration, and service-management contract for `sdkwork-api-router` that matches mature server software conventions instead of relying on a single portable install layout.

The target outcome is:

- one clear production release path
- one clear server-mode persistence contract based on PostgreSQL
- operating-system-standard install and config directories
- configuration governance centered on config files, with environment variables as discovery and fallback inputs
- service-manager-based startup on Linux, macOS, and Windows

## Current Evidence

- `.github/workflows/release.yml` already defines the governed release pipeline and produces multi-platform native artifacts.
- `deploy/docker/docker-compose.yml` already treats PostgreSQL as the default production-like database for server deployments.
- `deploy/helm/sdkwork-api-router/values.yaml` already expects an externally managed PostgreSQL URL for Kubernetes deployments.
- `deploy/README.md` already describes Docker Compose as a PostgreSQL-backed deployment path and Helm as a PostgreSQL-backed Kubernetes path.
- `services/router-product-service/src/main.rs` already supports config discovery via `SDKWORK_CONFIG_DIR` and `SDKWORK_CONFIG_FILE`, plus direct overrides such as `SDKWORK_DATABASE_URL`.
- `crates/sdkwork-api-config/src/standalone_config.rs` currently loads the config file first and then applies environment overrides, which means environment variables presently override file values.
- `bin/lib/router-runtime-tooling.mjs` currently renders a portable release `router.env` template that defaults `SDKWORK_DATABASE_URL` to a SQLite path under the install root.
- `docs/getting-started/release-builds.md` already documents `bin/build.*`, `bin/install.*`, `bin/start.*`, `bin/stop.*`, plus `service/systemd/`, `service/launchd/`, and `service/windows-task/`, but the documented release story is still portable-install-oriented rather than OS-standard-install-oriented.

## Problem Statement

The current release and deployment assets are directionally strong, but the production contract is still fragmented:

- the release workflow is governed, but the runtime install layout is still centered on a portable install home
- Docker and Helm already imply PostgreSQL for server deployments, while the generated install template still defaults to SQLite
- configuration supports both files and environment variables, but the current precedence is opposite of the desired operational contract
- Windows service management is still represented as Task Scheduler instead of a true long-running service model
- documentation is split across quickstart, release builds, and deployment notes instead of presenting one authoritative production path

This leaves the product below the standard expected from mature infrastructure software such as Nginx, PostgreSQL, or Redis, where release artifacts, install locations, config locations, service registration, and validation flows are explicit and stable.

## Options Considered

### Option A: Keep the existing portable install model and improve docs only

Pros:

- minimal implementation churn
- preserves all existing local workflows
- low short-term risk

Cons:

- production and local install concerns remain mixed
- server-mode PostgreSQL still would not be the actual default install contract
- config governance remains weaker than the target standard
- Windows still lacks a real service-management story

### Option B: Introduce a first-class system install model while preserving portable installs for local and CI use

Pros:

- aligns production installs with mature OS conventions
- keeps existing portable install behavior available for local smoke tests and development workflows
- allows configuration governance, service registration, and validation to become production-grade without breaking all current tooling
- creates a clean migration path from the current portable release model

Cons:

- requires more installer, docs, and service-asset work than a docs-only change
- requires a clear distinction between portable and system installs in scripts and docs

### Option C: Replace the native install path entirely with Docker and Helm

Pros:

- smallest number of supported production topologies
- strongest standardization for modern infrastructure teams

Cons:

- drops an important private-deployment and regulated-environment requirement
- does not satisfy the requirement for OS-standard installs and service managers
- leaves Windows and macOS server installs without a supported story

## Recommendation

Choose Option B.

`sdkwork-api-router` should keep portable installs for local validation and CI smoke coverage, but production delivery must add a first-class system install model with:

- OS-standard program, config, data, log, and run directories
- config-file-first governance
- PostgreSQL as the server-mode default database contract
- service-manager startup as the standard daemon model

This is the smallest change set that reaches infrastructure-software quality without discarding the existing portable toolchain.

## Design

### 1. Release Architecture

The governed release pipeline remains `.github/workflows/release.yml`. Production release output should be described as three standard delivery forms:

- native release bundles
- Linux container image
- Helm chart

The primary production path is:

- Linux `router-product-service`
- PostgreSQL
- Docker Compose for fast single-host deployment
- Helm for Kubernetes deployment

Native installs remain supported on Linux, macOS, and Windows, but they move under a formal system-install contract instead of being documented as a portable-layout-only runtime.

### 2. Install Modes

The product should support two install modes:

- `portable`
- `system`

`portable` mode exists for:

- local development
- CI/runtime smoke tests
- explicit single-directory installs

`system` mode exists for:

- production deployments
- private deployments that require OS-standard locations
- service-manager-based operations

The current `artifacts/install/sdkwork-api-router/current/` layout remains valid only as a portable install target. It should no longer be treated as the canonical production layout in docs or generated server guidance.

### 3. Standard Directory Model

All operating systems should follow the same logical directory categories:

- program home
- config home
- data home
- log home
- run home
- service definition

Program files are versioned and replaceable. Config, data, logs, and run state are external to the program version and survive upgrades and rollbacks.

### 4. Operating-System Default Layouts

#### Linux system install defaults

- program home:
  - `/opt/sdkwork-api-router/releases/<version>/`
  - `/opt/sdkwork-api-router/current/`
- config home:
  - `/etc/sdkwork-api-router/router.yaml`
  - `/etc/sdkwork-api-router/conf.d/`
  - `/etc/sdkwork-api-router/router.env`
- data home:
  - `/var/lib/sdkwork-api-router/`
- log home:
  - `/var/log/sdkwork-api-router/`
- run home:
  - `/run/sdkwork-api-router/`
- service definition:
  - `/etc/systemd/system/sdkwork-api-router.service`

#### macOS system install defaults

- program home:
  - `/usr/local/lib/sdkwork-api-router/releases/<version>/`
  - `/usr/local/lib/sdkwork-api-router/current/`
- config home:
  - `/Library/Application Support/SDKWork/ApiRouter/config/router.yaml`
  - `/Library/Application Support/SDKWork/ApiRouter/config/conf.d/`
  - `/Library/Application Support/SDKWork/ApiRouter/config/router.env`
- data home:
  - `/Library/Application Support/SDKWork/ApiRouter/data/`
- log home:
  - `/Library/Logs/SDKWork/ApiRouter/`
- run home:
  - `/Library/Application Support/SDKWork/ApiRouter/run/`
- service definition:
  - `/Library/LaunchDaemons/com.sdkwork.api-router.plist`

#### Windows system install defaults

- program home:
  - `C:\Program Files\SDKWork\ApiRouter\releases\<version>\`
  - `C:\Program Files\SDKWork\ApiRouter\current\`
- config home:
  - `C:\ProgramData\SDKWork\ApiRouter\config\router.yaml`
  - `C:\ProgramData\SDKWork\ApiRouter\config\conf.d\`
  - `C:\ProgramData\SDKWork\ApiRouter\config\router.env`
- data home:
  - `C:\ProgramData\SDKWork\ApiRouter\data\`
- log home:
  - `C:\ProgramData\SDKWork\ApiRouter\logs\`
- run home:
  - `C:\ProgramData\SDKWork\ApiRouter\run\`
- service definition:
  - Windows Service named `SDKWorkApiRouter`

On Windows, mutable files must not be written back into `Program Files`.

### 5. Configuration Contract

The configuration contract should center on three file layers:

- primary config file: `router.yaml`
- config fragments: `conf.d/*.yaml`
- environment file: `router.env`

Responsibilities:

- `router.yaml`
  - canonical server configuration
- `conf.d/*.yaml`
  - modular overlays for domain-specific concerns such as database, security, or providers
- `router.env`
  - config discovery inputs and minimal environment fallback values

Fragment load order should be lexical within `conf.d/` after `router.yaml`.

### 6. Configuration Precedence

The supported precedence order should be:

- CLI flags
- config file values from `router.yaml` and `conf.d/*.yaml`
- environment variables
- built-in defaults

There is one narrow exception:

- `SDKWORK_CONFIG_FILE`
- `SDKWORK_CONFIG_DIR`

These are config-discovery inputs, not business configuration values. They must be read first so the runtime can locate the config files to load.

Operationally, the runtime should work as follows:

1. resolve `SDKWORK_CONFIG_FILE` and `SDKWORK_CONFIG_DIR`
2. load `router.yaml`
3. load `conf.d/*.yaml`
4. fill only missing fields from environment variables
5. apply CLI overrides last

This changes the current runtime behavior. Today the config loader loads file values and then lets environment variables overwrite them. That behavior should be reversed for all business configuration fields.

### 7. Server Database Contract

The server-mode persistence contract should be explicit:

- PostgreSQL is the default database for `system` installs and documented production deployments
- SQLite remains supported only for local development, testing, and explicit portable validation flows

Concrete changes implied by this contract:

- generated system install templates must default `SDKWORK_DATABASE_URL` to a PostgreSQL placeholder, not a SQLite file path
- Docker Compose remains PostgreSQL-backed by default
- Helm remains PostgreSQL-backed by default
- production docs must describe SQLite as a development-only path

For stricter safety, the production contract should fail fast when both are true:

- the runtime is in a production-facing server posture
- the resolved database URL still points to SQLite

At minimum, the runtime should surface a strong validation error. The preferred behavior is startup refusal in production profile or system install mode unless an explicit development override is present.

### 8. Service-Manager Model

The application should run in the foreground. Service supervision belongs to the operating system.

Standard service managers:

- Linux: `systemd`
- macOS: `launchd`
- Windows: Windows Service Control Manager

Task Scheduler should not remain the primary Windows production story. It may survive temporarily as a compatibility path, but the formal standard should move to a real Windows Service wrapper.

The service registration assets should therefore converge to:

- `service/systemd/`
- `service/launchd/`
- `service/windows-service/`

Each service manager should invoke the foreground startup form of the runtime and reference the OS-standard config and log locations.

### 9. Install Lifecycle Commands

The managed install flow should become explicit and auditable, similar to mature server products:

- `install`
- `configure`
- `validate-config`
- `register-service`
- `start`
- `stop`
- `upgrade`
- `rollback`
- `uninstall`

`validate-config` is especially important. It should play the same governance role as `nginx -t`:

- parse config files
- validate precedence resolution
- confirm required directories
- validate PostgreSQL connectivity when configured
- confirm required secrets are present
- reject invalid bind or path combinations before service start

### 10. Documentation Structure

Production docs should be reorganized so one page is the canonical entry point.

Recommended structure:

- `README.md`
  - short entrypoint only
- `docs/getting-started/production-deployment.md`
  - the canonical production deployment guide
- `docs/operations/install-layout.md`
  - OS-specific layout and directory standards
- `docs/operations/service-management.md`
  - systemd, launchd, and Windows Service registration and operations
- `docs/getting-started/release-builds.md`
  - build and package generation only
- `docs/getting-started/quickstart.md`
  - local development only
- `deploy/README.md`
  - Docker and Helm asset notes only

The docs must clearly separate:

- source development
- portable install validation
- production deployment

### 11. Migration Strategy

This slice should preserve current workflows while introducing the new standard:

- keep portable install behavior for existing local scripts and release smoke tests
- add a first-class system install path and document it as the production standard
- keep current Windows Task Scheduler assets only as an interim compatibility path
- introduce Windows Service assets as the target standard
- migrate generated runtime templates from SQLite defaults toward PostgreSQL placeholders for system installs

### 12. Verification And Acceptance

The design is only complete when all of the following are true:

- generated install assets can produce a portable install and a system install
- system install defaults match the target OS conventions
- runtime config resolution obeys:
  - config discovery via env
  - config-file-first business settings
  - env fallback only for missing fields
- server/system install templates default to PostgreSQL placeholders
- service assets exist for systemd, launchd, and Windows Service
- docs provide one authoritative production release and deployment path
- regression tests protect the new config precedence, install layout generation, and service asset generation behavior

## Risks And Mitigations

### Risk: changing config precedence breaks current environment-driven deployments

Mitigation:

- keep config discovery env keys stable
- document the precedence change clearly
- support environment fallback for fields absent from config files
- add regression tests for both config-file-first and env-fallback paths

### Risk: OS-standard layouts increase installer complexity

Mitigation:

- keep portable mode for local and CI workflows
- isolate system-install layout generation in dedicated installer helpers
- preserve `--home` style overrides for advanced operators

### Risk: Windows Service support introduces new operational surface area

Mitigation:

- stage Windows Service support as the new standard while preserving Task Scheduler as a temporary fallback
- verify service registration and startup via explicit installer tests

### Risk: production posture still allows SQLite through hidden paths

Mitigation:

- change generated system templates to PostgreSQL placeholders
- add validation that flags or blocks SQLite in production-facing server mode
- align docs, Docker assets, and Helm assets around the same database contract

## Success Condition

This work is successful when `sdkwork-api-router` behaves like professional server software rather than a portable developer bundle:

- the release pipeline still governs artifacts
- production installs use OS-standard locations
- the runtime uses config files as the primary source of truth
- environment variables are discovery and fallback inputs, not silent overrides
- server deployments default to PostgreSQL
- the product runs under systemd, launchd, or Windows Service management
- documentation exposes one clear production release and deployment story

# Public Portal and Cross-Platform Console Design

**Date:** 2026-03-14

## Context

The repository already has:

- a stateful OpenAI-compatible gateway
- an operator-facing admin control plane
- a React console focused on admin and routing observability
- Tauri support for an embedded desktop shell

It does not yet have:

- a public self-service user system
- browser-friendly user registration and login flows
- self-service API key issuance for external users
- a dedicated portal backend surface separated from admin APIs
- a package-bounded frontend user portal module
- startup documentation that treats Windows, Linux, and macOS as first-class operational targets

## Goal

Add a public self-service portal that lets end users register, log in, inspect their workspace, and issue gateway API keys without using operator-only admin APIs, while keeping the current DDD and layered architecture intact.

## Approaches

### Option A: Extend `/admin/*` with public routes

Pros:

- least code movement
- reuses the current service and router

Cons:

- mixes operator and end-user concerns in one interface boundary
- makes auth semantics harder to reason about
- weakens the long-term bounded-context split

### Option B: Add a dedicated portal interface and service

Pros:

- keeps admin and public concerns separate
- preserves controller -> app -> repository layering
- gives the frontend a clean public API surface
- scales to hosted browser access and embedded desktop access

Cons:

- adds one more service and router
- requires additive storage and identity modeling

### Option C: Frontend-only portal backed by admin APIs

Pros:

- fastest initial UI

Cons:

- not a real public system
- admin auth and public auth remain conflated
- impossible to expose safely to external users

## Recommendation

Choose **Option B**.

The repository already treats interfaces as explicit boundaries. A dedicated public portal interface is the correct additive move: admin remains operator-only, gateway remains OpenAI-compatible, and portal becomes the self-service product surface.

## Target Architecture

### Backend

Add a new public interface boundary:

- `crates/sdkwork-api-interface-portal`
- `services/portal-api-service`

Reuse existing storage and application layers where possible, but keep public contracts isolated.

Recommended additive backend responsibilities:

- `sdkwork-api-domain-identity`
  - extend with `PortalUserRecord`
  - extend with workspace ownership metadata
- `sdkwork-api-app-identity`
  - portal registration
  - portal login
  - portal JWT issue and verify
  - workspace-scoped API key listing and creation
- `sdkwork-api-storage-core`
  - add repository methods for portal users
- `sdkwork-api-storage-sqlite`
  - migrate `identity_users` from placeholder storage to a real portal account table
- `sdkwork-api-storage-postgres`
  - mirror the same schema evolution

### Public API Surface

The public router should expose:

- `POST /portal/auth/register`
- `POST /portal/auth/login`
- `GET /portal/auth/me`
- `GET /portal/workspace`
- `GET /portal/api-keys`
- `POST /portal/api-keys`

Behavior:

- registration creates a portal user plus a default tenant and project workspace
- login returns a portal JWT, not an admin JWT
- authenticated portal users only see their own tenant or project scope
- API key creation reuses the existing gateway API key issuance path but is constrained to the caller's workspace

### Identity Model

For the current implementation batch, one portal user owns one default workspace:

- one tenant
- one default project

This is the right YAGNI line for now. It gives users a usable self-service environment without introducing multi-workspace membership, invitations, or RBAC before the portal exists.

Recommended persisted fields:

- `id`
- `email`
- `display_name`
- `password_salt`
- `password_hash`
- `workspace_tenant_id`
- `workspace_project_id`
- `active`
- `created_at_ms`

### Frontend

Keep the root app thin and compositional. Add dedicated portal packages under `console/packages/`:

- `sdkwork-api-portal-sdk`
- `sdkwork-api-portal-auth`
- `sdkwork-api-portal-user`

Responsibilities:

- `portal-sdk`
  - typed fetch client for `/portal/*`
- `portal-auth`
  - register and login pages
  - session persistence
- `portal-user`
  - portal dashboard
  - API key management
  - workspace summary

The root `console/src/` remains the shell and route composition layer.

### Routing and UX

The console should support both browser and Tauri access from the same build.

Recommended route map:

- `#/portal/register`
- `#/portal/login`
- `#/portal/dashboard`
- `#/admin`

Use hash routing for this batch because it works cleanly in:

- browser-hosted Vite dev
- browser preview mode
- Tauri desktop file-hosted runtime

### Browser Access During Desktop Development

The same portal UI should be reachable through both:

- the Tauri window
- a normal browser pointed at the console dev or preview server

That means the console shell must not assume a Tauri-only environment. Tauri becomes a host, not a separate frontend.

### Cross-Platform Startup

README must explicitly document:

- Windows PowerShell examples
- Linux shell examples
- macOS shell examples
- standalone server startup
- portal service startup
- console browser startup
- Tauri desktop startup
- combined desktop plus browser workflow

## Security and Error Handling

Portal auth must be distinct from admin auth:

- different JWT audience and issuer
- different auth extractor
- different route namespace

Registration and login errors should return:

- `400` for invalid payload
- `401` for invalid credentials
- `409` for duplicate email
- `500` only for unexpected persistence failures

API key plaintext should be shown exactly once on creation and never returned from list endpoints.

## Testing Strategy

Add TDD-first coverage for:

- portal registration
- portal login
- portal auth guard
- portal workspace summary
- portal API key self-service
- browser-friendly portal SDK flows where practical
- package-level typecheck and build verification

## Non-Goals For This Batch

Do not add:

- multi-user workspace membership
- invitations
- password reset email
- OAuth or SSO
- packaged desktop static web serving without the console web server

Those can be layered later once the public portal surface is stable.

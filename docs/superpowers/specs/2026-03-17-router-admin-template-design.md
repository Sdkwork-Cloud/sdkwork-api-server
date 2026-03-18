# SDKWork Router Admin Template Design

## Goal

Build  into a reusable super-admin template for SDKWork
projects while preserving the current operator-facing module vocabulary:
, sdkwork, , , , , and
.

The template must satisfy four constraints:

1. business modules are easy to recognize at a glance
2. module ownership remains high-cohesion and low-coupling
3. shared admin foundations are reusable across future projects
4. the frontend reflects the actual control-plane capabilities of
   

## Current Diagnosis

### What is already good

- The repository already has a real control plane, not just a demo admin UI.
- Business module naming is clear and operator-friendly.
- The backend control plane already owns identities, workspaces, catalog,
  traffic, billing, routing, runtime status, extension installations, and
  rollout orchestration.

### What is missing

- The admin browser app does not expose the full backend control-plane surface.
- The frontend package layout does not follow  package standards.
-  and  are too
  centralized.
- Audit is not treated as a first-class operator product surface.
- Local bootstrap credentials are too visible for a production-grade super-admin
  posture.

## Product Domain Model

The admin template keeps the current operator-facing module names and adds one
governance module:



### Overview

- purpose: control tower and summary surface
- owns: posture cards, risk summary, pending operator attention, drill-down entry
  points
- does not own: CRUD flows for business entities

### Users

- purpose: identity and access governance
- owns: operator users, portal users, status changes, password resets, access
  posture
- references: tenant and project bindings
- does not own: workspace lifecycle

### Tenants

- purpose: workspace lifecycle management
- owns: tenants, projects, gateway API keys, ownership safety checks
- does not own: user authentication or usage analytics logic

### Coupons

- purpose: commercial promotion assets
- owns: coupon campaigns, inventory, activation posture, archive/restore
- future extension: credits, trial campaigns, redemption policy

### Catalog

- purpose: routing asset registry
- owns: channels, providers, credentials, models, asset integrity checks
- future extension: routing asset configuration views
- does not own: runtime execution

### Traffic

- purpose: analytical and financial visibility
- owns: usage records, usage summary, billing summary, quota posture, exports,
  leaderboards
- does not own: catalog mutation or runtime control

### Operations

- purpose: live runtime and operational control
- owns: extension packages, installations, instances, runtime statuses, health,
  reloads, rollouts, config rollout status
- does not own: static catalog definition

### Audit

- purpose: operator governance and change traceability
- owns: admin actions, sensitive changes, runtime operation history, resource
  history, exportable audit evidence

## Engineering Package Model

The top-level package graph stays readable and business-first:



### Foundation packages

- : stable shared contracts only
- : admin-localized copy and formatting helpers
- : shared UI and interaction primitives
- : app shell, route registration, guards, layout, notifications, root
  providers
- : HTTP client and route-family repositories
- : login/session/change-password surface

### Business packages

- 
- sdkwork
- 
- 
- 
- 
- 
- 

## Required Internal Package Structure

Every business package must grow into a real module instead of a single-page
file:



Notes:

-  and  may start small but should exist to establish the
  template boundary.
-  contains rules, value objects, and module-owned derived logic.
-  is the route-level composition entry.
-  isolates API DTOs from page components.

## Cross-Domain Collaboration Rules

- business packages must not import other business packages directly
- shared contracts flow through 
- shared UI flows through 
- routing, layout, permissions, and shell orchestration live in 
- transport and DTO translation live in 
- overview only consumes aggregated read models
- audit consumes event history and read models, but does not own mutations

## Backend Alignment Rules

The admin frontend must eventually represent the real control-plane surface that
already exists in :

- identities
- tenants/projects/api keys
- catalog assets
- usage and billing summaries
- quota policies
- routing policies and simulations
- extension packages/installations/instances
- runtime reloads and rollouts
- runtime config rollouts

This alignment should happen without renaming the current business modules.
Missing backend capabilities should be mapped into existing modules by ownership:

- routing policies and simulations ->  or  read models,
  depending on write/read concerns
- quota policies -> 
- extension packages/installations/instances -> 
- rollout and config rollout history ->  plus 

## Security Direction

The admin template must retain local bootstrap ergonomics without exposing
default credentials as part of the steady-state product UX.

Direction:

- remove visible default credentials from browser UI
- move bootstrap handling behind explicit development posture
- keep local-first onboarding, but make it operationally safe

## Implementation Strategy

The migration should proceed in phases:

1. codify the target package architecture with tests
2. add missing foundation packages and business packages
3. split frontend transport and shell logic by responsibility
4. expand operations and audit surfaces to reflect backend capabilities
5. harden security posture around admin bootstrap and credential exposure

## Success Criteria

- the admin app package graph matches 
- business modules remain readable at first glance
- each business package owns its own domain, repository, service, and page
  boundaries
- frontend exposes the key existing backend control-plane capabilities
- audit becomes a first-class operator product surface
- admin auth posture no longer relies on visible hard-coded credentials

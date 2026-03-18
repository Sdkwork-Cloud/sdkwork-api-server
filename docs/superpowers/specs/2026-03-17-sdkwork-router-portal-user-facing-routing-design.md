# SDKWork Router Portal User-Facing Routing Design

## Context

`sdkwork-api-server` already ships four runtime surfaces:

- gateway: OpenAI-compatible `/v1/*` data plane
- admin: operator-facing `/admin/*` control plane
- portal: public `/portal/*` self-service API
- web host: public delivery for `/admin/*` and `/portal/*`

The current browser portal already supports:

- registration and login
- workspace summary
- dashboard snapshot
- API key issuance
- usage records and summary
- billing summary and ledger

The current gateway and admin surfaces already support:

- routing policy persistence
- provider health snapshots
- routing simulations
- routing decision logs
- deterministic, weighted, SLO-aware, and geo-affinity routing
- project-scoped gateway authentication
- project-scoped quota admission
- usage and ledger recording after real gateway execution

The current gap is not backend maturity. The gap is that the public portal does not yet translate those routing capabilities into a user-facing product.

## Product Positioning

`sdkwork-router-portal` should be the customer-facing self-service routing workspace for SDKWork Router.

It is not:

- a simplified admin console
- a usage-only dashboard
- a billing-only developer portal

It should help a customer do four things without touching operator controls:

1. create and secure their workspace
2. issue and govern API keys
3. understand and control how requests route
4. understand usage, spend, quota, and account posture

## Product Goals

Primary goals:

- make first-time API onboarding easy
- make routing behavior understandable, not magical
- make cost and quota consequences visible next to routing choices
- keep the user in a customer-safe boundary without leaking operator-only controls

Non-goals for this batch:

- organization-wide RBAC
- approval workflows
- invoices and payment processors
- operator-side provider or global routing governance

## Capability Audit

### What the backend already supports

- project-bound API keys through `GatewayApiKeyRecord`
- project-bound portal users through workspace tenant and project fields
- gateway request context resolution from API keys
- project-level usage and billing summaries
- route simulation with explainable candidate assessments
- geo-affinity through `x-sdkwork-region`
- SLO-aware routing via cost, latency, and health thresholds
- weighted provider selection
- persisted routing decision logs

### What the public portal does not yet support

- reading routing strategy state as a portal user
- previewing route decisions as a portal user
- reading project routing evidence from the portal
- separating `user` from `account`
- expressing a professional OpenRouter-like routing workflow in the UI

### Important current backend constraint

Current `RoutingPolicy` is global and does not carry `tenant_id` or `project_id`.

That means the portal should not directly expose raw routing-policy CRUD to end users in this batch. Instead, the user-facing portal should expose:

- read-only routing substrate visibility
- project-default routing preferences
- route preview against the active project
- recent routing evidence scoped to the project

## Target Information Architecture

Primary navigation:

- Overview
- Routing
- API Keys
- Usage
- User
- Account
- Credits
- Billing

Supporting flows:

- Register
- Login
- First-run onboarding guidance inside Overview and Routing

## Package Map

Foundation:

- `sdkwork-router-portal-types`
- `sdkwork-router-portal-i18n`
- `sdkwork-router-portal-commons`
- `sdkwork-router-portal-core`
- `sdkwork-router-portal-portal-api`
- `sdkwork-router-portal-commerce-api`

Business:

- `sdkwork-router-portal-auth`
- `sdkwork-router-portal-dashboard`
- `sdkwork-router-portal-routing`
- `sdkwork-router-portal-api-keys`
- `sdkwork-router-portal-usage`
- `sdkwork-router-portal-user`
- `sdkwork-router-portal-account`
- `sdkwork-router-portal-credits`
- `sdkwork-router-portal-billing`

Future:

- `sdkwork-router-portal-workspace`
- `sdkwork-router-portal-members`
- `sdkwork-router-portal-support`

## Module Design

### Overview

Purpose:

- orient the user in one screen
- answer "what should I do next?"
- summarize routing, usage, and financial posture

Key surfaces:

- workspace pulse
- current default routing mode
- recent routing evidence
- launch and production readiness
- usage and quota snapshot
- lead risk and recommended next move

Source data:

- `/portal/dashboard`
- `/portal/routing/summary`
- `/portal/routing/decision-logs`

### Routing

Purpose:

- make routing behavior explainable
- let the user choose a default routing posture for the project
- let the user preview outcomes before sending live traffic

Key surfaces:

- default strategy card
- strategy presets
- provider ordering preferences
- health and latency guardrails
- region preference and geo-affinity explanation
- route preview workbench
- recent routing evidence and why selections happened

User-facing language should translate backend strategy into product concepts:

- `deterministic_priority` -> predictable order
- `weighted_random` -> traffic distribution
- `slo_aware` -> reliability guardrails
- `geo_affinity` -> regional preference

Source data:

- `/portal/routing/summary`
- `/portal/routing/preferences`
- `/portal/routing/preview`
- `/portal/routing/decision-logs`

### API Keys

Purpose:

- issue project-scoped credentials
- make environment posture obvious
- connect new keys to routing and usage verification

Key surfaces:

- environment coverage
- issue key
- copy-once plaintext handling
- route inheritance note
- quickstart request sample
- next-step links into Routing and Usage

### Usage

Purpose:

- show actual request behavior, not only configured intent
- help users connect routing choices to spend and provider outcomes

Key surfaces:

- request telemetry metrics
- model distribution
- provider distribution
- request table
- spend watch
- routing evidence summary

Important improvement:

Usage should explicitly surface "observed provider path" so routing and usage feel like one workflow.

### User

Purpose:

- manage the current signed-in person
- separate personal profile and credentials from workspace money

Key surfaces:

- profile facts
- password rotation
- personal security checklist
- user identity summary

Source data:

- `/portal/auth/me`
- `/portal/auth/change-password`
- `/portal/workspace`

### Account

Purpose:

- represent the customer money and account posture domain
- separate account balance posture from plan merchandising

Key surfaces:

- account summary
- ledger overview
- spend breakdown
- account health
- financial next action

Source data in this batch:

- `/portal/billing/summary`
- `/portal/billing/ledger`

Note:

This batch uses billing summary plus ledger as the practical account read model. A richer account aggregate can come later.

### Credits

Purpose:

- present quota and promo mechanics in a user-friendly way
- keep depletion risk understandable

Key surfaces:

- remaining quota
- used units
- guardrails
- coupon preview
- recharge recommendation

### Billing

Purpose:

- productize commercial choice, not just display price cards

Key surfaces:

- current posture
- recommended plan and pack
- projected runway
- plan comparison
- recharge path
- checkout-intent placeholder

## Public Portal API Changes

### New route family: routing

Add project-scoped portal routing endpoints:

- `GET /portal/routing/summary`
- `GET /portal/routing/preferences`
- `POST /portal/routing/preferences`
- `POST /portal/routing/preview`
- `GET /portal/routing/decision-logs`

### New portal routing contracts

Portal routing summary should include:

- current project id
- active user-facing strategy preset
- effective strategy
- current routing guardrails
- recent selected provider
- recent matched policy id
- route health snapshot

Portal routing preferences should include:

- preset id
- strategy
- ordered provider ids
- default provider id
- max cost
- max latency ms
- require healthy
- preferred region

Routing preview request should include:

- capability
- model
- optional requested region
- optional selection seed

Routing preview response should expose existing routing explainability fields:

- selected provider id
- candidate ids
- matched policy id
- strategy
- selection reason
- requested region
- slo state
- assessments

Decision log response should be project-scoped and portal-safe.

## Backend Ownership Model

This batch should not let portal users mutate global `RoutingPolicy`.

Instead it should add a project-scoped `PortalRoutingPreferences` aggregate that:

- belongs to one project
- stores a user-facing default routing posture
- translates into preview defaults and UI summaries
- remains compatible with the existing operator routing substrate

This keeps control-plane ownership clear:

- admin owns provider catalog and global routing policy
- portal user owns project preference and self-service interpretation

## Frontend Ownership Model

The root app should become a real composition layer:

- route manifest
- shell composition
- provider wiring

`sdkwork-router-portal-core` should shrink to shell/runtime helpers and shared route composition helpers, not import every business page directly.

## User Experience Principles

- always explain why a route was chosen
- always connect routing to usage and cost consequences
- never expose operator-only identifiers without customer-safe translation
- never end a page without a meaningful next move
- keep first-run guidance visible until the user has a key, a previewed route, and usage evidence

## Implementation Scope for This Batch

Included:

- add `Routing` to portal IA
- add `User` as a separate module
- repurpose `Account` to financial account posture
- add portal routing APIs
- add project-scoped portal routing preferences
- add routing preview and recent project routing evidence
- update Overview to surface routing posture

Deferred:

- member management
- workspace settings package
- payment methods
- invoices
- policy approval workflows
- per-key routing override

## Success Criteria

- a new user can understand how their requests will route before sending traffic
- a signed-in user can preview route decisions without admin access
- the portal clearly separates person, money, quota, and routing concepts
- routing, usage, and billing feel like one coherent workflow
- package ownership aligns with `ARCHITECT.md`

# Router Gateway Command Center Design

**Date:** 2026-03-28

**Status:** Approved by the user's standing instruction to continue autonomously and choose the strongest product direction without waiting for interactive review

## Goal

Turn the existing router runtime, compatibility layers, desktop host, server host, and commerce posture into one explicit product surface inside `sdkwork-router-portal`.

This batch adds a new portal workspace area, `Gateway`, that acts as the command center for:

- protocol compatibility
- local versus server deployment modes
- desktop-owned service orchestration visibility
- role-sliced server topology planning
- commercial readiness cues that connect access, traffic posture, and billing recovery

## Why This Batch

The repository already has unusually strong substrate coverage:

- OpenAI-compatible gateway routes
- Anthropic Messages compatibility
- Gemini Generative Language compatibility
- desktop mode that starts gateway, admin, portal, and public web host together
- server mode with role slicing across `web`, `gateway`, `admin`, and `portal`
- portal billing, credits, account, and API key workflows

What is still weak is the product surface that explains and operationalizes those capabilities.

Right now a large share of the real value is discoverable only through:

- `README.md`
- runtime docs
- product scripts
- compatibility docs
- API key quick-setup dialogs

That is not an OpenRouter-grade product posture. A powerful gateway must make its operating model obvious before the user reads source code.

## Scope

This batch will implement:

1. a first-class `Gateway` route in `sdkwork-router-portal`
2. a page that presents the router as one product with three linked concerns:
   - compatibility and client onboarding
   - deployment modes and role topology
   - commerce and readiness posture
3. navigation and route-model updates so the new area is part of the shell, not an isolated experiment
4. product-facing copy and layout that make desktop mode and server mode legible
5. documentation updates so the new product story is reflected outside the UI

This batch will not implement:

- a new backend API for runtime control
- remote process management from the browser
- payment gateway integration
- a full OpenRouter-style public catalog marketplace
- protocol additions that require net-new gateway relay handlers

## Options Considered

### Option A: Only expand docs and README

Pros:

- cheapest implementation
- no new UI surface

Cons:

- leaves the portal product incomplete
- keeps key operating knowledge outside the actual product
- does not improve discoverability for desktop users

### Option B: Add a new product-facing `Gateway` command center inside the portal

Pros:

- converts buried runtime capability into visible product capability
- connects onboarding, deployment, and billing into one decision surface
- fits the existing portal shell and route model cleanly
- highest leverage change without inventing new backend contracts

Cons:

- requires route, page, and shell updates
- must stay disciplined to avoid becoming a duplicate of README plus docs

### Option C: Build a full runtime-control console first

Pros:

- ambitious and operator-friendly

Cons:

- too large for a single autonomous batch
- would require new backend control APIs and deeper orchestration contracts
- risks replacing product clarity with half-finished control knobs

## Recommendation

Use **Option B**.

The right move is not to expand backend scope again. The right move is to surface the existing platform as a coherent product layer that users can understand immediately.

## Product Model

The new `Gateway` area should answer four questions in one page:

1. What protocols and tools does this router speak?
2. How do I run it locally on my machine?
3. How do I deploy it as a server product with split roles?
4. Is my workspace commercially ready for real traffic?

Those four questions map directly onto user outcomes instead of internal implementation layers.

## Information Architecture

The page should be split into five sections:

1. **Gateway posture**
   - present the router as a single product entrypoint
   - summarize desktop mode, server mode, and gateway endpoint posture
2. **Compatibility matrix**
   - tool rows such as Codex, Claude Code, OpenCode, Gemini-compatible clients, and OpenClaw
   - protocol truth labels such as OpenAI, Anthropic, and Gemini
   - note which path is native, translated, or desktop-assisted
3. **Mode switchboard**
   - local desktop mode
   - server mode
   - role-sliced deployment
   - clarify which services run in each mode
4. **Topology and launch playbooks**
   - single-node local product
   - single-node server
   - edge-only web node
   - split control-plane versus data-plane example
5. **Readiness and commerce**
   - tie API key readiness, routing posture, remaining runway, and billing recovery into one operating summary

## UI Principles

The page should feel like a control surface, not marketing copy.

Rules:

- lead with operating clarity, not slogans
- use compact evidence cards and structured lists
- reuse existing portal surface patterns so the route feels native
- connect to existing modules through strong next actions such as `Open API Keys`, `Open Routing`, and `Open Billing`

## Data Model

This batch should stay frontend-local.

The page can use a small, explicit local view model that derives from:

- hard-coded product constants for compatibility and topology
- existing dashboard and billing snapshot posture when available

This avoids inventing backend dependencies for content that is effectively product reference material.

## Architecture Impact

Frontend changes:

- add `gateway` to `PortalRouteKey`
- add route manifest and path wiring
- add a dedicated portal package for the page so the module boundary stays clean

Documentation changes:

- update compatibility and runtime docs to reflect the product-entrypoint story
- align portal README with the new route

No backend contract changes are required in this batch.

## Testing Strategy

This batch should be proven through:

1. route and architecture tests proving the new package and route are wired into the shell
2. product-polish tests proving the page contains:
   - desktop mode
   - server mode
   - role topology
   - tool compatibility
   - commerce readiness links
3. typecheck and build verification for the portal app

## Follow-On Work

After this batch, the strongest remaining gaps should be:

1. browser-visible runtime health and process status from a real API
2. checkout and payment-provider integration behind the existing commerce seam
3. public model catalog and provider ranking views comparable to hosted router products

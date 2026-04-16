# Gateway Mirror Protocol Baseline Design

**Date:** 2026-04-16

## Goal

Formalize the gateway as a mirror-style API router that lets supported clients switch only the `base_url` and continue calling the corresponding official protocol surface without SDK-level request rewrites. The first delivery slice fixes the gateway OpenAPI baseline and promotes the existing OpenAI/Codex, Claude, and Gemini protocol surfaces into first-class public contract groups.

## Current State

- the live gateway router already exposes official-style protocol paths across `/v1/*`, `/v1/messages*`, and `/v1beta/models/*`
- the generated gateway OpenAPI document does not cover all public routes currently exposed by the real router
- the public OpenAPI tags still publish a generic `compatibility` group that mixes Anthropic and Gemini protocol surfaces into one implementation-facing bucket
- [`crates/sdkwork-api-interface-http/src/gateway_openapi_paths_vector_compat.rs`](D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router/crates/sdkwork-api-interface-http/src/gateway_openapi_paths_vector_compat.rs) currently mixes `vector-stores` with Claude and Gemini protocol documentation, which is a broken responsibility boundary
- route families such as `containers`, `videos`, `music`, `fine_tuning`, `webhooks`, `evals`, and `files content` are exposed by the real router but not fully represented in the gateway OpenAPI contract
- existing docs already state that the gateway is an OpenAI-compatible data plane plus translated Anthropic and Gemini surfaces, but the external contract taxonomy is not yet governed as a mirror-protocol product surface

## Problem Statement

The gateway needs to behave like a protocol mirror, not a custom wrapper API. Today the implementation already contains enough routing behavior to serve that direction, but the public contract surface still leaks internal concepts:

- `compatibility` is an internal implementation concern, not a public product category
- OpenAPI drift means clients and operators cannot treat `/openapi.json` as a trustworthy mirror of the live gateway router
- the current tag structure does not establish a stable taxonomy for future protocol families across code, images, video, and music
- if future provider-specific routes are added without contract rules, the project can easily regress into a mix of official paths, wrapper paths, and undocumented custom surfaces

## Options Considered

### Option A: Keep the current `compatibility` grouping and expand it

This path would preserve the current public tag model and add more provider-specific routes under the same umbrella.

Pros:

- lowest short-term churn for the gateway OpenAPI source file
- avoids immediate tag migration for the existing Anthropic and Gemini entries

Cons:

- keeps leaking an internal implementation term into the public contract
- scales poorly when images, video, and music provider mirrors are added
- does not clearly express that clients should continue using official protocol paths

### Option B: Use official protocol paths as the public contract and use `capability.protocol` only for OpenAPI grouping

This path keeps the external HTTP paths identical to the official protocol surfaces while rebuilding documentation, tags, and governance around stable `capability.protocol` groups.

Pros:

- matches the product requirement that clients only switch `base_url`
- keeps router behavior aligned with official SDK expectations
- gives a stable taxonomy for future `images.*`, `video.*`, and `music.*` protocol groups
- avoids creating fake wrapper prefixes that would break the mirror model

Cons:

- requires a larger OpenAPI cleanup in the current gateway interface crate
- requires explicit governance to keep router paths and OpenAPI path lists aligned

### Option C: Introduce new platform-specific wrapper prefixes such as `/code/*`, `/claude/*`, or `/gemini/*`

Pros:

- can look tidy from an internal platform taxonomy perspective

Cons:

- breaks the core mirror-router requirement
- forces client rewrites beyond changing the base URL
- creates a second public protocol that would need separate lifecycle management

## Recommendation

Adopt Option B.

The gateway should mirror the official external protocol paths and reserve grouping changes for documentation, OpenAPI navigation, and governance only. The mirror rule is the actual product requirement, while capability grouping is only a presentation and maintenance tool.

## Design

### External Contract Rules

The gateway public contract follows these non-negotiable rules:

- if a provider already has a stable official protocol surface, mirror that protocol exactly
- if OpenAI already provides an equivalent widely adopted standard surface, use the OpenAI contract as the gateway's general standard contract
- if no OpenAI-equivalent standard exists, expose the provider-specific official protocol as an independent mirror surface instead of pretending it is a gateway-wide generic contract
- do not invent wrapper prefixes or alternate public paths for grouping convenience
- do not expose `compatibility` as a top-level public product concept

### Phase 1 Scope

Phase 1 is intentionally narrow. It only delivers the mirror baseline and the code protocol taxonomy.

Phase 1 includes:

- correcting the gateway OpenAPI path inventory so it matches the live public router
- removing the public `compatibility` tag
- productizing three first-class code protocol groups:
  - `code.openai`
  - `code.claude`
  - `code.gemini`
- documenting the existing OpenAI/Codex, Claude, and Gemini protocol surfaces as official mirror entry points
- adding OpenAPI coverage for already-exposed route families that are currently missing from `/openapi.json`

Phase 1 does not include:

- adding any new public wrapper routes
- changing the existing official mirror paths
- introducing provider-specific images, video, or music mirror routes that have not yet been formally specified
- changing the shared routing, quota, billing, settlement, or audit execution model behind the gateway

### Public Path Preservation

The public HTTP paths remain official and unchanged:

- OpenAI/Codex mirror routes stay on `/v1/*`
- Claude mirror routes stay on `/v1/messages` and `/v1/messages/count_tokens`
- Gemini mirror routes stay on `/v1beta/models/{model}:*`

Grouping is a documentation concern only. Tags must not become path prefixes.

### OpenAPI Tag Taxonomy

The gateway OpenAPI must move to stable ASCII-only `capability.protocol` tags.

Phase 1 tags:

- `code.openai`
- `code.claude`
- `code.gemini`
- `images.openai`
- `audio.openai`
- `storage.openai`
- `agents.openai`
- `jobs.openai`
- `video.openai`
- `music.openai`
- `market.sdkwork`
- `marketing.sdkwork`
- `commercial.sdkwork`
- `system.sdkwork`

The first three are the important contract changes in Phase 1. The other tags exist to stop future route families from collapsing back into generic buckets.

Reserved future protocol tags:

- `images.nanobanana`
- `images.midjourney`
- `images.volcengine`
- `images.aliyun`
- `images.kling`
- `video.sora`
- `video.minimax`
- `video.vidu`
- `video.volcengine`
- `video.google-veo`
- `video.aliyun`
- `video.kling`
- `music.suno`
- `music.google`
- `music.minimax`

The tags remain stable even when provider model generations change. For example, `video.sora` is the tag, while `Sora 2` remains descriptive text in operation summaries and descriptions.

### Operation Naming

OpenAPI `operationId` values should use `capability_protocol_action`.

Examples:

- `code_openai_chat_completions_create`
- `code_claude_messages_create`
- `code_gemini_generate_content`
- `video_openai_create`
- `music_openai_lyrics_create`

These IDs are stable, ASCII-only, and readable in generated client tooling.

### OpenAPI Source Layout

[`crates/sdkwork-api-interface-http/src/gateway_openapi.rs`](D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router/crates/sdkwork-api-interface-http/src/gateway_openapi.rs) remains the single aggregation point for the gateway OpenAPI document, but the path modules must be split by responsibility.

Target module layout:

- `gateway_openapi_paths_code_openai.rs`
- `gateway_openapi_paths_code_claude.rs`
- `gateway_openapi_paths_code_gemini.rs`
- `gateway_openapi_paths_storage.rs`
- `gateway_openapi_paths_agents.rs`
- `gateway_openapi_paths_jobs.rs`
- `gateway_openapi_paths_media.rs`
- `gateway_openapi_paths_video.rs`
- `gateway_openapi_paths_music.rs`
- `gateway_openapi_paths_market_commercial.rs`

This split fixes the current boundary problem where vector store documentation and mirror protocol documentation are co-located in the same module.

### Router To OpenAPI Parity Boundary

The live router remains the real runtime source of truth, but the OpenAPI document is the published contract surface and must stay in lockstep with it.

Phase 1 requires explicit OpenAPI coverage for these already-public route families:

- `/v1/containers*`
- `/v1/files/{file_id}/content`
- `/v1/videos*`
- `/v1/music*`
- `/v1/fine_tuning/*`
- `/v1/webhooks*`
- `/v1/evals*`

The goal is not to change these routes. The goal is to stop publishing an incomplete contract.

### Execution Model

The gateway continues using one shared execution kernel behind all protocol surfaces:

- request identity resolution
- tenant and project binding
- routing candidate selection
- quota and admission checks
- billing and settlement accounting
- audit and tracing capture
- provider execution and streaming relay

Mirror entry points must converge into the same execution chain instead of creating parallel billing or routing semantics by protocol family.

### Documentation Model

Gateway docs should stop describing Claude and Gemini as sidecar compatibility leftovers. They should instead describe them as first-class mirror protocol families under the broader code surface:

- OpenAI/Codex mirror
- Claude mirror
- Gemini mirror

Documentation may still explain that some protocol families translate into shared execution flows internally, but that detail belongs in architecture notes, not in the public contract taxonomy.

## Testing And Acceptance

Phase 1 acceptance needs automated proof, not only documentation updates.

### OpenAPI Baseline Coverage

[`crates/sdkwork-api-interface-http/tests/openapi_route.rs`](D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router/crates/sdkwork-api-interface-http/tests/openapi_route.rs) must be expanded to assert that all real public gateway routes appear in `/openapi.json`, including the route families currently missing from the document.

### Tag Governance

The same test surface must assert:

- `code.openai` exists
- `code.claude` exists
- `code.gemini` exists
- `compatibility` does not exist

### Mirror Path Guardrails

The gateway contract must explicitly reject future wrapper-path drift.

The verification surface should assert that the published OpenAPI document does not introduce custom grouping prefixes such as:

- `/code/*`
- `/claude/*`
- `/gemini/*`
- `/images/openai/*`
- `/video/sora/*`

Grouping belongs in tags, not in public path invention.

### Protocol Regression

Existing regression coverage for Claude and Gemini behavior must continue to pass for:

- official path shape
- official auth inputs
- official streaming behavior where already supported

This includes the current supported auth modes:

- Claude: `Authorization` and `x-api-key`
- Gemini: `Authorization`, `x-goog-api-key`, and `?key=`

### Success Condition

Phase 1 is complete when all of the following are true:

- `/openapi.json` fully covers the real public gateway router
- the public `compatibility` tag is removed
- `code.openai`, `code.claude`, and `code.gemini` are stable public tags
- no custom wrapper prefixes are introduced
- existing OpenAI/Codex, Claude, and Gemini mirror behaviors continue to work through their official path families

## Risks And Mitigations

### Risk: manual `paths(...)` lists drift from the real router

Mitigation:

- expand the gateway OpenAPI regression test into the parity oracle for the full public route surface
- group OpenAPI modules by route-family responsibility so missing entries are easier to spot during review

### Risk: future work reintroduces public wrapper paths for grouping convenience

Mitigation:

- codify the mirror-path rule in this design and in route-level regression tests
- treat any non-official grouping prefix as a contract regression

### Risk: provider-specific media protocols are added without standard-governance rules

Mitigation:

- require future phases to choose between OpenAI standard adoption and provider-specific mirror exposure explicitly
- do not merge provider-specific media routes until their official path, header, query, and body contracts are formally specified

## Future Phases

This design intentionally sets up the later protocol slices without trying to implement them all at once.

### Phase 2: Images

- keep OpenAI `images` as the general standard where it applies
- add provider-specific mirror surfaces only where no OpenAI-equivalent standard exists or where the provider has a clearly valuable official protocol surface

### Phase 3: Video

- mirror official video task and resource protocols directly
- keep long-running job and resource semantics intact instead of flattening them into a fake universal contract

### Phase 4: Music

- mirror official music generation protocols directly where stable
- keep the current resource-oriented internal execution model aligned with the published protocol contract instead of inventing a transport-only façade

## Final Principle

The gateway is a mirror-style API router, not a wrapper-style API product.

That means:

- official protocol first
- OpenAI standard second when it already solves the capability generically
- provider-specific mirror only when no shared standard exists
- no fake standardization through custom public paths

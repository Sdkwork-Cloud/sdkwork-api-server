# Gateway Video Mirror Governance Design

**Date:** 2026-04-16

## Superseded Status

This design predates the later provider-specific video mirror rollout. The current implemented/public contract keeps `video.openai` as the shared OpenAI video family for Sora 2 and Sora 2 Pro, does not publish `video.sora`, and now publishes `video.kling`, `video.aliyun`, `video.google-veo`, `video.minimax`, `video.vidu`, and `video.volcengine` as active provider-specific mirror families.

## Goal

Formalize the gateway `video` public contract as a mirror-style API surface that keeps the existing official `/v1/videos*` family stable while publishing it as a first-class mirror protocol group. This slice governs OpenAPI taxonomy, operation naming, docs, and regression guardrails without inventing provider-specific wrapper paths or new runtime routes.

## Current State

- the live gateway already exposes a shared video surface on `/v1/videos*`
- the OpenAPI document still groups those routes under the generic `videos` tag
- the current docs describe the video route family but do not distinguish the active mirror family from reserved future provider-specific families
- no public provider-specific video mirror routes are currently formalized for `sora`, `minimax`, `vidu`, `volcengine`, `google-veo`, `aliyun`, or `kling`
- the current runtime regression suite already covers `/v1/videos*` behavior, so this slice can focus on public contract governance instead of runtime expansion

## Problem Statement

The gateway needs to keep the public `video` contract precise:

- `/openapi.json` should describe only active, callable public mirror protocols
- public grouping should identify the mirrored protocol family instead of publishing a generic `videos` bucket
- future provider-specific video protocol families need reserved names so later work can expand cleanly without contract drift
- OpenAPI regression coverage must prevent wrapper-path drift such as `/video/openai/*`, `/video/sora/*`, or `/v1/videos/sora/*`

## Options Considered

### Option A: Keep the generic `videos` tag

Pros:

- smallest OpenAPI change
- no immediate taxonomy migration

Cons:

- does not communicate which protocol family is actually being published
- scales poorly when provider-specific video mirrors are introduced
- weakens the gateway's `base_url`-switch-only contract story

### Option B: Publish `video.openai` as the only active video mirror tag and reserve provider-specific future tags

Pros:

- matches the current live `/v1/videos*` router surface
- aligns with the prior baseline taxonomy that reserved `video.openai` and future provider groups
- provides a stable naming pattern for later `video.sora`, `video.minimax`, and other provider-specific mirrors
- is straightforward to enforce with OpenAPI regression tests

Cons:

- requires tag, `operationId`, doc, and test updates
- requires explicit documentation so reserved provider names are not mistaken for active support

### Option C: Publish provider-specific video mirror groups now

Pros:

- advertises future taxonomy immediately

Cons:

- turns `/openapi.json` into a roadmap instead of a truthful public contract
- invites clients to depend on routes that do not exist yet
- increases the risk of inventing wrapper paths to justify the published tags

## Recommendation

Adopt Option B.

The gateway should publish only one active video mirror family in this slice: `video.openai`. Future provider-specific groups stay reserved in design governance and docs until their official protocols and runtime routes are fully specified.

## Design

### External Contract Rules

- preserve the existing official public video paths exactly:
  - `GET /v1/videos`
  - `POST /v1/videos`
  - `GET /v1/videos/{video_id}`
  - `DELETE /v1/videos/{video_id}`
  - `GET /v1/videos/{video_id}/content`
  - `POST /v1/videos/{video_id}/remix`
  - `POST /v1/videos/characters`
  - `GET /v1/videos/characters/{character_id}`
  - `POST /v1/videos/edits`
  - `POST /v1/videos/extensions`
  - `GET /v1/videos/{video_id}/characters`
  - `GET /v1/videos/{video_id}/characters/{character_id}`
  - `POST /v1/videos/{video_id}/characters/{character_id}`
  - `POST /v1/videos/{video_id}/extend`
- do not invent gateway-specific wrapper prefixes for video grouping
- keep provider routing behind the shared video contract whenever the shared `/v1/videos*` protocol is sufficient
- only add a provider-specific public video mirror family when the provider has an official protocol that cannot be losslessly represented by the shared `/v1/videos*` contract

### Phase 3A Scope

Phase 3A includes:

- renaming the active OpenAPI video tag from `videos` to `video.openai`
- stabilizing video `operationId` values under the `video_openai_*` pattern
- updating gateway docs to distinguish the active video mirror family from reserved future provider families
- adding OpenAPI regression guardrails for tag presence, tag absence, `operationId` stability, and wrapper-path absence

Phase 3A does not include:

- adding any provider-specific runtime video routes
- changing request or response semantics for `/v1/videos*`
- exposing reserved provider-specific video tags in `/openapi.json`
- inventing new public paths such as `/video/openai/*`, `/video/sora/*`, or `/v1/videos/sora/*`

### OpenAPI Tag Taxonomy

Active public video tag:

- `video.openai`

Reserved future video tags:

- `video.sora`
- `video.minimax`
- `video.vidu`
- `video.volcengine`
- `video.google-veo`
- `video.aliyun`
- `video.kling`

Reserved tags belong in design and governance docs only until the corresponding mirror routes are formalized. They must not appear in the generated OpenAPI document during Phase 3A.

### Operation Naming

The OpenAPI document should publish stable ASCII-only video `operationId` values:

- `video_openai_list`
- `video_openai_create`
- `video_openai_get`
- `video_openai_delete`
- `video_openai_content_get`
- `video_openai_remix_create`
- `video_openai_characters_create`
- `video_openai_character_canonical_get`
- `video_openai_edits_create`
- `video_openai_extensions_create`
- `video_openai_characters_list`
- `video_openai_character_get`
- `video_openai_character_update`
- `video_openai_extend_create`

These IDs communicate both the capability family and the mirrored protocol without changing the public HTTP path.

### Documentation Model

[`docs/api-reference/gateway-api.md`](D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router/.worktrees/gateway-mirror-protocol-baseline/docs/api-reference/gateway-api.md) and [`docs/reference/api-compatibility.md`](D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router/.worktrees/gateway-mirror-protocol-baseline/docs/reference/api-compatibility.md) should describe:

- `video.openai` as the current active public video mirror family
- provider-specific video families as reserved future mirror groups, not currently implemented public contracts
- the same mirror rule already used for code and images: clients keep the official path and only switch the gateway `base_url`

### Testing And Acceptance

[`crates/sdkwork-api-interface-http/tests/openapi_route.rs`](D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router/.worktrees/gateway-mirror-protocol-baseline/crates/sdkwork-api-interface-http/tests/openapi_route.rs) must assert:

- `video.openai` exists
- `videos` does not exist
- reserved provider tags do not exist in `/openapi.json`
- all currently published `/v1/videos*` operations use `video.openai`
- the published video operations expose the expected `operationId` values
- wrapper paths such as `/video/openai/*`, `/video/sora/*`, `/v1/videos/sora/*`, and `/v1/video/*` do not exist in the published OpenAPI document

Existing runtime regressions in [`crates/sdkwork-api-interface-http/tests/videos_route/mod.rs`](D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router/.worktrees/gateway-mirror-protocol-baseline/crates/sdkwork-api-interface-http/tests/videos_route/mod.rs) continue to prove the live router behavior. Phase 3A does not expand runtime routing scope.

## Risks And Mitigations

### Risk: reserved provider families leak into the public contract early

Mitigation:

- keep reserved names in docs and design docs only
- assert their absence in `/openapi.json`

### Risk: future contributors reintroduce generic `videos` tagging

Mitigation:

- add explicit tag absence assertions for `videos`
- use descriptive `video.openai` tag wording in `gateway_openapi.rs`

### Risk: wrapper paths get added for documentation convenience

Mitigation:

- add negative OpenAPI path assertions for fake grouping prefixes
- keep the design principle explicit in docs and tests

## Future Phases

- provider-specific video mirror protocols can be added later under `video.sora`, `video.minimax`, `video.vidu`, `video.volcengine`, `video.google-veo`, `video.aliyun`, or `video.kling`
- each provider-specific phase must formalize official path shape, auth inputs, request bodies, response bodies, async job semantics, and regression tests before the public tag is published

## Final Principle

For `video`, the gateway remains a mirror-style router:

- current shared `/v1/videos*` contract first
- provider routing hidden behind the shared contract when compatible
- provider-specific mirror protocols only when officially defined and necessary
- no fake standardization through custom public paths or premature OpenAPI tags

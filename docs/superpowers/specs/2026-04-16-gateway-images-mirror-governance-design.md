# Gateway Images Mirror Governance Design

**Date:** 2026-04-16

## Goal

Formalize the gateway `images` public contract as a mirror-style API surface that lets existing OpenAI image clients switch only the `base_url` while preserving the official `/v1/images/*` protocol. This slice governs OpenAPI taxonomy, docs, and regression guardrails without inventing new provider-specific wrapper paths.

## Current State

- the live gateway already exposes the OpenAI image protocol on `/v1/images/generations`, `/v1/images/edits`, and `/v1/images/variations`
- the OpenAPI document still groups those routes under the generic `images` tag instead of a mirror-protocol taxonomy
- the current docs describe the shared image surface but do not distinguish active mirror protocols from reserved future provider-specific families
- no public provider-specific image mirror routes are currently formalized for `nanobanana`, `midjourney`, `volcengine`, `aliyun`, or `kling`

## Problem Statement

The gateway needs to keep the public `images` contract precise:

- `/openapi.json` should describe only active, callable mirror protocols
- public grouping should identify the official protocol being mirrored, not collapse all image capability work into one generic bucket
- provider-specific image protocol families need stable reserved names so future work can expand cleanly without leaking roadmap tags into the published contract
- the OpenAPI regression surface must prevent wrapper-path drift such as `/images/openai/*` or `/v1/images/nanobanana/*`

## Options Considered

### Option A: Keep the generic `images` tag

Pros:

- smallest documentation change
- no immediate OpenAPI taxonomy churn

Cons:

- hides which protocol is actually mirrored
- scales poorly once provider-specific mirror protocols are added
- weakens the `base_url`-switch-only contract story

### Option B: Publish `images.openai` as the only active image mirror tag and reserve future provider tags

Pros:

- exactly matches the current live router
- keeps the active contract limited to official OpenAI image paths
- creates a stable naming pattern for future provider-specific mirror families without publishing fake support
- easy to enforce with OpenAPI regression tests

Cons:

- requires tag, `operationId`, doc, and test updates
- requires explicit wording so reserved provider names are not mistaken for active support

### Option C: Publish provider-specific image groups now even before their mirror routes exist

Pros:

- advertises a larger future taxonomy immediately

Cons:

- turns `/openapi.json` into a roadmap instead of a truthful public contract
- invites clients to rely on routes that do not yet exist
- increases pressure to invent wrapper paths to justify the published tags

## Recommendation

Adopt Option B.

The gateway should publish only one active image mirror family in this slice: `images.openai`. Future provider-specific groups stay reserved in docs and design governance until their official protocol surfaces are fully specified and implemented.

## Design

### External Contract Rules

- preserve the official OpenAI image protocol paths exactly:
  - `/v1/images/generations`
  - `/v1/images/edits`
  - `/v1/images/variations`
- do not invent gateway-specific wrapper prefixes for image grouping
- if a provider can be served behind the OpenAI image contract, keep that provider behind `images.openai`
- only add a provider-specific public image mirror family when the provider has an official protocol that cannot be losslessly represented by the OpenAI image contract

### Phase 2A Scope

Phase 2A includes:

- renaming the active OpenAPI image tag from `images` to `images.openai`
- stabilizing image `operationId` values under the `images_openai_*` pattern
- updating gateway docs to distinguish active image mirror families from reserved future families
- adding OpenAPI regression guardrails for tag presence, tag absence, and wrapper-path absence

Phase 2A does not include:

- adding any provider-specific runtime image routes
- changing request or response semantics for `/v1/images/*`
- exposing reserved provider tags in `/openapi.json`
- inventing new public paths such as `/images/openai/*` or `/v1/images/nanobanana/*`

### OpenAPI Tag Taxonomy

Active public image tag:

- `images.openai`

Reserved future image tags:

- `images.nanobanana`
- `images.midjourney`
- `images.volcengine`
- `images.aliyun`
- `images.kling`

Reserved tags belong in design and governance docs only until the corresponding mirror routes are formalized. They must not appear in the generated OpenAPI document during Phase 2A.

### Operation Naming

The OpenAPI document should publish stable ASCII-only image `operationId` values:

- `images_openai_generations_create`
- `images_openai_edits_create`
- `images_openai_variations_create`

These IDs communicate both the capability family and the mirrored protocol without changing the public HTTP path.

### Documentation Model

[`docs/api-reference/gateway-api.md`](D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router/.worktrees/gateway-mirror-protocol-baseline/docs/api-reference/gateway-api.md) and [`docs/reference/api-compatibility.md`](D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router/.worktrees/gateway-mirror-protocol-baseline/docs/reference/api-compatibility.md) should describe:

- `images.openai` as the current active public image mirror family
- provider-specific image families as reserved future mirror groups, not currently implemented public contracts
- the same mirror rule already used for code protocols: clients keep the official path and only switch the gateway `base_url`

### Testing And Acceptance

[`crates/sdkwork-api-interface-http/tests/openapi_route.rs`](D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router/.worktrees/gateway-mirror-protocol-baseline/crates/sdkwork-api-interface-http/tests/openapi_route.rs) must assert:

- `images.openai` exists
- `images` does not exist
- reserved provider tags do not exist in `/openapi.json`
- `/v1/images/generations`, `/v1/images/edits`, and `/v1/images/variations` use `images.openai`
- the three image operations expose the expected `operationId` values
- wrapper paths such as `/images/openai/*`, `/images/nanobanana/*`, and `/v1/images/nanobanana/*` do not exist in the published OpenAPI document

Existing runtime regressions in [`crates/sdkwork-api-interface-http/tests/images_route.rs`](D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router/.worktrees/gateway-mirror-protocol-baseline/crates/sdkwork-api-interface-http/tests/images_route.rs) continue to prove the live router behavior. Phase 2A does not expand runtime routing scope.

## Risks And Mitigations

### Risk: reserved provider families leak into the public contract early

Mitigation:

- keep reserved names in docs and design docs only
- assert their absence in `/openapi.json`

### Risk: future contributors reintroduce generic `images` tagging

Mitigation:

- add explicit tag absence assertions for `images`
- use descriptive `images.openai` tag wording in `gateway_openapi.rs`

### Risk: wrapper paths get added for documentation convenience

Mitigation:

- add negative OpenAPI path assertions for fake grouping prefixes
- keep the design principle explicit in docs and tests

## Future Phases

- provider-specific image mirror protocols can be added later under `images.nanobanana`, `images.midjourney`, `images.volcengine`, `images.aliyun`, or `images.kling`
- each provider-specific phase must formalize official path shape, auth inputs, request bodies, response bodies, async job semantics, and regression tests before the public tag is published

## Final Principle

For `images`, the gateway remains a mirror-style router:

- official OpenAI image protocol first
- provider routing hidden behind the shared contract when compatible
- provider-specific mirror protocols only when officially defined and necessary
- no fake standardization through custom public paths or premature OpenAPI tags

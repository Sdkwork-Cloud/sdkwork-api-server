# Gateway Audio And Music Mirror Governance Design

**Date:** 2026-04-16

## Superseded Status

This design predates the later provider-specific music mirror rollout. The current implemented/public contract now publishes `music.suno`, `music.google`, and `music.minimax` as active provider-specific mirror families, and the Chinese public docs have already been brought in line with the English contract text.

## Goal

Formalize the gateway `audio` and `music` public contracts as mirror-style API surfaces that let existing clients switch only the `base_url` while preserving the current official-style `/v1/audio/*` and `/v1/music*` paths. This slice governs OpenAPI taxonomy, operation naming, docs, and regression guardrails without inventing provider-specific wrapper paths or publishing roadmap-only mirror families.

## Current State

- the live gateway already exposes shared audio routes on `/v1/audio/transcriptions`, `/v1/audio/translations`, `/v1/audio/speech`, `/v1/audio/voices`, and `/v1/audio/voice_consents`
- the live gateway already exposes shared music routes on `/v1/music`, `/v1/music/{music_id}`, `/v1/music/{music_id}/content`, and `/v1/music/lyrics`
- the generated OpenAPI document still publishes these routes under generic `audio` and `music` tags instead of mirror-protocol taxonomy
- the audio and music operations do not yet use stable `operationId` values
- the English docs still describe `audio` and `music` generically instead of distinguishing active mirror families from reserved future groups
- the Chinese docs are behind the current English governance language and still need a later sync pass

## Problem Statement

The gateway needs to keep the public media contract truthful:

- `/openapi.json` should publish only active, callable mirror families
- public grouping should identify the mirrored protocol family instead of collapsing everything into generic capability buckets
- future provider-specific music protocol families need reserved names so later work can expand cleanly without contract drift
- the OpenAPI regression surface must prevent wrapper-path drift such as `/audio/openai/*`, `/music/openai/*`, `/music/suno/*`, or `/v1/music/suno/*`

## Options Considered

### Option A: Keep the generic `audio` and `music` tags

Pros:

- smallest OpenAPI change
- no immediate taxonomy churn

Cons:

- hides which protocol family is actually being published
- scales poorly once provider-specific music mirrors are added
- weakens the gateway `base_url`-switch-only contract story

### Option B: Publish `audio.openai` and `music.openai` as the active shared mirror families and reserve future provider-specific music tags

Pros:

- matches the current live router surface
- keeps the active contract limited to routes that already exist
- creates a stable naming pattern for future provider-specific music mirrors without publishing fake support
- is straightforward to enforce with OpenAPI regression tests

Cons:

- requires tag, `operationId`, doc, and test updates
- requires explicit wording so reserved provider names are not mistaken for active support

### Option C: Publish provider-specific music groups now

Pros:

- advertises a larger taxonomy immediately

Cons:

- turns `/openapi.json` into a roadmap instead of a truthful public contract
- invites clients to depend on routes that do not exist yet
- increases pressure to invent wrapper paths to justify the published tags

## Recommendation

Adopt Option B.

The gateway should publish two active shared media mirror families in this slice:

- `audio.openai`
- `music.openai`

Future provider-specific music groups stay reserved in governance and docs until their official protocols and runtime routes are fully specified.

## Design

### External Contract Rules

- preserve the existing public audio paths exactly:
  - `POST /v1/audio/transcriptions`
  - `POST /v1/audio/translations`
  - `POST /v1/audio/speech`
  - `GET /v1/audio/voices`
  - `POST /v1/audio/voice_consents`
- preserve the existing public music paths exactly:
  - `GET /v1/music`
  - `POST /v1/music`
  - `GET /v1/music/{music_id}`
  - `DELETE /v1/music/{music_id}`
  - `GET /v1/music/{music_id}/content`
  - `POST /v1/music/lyrics`
- do not invent gateway-specific wrapper prefixes for grouping convenience
- if provider routing can stay behind the shared audio or music contract, keep it there
- only add provider-specific public music mirror families when the provider has an official protocol that cannot be losslessly represented by the shared contract

### Active OpenAPI Tag Taxonomy

Active public tags in this slice:

- `audio.openai`
- `music.openai`

Reserved future music tags:

- `music.suno`
- `music.google`
- `music.minimax`

Reserved tags belong in governance docs only until the corresponding mirror routes are formalized. They must not appear in the generated OpenAPI document during this slice.

### Operation Naming

The OpenAPI document should publish stable ASCII-only `operationId` values.

Audio:

- `audio_openai_transcriptions_create`
- `audio_openai_translations_create`
- `audio_openai_speech_create`
- `audio_openai_voices_list`
- `audio_openai_voice_consents_create`

Music:

- `music_openai_list`
- `music_openai_create`
- `music_openai_get`
- `music_openai_delete`
- `music_openai_content_get`
- `music_openai_lyrics_create`

These IDs communicate both the capability family and the mirrored protocol without changing the public HTTP path.

### Documentation Model

`docs/api-reference/gateway-api.md` and `docs/reference/api-compatibility.md` should describe:

- `audio.openai` as the current active shared audio mirror family
- `music.openai` as the current active shared music mirror family
- provider-specific music families as reserved future groups, not currently implemented public contracts
- the same mirror rule already used for code, images, and video: clients keep the official path and only switch the gateway `base_url`

This slice does not try to solve the Chinese docs drift. That remains a follow-up after the English public contract text is fully stabilized.

### Testing And Acceptance

`crates/sdkwork-api-interface-http/tests/openapi_route.rs` must assert:

- `audio.openai` exists
- `music.openai` exists
- generic `audio` and `music` tags do not exist
- reserved music tags do not exist in `/openapi.json`
- all currently published `/v1/audio/*` operations use `audio.openai`
- all currently published `/v1/music*` operations use `music.openai`
- the published audio and music operations expose the expected `operationId` values
- wrapper paths such as `/audio/openai/*`, `/music/openai/*`, `/music/suno/*`, `/v1/music/suno/*`, and `/v1/audio/openai/*` do not exist in the published OpenAPI document

Existing runtime regressions in `crates/sdkwork-api-interface-http/tests/music_route.rs` and other route-level suites continue to prove the live router behavior. This slice does not expand runtime routing scope.

## Risks And Mitigations

### Risk: reserved future music families leak into the public contract early

Mitigation:

- keep reserved names in docs and design docs only
- assert their absence in `/openapi.json`

### Risk: future contributors reintroduce generic `audio` or `music` tagging

Mitigation:

- add explicit tag absence assertions for `audio` and `music`
- use descriptive `audio.openai` and `music.openai` wording in `gateway_openapi.rs`

### Risk: wrapper paths get added for taxonomy convenience

Mitigation:

- add negative OpenAPI path assertions for fake grouping prefixes
- keep the mirror-path rule explicit in docs and tests

## Non-Goals

- no new runtime provider-specific audio or music routes
- no public `music.suno`, `music.google`, or `music.minimax` routes in this slice
- no changes to request or response semantics for `/v1/audio/*` or `/v1/music*`
- no changes to routing, billing, or provider selection semantics beyond public contract governance

## Final Principle

For `audio` and `music`, the gateway remains a mirror-style router:

- current shared public contract first
- provider routing hidden behind the shared contract when compatible
- provider-specific mirror protocols only when officially defined and necessary
- no fake standardization through custom public paths or premature OpenAPI tags

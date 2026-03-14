# OpenAI Surface Parity Refresh Design

**Date:** 2026-03-14

**Status:** Approved by the user's standing instruction to continue autonomously without pausing for approval checkpoints

## Goal

Bring the SDKWork gateway closer to current OpenAI API surface parity by closing the highest-confidence, highest-value gaps confirmed through the official API reference and official OpenAI SDK resource surface.

## Audit Basis

This design is based on two primary sources:

1. the current OpenAI API reference navigation and endpoint pages on `platform.openai.com` and `developers.openai.com`
2. the current generated resource surface in the official OpenAI Python SDK and Node SDK repositories

Using both sources matters because some route families now move quickly. The public docs and SDK are mostly aligned, but there are a few areas where one source leads the other.

## Current Findings

The current SDKWork gateway already covers most public data-plane families, but the parity audit exposed four concrete issues.

### 1. Containers are entirely missing

The official surface now includes:

- `POST /v1/containers`
- `GET /v1/containers`
- `GET /v1/containers/{container_id}`
- `DELETE /v1/containers/{container_id}`
- `POST /v1/containers/{container_id}/files`
- `GET /v1/containers/{container_id}/files`
- `GET /v1/containers/{container_id}/files/{file_id}`
- `DELETE /v1/containers/{container_id}/files/{file_id}`
- `GET /v1/containers/{container_id}/files/{file_id}/content`

This is the largest remaining confirmed missing family.

### 2. Evals are not fully implemented

The gateway currently supports eval create, list, retrieve, update, delete, run list, run create, and run retrieve, but the official surface also includes:

- `DELETE /v1/evals/{eval_id}/runs/{run_id}`
- `POST /v1/evals/{eval_id}/runs/{run_id}/cancel`
- `GET /v1/evals/{eval_id}/runs/{run_id}/output_items`
- `GET /v1/evals/{eval_id}/runs/{run_id}/output_items/{output_item_id}`

### 3. Fine-tuning is not fully implemented

The gateway currently supports job create, list, retrieve, cancel, events, and checkpoints, but the official surface also includes:

- `POST /v1/fine_tuning/jobs/{fine_tuning_job_id}/pause`
- `POST /v1/fine_tuning/jobs/{fine_tuning_job_id}/resume`
- `POST /v1/fine_tuning/checkpoints/{fine_tuned_model_checkpoint}/permissions`
- `GET /v1/fine_tuning/checkpoints/{fine_tuned_model_checkpoint}/permissions`
- `GET /v1/fine_tuning/checkpoints/{fine_tuned_model_checkpoint}/permissions` as the older retrieve form the SDK still exposes
- `DELETE /v1/fine_tuning/checkpoints/{fine_tuned_model_checkpoint}/permissions/{permission_id}`

These checkpoint permission routes are admin-key dependent upstream, but they are still part of the official OpenAI surface and fit the gateway relay architecture.

### 4. Videos currently use an outdated route shape

The local gateway currently exposes nested video-character and extend routes:

- `/v1/videos/{video_id}/characters`
- `/v1/videos/{video_id}/characters/{character_id}`
- `/v1/videos/{video_id}/extend`

The current official SDK surface instead exposes:

- `POST /v1/videos/characters`
- `GET /v1/videos/characters/{character_id}`
- `POST /v1/videos/edits`
- `POST /v1/videos/extensions`
- `POST /v1/videos/{video_id}/remix`
- `GET /v1/videos/{video_id}/content`

That means current SDKWork video support is partially real but no longer aligned with the official route topology.

## Source Inconsistencies

The audit also found two areas where the public sources are not fully aligned.

### Audio voices and voice consents

The public API reference navigation currently lists voices and voice consent resources, but the current generated official SDK surface does not expose them. SDKWork already implements partial support for:

- `GET /v1/audio/voices`
- `POST /v1/audio/voice_consents`

The safest architectural choice is to avoid guessing the remaining route set until the official spec and SDK converge. This batch should not expand audio farther based only on partial navigation hints.

### Skills and graders

The official SDK repositories currently contain `skills` and `graders` resources, but these are not clearly surfaced in the public API reference used by the project as its compatibility target. They should remain out of scope for this batch.

## Options Considered

### Option A: Implement only containers

Pros:

- smallest safe batch
- closes the single largest confirmed missing family

Cons:

- leaves obvious parity gaps in evals and fine-tuning
- leaves videos aligned to an outdated route shape

### Option B: Implement containers plus the remaining confirmed subroutes and route-shape corrections

Pros:

- closes the highest-confidence gaps in one coherent parity batch
- fixes both missing features and incorrect route topology
- keeps the provider contract explicit and typed
- extends the plugin runtime vocabulary instead of bypassing it

Cons:

- touches several crates
- requires compatibility aliases for old local video routes to avoid regressions

### Option C: Add a generic unknown-route relay path

Pros:

- could reduce future route churn

Cons:

- weakens the contract model
- undermines extension operation stability
- makes stateful fallback and plugin capability reasoning ambiguous

## Recommendation

Use **Option B**.

The gateway already has the correct architecture. The right move is to extend the typed provider contract with the next confirmed families and update the route surface where the current local shape has drifted behind the official one.

## Scope

This batch should add first-class support for:

- the full `containers` family
- container file binary content relay
- eval run delete and cancel
- eval run output item list and retrieve
- fine-tuning job pause and resume
- fine-tuning checkpoint permission create, list, retrieve-compatible list, and delete
- official video create-character, get-character, edit, and extension routes

This batch should also add compatibility aliases for the older SDKWork-only video routes so the project does not regress existing clients immediately:

- `/v1/videos/{video_id}/characters`
- `/v1/videos/{video_id}/characters/{character_id}`
- `/v1/videos/{video_id}/extend`

Those aliases should be documented as compatibility shims, not as the canonical OpenAI route surface.

## Architecture

The implementation should preserve the current layering:

- `sdkwork-api-contract-openai`
  gains additive request and response structures for containers, eval run subresources, fine-tuning control subresources, and updated video requests
- `sdkwork-api-provider-core`
  gains additive `ProviderRequest` variants only
- `sdkwork-api-provider-openai`
  maps the new variants to the current official upstream paths
- `sdkwork-api-extension-host`
  maps the same variants to stable plugin operation names
- `sdkwork-api-app-gateway`
  exposes relay helpers and local compatible fallback behavior
- `sdkwork-api-interface-http`
  wires Axum routes for stateless and stateful execution modes

No generic passthrough route should be added.

## Extension Standard Impact

The extension vocabulary must stay explicit and stable across builtin, connector, and native-dynamic runtimes.

New operations should extend the existing naming convention:

- `containers.create`
- `containers.list`
- `containers.retrieve`
- `containers.delete`
- `containers.files.create`
- `containers.files.list`
- `containers.files.retrieve`
- `containers.files.delete`
- `containers.files.content.retrieve`
- `evals.runs.delete`
- `evals.runs.cancel`
- `evals.runs.output_items.list`
- `evals.runs.output_items.retrieve`
- `fine_tuning.jobs.pause`
- `fine_tuning.jobs.resume`
- `fine_tuning.checkpoints.permissions.create`
- `fine_tuning.checkpoints.permissions.list`
- `fine_tuning.checkpoints.permissions.retrieve`
- `fine_tuning.checkpoints.permissions.delete`
- `videos.characters.create`
- `videos.characters.retrieve`
- `videos.edits.create`
- `videos.extensions.create`

This keeps provider plugins highly pluggable without hiding their runtime behavior behind opaque transport-specific contracts.

## Route Compatibility Strategy

For videos, the gateway should expose both:

1. the current official OpenAI routes as the primary canonical surface
2. the older nested SDKWork routes as compatibility aliases that map onto the same provider requests or local fallbacks

That approach improves official parity without breaking already written integration tests or early consumers abruptly.

## Local Fallback Semantics

The new routes should follow the existing conservative semantics:

1. stateless mode relays to its configured upstream when one is available
2. missing stateless upstream configuration falls back to local compatible responses
3. upstream execution failures still surface as `502 Bad Gateway`
4. stateful mode prefers resolved provider relay through catalog, credential, and routing state
5. unresolved stateful routing may still fall back locally where the gateway already uses compatible emulation

Binary routes such as container file content must continue to use the provider stream abstraction rather than forcing JSON buffering.

## Testing Strategy

The batch should remain strict TDD:

- add failing HTTP route tests first
- verify failure before production edits
- implement the minimum contract and provider request additions
- wire provider mapping and extension operation mapping
- wire gateway helpers and Axum routes
- run focused tests, then full workspace verification

The tests should cover:

- official container routes in stateless, stateful, and local-fallback modes
- official eval run delete, cancel, and output-item routes
- official fine-tuning pause, resume, and checkpoint permission routes
- official video routes plus legacy alias preservation
- binary stream passthrough for container file content

## Out of Scope

This batch does not:

- add speculative audio routes beyond the already implemented voices and voice consent subset
- implement `skills` or `graders`
- add OpenAI organization administration APIs beyond fine-tuning checkpoint permissions already in the public fine-tuning surface
- redesign routing, quota, or tenancy behavior
- introduce a generic opaque passthrough proxy

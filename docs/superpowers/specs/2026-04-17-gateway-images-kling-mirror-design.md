# Gateway Images Kling Mirror Design

**Goal:** Publish the first provider-specific image mirror family as `images.kling` on Kling's official HTTP paths while preserving the existing shared `images.openai` contract.

## Scope

- Add the official Kling image generation submit route:
  - `POST /api/v1/services/aigc/image-generation/generation`
- Add the official Kling task query route:
  - `GET /api/v1/tasks/{task_id}`
- Keep the existing shared OpenAI image routes unchanged:
  - `/v1/images/generations`
  - `/v1/images/edits`
  - `/v1/images/variations`

## Contract Rules

- Public HTTP paths must stay identical to the official Kling client paths.
- The gateway must remain usable by changing only the `base_url`.
- The submit route must preserve provider-specific headers such as `X-DashScope-Async`.
- Provider selection for the stateful gateway must be constrained by `mirror_protocol_identity == "kling"`.

## Routing Design

- Reuse the generic JSON provider mirror relay introduced for `music.suno`.
- Add stateless handlers that require `upstream.mirror_protocol_identity() == "kling"`.
- Add stateful handlers that resolve providers through the identity-constrained planned execution helper.
- Use internal route keys:
  - `images.kling.generation`
  - `provider.kling.tasks.get`

The task query route key is intentionally provider-generic instead of image-specific so future Kling task-query reuse is not blocked by this slice.

## Usage and Billing

- Persist routing decision logs for both submit and task-query requests.
- Do not record usage or billing facts in this slice.

This keeps the mirror truthful without inventing premature billing semantics for async provider-specific image jobs.

## OpenAPI

- Add tag `images.kling`.
- Publish the two official paths above.
- Keep wrapper paths absent:
  - `/images/kling/*`
  - `/v1/images/kling/*`
  - `/api/v1/images/kling/*`

## Verification

- `cargo test -p sdkwork-api-interface-http --test openapi_route openapi_routes_expose_gateway_api_inventory -- --nocapture`
- `cargo test -p sdkwork-api-interface-http --test images_route stateless_images_kling_routes_relay_to_official_paths -- --nocapture`
- `cargo test -p sdkwork-api-interface-http --test images_route stateful_images_kling_routes_use_kling_provider_identity -- --nocapture`

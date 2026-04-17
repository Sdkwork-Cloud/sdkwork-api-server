# Gateway Music Suno Mirror Design

## Goal

Publish the first provider-specific public mirror family under `music.suno` using Suno's official HTTP protocol and official paths, while keeping the existing shared `music.openai` contract on `/v1/music*`.

This slice must satisfy the mirror-router rule: clients using Suno's official API should only need to switch the base URL to the gateway and keep the same method, path, query, body, and bearer-auth contract.

## Scope

This slice publishes four public Suno mirror endpoints:

- `POST /api/v1/generate`
- `GET /api/v1/generate/record-info?taskId=...`
- `POST /api/v1/lyrics`
- `GET /api/v1/lyrics/record-info?taskId=...`

The slice includes:

- gateway OpenAPI tags and paths under `music.suno`
- stateless direct relay using configured upstream `base_url + api_key`
- stateful direct relay using provider/account resolution plus `mirror_protocol_identity == "suno"` enforcement
- gateway reference docs updates

The slice does not add wrapper paths such as `/music/suno/*`, `/v1/music/suno/*`, or `/music/providers/suno/*`.

## Protocol Source

Grounding comes from Suno's official API reference on `docs.sunoapi.org`:

- Generate music: `POST /api/v1/generate`
- Get music generation details: `GET /api/v1/generate/record-info` with `taskId`
- Generate lyrics: `POST /api/v1/lyrics`
- Get lyrics generation details: `GET /api/v1/lyrics/record-info` with `taskId`

The gateway should treat request and response payloads as provider-native JSON. This slice mirrors transport semantics rather than re-modeling Suno's payload schema into shared internal DTOs.

## Design

### Public Contract

`music.suno` is a provider-specific top-level family because OpenAI does not define an equivalent official API shape for Suno's task-oriented music protocol.

OpenAPI should publish the official Suno paths exactly as-is and tag them as `music.suno`. Request and response schemas can remain `serde_json::Value` because the compatibility target is the provider-native JSON payload, not a gateway-invented abstraction.

### Relay Transport

The implementation should use direct HTTP relay, not the existing raw plugin execution path and not the shared `ProviderRequest::Music*` contract.

Reason:

- the shared music contract is OpenAI-shaped and would distort Suno's official protocol
- raw plugin execution currently depends on native-dynamic runtime support and does not satisfy the "switch base URL only" requirement as directly as plain HTTP relay
- the compatibility layer already proves the correct pattern: preserve official path, official query string, provider-native body, and provider-native error body

The relay should:

- forward the original request method
- forward the original path and query string unchanged
- pass through request body bytes unchanged
- overwrite outbound auth with `Authorization: Bearer <provider_api_key>`
- pass through provider response status, content type, and body unchanged

### Stateless Routing

Stateless Suno routes should only relay when the configured upstream has `mirror_protocol_identity == "suno"`. If the stateless upstream is configured for another mirror identity, the Suno routes should fail closed rather than incorrectly reusing another provider contract.

### Stateful Routing

Stateful Suno routes cannot rely only on the existing `capability + route_key` selection because a non-model route with no matching model catalog entry falls back to the entire provider set. That is unsafe for provider-specific mirror families.

Stateful Suno routing therefore needs an identity-constrained planned execution helper:

1. simulate/select a routing decision for a provider-specific route key
2. scan the ordered candidate providers in routing order
3. keep only candidates whose `mirror_protocol_identity == "suno"`
4. resolve execution credentials/bindings for the first valid Suno candidate
5. build a planned execution context from that candidate and persist the routing decision log against the provider-specific route key

If no Suno-capable provider is available, return an upstream/gateway error rather than relaying to a mismatched provider.

This keeps routing profile ordering, policy ordering, health ordering, and failover semantics intact while adding the missing protocol-family safety check.

### Route Keys

Use provider-specific route keys so routing logs and billing evidence distinguish this family from the shared `/v1/music*` contract:

- `music.suno.generate`
- `music.suno.generate.record-info`
- `music.suno.lyrics`
- `music.suno.lyrics.record-info`

Capability remains `music`.

### Billing and Usage

This slice should keep usage recording lightweight:

- create and lyrics endpoints record gateway usage against the provider-specific route keys
- detail endpoints also record usage against their provider-specific route keys
- no attempt is made in this slice to infer Suno-specific duration or task billing from opaque provider payloads

That keeps the transport mirror accurate without inventing unsupported metering assumptions.

## Risks

### Risk: provider mismatch in stateful mode

If stateful selection is not constrained by mirror identity, a generic music-capable provider could be chosen for Suno paths.

Mitigation:

- add identity-constrained planned execution helper
- add stateful test proving a non-Suno provider is ignored when a Suno provider is present

### Risk: wrapper-path drift

Future edits may accidentally add `/music/suno/*` convenience routes.

Mitigation:

- extend OpenAPI regression tests to assert official Suno paths exist
- assert wrapper paths remain absent

### Risk: direct relay becomes Suno-specific glue

If the relay is implemented with Suno-only assumptions, future mirrors like `images.kling` or `video.google-veo` will require another rewrite.

Mitigation:

- implement generic provider-official mirror HTTP relay helpers that accept method, path/query, body, and auth policy
- keep Suno as the first consumer of that generic helper

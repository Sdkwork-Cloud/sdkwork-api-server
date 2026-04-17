# Gateway Music MiniMax Mirror Design

## Goal

Publish the next provider-specific public mirror family under `music.minimax` using MiniMax's official HTTP protocol and official paths, while keeping the existing shared `music.openai` contract on `/v1/music*` and the existing `music.suno` contract unchanged.

This slice must satisfy the mirror-router rule: clients using MiniMax's official music APIs should only need to switch the base URL to the gateway and keep the same method, path, query, body, and bearer-auth contract.

## Scope

This slice publishes two public MiniMax mirror endpoints:

- `POST /v1/music_generation`
- `POST /v1/lyrics_generation`

The slice includes:

- gateway OpenAPI tags and paths under `music.minimax`
- stateless direct relay using configured upstream `base_url + api_key`
- stateful direct relay using provider/account resolution plus `mirror_protocol_identity == "minimax"` enforcement
- gateway reference docs updates
- usage recording that preserves music duration when MiniMax returns it on music generation responses

The slice does not add wrapper paths such as `/music/minimax/*`, `/v1/music/minimax/*`, or `/music/providers/minimax/*`.

## Protocol Source

Grounding comes from MiniMax's official API reference on `platform.minimax.io`:

- Music generation: `POST /v1/music_generation`
- Lyrics generation: `POST /v1/lyrics_generation`

The gateway should treat request and response payloads as provider-native JSON. This slice mirrors transport semantics rather than re-modeling MiniMax's payload schema into shared internal DTOs.

## Design

### Public Contract

`music.minimax` is a provider-specific top-level family because OpenAI does not define an equivalent official API shape for MiniMax's music-generation and lyrics-generation protocol.

OpenAPI should publish the official MiniMax paths exactly as-is and tag them as `music.minimax`. Request and response schemas can remain `serde_json::Value` because the compatibility target is the provider-native JSON payload, not a gateway-invented abstraction.

### Relay Transport

The implementation should reuse the generic provider-official JSON relay helper introduced for `music.suno` and reused by `images.kling`.

The relay should:

- forward the original request method
- forward the original path unchanged
- pass through request body bytes unchanged
- overwrite outbound auth with `Authorization: Bearer <provider_api_key>`
- pass through provider response status, content type, and body unchanged

### Stateless Routing

Stateless MiniMax routes should only relay when the configured upstream has `mirror_protocol_identity == "minimax"`. If the stateless upstream is configured for another mirror identity, the MiniMax routes should fail closed rather than incorrectly reusing another provider contract.

### Stateful Routing

Stateful MiniMax routes should use the existing identity-constrained planned execution helper so a generic music-capable provider cannot be selected for MiniMax paths.

Use provider-specific route keys:

- `music.minimax.generate`
- `music.minimax.lyrics`

Capability remains `music`.

### Billing and Usage

MiniMax music generation responses include provider-native metadata that can support more truthful usage recording than the first Suno slice.

This slice should:

- persist routing decision logs for both MiniMax endpoints
- record gateway usage for `music.minimax.generate`
- when the response includes `extra_info.music_duration`, convert milliseconds to seconds and persist that as `music_seconds`
- use the provider response `trace_id` as the reference ID when available
- record lightweight gateway usage for `music.minimax.lyrics` without inventing music duration or token counts when the response does not provide them

This keeps the transport mirror accurate while improving billing truth for the generation path.

## Risks

### Risk: provider mismatch in stateful mode

If stateful selection is not constrained by mirror identity, a generic music-capable provider could be chosen for MiniMax paths.

Mitigation:

- reuse the existing identity-constrained planned execution helper
- add stateful tests proving a non-MiniMax provider is ignored when a MiniMax provider is present

### Risk: wrapper-path drift

Future edits may accidentally add `/music/minimax/*` convenience routes.

Mitigation:

- extend OpenAPI regression tests to assert official MiniMax paths exist
- assert wrapper paths remain absent

### Risk: incorrect billing assumptions

If the gateway guesses duration or usage IDs from arbitrary fields, billing evidence drifts from the provider contract.

Mitigation:

- only derive `music_seconds` from the official MiniMax response field `extra_info.music_duration`
- only use `trace_id` as a reference ID when present
- keep lyrics usage lightweight instead of inferring missing duration or token metrics

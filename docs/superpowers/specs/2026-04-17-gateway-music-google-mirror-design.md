# Gateway Music Google Mirror Design

## Goal

Publish `music.google` as a provider-specific public mirror family on Google Vertex AI's official music-generation path while keeping the shared `music.openai` contract on `/v1/music*`.

This slice must preserve mirror-router semantics: a Google Vertex AI music client should only need to switch the base URL to the gateway and keep the same method, path, body, and bearer-auth behavior.

## Scope

This slice publishes one provider-specific Google music endpoint:

- `POST /v1/projects/{project}/locations/{location}/publishers/google/models/{model}:predict`

The slice includes:

- gateway OpenAPI tag and path under `music.google`
- stateless direct relay using configured upstream `base_url + api_key`
- stateful direct relay using model binding plus `mirror_protocol_identity == "google"` enforcement
- docs updates for the active `music.google` family
- a unified Google Vertex models dispatcher so Veo and Google music can coexist on the same wildcard router path

The slice does not add wrapper paths such as `/music/google/*`, `/v1/music/google/*`, or `/google/music/*`.

## Protocol Source

Grounding comes from Google Cloud Vertex AI's official Lyria music-generation REST reference. The official prediction path is:

- `POST /v1/projects/{project}/locations/{location}/publishers/google/models/{model}:predict`

The gateway should treat both request and response payloads as provider-native JSON. This slice mirrors Google transport semantics rather than redefining Lyria requests through a gateway-owned music DTO.

## Design

### Public Contract

`music.google` is a provider-specific top-level family because OpenAI does not define Google's Vertex AI prediction transport.

OpenAPI should publish the official Vertex AI predict path exactly as-is and tag it as `music.google`. Request and response schemas can remain `serde_json::Value` because compatibility depends on path and payload passthrough, not on a gateway-owned schema projection.

### Shared Google Router Entry

Google Veo already uses the official Vertex AI `publishers/google/models/*` path family. Music and video therefore share the same router entry in Axum even though they belong to different top-level API groups.

The router should keep one wildcard Google Vertex path:

- `/v1/projects/{project}/locations/{location}/publishers/google/models/{*tail}`

The dispatcher should parse the tail and route:

- `:predictLongRunning` and `:fetchPredictOperation` to the existing Veo flow
- `:predict` to the new Google music flow

This keeps the live router truthful while letting OpenAPI publish modality-specific path declarations.

### Stateless Routing

Stateless Google music routes should relay only when the configured upstream has `mirror_protocol_identity == "google"`. If a different stateless upstream is configured, the request should fail closed instead of reusing another protocol family.

### Stateful Routing

Stateful Google music routing should resolve a provider by:

1. extracting `{model}` from `{model}:predict`
2. selecting a planned execution context for capability `music`
3. constraining candidate providers to `mirror_protocol_identity == "google"`
4. using the bound model name to keep provider selection honest

This preserves the current routing, credential, billing, and decision-log pipeline while preventing protocol-family drift.

### Route Key and Usage

Use a provider-specific route key:

- `music.google.predict`

Successful relays should record gateway usage and billing evidence against that route key. This slice keeps billing lightweight and request-oriented rather than inferring generated duration from opaque Google payloads.

## Risks

### Risk: Google music and Veo diverge inside one wildcard path

If the shared dispatcher is not explicit about supported actions, `:predict` could be misrouted as Veo or rejected incorrectly.

Mitigation:

- parse the tail into explicit Google actions
- keep unsupported actions returning deterministic invalid-request responses
- extend tests for both Veo and Google music after the dispatcher change

### Risk: provider mismatch in stateful mode

If stateful routing ignores mirror identity, a generic provider could be selected for Google music paths.

Mitigation:

- use the existing mirror-identity-constrained planned execution helper
- add a stateful regression proving the generic provider is ignored when a Google mirror provider exists

### Risk: wrapper-path drift

Future edits may accidentally publish convenience routes like `/music/google/predict`.

Mitigation:

- extend OpenAPI regression tests to assert the official Google path exists
- assert wrapper paths remain absent

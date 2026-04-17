## Goal

Publish `video.minimax` on MiniMax's official `/v1/video_generation`, `/v1/query/video_generation`, and `/v1/files/retrieve` paths with direct HTTP relay and stateful provider identity enforcement.

## Architecture

Reuse the existing provider mirror JSON relay and the existing stateful mirror-identity-constrained routing helper. Keep the shared `/v1/videos*` contract unchanged while adding a separate MiniMax mirror family with official unique paths.

## Test-First Steps

1. Add OpenAPI guardrails.
   - `video.minimax` tag exists.
   - the three official paths exist.
   - each official path is tagged `video.minimax`.
   - wrapper paths remain absent.

2. Add route tests.
   - stateless mode relays only to an upstream whose identity is `minimax`.
   - stateful mode routes to a `minimax` provider instead of a generic OpenAI-compatible provider.
   - usage records, billing events, and decision logs are written for the three route keys.

Expected red state: tests fail because `video.minimax` paths and routes are not yet published.

## Implementation Steps

1. Add `video_minimax` stateless and stateful handler modules.
2. Register the three official routes in stateless and stateful inference/storage route groups.
3. Add a `gateway_openapi_paths_video_minimax.rs` module and register it from `gateway_openapi.rs`.
4. Record lightweight usage with preserved provider-native reference ids.
5. Update English and Chinese docs to mark `video.minimax` as active.

## Verification

- `cargo fmt -p sdkwork-api-interface-http`
- `cargo test -p sdkwork-api-interface-http --test openapi_route openapi_routes_expose_gateway_api_inventory -- --nocapture`
- `cargo test -p sdkwork-api-interface-http --test videos_route provider_minimax -- --nocapture`
- `cargo check -p sdkwork-api-interface-http -p sdkwork-api-app-gateway`

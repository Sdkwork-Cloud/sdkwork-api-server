## Goal

Publish `video.vidu` on Vidu's official `/ent/v2/*` video-generation paths with direct HTTP relay, upstream `Authorization: Token ...` compatibility, and stateful provider identity enforcement.

## Architecture

Extend the generic provider mirror relay so it can format upstream authorization per mirror family. Keep current provider-specific families working unchanged while adding `vidu` support. Then publish the Vidu video family on the official create/query/cancel routes.

## Test-First Steps

1. Add OpenAPI guardrails.
   - `video.vidu` tag exists.
   - official Vidu paths exist.
   - each official path is tagged `video.vidu`.
   - wrapper paths remain absent.

2. Add route tests.
   - stateless mode relays all official Vidu routes.
   - upstream receives `Authorization: Token ...` instead of `Bearer ...`.
   - stateful mode routes only to a `vidu` provider, not a generic provider.
   - usage records, billing events, and decision logs are written for the five route keys.

Expected red state: tests fail because `video.vidu` paths, routes, and Token auth formatting are not implemented yet.

## Implementation Steps

1. Generalize provider mirror relay auth formatting by mirror identity.
2. Add `video_vidu` stateless and stateful handler modules.
3. Register the five official routes in stateless and stateful route groups.
4. Add a `gateway_openapi_paths_video_vidu.rs` module and register it from `gateway_openapi.rs`.
5. Update English and Chinese docs to mark `video.vidu` as active.

## Verification

- `cargo fmt -p sdkwork-api-interface-http`
- `cargo test -p sdkwork-api-interface-http --test openapi_route openapi_routes_expose_gateway_api_inventory -- --nocapture`
- `cargo test -p sdkwork-api-interface-http --test videos_route provider_vidu -- --nocapture`
- `cargo test -p sdkwork-api-interface-http --test videos_route -- --nocapture`
- `cargo test -p sdkwork-api-interface-http --test files_route -- --nocapture`
- `cargo check -p sdkwork-api-interface-http -p sdkwork-api-app-gateway`

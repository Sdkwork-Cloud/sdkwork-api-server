# Native Dynamic Lifecycle Design

**Date:** 2026-03-14

**Status:** Approved by the existing extension runtime and OpenAI-compatible gateway baseline for direct implementation

## Goal

Close the strongest remaining `native_dynamic` runtime governance gap by adding optional lifecycle hooks, health contracts, and runtime observability for trusted in-process provider extensions.

## Why This Batch

The current `native_dynamic` runtime already supports:

- JSON provider execution
- SSE relay for `/v1/chat/completions`
- SSE relay for `/v1/responses`
- binary stream passthrough for:
  - `/v1/audio/speech`
  - `/v1/files/{file_id}/content`
  - `/v1/videos/{video_id}/content`

What is still missing is runtime governance:

- no explicit `init` hook when a library becomes active
- no native health contract beyond “the library loaded once”
- no explicit `shutdown` hook for cleanup
- no admin-visible status for loaded native dynamic runtimes

This means the data plane is increasingly complete, but the extension system is still weaker than the approved architecture on lifecycle management and observability.

## Scope

This batch will implement:

1. optional ABI lifecycle hooks for `init`, `health_check`, and `shutdown`
2. host-side runtime state tracking for loaded `native_dynamic` libraries
3. admin-visible runtime status reporting for both:
   - `connector`
   - `native_dynamic`
4. native fixture support and tests proving lifecycle behavior
5. documentation updates reflecting the new runtime governance capabilities

This batch will not implement:

- native dynamic hot reload
- background health polling loops
- per-instance lifecycle hooks tied to provider instance config
- weighted or health-scored routing decisions

## Options Considered

### Option A: Keep native dynamic lifecycle implicit

Pros:

- no ABI changes
- minimal host changes

Cons:

- leaves the main runtime governance gap unresolved
- makes health reporting shallow and misleading
- gives plugins no cleanup contract

### Option B: Add mandatory lifecycle exports

Pros:

- simple host logic
- consistent runtime surface for all plugins

Cons:

- breaks backward compatibility for already built native dynamic libraries
- raises the adoption cost for minimal plugins

### Option C: Add optional lifecycle exports with host-managed runtime status

Pros:

- backward compatible
- lets simple plugins stay small
- gives richer plugins an explicit initialization, health, and shutdown contract
- fits the current host design, where runtime loading already happens in one place

Cons:

- requires slightly more state handling because capabilities are optional

## Recommendation

Use **Option C**.

The current runtime needs governance, but it cannot afford ABI breakage. Optional symbols preserve compatibility while giving the host a real lifecycle boundary and a path toward stricter standards later.

## Runtime Model

This batch defines lifecycle at the **loaded package runtime** level, not at the provider instance level.

That means:

- one loaded native library equals one runtime lifecycle
- `init` is called once when the library is loaded into the host
- `health_check` is called by the host when status is requested
- `shutdown` is called when the loaded runtime is explicitly cleaned up or dropped

This is the correct boundary for the current architecture because:

- one extension package may back many provider instances
- provider instance config such as `base_url` and `credential_ref` already belongs to the dispatch path, not library boot
- package-level runtime state is what the host truly owns

## ABI Shape

The existing ABI remains unchanged for execution.

This batch adds optional exports:

- `sdkwork_extension_init_json`
- `sdkwork_extension_health_check_json`
- `sdkwork_extension_shutdown_json`

Each function exchanges JSON strings through the same string-allocation discipline already used by the execution ABI.

The host will introduce shared lifecycle payload and result types in `sdkwork-api-extension-abi` so plugins and host use the same schema.

The initial payload only needs package-runtime context:

- `extension_id`
- `entrypoint`

The health result should be host-neutral and JSON-based:

- `healthy`
- optional `message`
- optional `details`

This keeps the contract independent from HTTP connector semantics and leaves room for richer metadata later.

## Host Behavior

When a native dynamic runtime is loaded, the host should:

1. load the existing required exports
2. detect optional lifecycle exports if present
3. call `init` once
4. record runtime state, including whether init succeeded and whether health or shutdown hooks are available

When admin or application code requests runtime statuses, the host should:

1. list connector statuses from the existing connector registry
2. list native dynamic runtime statuses from a new native runtime registry
3. call `health_check` on active native runtimes when the hook exists
4. surface the result as a normalized runtime status record

When the runtime is explicitly cleaned up or dropped, the host should:

1. call `shutdown` if the hook exists
2. update runtime state with the shutdown outcome
3. remove dead runtimes from the registry

## Runtime Status Contract

The admin-facing runtime status should become runtime-neutral instead of connector-only.

The status record should include:

- `runtime`
- `extension_id`
- optional `instance_id`
- `display_name`
- `running`
- `healthy`
- `supports_health_check`
- `supports_shutdown`
- optional `base_url`
- optional `health_url`
- optional `process_id`
- optional `library_path`
- optional `message`

This allows:

- connector runtimes to keep reporting process and HTTP health details
- native dynamic runtimes to report package-level lifecycle and ABI health details

## Error Handling

The host should treat lifecycle failures conservatively:

- `init` failure blocks the runtime from becoming usable
- `health_check` failure marks the runtime unhealthy but does not unload it automatically
- `shutdown` failure is recorded as status detail but should not crash cleanup paths

Backward compatibility rules:

- missing optional symbols are valid
- existing plugins without lifecycle hooks continue to load and execute
- admin status should clearly show whether the plugin supports health or shutdown hooks

## Testing Strategy

The batch will be proven through:

1. extension-host tests for:
   - init hook execution
   - health check execution
   - shutdown hook execution
   - backward-compatible loading when hooks are absent
2. app-extension tests proving native dynamic runtime status records are visible
3. admin route tests proving `/admin/extensions/runtime-statuses` returns normalized records
4. fixture lifecycle counters or deterministic status messages in the native mock plugin

## Follow-On Work

After this batch, the strongest remaining runtime gaps should be:

1. explicit unload or reload orchestration for host-owned native runtimes
2. background polling or scheduled health supervision
3. routing decisions informed by runtime health and policy
4. stricter extension standardization for lifecycle metadata once the ABI stabilizes

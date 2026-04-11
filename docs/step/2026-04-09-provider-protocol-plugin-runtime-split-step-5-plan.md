# Provider Protocol / Plugin Runtime Split Step 5 Plan

> Date: 2026-04-09
> Scope: `sdkwork-api-extension-host`, `sdkwork-api-app-gateway`, `sdkwork-api-interface-http`, `sdkwork-api-ext-provider-native-mock`
> Goal: add first-class non-OpenAI plugin execution for Anthropic, Gemini, and heterogeneous custom runtimes without weakening standard-protocol passthrough

## Design

- `openai`, `anthropic`, `gemini` remain first-class industrial-standard protocols
- when upstream already speaks that protocol, gateway keeps direct HTTP passthrough
- when provider is explicitly bound to a runtime plugin and passthrough is not the right path, gateway may call the plugin directly
- plugin-owned execution must be explicit; missing or broken explicit plugins must not silently downgrade
- connector runtimes stay HTTP-supervised; native dynamic runtimes own the raw ABI execution path

## Root Gap

- current extension ABI can execute generic `operation/body/path_params` invocations, but only native dynamic runtime exposes that path
- current app-gateway runtime execution surface only accepts `ProviderRequest`, which is OpenAI-contract-shaped
- Anthropic/Gemini compat handlers can do:
  - standard HTTP passthrough
  - OpenAI translation fallback
- they cannot yet call an explicit plugin using native Anthropic/Gemini operation names

## Execution Rules

1. If routed provider `protocol_kind` matches requested standard protocol, use raw HTTP passthrough.
2. Else if routed provider is explicitly bound to a raw-plugin-capable runtime, call plugin-owned raw operation.
3. Else fall back to existing OpenAI translation path.
4. If provider has explicit plugin binding and plugin runtime is missing or unhealthy, fail safe; do not silently swap to a builtin default plugin.

## Raw Operation Contract

- Anthropic:
  - `anthropic.messages.create`
  - `anthropic.messages.count_tokens`
- Gemini:
  - `gemini.generate_content`
  - `gemini.stream_generate_content`
  - `gemini.count_tokens`
- Heterogeneous custom plugins may expose additional operation names later, but this step only wires the standard compat routes above.

## Implementation Tasks

### Task 1 - Native dynamic raw invocation surface

- expose extension-host helpers that execute raw `ProviderInvocation` JSON/stream payloads against native dynamic runtimes
- expose app-gateway helpers that resolve runtime key, base URL, and api key, then call those raw helpers
- keep existing `ProviderRequest` execution path unchanged

### Task 2 - Planned execution runtime context

- enrich planned execution context with resolved runtime execution metadata needed by compat routes
- avoid a second provider selection during compat execution
- preserve existing single decision-log semantics

### Task 3 - Compat route plugin dispatch

- Anthropic compat routes:
  - passthrough first
  - raw plugin invocation second
  - OpenAI translation fallback third
- Gemini compat routes:
  - passthrough first
  - raw plugin invocation second
  - OpenAI translation fallback third
- preserve current usage capture and commercial admission flow

### Task 4 - Tests first

- add failing interface-http tests for:
  - stateless Anthropic route using native dynamic raw plugin
  - stateful Anthropic route using native dynamic raw plugin
  - stateless Anthropic stream route using native dynamic raw plugin
  - stateful Anthropic stream route using native dynamic raw plugin
  - stateless Anthropic count-tokens route using native dynamic raw plugin
  - stateless Gemini route using native dynamic raw plugin
  - stateful Gemini route using native dynamic raw plugin
  - stateless Gemini stream route using native dynamic raw plugin
  - stateful Gemini stream route using native dynamic raw plugin
  - stateless Gemini count-tokens route using native dynamic raw plugin
  - fail-closed explicit-binding regressions that must still persist one routing decision log
- add focused app-gateway or extension-host tests if raw runtime helpers need direct coverage
- extend native mock fixture to implement the new raw operation names

### Task 5 - Docs and release sync

- update architecture doc with raw-plugin execution order
- update review doc with closed gap and residual risks
- update step update doc with verification
- append release note and changelog entry

## Verification

- `cargo test -p sdkwork-api-interface-http --test anthropic_messages_route`
- `cargo test -p sdkwork-api-interface-http --test gemini_generate_content_route`
- `cargo test -p sdkwork-api-app-gateway`
- `cargo test -p sdkwork-api-extension-host`
- `cargo check -p sdkwork-api-extension-host -p sdkwork-api-app-gateway -p sdkwork-api-interface-http`

## Non-Goals

- no new builtin Anthropic or Gemini provider crate in this step
- no connector ABI redesign in this step
- no removal of existing OpenAI translation fallback

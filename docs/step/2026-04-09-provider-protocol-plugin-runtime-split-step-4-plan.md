# Provider Protocol / Plugin Runtime Split Step 4 Plan

> Date: 2026-04-09
> Scope: `sdkwork-api-extension-core`, `sdkwork-api-extension-host`, `sdkwork-api-app-gateway`
> Goal: make wire protocol capability first-class in plugin/runtime manifest and host resolution while keeping runtime family and extension identity separate

## Design

- `protocol_kind` stays the source of truth for upstream wire protocol: `openai`, `anthropic`, `gemini`, `custom`
- `adapter_kind` stays the runtime/plugin family: builtin default adapters, connector adapters, native dynamic adapters
- `extension_id` stays the concrete implementation identity
- standard protocols prefer passthrough or homogeneous default plugins
- only heterogeneous upstream APIs use custom conversion plugins

## Root Gap

- manifest contract still treats protocol as a narrow optional field tied to legacy families
- host registration aliases providers mostly by `adapter_kind`
- discovered provider registration only elevates `openai`, `openrouter`, `ollama`
- provider execution fallback still resolves by `extension_id` first, not by protocol capability

## Compatibility Rules

- existing manifests using `protocol = "openai"` keep working unchanged
- legacy manifest values that encode runtime family rather than protocol must normalize safely:
  - `openrouter` => protocol capability `openai`
  - `ollama` => protocol capability `custom`
- existing provider `extension_id` addressing keeps working
- new protocol-capability addressing must be additive, not breaking

## Implementation Tasks

### Task 1 - Extend manifest protocol contract

- add first-class `ExtensionProtocol::{OpenAi, Anthropic, Gemini, Custom}`
- preserve backward-compatible parsing for legacy manifest values
- add helper APIs for normalized protocol capability lookup
- add manifest contract tests for:
  - standard protocol serialization
  - legacy protocol normalization
  - custom heterogeneous protocol declaration

### Task 2 - Generalize host protocol registration

- make host register provider aliases from both runtime family and normalized protocol capability
- keep extension-id resolution intact
- keep adapter-family aliases for existing builtin/discovered providers
- add host-level tests for:
  - builtin providers resolvable by extension id, adapter alias, protocol alias
  - discovered providers resolvable by declared protocol capability
  - heterogeneous custom plugins not shadowing standard protocol aliases incorrectly

### Task 3 - Rewire configured host discovery

- let discovered manifest registration branch on normalized protocol capability rather than legacy protocol enum shape
- keep builtin defaults for homogeneous protocols
- keep custom manifests loadable without forcing standard-protocol adapter binding
- add discovery/app-gateway tests for:
  - discovered Anthropic protocol provider registration
  - discovered Gemini protocol provider registration
  - custom connector manifest remaining manifest-only unless an explicit runtime plugin owns execution

### Task 4 - Update provider execution resolution

- keep provider instance execution pinned to concrete `extension_id` when installed
- improve fallback path so protocol-capable builtin/default plugins can be resolved without re-encoding adapter logic at call sites
- ensure runtime resolution order is deterministic:
  1. mounted instance `extension_id`
  2. provider `extension_id`
  3. protocol capability alias
  4. adapter family alias
- add regression tests covering fallback order and standard-protocol resolution

### Task 5 - Sync docs and release notes

- update architecture doc with manifest/runtime capability contract
- update review doc with closed findings and residual risks
- update step update doc with verification commands
- append release note and `docs/release/CHANGELOG.md`

## Verification

- `cargo test -p sdkwork-api-extension-core --test manifest_contract`
- `cargo test -p sdkwork-api-extension-host --test discovery`
- `cargo test -p sdkwork-api-app-gateway --test extension_dispatch`
- `cargo check -p sdkwork-api-extension-core -p sdkwork-api-extension-host -p sdkwork-api-app-gateway`

## Non-Goals

- no new external provider SDK integration in this step
- no protocol translation logic for Anthropic/Gemini here; that already lives at HTTP/app gateway layers
- no removal of existing OpenRouter/Ollama adapters in this step

# SDKWork Extension Architecture Design

**Date:** 2026-03-14

**Status:** Approved for implementation planning

## Goal

Upgrade the current SDKWork API gateway from a contract-and-relay skeleton into a production-oriented, highly pluggable gateway platform with:

- production-grade data-plane identity and tenancy propagation
- standardized channel and proxy provider extension contracts
- configuration-driven extension installation and instance loading
- dynamic and built-in extension runtime support
- a capability-first routing model that can evolve with the current and future OpenAI-style API ecosystem

## Current State Audit

The repository already has strong contract coverage and broad route coverage, but it is not yet accurate to describe the system as a complete, production-grade implementation of all standard APIs.

### Confirmed strengths

- The workspace already exposes a broad OpenAI-compatible `/v1/*` surface through Axum.
- Stateful relay paths exist for chat completions, responses, embeddings, files, images, audio, fine-tuning, assistants, threads, runs, vector stores, batches, evals, and more.
- The gateway already supports multiple storage backends for the control plane, with active SQLite and PostgreSQL implementations.
- The system already models `Channel` and `ProxyProvider` as distinct domain concepts.
- Three secret persistence strategies already exist:
  - `database_encrypted`
  - `local_encrypted_file`
  - `os_keyring`

### Confirmed gaps

1. Many local fallback behaviors are still placeholders.
   - Example: response creation, embedding creation, token counting, assistants, and runs are locally synthesized instead of fully executed when no provider relay path is available.
2. Data-plane tenant and project propagation is not real yet.
   - The HTTP interface still hardcodes `"tenant-1"` and `"project-1"` across most handlers.
3. Admin authentication is not production-safe yet.
   - Current JWT handling is a demo-only token envelope without signature, expiry, issuer, or audience checks.
   - Admin routes are not uniformly guarded by an authentication layer.
4. Provider registration is still compile-time static.
   - A default in-memory registry is rebuilt and populated with `openai`, `openrouter`, and `ollama`.
   - There is no extension manifest, dynamic discovery, runtime lifecycle contract, or configuration-driven loading.
5. The current catalog model is too thin for long-term routing and extension needs.
   - `ProxyProvider` currently assumes a single `channel_id`.
   - `ModelCatalogEntry` currently holds only `external_name` and `provider_id`.

## Industry Model

The gateway must reflect how the market actually behaves:

- `Channel` represents an upstream capability family or ecosystem.
  - Examples: OpenAI, Google, Anthropic, Aliyun, DeepSeek
- `ProxyProvider` represents a concrete access path or broker.
  - Examples: official vendor endpoint, OpenRouter, enterprise relay, local Ollama runtime

One capability such as `chat.completions.create` may be delivered by many channels, and one proxy provider may expose models or capabilities that map to multiple channels. This means:

- `provider -> channel` must not stay one-to-one
- routing must be capability-first, not vendor-first
- compatibility must be expressed as a matrix, not as a boolean

## Architecture Options

### Option A: Static trait registry only

Keep the current trait-based registry model and add more adapter kinds.

Pros:

- easiest short-term implementation
- no ABI complexity
- low migration cost

Cons:

- no real plugin system
- no dynamic discovery or installation
- no clear third-party extension story
- poor fit for desktop and future ecosystem growth

### Option B: Native dynamic library plugins only

Move all channel and provider extensions to dynamic libraries.

Pros:

- high performance
- deep host integration
- direct in-process execution

Cons:

- Rust ABI evolution is fragile
- Windows hot-reload and unload behavior is operationally awkward
- higher crash risk for third-party plugins
- weak cross-language ecosystem support

### Option C: Layered extension system with multiple runtimes

Define one standard extension contract and support multiple execution runtimes:

- `builtin`
- `native_dynamic`
- `connector`

Pros:

- supports dynamic loading and configuration-driven attachment
- keeps built-in high-performance paths for first-party extensions
- allows out-of-process isolation for third-party or experimental integrations
- works in both standalone server mode and embedded Tauri mode

Cons:

- more host-side architecture work
- requires explicit lifecycle and compatibility modeling

### Recommendation

Use **Option C**.

It is the only approach that cleanly satisfies:

- dynamic loading
- pluggability
- configuration-based loading
- future cross-language or sidecar integrations
- embedded desktop runtime safety

## Target Domain Model

The catalog and runtime model should be upgraded to the following bounded concepts:

- `Channel`
- `ProxyProvider`
- `ProviderChannelBinding`
- `Capability`
- `ModelVariant`
- `ExtensionPackage`
- `ExtensionInstallation`
- `ExtensionInstance`
- `CredentialBinding`
- `ExtensionHealthSnapshot`
- `CompatibilityDeclaration`

### Key domain rules

1. `Channel` and `ProxyProvider` remain distinct.
2. A `ProxyProvider` may bind to multiple channels through `ProviderChannelBinding`.
3. `Capability` is first-class and becomes part of routing, extension manifests, and model catalog entries.
4. `ModelVariant` replaces the minimal catalog entry and must carry:
   - external name
   - provider instance
   - channel binding
   - capabilities
   - modality
   - lifecycle
   - context limits
   - streaming support
   - tool support
   - structured output support
   - pricing and billing metadata
   - routing hints

## Extension Standard

Every extension must implement a standardized manifest and lifecycle surface.

### Extension kinds

- `channel`
- `provider`

### Runtime modes

- `builtin`
- `native_dynamic`
- `connector`

### Required manifest fields

- `api_version`
- `id`
- `kind`
- `version`
- `display_name`
- `runtime`
- `entrypoint`
- `supported_channel_bindings`
- `capabilities`
- `config_schema`
- `credential_schema`
- `health_contract`
- `compatibility`

### Example manifest shape

```toml
api_version = "sdkwork.extension/v1"
id = "sdkwork.provider.openrouter"
kind = "provider"
version = "0.1.0"
display_name = "OpenRouter"
runtime = "connector"
entrypoint = "bin/sdkwork-provider-openrouter.exe"

[channel_bindings]
supported = [
  "sdkwork.channel.openai",
  "sdkwork.channel.anthropic",
  "sdkwork.channel.google",
]

[capabilities]
operations = [
  "chat.completions.create",
  "chat.completions.stream",
  "responses.create",
  "responses.stream",
  "embeddings.create",
  "models.list",
]

[config]
schema = "schemas/config.schema.json"

[credentials]
schema = "schemas/credential.schema.json"
required = ["api_key"]

[health]
path = "/health"
interval_secs = 30
```

### Compatibility declaration

The system should stop using a binary `implemented / not implemented` model for extension capability claims. Each endpoint or operation should be marked as:

- `native`
- `relay`
- `translated`
- `emulated`
- `unsupported`

This more accurately reflects real gateway behavior and allows routing, observability, and documentation to stay honest.

## Runtime Host Design

The extension host should be introduced as its own runtime capability instead of burying plugin logic inside the provider registry.

### New host-side components

- `sdkwork-api-extension-core`
  - manifest types
  - capability descriptors
  - lifecycle contracts
  - config schema contracts
  - compatibility types
- `sdkwork-api-extension-host`
  - extension discovery
  - installation registry
  - runtime loading
  - health polling
  - capability lookup
  - shutdown orchestration
- `sdkwork-api-extension-abi`
  - stable FFI boundary for native dynamic plugins

### Lifecycle

Each extension instance should participate in:

- `discover`
- `load`
- `validate_config`
- `bind_credentials`
- `init`
- `health_check`
- `execute`
- `execute_stream`
- `shutdown`

## Configuration Model

The platform should support three levels of extension configuration:

1. `ExtensionPackage`
   - what the extension is
2. `ExtensionInstallation`
   - where it came from, whether it is trusted, and whether it is enabled
3. `ExtensionInstance`
   - environment-specific mounted configuration such as `base_url`, `region`, `timeouts`, `weights`, and `credential_ref`

This enables one package to back multiple installed instances such as:

- `provider-openrouter-main`
- `provider-openrouter-backup`
- `provider-ollama-local`

## Security Model

The plugin system must be designed for safe embedded and server use.

### Rules

- extension configs must not store plaintext secrets
- secrets are resolved through credential references only
- manifest permissions must be explicit
- embedded mode should default to built-in or trusted connector extensions only
- native dynamic extensions should be opt-in in desktop mode
- extension packages should support signature verification and version compatibility checks

### Suggested manifest permission vocabulary

- `network_outbound`
- `filesystem_read`
- `filesystem_write`
- `spawn_process`
- `loopback_bind`

## Required Near-Term Refactors

Before the plugin system can be considered real, the following architectural debt must be cleared:

1. Replace static provider registry construction with an extension host abstraction.
2. Replace single `channel_id` ownership with `ProviderChannelBinding`.
3. Expand model catalog modeling to capability-rich `ModelVariant`.
4. Implement real data-plane authentication and request context extraction from gateway API keys.
5. Replace demo JWT handling with signed JWTs and authenticated admin route guards.
6. Upgrade compatibility documentation from binary status labels to a compatibility level matrix.

## Delivery Strategy

Implementation should proceed in phases.

### Phase 1

- productionize the currently covered API families
- add real data-plane request context extraction
- harden admin authentication
- introduce extension core contracts and host abstractions
- migrate existing `openai`, `openrouter`, and `ollama` adapters into built-in extensions

### Phase 2

- add configuration-driven extension installation and instance mounting
- support connector runtime loading
- add runtime health and compatibility reporting

### Phase 3

- add native dynamic plugin loading through a stable ABI
- expand official compatibility coverage for additional API families
- add richer capability routing, policy, and observability

## Naming Convention

Three names should be kept distinct:

### Distribution or package name

- `sdkwork-channel-openai`
- `sdkwork-provider-openrouter`

### Rust crate name

- `sdkwork-api-ext-channel-openai`
- `sdkwork-api-ext-provider-openrouter`

### Runtime extension ID

- `sdkwork.channel.openai`
- `sdkwork.provider.openrouter`

This keeps user-facing naming intuitive while preserving consistent Rust workspace naming.

# Runtime Modes

## Server Mode

Server mode is the standalone deployment shape.

Characteristics:

- services run as independent binaries
- gateway and admin APIs are exposed over HTTP
- PostgreSQL is the preferred deployment database and is now supported by the shared admin store runtime
- upstream credentials are expected to be managed by a server-side secret backend strategy
- the current repository can run `gateway-service` and `admin-api-service` against either SQLite or PostgreSQL through the same storage abstraction

## Embedded Mode

Embedded mode is the desktop-oriented deployment shape.

Characteristics:

- the runtime is hosted in-process through `sdkwork-api-runtime-host`
- the Tauri shell under `console/src-tauri/` can start or call into the embedded runtime
- loopback binding is the default trust boundary
- SQLite is the preferred local persistence strategy
- OS keyring is the preferred secret backend when available
- the React console packages can target the same admin API surface in both standalone and embedded modes

## Current Implementation State

The current repository includes:

- a minimal `EmbeddedRuntime` abstraction
- a loopback base URL contract
- a Tauri shell scaffold with an initial runtime command
- a live React console that consumes admin APIs for workspace, channel mesh, routing simulation, and telemetry views
- SQLite-backed control-plane persistence for identity, catalog, usage, and billing slices
- PostgreSQL-backed control-plane persistence for the same admin store contract
- encrypted upstream credential persistence and runtime secret resolution
- stateful OpenAI-compatible upstream relay for chat completions, responses, embeddings, and chat SSE when provider configuration is present
- runtime config modeling for storage dialect inference plus secret backend strategy selection
- three live secret persistence strategies:
  - `database_encrypted`
  - `local_encrypted_file`
  - `os_keyring`
- per-credential backend tracking so previously stored secrets can still be resolved after a default backend switch
- environment-driven standalone config loading for bind addresses, database URL, secret backend, credential master key, local secret file path, and keyring service name
- environment-driven extension discovery config for manifest search paths plus connector and native-dynamic runtime toggles
- signed admin JWT authentication for the control plane, with the signing secret now provided by runtime config instead of a hardcoded development constant
- gateway request tenancy derived from persisted gateway API keys instead of hardcoded tenant or project placeholders
- a built-in extension host with manifest registration for OpenAI, OpenRouter, and Ollama provider extensions
- filesystem extension discovery through `sdkwork-extension.toml` package manifests loaded from configured search paths
- persisted extension installation and instance records for configuration-driven mounting
- provider runtime dispatch keyed by `ProxyProvider.extension_id`, with `adapter_kind` kept as a compatibility alias for older records and protocol classification
- real provider dispatch now consumes persisted extension load state so instance-level `base_url` overrides are honored and disabled installations or instances short-circuit to local fallback
- discovered provider manifests can now bind to the current protocol adapters so connector-style extensions participate in real relay execution when they declare a supported protocol

## Extension Runtime Status

The extension architecture is intentionally layered.

| Runtime | Current Status | Notes |
|---|---|---|
| `builtin` | Active | First-party provider extensions are registered in-process through `sdkwork-api-extension-host` |
| `native_dynamic` | Discovery-only | Manifest discovery and policy filtering are active, but ABI loading and execution are still design-only |
| `connector` | Active with protocol mapping | Manifest discovery and config-driven loading are active; supported protocols can relay through the current adapter set, but connector process lifecycle is still not supervised by the host |

## Configuration Layers

The current extension runtime now separates three concerns:

| Layer | Responsibility |
|---|---|
| `ExtensionManifest` | Package identity, compatibility, and capability declaration |
| `ExtensionInstallation` | Installed runtime choice, trust or entrypoint state, enablement, and package-level config |
| `ExtensionInstance` | Mounted environment-specific config such as `base_url`, credential reference, and per-instance settings |

This enables one extension package to back multiple concrete instances in either standalone or embedded mode.

Provider execution identity is now split deliberately:

- `extension_id`: the canonical runtime key used by the gateway to resolve a provider extension implementation
- `adapter_kind`: a compatibility alias and protocol hint that lets older records and OpenAI-compatible provider families continue to map cleanly during migration

The extension package standard also distinguishes three names:

- runtime ID:
  - `sdkwork.provider.openrouter`
  - `sdkwork.channel.openai`
- distribution package name:
  - `sdkwork-provider-openrouter`
  - `sdkwork-channel-openai`
- Rust crate name:
  - `sdkwork-api-ext-provider-openrouter`
  - `sdkwork-api-ext-channel-openai`

Configuration-driven loading now uses a stable merge order:

1. manifest metadata and default entrypoint references
2. installation-level runtime choice and package config
3. instance-level overrides such as `base_url`, `credential_ref`, and rollout weights

This means `connector` and `native_dynamic` extensions can already be represented, validated, and mounted through config even before the actual executable loader lifecycle is fully wired.

The runtime now also supports two manifest sources:

1. built-in manifests registered in process
2. external package manifests discovered from configured extension search paths

Discovered provider manifests become executable only when:

1. the runtime policy enables their declared runtime
2. the manifest declares a supported protocol
3. persisted installation and instance state resolves to an enabled runtime load plan

The runtime host is still intentionally lightweight, but the core gateway, admin, routing, credential, and provider relay slices now run against the same Rust workspace and can be assembled in-process for embedded mode.

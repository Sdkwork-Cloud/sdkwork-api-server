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
- explicit stateless gateway runtime modeling with synthetic tenant and project scope plus optional single-upstream relay across the current OpenAI-compatible data-plane surface, including files, uploads, audio, images, assistants, threads, conversations, vector stores, batches, fine-tuning jobs, webhooks, evals, and videos
- shared Prometheus-compatible HTTP metrics exposed through `/metrics` on both admin and gateway routers
- shared `x-request-id` propagation and structured HTTP request tracing across both admin and gateway routers, with tracing initialized by the standalone binaries
- persisted routing policies shared by admin simulations and real gateway provider dispatch
- routing decisions now join persisted policy order with runtime status and instance-level config hints for deterministic health-aware selection, seeded weighted-random balancing, or first-class `slo_aware` selection with best-effort degraded fallback
- routing decision logs are now persisted for both gateway execution and admin simulations so recent selection evidence can be inspected through the control plane
- a built-in extension host with manifest registration for OpenAI, OpenRouter, and Ollama provider extensions
- filesystem extension discovery through `sdkwork-extension.toml` package manifests loaded from configured search paths
- persisted extension installation and instance records for configuration-driven mounting
- authenticated admin visibility for discovered extension packages and normalized runtime statuses across connector and native dynamic runtimes
- discovery-time validation of manifest permissions, bindings, capabilities, entrypoints, and connector health contracts
- discovery-time trust verification for external extension packages, including publisher identity, ed25519 signature checks, and trusted-signer policy evaluation
- provider runtime dispatch keyed by `ProxyProvider.extension_id`, with `adapter_kind` kept as a compatibility alias for older records and protocol classification
- real provider dispatch now consumes persisted extension load state so instance-level `base_url` overrides are honored and disabled installations or instances short-circuit to local fallback
- discovered provider manifests can now bind to the current protocol adapters so connector-style extensions participate in real relay execution when they declare a supported protocol
- connector runtime supervision is now active for discovered packages, including host-managed process startup, HTTP health probing, and reuse of already healthy external endpoints
- native dynamic runtime execution is now active for trusted provider packages through a narrow JSON ABI, with manifest matching, optional lifecycle hooks, health contracts, and symbol validation at load time
- gateway runtime loading now skips external packages whose trust policy does not allow execution, so blocked connector or native-dynamic packages fall back cleanly instead of entering the execution host
- standalone services can now supervise provider runtime health in the background and persist provider-centric health snapshots for later routing fallback or admin inspection

## Extension Runtime Status

The extension architecture is intentionally layered.

| Runtime | Current Status | Notes |
|---|---|---|
| `builtin` | Active | First-party provider extensions are registered in-process through `sdkwork-api-extension-host` |
| `native_dynamic` | Active with JSON, generic stream ABI, and lifecycle execution | Trusted packages can be loaded in-process through `libloading`, validated against the exported manifest, executed for JSON-capable provider operations plus `/v1/chat/completions` and `/v1/responses` SSE relay and binary stream passthrough for `/v1/audio/speech`, `/v1/files/{file_id}/content`, and `/v1/videos/{video_id}/content`, and observed through optional `init`, `health_check`, and `shutdown` hooks |
| `connector` | Active with supervised execution | Manifest discovery and config-driven loading are active; the host can start configured connector processes, probe HTTP health endpoints, reuse healthy external endpoints, and relay through the current protocol adapter set |

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

The stateless bootstrap runtime now mirrors that split:

- `StatelessGatewayUpstream.runtime_key` can point at either a canonical `extension_id` or a compatibility alias
- built-in compatibility aliases currently include `openai`, `openai-compatible`, `custom-openai`, `openrouter`, `openrouter-compatible`, `ollama`, and `ollama-compatible`
- library consumers can keep bootstrap wiring narrow while still moving cleanly to explicit extension IDs later

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

This means `connector` extensions are executable through the host runtime contract, while `native_dynamic` extensions are now executable for JSON-capable provider operations, chat or responses SSE stream relay, the current binary stream routes for audio speech plus file and video content, and package-runtime lifecycle or health contracts through the same ABI boundary. Background health snapshot supervision is now active for standalone services, while hot reload remains future work.

The runtime now also supports two manifest sources:

1. built-in manifests registered in process
2. external package manifests discovered from configured extension search paths

Discovered package observability now includes:

1. normalized distribution and crate naming
2. manifest validation results for explicit permissions, channel bindings, capabilities, and runtime contract completeness
3. trust verification results for signature presence, publisher trust, signature validity, and load eligibility

Discovered provider manifests become executable only when:

1. the runtime policy enables their declared runtime
2. the manifest declares a supported protocol
3. the trust policy marks the package as loadable
4. persisted installation and instance state resolves to an enabled runtime load plan

Routing remains intentionally conservative in this batch:

- policy precedence is deterministic by `priority DESC` then `policy_id ASC`
- model matching supports exact and glob-style `*` patterns
- provider selection now considers availability, runtime health, and instance-level `cost`, `latency_ms`, and `weight` hints in addition to ordered preference plus optional default provider
- provider health now falls back to the latest persisted snapshot when live runtime status is unavailable
- project-scoped quota-aware admission now rejects over-budget requests for `/v1/chat/completions`, `/v1/completions`, `/v1/responses`, and `/v1/embeddings` before upstream dispatch
- regional policy dimensions such as geo affinity remain future work

The runtime host is still intentionally lightweight, but the core gateway, admin, routing, credential, and provider relay slices now run against the same Rust workspace and can be assembled in-process for embedded mode.

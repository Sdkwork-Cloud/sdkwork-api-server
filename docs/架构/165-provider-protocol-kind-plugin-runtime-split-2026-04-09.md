# Provider Protocol Kind / Plugin Runtime Split

> Date: 2026-04-09
> Status: active
> Scope: provider catalog, admin API, portal API, storage, app gateway, HTTP gateway

## Goal

Provider integration must split three concerns:

- `protocol_kind`: external wire protocol seen by upstream, such as `openai`, `anthropic`, `gemini`, `custom`
- `adapter_kind`: runtime execution adapter/plugin family, such as `openai`, `openrouter`, `ollama`, `native-dynamic`
- `extension_id`: concrete plugin/runtime implementation identity

Rule:

- industrial standard protocols should stay standard
- `openai`, `anthropic`, `gemini` should prefer direct passthrough when the upstream already speaks that standard
- only non-standard or wrapped provider APIs should use conversion plugins

## Completed

### 1. Provider metadata split

- added `protocol_kind` to provider domain, app catalog, admin API, SQLite, PostgreSQL
- kept backward compatibility by deriving protocol from `adapter_kind` when the field is absent
- added explicit stateless upstream protocol metadata while keeping legacy constructors

### 2. Gateway edge passthrough

- stateless gateway now passthroughs native Anthropic and Gemini JSON/SSE routes when `protocol_kind` matches
- stateful gateway now supports the same passthrough for routed providers
- native token usage extraction was added for Anthropic and Gemini response envelopes

### 3. Single audit semantics

- added a no-log planned provider context helper in `sdkwork-api-app-gateway`
- stateful standard-protocol probing now resolves provider, secret, and usage context without creating a duplicate routing decision log
- when passthrough is actually chosen, the gateway persists exactly one routing decision log before execution
- translated OpenAI-compatible fallback keeps its original single-log behavior

### 4. Unified planned execution

- `sdkwork-api-app-gateway` now exposes chat JSON, chat SSE, and count-tokens execution entrypoints that consume `PlannedExecutionProviderContext`
- stateful Anthropic/Gemini compat handlers now select one provider plan per request
- if `protocol_kind` matches, they relay raw HTTP from that plan
- if `protocol_kind` does not match, they execute translated OpenAI-compatible runtime from that same plan
- mixed-protocol routed groups no longer re-select a second provider between protocol inspection and translated execution

### 5. Plugin runtime protocol capability

- `sdkwork-api-extension-core` now models canonical runtime protocol capability as `openai`, `anthropic`, `gemini`, `custom`
- legacy manifest protocol values still load, but normalize safely:
  - `openrouter` => `openai`
  - `ollama` => `custom`
- builtin runtime manifests now declare explicit protocol capability instead of relying on adapter-family inference
- app gateway runtime fallback now resolves in this order:
  1. mounted instance / explicit extension binding
  2. concrete `extension_id`
  3. canonical `protocol_kind`
  4. `adapter_kind`
- protocol fallback is only allowed when the provider has no persisted extension installation or instance, so blocked or missing explicit plugins do not silently downgrade to a default builtin adapter

### 6. Raw plugin execution surface

- `sdkwork-api-extension-host` now exposes raw native-dynamic JSON/SSE execution on top of the existing `ProviderInvocation` ABI contract
- `sdkwork-api-app-gateway` now exposes raw runtime execution helpers and planned execution context now carries resolved runtime metadata:
  - `runtime_key`
  - resolved `base_url`
  - `runtime`
  - `local_fallback`
- Anthropic and Gemini compat routes now execute in this order:
  1. standard-protocol passthrough
  2. explicit raw plugin execution
  3. translated OpenAI-compatible fallback
- stateful compat flows now fail closed when an explicitly bound plugin runtime degrades to `local_fallback`
- fail-closed explicit-binding paths now still persist one routing decision log before returning gateway failure, so broken plugin bindings stay auditable instead of disappearing from routing history
- native mock runtime fixtures and signed test manifests now advertise and implement:
  - `anthropic.messages.create`
  - `anthropic.messages.count_tokens`
  - `gemini.generate_content`
  - `gemini.stream_generate_content`
  - `gemini.count_tokens`

### 7. Raw execution governance parity

- stateful raw-plugin execution now reuses gateway execution-context controls instead of bypassing them:
  - deadline rejection
  - request timeout
  - in-flight overload protection
- raw JSON execution now runs under a blocking execution-context wrapper, so native-dynamic ABI calls can time out without skipping gateway failure accounting
- raw JSON/SSE planned execution now records the same gateway upstream attempt/success/failure metrics used by OpenAI-shaped execution
- raw JSON/SSE planned execution now persists provider health snapshots on success and on health-impacting failures
- Anthropic/Gemini stateful compat handlers now call the governed raw execution entrypoints instead of the old thin runtime wrappers
- governance boundary is explicit: stream execution-context currently covers stream startup/handshake, not full downstream body drain lifetime

### 8. Conservative raw JSON retry classification

- raw JSON planned execution now derives retry policy from the selected planned routing decision instead of ignoring `matched_policy_id`
- raw JSON retries only when the failure class is explicitly retryable under gateway policy; verified path today is execution-context timeout
- opaque native-dynamic plugin errors still fail without retry, so plugin-owned business failures are not guessed into retry loops
- raw SSE keeps the current no-retry posture because stream governance still ends at startup/acquisition

### 9. Explicit native-dynamic transient error contract

- native-dynamic ABI errors can now carry optional structured retry hints instead of forcing the gateway to parse free-form error text
- the runtime host preserves that metadata on `NativeDynamicInvocationFailed`, so gateway governance can classify retryability from typed plugin intent
- verified contract today:
  - plugin marks error as retryable => raw JSON may retry under route policy
  - plugin omits retryable hint => gateway treats the error as opaque and does not retry
  - plugin may provide `retry_after_ms`, and gateway uses it as the retry delay source when it exceeds backoff

### 10. Raw SSE startup retry boundary

- raw SSE planned execution now derives retry policy from the selected planned routing decision instead of hard-disabling retry for the whole stream path
- retry is still constrained to stream startup only: the gateway may retry only before the plugin emits the first content type or body chunk
- verified contract today:
  - plugin marks startup failure as retryable => raw SSE may retry under route policy
  - plugin omits retryable hint => startup failure stays opaque and does not retry
  - once stream metadata or body starts flowing, retry responsibility ends and downstream drain stays unchanged

### 11. Connector supervision async boundary

- connector runtimes still stay on the supervised HTTP path and never enter the raw native-dynamic execution surface
- connector startup/health supervision is blocking by nature, so planned provider resolution now runs it on `spawn_blocking` instead of inline on the async request executor
- this closes a current-thread runtime failure mode where compat planning could block the executor, starve a spawned connector health server, and then misclassify the provider as a missing-entrypoint connector failure
- when no local filesystem entrypoint exists, connector startup may now adopt an externally managed endpoint that becomes healthy within the configured startup budget

### 12. Planned execution connector supervision dedup

- planned execution context now resolves connector runtime supervision once and reuses that execution target when building usage context
- this removes duplicate `/health` probing for the same connector provider inside one `planned_execution_provider_context_for_route_without_log(...)` call
- connector runtimes still stay on the supervised HTTP path; this change is reuse-only and does not widen raw plugin execution to connector runtime families

### 13. Runtime capability boundary codification

- `ExtensionRuntime` now exposes capability helpers for raw provider execution and structured retry hints
- current canonical rule is executable instead of implicit:
  - `native_dynamic` => raw plugin surface allowed, structured retry hints allowed
  - `connector` => raw plugin surface denied, structured retry hints denied
  - `builtin` => raw plugin surface denied, structured retry hints denied
- app gateway raw JSON/SSE guards and extension-host raw runtime resolution now consume that shared runtime capability contract instead of open-coded `NativeDynamic` equality checks

### 14. Planned execution preflight failover for connector startup failures

- stateful planned execution context construction now treats selected-provider runtime-target resolution failures as failover candidates when the matched route policy enables execution failover
- current verified preflight failover case is connector runtime startup / health-adoption failure that occurs before request relay begins
- when a backup provider is selected during planning:
  - planned decision `selected_provider_id` is rewritten to the executable backup provider
  - `fallback_reason` appends `gateway_execution_failover`
  - planned usage context and persisted decision log stay aligned with the provider that actually executes
- this change is preflight-only:
- connector runtimes still remain on the supervised HTTP path
- raw plugin execution is still native-dynamic only
- explicit local-fallback / fail-closed plugin-binding semantics stay unchanged

### 15. Planned execution preflight failover for missing tenant credentials

- stateful planned execution context construction now also treats selected-provider missing tenant credential as a preflight failover candidate when route policy enables execution failover
- current verified standard-protocol case is stateful Anthropic passthrough:
  - ordered policy selects the primary provider first
  - primary provider has no tenant credential
  - backup provider has both tenant credential and executable target
  - request is sent directly to the backup upstream without touching the primary upstream or local fallback
- when backup selection happens during planning:
  - planned `selected_provider_id` is rewritten to the executable backup provider
  - `fallback_reason` appends `gateway_execution_failover`
  - planned usage context and persisted decision log stay aligned with the provider that actually executes
- this widening is still preflight-only:
  - passthrough-first for standard `openai` / `anthropic` / `gemini` remains unchanged
  - connector stays on the supervised HTTP path
- raw plugin execution stays native-dynamic only
- explicit fail-closed plugin-binding semantics still win over silent fallback

### 16. Direct store-relay OpenAI preflight failover parity

- direct store-relay `chat/responses` JSON and SSE execution now reuse the same selected-provider missing-tenant-credential preflight failover rule as planned compat execution
- current verified cases:
  - stateful OpenAI chat JSON
  - stateful OpenAI chat SSE
  - stateful OpenAI responses JSON
  - stateful OpenAI responses SSE
- when ordered routing selects a primary provider that lacks the tenant credential and route policy enables execution failover:
  - the primary upstream is skipped before any request is sent
  - the first backup provider with a tenant credential executes directly
  - usage record and routing decision log point at the provider that actually executed
  - `fallback_reason` includes `gateway_execution_failover`
- unchanged boundaries:
  - this widening is still about missing tenant credential only
  - missing-provider remains outside execution-layer preflight failover
  - route selection now skips policy-declared candidates that are absent from the current available candidate set and marks `fallback_reason=policy_candidate_unavailable`
  - execution/planning replacement still uses `gateway_execution_failover`, so routing-layer candidate unavailability stays distinct from real execution failover
  - passthrough-first for standard `openai` / `anthropic` / `gemini` remains unchanged
  - heterogeneous conversion stays plugin-owned and only for non-standard upstreams

### 17. OpenAI-route incompatible-standard preflight failover

- direct store-relay OpenAI `chat/responses` JSON and SSE execution now also treats a selected provider as preflight-incompatible when:
  - the route requires the OpenAI-shaped provider-adapter surface
  - the selected provider only exposes a different industrial standard such as `anthropic` or `gemini`
  - no explicit conversion plugin/runtime is installed for that provider
- verified contract:
  - the incompatible primary provider is not contacted
  - the gateway does not silently drop to local mock/local fallback
  - if execution failover is enabled, the first compatible backup provider executes and audit rewrites to that provider with `gateway_execution_failover`
  - if no compatible executable provider exists, the gateway returns execution failure instead of fabricating success from local fallback
- this keeps the core design explicit:
  - standard-to-standard passthrough only happens on the matching standard route family
  - cross-standard execution on OpenAI routes still requires an explicit plugin/runtime that can execute the OpenAI-shaped adapter contract
  - non-standard heterogeneous normalization remains plugin-owned only when the upstream cannot already speak a supported industrial standard

### 18. Provider executability observability

- `/admin/providers` now returns an additive `execution` view derived from gateway provider-resolution truth instead of exposing only static catalog metadata
- current operator-visible fields are:
  - `binding_kind`
  - `runtime`
  - `runtime_key`
  - `passthrough_protocol`
  - `supports_provider_adapter`
  - `supports_raw_plugin`
  - `fail_closed`
  - `reason`
- verified contracts:
  - implicit standard providers such as official OpenAI now show implicit-default execution plus standard passthrough visibility
  - explicit native-dynamic plugins now show raw-plugin capability without hiding adapter-surface capability when the plugin also exposes it
  - explicit persisted bindings that are missing, disabled, or otherwise unloadable now show `fail_closed=true` instead of forcing operators to infer that risk from separate runtime and routing pages
- this keeps management truth aligned with runtime truth:
  - admin observability uses the same gateway resolution rules that execution uses
  - the view is side-effect free and does not start connector/native-dynamic runtimes
  - runtime-status and provider-health endpoints remain separate live-signal sources; the provider catalog view explains executability boundaries, not live health

### 19. Route-family readiness observability

- found an operator ambiguity after section 18:
  - `fail_closed=true` only means the adapter/raw-plugin surface is not executable
  - a provider may still serve its matching industrial-standard route family through direct passthrough
- `/admin/providers.execution` now adds `route_readiness`:
  - `openai`
  - `anthropic`
  - `gemini`
- each family exposes:
  - `ready`
  - `mode`
- current `mode` values are:
  - `provider_adapter`
  - `standard_passthrough`
  - `raw_plugin`
  - `fail_closed`
  - `unavailable`
- verified contracts:
  - official OpenAI provider shows `openai/anthropic/gemini => provider_adapter`
  - explicit Anthropic native-dynamic plugin shows:
    - `anthropic => standard_passthrough`
    - `gemini => raw_plugin`
    - `openai => provider_adapter`
  - broken explicit Anthropic plugin binding still shows:
    - `anthropic => standard_passthrough`
    - `openai/gemini => fail_closed`
- this keeps the design explicit:
  - industrial-standard matching routes stay passthrough-first
  - heterogeneous normalization is only reported as `raw_plugin` when the manifest actually exposes the required operation family
  - global `fail_closed` remains valuable, but it no longer has to carry route-family semantics by itself

### 20. Default-plugin protocol truth for custom-compatible providers

- found one remaining classification gap in the default-plugin path:
  - `ollama` and `ollama-compatible` were still being derived as `protocol_kind=openai` in domain and stateless config helpers
  - builtin runtime truth already declared `sdkwork.provider.ollama` as `protocol=custom`
  - this mislabeled a default-plugin-capable provider as an industrial-standard passthrough provider
- fixed the rule:
  - `openai`, `openai-compatible`, `custom-openai`, `openrouter`, `openrouter-compatible` => `protocol_kind=openai`
  - `anthropic*` / `claude*` => `protocol_kind=anthropic`
  - `gemini*` => `protocol_kind=gemini`
  - `ollama`, `ollama-compatible` => `protocol_kind=custom`
- resulting contract:
  - standard providers still stay passthrough-first on matching industrial-standard routes
  - same-structure but non-standard providers can still onboard through default plugins/adapters
  - non-standard providers are no longer overreported as standard-protocol passthrough-capable

### 21. Admin default-plugin onboarding semantics

- found one remaining management-side ambiguity after sections 19 and 20:
  - operators still had to express same-structure builtin onboarding through low-level `adapter_kind`
  - `/admin/providers` did not expose whether a provider was using standard passthrough, a default plugin family, or a custom plugin contract
  - this made `openrouter` and `ollama` onboarding harder to distinguish from official-standard passthrough or explicit heterogeneous plugin binding
- fixed the admin contract:
  - `POST /admin/providers` now accepts additive `default_plugin_family`
  - current first-class default families are:
    - `openrouter`
    - `ollama`
  - when `default_plugin_family` is used:
    - `adapter_kind` is normalized to that family
    - `protocol_kind` is pinned to the family-derived protocol
    - `extension_id` is pinned to the builtin default-plugin runtime id
    - conflicting `adapter_kind` / `protocol_kind` / `extension_id` input is rejected instead of silently drifting into a custom-plugin path
- `/admin/providers` now also returns additive `integration`:
  - `mode=standard_passthrough`
  - `mode=default_plugin`
  - `mode=custom_plugin`
  - `default_plugin_family` is surfaced when the provider is on the default-plugin path
- resulting contract:
  - standard industrial protocols still stay distinct from plugin onboarding
  - same-structure builtin providers now have a first-class config-only onboarding path
  - heterogeneous or explicitly bound runtimes stay on the custom-plugin path and remain clearly separated from builtin default families

### 22. Default-plugin normalization ownership in app catalog

- found one remaining layering gap after section 21:
  - `default_plugin_family` normalization still lived only inside the admin HTTP layer
  - other control-plane callers could still recreate the same policy differently
  - admin integration tests still contained legacy `openrouter -> adapter_kind=openai` fixtures, which reinforced the wrong onboarding shape
- fixed the ownership boundary:
  - `sdkwork-api-app-catalog` now owns default-plugin normalization as reusable application logic
  - the shared normalization contract now resolves:
    - canonical `adapter_kind`
    - derived `protocol_kind`
    - derived `extension_id`
    - conflict rejection for incompatible low-level fields
  - admin provider creation now consumes that shared app-catalog contract instead of re-implementing it locally
  - admin routing/model tests now create OpenRouter through `default_plugin_family=openrouter` rather than the old `adapter_kind=openai` shortcut
- resulting contract:
  - default-plugin onboarding semantics now live in the application layer, not in one HTTP controller
  - management examples no longer teach the old conflated OpenRouter shape
  - future control-plane entrypoints can reuse one normalization rule instead of forking provider identity policy

### 23. Non-admin fixture convergence on default-plugin identity

- found one remaining follow-through gap after section 22:
  - `sdkwork-api-app-routing` regressions still created OpenRouter through the old official-OpenAI fixture shape
  - `sdkwork-api-app-gateway` routing relay regression still mounted OpenRouter through `sdkwork.provider.openai.official`
  - non-admin tests could therefore stay green while teaching the wrong provider identity
- fixed the regression boundary:
  - app-routing test support now provides explicit OpenRouter fixture construction with:
    - primary `channel_id=openrouter`
    - `adapter_kind=openrouter`
    - derived `protocol_kind=openai`
    - secondary `openai` channel binding for OpenAI-route coverage
  - simulate-route and geo/profile regressions now create OpenRouter through that canonical default-plugin identity instead of the old `openai/openai` shortcut
  - app-gateway routing relay regression now installs and binds `sdkwork.provider.openrouter` instead of the official OpenAI builtin runtime
  - routing and relay regressions now assert OpenRouter identity directly:
    - `adapter_kind=openrouter`
    - `protocol_kind=openai`
    - `extension_id=sdkwork.provider.openrouter`
- resulting contract:
  - management and non-management test surfaces now teach the same provider identity model
  - OpenRouter remains industrial-standard `openai` protocol, but not official OpenAI runtime identity
  - future routing/relay changes now have regression coverage against drifting back to `sdkwork.provider.openai.official`

### 24. Stateless protocol truth and integration-sample convergence

- found one remaining follow-up gap after section 23:
  - stateless gateway upstream construction in `sdkwork-api-interface-http` still owned a duplicate protocol-derivation table
  - portal routing fixtures still inserted OpenRouter through the old `openai/openai` identity shortcut
  - HTTP relay examples still taught OpenRouter/Ollama onboarding through raw low-level `adapter_kind` input instead of the first-class default-plugin path
- fixed the remaining boundary drift:
  - `StatelessGatewayUpstream` now reuses the shared domain-catalog protocol derive/normalize helpers, so stateless config follows the same protocol truth as provider metadata and storage normalization
  - portal routing fixtures now build OpenRouter with canonical default-plugin identity:
    - primary `channel_id=openrouter`
    - `adapter_kind=openrouter`
    - `protocol_kind=openai`
    - `extension_id=sdkwork.provider.openrouter`
    - secondary `openai` binding for OpenAI-route coverage
  - HTTP relay regressions now create OpenRouter/Ollama through `default_plugin_family` and assert the derived provider identity fields returned by admin creation
- resulting contract:
  - provider protocol truth now has one owner across domain, storage, admin, and stateless HTTP entrypoints
  - config-only builtin onboarding examples no longer teach operators to express same-structure providers as raw adapter strings
  - OpenRouter/Ollama remain distinct from official industrial-standard passthrough even when they serve those route families

### 25. Shared default-plugin seeding helper convergence

- found one remaining governance gap after section 24:
  - app/domain layers already owned default-plugin identity normalization
  - high-value non-admin fixtures still had to open-code "default family + extra channel bindings"
  - that kept behavior green but duplicated canonical OpenRouter seeding
- fixed the gap in application-layer helpers:
  - `sdkwork-api-app-catalog` now exposes `create_provider_with_default_plugin_family_and_bindings(...)`
  - the existing default-plugin constructor now reuses that helper
  - high-value app-routing, app-gateway, and portal fixtures now consume shared app-catalog constructors instead of rebuilding canonical OpenRouter identity inline
  - adjacent standard OpenAI fixtures at those seams now also reuse `create_provider_with_config(...)`
- resulting contract:
  - provider identity truth stays in app/domain helpers even when fixtures need secondary bindings such as `openai`
  - same-structure builtin providers stay distinct from official passthrough providers in test/support code as well as runtime code
  - future non-admin seeders can reuse one helper instead of re-teaching `channel_id/adapter_kind/protocol_kind/extension_id` manually

### 26. Admin provider create-response integration convergence

- found one remaining control-plane mutation gap after section 25:
  - `POST /admin/providers` returned normalized provider fields only
  - callers had to re-fetch `/admin/providers` to learn whether onboarding landed on `standard_passthrough`, `default_plugin`, or `custom_plugin`
  - create-time control-plane truth was therefore weaker than list-time control-plane truth even though both were derived from the same provider metadata
- fixed the contract conservatively:
  - `POST /admin/providers` now returns additive `integration`
  - the create response still does not return `execution`
  - `integration` is derived from shared provider metadata, not from live runtime probing
- resulting contract:
  - create response = normalized onboarding truth
  - list response = normalized onboarding truth + runtime execution truth
  - mutation semantics and runtime observability stay separate

### 27. Portal routing provider integration visibility

- found one remaining frontend/operator visibility gap after section 26:
  - portal routing summary provider options still exposed only `provider_id`, `display_name`, and `channel_id`
  - frontend/provider-choice surfaces could not tell whether a provider was industrial-standard passthrough, builtin default-plugin onboarding, or custom-plugin normalization
  - transport layers were at risk of re-implementing provider-integration classification independently
- fixed the ownership boundary:
  - `sdkwork-api-app-catalog` now owns reusable `ProviderIntegrationView` derivation
  - admin create/list responses now consume that shared metadata helper instead of owning their own classification implementation
  - portal routing provider options now expose additive:
    - `protocol_kind`
    - `integration`
- resulting contract:
  - frontend/provider-choice surfaces can now preserve the industrial-standard-first rule without inferring from runtime internals
  - onboarding metadata is shared application truth, not duplicated per transport
  - portal visibility stays static/side-effect-free and does not mix in secret presence or runtime health

### 28. Portal tenant credential-readiness split

- found the next frontend/provider-choice gap after section 27:
  - portal now exposed static `protocol_kind` and `integration`
  - portal still could not tell whether the current workspace tenant had configured a credential for each provider
  - folding that state into `integration` would have mixed static onboarding truth with tenant-scoped secret state
- fixed the contract conservatively:
  - portal routing `provider_options` now expose additive `credential_readiness`
  - current fields are:
    - `ready`
    - `state`
  - current states are:
    - `configured`
    - `missing`
  - readiness is derived from tenant-scoped credential presence only
  - readiness stays separate from:
    - `protocol_kind`
    - `integration`
    - admin-side `execution`
    - live runtime health probing
- resulting contract:
  - frontend/provider-choice UX can distinguish "standard/default-plugin/custom" from "credential configured for this tenant"
  - static provider metadata remains stable across tenants
  - dynamic secret state stays additive and explicit instead of being smuggled into static catalog fields

### 29. Admin tenant-scoped credential-readiness split

- found the next operator gap after section 28:
  - `/admin/providers` now exposed static onboarding truth plus runtime execution truth
  - operators still lacked one catalog view for "is this provider configured for tenant X"
  - adding that state unconditionally to the global list would have polluted the default catalog semantics with tenant-scoped secret presence
- fixed the contract conservatively:
  - `/admin/providers` keeps its existing global semantics by default
  - `GET /admin/providers?tenant_id=<tenant>` now adds `credential_readiness`
  - current fields are:
    - `ready`
    - `state`
  - current states are:
    - `configured`
    - `missing`
  - readiness is derived from tenant-scoped credential presence only
  - readiness stays separate from:
    - static `integration`
    - runtime `execution`
    - live runtime health
- implementation ownership:
  - tenant-scoped readiness derivation now lives in `sdkwork-api-app-credential`
  - portal and admin both reuse the same readiness contract instead of duplicating stringly typed transport-local logic
- resulting contract:
  - global admin catalog remains side-effect free and tenant-agnostic unless a tenant scope is requested explicitly
- operator tooling can ask one focused tenant-scoped readiness question without collapsing provider metadata, runtime executability, and secret presence into one field
- portal/admin now share one app-layer readiness semantic

### 30. Admin provider OpenAPI contract convergence

- found one remaining control-plane contract gap after section 29:
  - runtime already supported `GET /admin/providers?tenant_id=...`
  - admin OpenAPI still did not publish provider list/create routes or the optional tenant scope explicitly
  - generated clients and operator docs could therefore treat `credential_readiness` as undocumented transport behavior
- fixed the contract without changing runtime semantics:
  - added admin OpenAPI `catalog` documentation for:
    - `GET /admin/providers`
    - `POST /admin/providers`
  - published the optional `tenant_id` query as an explicit opt-in scope that only adds `credential_readiness`
  - exposed real schemas for:
    - normalized provider metadata
    - `integration`
    - `execution`
    - optional tenant-scoped `credential_readiness`
- resulting contract:
- admin schema now preserves the same three-way split used by runtime and transport code
- client generation no longer depends on route-local tribal knowledge
- tenant overlay stays explicit instead of implicit

### 31. Dedicated tenant provider-readiness endpoint

- found the next governance gap after section 30:
  - `GET /admin/providers?tenant_id=...` now documents tenant-scoped readiness correctly
  - but that shape is still a global provider catalog with a tenant overlay bolted onto it
  - future tenant-scoped dimensions would keep stretching the global catalog route unless tenant-state gets its own boundary
- fixed the boundary conservatively:
  - kept `GET /admin/providers?tenant_id=...` as the convenience combined operator view
  - added dedicated `GET /admin/tenants/{tenant_id}/providers/readiness`
  - dedicated endpoint returns:
    - stable provider identity for display and joining
    - static `integration`
    - tenant-scoped `credential_readiness`
  - dedicated endpoint intentionally does not inline `execution`, so tenant overlay stays separate from global runtime executability truth
- resulting contract:
  - global provider catalog remains the place for static + runtime truth
  - tenant provider readiness now has a focused extensible surface for future tenant-state dimensions
  - query overlay and dedicated endpoint can coexist without semantic conflict

### 32. Stateless default-plugin ingress contract

- found one remaining non-admin ingress gap after section 31:
  - admin create/list and shared fixtures already had first-class `default_plugin_family`
  - stateless direct upstream configuration still mostly exposed raw `runtime_key` / adapter semantics
  - same-structure builtin providers such as `openrouter` and `ollama` therefore still required callers to encode low-level runtime identity manually outside admin
- fixed the boundary conservatively:
  - added `StatelessGatewayUpstream::from_default_plugin_family(...)`
  - added `StatelessGatewayConfig::try_with_default_plugin_upstream(...)`
  - both stateless entrypoints normalize through shared domain-catalog default-plugin family rules instead of inventing transport-local identity policy
  - unsupported families fail fast instead of silently degrading to raw `runtime_key` input
  - legacy `new(...)`, `new_with_protocol_kind(...)`, and `from_adapter_kind(...)` remain available for explicit advanced or backward-compatible paths
- resulting contract:
  - stateless non-admin ingress now matches admin's config-only builtin default-plugin onboarding semantics
  - industrial-standard passthrough remains explicit and unchanged
  - shared family normalization stays the source of truth, so stateless config no longer forks provider identity policy

## Stateful Flow

1. preview routed provider without decision-log side effects
2. if `provider.protocol_kind` matches requested standard, persist one routing decision log and relay raw HTTP
3. otherwise, if the planned provider resolves to a raw-plugin-capable runtime, persist one routing decision log and execute the plugin-owned raw contract
4. if explicit runtime binding degraded to `local_fallback`, persist one routing decision log and return fail-closed gateway error
5. otherwise persist one routing decision log and execute translated runtime from the same planned provider
6. record usage with the same planned usage context

## Why This Shape

- protocol choice belongs to provider metadata, not to runtime adapter identity
- HTTP compatibility should consume routing results, not re-implement routing policy
- auditing must stay stable: one executed request, one routing decision log, one usage record

## Current Boundaries

- direct raw plugin execution is currently native-dynamic only; connector runtimes remain HTTP-supervised by design
- connector supervision may block only on dedicated blocking workers; async compat/routing flows must not execute connector health polling inline
- raw `Ok(None)` means the raw surface was not applicable and must stay accounting-neutral: no upstream attempt/success/failure metric and no provider health snapshot
- standard `openai` / `anthropic` / `gemini` upstreams still prefer raw HTTP passthrough over plugin conversion
- heterogeneous `custom` plugins can now normalize Anthropic/Gemini standard compat routes, but any new non-standard operation family still needs an explicit plugin operation contract
- raw JSON retry is available only for explicitly classified retryable failures; current verified classifications are execution timeout and plugin-declared transient native-dynamic errors
- raw SSE startup retry uses the same explicit classification, but only before the first content type or chunk is emitted
- opaque plugin errors still do not retry, and full stream-body timeout/retry semantics are still the same as the existing OpenAI-shaped streaming model: execution policy stops at stream acquisition, not end-to-end consumer drain
- planned execution context no longer duplicates connector supervision internally; a single planned context now owns one connector runtime health adoption/probe path
- connector runtime exclusion from raw plugin execution is now guarded both by shared runtime capability helpers and by raw execution regressions, not only by architecture prose
- planned execution and direct store-relay OpenAI execution can now fail over before relay execution when the selected provider lacks a tenant credential
- policy-declared missing/unavailable providers are now handled at route-selection time by skipping them and appending `policy_candidate_unavailable`; execution-layer `gateway_execution_failover` stays reserved for providers that were actually selected and then replaced during planning/execution
- direct store-relay OpenAI execution now also fails over before relay when the selected provider only speaks another standard protocol and has no explicit conversion plugin/runtime, so incompatible standard providers cannot silently fall through to local mock behavior on commercial routes
- `/admin/providers` execution observability is side-effect free: it explains runtime/binding boundaries and per-route-family readiness without starting runtimes, probing health, or folding credential presence into the catalog view
- `/admin/providers.integration` is management metadata, not runtime readiness:
  - it explains onboarding shape
  - runtime truth still lives in `execution` / `route_readiness`
  - standard passthrough vs default-plugin vs custom-plugin now stay explicit without reintroducing protocol/runtime conflation
- `/admin/providers.credential_readiness` is optional tenant-scoped secret-presence metadata:
  - it is absent on the default global list
  - it is returned only when `tenant_id` scope is requested
  - it does not embed runtime health or live execution verdicts
- portal routing `provider_options.protocol_kind` and `provider_options.integration` are static provider metadata:
  - they explain protocol and onboarding shape for frontend/provider-choice UX
- portal routing `provider_options.credential_readiness` is additive tenant-scoped secret-presence metadata:
  - it explains whether the current workspace tenant has a provider credential configured
  - it does not embed runtime health or live readiness probes
- default-plugin normalization ownership now sits in `sdkwork-api-app-catalog`; transport layers may validate payload shape, but they should not become the source of provider-identity policy
- stateless direct ingress now also has a first-class default-plugin family contract:
  - `from_default_plugin_family(...)` / `try_with_default_plugin_upstream(...)` reuse the shared family normalization
  - same-structure builtin onboarding no longer requires manual low-level runtime identity encoding on the stateless path
- non-admin test/support fixtures should construct canonical provider identity directly; they should not reuse the old `openai/openai/openai-official` shortcut for OpenRouter-style default-plugin providers

## Next Step

Extend plugin catalog and runtime governance around the new surface:

- add more heterogeneous custom operation families only when the upstream cannot already speak a supported industrial-standard protocol
- decide whether additional builtin same-structure families should become first-class `default_plugin_family` values or whether they need a persisted onboarding field beyond current adapter-derived semantics
- if future non-admin provider-ingest surfaces are added beyond admin/stateless, decide whether they should expose `default_plugin_family` directly or stay pinned to the shared normalized identity contract
- decide whether admin/provider-catalog observability should also surface credential-readiness without mixing static catalog truth with live secret state
- decide whether admin tenant-scoped readiness should stay query-driven on `/admin/providers` or graduate into a dedicated operator endpoint if additional tenant-state dimensions are added later
- keep connector runtimes on the supervised HTTP path unless the runtime model itself changes
- keep route-selection `policy_candidate_unavailable` distinct from execution-layer `gateway_execution_failover` in future audit/reporting and policy expansion
- decide whether connector runtimes need an equivalent structured transient error contract or should stay permanently outside the raw-plugin retry surface
- if connector transient-state standardization is needed, keep it on supervised HTTP/runtime health semantics instead of widening connector runtimes onto the raw native-dynamic ABI surface
- keep raw SSE retry permanently capped at pre-first-byte startup unless product semantics intentionally expand to full stream-budget governance
- if future product requirements want cross-standard execution on OpenAI routes for Anthropic/Gemini providers, add it only through explicit plugin/runtime contracts; do not reintroduce silent local fallback or implicit standard conversion

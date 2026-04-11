# Provider Protocol / Plugin Runtime Split Review

> Date: 2026-04-09
> Scope: `sdkwork-api-app-gateway`, `sdkwork-api-interface-http`, provider protocol routing

## Closed Findings

### P0 - Compat routes could not execute explicit custom Anthropic/Gemini plugins

- root cause: extension host already had a generic `ProviderInvocation` ABI, but app gateway and HTTP compat execution were still restricted to OpenAI-shaped `ProviderRequest`
- fix: exposed raw native-dynamic JSON/SSE execution in `sdkwork-api-extension-host`, exposed raw runtime helpers in `sdkwork-api-app-gateway`, extended planned execution with resolved runtime metadata, and rewired Anthropic/Gemini compat routes to run `passthrough -> raw plugin -> translated fallback`
- result: explicit custom plugins now return provider-native Anthropic/Gemini payloads while stateful routes still keep one selected provider plan, one decision log, and one usage record

### P0 - Duplicate routing decision logs in stateful compat passthrough

- root cause: stateful Anthropic/Gemini passthrough resolved planned usage context before fallback execution, which caused an extra route selection and an extra routing decision log
- fix: introduced `planned_execution_provider_context_for_route_without_log(...)` in app gateway and used it for protocol inspection
- result: translated OpenAI-compatible paths are back to one routing decision log and one usage record

### P0 - Missing routing decision log after no-log passthrough preview

- root cause: after removing duplicate route selection, native passthrough requests no longer produced the single expected routing decision log
- fix: introduced `persist_planned_execution_decision_log(...)` and call it only when native passthrough is actually selected
- result: native passthrough paths now also produce exactly one routing decision log

### P1 - HTTP compatibility leaked routing concerns

- root cause: interface HTTP layer started carrying too much provider-resolution behavior
- fix: moved preview-plan and decision-log helpers into `sdkwork-api-app-gateway`
- result: routing semantics stay in gateway application layer; HTTP compat only branches on planned protocol

### P1 - Mixed-protocol fallback could re-select a different provider

- root cause: stateful compat passthrough preview used a no-log simulated route, but translated fallback still called legacy `*_from_store_with_execution_context` entrypoints that selected again
- fix: added planned execution entrypoints in `sdkwork-api-app-gateway` and rewired stateful Anthropic/Gemini JSON, SSE, and count-tokens handlers to execute passthrough or translated fallback from the same `PlannedExecutionProviderContext`
- result: mixed-protocol compat requests now select once, log once, and execute from one provider plan

### P0 - Protocol fallback could bypass explicit plugin binding

- root cause: after adding protocol-capability fallback, providers with a persisted extension installation or instance could silently downgrade to a builtin adapter when the explicit plugin failed discovery, trust validation, or reload
- fix: protocol/adaptor fallback now runs only when the provider has no persisted runtime binding in store
- result: explicit connector/native-dynamic/plugin bindings fail closed; only legacy or unbound providers use protocol-default fallback

### P1 - Extension dispatch tests raced on global host cache

- root cause: several extension-dispatch tests touched the shared configured extension host without joining the existing `serial(extension_env)` lock
- fix: serialized all cache- and env-sensitive extension dispatch tests under the same group
- result: extension-host cache, connector runtime, and native-dynamic reload tests are stable again under full-suite execution

### P1 - Fail-closed explicit plugin errors were invisible in routing history

- root cause: raw planned-execution helpers returned early when `execution.local_fallback == true`, so explicit broken bindings failed closed but skipped routing decision-log persistence
- fix: persist the selected planned routing decision before returning the fail-closed gateway error for raw JSON/SSE plugin execution
- result: broken explicit Anthropic/Gemini plugin bindings now fail safe and still leave one audit record for operator diagnosis

### P1 - Raw compat regressions did not yet lock stream and count-tokens plugin paths

- root cause: initial raw-plugin coverage focused on Anthropic/Gemini request-response JSON flows and missed dedicated compat regressions for stream and count-tokens branches
- fix: added interface-http regressions for Anthropic and Gemini raw native-dynamic stream/count-tokens routes, plus broken-binding fail-closed coverage that uses isolated uncached extension ids
- result: `passthrough -> raw plugin -> translated fallback` is now pinned across JSON, SSE, count-tokens, and explicit-binding failure branches

### P1 - Raw planned execution bypassed execution-context telemetry and health governance

- root cause: stateful compat raw-plugin branches called thin runtime wrappers that skipped gateway execution-context controls, upstream metrics, and provider health snapshot persistence
- fix: added governed raw planned-execution entrypoints in `sdkwork-api-app-gateway`, wrapped raw JSON in blocking timeout-aware execution context, reused execution-context gating for raw SSE startup, and rewired Anthropic/Gemini compat handlers to call those entrypoints
- result: raw-plugin execution now shares deadline/timeout/overload controls, upstream attempt/success/failure metrics, and provider health snapshots with the existing governed execution path

### P1 - Raw JSON planned execution ignored matched retry policy

- root cause: `PlannedExecutionProviderContext` carried `decision.matched_policy_id`, but raw JSON execution never converted that routing decision into a gateway retry policy, so route-configured retries were silently skipped on the raw-plugin path
- fix: derived raw JSON retry policy from the planned routing decision, reused the existing gateway retry accounting, and kept retry entry limited to explicitly classified retryable failures
- result: native-dynamic raw JSON can now recover from verified execution-context timeouts under route policy, while opaque plugin failures still fail closed without guessed retries

### P1 - Native-dynamic plugin errors were too opaque for safe retry expansion

- root cause: plugin ABI errors only exposed a free-form message, so gateway governance had no typed way to distinguish transient plugin failures from permanent business failures
- fix: added optional retry metadata to the native-dynamic ABI error envelope, preserved it in `ExtensionHostError::NativeDynamicInvocationFailed`, and taught gateway retry classification/delay selection to consume only explicit plugin retry hints
- result: plugin-declared transient raw JSON failures now retry safely under route policy with optional `retry_after_ms`, while opaque plugin failures remain non-retryable

### P1 - Raw SSE startup ignored explicit transient plugin hints

- root cause: stream governance had already expanded to startup/acquisition telemetry, but the raw SSE path still short-circuited on the first startup failure and never consumed the same explicit retry classification used by raw JSON
- fix: derived raw SSE retry policy from the matched planned routing decision, reused the same retryable classification and retry-delay selection, and kept the retry loop strictly outside the point where the first content type or chunk is emitted
- result: native-dynamic raw SSE can now recover from explicit transient startup failures under route policy without changing the existing no-retry contract after stream output begins

### P1 - Raw-plugin fallthrough polluted metrics and provider health

- root cause: governed raw JSON/SSE helpers recorded success telemetry and persisted healthy snapshots on `Ok(response)` before distinguishing `Some(payload)` from `None`, and they did not explicitly short-circuit non-`native_dynamic` runtimes
- fix: raw planned execution now exits early for non-`native_dynamic` runtimes and only records attempt/success/health after confirming `Some(response)`
- result: raw `Ok(None)` is now accounting-neutral control fallthrough, so translated fallback can continue without phantom upstream success or false provider-health recovery signals

### P1 - Connector startup supervision blocked current-thread compat planning

- root cause: planned provider resolution called blocking connector startup/health polling inline on the async request executor; under current-thread runtimes, that could starve spawned connector health servers and collapse stateful Anthropic/Gemini translated fallback into false missing-entrypoint failures
- fix: connector startup supervision now runs through `spawn_blocking` during planned provider resolution, and connector startup can adopt delayed external health when no local filesystem entrypoint exists
- result: connector-backed Anthropic/Gemini translated fallback now plans successfully without breaking the architecture rule that connector runtimes stay on the supervised HTTP path outside raw native-dynamic execution

### P1 - Planned execution context probed connector health twice

- root cause: `planned_execution_provider_context_for_route_without_log(...)` resolved connector execution metadata first and then called usage-context construction that independently resolved the same provider execution target again
- fix: planned execution context now resolves one `ProviderExecutionTarget` and reuses it while building usage context
- result: one planned connector execution context now performs one connector supervision/probe sequence, which removes duplicate external `/health` checks without changing passthrough/raw-plugin/runtime-boundary rules

### P1 - Raw plugin boundary was still partly encoded as scattered runtime equality checks

- root cause: the architecture rule "only native-dynamic enters the raw plugin surface" was true in behavior, but several entrypoints still relied on repeated `runtime == NativeDynamic` checks instead of one shared runtime capability contract
- fix: added `ExtensionRuntime` capability helpers for raw execution and structured retry hints, then rewired extension-host raw runtime resolution and app-gateway raw JSON/SSE guards to use that contract
- result: connector and builtin runtimes stay explicitly outside the raw plugin surface by shared capability rule, and connector raw fallthrough is now pinned by dedicated accounting-neutral regressions

### P1 - Planned compat execution could abort before failover when selected connector failed during context construction

- root cause: stateful compat routes build `PlannedExecutionProviderContext` before relay execution starts, and connector runtime startup/health adoption happens inside that planning phase; if the selected connector could not produce an execution target, the request failed before the existing execution-layer failover loop could try the backup provider
- fix: planned execution context construction now checks route failover policy, keeps existing missing-provider/missing-secret semantics unchanged, and only widens failover for runtime-target resolution failures; when backup selection happens during planning, the planned decision rewrites `selected_provider_id` and appends `gateway_execution_failover`
- result: connector startup failures now participate in stateful planned failover without widening connector runtimes onto the raw plugin surface or desynchronizing routing log / usage context from the provider that actually executed

### P1 - Missing-credential preflight failover was not pinned end to end on standard compat routes

- root cause: app-gateway planned execution was widened to fail over when the selected provider lacks a tenant credential, but interface-http compat coverage still only locked runtime-failure failover and broken-binding cases
- fix: added an end-to-end Anthropic compat regression where the ordered primary provider lacks the tenant credential, the backup provider has both credential and executable target, and the request remains native Anthropic passthrough on the backup provider
- result: the contract is now executable and audited end to end: the primary upstream is never touched, the backup upstream receives the request, and both usage record and routing decision log point to the backup provider with `gateway_execution_failover`

### P1 - Direct store-relay OpenAI chat/responses diverged from planned preflight failover

- root cause: direct store-relay `chat/responses` resolved the selected provider secret inline and returned `None` immediately when the selected provider lacked the tenant credential, so the request dropped to local fallback before the execution failover path could try the backup provider
- fix: added a shared store-relay provider-resolution helper that reuses the selected-provider missing-tenant-credential preflight failover rule, rewrites the effective routing decision to the executable backup provider, and records failover decision logs/usage against the provider that actually executed
- result: stateful OpenAI chat/responses JSON and SSE now match planned compat semantics: the primary upstream is skipped before execution, the backup provider executes directly, and routing/usage audit stays truthful

### P1 - Missing-provider route skip was invisible in fallback audit

- root cause: route selection already filtered policy-declared providers that were absent from the current available candidate set, so backup execution succeeded, but `RoutingDecision.fallback_reason` stayed empty and operators could not see that a declared primary candidate had been skipped before execution
- fix: `route_selection` now detects policy-declared unavailable candidates and appends `policy_candidate_unavailable`, while preserving `gateway_execution_failover` for real planning/execution replacement
- result: routing-layer candidate unavailability is now explicitly auditable across direct routing, planned execution, and stateful `chat/responses` flows without widening missing-provider into execution failover

### P0 - OpenAI routes could silently fabricate local success after selecting an incompatible standard provider

- root cause: direct store-relay OpenAI `chat/responses` execution treated "selected provider has credential but no executable OpenAI adapter surface" as `Ok(None)` instead of execution failure
- trigger shape:
  - ordered routing selected a provider with `protocol_kind=anthropic|gemini`
  - no explicit conversion plugin/runtime existed for OpenAI-shaped execution
  - relay execution returned empty
  - HTTP route fell through to local mock/local fallback and could record usage against fabricated local success
- fix: app-gateway provider-adapter execution now treats missing executable adapter as execution failure while preserving explicit `local_fallback` semantics for the real fail-closed/builtin-local paths
- result:
  - incompatible standard providers are skipped through the normal execution failover path
  - backup OpenAI-compatible providers execute for both JSON and SSE on `chat` and `responses`
  - primary incompatible providers are never contacted
  - usage record and routing decision log now remain aligned with the executed backup provider instead of local mock fallback

### P1 - Admin provider catalog hid executability boundaries after the plugin/runtime split

- root cause: `/admin/providers` still exposed only static provider metadata, so operators had to manually correlate runtime bindings, extension runtime status, and fail-closed behavior from multiple endpoints
- fix: added `ProviderExecutionView` in `sdkwork-api-app-gateway` and reused it in `/admin/providers` as an additive `execution` object driven by the same provider-resolution rules as runtime execution
- result:
  - implicit-default standard passthrough providers are visible as such
  - explicit native-dynamic plugins expose raw-plugin and adapter-surface capability truthfully
  - explicit broken bindings now show `fail_closed=true` instead of disappearing behind static metadata
  - the management view is side-effect free and does not start runtimes while explaining execution boundaries

### P1 - Global `fail_closed` could overstate real provider unavailability

- root cause: the first admin execution view treated `fail_closed` as the main operator verdict, but a provider can still serve its matching industrial-standard route family through direct passthrough even when adapter/raw-plugin execution is broken
- fix: added `execution.route_readiness.openai|anthropic|gemini`, each with `ready` plus `mode={provider_adapter|standard_passthrough|raw_plugin|fail_closed|unavailable}`
- result:
  - explicit broken Anthropic bindings now still show `anthropic=standard_passthrough`
  - heterogeneous native-dynamic plugins now distinguish matching-standard passthrough from raw conversion readiness
  - operators can read route-family truth without forcing credential/live-health state into the static provider catalog

### P1 - Ollama default-plugin providers were mislabeled as standard OpenAI protocol

- root cause: domain-level and stateless-config protocol derivation still mapped `ollama` / `ollama-compatible` to `openai`, while builtin runtime truth already declared `sdkwork.provider.ollama` as `protocol=custom`
- fix: changed the shared derivation rule so `ollama* => custom`, while keeping `openrouter* => openai`
- result:
  - Ollama remains config-only onboardable through the default plugin adapter
  - Ollama is no longer misreported as industrial-standard passthrough-capable
  - "same-structure default plugin" and "matching standard-protocol passthrough" now stay cleanly separated

### P1 - Admin provider onboarding still hid the default-plugin path behind low-level adapter fields

- root cause:
  - `/admin/providers` creation still required operators to express builtin same-structure onboarding through `adapter_kind`
  - `/admin/providers` listing exposed runtime executability but not the higher-level onboarding mode
  - `openrouter` and `ollama` therefore looked too close to either official-standard passthrough or custom heterogeneous plugin binding
- fix:
  - added additive `default_plugin_family` support on `POST /admin/providers`
  - current first-class builtin families are `openrouter` and `ollama`
  - default-family requests now pin the canonical builtin `adapter_kind`, derived `protocol_kind`, and derived `extension_id`
  - conflicting low-level fields now fail fast instead of silently drifting into another integration mode
  - added additive `/admin/providers.integration` with `mode={standard_passthrough|default_plugin|custom_plugin}` plus `default_plugin_family` when applicable
- result:
  - operators can now onboard builtin same-structure providers without reasoning from runtime internals
  - standard passthrough, builtin default-plugin, and custom-plugin paths are now explicit in management APIs
  - runtime truth still stays in `/admin/providers.execution`, so onboarding semantics and execution semantics no longer compete for the same field

### P1 - Default-plugin identity policy still lived only in the admin controller

- root cause:
  - after the initial admin onboarding fix, `default_plugin_family` normalization still existed only in the HTTP layer
  - application-layer callers had no shared provider-identity normalization contract to reuse
  - admin routing/model tests still contained legacy OpenRouter fixtures created through `adapter_kind=openai`, which preserved the old conflated example path
- fix:
  - moved default-plugin identity normalization into `sdkwork-api-app-catalog`
  - added application-level regressions for default-plugin creation, canonical identity derivation, and conflict rejection
  - rewired admin provider creation to consume the shared app-catalog normalization contract
  - updated admin routing/model fixtures to create OpenRouter through `default_plugin_family=openrouter`
- result:
  - provider onboarding policy is now owned by the application layer instead of one transport
  - test fixtures now reinforce the intended plugin architecture
  - future control-plane entrypoints can reuse one normalization contract instead of cloning controller logic

### P1 - Non-admin regressions still encoded OpenRouter as official OpenAI runtime

- root cause:
  - after admin and app-catalog normalization were fixed, app-routing simulate-route helpers still created `provider-openrouter` through `channel_id=openai`, `adapter_kind=openai`
  - app-gateway routing relay regression still bound `provider-openrouter` to `sdkwork.provider.openai.official`
  - those regressions verified routing outcomes, but they no longer verified the intended default-plugin identity contract
- fix:
  - added explicit OpenRouter fixture construction in app-routing support with:
    - primary `channel_id=openrouter`
    - `adapter_kind=openrouter`
    - derived `protocol_kind=openai`
    - secondary `openai` channel binding
  - rewired simulate-route and geo/profile regressions to use that canonical fixture
  - rewired the relay routing-policy regression to install and bind `sdkwork.provider.openrouter`
  - added direct identity assertions for `adapter_kind`, `protocol_kind`, and `extension_id`
- result:
  - non-admin regressions now enforce the same provider-identity contract as admin/control-plane surfaces
  - OpenRouter remains standard-`openai` protocol but no longer masquerades as official OpenAI runtime identity
  - future routing or relay changes now fail fast if they drift back to the old `openai/openai/openai-official` shortcut

### P1 - Stateless gateway protocol truth still forked from shared provider identity rules

- root cause:
  - provider protocol derivation had already been centralized in `sdkwork-api-domain-catalog`
  - `sdkwork-api-interface-http::StatelessGatewayUpstream` still carried its own duplicate derive/normalize table
  - future builtin-family protocol changes could therefore have updated admin/storage truth without updating stateless HTTP truth
- fix:
  - rewired stateless upstream construction to reuse `sdkwork-api-domain-catalog::{derive_provider_protocol_kind, normalize_provider_protocol_kind}`
  - added stateless regression coverage that explicitly pins `openrouter => openai` next to the existing Ollama `custom` case
- result:
  - stateless and stateful provider metadata now consume one protocol-truth source
  - new default-plugin families or protocol corrections no longer require a second mapping change inside `gateway_auth.rs`

### P2 - Portal and HTTP relay samples still taught low-level same-structure onboarding

- root cause:
  - portal routing tests still inserted OpenRouter through the old `channel_id=openai`, `adapter_kind=openai` shortcut
  - HTTP relay regressions still created OpenRouter/Ollama through raw `adapter_kind` payloads instead of the explicit default-plugin family input
  - runtime behavior was already correct, but sample/test surfaces still taught the wrong integration shape
- fix:
  - portal routing fixtures now construct canonical OpenRouter identity with primary `openrouter` channel plus secondary `openai` binding
  - HTTP relay regressions now create OpenRouter and Ollama through `default_plugin_family` and assert the returned derived identity fields
- result:
  - control-plane, stateless, portal, and relay examples now all reinforce the same rule:
    - industrial-standard providers pass through directly
    - builtin same-structure providers onboard through the default plugin family
    - only incompatible APIs require explicit conversion plugins

### P2 - Default-plugin fixture seeding still duplicated canonical identity outside admin

- root cause:
  - `sdkwork-api-app-catalog` already normalized default-plugin identity, but it had no shared constructor for providers that also needed secondary channel bindings
  - app-routing support/tests, app-gateway relay coverage, and portal routing coverage therefore still rebuilt canonical OpenRouter identity inline
- fix:
  - added `create_provider_with_default_plugin_family_and_bindings(...)` to `sdkwork-api-app-catalog`
  - rewired the existing default-plugin constructor to reuse that helper
  - migrated the high-value non-admin fixtures to shared app-catalog constructors, and reused `create_provider_with_config(...)` for adjacent standard OpenAI fixtures
- result:
  - default-plugin seeding now stays application-owned even when secondary bindings such as `openai` are required
  - high-value tests no longer teach hand-built OpenRouter identity
  - future non-admin seeders can reuse one helper instead of duplicating provider identity truth

### P2 - Admin create response still hid onboarding mode after default-plugin normalization

- root cause:
  - `/admin/providers` list had already grown additive `integration`
  - `POST /admin/providers` still returned only the flattened `ProxyProvider`
  - control-plane callers had to issue a second catalog read to discover whether the created provider was `standard_passthrough`, `default_plugin`, or `custom_plugin`
- fix:
  - added additive `ProviderCreateResponse.integration`
  - kept `execution` out of the create response so mutation semantics still stay separate from runtime observability
  - derived create-time integration from the same shared provider metadata truth as the list response
- result:
  - create-time onboarding truth is immediately visible
  - list-time runtime truth remains additive and side-effect free
  - management APIs no longer force a read-after-write roundtrip just to classify onboarding mode

### P2 - Portal routing provider options hid protocol/plugin semantics from frontend surfaces

- root cause:
  - portal routing summary provider options only exposed `provider_id`, `display_name`, and `channel_id`
  - frontend/provider-choice UX had to infer industrial-standard passthrough vs default-plugin vs custom-plugin shape from incomplete metadata
  - provider-integration classification still lived effectively inside the admin transport layer
- fix:
  - moved reusable `ProviderIntegrationView` derivation into `sdkwork-api-app-catalog`
  - rewired admin create/list responses to consume the shared helper
  - extended portal routing provider options with additive `protocol_kind` and `integration`
- result:
  - admin and portal now read one shared onboarding-truth contract
  - frontend/provider-choice surfaces can honor the industrial-standard-first rule without guessing from runtime internals
  - portal visibility stays static and does not mix in credential or live-health state

### P2 - Portal provider choice still lacked tenant-scoped credential readiness

- root cause:
  - after adding `protocol_kind` and `integration`, portal still only exposed static provider metadata
  - frontend/provider-choice UX could not tell whether the current workspace tenant had a credential configured for each provider
  - pushing that state into `integration` would have conflated onboarding classification with secret presence
- fix:
  - added additive `provider_options.credential_readiness`
  - current contract is `ready` plus `state={configured|missing}`
  - readiness is derived from tenant-scoped credential presence and remains separate from `protocol_kind`, `integration`, and admin `execution`
- result:
  - portal can now distinguish provider shape from provider readiness for the current tenant
  - static provider metadata remains transport-stable
  - secret presence is visible without introducing runtime-health side effects

### P2 - Admin provider catalog still lacked a tenant-scoped credential view

- root cause:
  - `/admin/providers` exposed static onboarding truth and runtime execution truth only
  - operators could not answer "is tenant X credentialed for this provider" from the provider catalog surface
  - adding tenant readiness to every default response would have made the global catalog tenant-relative by accident
- fix:
  - added query-scoped tenant readiness on `GET /admin/providers?tenant_id=...`
  - kept readiness absent on the default unscoped list
  - moved shared readiness derivation into `sdkwork-api-app-credential`, then rewired both admin and portal to consume that app-layer contract
- result:
  - admin catalog now supports explicit tenant-scoped secret-presence inspection without weakening its default global meaning
  - portal/admin no longer duplicate readiness classification
  - `integration`, `execution`, and `credential_readiness` remain three separate concerns

### P2 - Admin provider readiness contract was not published in OpenAPI

- root cause:
  - runtime already implemented `/admin/providers` plus optional `tenant_id` scoping
  - admin OpenAPI still did not publish provider catalog routes, request bodies, or response schemas
  - operator docs and generated clients could therefore miss that `credential_readiness` is an explicit tenant-scoped overlay rather than implicit global provider state
- fix:
  - added admin OpenAPI `catalog` docs for `GET /admin/providers` and `POST /admin/providers`
  - documented `tenant_id` as an opt-in query scope
  - added schema support on the real shared provider/integration/execution/readiness types instead of creating transport-local documentation-only copies
- result:
  - admin docs now expose the same three-way semantic split as runtime code
  - provider-management clients can consume the contract without hidden route knowledge
  - unscoped global catalog truth stays distinct from tenant-scoped readiness overlays

### P2 - Tenant-scoped provider readiness still lacked a dedicated boundary

- root cause:
  - after publishing `/admin/providers?tenant_id=...`, tenant readiness was still only available as a global catalog overlay
  - future tenant-state dimensions would have to keep widening the provider catalog response
  - operator tooling had no focused tenant/provider inventory endpoint
- fix:
  - kept the query-scoped overlay for convenience
  - added `GET /admin/tenants/{tenant_id}/providers/readiness`
  - dedicated response carries only:
    - provider identity for display/joining
    - static `integration`
    - tenant-scoped `credential_readiness`
  - left `execution` on the global provider catalog surface
- result:
  - tenant-scoped provider readiness now has its own extensible admin boundary
  - global runtime truth is no longer the mandatory carrier for tenant overlays
  - the semantic split stays explicit even as operator UX gains a more focused route

### P2 - Stateless default-plugin ingress still required raw runtime-key encoding

- root cause:
  - admin create already exposed `default_plugin_family`
  - stateless direct upstream configuration still required same-structure builtin providers to be expressed through low-level `runtime_key` / adapter semantics
  - non-admin config-only onboarding therefore did not fully match the intended plugin-family contract
- fix:
  - added `StatelessGatewayUpstream::from_default_plugin_family(...)`
  - added `StatelessGatewayConfig::try_with_default_plugin_upstream(...)`
  - both paths now reuse shared domain-catalog family normalization and reject unsupported families explicitly
- result:
  - stateless OpenRouter/Ollama onboarding now matches admin's config-only default-plugin semantics
  - callers no longer need to hand-encode low-level runtime identity for the common same-structure builtin path
  - explicit advanced constructors remain available without reintroducing policy drift

## Verification

- `cargo test -p sdkwork-api-domain-catalog provider_derives_protocol_kind_from_adapter_kind -- --nocapture`
- `cargo test -p sdkwork-api-domain-catalog provider_normalizes_default_plugin_families_for_builtin_nonstandard_families -- --nocapture`
- `cargo test -p sdkwork-api-interface-http --test stateless_upstream_protocol -- --nocapture`
- `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes create_provider_accepts_default_plugin_family_for_openrouter -- --nocapture`
- `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes create_provider_accepts_default_plugin_family_for_ollama -- --nocapture`
- `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes create_provider_rejects_conflicting_default_plugin_family_and_adapter_kind -- --nocapture`
- `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes list_providers_exposes_implicit_standard_passthrough_execution_view -- --nocapture`
- `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes list_providers_exposes_native_dynamic_raw_plugin_execution_view -- --nocapture`
- `cargo test -p sdkwork-api-app-catalog --test create_provider -- --nocapture`
- `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes routing -- --nocapture`
- `cargo test -p sdkwork-api-app-routing --test simulate_route route_simulation_uses_catalog_model_candidates -- --nocapture`
- `cargo test -p sdkwork-api-app-gateway --test routing_policy_dispatch relay_chat_completion_honors_routing_policy_provider_order -- --nocapture`
- `cargo test -p sdkwork-api-app-routing --test simulate_route -- --nocapture`
- `cargo test -p sdkwork-api-app-gateway --test routing_policy_dispatch -- --nocapture`
- `cargo check -p sdkwork-api-app-routing -p sdkwork-api-app-gateway`
- `cargo check -p sdkwork-api-domain-catalog -p sdkwork-api-interface-http`
- `cargo test -p sdkwork-api-interface-http --test stateless_upstream_protocol -- --nocapture`
- `cargo test -p sdkwork-api-interface-portal --test portal_routing -- --nocapture`
- `cargo check -p sdkwork-api-app-catalog -p sdkwork-api-app-routing -p sdkwork-api-app-gateway -p sdkwork-api-interface-portal`
- `cargo check -p sdkwork-api-interface-http -p sdkwork-api-interface-portal`
- `cargo test -p sdkwork-api-app-gateway --test extension_dispatch planned_execution_context_fails_over_when_selected_connector_cannot_start -- --nocapture`
- `cargo test -p sdkwork-api-interface-http --test anthropic_messages_route stateful_anthropic_messages_route_fails_over_before_execution_when_primary_lacks_tenant_credential -- --nocapture`
- `cargo test -p sdkwork-api-extension-core --test manifest_contract runtime_capability_helpers_keep_connector_off_raw_plugin_surface -- --nocapture`
- `cargo test -p sdkwork-api-app-gateway --test raw_execution_governance connector_runtime -- --nocapture`
- `cargo test -p sdkwork-api-app-gateway --test extension_dispatch planned_execution_context_probes_external_connector_health_once -- --nocapture`
- `cargo test -p sdkwork-api-app-gateway --test planned_execution -- --nocapture`
- `cargo test -p sdkwork-api-app-gateway --test extension_dispatch -- --nocapture`
- `cargo test -p sdkwork-api-extension-host`
- `cargo test -p sdkwork-api-extension-host --test connector_runtime -- --nocapture`
- `cargo test -p sdkwork-api-app-gateway`
- `cargo test -p sdkwork-api-app-gateway --test raw_execution_governance -- --nocapture`
- `cargo test -p sdkwork-api-app-gateway --test raw_execution_governance none_is_accounting_neutral -- --nocapture`
- `cargo test -p sdkwork-api-interface-http --test anthropic_messages_route native_dynamic_raw_plugin -- --nocapture`
- `cargo test -p sdkwork-api-interface-http --test gemini_generate_content_route native_dynamic_raw_plugin -- --nocapture`
- `cargo test -p sdkwork-api-interface-http --test anthropic_messages_route stateful_anthropic_messages_stream_route_prefers_native_dynamic_raw_plugin_for_explicit_custom_runtime_and_records_usage -- --nocapture`
- `cargo test -p sdkwork-api-interface-http --test gemini_generate_content_route stateful_gemini_stream_generate_content_route_prefers_native_dynamic_raw_plugin_for_explicit_custom_runtime_and_records_usage -- --nocapture`
- `cargo test -p sdkwork-api-interface-http --test anthropic_messages_route`
- `cargo test -p sdkwork-api-interface-http --test gemini_generate_content_route`
- `cargo test -p sdkwork-api-app-routing --test simulate_route -- --nocapture`
- `cargo test -p sdkwork-api-interface-http stateful_chat_route_fails_over_before_execution_when_primary_requires_non_openai_standard_without_plugin -- --nocapture`
- `cargo test -p sdkwork-api-interface-http stateful_chat_stream_route_fails_over_before_execution_when_primary_requires_non_openai_standard_without_plugin -- --nocapture`
- `cargo test -p sdkwork-api-interface-http stateful_responses_route_fails_over_before_execution_when_primary_requires_non_openai_standard_without_plugin -- --nocapture`
- `cargo test -p sdkwork-api-interface-http stateful_responses_stream_route_fails_over_before_execution_when_primary_requires_non_openai_standard_without_plugin -- --nocapture`
- `cargo test -p sdkwork-api-interface-http --test chat_route -- --nocapture`
- `cargo test -p sdkwork-api-interface-http --test responses_route -- --nocapture`
- `cargo test -p sdkwork-api-interface-admin list_providers_exposes_implicit_standard_passthrough_execution_view -- --nocapture`
- `cargo test -p sdkwork-api-interface-admin list_providers_exposes_native_dynamic_raw_plugin_execution_view -- --nocapture`
- `cargo test -p sdkwork-api-interface-admin list_providers_exposes_fail_closed_execution_view_for_broken_explicit_binding -- --nocapture`
- `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes providers_models_coupons -- --nocapture`
- `cargo test -p sdkwork-api-interface-portal --test portal_routing portal_routing_preferences_preview_and_logs_are_project_scoped -- --nocapture`
- `cargo test -p sdkwork-api-interface-portal --test portal_routing -- --nocapture`
- `cargo check -p sdkwork-api-interface-portal`
- `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes list_providers_exposes_tenant_scoped_credential_readiness_only_when_requested -- --nocapture`
- `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes providers_models_coupons -- --nocapture`
- `cargo test -p sdkwork-api-app-credential -- --nocapture`
- `cargo check -p sdkwork-api-app-credential -p sdkwork-api-interface-admin -p sdkwork-api-interface-portal`
- `cargo test -p sdkwork-api-interface-admin --test openapi_route openapi_routes_expose_admin_api_inventory_with_schema_components -- --nocapture`
- `cargo check -p sdkwork-api-domain-catalog -p sdkwork-api-app-catalog -p sdkwork-api-app-gateway -p sdkwork-api-app-credential -p sdkwork-api-interface-admin`
- `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes list_tenant_provider_readiness_exposes_focused_tenant_overlay_inventory -- --nocapture`
- `cargo check -p sdkwork-api-interface-admin -p sdkwork-api-app-credential -p sdkwork-api-app-catalog -p sdkwork-api-domain-catalog`
- `cargo check -p sdkwork-api-app-gateway -p sdkwork-api-interface-admin`
- `cargo check -p sdkwork-api-app-catalog -p sdkwork-api-interface-admin -p sdkwork-api-interface-portal`
- `cargo check -p sdkwork-api-app-gateway -p sdkwork-api-interface-http`
- `cargo check -p sdkwork-api-ext-provider-native-mock -p sdkwork-api-app-gateway -p sdkwork-api-interface-http`
- `cargo check -p sdkwork-api-extension-host -p sdkwork-api-app-gateway -p sdkwork-api-interface-http`
- `cargo check -p sdkwork-api-interface-http`

## Residual Risk

### P2 - Raw plugin execution is intentionally limited to native-dynamic runtimes

- current state: direct raw plugin execution now exists for native-dynamic runtimes, while connector runtimes stay on the supervised HTTP path
- risk: future plugin authors may assume connector runtimes can execute ABI-native raw operations when they currently cannot
- required next fix: keep connector vs native-dynamic runtime boundaries explicit and only add connector-side raw execution if the runtime supervision model is intentionally redesigned

### P2 - Streaming governance still ends at stream acquisition

- current state: raw SSE now retries explicitly classified startup failures before first metadata/chunk, but raw SSE and existing OpenAI-shaped SSE still apply execution policy only while acquiring the stream, not while the caller drains the body
- risk: operators may assume `request_timeout_ms` governs full stream lifetime when it currently governs startup only
- required next fix: decide whether the product needs explicit end-to-end stream budget enforcement or whether startup-only governance remains the intended contract

### P2 - Raw retry taxonomy is still intentionally narrow

- current state: raw JSON and raw SSE startup now retry only for explicitly classified retryable failures; verified production paths are execution-context timeout and plugin-declared transient native-dynamic errors, while connector runtimes still stay outside this surface
- risk: operators may over-assume parity between native-dynamic raw execution and connector runtimes, or assume retry continues after stream output begins
- required next fix: keep connector vs native-dynamic boundaries explicit and only widen retry beyond startup if product semantics intentionally change

### P2 - Standalone usage-context callers still own their own connector supervision path

- current state: planned execution context no longer duplicates connector supervision internally, but any separate caller that requests planned usage context in isolation still performs its own provider-resolution path
- risk: there is no correctness gap on the main compat execution path anymore, but isolated non-execution callers can still pay their own connector health probe cost
- required next fix: only add broader request-scoped memoization if profiling shows a real cross-call overhead outside the existing planned execution flow

### P2 - Planned preflight failover is intentionally narrower than full execution failover

- current state: planned compat and direct store-relay OpenAI execution now both cover selected-provider missing tenant credential, while policy-declared missing/unavailable providers are skipped earlier at route selection and audited as `policy_candidate_unavailable`
- risk: operators or future code changes could collapse routing-layer skip reasons and execution-layer failover reasons into one generic fallback bucket
- required next fix: preserve the explicit boundary between `policy_candidate_unavailable` and `gateway_execution_failover` if dashboards, policy controls, or new preflight failover classes expand

### P2 - Provider catalog execution view is intentionally not a live credential/health verdict

- current state: `/admin/providers` now explains execution surfaces, fail-closed binding boundaries, and per-route-family readiness, but it does not embed tenant-credential presence or live runtime health
- risk: operators may overread the catalog response as a full readiness verdict instead of combining it with runtime-status and provider-health endpoints
- required next fix: decide whether to add explicit credential-readiness and route-family-specific execution-readiness summaries without mixing static catalog truth with secret-state or side-effectful health probes

### P2 - First-class default-plugin onboarding currently covers only builtin families with distinct runtime identity

- current state: admin now exposes `default_plugin_family`, but the first-class builtins are intentionally limited to `openrouter` and `ollama`
- risk: future same-structure builtin families may still need operators to infer onboarding semantics from `adapter_kind` until their family contract is made explicit
- required next fix: decide whether more builtin same-structure families deserve explicit `default_plugin_family` support or whether that requires a persisted onboarding field beyond current adapter-derived semantics

### P2 - Admin and stateless ingress now share the family contract, but future non-admin surfaces may still drift

- current state: admin and stateless ingress now both expose first-class default-plugin-family onboarding, while other future provider-ingest paths may still choose between explicit family input and the normalized helper contract
- risk: a new ingestion entrypoint could reintroduce raw `adapter_kind` shortcuts if it bypasses the shared family policy
- required next fix: when future non-admin entrypoints appear, either expose `default_plugin_family` explicitly or pin them to the normalized helper contract; do not invent a third provider-family shape

### P2 - Portal credential readiness is secret-presence only, not live runtime readiness

- current state: portal routing provider options now expose static `protocol_kind`/`integration` plus tenant-scoped `credential_readiness`
- risk: frontend/operator UX could still overread `credential_readiness.configured` as a live runtime-health or execution-success verdict
- required next fix: keep runtime health, connector liveness, and route-family execution truth separate from credential presence if portal readiness expands

### P2 - Admin tenant-scoped readiness is intentionally query-scoped, not global catalog truth

- current state: `/admin/providers` now returns tenant-scoped `credential_readiness` only when `tenant_id` is supplied
- risk: future callers could treat the scoped response as the default provider catalog contract and accidentally reintroduce tenant-relative semantics into global operator flows
- required next fix: if additional tenant-scoped provider state is needed, decide whether `/admin/providers` should keep expanding query-scoped overlays or whether a dedicated tenant/provider readiness endpoint becomes clearer

## Recommendation

Keep the next iteration plugin-runtime centric:

- one provider manifest declares runtime family and wire protocol separately
- one resolver uses safe protocol fallback only for unbound providers
- one explicit execution path now exists for standard Anthropic/Gemini compat routes on heterogeneous plugins
- add retry only where raw-plugin failure classes can be proven safe to retry; current verified set is execution-context timeout plus explicit plugin transient hints
- keep connector optimization on the supervised HTTP path; do not widen connector runtimes onto the raw native-dynamic execution surface just to chase retry symmetry
- keep planned preflight failover narrow and explicit; route-selection `policy_candidate_unavailable` should remain separate from execution-layer `gateway_execution_failover`
- keep `/admin/providers` as a side-effect-free execution-boundary and route-readiness view unless a future operator requirement proves that credential/live-health aggregation belongs on the same surface
- keep tenant-scoped admin readiness explicitly query-driven for now; do not silently widen the default global provider catalog into a tenant-relative view
- keep portal/provider-choice metadata split into:
  - shared static onboarding truth
  - separate tenant-scoped credential-readiness truth
- if portal readiness expands again, add runtime/live-health as another separate contract instead of widening `integration` or `credential_readiness`
- add new plugin-owned operation families only when the upstream standard is genuinely incompatible
- grow `default_plugin_family` only for builtin families whose identity and protocol semantics stay unambiguous; do not re-collapse same-structure default-plugin onboarding back into raw `adapter_kind` strings
- keep stateless ingress aligned with the same rule:
  - existing stateless default-plugin shortcuts should remain the config-only path for builtin same-structure families
  - future non-admin ingress should either expose the same family contract or reuse the shared normalization helper directly
- keep provider identity normalization in app/domain layers; transport layers may validate and shape requests, but they should not become the source of plugin-family policy
- if OpenAI-route cross-standard execution is ever expanded further, require explicit plugin/runtime contracts and keep "silent local fallback after incompatible provider selection" forbidden

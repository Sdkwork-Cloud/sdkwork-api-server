# 2026-04-09 Provider Protocol / Plugin Runtime Split Step Update

## Completed

### Step 1 - Metadata split

- separated `protocol_kind` from `adapter_kind`
- persisted it through domain, catalog, admin API, SQLite, PostgreSQL, stateless upstream config

### Step 2 - Standard protocol passthrough

- implemented stateless Anthropic and Gemini passthrough
- implemented stateful Anthropic and Gemini passthrough
- added native usage extraction for passthrough responses
- fixed stateful duplicate routing decision logs
- restored exactly one routing decision log for native passthrough

### Step 3 - Unified planned execution

- added planned execution entrypoints for chat JSON, chat SSE, and count-tokens in `sdkwork-api-app-gateway`
- rewired stateful Anthropic/Gemini compat handlers to obtain one planned provider context
- native passthrough and translated fallback now execute from the same selected provider plan
- closed the mixed-protocol non-deterministic re-selection gap in stateful compat

### Step 4 - Protocol capability contract and safe fallback

- extended extension runtime protocol contract to canonical `openai`, `anthropic`, `gemini`, `custom`
- preserved backward-compatible manifest parsing for legacy `openrouter` and `ollama` values via canonical capability normalization
- added explicit protocol capability metadata to builtin runtime manifests
- added safe runtime fallback from unresolved `extension_id` to canonical `protocol_kind` only for providers without persisted plugin bindings
- prevented explicit connector/native-dynamic/plugin bindings from silently degrading to builtin defaults when discovery, trust, or reload fails
- stabilized extension dispatch integration tests by serializing global host-cache and env-sensitive cases

### Step 5 - Raw plugin execution for standard compat routes

- exposed raw native-dynamic JSON/SSE execution in `sdkwork-api-extension-host`
- exposed raw runtime helpers in `sdkwork-api-app-gateway`
- extended `PlannedExecutionProviderContext` with resolved runtime execution metadata
- wired Anthropic and Gemini compat routes to run:
  1. standard passthrough
  2. raw plugin execution
  3. translated OpenAI fallback
- added fail-closed behavior for stateful routes when an explicitly bound runtime resolves to `local_fallback`
- persisted one routing decision log before fail-closed raw-plugin errors, so explicit broken bindings remain auditable
- extended native mock fixtures and signed test manifests to support Anthropic/Gemini raw operations
- expanded interface-http regressions to cover raw native-dynamic stream and count-tokens compat paths for Anthropic and Gemini

### Step 6 - Raw execution governance refinement

- added governed planned-execution entrypoints for raw JSON and raw SSE in `sdkwork-api-app-gateway`
- raw planned execution now shares gateway execution-context controls:
  1. deadline rejection
  2. request timeout
  3. local in-flight overload protection
- raw planned execution now emits the same upstream attempt/success/failure metrics and provider health snapshots used by the existing OpenAI-shaped path
- raw JSON execution now uses a blocking execution-context wrapper, so native-dynamic ABI calls can time out without bypassing gateway telemetry
- rewired stateful Anthropic/Gemini compat handlers to use the governed raw execution entrypoints instead of thin direct runtime calls
- documented the current stream boundary explicitly: governance covers stream acquisition/startup, not full downstream body drain lifetime

### Step 7 - Conservative raw JSON retry semantics

- raw JSON planned execution now derives retry policy from the matched routing decision instead of silently ignoring route retry configuration
- raw JSON retry stays conservative: only explicitly classified retryable failures enter the retry loop
- verified execution-context timeout retry on native-dynamic raw JSON: first attempt times out, retry is scheduled, second attempt succeeds, and the final provider snapshot remains healthy
- kept raw SSE startup on the existing no-retry contract because stream governance still ends at acquisition
- extended the native mock fixture with per-invocation JSON delay sequencing so timeout-then-success retry behavior is regression-testable

### Step 8 - Explicit transient plugin error contract

- extended the native-dynamic ABI error envelope with optional structured retry hints instead of relying on free-form plugin error text
- preserved those retry hints through `sdkwork-api-extension-host`, so gateway governance can inspect typed plugin intent on `NativeDynamicInvocationFailed`
- verified native-dynamic raw JSON retries plugin-declared transient errors under route policy and uses plugin-provided `retry_after_ms` as the retry delay source
- verified opaque plugin errors still fail without retry, so plugin-owned business failures are not widened into retry loops
- kept raw SSE on the current no-retry contract even though stream error envelopes now support the same optional retry metadata for future use

### Step 9 - Raw SSE startup retry semantics

- raw SSE planned execution now derives retry policy from the matched planned routing decision instead of hard-disabling retry on the stream path
- retry remains strictly bounded to startup: the gateway may retry only before the plugin emits the first content type or chunk
- verified native-dynamic raw SSE retries plugin-declared transient startup errors under route policy and succeeds on the next attempt
- verified opaque native-dynamic raw SSE startup errors still fail without retry
- kept post-start stream drain lifetime outside retry governance, so once the stream starts flowing the existing no-retry downstream contract remains unchanged

### Step 10 - Raw runtime boundary and fallthrough neutrality

- raw planned execution now short-circuits non-`native_dynamic` runtimes before entering raw-plugin governance, so `builtin` and connector paths stay outside the raw execution surface
- raw JSON/SSE planned execution now treats `Ok(None)` as control fallthrough instead of execution success
- raw fallthrough no longer emits phantom upstream attempt/success metrics and no longer persists provider health snapshots when the raw path is not applicable
- verified the accounting-neutral contract for both raw JSON and raw SSE fallthrough, while keeping existing native-dynamic success, timeout, and retry governance green

### Step 11 - Connector supervision / translated fallback stabilization

- moved connector runtime startup supervision out of the async request path and into `spawn_blocking` during planned provider resolution
- fixed a current-thread runtime failure mode where compat planning could block the executor, prevent connector health tasks from starting, and then misclassify the provider as a missing-entrypoint connector failure
- added delayed-external-endpoint regression coverage in `sdkwork-api-extension-host`
- re-verified Anthropic/Gemini stateful connector translated fallback after the async-boundary fix
- normalized HTTP compat regressions that explicitly wait on `/health`, so their mock upstreams now expose `/health`

### Step 12 - Planned execution connector supervision dedup

- removed duplicate connector supervision inside `planned_execution_provider_context_for_route_without_log(...)`
- planned execution now resolves one connector execution target and reuses it for usage-context construction instead of probing `/health` twice for the same provider
- added app-gateway regression coverage that proves an externally managed connector endpoint is probed exactly once during planned execution context construction

### Step 13 - Runtime capability boundary codification

- added shared `ExtensionRuntime` capability helpers for:
  1. raw provider execution eligibility
  2. structured retry-hint eligibility
- rewired app-gateway raw JSON/SSE guards and extension-host raw runtime resolution to consume that shared runtime capability contract instead of repeating `native_dynamic` equality checks
- added extension-core regression coverage that pins `native_dynamic` as raw-capable and `connector`/`builtin` as raw-ineligible
- added app-gateway regressions that prove connector runtimes stay accounting-neutral on raw JSON/SSE paths

### Step 14 - Planned connector preflight failover

- fixed a pre-execution failover gap in stateful planned execution:
  1. selected provider could be routed successfully
  2. connector runtime startup / health adoption could fail during planned context construction
  3. request aborted before the existing execution-layer failover loop could try the backup provider
- planned execution context construction now:
  1. keeps existing selected-provider missing-secret / missing-provider semantics unchanged
  2. treats selected-provider runtime-target resolution failure as a failover trigger when route policy enables execution failover
  3. rewrites planned `selected_provider_id` and `fallback_reason` to the executable backup provider so usage context and persisted decision log stay truthful
- architecture boundaries remain unchanged:
  1. connector stays on the supervised HTTP path
  2. raw plugin execution stays native-dynamic only
  3. explicit fail-closed plugin-binding behavior still wins over silent fallback

### Step 15 - Missing-credential preflight failover contract

- widened planned execution context construction one step further: when route policy enables execution failover, selected-provider missing tenant credential now joins runtime-target resolution failure as a preflight failover trigger
- kept the architecture boundaries unchanged:
  1. standard `openai` / `anthropic` / `gemini` still prefer passthrough-first
  2. only incompatible upstreams still enter plugin conversion
  3. connector remains supervised HTTP-only
  4. raw plugin execution remains native-dynamic-only
- added an end-to-end Anthropic compat regression that proves:
  1. ordered routing still selects the primary provider first
  2. the primary provider is skipped before any upstream request when it lacks the tenant credential
  3. the backup provider executes the native Anthropic passthrough
  4. usage record and routing decision log both point at the backup provider with `gateway_execution_failover`

### Step 16 - Direct store-relay OpenAI preflight failover parity

- extended the same selected-provider missing-tenant-credential preflight failover contract to direct store-relay OpenAI `chat/responses` paths
- kept scope deliberately narrow:
  1. only missing tenant credential joined the preflight failover surface
  2. missing-provider semantics remain unchanged
  3. standard `openai` / `anthropic` / `gemini` still stay passthrough-first
  4. only non-standard upstreams still require plugin conversion
- added end-to-end stateful regressions that prove, for both JSON and SSE:
  1. ordered routing still picks the primary provider first
  2. the primary upstream is never touched when it lacks the tenant credential
  3. the first backup provider with a tenant credential executes directly
  4. usage record and routing decision log both point at the backup provider with `gateway_execution_failover`

### Step 17 - Route-layer audit for unavailable policy candidates

- confirmed the real missing-provider behavior: route selection already filtered policy-declared candidates that were absent from the current available candidate set before execution started
- fixed the audit gap instead of widening execution semantics:
  1. route selection now appends `policy_candidate_unavailable` when a declared policy candidate is skipped before execution
  2. execution-layer/provider-replacement semantics still use `gateway_execution_failover`
  3. routing skip vs execution failover now stay distinct and truthful in logs
- added direct routing, planned execution, and stateful `chat/responses` regressions for the missing-provider path

### Step 18 - Incompatible-standard OpenAI preflight failover

- found a real commercial execution gap on direct store-relay OpenAI `chat/responses` paths:
  1. ordered routing could select a provider whose `protocol_kind` was another industrial standard such as `anthropic`
  2. no explicit conversion plugin/runtime was bound for that provider
  3. execution returned adapter-surface `None`
  4. HTTP routes then dropped to local mock/local fallback instead of failing over to the compatible backup provider
- fixed the root cause in app-gateway execution:
  1. selected-provider local fallback still stays `None`
  2. selected-provider missing executable OpenAI adapter is now an execution failure, not a silent empty result
  3. existing execution failover now promotes the first compatible backup provider
- verified end to end for both JSON and SSE on both route families:
  1. `chat`
  2. `responses`
  3. primary incompatible standard provider is never contacted
  4. backup OpenAI-compatible provider executes
  5. usage record and routing decision log both point at the executed backup provider with `gateway_execution_failover`

### Step 19 - Provider executability observability

- found an operator-facing commercial gap after the protocol/plugin split:
  1. `/admin/providers` only returned static catalog metadata
  2. runtime status, provider health, and fail-closed plugin-binding risk were scattered across separate endpoints
  3. operators could not directly see whether a provider was standard passthrough only, adapter-surface executable, raw-plugin executable, or explicit-binding fail-closed
- fixed the gap with one gateway-derived management view:
  1. added `ProviderExecutionView` in `sdkwork-api-app-gateway` as the shared truth source
  2. `/admin/providers` now returns an additive `execution` object without breaking existing top-level provider fields
  3. the execution view is side-effect free and does not start runtimes
- current exposed fields are:
  1. `binding_kind`
  2. `runtime`
  3. `runtime_key`
  4. `passthrough_protocol`
  5. `supports_provider_adapter`
  6. `supports_raw_plugin`
  7. `fail_closed`
  8. `reason`
- verified commercial cases:
  1. official OpenAI provider shows implicit-default builtin execution plus standard passthrough visibility
  2. explicit native-dynamic provider shows raw-plugin capability and adapter-surface capability when the plugin exports both
  3. explicit persisted but unloadable native-dynamic binding shows `fail_closed=true` instead of silently reading as healthy/executable

### Step 20 - Route-family readiness observability

- found a semantic gap in Step 19:
  1. `execution.fail_closed` only described adapter/raw-plugin executability
  2. matching standard passthrough routes could still be available
  3. operators could misread one broken explicit plugin binding as "provider fully unusable"
- fixed it without changing runtime behavior:
  1. added `execution.route_readiness.openai|anthropic|gemini`
  2. each route family now reports `ready` plus `mode`
  3. mode is one of `provider_adapter`, `standard_passthrough`, `raw_plugin`, `fail_closed`, `unavailable`
- current resolution rule is explicit:
  1. `openai` uses provider-adapter executability as the canonical readiness contract
  2. `anthropic/gemini` prefer matching standard passthrough first
  3. heterogeneous raw plugin readiness is only reported when the bound manifest exposes the required operation family
  4. `fail_closed` remains scoped to families that actually need adapter/raw-plugin execution
- verified contracts:
  1. official OpenAI provider => all three route families report `provider_adapter`
  2. Anthropic native-dynamic plugin => `anthropic=standard_passthrough`, `gemini=raw_plugin`, `openai=provider_adapter`
  3. broken explicit Anthropic plugin => `anthropic=standard_passthrough`, `openai/gemini=fail_closed`

### Step 21 - Default-plugin protocol truth for Ollama

- found one remaining plugin-architecture mismatch:
  1. `ollama` / `ollama-compatible` were still deriving `protocol_kind=openai`
  2. builtin runtime truth already treats `sdkwork.provider.ollama` as `protocol=custom`
  3. this could mislead operators and stateless config into reading Ollama as standard-protocol passthrough instead of default-plugin adapter execution
- fixed the boundary in both domain and stateless config helpers:
  1. `openrouter*` stays `openai`
  2. `ollama*` now resolves to `custom`
  3. standard `openai|anthropic|gemini` mappings stay unchanged
- resulting contract:
  1. Ollama still supports config-only onboarding through the default plugin
  2. Ollama no longer advertises itself as an industrial-standard passthrough provider
  3. "standard passthrough" and "default plugin adapter" now stay separated cleanly

### Step 22 - Admin default-plugin onboarding semantics

- found one remaining admin/control-plane ambiguity:
  1. same-structure builtin providers still had to be created through low-level `adapter_kind`
  2. `/admin/providers` did not explicitly tell operators whether a provider was on the standard passthrough path, builtin default-plugin path, or custom-plugin path
  3. `openrouter` and `ollama` onboarding could still look too similar to either official-standard passthrough or custom heterogeneous plugin binding
- fixed the management contract without changing runtime execution rules:
  1. `POST /admin/providers` now accepts additive `default_plugin_family`
  2. current first-class builtin families are `openrouter` and `ollama`
  3. `default_plugin_family` now normalizes and pins:
     - `adapter_kind`
     - `protocol_kind`
     - `extension_id`
  4. conflicting low-level fields now fail fast with `400 Bad Request` instead of silently drifting into a different integration mode
  5. `/admin/providers` now returns additive `integration.mode={standard_passthrough|default_plugin|custom_plugin}`
  6. `/admin/providers` now returns `integration.default_plugin_family` when the provider uses the builtin default-plugin path
- kept the core architecture boundaries unchanged:
  1. standard industrial protocols still stay passthrough-first
  2. same-structure builtin families stay config-only onboarding
  3. heterogeneous providers still require the custom-plugin path
  4. `integration` explains onboarding shape; `execution` still explains runtime truth

### Step 23 - Default-plugin normalization ownership and fixture convergence

- found one remaining follow-through gap after Step 22:
  1. `default_plugin_family` normalization still lived in the admin HTTP layer
  2. `sdkwork-api-app-catalog` did not yet expose the default-plugin identity contract as shared application logic
  3. some admin routing/model tests still created OpenRouter through the old `adapter_kind=openai` shortcut, which contradicted the new onboarding semantics
- fixed the ownership boundary:
  1. added shared app-catalog normalization for provider integration identity
  2. added first-class app-catalog tests for:
     - OpenRouter default-plugin creation
     - default-plugin identity normalization
     - conflicting low-level field rejection
  3. rewired admin provider creation to consume the app-catalog normalization contract instead of duplicating the rule in the controller
  4. updated admin routing/model fixtures to create OpenRouter through `default_plugin_family=openrouter`
- resulting contract:
  1. default-plugin onboarding semantics now live in the application layer
  2. admin transport no longer owns provider identity policy
  3. management-side test samples now reinforce the intended architecture instead of the old shortcut path

### Step 24 - Non-admin OpenRouter fixture convergence

- found one remaining architecture drift after Step 23:
  1. `sdkwork-api-app-routing` simulate-route helpers still created OpenRouter through the old official OpenAI fixture shape
  2. `sdkwork-api-app-gateway` routing relay regression still mounted OpenRouter through `sdkwork.provider.openai.official`
  3. non-admin regressions could therefore keep passing while teaching the wrong default-plugin identity
- fixed the drift in test/support layers:
  1. app-routing support now preloads the `openrouter` channel and provides explicit OpenRouter fixture insertion
  2. OpenRouter fixture identity is now pinned to:
     - `channel_id=openrouter`
     - `adapter_kind=openrouter`
     - `protocol_kind=openai`
     - `extension_id=sdkwork.provider.openrouter`
     - secondary `openai` channel binding for OpenAI-route coverage
  3. simulate-route basic-selection and geo/profile regressions now use the canonical OpenRouter fixture instead of the old `openai/openai` shortcut
  4. app-gateway routing relay regression now installs and binds the builtin OpenRouter runtime instead of the official OpenAI builtin runtime
  5. routing and relay regressions now assert OpenRouter identity directly, so future fixture drift fails fast
- kept the core architecture boundaries unchanged:
  1. OpenRouter stays `protocol_kind=openai`
  2. OpenRouter does not collapse into official OpenAI runtime identity
  3. standard passthrough vs default-plugin onboarding remains explicit

### Step 25 - Stateless protocol normalization reuse and sample convergence

- found two remaining follow-up drifts after Step 24:
  1. `sdkwork-api-interface-http::StatelessGatewayUpstream` still owned a duplicate protocol-derivation table instead of reusing shared provider protocol truth
  2. portal and HTTP relay examples still contained low-level OpenRouter/Ollama onboarding samples that no longer matched the intended default-plugin contract
- fixed the boundary:
  1. stateless upstream construction now reuses `sdkwork-api-domain-catalog` protocol derive/normalize helpers
  2. added stateless regression coverage that pins `openrouter => openai` alongside the existing Ollama `custom` coverage
  3. portal routing fixtures now mount OpenRouter with canonical default-plugin identity plus a secondary `openai` channel binding
  4. HTTP relay regressions now create OpenRouter and Ollama through `default_plugin_family` and assert the derived `adapter_kind` / `protocol_kind` / `extension_id` returned by admin creation
- kept the core architecture boundaries unchanged:
  1. standard `openai` / `anthropic` / `gemini` upstreams still prefer direct passthrough
  2. same-structure builtin providers still use config-only default-plugin onboarding
  3. only incompatible APIs still require explicit plugin conversion

### Step 26 - Shared default-plugin seeding helper convergence

- found one remaining governance gap after Step 25:
  1. app/domain layers already owned default-plugin identity normalization, but there was no shared constructor for `default_plugin_family + extra channel bindings`
  2. high-value non-admin fixtures therefore still open-coded canonical OpenRouter identity in support/tests
- fixed the gap:
  1. added `create_provider_with_default_plugin_family_and_bindings(...)` to `sdkwork-api-app-catalog`
  2. rewired the existing default-plugin constructor to reuse that helper
  3. rewired app-routing support/basic-selection/geo-profile fixtures, app-gateway routing relay regression, and portal routing fixtures to consume shared app-catalog constructors
  4. reused `create_provider_with_config(...)` for adjacent standard OpenAI fixtures at those seams
- kept the architecture boundary unchanged:
  1. industrial-standard providers still stay passthrough-first
  2. same-structure builtin providers still onboard through config-only default-plugin helpers
  3. only incompatible APIs still require explicit conversion plugins

### Step 27 - Admin create-response integration convergence

- found one remaining management mutation gap after Step 26:
  1. `/admin/providers` list already exposed `integration`
  2. `POST /admin/providers` still returned normalized provider fields only
  3. callers had to refetch the provider catalog just to learn whether the created provider landed on `standard_passthrough`, `default_plugin`, or `custom_plugin`
- fixed the contract conservatively:
  1. `POST /admin/providers` now returns additive `integration`
  2. the create response still does not return `execution`
  3. create-time onboarding truth and list-time runtime truth now stay intentionally separated

### Step 28 - Portal provider-option integration visibility

- found one remaining frontend/provider-choice gap after Step 27:
  1. portal routing summary provider options exposed only `provider_id`, `display_name`, and `channel_id`
  2. frontend surfaces could not tell whether a candidate provider was industrial-standard passthrough, builtin default-plugin onboarding, or custom-plugin normalization
  3. admin and portal risked forking provider-integration classification rules
- fixed the ownership boundary:
  1. moved reusable `ProviderIntegrationView` derivation into `sdkwork-api-app-catalog`
  2. rewired admin create/list responses to consume that shared application-layer metadata helper
  3. portal routing provider options now expose additive `protocol_kind` plus `integration`
- kept the architecture boundary unchanged:
  1. portal visibility is still static/side-effect free
  2. no credential-readiness or runtime-health probing was folded into the portal summary
  3. industrial-standard passthrough vs default-plugin vs custom-plugin now stay explicit on both admin and portal surfaces

### Step 29 - Portal tenant credential-readiness visibility

- found the next portal UX gap after Step 28:
  1. frontend/provider-choice surfaces could now see static onboarding shape
  2. they still could not tell whether the current workspace tenant had configured a credential for each provider
  3. mixing that into `integration` or `protocol_kind` would have collapsed static provider truth and tenant-scoped secret state into one field
- fixed the contract conservatively:
  1. portal routing `provider_options` now expose additive `credential_readiness`
  2. current shape is:
     - `ready`
     - `state={configured|missing}`
  3. readiness is derived from tenant-scoped credential presence only
  4. readiness stays separate from:
     - `protocol_kind`
     - `integration`
     - admin `execution`
     - live runtime-health probing
- kept the architecture boundary unchanged:
  1. industrial-standard passthrough vs default-plugin vs custom-plugin remains static provider metadata
  2. tenant credential presence is now explicit dynamic metadata, not inferred onboarding shape
  3. portal summary remains side-effect free and does not perform live health probing

### Step 30 - Admin tenant-scoped credential-readiness visibility

- found the next operator/catalog gap after Step 29:
  1. admin provider list already exposed static `integration` and runtime `execution`
  2. operators still could not ask "is tenant X configured for provider Y" from the provider catalog surface
  3. adding that state to every unscoped `/admin/providers` response would have polluted a global catalog view with tenant-specific secret presence
- fixed the contract conservatively:
  1. `/admin/providers` default response remains tenant-agnostic
  2. `GET /admin/providers?tenant_id=<tenant>` now adds `credential_readiness`
  3. current shape is:
     - `ready`
     - `state={configured|missing}`
  4. readiness is derived from tenant-scoped credential presence only
  5. readiness stays separate from:
     - `integration`
     - `execution`
     - live runtime health
  6. readiness derivation now lives in shared app-layer `sdkwork-api-app-credential`, and portal was rewired to reuse that same contract
- kept the architecture boundary unchanged:
  1. global provider catalog truth remains static unless tenant scope is requested explicitly
  2. tenant secret presence stays additive and explicit
  3. portal/admin no longer duplicate credential-readiness classification logic

### Step 31 - Admin provider OpenAPI contract convergence

- found the next delivery gap after Step 30:
  1. runtime and tests already supported `/admin/providers?tenant_id=...`
  2. admin OpenAPI still did not publish `/admin/providers` get/post or the optional `tenant_id` query semantics
  3. plugin-management clients could therefore miss the explicit split between `integration`, `execution`, and `credential_readiness`
- fixed the contract without changing provider execution:
  1. added admin OpenAPI `catalog` entries for `GET /admin/providers` and `POST /admin/providers`
  2. documented `tenant_id` as an opt-in query scope that only adds tenant-scoped `credential_readiness`
  3. exposed schema support for real shared provider metadata and shared app-layer integration/execution/readiness views instead of inventing transport-local shadow shapes
- kept the architecture boundary unchanged:
  1. `integration` remains static onboarding truth
  2. `execution` remains runtime executability truth
  3. `credential_readiness` remains optional tenant-scoped secret-presence truth
  4. unscoped `/admin/providers` remains the global catalog contract

### Step 32 - Dedicated tenant provider-readiness endpoint

- found the next governance gap after Step 31:
  1. `/admin/providers?tenant_id=...` is useful, but it is still a global catalog route carrying a tenant overlay
  2. future tenant-scoped provider state would keep widening that combined surface
  3. operator tooling needs one focused tenant/provider readiness endpoint that does not blur global runtime truth
- fixed the boundary conservatively:
  1. kept `/admin/providers?tenant_id=...` for convenience and backward-compatible combined inspection
  2. added `GET /admin/tenants/{tenant_id}/providers/readiness`
  3. new endpoint returns:
     - `id`
     - `display_name`
     - `protocol_kind`
     - `integration`
     - `credential_readiness`
  4. new endpoint intentionally does not return `execution`, so tenant overlay stays separate from global executability truth
- kept the architecture boundary unchanged:
  1. static onboarding truth still lives in `integration`
  2. global runtime truth still lives in `/admin/providers.execution`
  3. tenant-scoped secret-presence truth now has its own dedicated admin surface
  4. the existing query-scoped overlay remains additive convenience, not the only tenant-state contract

### Step 33 - Stateless default-plugin ingress contract

- found one remaining non-admin ingress gap after Step 32:
  1. admin provider creation already supported `default_plugin_family`
  2. stateless upstream configuration still primarily exposed low-level `runtime_key` / adapter semantics
  3. config-only builtin same-structure onboarding was therefore not symmetrical outside the admin surface
- fixed it conservatively:
  1. added `StatelessGatewayUpstream::from_default_plugin_family(...)`
  2. added `StatelessGatewayConfig::try_with_default_plugin_upstream(...)`
  3. both stateless entrypoints now reuse shared default-plugin family normalization and reject unsupported families explicitly
  4. legacy `new(...)`, `new_with_protocol_kind(...)`, and `from_adapter_kind(...)` remain available for explicit advanced or backward-compatible paths
- verified the stateless contract:
  1. OpenRouter default-plugin ingress resolves to `runtime_key=openrouter`, `protocol_kind=openai`
  2. Ollama default-plugin ingress resolves to `runtime_key=ollama`, `protocol_kind=custom`
  3. unsupported `default_plugin_family` fails fast
  4. the config-level shortcut wires the upstream without extra low-level identity plumbing

## Verified

- `cargo test -p sdkwork-api-interface-http --test anthropic_messages_route stateful_anthropic_messages_route_fails_over_before_execution_when_primary_lacks_tenant_credential -- --nocapture`
- `cargo test -p sdkwork-api-interface-http --test chat_route stateful_chat_route_fails_over_before_execution_when_primary_lacks_tenant_credential -- --nocapture`
- `cargo test -p sdkwork-api-interface-http --test chat_route stateful_chat_stream_route_fails_over_before_execution_when_primary_lacks_tenant_credential -- --nocapture`
- `cargo test -p sdkwork-api-interface-http --test responses_route stateful_responses_route_fails_over_before_execution_when_primary_lacks_tenant_credential -- --nocapture`
- `cargo test -p sdkwork-api-interface-http --test responses_route stateful_responses_stream_route_fails_over_before_execution_when_primary_lacks_tenant_credential -- --nocapture`
- `cargo test -p sdkwork-api-interface-http stateful_chat_route_fails_over_before_execution_when_primary_requires_non_openai_standard_without_plugin -- --nocapture`
- `cargo test -p sdkwork-api-interface-http stateful_chat_stream_route_fails_over_before_execution_when_primary_requires_non_openai_standard_without_plugin -- --nocapture`
- `cargo test -p sdkwork-api-interface-http stateful_responses_route_fails_over_before_execution_when_primary_requires_non_openai_standard_without_plugin -- --nocapture`
- `cargo test -p sdkwork-api-interface-http stateful_responses_stream_route_fails_over_before_execution_when_primary_requires_non_openai_standard_without_plugin -- --nocapture`
- `cargo test -p sdkwork-api-app-gateway --test extension_dispatch planned_execution_context_fails_over_when_selected_connector_cannot_start -- --nocapture`
- `cargo test -p sdkwork-api-extension-core --test manifest_contract runtime_capability_helpers_keep_connector_off_raw_plugin_surface -- --nocapture`
- `cargo test -p sdkwork-api-app-gateway --test raw_execution_governance connector_runtime -- --nocapture`
- `cargo test -p sdkwork-api-app-gateway --test extension_dispatch planned_execution_context_probes_external_connector_health_once -- --nocapture`
- `cargo test -p sdkwork-api-app-gateway --test planned_execution -- --nocapture`
- `cargo test -p sdkwork-api-app-gateway --test extension_dispatch -- --nocapture`
- `cargo test -p sdkwork-api-app-gateway --test raw_execution_governance -- --nocapture`
- `cargo test -p sdkwork-api-app-gateway --test raw_execution_governance none_is_accounting_neutral -- --nocapture`
- `cargo test -p sdkwork-api-app-gateway --test raw_execution_governance raw_json_planned_execution_retries_timeout_and_succeeds_when_policy_allows -- --nocapture`
- `cargo test -p sdkwork-api-app-gateway --test raw_execution_governance raw_json_planned_execution_retries_retryable_plugin_error_and_succeeds -- --nocapture`
- `cargo test -p sdkwork-api-app-gateway --test raw_execution_governance raw_json_planned_execution_does_not_retry_opaque_plugin_error -- --nocapture`
- `cargo test -p sdkwork-api-app-gateway --test raw_execution_governance raw_stream_planned_execution_ -- --nocapture`
- `cargo test -p sdkwork-api-extension-abi --test abi_contract -- --nocapture`
- `cargo test -p sdkwork-api-extension-host --test discovery`
- `cargo test -p sdkwork-api-extension-host`
- `cargo test -p sdkwork-api-app-gateway --test extension_dispatch`
- `cargo test -p sdkwork-api-app-gateway`
- `cargo test -p sdkwork-api-interface-http --test anthropic_messages_route native_dynamic_raw_plugin -- --nocapture`
- `cargo test -p sdkwork-api-interface-http --test gemini_generate_content_route native_dynamic_raw_plugin -- --nocapture`
- `cargo test -p sdkwork-api-interface-http --test anthropic_messages_route stateful_anthropic_messages_stream_route_prefers_native_dynamic_raw_plugin_for_explicit_custom_runtime_and_records_usage -- --nocapture`
- `cargo test -p sdkwork-api-interface-http --test gemini_generate_content_route stateful_gemini_stream_generate_content_route_prefers_native_dynamic_raw_plugin_for_explicit_custom_runtime_and_records_usage -- --nocapture`
- `cargo test -p sdkwork-api-interface-http --test anthropic_messages_route`
- `cargo test -p sdkwork-api-interface-http --test gemini_generate_content_route`
- `cargo test -p sdkwork-api-interface-http --test chat_route -- --nocapture`
- `cargo test -p sdkwork-api-interface-http --test responses_route -- --nocapture`
- `cargo test -p sdkwork-api-interface-admin list_providers_exposes_implicit_standard_passthrough_execution_view -- --nocapture`
- `cargo test -p sdkwork-api-interface-admin list_providers_exposes_native_dynamic_raw_plugin_execution_view -- --nocapture`
- `cargo test -p sdkwork-api-interface-admin list_providers_exposes_fail_closed_execution_view_for_broken_explicit_binding -- --nocapture`
- `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes providers_models_coupons -- --nocapture`
- `cargo test -p sdkwork-api-app-routing --test simulate_route -- --nocapture`
- `cargo test -p sdkwork-api-app-gateway --test planned_execution planned_chat_execution_fails_over_when_selected_provider_is_missing -- --nocapture`
- `cargo test -p sdkwork-api-interface-http --test chat_route stateful_chat_route_fails_over_before_execution_when_primary_provider_is_missing -- --nocapture`
- `cargo test -p sdkwork-api-interface-http --test responses_route stateful_responses_route_fails_over_before_execution_when_primary_provider_is_missing -- --nocapture`
- `cargo test -p sdkwork-api-extension-host --test connector_runtime -- --nocapture`
- `cargo check -p sdkwork-api-extension-abi -p sdkwork-api-extension-host -p sdkwork-api-ext-provider-native-mock -p sdkwork-api-app-gateway -p sdkwork-api-interface-http`
- `cargo check -p sdkwork-api-extension-host -p sdkwork-api-app-gateway -p sdkwork-api-interface-http`
- `cargo check -p sdkwork-api-app-gateway -p sdkwork-api-interface-admin`
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
- `cargo test -p sdkwork-api-app-catalog --test create_provider -- --nocapture`
- `cargo test -p sdkwork-api-app-routing --test simulate_route -- --nocapture`
- `cargo test -p sdkwork-api-app-gateway --test routing_policy_dispatch -- --nocapture`
- `cargo test -p sdkwork-api-interface-portal --test portal_routing -- --nocapture`
- `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes providers_models_coupons -- --nocapture`
- `cargo test -p sdkwork-api-interface-portal --test portal_routing portal_routing_preferences_preview_and_logs_are_project_scoped -- --nocapture`
- `cargo check -p sdkwork-api-app-catalog -p sdkwork-api-interface-admin -p sdkwork-api-interface-portal`
- `cargo check -p sdkwork-api-app-catalog -p sdkwork-api-app-routing -p sdkwork-api-app-gateway -p sdkwork-api-interface-portal`
- `cargo check -p sdkwork-api-app-routing -p sdkwork-api-app-gateway`
- `cargo check -p sdkwork-api-domain-catalog -p sdkwork-api-interface-http`
- `cargo check -p sdkwork-api-interface-http`
- `cargo check -p sdkwork-api-app-gateway -p sdkwork-api-interface-http`
- `cargo check -p sdkwork-api-ext-provider-native-mock -p sdkwork-api-app-gateway -p sdkwork-api-interface-http`
- `cargo test -p sdkwork-api-interface-portal --test portal_routing -- --nocapture`
- `cargo check -p sdkwork-api-interface-http -p sdkwork-api-interface-portal`
- `cargo test -p sdkwork-api-interface-portal --test portal_routing portal_routing_preferences_preview_and_logs_are_project_scoped -- --nocapture`
- `cargo test -p sdkwork-api-interface-portal --test portal_routing -- --nocapture`
- `cargo check -p sdkwork-api-interface-portal`
- `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes list_providers_exposes_tenant_scoped_credential_readiness_only_when_requested -- --nocapture`
- `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes providers_models_coupons -- --nocapture`
- `cargo test -p sdkwork-api-app-credential -- --nocapture`
- `cargo check -p sdkwork-api-app-credential -p sdkwork-api-interface-admin -p sdkwork-api-interface-portal`
- `cargo test -p sdkwork-api-interface-admin --test openapi_route openapi_routes_expose_admin_api_inventory_with_schema_components -- --nocapture`
- `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes list_providers_exposes_tenant_scoped_credential_readiness_only_when_requested -- --nocapture`
- `cargo check -p sdkwork-api-domain-catalog -p sdkwork-api-app-catalog -p sdkwork-api-app-gateway -p sdkwork-api-app-credential -p sdkwork-api-interface-admin`
- `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes list_tenant_provider_readiness_exposes_focused_tenant_overlay_inventory -- --nocapture`
- `cargo check -p sdkwork-api-interface-admin -p sdkwork-api-app-credential -p sdkwork-api-app-catalog -p sdkwork-api-domain-catalog`
- `cargo test -p sdkwork-api-interface-http --test stateless_runtime -- --nocapture`
- `cargo test -p sdkwork-api-interface-http --test stateless_upstream_protocol -- --nocapture`

## Next

### Step 34 - Follow-up governance

- keep route-selection `policy_candidate_unavailable` separate from execution-layer `gateway_execution_failover` if new preflight failover classes are added later
- decide whether tenant-scoped readiness should eventually grow a provider-specific detail endpoint or additional tenant-state fields beyond credential presence
- decide whether connector runtimes need a parallel transient-error contract or should stay permanently outside raw-plugin retry semantics
- keep connector-runtime vs native-dynamic raw-execution boundaries explicit in admin/provider docs
- keep connector supervision reuse and startup checks off the async executor path if runtime governance expands further
- keep raw SSE retry bounded to startup-before-first-byte unless product semantics intentionally change
- add new custom plugin operation families only when a provider cannot already speak the supported industrial-standard protocols
- decide whether additional builtin same-structure families beyond `openrouter` and `ollama` should become first-class `default_plugin_family` values or require persisted onboarding metadata
- if future non-admin provider-ingest surfaces appear beyond admin/stateless, either expose `default_plugin_family` directly or pin them to the shared normalization contract; do not reintroduce raw `adapter_kind` shortcuts

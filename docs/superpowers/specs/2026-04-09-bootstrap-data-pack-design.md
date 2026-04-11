# Bootstrap Data Pack Design

**Context**

The current runtime bootstrap only guarantees a thin baseline:

- local bootstrap admin / portal identities
- a small official provider catalog
- a starter model subset

That is not enough for a "ready to operate" dev or install-time commercial experience. The system needs a repository-backed, environment-aware bootstrap data pack that can materialize richer default data on startup without scattering seed logic across Rust modules.

**Goals**

- Move bootstrap content into `/data` JSON assets grouped by domain.
- Support both dev and production profiles from the same importer.
- Keep imports idempotent so repeated startups do not duplicate records.
- Preserve safe production behavior by shipping metadata and defaults, not real upstream secrets.
- Materialize a richer commercial-ready surface:
  - official channels and official providers
  - openrouter / siliconflow / ollama and other default ecosystem providers
  - channel models, provider model variants, model prices
  - tenants, projects, API key groups, routing profiles, routing policies, routing preferences
  - payment methods, pricing plans, pricing rates
  - marketing coupons and workspace-facing demo/commercial data
  - account kernel records for balances, lots, holds, ledger, request metering, settlements, and reconciliation

**Non-goals**

- Shipping real provider API keys or payment secrets in repository JSON.
- Turning startup bootstrap into a migration framework replacement.
- Auto-generating data from remote APIs at startup.

**Architecture**

Add a bootstrap data importer in the runtime initialization path after schema migrations complete and before listeners start serving traffic.

The importer reads a configured `bootstrap_data_dir` and `bootstrap_profile`, loads one profile manifest plus its ordered update manifests, then applies grouped JSON bundles in a fixed dependency order:

1. channels
2. providers
3. extensions and runtime governance
4. official provider configs
5. models
6. channel models
7. provider models
8. model prices
9. tenants
10. projects
11. users / identities
12. routing profiles
13. routing policies
14. routing preferences
15. API key groups and observability seeds
16. quota policies and rate limits
17. pricing plans
18. pricing rates
19. accounts
20. account benefit lots
21. account holds bundle
22. account ledger bundle
23. request settlements
24. request metering bundle
25. payment methods and memberships
26. marketing, commerce, and jobs
27. account reconciliation state

Each bundle is plain JSON. Cross-record references use stable IDs already stored in the JSON assets. The importer validates reference existence between stages so broken data fails fast with a useful error message.

**Canonical Catalog Model**

- `channel` is the canonical model inventor / vendor surface.
  - examples: `openai`, `anthropic`, `gemini`, `deepseek`, `qwen`, `doubao`, `xai`
- `channel-model` is the canonical public model identity published under a channel.
  - examples: `openai/gpt-4.1`, `anthropic/claude-3-7-sonnet`, `gemini/gemini-2.5-pro`
- `provider` is the executable upstream entry.
  - official providers expose the vendor's own API
  - proxy providers expose a selectable subset of canonical models
  - local providers such as Ollama expose local execution or open-weight serving
- `provider-model` is the binding from a canonical `channel-model` to one provider-specific model id or family.
  - this record owns `provider_model_id`, `provider_model_family`, capability flags, cache support, route eligibility, and active/default posture
  - proxy providers do not imply full channel coverage; supported subsets are explicitly declared here
  - shipped seed data must not declare a proxy/local `channel_binding` unless at least one active `provider-model` and one active `model-price` row exist for that bound channel
  - admin provider management must treat this as the operator-owned execution subset:
    - enabling or disabling canonical model support happens here
    - provider-specific ids, families, capability deltas, route posture, and token limits are curated here
    - pricing coverage in admin is derived by joining `provider-model` to `model-price`
  - admin model management should treat `channel-model` as the full inventor catalog and `provider-model` as the provider-supported subset an operator can enable or curate for each official, proxy, or local provider
- `route config` chooses providers, not channels.
  - route execution resolves the selected provider plus canonical model through `provider-model`
  - runtime may rewrite the outbound request to the provider-specific `provider_model_id`

**Pricing Contract**

- `model-price` is the operator-facing catalog/reference price relationship between one canonical `channel-model` and one execution provider.
- `model-price` must not exist unless the matching `provider-model` already exists for the same `(provider, channel, model)` key.
  - this keeps catalog pricing aligned with actual provider support
  - admin create/update flows should reject price rows that are not backed by provider support
  - repository update packs should preserve the same rule so bootstrapped data never drifts into "priced but unroutable"
- bootstrap keeps the normalized top-level columns used by routing, accounting, and reporting:
  - `input_price`
  - `output_price`
  - `cache_read_price`
  - `cache_write_price`
  - `request_price`
- bootstrap also carries operator-friendly pricing metadata:
  - `price_source_kind`
    - `official`
    - `proxy`
    - `local`
    - `reference`
  - `billing_notes`
  - `pricing_tiers`
- `pricing_tiers` covers cases where one flat row is not expressive enough:
  - prompt-length breakpoints
  - modality-specific pricing
  - cache-window variants
  - cache-storage pricing
  - request-class overrides
- admin should expose the top-level row as a friendly summary while still letting operators inspect the richer tier metadata, compare official versus proxy versus local rows, and identify missing pricing coverage for provider-supported models
- top-level prices remain the default summary surface for routing and accounting, while `pricing_tiers` preserves the richer explanation operators need in admin and data packs.
- `pricing-plan` and `pricing-rate` remain the internal commercial charge contract used by metering and settlement.
  - `model-price` answers "what does this provider charge for this model posture?"
  - `pricing-plan` / `pricing-rate` answer "how does this tenant/account get charged or costed?"
  - request metering can reference both cost and retail pricing plans, so official pricing and proxy pricing remain explainable without collapsing catalog price and billing policy into one table.
  - pricing plans should represent a billing posture or price class, not duplicate every provider-model reference row one-for-one.
  - `request-meter-fact.cost_pricing_plan_id` links the request to the internal or upstream cost posture.
  - `request-meter-fact.retail_pricing_plan_id` links the same request to the sell-side commercial posture.
  - additive update packs may relink request facts to newer pricing classes without mutating account holds, settlements, or ledger history, because the exact provider-model reference remains on `model-price`.

**Commercial Account Kernel**

- `accounts` seed the payable balance surface for production installs and richer demo workspaces.
- `account-benefit-lots` materialize prepaid or promotional credit buckets with expiry and status.
- `account-holds` is a cohesive bundle containing:
  - `holds`
  - `allocations`
- `account-ledger` is a cohesive bundle containing:
  - `entries`
  - `allocations`
- `request-metering` is a cohesive bundle containing:
  - `facts`
  - `metrics`
- `request-settlements` capture the outcome of each metered request: hold, capture, release, provider cost, retail charge, refund, and shortfall.
- `account-reconciliation` links the account layer back to project commerce history through stable `(account_id, project_id)` state.

This keeps the commercial model decomposed but connected:

- routing chooses a provider through `route config`
- provider-model confirms the provider can actually execute the canonical model
- model-price exposes official/proxy/local provider catalog pricing
- pricing-plan and pricing-rate define internal billing and costing policy
- request-meter-fact links account, tenant, user, channel, model, provider, api key, and pricing plans
- request-settlement and account-ledger close the financial trail
- reconciliation state links the account layer back to project orders

**Commercial Walkthrough Packs**

- Additive update packs may intentionally seed install-ready walkthrough data for distinct commercial postures:
  - official direct
  - marketplace proxy
  - regional distribution
  - local edge
  - enterprise contract and manual-review invoice flows
- These walkthrough packs should stay split by domain JSON under `/data`, but may span `marketing`, `payment-methods`, `commerce`, `billing`, `jobs`, and optional reconciliation slices together in one ordered update manifest.
- Walkthrough records should prefer stable IDs and real cross-record references so repeated bootstrap remains idempotent while operators still get immediately explorable checkout, refund, and reconciliation examples on first install.
- Pending or partially settled walkthroughs are valid shipped samples as long as the lifecycle is explicit:
  - order and payment attempt statuses stay operator-readable
  - async jobs can represent pre-sales, contract, or finance side work
  - reconciliation state advances with the latest shipped commercial order so admin and portal views stay aligned with the newest walkthrough timeline
- If a walkthrough introduces a later commercial order for an existing `(account_id, project_id)` pair, ship the matching `account-reconciliation` override in the same or a dependent update pack so portal and admin reconciliation views do not drift behind the commerce timeline.
- Do not rewrite baseline packs to add new walkthroughs. Add a new update pack and let last-wins merge semantics extend the shipped commercial surface safely.

**Data Layout**

`/data` becomes the canonical seed root:

- `data/profiles/dev.json`
- `data/profiles/prod.json`
- `data/updates/*.json`
- `data/channels/*.json`
- `data/providers/*.json`
- `data/official-provider-configs/*.json`
- `data/models/*.json`
- `data/channel-models/*.json`
- `data/provider-models/*.json`
- `data/model-prices/*.json`
- `data/tenants/*.json`
- `data/projects/*.json`
- `data/api-key-groups/*.json`
- `data/routing/*.json`
- `data/quota-policies/*.json`
- `data/pricing/*.json`
- `data/accounts/*.json`
- `data/account-benefit-lots/*.json`
- `data/account-holds/*.json`
- `data/account-ledger/*.json`
- `data/request-metering/*.json`
- `data/request-settlements/*.json`
- `data/account-reconciliation/*.json`
- `data/payment-methods/*.json`
- `data/marketing/*.json`

Profiles declare which files to include, so dev can load richer demo workspace data while production loads only safe commercial-ready metadata and default operational scaffolding.

**Environment Model**

- `dev` profile:
  - includes safe demo workspace data
  - includes local bootstrap identities
  - includes operational examples that make portal/admin immediately usable
- `prod` profile:
  - includes official channels/providers/models and commercial scaffolding
  - includes default routing/pricing/payment metadata
  - excludes demo-only user traffic history and fake secrets

**Importer Behavior**

- Idempotent by design:
  - use existing `insert_*` / `upsert_*` semantics where available
  - domain records use stable IDs in JSON
  - profile refs and ordered `updates/*.json` manifests are merged first
  - merged records then collapse with last-wins semantics by stable domain key, so additive update packs can override or extend shipped defaults without mutating baseline files in place
  - creating a provider-scoped model through bootstrap or admin should materialize the canonical `channel-model`, the provider-supported `provider-model`, and the default `model-price` row together when that combination is intended to be routable
  - repeated startup updates records instead of multiplying them
- Strict ordering:
  - fail if a child record references a missing parent
- Strict parsing:
  - reject malformed JSON, duplicate IDs within a bundle, or unsupported profile references
- Strict relational validation:
  - request metering must reference a valid account, channel, provider, provider-supported model, api key, and optional pricing plans
  - holds, hold allocations, ledger allocations, settlements, and reconciliation state must match ownership and lifecycle timestamps
  - commerce reconciliation must point to real projects and orders
  - repository-level coverage tests should assert that declared proxy/local channel bindings are backed by active `provider-model` plus `model-price` records, so update packs cannot silently drift into "bound but unroutable" states
- Safe secrets:
  - official provider config metadata can be imported from JSON
  - real secrets remain env/config/secret-manager driven
- Separated write boundaries:
  - catalog, identity, routing, and commerce seeds write through `AdminStore`
  - account kernel domains write through `AccountKernelStore`
  - pricing plan/rate writes go through `CommercialBillingAdminKernel`

This keeps the bootstrap framework high-cohesion and low-coupling while still letting one profile materialize a commercially usable environment.

**Catalog Coverage**

The bundled catalog should include at least:

- official channels/providers:
  - OpenAI
  - Anthropic / Claude
  - Gemini
  - DeepSeek
  - Qwen
  - Doubao
  - Hunyuan
  - xAI
- ecosystem providers:
  - OpenRouter
  - SiliconFlow
  - Ollama

For each channel:

- one official provider mapping when an official API exists
- appropriate protocol/adapter metadata
- a curated canonical model set with capabilities, context windows, and starter prices where known
- provider-model subsets that declare which proxy providers support which canonical models
- price rows for official, proxy, and local execution paths when known

**Routing Defaults**

Bootstrap routing should expose an immediately usable default posture:

- default deterministic priority profile
- balanced weighted profile
- low-cost profile
- low-latency profile
- wildcard routing policies for `responses`, `chat_completions`, and `embeddings`
- project routing preferences pointing at the default profile
- default route candidates aligned to provider-model coverage, so official channels can run immediately while proxy providers remain opt-in and model-subset aware

**Testing**

Add tests for:

- config parsing of `bootstrap_data_dir` and `bootstrap_profile`
- importer profile resolution
- importer idempotency across repeated runs
- startup bootstrap populating expected channels/providers/models/routing data
- startup bootstrap populating account kernel, request metering, settlement, and reconciliation data
- runtime startup with profile-based seed loading
- repository profile tests for both production and developer overlays

**Recommendation**

Implement a typed importer with domain-specific bundle structs rather than a generic untyped JSON interpreter. The domain types already derive serde in most cases, so typed bundles keep the importer simple, testable, and predictable.

Keep compound domains only where cohesion is strong:

- `account-holds` for holds plus allocations
- `account-ledger` for entries plus allocations
- `request-metering` for facts plus metrics
- `payment-methods` for methods plus credential bindings plus memberships

Everything else should stay as small, stable JSON arrays so update packs remain easy to diff, review, and evolve.

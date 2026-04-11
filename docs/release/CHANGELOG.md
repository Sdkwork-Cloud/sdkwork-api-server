# SDKWork API Router CHANGELOG

## v0.1.52 - Commercial S08 Release Sync External Blocker Formalization

- Date: 2026-04-11
- Type: patch
- Highlights:
  - refined the last `S08` blocker from a generic `release-sync-audit fail` label into a stop-condition-grade external blocker with exact repository evidence: the current `sdkwork-api-router` branch has no upstream configured, `sdkwork-core` is not a standalone git root and points at the wrong origin URL, and the governed sibling repositories remain dirty and remote-unverifiable
  - backwrote `S08` step, review, release, and architecture records so the repo now states explicitly that commercialization capability is closed while release closure is blocked by cross-repository release hygiene, not by unresolved product/runtime work
  - kept release truth fresh by reusing the governed latest artifacts posture: `release-window-snapshot = pass`, `release-slo-governance = pass`, `release-sync-audit = fail`, with explicit unlock conditions and next-loop entry

## v0.1.51 - Commercial S08 Live Governance Pass Replay

- Date: 2026-04-11
- Type: patch
- Highlights:
  - fixed the Windows PowerShell compatibility hole in `live-commercial-governance-replay.ps1`, so the governed replay can now finish artifact generation on the current host instead of failing on `ConvertFrom-Json -Depth`
  - replayed the source-backed admin, gateway, and portal stack against the repo-local `dev` bootstrap database, proved the canonical admin commercial-control-plane routes now return `200`, and materialized both isolated and latest governed telemetry export / snapshot / SLO artifacts from that live evidence
  - advanced `S08` from `window pass / sync fail / SLO fail` to `window pass / sync fail / SLO pass`, leaving `release-sync-audit` as the only remaining `no-go` blocker

## v0.1.50 - Commercial S08 Admin Service Commercial Billing Runtime Fix

- Date: 2026-04-11
- Type: patch
- Highlights:
  - fixed standalone `admin-api-service` runtime wiring so it now builds `commercial_billing` through `build_admin_store_and_commercial_billing_from_config(...)` and threads that kernel into both `AdminApiState` and runtime reload handles
  - added service-level regression coverage proving `/admin/billing/account-holds`, `/admin/billing/request-settlements`, and `/admin/billing/pricing-lifecycle/synchronize` all record `status="200"` metrics and no longer emit `status="501"` from the current source runtime
  - kept `S08` release truth honest: this loop closes the repo-local root cause, but the current shell policy still blocks source-backed background stack replay, so governed latest artifacts were not re-materialized and commercialization release remains `no-go` pending a fresh host-level replay

## v0.1.49 - Commercial S08 Governance Fail Replay

- Date: 2026-04-11
- Type: patch
- Highlights:
  - fixed the governed release telemetry snapshot parser so truthful live admin Prometheus text with route-template labels such as `/admin/runtime-config/rollouts/{rollout_id}` can be materialized instead of crashing the snapshot derivation lane
  - replayed `S08` with a truthful local stateful-provider failover probe, closing `gateway-fallback-success-rate` at `1/1` and `gateway-provider-timeout-budget` at `0`, then materialized `docs/release/release-telemetry-export-latest.json`, `release-telemetry-snapshot-latest.json`, and `slo-governance-latest.json`
  - converted the release gate from `window pass / sync fail / SLO blocked` into `window pass / sync fail / SLO fail`, with the remaining governed failures now explicit: `admin-api-availability`, `account-hold-creation-success-rate`, `request-settlement-finalize-success-rate`, and `pricing-lifecycle-synchronize-success-rate`

## v0.1.48 - Commercial S08 Local Derived Target Probe

- Date: 2026-04-11
- Type: patch
- Highlights:
  - proved through a live local derived-target probe that six previously-missing non-availability SLO targets are already truthfully closeable, including route simulation latency, API-key issuance, runtime rollout, gateway non-streaming and streaming success, and billing-event writes
  - replayed the governed telemetry chain with those truthful supplemental targets and advanced the first exact snapshot failure from `gateway-non-streaming-success-rate` to `gateway-fallback-success-rate`, tightening the remaining blocker to fallback / timeout evidence instead of a broad supplemental-target gap
  - documented the smaller residual blocker set explicitly: provider-level failover and timeout samples are still absent, commercial billing kernel routes still return `501`, portal billing account truth is still unprovisioned, and `release-sync-audit` remains independently failing

## v0.1.47 - Commercial S08 Managed Telemetry Availability Proof

- Date: 2026-04-10
- Type: patch
- Highlights:
  - proved that the current host already carries live managed telemetry on `127.0.0.1:9980 / 9981 / 9982`, with authenticated `/metrics` available for gateway, admin, and portal even though the older raw `8080 / 8081 / 8082` defaults remain absent
  - materialized a truthful release telemetry export from those live managed metrics into an isolated evidence pack, then reproduced the next exact failure point at snapshot derivation: `release telemetry snapshot is missing target gateway-non-streaming-success-rate`
  - tightened `S08` and `166` again so the commercialization gate now distinguishes "availability telemetry exists locally" from "the governed SLO baseline still lacks truthful supplemental target coverage", while keeping the honest posture unchanged: `window pass`, `sync fail`, `SLO blocked`

## v0.1.46 - Commercial S08 Telemetry Input External Blocker Refinement

- Date: 2026-04-10
- Type: patch
- Highlights:
  - converted the remaining `release-slo-governance` blocker from a generic “missing telemetry” statement into a reproducible external-input diagnosis: no telemetry env injection, no governed telemetry latest artifacts, and no reachable localhost `/metrics` sources on the documented ports
  - confirmed the repo already owns the telemetry export, snapshot, and SLO materialization chain, so the remaining gap is a truthful hosted control-plane handoff plus `supplemental.targets`, not a local implementation omission
  - tightened `S08` records again so the commercialization gate now distinguishes product closure from release-evidence supply more sharply: `window pass`, `sync fail`, `SLO blocked by external telemetry handoff`

## v0.1.45 - Commercial S08 Release Window Freshness Replay

- Date: 2026-04-10
- Type: patch
- Highlights:
  - investigated and fixed a new governed release-window evidence drift where default latest-artifact replay still carried `workingTreeEntryCount = 506` after later `S08` doc/release backwrite changed the live workspace truth
  - refreshed `docs/release/release-window-snapshot-latest.json` from current shell git truth and documented the operational rule that later `S08` backwrite must re-materialize that artifact before the next governance replay
  - reaffirmed the honest commercialization gate posture as `release-window pass`, `release-sync fail`, `release-slo blocked`, so the remaining blocker stays release governance rather than product convergence

## v0.1.44 - Commercial S08 Release Window Pass and Sync Audit Refinement

- Date: 2026-04-10
- Type: patch
- Highlights:
  - materialized governed `release-window-snapshot-latest.json` from current shell git truth, converting that release-governance lane from host-exec blocked to passing
  - materialized governed `release-sync-audit-latest.json` from current repository and sibling-repository shell truth, converting that lane from opaque `command-exec-blocked` into a concrete release `fail` with dirty-tree, branch-sync, remote-root, and remote-url evidence
  - tightened `S08` documentation so the commercialization gate now reflects the more accurate structure: `window pass`, `sync fail`, `SLO blocked`, instead of the older all-blocked posture

## v0.1.43 - Commercial S08 Integrated Acceptance and Release Gate

- Date: 2026-04-10
- Type: patch
- Highlights:
  - aggregated fresh admin, portal, and public gateway verification evidence into the `S08` commercialization acceptance matrix, showing the coupon-first `product / account / marketing` line is internally converged on current runtime and UI surfaces
  - backwrote `docs/step/110`, `docs/架构/166`, `docs/架构/133`, and `docs/架构/03` with the same final gate posture so the repository no longer describes `S08` as only a plan instead of an evidence-backed decision point
  - recorded the honest `S08` release result as `no-go`, with the remaining blockers narrowed to governed release telemetry input and release window / sync truth on a host that can prove remote Git state

## v0.1.42 - Commercial S07 Historical Legacy Coupon Doc Closure

- Date: 2026-04-10
- Type: patch
- Highlights:
  - cleaned `docs/step`, `docs/review`, and `docs/superpowers` so legacy coupon wording now reads as historical baseline or superseded design, not live product truth
  - removed exact legacy coupon crate, route, helper, and UI residue from planning and review docs, making `marketing` the only active coupon vocabulary across architecture and execution records
  - kept historical intent and audit value while tightening release-grade documentation posture for the commercial coupon-first marketing product line

## v0.1.41 - Commercial S07 Legacy Coupon Full Exit

- Date: 2026-04-10
- Type: patch
- Highlights:
  - removed the last active-surface legacy coupon residue from admin coupon tests, stale i18n catalogs, and the admin API reference, so `/admin/coupons` is no longer described or implied anywhere in the live control-plane product surface
  - updated active architecture docs to treat `marketing` as the only coupon truth and removed `sdkwork-api-domain-coupon` / `sdkwork-api-app-coupon` from the live module-boundary inventory
  - verified that active runtime and product surfaces no longer ship legacy coupon crate, projection, or route truth; remaining mentions are removal tests or historical records only

## v0.1.40 - Commercial S07 Main-Path Legacy Coupon Exit and S06 API Reference Closure

- Date: 2026-04-10
- Type: patch
- Highlights:
  - removed legacy coupon fallback from gateway, portal, and commerce main coupon flows, making canonical marketing the only coupon fact source on those runtime surfaces
  - removed admin primary-view `Legacy coupon compatibility` copy and dropped main-path legacy coupon crate dependencies from commerce and portal
  - closed the public consumer documentation gap by publishing `market / marketing / commercial` gateway route families plus `after_lot_id`, `next_after_lot_id`, and `scope_order_id` in the main API reference

## v0.1.39 - Commercial S06 Public Commercial Benefit-Lot Pagination

- Date: 2026-04-10
- Type: patch
- Highlights:
  - upgraded public `GET /commercial/account/benefit-lots` from store-wide filtering to account-scoped storage reads, closing the remaining `S06` scale gap on commercial benefit-lot visibility
  - added cursor pagination with `after_lot_id`, `limit`, and additive `page` metadata, so commercial account lot history can be traversed safely without weakening coupon/account-arrival semantics
  - added `idx_ai_account_benefit_lot_account_lot` on sqlite and postgres and synchronized gateway `/openapi.json` to the same pagination contract

## v0.1.38 - Commercial S06 Public Market/Coupon/Commercial API Convergence

- Date: 2026-04-10
- Type: patch
- Highlights:
  - added authenticated public gateway routes for `market / marketing / commercial`, covering API products, quotes, coupon validate-reserve-confirm-rollback, and commercial account plus benefit-lot visibility
  - kept public coupon semantics explicit with `template / campaign / applicability / effect`, making outward coupon effect visible as `checkout_discount | account_entitlement`
  - exposed `scope_order_id` on public commercial benefit-lot payloads, so order-to-account-arrival evidence is outwardly inspectable instead of portal-only
  - closed runtime versus `/openapi.json` drift by publishing the same 9-route public surface under gateway OpenAPI tags `market`, `marketing`, and `commercial`

## v0.1.37 - Commercial S03 Portal Coupon Account Arrival Correlation

- Date: 2026-04-10
- Type: patch
- Highlights:
  - added additive portal `reward-history.account_arrival` so grant-style coupon redemptions now expose linked commercial-account arrival evidence instead of stopping at redemption status
  - correlated portal reward-history to real runtime lot evidence through `CouponRedemptionRecord.order_id` and current-account `AccountBenefitLotRecord.scope_json.order_id`, constrained to `source_type = order`
  - synchronized manual portal OpenAPI, shared portal TS types, and portal credits reward-history UI with explicit arrival semantics `Arrived to account`, `No linked account lot evidence yet`, and `No account arrival for checkout discount`
  - kept the slice additive and coupon-first while narrowing the next `S03` outward gap from portal arrival evidence to wider public API convergence

## v0.1.36 - Commercial S03 Portal Coupon Semantic Convergence

- Date: 2026-04-10
- Type: patch
- Highlights:
  - upgraded portal `my-coupons / reward-history` into coupon-semantic read models carrying `template / campaign / applicability / effect / ownership` instead of only operational status fragments
  - made outward coupon effect explicit as `checkout_discount | account_entitlement`, keeping coupon-first semantics clear inside portal wallet and reward-history flows
  - surfaced target applicability and current-subject ownership truth through additive semantic fields and synchronized portal OpenAPI, shared TS types, and credits UI copy
  - restored sqlite compile continuity with a missing `PricingPlanOwnershipScope` import so the new portal commercialization slice can be rebuilt and verified cleanly

## v0.1.35 - Commercial S03 Admin Marketing Campaign Revision Governance

- Date: 2026-04-10
- Type: patch
- Highlights:
  - upgraded canonical `marketing campaign` with additive `approval_state / revision / root_marketing_campaign_id / parent_marketing_campaign_id`, so campaign revisions now carry explicit approval and lineage evidence instead of only rollout status
  - added semantic admin revision-governance APIs `clone / compare / submit-for-approval / approve / reject` for campaigns, keeping coupon-first campaign semantics explicit inside `market`
  - made campaign rollout approval-aware for governed revisions and extended lifecycle audit with `source_marketing_campaign_id`, approval-state transitions, and revision deltas
  - synchronized OpenAPI, admin shared TS types, and the admin API client, and backwrote architecture doc `166` so campaign-side `revision / approval / compare / clone` is no longer the next `S03` governance gap

## v0.1.34 - Commercial S03 Admin Marketing Coupon Template Revision Governance

- Date: 2026-04-10
- Type: patch
- Highlights:
  - upgraded canonical `coupon template` with additive `approval_state / revision / root_coupon_template_id / parent_coupon_template_id`, so template revisions now carry explicit approval and lineage evidence instead of only rollout status
  - added semantic admin revision-governance APIs `clone / compare / submit-for-approval / approve / reject`, keeping coupon semantics explicit inside the coupon-first `market` model
  - made template rollout approval-aware for governed revisions and extended lifecycle audit with `source_coupon_template_id`, approval-state transitions, and revision deltas
  - synchronized OpenAPI, admin shared TS types, and the admin API client, and backwrote architecture doc `166` so template-side `revision / approval / compare / clone` is no longer the next `S03` gap

## v0.1.33 - Commercial S03 Admin Marketing Budget/Code Lifecycle

- Date: 2026-04-10
- Type: patch
- Highlights:
  - promoted canonical budget control to semantic `activate / close` APIs and canonical coupon code control to semantic `disable / restore` APIs, keeping legacy `/status` routes only as compatibility
  - persisted and exposed budget/code lifecycle audit for both applied and rejected decisions, carrying `operator_id / request_id / reason / decision_reasons / requested_at_ms`
  - enforced coupon-first governance guardrails where budget activation requires headroom and non-closed campaign context, while expired code cannot be restored and runtime-owned `reserved / redeemed` code states stop behaving like normal admin toggles
  - synchronized OpenAPI, admin shared types, and the admin API client with `CampaignBudget*` and `CouponCode*` lifecycle schemas and methods, and updated architecture doc `166` so `budget / code` is no longer the next `S03` gap

## v0.1.32 - Commercial S03 Admin Marketing Lifecycle Audit

- Date: 2026-04-10
- Type: patch
- Highlights:
  - persisted template/campaign lifecycle audit for applied and rejected decisions on canonical marketing truth, carrying `operator_id / request_id / reason / decision_reasons / requested_at_ms`
  - added dedicated admin audit list APIs plus sqlite/postgres audit tables and indexes, so lifecycle evidence is now queryable rather than response-only
  - synchronized OpenAPI, admin shared types, and the admin API client with `CouponTemplateLifecycleAuditRecord` and `MarketingCampaignLifecycleAuditRecord`
  - backwrote architecture doc `166` so repo truth now treats template/campaign lifecycle audit as closed and re-ranks `budget / code` plus `revision / approval / compare / clone` as the next `S03` gaps

## v0.1.31 - Commercial S03 Admin Marketing Coupon Template Lifecycle

- Date: 2026-04-10
- Type: patch
- Highlights:
  - added semantic admin template lifecycle mutations on canonical coupon template ids, so operators can now `publish / schedule / retire` coupon definitions instead of overloading a generic `status` toggle
  - introduced `CouponTemplateMutationResult { detail, audit }` plus additive `CouponTemplateStatus::Scheduled` and `activation_at_ms`, making definition-side activation timing explicit inside the coupon-first `market` model
  - synchronized admin/portal shared types and the admin API client to the new template lifecycle contract, preventing frontend-facing coupon semantics from falling behind backend truth
  - backwrote architecture doc `166` to reflect that marketing no longer only exposes `create + status toggle`; template/campaign lifecycle is now semantic while budget/code remain the next governance gap

## v0.1.30 - Commercial S03 Admin Marketing Campaign Lifecycle

- Date: 2026-04-10
- Type: patch
- Highlights:
  - added semantic admin campaign lifecycle mutations on canonical marketing campaign ids, so operators can now `publish / schedule / retire` coupon campaigns instead of mutating a generic `status` field with hidden timing rules
  - introduced `MarketingCampaignMutationResult { detail, audit }`, returning campaign truth, linked coupon template context, lifecycle actionability, and operator reason in one response so coupon semantics stay explicit inside `market`
  - enforced coupon-first lifecycle guardrails where inactive templates block `publish / schedule`, future `start_at_ms` blocks `publish`, and `retire` converges campaigns to `ended`
  - kept legacy `/status` compatibility while moving the admin contract toward semantic lifecycle actions, and verified the full `sdkwork-api-interface-admin` package stays green

## v0.1.29 - Commercial S01 Admin Publication Mutation Audit

- Date: 2026-04-10
- Type: patch
- Highlights:
  - added semantic admin publication lifecycle mutations on canonical publication ids, so operators can now `publish / schedule / retire` from the publication surface instead of leaking governed pricing-plan ids into workflow tooling
  - introduced `CommercialCatalogPublicationMutationResult { detail, audit }` and persisted `CatalogPublicationLifecycleAuditRecord`, making publication lifecycle decisions durable with operator id, request id, operator reason, before-after status, and rejected decision reasons
  - hardened governed `planned` publication semantics so repeat `schedule` is blocked as `publication is already scheduled` while outward publication status stays `draft`, matching canonical catalog behavior
  - repaired a stale sqlite admin integration test uncovered by full-package verification, aligning admin `model-price` setup with the current `provider-model` prerequisite and restoring green `sdkwork-api-interface-admin` package validation

## v0.1.28 - Commercial S01 Admin Publication Actionability

- Date: 2026-04-10
- Type: patch
- Highlights:
  - extended admin publication detail with additive `actionability.publish / schedule / retire`, so operators can evaluate publication lifecycle readiness from one canonical read model instead of inferring pricing lifecycle rules manually
  - derived publication actionability from canonical publication status, governed pricing-plan presence, governed pricing-rate presence, and effective time while returning explicit block reasons for non-actionable states
  - hardened publication regressions around real future `effective_from_ms` semantics, preventing stale pseudo-future test data from masking lifecycle behavior drift
  - updated commercialization loop evidence under `docs/step`, `docs/review`, and `docs/release`, keeping S01 admin-publication-control convergence traceable by loop and version

## v0.1.27 - Commercial S01 Admin Publication Detail

- Date: 2026-04-10
- Type: patch
- Highlights:
  - added additive `GET /admin/commerce/catalog-publications/{publication_id}`, so admin can now load one canonical publication with its full governed pricing context instead of joining pricing evidence manually
  - introduced `CommercialCatalogPublicationDetail { projection, governed_pricing_plan, governed_pricing_rates }`, keeping publication semantics explicit while exposing the exact pricing-plan evidence backing the current revision
  - resolved governed publication detail by canonical commercial `plan_code + publication_version`, avoiding admin-side inference from unstable ids or client-side joins
  - updated commercialization loop evidence under `docs/step`, `docs/review`, and `docs/release`, keeping S01 admin-publication-detail convergence traceable by loop and version

## v0.1.26 - Commercial S01 Admin Publication Projection

- Date: 2026-04-10
- Type: patch
- Highlights:
  - added additive `GET /admin/commerce/catalog-publications`, so admin now has a first-class read model for canonical `ApiProduct -> ProductOffer -> CatalogPublication` truth instead of inferring publication state from pricing-plan records
  - introduced `CommercialCatalogPublicationProjection { product, offer, publication }`, reusing canonical domain schemas rather than creating a duplicate admin-only publication contract
  - opened shared catalog projection reuse through `current_canonical_commercial_catalog_for_store(...)` and synchronized pricing lifecycle before admin projection, so operator visibility now follows the same governed publication truth as runtime commerce
  - updated commercialization loop evidence under `docs/step`, `docs/review`, and `docs/release`, keeping S01 admin-publication convergence traceable by loop and version

## v0.1.25 - Commercial S01 Publication-Revision Governance

- Date: 2026-04-10
- Type: patch
- Highlights:
  - upgraded canonical `CatalogPublication` with additive `publication_revision_id / publication_version / publication_source_kind / publication_effective_from_ms`, so publication governance is now explicit revision evidence instead of only `publication_status`
  - added deterministic publication revision identity `publication_revision:<publication_kind>:offer:<product_kind>:<target_id>:v<version>` and derived it from governed pricing-plan truth or catalog-seed fallback without storage-schema churn
  - threaded publication revision evidence through portal catalog offers, commerce quotes, order snapshots, order read views, and portal OpenAPI while preserving existing `pricing_plan_*` compatibility
  - updated commercialization loop evidence under `docs/step`, `docs/review`, and `docs/release`, keeping S01 publication-governance convergence traceable by loop and version

## v0.1.24 - Commercial S01 Plan-Code Governance

- Date: 2026-04-10
- Type: patch
- Highlights:
  - introduced shared commercial pricing-code normalization in `sdkwork-api-app-catalog`, so product-bound pricing codes now converge to canonical `<product_kind>:<target_id>` instead of relying on operator-perfect string entry
  - upgraded canonical catalog governance matching to use the shared normalization helper, allowing historical commercial code variants to align with runtime catalog truth without a storage migration
  - upgraded admin pricing-plan create/update to persist normalized commercial codes while preserving generic non-commercial `plan_code` compatibility, and added explicit rejection for malformed commercial codes with empty `target_id`
  - updated commercialization loop evidence under `docs/step`, `docs/review`, and `docs/release`, keeping S01 plan-code governance convergence traceable by loop and version

## v0.1.23 - Commercial S01 Pricing Governance Overlay

- Date: 2026-04-10
- Type: patch
- Highlights:
  - introduced canonical commercial `plan_code` alignment `<product_kind>:<target_id>`, so pricing governance can now safely project onto outward commercial catalog semantics without replacing the existing compatibility `pricing_plan_id`
  - upgraded canonical catalog building and portal commerce quote generation to overlay governed `pricing_plan_version / publication_status` from aligned pricing-plan lifecycle records, while keeping the old fallback `version=1 / published` path for non-aligned or legacy cases
  - added an `AdminStore -> AccountKernelStore` bridge for sqlite/postgres-backed commerce reads, allowing portal catalog and quote flows to consume pricing governance without widening router state types or forking a second store abstraction
  - updated commercialization loop evidence under `docs/step`, `docs/review`, and `docs/release`, keeping S01 pricing-governance convergence traceable by loop and version

## v0.1.22 - Commercial S01 Outward Order Owner Chain

- Date: 2026-04-10
- Type: patch
- Highlights:
  - extended outward portal order reads with additive canonical `product_id / offer_id / publication_id / publication_kind / publication_status`, so catalog-owner semantics now reach order detail and order-center instead of stopping inside quote/snapshot evidence
  - promoted reusable app-layer `PortalCommerceCatalogBinding` and reused it from the portal interface, avoiding a second snapshot-parsing truth path
  - aligned `PortalCommerceOrder` OpenAPI and portal shared TS contracts with runtime order payloads, including the previously missing `pricing_plan_id / pricing_plan_version` plus new owner-chain fields
  - updated commercialization loop evidence under `docs/step`, `docs/review`, and `docs/release`, keeping S01 outward-order convergence traceable by loop and version

## v0.1.21 - Commercial S01 Catalog Publication Owner Chain

- Date: 2026-04-10
- Type: patch
- Highlights:
  - extended portal catalog offers and portal quotes with additive canonical `product_id / offer_id / publication_id / publication_kind / publication_status`, so portal-facing commerce payloads now carry explicit `ApiProduct -> ProductOffer -> CatalogPublication` owner chain instead of relying on local inference
  - upgraded quote binding resolution from pricing-only to full catalog-owner binding and added explicit `catalog_binding` evidence into order snapshots, tightening quote-time and order-time auditability without storage-schema churn
  - made settlement-side quote reconstruction prefer stored owner-chain evidence while retaining a compatibility fallback for older snapshots
  - updated commercialization loop evidence under `docs/step`, `docs/review`, and `docs/release`, keeping S01 publication-owner convergence traceable by loop and version

## v0.1.20 - Commercial S01 Quote Pricing Evidence

- Date: 2026-04-10
- Type: patch
- Highlights:
  - extended `PortalCommerceQuote` with additive `pricing_plan_id / pricing_plan_version / pricing_rate_id / pricing_metric_code`, so quote payload now exposes the same canonical pricing identity already carried by catalog offer truth
  - changed portal order creation to persist pricing-plan identity from quote truth and added explicit `pricing_binding` evidence into `pricing_snapshot_json`, tightening quote-time auditability without storage-schema churn
  - verified recharge-pack and custom-recharge quote parity through Rust regressions, re-checked portal OpenAPI inventory exposure, and updated portal shared TS quote contracts
  - updated commercialization loop evidence under `docs/step`, `docs/review`, and `docs/release`, keeping S01 quote-evidence convergence traceable by loop and version

## v0.1.19 - Commercial S01 Offer Pricing Binding

- Date: 2026-04-10
- Type: patch
- Highlights:
  - extended canonical `ProductOffer` semantics with additive `pricing_plan_id / pricing_plan_version / pricing_rate_id / pricing_metric_code`, so offer truth now carries explicit pricing binding instead of only a display label
  - added deterministic pricing binding generation in `sdkwork-api-app-catalog` and exposed those bindings through portal catalog `offers` plus portal shared TS types
  - fixed portal order creation so `pricing_plan_id` now resolves from canonical offer pricing truth, eliminating the previous misuse of subscription `target_id` as a pricing-plan identity
  - updated commercialization loop evidence under `docs/step`, `docs/review`, and `docs/release`, keeping S01 pricing-convergence progress traceable by loop and version

## v0.1.18 - Commercial S01 Domain/App Canonical Commercial Catalog

- Date: 2026-04-10
- Type: patch
- Highlights:
  - introduced shared `ApiProduct / ProductOffer / CatalogPublication / CommercialCatalog` semantics in `sdkwork-api-domain-catalog`, giving S01 an explicit domain owner for canonical commercial catalog truth
  - added `sdkwork-api-app-catalog` canonical seed builders and rewired `sdkwork-api-app-commerce` so portal catalog `products / offers` now derive from the shared domain/app model instead of local compatibility assembly
  - repaired the sqlite compile blockers exposed by fresh-loop verification and removed the leftover provider-account decoder warning debt, so the new canonical catalog slice can be rebuilt and proven without warm-state artifacts
  - updated commercialization loop evidence under `docs/step`, `docs/review`, and `docs/release`, keeping S01 progress traceable by loop and version

## v0.1.17 - Commercial S01 Catalog Product Offer Layer

- Date: 2026-04-10
- Type: patch
- Highlights:
  - added additive `products` and `offers` to portal commerce catalog as a canonical product/offer truth layer, while preserving legacy `plans`, `packs`, `recharge_options`, `custom_recharge_policy`, and `coupons`
  - introduced `PortalApiProduct` and `PortalProductOffer` across Rust DTOs, portal OpenAPI, and shared portal TypeScript contracts so coupon-first commercialization can evolve toward explicit `ApiProduct / ProductOffer / Quote / Transaction` semantics without immediate storage churn
  - published `/portal/commerce/catalog` in the manual portal OpenAPI and verified the new catalog surface, quote flow compatibility, and TS contract exposure through Rust tests plus direct Node assertions
  - updated commercialization loop evidence under `docs/step`, `docs/review`, and `docs/release`, keeping S01 progress traceable by loop and version

## v0.1.16 - Commercial S01 Coupon Semantics Split

- Date: 2026-04-10
- Type: patch
- Highlights:
  - added additive `product_kind` and `quote_kind` to portal commerce quote output, keeping legacy `target_kind` wire compatibility while separating API product purchase semantics from coupon redemption semantics
  - added additive `product_kind` and `transaction_kind` to portal commerce order responses through an interface-layer view model, avoiding storage-schema churn while making coupon-first marketing and commerce boundaries explicit
  - split portal TypeScript contract semantics into `PortalCommerceTargetKind`, `PortalMarketingTargetKind`, `ApiProductKind`, `PortalQuoteKind`, and `CommercialTransactionKind`, while retaining `PortalCommerceQuoteKind` as a compatibility alias
  - updated portal OpenAPI order schema and added commercialization loop evidence under `docs/step`, `docs/review`, and `docs/release`

## v0.1.15 - Commercial Step Loop Prompt Final Tightening

- Date: 2026-04-10
- Type: patch
- Highlights:
  - compressed `docs/prompts/反复执行Step指令.md` again into a shorter final execution contract for the commercialization step loop
  - merged repeated prose into denser rules while preserving the hard flow: truth sync, batch selection, batch implementation, per-step closure, release writeback, and next-loop continuation
  - kept the serial backbone, allowed parallel windows, single-writer shared-file controls, mandatory `go / conditional-go / no-go`, and forced `docs/release` updates for every substantive iteration

## v0.1.14 - Commercial Step Loop Prompt Short Core

- Date: 2026-04-10
- Type: patch
- Highlights:
  - compressed `docs/prompts/反复执行Step指令.md` again into a shorter commercial step execution contract
  - removed explanatory redundancy while preserving the hard closure loop: truth sync, batch selection, batch implementation, per-step closure, release writeback, and next-loop continuation
  - kept the serial backbone, allowed parallel windows, single-writer shared-file controls, mandatory `go / conditional-go / no-go`, and required `docs/release` updates for every substantive iteration

## v0.1.13 - Commercial Step Loop Prompt Tightening

- Date: 2026-04-10
- Type: patch
- Highlights:
  - compressed `docs/prompts/反复执行Step指令.md` into a shorter commercialization loop prompt while preserving the hard closure logic
  - kept the accelerated batch-first flow: finish the current unlocked implementation batch first, then enter per-step testing, review, architecture writeback, and release closure
  - retained explicit serial backbone, allowed parallel windows, single-writer shared-contract rules, and mandatory `go / conditional-go / no-go` step exits
  - preserved professional change-log governance under `docs/release`, including dated/versioned loop notes and evidence-oriented release writeback

## v0.1.12 - Commercial Step Loop Prompt Refresh

- Date: 2026-04-10
- Type: patch
- Highlights:
  - rewrote `docs/prompts/反复执行Step指令.md` into a commercialization-specific repeatable execution contract aligned to `docs/step/100-110` and `docs/架构/166/133/03`
  - formalized the accelerated wave loop: finish the current unlocked batch of code implementation first, then enter per-step testing, review, architecture writeback, and release closure
  - made the serial backbone and allowed parallel windows explicit for `S00-S08`, while pinning shared-contract files to single-writer control
  - required professional change-log governance under `docs/release`, including dated and versioned loop notes, verification evidence, rollback context, and remaining-risk disclosure

## Unreleased - Provider Protocol / Plugin Runtime Split

- Date: 2026-04-09
- Type: minor
- Highlights:
  - added explicit `protocol_kind` to provider metadata so external protocol standard and plugin runtime identity are no longer conflated inside `adapter_kind`
  - threaded `protocol_kind` through `/admin/providers`, SQLite, PostgreSQL, and stateless gateway upstream configuration while preserving legacy payload compatibility
  - added native Anthropic and Gemini passthrough at the HTTP gateway edge for both stateless and stateful flows
  - added no-log planned provider preview plus shared planned execution entrypoints so stateful Anthropic/Gemini passthrough and translated fallback now keep the same one-request/one-decision-log/one-provider-plan audit shape
  - canonicalized extension runtime protocol capability to `openai`, `anthropic`, `gemini`, and `custom`, while preserving legacy manifest compatibility for `openrouter` and `ollama`
  - added explicit builtin manifest protocol capability metadata and safe fallback from unresolved `extension_id` to protocol-default runtime only for unbound providers
  - closed the safety gap where blocked explicit plugins could silently degrade to builtin defaults
  - added first-class raw native-dynamic plugin execution for Anthropic and Gemini compat routes, so heterogeneous plugins can return provider-native JSON/SSE without collapsing back into OpenAI translation
  - extended planned execution context with resolved runtime metadata, kept explicit broken bindings fail-closed in stateful compat flows, and now still persist one routing decision log when those failures occur
  - upgraded stateful raw-plugin execution to reuse gateway execution-context controls, upstream outcome metrics, and provider health snapshot persistence instead of bypassing governance
  - taught raw JSON planned execution to honor matched routing retry policy conservatively, with verified timeout->retry->success recovery for native-dynamic compat plugins while keeping opaque plugin failures non-retryable
  - extended the native-dynamic ABI error envelope with optional retry metadata, so plugins can explicitly declare transient failures and `retry_after_ms` without forcing gateway-side message heuristics
  - extended the same explicit transient-error contract to raw SSE startup, so native-dynamic compat streams may retry only before the first content type or chunk is emitted and still remain no-retry after stream output starts
  - corrected raw-plugin fallthrough accounting so non-`native_dynamic` or raw-not-applicable paths return `None` without emitting phantom upstream success metrics or healthy provider snapshots
  - expanded Anthropic/Gemini compat regressions to lock native-dynamic raw stream and count-tokens branches, using isolated broken-binding extension ids so fail-closed behavior cannot be masked by host cache reuse
  - added targeted raw planned-execution governance regressions in `sdkwork-api-app-gateway`, including raw SSE startup retry scheduling, opaque-plugin no-retry coverage, and accounting-neutral raw fallthrough, and re-verified `sdkwork-api-extension-abi`, `sdkwork-api-extension-host`, `sdkwork-api-app-gateway`, `anthropic_messages_route`, `gemini_generate_content_route`, and `cargo check` for the touched runtime chain
  - moved connector runtime startup supervision out of the async compat-planning path and into `spawn_blocking`, preventing current-thread runtime starvation from breaking connector-backed Anthropic/Gemini translated fallback
  - taught connector startup to adopt delayed external health when no local filesystem entrypoint exists, so externally managed connector endpoints can come online within the configured startup budget instead of immediately collapsing into spawn errors
  - removed duplicate connector supervision inside planned execution context construction, so one planned connector request now probes external `/health` once instead of resolving the same execution target twice
  - codified runtime capability boundaries in `ExtensionRuntime`, so raw provider execution and structured retry hints are now explicitly native-dynamic-only instead of being guarded by scattered equality checks
  - added connector raw JSON/SSE accounting-neutral regressions, proving connector runtimes stay off the raw plugin surface while still remaining available on the supervised HTTP path
  - taught planned execution context construction to fail over when the selected connector cannot produce an execution target before relay execution starts, and now rewrite planned decision / usage context to the executable backup provider with `gateway_execution_failover`
  - widened planned execution preflight failover to also cover selected-provider missing tenant credential, and added an Anthropic compat regression that proves the primary upstream is skipped before execution while usage and routing audit both point at the backup provider
  - extended the same missing-tenant-credential preflight failover contract to direct store-relay OpenAI `chat/responses` JSON and SSE, so stateful OpenAI routes now skip the non-credentialed primary upstream before execution and keep usage/routing audit aligned with the backup provider that actually ran
  - fixed a commercial correctness gap where direct store-relay OpenAI `chat/responses` routes could select a standard Anthropic/Gemini provider without an explicit conversion plugin and then silently fabricate local fallback success instead of failing over to the first compatible backup provider
  - added route-selection fallback auditing for policy-declared unavailable candidates, so missing/unavailable primaries now record `policy_candidate_unavailable` while real planning/execution replacement remains `gateway_execution_failover`
  - added additive `/admin/providers.execution` observability driven by gateway provider-resolution truth, so operators can now see implicit-default passthrough, adapter-surface executability, raw-plugin capability, and explicit-binding `fail_closed` risk in one catalog response
  - refined `/admin/providers.execution` with `route_readiness` for `openai`, `anthropic`, and `gemini`, so operators can distinguish matching-standard passthrough from raw-plugin conversion and from true fail-closed families instead of overreading global `fail_closed`
  - corrected default-plugin protocol derivation for `ollama` and `ollama-compatible`, so Ollama remains default-plugin onboardable without being mislabeled as `openai` standard-protocol passthrough; `openrouter` stays `openai`
  - added additive `default_plugin_family` on `POST /admin/providers`, with first-class builtin onboarding for `openrouter` and `ollama`
  - pinned default-plugin onboarding to the canonical builtin `adapter_kind`, derived `protocol_kind`, and derived `extension_id`, and now reject conflicting low-level fields with `400 Bad Request`
  - added additive `/admin/providers.integration` with `mode={standard_passthrough|default_plugin|custom_plugin}` plus `default_plugin_family` when applicable, so onboarding semantics are explicit alongside runtime execution truth
  - added additive `integration` to `POST /admin/providers`, so create-time control-plane callers now receive normalized onboarding truth immediately without needing a follow-up list read
  - moved default-plugin identity normalization into `sdkwork-api-app-catalog`, so provider onboarding policy is reusable application logic instead of an admin-controller-only rule
  - moved reusable `ProviderIntegrationView` derivation into `sdkwork-api-app-catalog`, so admin and portal now share one onboarding-truth contract instead of classifying provider mode independently
  - updated admin routing/model fixtures to create OpenRouter through `default_plugin_family=openrouter`, removing the old `adapter_kind=openai` shortcut from management-side examples
  - converged non-admin routing and relay regressions on the same OpenRouter identity contract, so simulate-route helpers and routing relay coverage now pin `adapter_kind=openrouter`, `protocol_kind=openai`, and `extension_id=sdkwork.provider.openrouter` instead of silently using the official OpenAI runtime identity
  - removed the remaining stateless protocol-truth fork by rewiring `StatelessGatewayUpstream` to reuse shared domain-catalog protocol derive/normalize helpers, keeping stateless config aligned with provider metadata for `openrouter/openai` and `ollama/custom`
  - converged portal and HTTP relay samples on the same default-plugin onboarding contract, so Portal fixtures now mount canonical OpenRouter identity and HTTP relay regressions now create OpenRouter/Ollama through `default_plugin_family`
  - extended portal routing `provider_options` with additive `protocol_kind` plus `integration`, so frontend/provider-choice surfaces can distinguish industrial-standard passthrough, builtin default-plugin onboarding, and custom-plugin normalization directly
  - extended portal routing `provider_options` with additive tenant-scoped `credential_readiness`, so frontend/provider-choice surfaces can now distinguish static onboarding shape from current-tenant credential configuration without mixing secret presence into `integration`
  - extended `GET /admin/providers?tenant_id=...` with additive tenant-scoped `credential_readiness`, while keeping the default unscoped admin provider catalog tenant-agnostic
  - moved shared provider credential-readiness derivation into `sdkwork-api-app-credential`, so admin and portal now consume one app-layer secret-presence contract instead of duplicating readiness strings per transport
  - published `GET /admin/providers` and `POST /admin/providers` in admin OpenAPI with the real provider/integration/execution/credential-readiness schemas, and documented `tenant_id` as an explicit opt-in tenant overlay instead of undocumented behavior
  - added dedicated `GET /admin/tenants/{tenant_id}/providers/readiness`, so tenant-scoped provider readiness now has a focused operator endpoint that returns provider identity, static `integration`, and tenant `credential_readiness` without dragging global `execution` into the tenant overlay surface
  - added first-class stateless default-plugin ingress via `StatelessGatewayUpstream::from_default_plugin_family(...)` and `StatelessGatewayConfig::try_with_default_plugin_upstream(...)`, so non-admin direct onboarding of OpenRouter/Ollama now follows the same config-only builtin plugin-family contract as admin
  - added `create_provider_with_default_plugin_family_and_bindings(...)` to `sdkwork-api-app-catalog`, so non-admin seeders now reuse canonical default-plugin identity plus secondary bindings instead of hand-building OpenRouter fixtures
  - normalized HTTP compat regressions that explicitly probe `/health`, so native-protocol and connector suites now verify protocol/plugin behavior instead of failing on incomplete mock upstream health fixtures

## Unreleased - Release Sync Audit SSH Remote Equivalence

- Date: 2026-04-09
- Type: patch
- Highlights:
  - normalized GitHub remote comparison in `scripts/release/verify-release-sync.mjs` so valid SSH clones no longer fail the release-sync gate only because the expected repository URL is stored in HTTPS form
  - added a red-first regression in `scripts/release/tests/release-sync-audit.test.mjs` and re-verified adjacent release-governance coverage through `materialize-release-sync-audit.test.mjs` and `release-governance-runner.test.mjs`
  - kept governance truth honest: `node scripts/release/run-release-governance-checks.mjs --format json` still returns `ok=false`, `blocked=true`, `passingIds=7`, `blockedIds=3`, `failingIds=[]` because this host still lacks live telemetry input and still blocks Node child-Git execution

## Unreleased - Windows Startup Smoke Follow-up

- Date: 2026-04-09
- Type: patch
- Highlights:
  - restored `crates/sdkwork-api-storage-postgres/tests/integration_postgres/routing_usage.rs` quota-policy coverage as a real integration test by adding the missing `#[tokio::test]`
  - re-verified `cargo test --workspace --no-run -j 1` after the test-entry repair
  - re-verified Windows startup-related repository contracts through `start-dev-windows-backend-warmup`, `start-workspace`, `process-supervision`, `windows-rust-toolchain-guard`, and `router-runtime-tooling` Node smoke suites
  - executed a fresh managed preview launch/stop smoke through `node scripts/dev/start-workspace.mjs` with temporary runtime state and green gateway/admin/portal/web readiness
  - kept release truth honest: direct `PowerShell -> bin/start-dev.ps1` execution remains blocked by this shell session's policy boundary, so this iteration proves the managed runtime path but not the direct PowerShell wrapper layer


## Unreleased - Workspace Test Module Visibility Stabilization

- Date: 2026-04-09
- Type: patch
- Highlights:
  - fixed sibling-module test support visibility in `sdkwork-api-app-runtime/tests/standalone_runtime_supervision/support.rs`, restoring shared runtime supervision fixtures across split child modules
  - fixed sibling-module test support visibility in `sdkwork-api-app-gateway/tests/extension_dispatch/support.rs`, restoring shared extension dispatch fixtures, guards, and native-dynamic helpers across split child modules
  - verified `cargo test -p sdkwork-api-app-runtime --test standalone_runtime_supervision -j 1` at `16 / 16`
  - verified `cargo test -p sdkwork-api-app-gateway --test extension_dispatch -j 1` at `14 / 14`
  - restored a green `cargo test --workspace --no-run -j 1` gate without changing provider passthrough or plugin-adapter runtime behavior
  - kept release truth honest: one dead-code warning remains in `sdkwork-api-storage-postgres`, and startup smoke was not re-run in this iteration

## Unreleased - Step Loop Prompt Governance Refresh

- Date: 2026-04-09
- Type: patch
- Highlights:
  - rewrote `docs/prompts/反复执行Step指令.md` into a project-specific repeatable execution contract aligned with `docs/架构/`、`docs/step/`、`docs/review/`、`docs/release/`
  - enforced a batch-first execution model: finish the current unlocked step code batch first, then enter per-step verification, review writeback, and release-note governance
  - made serial vs parallel boundaries explicit, so repeated prompt reuse can accelerate safe iteration without hiding shared-file, migration, protocol, or release-gate dependencies
  - added professional change-log rules covering date, type, verification, known gaps, and version discipline for every meaningful iteration
  - kept completion truth strict: no “done” state is allowed until step closure, test closure, document closure, and release closure all converge


## Unreleased - Step 10 Release Governance Bundle

- Date: 2026-04-09
- Type: patch
- Highlights:
  - added `scripts/release/materialize-release-governance-bundle.mjs`, which validates and packages the five governed latest artifacts into one restore-oriented bundle plus manifest
  - updated `.github/workflows/release.yml` so `web-release` now uploads `release-governance-bundle-web`
  - updated `scripts/release/release-workflow-contracts.mjs` and related tests so the single-download operator handoff is repository-enforced
  - re-ran the full `scripts/release/tests/*.test.mjs` suite at `76 / 76`
  - kept release truth honest: `node scripts/release/run-release-governance-checks.mjs --format json` on this host still returns `ok=false`, `blocked=true`, `passingIds=7`, `blockedIds=3`, `failingIds=[]`

## Unreleased - Step 10 Release Governance Default Latest CLI Replay

- Date: 2026-04-08
- Type: patch
- Highlights:
  - updated `compute-release-window-snapshot.mjs` and `verify-release-sync.mjs` so restored default latest artifacts are now consumed by the real CLI path before any live Git attempt
  - added red-first regression coverage proving both lanes skip live Git when the repository-owned latest artifacts already exist, and re-verified `release-window-snapshot.test.mjs` at `6 / 6` plus `release-sync-audit.test.mjs` at `3 / 3`
  - re-ran the release-governance aggregate suite at `74 / 74` and manually verified the operator flow on this host: restore `5` governed artifacts, then run `run-release-governance-checks.mjs --format json`, which now returns `ok=true` with no blocked or failing lanes
  - kept release truth honest: default local governance still blocks when latest artifacts or telemetry evidence have not actually been restored or materialized

## Unreleased - Step 10 Release Governance Latest Artifact Restore

- Date: 2026-04-08
- Type: patch
- Highlights:
  - added `scripts/release/restore-release-governance-latest.mjs`, so blocked hosts can now restore downloaded governance artifacts into the default latest paths under `docs/release/`
  - validated restored release-window, release-sync, telemetry export, telemetry snapshot, and SLO evidence artifacts with existing repository contracts, while rejecting conflicting duplicate downloads
  - added `restore-release-governance-latest.test.mjs` and proved that after restoring real latest artifacts, `runReleaseGovernanceChecks()` can replay cleanly even when Node child execution is forced to `EPERM`
  - kept release truth honest: restore rehydrates governed evidence only and does not fabricate fresh Git or telemetry facts

## Unreleased - Step 10 Release Window / Sync Governed Latest Artifacts

- Date: 2026-04-08
- Type: patch
- Highlights:
  - added `scripts/release/materialize-release-window-snapshot.mjs` and `scripts/release/materialize-release-sync-audit.mjs`, so both Git-derived release-governance lanes now have repository-owned latest artifact producers
  - rewired `.github/workflows/release.yml` so native and web release jobs materialize, upload, attest, and explicitly feed `release-window-snapshot-latest.json` and `release-sync-audit-latest.json` into the governance gate
  - updated `run-release-governance-checks.mjs` so blocked-host replay now prefers explicit governed env input, then repository-owned default latest artifacts, before attempting live Git replay
  - extended `verify-release-attestations.mjs`, `release-attestation-verification-contracts.mjs`, `release-workflow-contracts.mjs`, and their tests so `release-window-snapshot` and `release-sync-audit` are now first-class governed evidence subjects
  - re-verified `materialize-release-window-snapshot.test.mjs` at `3 / 3`, `materialize-release-sync-audit.test.mjs` at `3 / 3`, `release-window-snapshot.test.mjs` at `5 / 5`, `release-sync-audit.test.mjs` at `2 / 2`, `release-governance-runner.test.mjs` at `16 / 16`, `release-attestation-verify.test.mjs` at `4 / 4`, and `release-workflow.test.mjs` at `13 / 13`
  - kept release truth honest: local default governance still blocks until latest artifacts or telemetry evidence are actually supplied

## Unreleased - Step 10 Release Telemetry Export Control-Plane Handoff

- Date: 2026-04-08
- Type: patch
- Highlights:
  - added `scripts/release/materialize-release-telemetry-export.mjs` so release telemetry export is now materialized as a governed artifact at `docs/release/release-telemetry-export-latest.json`
  - rewired `.github/workflows/release.yml` so native and web release jobs both materialize, upload, and attest the governed telemetry export before deriving telemetry snapshot and SLO evidence
  - updated `materialize-release-telemetry-snapshot.mjs` so blocked-host replay can auto-discover the default export artifact when explicit telemetry input is absent
  - extended `verify-release-attestations.mjs`, `release-attestation-verification-contracts.mjs`, `release-workflow-contracts.mjs`, and the corresponding tests so `release-telemetry-export` is now part of executable release truth
  - re-verified `materialize-release-telemetry-export.test.mjs` at `3 / 3`, `materialize-release-telemetry-snapshot.test.mjs` at `4 / 4`, `release-governance-runner.test.mjs` at `14 / 14`, `release-attestation-verify.test.mjs` at `4 / 4`, `release-workflow.test.mjs` at `13 / 13`, and the default governance summary at `7` pass / `3` block / `0` fail
  - kept release truth honest: the export artifact boundary is now repository-owned, but the current local host still lacks default live telemetry input and still blocks the Git-exec lanes

## Unreleased - Step 10 Release Sync Audit Governed Input

- Date: 2026-04-08
- Type: patch
- Highlights:
  - extended `verify-release-sync.mjs` with governed input support so multi-repository sync facts can now arrive through `--audit`, `--audit-json`, `SDKWORK_RELEASE_SYNC_AUDIT_PATH`, or `SDKWORK_RELEASE_SYNC_AUDIT_JSON` instead of only through live local Git execution
  - added validation for both governed artifact envelopes and raw sync-audit summaries, while preserving the default live Git path when no governed input is provided
  - updated `run-release-governance-checks.mjs` so the `release-sync-audit` fallback lane can consume governed input through the same environment contract during blocked-host replay
  - re-verified `release-sync-audit.test.mjs` at `2 / 2`, `release-governance-runner.test.mjs` at `13 / 13`, and `release-window-snapshot.test.mjs` at `5 / 5`
  - improved governance closure without faking green: default local truth remains `7` pass / `3` block / `0` fail, governed sync-audit input yields `8` pass / `2` block / `0` fail, and governed sync-audit plus governed release-window input yields `9` pass / `1` block / `0` fail

## Unreleased - Step 10 Release Window Snapshot Governed Input

- Date: 2026-04-08
- Type: patch
- Highlights:
  - extended `compute-release-window-snapshot.mjs` with governed input support so release-window facts can now arrive through `--snapshot`, `--snapshot-json`, `SDKWORK_RELEASE_WINDOW_SNAPSHOT_PATH`, or `SDKWORK_RELEASE_WINDOW_SNAPSHOT_JSON` instead of only through live local Git execution
  - added validation for both governed artifact envelopes and raw snapshot payloads, while preserving the default live Git path when no governed input is provided
  - updated `run-release-governance-checks.mjs` so the `release-window-snapshot` fallback lane can consume governed input through the same environment contract during blocked-host replay
  - re-verified `release-window-snapshot.test.mjs` at `5 / 5`, `release-governance-runner.test.mjs` at `12 / 12`, and `release-sync-audit.test.mjs` at `1 / 1`
  - kept release truth honest: the current host still blocks Node -> Git by default, but the repository now owns a governed evidence ingress for release-window facts instead of pretending the lane is green

## Unreleased - Step 10 Release Live Git Runner Policy Correction

- Date: 2026-04-08
- Type: patch
- Highlights:
  - corrected Windows live Git governance runners so `compute-release-window-snapshot.mjs` and `verify-release-sync.mjs` now use direct `git.exe` execution with `shell: false` instead of routing through `cmd.exe`
  - expanded blocked-execution classification from `EPERM` only to `EPERM|EACCES` across release-window snapshotting, release-sync audit, attestation verification, and the top-level governance runner fallback path
  - hardened release-window and release-sync contract coverage so regressions back to shell-wrapped Windows Git execution now fail repository verification immediately
  - re-verified `release-window-snapshot.test.mjs` at `4 / 4`, `release-sync-audit.test.mjs` at `1 / 1`, `release-attestation-verify.test.mjs` at `4 / 4`, and `release-governance-runner.test.mjs` at `11 / 11`
  - kept release truth honest: the wrapper defect is fixed, but the current local host still blocks direct Node -> Git execution, so the live Git lanes remain blocked until a different ingress path is introduced

## Unreleased - Step 10 Release Windows Installed Runtime Smoke Parity

- Date: 2026-04-08
- Type: patch
- Highlights:
  - added `scripts/release/run-windows-installed-runtime-smoke.mjs` so Windows native release lanes now own a real installed-runtime smoke entrypoint that installs built outputs, runs installed `start.ps1`, probes unified health endpoints, runs installed `stop.ps1`, and emits structured JSON evidence
  - updated `.github/workflows/release.yml` so Windows native release lanes now run the smoke gate, upload `release-governance-windows-installed-runtime-smoke-*`, and generate a dedicated smoke evidence attestation before asset packaging
  - extended `scripts/release/verify-release-attestations.mjs` and `scripts/release/release-attestation-verification-contracts.mjs` so Windows smoke evidence is part of repository-owned attestation verification truth
  - re-verified `run-windows-installed-runtime-smoke.test.mjs` at `2 / 2`, `release-attestation-verify.test.mjs` at `4 / 4`, `release-workflow.test.mjs` at `13 / 13`, `release-governance-runner.test.mjs` at `11 / 11`, and the default governance summary at `7` pass / `3` block / `0` fail
  - kept release truth honest: this slice closes workflow and evidence parity, but it does not claim a hosted end-to-end Windows smoke execution on the current local host

## Unreleased - Step 10 Release Governance Attestation Verification

- Date: 2026-04-08
- Type: patch
- Highlights:
  - added `scripts/release/verify-release-attestations.mjs` so the repository now owns operator-facing attestation verification for governed evidence, Unix smoke evidence, and packaged release assets
  - added `scripts/release/release-attestation-verification-contracts.mjs` and wired `release-attestation-verify-test` into `scripts/release/run-release-governance-checks.mjs`, including fallback support for child-exec-restricted hosts
  - re-verified `release-attestation-verify.test.mjs` at `4 / 4`, `release-governance-runner.test.mjs` at `11 / 11`, `release-workflow.test.mjs` at `13 / 13`, and the default governance summary at `7` pass / `3` block / `0` fail
  - kept release truth honest: this local session added repository-owned verification entrypoints, but it did not verify a real hosted GitHub attestation record end to end and live verification still blocks without governed evidence plus usable `gh` execution

## Unreleased - Step 10 Release Governance Artifact Attestation

- Date: 2026-04-08
- Type: patch
- Highlights:
  - updated `.github/workflows/release.yml` so release jobs now generate build-provenance attestations for governed telemetry/SLO evidence, Unix installed-runtime smoke evidence, and packaged release assets
  - added workflow permissions required by the official attestation flow and guarded attestation execution by repository support rules, using `SDKWORK_RELEASE_ARTIFACT_ATTESTATIONS_ENABLED` for private/internal opt-in
  - hardened `scripts/release/release-workflow-contracts.mjs` and `scripts/release/tests/release-workflow.test.mjs` so missing attestation permissions or attestation steps now fail release verification immediately
  - re-verified `release-workflow.test.mjs` at `13 / 13`, `release-governance-runner.test.mjs` at `10 / 10`, and the default governance summary at `6` pass / `3` block / `0` fail
  - kept release truth honest: this local session encoded attestation into the workflow contract, but it did not execute a hosted GitHub attestation step directly

## Unreleased - Step 10 Release Governance Governed Evidence Artifacts

- Date: 2026-04-08
- Type: patch
- Highlights:
  - updated `.github/workflows/release.yml` so native and web release jobs now upload `docs/release/release-telemetry-snapshot-latest.json` and `docs/release/slo-governance-latest.json` as dedicated `release-governance-*` artifacts instead of leaving them as transient job-local files
  - hardened `scripts/release/release-workflow-contracts.mjs` and `scripts/release/tests/release-workflow.test.mjs` so the governed evidence uploads, their artifact names, and their required order before the governance gate are now part of executable release truth
  - added rejection coverage for workflows that omit telemetry snapshot uploads or SLO evidence uploads, preventing silent regression back to non-persisted governance evidence
  - re-verified `release-workflow.test.mjs` at `11 / 11`, `release-governance-runner.test.mjs` at `10 / 10`, and the default governance summary at `6` pass / `3` block / `0` fail
  - kept release truth honest: governed evidence is now retained, but the repository still lacks a real release-time telemetry export producer and still does not generate artifact attestations

## Unreleased - Step 10 Release Governance Live SLO Entrypoint Materialization

- Date: 2026-04-08
- Type: patch
- Highlights:
  - updated `scripts/release/run-release-governance-checks.mjs` so `release-slo-governance` now replays governed materialization when evidence is missing, instead of stopping at a pre-existing file check
  - the live SLO fallback now reuses a governed telemetry snapshot when available, otherwise materializes snapshot plus SLO evidence from governed telemetry input and then evaluates the live lane
  - upgraded the default blocked reason from generic `evidence-missing` to precise `telemetry-input-missing`, which better reflects the real upstream gap
  - re-verified `release-governance-runner.test.mjs` at `10 / 10`, `materialize-release-telemetry-snapshot.test.mjs` at `4 / 4`, `materialize-slo-governance-evidence.test.mjs` at `5 / 5`, `release-slo-governance.test.mjs` at `5 / 5`, the default governance summary at `6` pass / `3` block / `0` fail, and the telemetry-export-driven governance summary at `7` pass / `2` block / `0` fail
  - kept release truth honest: the single entrypoint now aligns with the materialization chain, but the repository still does not own a real release-time telemetry export producer

## Unreleased - Step 10 Release Unix Installed Runtime Smoke Evidence Artifact

- Date: 2026-04-08
- Type: patch
- Highlights:
  - extended `scripts/release/run-unix-installed-runtime-smoke.mjs` with `--evidence-path` and structured JSON evidence generation for both success and failure cases
  - rewired `.github/workflows/release.yml` so Unix native release lanes now upload `release-governance-unix-installed-runtime-smoke-*` under `artifacts/release-governance/` with `if: ${{ always() && matrix.platform != 'windows' }}`
  - hardened `scripts/release/release-workflow-contracts.mjs` and `scripts/release/tests/release-workflow.test.mjs` so the evidence path and dedicated governance artifact upload are enforced as release-truth contracts
  - re-verified `run-unix-installed-runtime-smoke.test.mjs` at `2 / 2`, `release-workflow.test.mjs` at `9 / 9`, `release-governance-runner.test.mjs` at `9 / 9`, and the live governance summary at `6` pass / `3` block / `0` fail
  - kept release truth honest: this slice closes persisted Unix smoke evidence, but it does not claim a local full built-artifact release smoke run or Windows parity

## Unreleased - Step 10 Release Unix Installed Runtime Smoke Gate

- Date: 2026-04-08
- Type: patch
- Highlights:
  - added `scripts/release/run-unix-installed-runtime-smoke.mjs` so native Unix release lanes now install real build outputs into a temporary runtime home, rewrite `router.env` to random loopback ports, run installed `start.sh`, probe unified gateway/admin/portal health endpoints, and run installed `stop.sh`
  - inserted `Run installed native runtime smoke on Unix` into `.github/workflows/release.yml` after native desktop builds and before `package-release-assets.mjs`, closing the artifact-level gap between build and packaging
  - hardened `scripts/release/release-workflow-contracts.mjs` and `scripts/release/tests/release-workflow.test.mjs` so removing or reordering the Unix installed-runtime gate now breaks release verification immediately
  - added `scripts/release/tests/run-unix-installed-runtime-smoke.test.mjs` to lock the smoke script CLI and plan contract
  - re-verified `release-workflow.test.mjs` at `8 / 8`, `run-unix-installed-runtime-smoke.test.mjs` at `2 / 2`, `release-governance-runner.test.mjs` at `9 / 9`, and the live governance summary at `6` pass / `3` block / `0` fail
  - kept release truth honest: the workflow gate is now real and explicit, but this local session did not claim a full built-artifact Unix smoke execution without release binaries built in-place

## Unreleased - Step 10 Release Governance Telemetry Export Producer

- Date: 2026-04-08
- Type: patch
- Highlights:
  - added export-bundle ingress to `scripts/release/materialize-release-telemetry-snapshot.mjs`, including `SDKWORK_RELEASE_TELEMETRY_EXPORT_PATH` and `SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON`, while preserving direct snapshot override support for manual/local use
  - derived `gateway-availability`, `admin-api-availability`, and `portal-api-availability` directly from raw `sdkwork_http_requests_total` Prometheus text and computed governed burn rates from the existing quantitative SLO baseline
  - kept the remaining `11` targets in `supplemental.targets` instead of shrinking the SLO baseline or falsely claiming direct raw derivation where current metrics are insufficient
  - rewired `.github/workflows/release.yml` and release-workflow contracts so both release jobs now expect `SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON` upstream of the snapshot step
  - re-verified `materialize-release-telemetry-snapshot.test.mjs` at `4 / 4`, `release-workflow.test.mjs` at `7 / 7`, `materialize-slo-governance-evidence.test.mjs` at `5 / 5`, `release-governance-runner.test.mjs` at `9 / 9`, and the live governance summary at `6` pass / `3` block / `0` fail
  - kept release truth honest: the producer boundary is now governed in-repo, but the live release lane still blocks until a real export producer exists

## Unreleased - Step 10 Release Governance Telemetry Snapshot Contract

- Date: 2026-04-08
- Type: patch
- Highlights:
  - added `scripts/release/materialize-release-telemetry-snapshot.mjs` so release jobs now materialize a governed telemetry snapshot into `docs/release/release-telemetry-snapshot-latest.json` before SLO evidence is derived
  - updated `scripts/release/materialize-slo-governance-evidence.mjs` so governed SLO evidence can be derived from the snapshot artifact while preserving direct evidence input for manual/local use
  - rewired `.github/workflows/release.yml` and release-workflow contracts so both release jobs now expect `SDKWORK_RELEASE_TELEMETRY_SNAPSHOT_JSON` upstream and consume `SDKWORK_RELEASE_TELEMETRY_SNAPSHOT_PATH` between the snapshot and SLO steps
  - re-verified `materialize-release-telemetry-snapshot.test.mjs` at `4 / 4`, `materialize-slo-governance-evidence.test.mjs` at `5 / 5`, `release-workflow.test.mjs` at `7 / 7`, `release-governance-runner.test.mjs` at `9 / 9`, and the live governance summary at `6` pass / `3` block / `0` fail
  - kept release truth honest: the snapshot contract is now governed in-repo, but the live SLO lane still blocks until a real snapshot producer exists

## Unreleased - Step 10 Release Governance Live SLO Evidence Lane

- Date: 2026-04-08
- Type: patch
- Highlights:
  - added `scripts/release/materialize-slo-governance-evidence.mjs` so release jobs now materialize governed SLO evidence into `docs/release/slo-governance-latest.json` before the governance gate runs
  - promoted `release-slo-governance` into `scripts/release/run-release-governance-checks.mjs`, including in-process fallback behavior for child-exec-restricted hosts
  - hardened `.github/workflows/release.yml` plus release-workflow contracts so both release jobs must wire `SDKWORK_SLO_GOVERNANCE_EVIDENCE_JSON` into the materializer step
  - fixed UTF-8 BOM evidence parsing for Windows interoperability after command-level verification exposed the defect
  - re-verified `materialize-slo-governance-evidence.test.mjs` at `4 / 4`, `release-governance-runner.test.mjs` at `9 / 9`, `release-workflow.test.mjs` at `7 / 7`, and the live governance summary at `6` pass / `3` block / `0` fail
  - kept release truth honest: the live SLO lane is now wired but still blocks until real evidence is supplied

## Unreleased - Step 10 Release Governance SLO Threshold Baseline

- Date: 2026-04-08
- Type: patch
- Highlights:
  - added a machine-readable quantitative SLO baseline at `scripts/release/slo-governance.mjs` with `14` governed targets across the data, control, and commercial planes plus fixed `1h` and `6h` burn-rate windows
  - added `scripts/release/slo-governance-contracts.mjs`, `scripts/release/tests/release-slo-governance.test.mjs`, and inserted `release-slo-governance-test` into `scripts/release/run-release-governance-checks.mjs`
  - re-verified red-first focused proof at `5 / 5`, the release-governance runner at `8 / 8`, and the full governance summary with `6` passing lanes, `2` blocked live Git lanes, and `0` failing lanes
  - confirmed the new quantitative evaluator does not overclaim live readiness: `node scripts/release/slo-governance.mjs --format json` reports `evidence-missing` until a governed live artifact is exported

## Unreleased - Step 10 Release Governance Observability Gate

- Date: 2026-04-08
- Type: patch
- Highlights:
  - added `scripts/release/observability-contracts.mjs` so request, routing, runtime, and billing observability assets are now part of executable release truth
  - inserted `release-observability-test` into `scripts/release/run-release-governance-checks.mjs` and added fallback support for child-exec-restricted hosts
  - verified the observability lane, release-governance runner, and governance summary without overstating maturity: observability contract proof is closed, while live Git-based release truth and quantitative SLO blocking remained follow-up work

## Unreleased - Portal Billing Payment Reference Anchor Description Productization

- Date: 2026-04-08
- Type: patch
- Highlights:
  - productized the remaining active Portal billing payment-reference detail so the tenant-facing workbench now says `{reference} is the current {provider} / {channel} payment reference for this order`
  - kept checkout presentation composition, repository behavior, and runtime finance behavior unchanged while aligning shared Portal i18n and `zh-CN` coverage
  - re-verified red-first focused proof, Portal `typecheck`, and the full Portal Node suite at `224 / 224`

## Unreleased - Portal Billing Failed Payment Description Productization

- Date: 2026-04-08
- Type: patch
- Highlights:
  - productized the remaining active Portal billing failed-payment lane description so the tenant-facing workbench now says `Failed payment keeps checkout attempts that need coupon updates, a different payment method, or a fresh checkout visible for follow-up`
  - kept failed-payment lane composition, repository behavior, and runtime finance behavior unchanged while aligning shared Portal i18n and `zh-CN` coverage
  - re-verified red-first focused proof, Portal `typecheck`, and the full Portal Node suite at `223 / 223`

## Unreleased - Portal Billing Commercial Account Description Productization

- Date: 2026-04-08
- Type: patch
- Highlights:
  - productized the remaining active Portal billing commercial-account summary description so the tenant-facing workbench now says `Commercial account keeps balance, holds, and account identity visible beside the workspace billing posture`
  - kept commercial-account payloads, repository composition, and runtime finance behavior unchanged while aligning shared Portal i18n and `zh-CN` coverage
  - re-verified red-first focused proof, Portal `typecheck`, and the full Portal Node suite at `222 / 222`

## Unreleased - Portal Billing Formal Checkout Attempt Description Productization

- Date: 2026-04-08
- Type: patch
- Highlights:
  - productized the remaining active Portal billing checkout-attempt history description so the tenant-facing workbench now says `Checkout attempts keep checkout access, retries, and checkout references visible inside the same workbench`
  - kept payment-attempt payloads, repository composition, latest-attempt logic, and runtime finance behavior unchanged while aligning shared Portal i18n and `zh-CN` coverage
  - re-verified red-first focused proof, Portal `typecheck`, and the full Portal Node suite at `221 / 221`

## Unreleased - Portal Billing Formal Checkout Wording Productization

- Date: 2026-04-08
- Type: patch
- Highlights:
  - productized the remaining active Portal billing `formal checkout` wording so the tenant-facing workbench now says `Checkout workbench keeps checkout access, selected reference, and payable price aligned under one payment method`, `No checkout guidance is available for this order yet`, and checkout-focused provider launch status messages
  - kept checkout payloads, repository composition, launch decisions, and runtime finance behavior unchanged while aligning shared Portal i18n and `zh-CN` coverage
  - re-verified red-first focused proof, Portal `typecheck`, and the full Portal Node suite at `221 / 221`

## Unreleased - Portal Billing Fallback Reason Description Productization

- Date: 2026-04-08
- Type: patch
- Highlights:
  - productized the remaining Portal billing fallback-reason description so the tenant-facing workbench now says `Fallback reasoning stays visible so you can distinguish degraded routing from the preferred routing path`
  - kept billing analytics payloads, repository composition, and runtime finance behavior unchanged while aligning shared Portal i18n and `zh-CN` coverage
  - re-verified red-first focused proof, Portal `typecheck`, and the full Portal Node suite at `221 / 221`

## Unreleased - Portal Billing Payment History Refund Status Productization

- Date: 2026-04-08
- Type: patch
- Highlights:
  - productized the remaining Portal billing payment-history description so the tenant-facing finance workbench now says `refund status` instead of `refund closure`
  - kept payment-history payloads, billing repository composition, and runtime finance behavior unchanged while aligning shared Portal i18n and `zh-CN` coverage
  - re-verified red-first focused proof, Portal `typecheck`, and the full Portal Node suite at `221 / 221`

## Unreleased - Portal Billing Refund History Description Productization

- Date: 2026-04-08
- Type: patch
- Highlights:
  - productized the remaining Portal billing refund-history description so the tenant-facing finance workbench now says `Refund history keeps completed refund outcomes, payment method evidence, and the resulting order status visible without reopening each order`
  - kept refund-history payloads, billing repository composition, and runtime finance behavior unchanged while aligning shared Portal i18n and `zh-CN` coverage
  - re-verified red-first focused proof, Portal `typecheck`, and the full Portal Node suite at `221 / 221`

## Unreleased - Portal Billing Payment Attempt Vocabulary Productization

- Date: 2026-04-08
- Type: patch
- Highlights:
  - productized the remaining Portal billing `payment attempt` checkout-history wording so the tenant-facing workbench now says `Checkout attempts` and related checkout-attempt guidance
  - kept `payment_attempt_id`, canonical payment-attempt payloads, launch decisions, and runtime checkout behavior unchanged while aligning shared Portal i18n and `zh-CN` coverage
  - re-verified red-first focused proof, Portal `typecheck`, and the full Portal Node suite at `221 / 221`

## Unreleased - Portal Billing Provider Checkout Action Vocabulary Productization

- Date: 2026-04-08
- Type: patch
- Highlights:
  - productized the remaining Portal billing `provider checkout` action wording so the tenant-facing workbench now says `Opening checkout...`, `Open checkout link`, `Start checkout`, and `Resume checkout`
  - kept payment-attempt launch decisions, checkout URL sourcing, and runtime provider handoff behavior unchanged while aligning shared Portal i18n and `zh-CN` coverage
  - re-verified red-first focused proof, Portal `typecheck`, and the full Portal Node suite at `221 / 221`

## Unreleased - Portal Billing Payment Update Reference Productization

- Date: 2026-04-08
- Type: patch
- Highlights:
  - productized the remaining Portal billing payment-history `Provider event` label so the tenant-facing finance workbench now says `Payment update reference`
  - kept `provider_event_id`, payment-history row contracts, repository composition, and runtime behavior unchanged while aligning shared Portal i18n and `zh-CN` coverage
  - re-verified red-first focused proof, Portal `typecheck`, and the full Portal Node suite at `221 / 221`

## Unreleased - Portal Billing Checkout Session Vocabulary Productization

- Date: 2026-04-08
- Type: patch
- Highlights:
  - productized the remaining Portal billing checkout-session wording so the tenant-facing workbench now says `Checkout details`, `Open checkout`, `Manual step`, `Hosted checkout flow`, and `QR checkout flow` instead of `session`-oriented terminology
  - kept `session_kind`, checkout payloads, repository composition, and payment runtime behavior unchanged while tightening shared Portal i18n and `zh-CN` coverage
  - re-verified red-first focused proof, Portal `typecheck`, and the full Portal Node suite at `221 / 221`

## Unreleased - Portal Billing Checkout Evidence Label Productization

- Date: 2026-04-08
- Type: patch
- Highlights:
  - productized the remaining Portal billing checkout-evidence labels so the tenant-facing workbench now says `Checkout reference` and `QR code content` instead of `Session reference` and `QR payload`
  - kept `session_reference`, `qr_code_payload`, repository composition, and payment runtime behavior unchanged while tightening shared Portal i18n and `zh-CN` coverage
  - re-verified red-first focused proof, Portal `typecheck`, and the full Portal Node suite at `221 / 221`

## Unreleased - Portal Billing Verification Method Display Productization

- Date: 2026-04-08
- Type: patch
- Highlights:
  - productized the remaining Portal billing verification-strategy values so checkout-method evidence now renders readable labels such as `Manual confirmation`, `Stripe signature check`, and `WeChat Pay RSA-SHA256 check` instead of raw strategy codes
  - kept `webhook_verification` source data, repository composition, and payment runtime behavior unchanged while adding a display-only verification-label mapping on the billing page
  - re-verified red-first focused proof, Portal `typecheck`, and the full Portal Node suite at `221 / 221`

## Unreleased - Portal Billing Sandbox Surface Vocabulary Productization

- Date: 2026-04-08
- Type: patch
- Highlights:
  - productized the remaining Portal billing sandbox title, selector, and verification labels so the tenant-facing surface now says `Payment outcome sandbox`, `Sandbox method`, and `Verification method` instead of low-level `event` / `target` / `signature` wording
  - rewrote the active sandbox status sentence to product-facing outcome guidance while keeping provider selection, simulation behavior, and `webhook_verification` sourcing unchanged
  - re-verified red-first focused proof, Portal `typecheck`, and the full Portal Node suite at `221 / 221`

## Unreleased - Portal Billing Payment Outcome Sandbox Productization

- Date: 2026-04-08
- Type: patch
- Highlights:
  - productized the remaining Portal billing sandbox provider-event replay wording so badges, guidance, progress feedback, and action buttons now speak in payment-outcome language
  - kept runtime payment behavior unchanged while aligning shared Portal i18n, `zh-CN`, and the broader payment-rails proof lane with the new payment-outcome vocabulary
  - re-verified focused tests, Portal `typecheck`, recovered one stale proof lane exposed at `220 / 221`, and finished with the full Portal Node suite at `221 / 221`

## Unreleased - Portal Billing Provider Handoff Vocabulary Productization

- Date: 2026-04-08
- Type: patch
- Highlights:
  - productized the remaining Portal billing `Provider handoff` wording so the tenant-facing action label and related explanations now describe `Checkout access` instead of the runtime action concept
  - kept runtime payment behavior unchanged while aligning shared Portal i18n and `zh-CN` coverage with the new checkout-access language
  - tightened the Portal billing i18n and product proof lanes, then re-verified focused tests, Portal `typecheck`, and the full Portal Node suite at `221 / 221`

## Unreleased - Portal Billing Callback Confirmation Vocabulary Productization

- Date: 2026-04-08
- Type: patch
- Highlights:
  - productized the remaining Portal billing replay outcome wording so settled, failed, and canceled statuses now describe provider payment confirmation instead of callback flow mechanics
  - kept runtime payment behavior unchanged while aligning shared Portal i18n and `zh-CN` coverage with the new payment-confirmation language
  - tightened the Portal billing i18n and product proof lanes, then re-verified focused tests, Portal `typecheck`, and the full Portal Node suite at `221 / 221`

## Unreleased - Portal Billing Payment Method Vocabulary Productization

- Date: 2026-04-08
- Type: patch
- Highlights:
  - productized the remaining Portal billing `rail` vocabulary so the workbench now presents `Payment method`, `Primary method`, `Event target`, and `Choose event target` instead of internal routing terminology
  - added the missing shared Portal `Payment method` i18n key and aligned `zh-CN` coverage so the new payment-method vocabulary no longer falls back to English
  - tightened the Portal billing i18n and product proof lanes, then re-verified focused tests, Portal `typecheck`, and the full Portal Node suite at `221 / 221`

## Unreleased - Portal Billing Checkout Metadata Productization

- Date: 2026-04-08
- Type: patch
- Highlights:
  - productized the remaining Portal billing checkout metadata labels so method cards now present `Manual action`, `Provider events`, `Event signature`, `Refund coverage`, and `Partial refunds` instead of operator/webhook/refund-support terminology
  - kept payment capabilities and runtime behavior unchanged while updating shared Portal i18n plus `zh-CN` coverage for the new checkout metadata vocabulary
  - tightened the Portal payment-rails and billing-i18n proof lanes, then re-verified focused tests, Portal `typecheck`, and the full Portal Node suite at `221 / 221`

## Unreleased - Portal Billing Settlement And Sandbox Posture Productization

- Date: 2026-04-08
- Type: patch
- Highlights:
  - renamed the Portal billing `Commercial settlement rail` surface to `Settlement coverage` and rewrote its description so benefit lots, credit holds, and request capture now read as one billing snapshot instead of an operator-facing posture
  - relabeled the explicit billing simulation surface as `Payment event sandbox`, including the sandbox badge, rail selector, active-rail sentence, and replay action buttons, so the remaining simulation tooling has a clearer product boundary
  - updated shared Portal i18n and `zh-CN` coverage plus the billing workspace/product proof lanes, then re-verified focused tests, Portal `typecheck`, and the full Portal Node suite at `221 / 221`

## Unreleased - Portal Billing Payment Journey Copy Productization

- Date: 2026-04-08
- Type: patch
- Highlights:
  - productized the remaining Portal billing and recharge payment-journey copy so checkout guidance now points to the checkout workbench / checkout completion flow instead of settlement-oriented wording
  - renamed the recharge CTA to `Open billing workbench`, updated no-membership guidance to `Complete a subscription checkout`, and rewrote the pending / failed / payment-history descriptions away from operator-console language
  - added shared Portal i18n and `zh-CN` coverage for the new wording, relaxed the last billing-i18n formatter-sensitive regex, and re-verified Portal `typecheck` plus the full Node suite at `221 / 221`

## Unreleased - Portal Billing Queue Action Workbench Boundary Closure

- Date: 2026-04-08
- Type: patch
- Highlights:
  - removed queue-row `Settle order` and `Cancel order` actions from the default Portal billing `Pending payment queue`, keeping those explicit bridge/manual actions inside the opened checkout workbench instead of the queue inventory surface
  - updated post-order billing guidance so users are told to open the checkout workbench to complete payment before quota or membership changes are applied, and added the matching shared Portal i18n plus `zh-CN` coverage
  - repaired the related billing i18n source-contract assertion, then re-verified the focused billing/product suites, Portal `typecheck`, and the full Portal Node suite at `221 / 221`

## Unreleased - Portal Billing Operator Bridge Visibility Closure

- Date: 2026-04-08
- Type: patch
- Highlights:
  - filtered the Portal billing checkout workbench so `settle_order` no longer appears in the default checkout method list when payment simulation is off, keeping operator bridge posture out of the main user-facing payment grid
  - replaced the remaining billing page `Operator settlement` copy with `Manual settlement` and rebuilt the secondary `Payment rail` panel so it now stays focused on formal primary rail, selected reference, and payable price
  - added shared Portal i18n and `zh-CN` coverage plus source-contract regressions for the new visibility boundary, then re-verified the focused billing/product suites, Portal `typecheck`, and the full Portal Node suite at `221 / 221`

## Unreleased - Portal Billing Formal Checkout Presentation Shell

- Date: 2026-04-08
- Type: patch
- Highlights:
  - added `buildBillingCheckoutPresentation(...)` so the Portal billing workbench now composes checkout reference, provider/channel, status, guidance, and payable-price shell facts from canonical order, payment-method, and payment-attempt truth before compatibility checkout-session fallback
  - updated the pending-payment workbench to render formal-first `Primary rail`, `Current status`, guidance copy, selected reference, and payable price while intentionally keeping operator settlement and callback rehearsal as bridge behavior
  - added shared Portal i18n and `zh-CN` coverage for the new shell copy, re-verified the focused billing/product suites plus Portal `typecheck`, and restored the full Portal Node proof lane to green after one residual product-contract fallback reference was reintroduced explicitly

## Unreleased - Portal Billing Checkout Retry Reopen Decision Clarity

- Date: 2026-04-08
- Type: patch
- Highlights:
  - formalized provider checkout launch posture in the Portal billing workbench through `buildBillingCheckoutLaunchDecision(...)`, so canonical payment attempts now classify provider handoff as `resume_existing_attempt`, `create_retry_attempt`, or `create_first_attempt` instead of leaving the behavior implicit inside the page
  - updated the Portal billing checkout method cards and in-flight status messaging to explain whether the workbench is reopening an existing provider checkout, retrying with a fresh attempt, or launching the first attempt, and aligned the CTA labels with that decision
  - added shared Portal i18n and `zh-CN` coverage plus source-contract/service tests for the new retry-versus-reopen decision copy, then re-verified the focused Portal commercial/i18n suites, full Portal Node suite, and Portal typecheck on Windows

## Unreleased - Admin Typecheck Readability Recovery

- Date: 2026-04-08
- Type: patch
- Highlights:
  - added a repo-owned `scripts/dev/run-tsc-cli.mjs` entry and switched the admin frontend `typecheck` contract to it, so admin typecheck no longer depends on the unreadable app-local `typescript@6.0.2` pnpm bin shim in the current Windows sandbox
  - replaced the admin root's unreadable local `node` / `vite/client` type dependency path with repository-owned readable shims and sibling declaration routing, and added a real declaration surface for `scripts/dev/vite-runtime-lib.mjs`
  - repaired the newly surfaced admin source typing defects in the admin API, commercial overview, i18n interpolation, and Vite config layers, then re-verified admin typecheck, the full admin Node suite, and the frontend release/runtime helper contract tests

## Unreleased - Admin Commercial I18n Source Contract Closure

- Date: 2026-04-08
- Type: patch
- Highlights:
  - localized the remaining admin commercial, apirouter, and pricing `zh-CN` contract-mirror strings in `apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/i18n.tsx`, removing the last known English placeholder values from the active Step 06 admin proof surface
  - added a page-level `commercialPageCopyContract` in `apps/sdkwork-router-admin/packages/sdkwork-router-admin-commercial/src/index.tsx` so the commercial module entry now owns the operator-facing `t('...')` copy expected by the source-contract suite without changing runtime behavior
  - added the missing order-audit detail key to `apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/i18nTranslationsCommercial.ts`, keeping the real translation catalog aligned with the contract mirror that tests inspect
  - re-verified `apps/sdkwork-router-admin/tests/admin-i18n-coverage.test.mjs` at `5 / 5` passing and the full admin Node suite at `109 / 109` passing with `--experimental-test-isolation=none`

## Unreleased - Portal Claw Theme Source Contract Recovery

- Date: 2026-04-08
- Type: patch
- Highlights:
  - repaired the last remaining Portal frontend red proof by replacing the stale hardcoded claw `@source "../../../../";` assertion with a structural parent-traversal contract that matches the current sibling theme layout without weakening the Portal-versus-claw boundary
  - kept the Portal theme proof strict for `@source "./";` and `@source "../packages";`, and continued rejecting any silent adoption of the claw repo-relative source path
  - re-verified `apps/sdkwork-router-portal/tests/portal-claw-theme-parity.test.mjs`, the full Portal `219 / 219` frontend proof suite, the managed Windows `start-dev.ps1 -DryRun` path, and the real backend warm-up build under `bin/.sdkwork-target-vs2022`

## Unreleased - Portal Billing Payment Attempt History Composition

- Date: 2026-04-08
- Type: patch
- Highlights:
  - updated the Portal billing checkout workbench to surface canonical order-scoped `payment_attempts` directly inside the existing `Checkout session` panel, so retry history, latest-attempt posture, provider reference, and attempt timing are now visible without inventing a separate frontend-only payment console
  - hardened the billing repository against malformed runtime attempt payloads and aligned the pending-order source-contract fixture to the formal `GET /portal/commerce/orders/{order_id}/payment-attempts` route, keeping the Portal proof lane consistent with the canonical backend contract
  - added shared Portal i18n and `zh-CN` coverage for the new payment-attempt history copy, then re-verified the Portal commercial api, billing i18n, product-polish, payment-history, workspace, and TypeScript proof lanes on Windows

## Unreleased - Portal Billing Formal Payment Attempt Launch Composition

- Date: 2026-04-08
- Type: patch
- Highlights:
  - exposed formal order-scoped payment-attempt listing and creation through the Portal TypeScript SDK and billing repository, so the billing checkout workbench can now reopen an existing canonical checkout URL or create a fresh formal payment attempt without inventing a new frontend-only checkout model
  - updated the Portal billing page to use the real payment-attempt launch path for supported Stripe hosted-checkout rails while intentionally keeping compatibility operator settlement and callback simulation as explicit bridge behavior
  - added shared Portal i18n coverage for provider-launch status/button copy and re-verified the Portal commercial api, product-polish, billing i18n, payment-history, workspace, and TypeScript proof lanes on Windows

## Unreleased - Portal Billing Checkout Method Formal Action Composition

- Date: 2026-04-08
- Type: patch
- Highlights:
  - added `buildBillingCheckoutMethods(...)` so the Portal billing checkout panel now normalizes canonical payment methods and latest payment-attempt posture into a formal-first checkout method list instead of trusting compatibility `checkout_session.methods` as the only source
  - updated the pending-payment workbench to render checkout method cards and provider callback rehearsal rails from `checkoutDetail.checkout_methods`, while still keeping the compatibility checkout-session payload available as an operator bridge and fallback source
  - hardened the Portal Node source-contract and repository regression suite so future regressions are caught if billing falls back to compatibility-only method/action identity again

## Unreleased - Portal Billing Pending Payment Formal Detail Composition

- Date: 2026-04-08
- Type: patch
- Highlights:
  - added a `getBillingCheckoutDetail(...)` repository composition path so the Portal billing checkout panel now loads canonical order detail, available payment methods, and latest payment-attempt detail before falling back to the compatibility checkout-session payload
  - updated the Portal billing checkout panel to prefer canonical attempt references and canonical payment-method labels in the pending-payment detail surface while preserving the existing compatibility checkout-session workbench and callback rehearsal flows
  - hardened the Portal Node source-contract suite so regressions are caught if pending-order detail falls back to compatibility checkout-session as the only data source again

## Unreleased - Portal Billing Formal Payment Read Composition

- Date: 2026-04-08
- Type: patch
- Highlights:
  - migrated the Portal billing repository away from treating `GET /portal/commerce/order-center` as the only payment-detail truth source by composing formal `getPortalCommerceOrder(...)`, `listPortalCommercePaymentMethods(...)`, and `getPortalCommercePaymentAttempt(...)` reads on top of the compatibility aggregate
  - rebuilt billing payment/refund history rows from a formal order-payment source model, so canonical order status, payment-attempt reference, and selected payment-method label now survive into the Portal billing audit views while compatibility `payment_events` remain available as the event-evidence bridge
  - hardened the Portal Node source-contract suite so regressions are caught if billing falls back to aggregate-only payment detail assembly again

## Unreleased - Portal Formal Commerce Read APIs

- Date: 2026-04-08
- Type: patch
- Highlights:
  - closed the missing Portal formal commerce read/detail API gap by shipping `GET /portal/commerce/orders/{order_id}`, `GET /portal/commerce/orders/{order_id}/payment-methods`, and `GET /portal/commerce/payment-attempts/{payment_attempt_id}` on the real runtime router
  - promoted the corresponding app-commerce read helpers into the public application boundary, keeping workspace/user ownership checks intact while exposing canonical order, filtered payment-method, and payment-attempt detail reads
  - published the new routes through the Portal OpenAPI contract and the Portal TypeScript SDK/types package so runtime, schema, and frontend caller surfaces now agree on the same formal payment detail paths
  - added red-first regression coverage in Rust and Node for the new formal detail APIs, then re-verified the full `sdkwork-api-interface-portal` commerce suite, Portal OpenAPI route coverage, and the Portal commercial API source-contract test on Windows

## Unreleased - Portal Payment Simulation Posture Hardening

- Date: 2026-04-08
- Type: patch
- Highlights:
  - exposed `payment_simulation_enabled` through the Portal commerce aggregate `order-center` response and aligned the TypeScript portal/billing contracts with the already-hardened checkout-session posture
  - updated the Portal billing workspace to consume the aggregate posture, hide manual settlement and provider callback simulation actions when production posture disables payment simulation, and keep cancel-order handling available for pending orders
  - added regression coverage proving the default production Portal router reports `payment_simulation_enabled = false`, while the explicit lab/test router keeps the compatibility posture available
  - verified the Portal commercial/billing source-contract tests and product-polish tests in Node, and re-ran the full `sdkwork-api-interface-portal` commerce suite successfully on Windows through a serial `cargo test` workaround after the default MSVC parallel debug-link path hit transient `LNK1201`/`LNK1136` PDB write failures in the shared target directory

## Unreleased - Windows Startup Runtime Home Recovery

- Date: 2026-04-08
- Type: patch
- Highlights:
  - added a shared `Write-RouterUtf8File` runtime helper and routed startup plan / pid / managed-state writes through it, so the Windows router entrypoints no longer depend on raw `Set-Content` for generated control files
  - introduced `SDKWORK_ROUTER_DEV_HOME` support in the PowerShell runtime helpers, allowing `start-dev.ps1` and `stop-dev.ps1` to run against an isolated dev runtime home instead of assuming the repository-owned `artifacts/runtime/dev/<platform>` tree
  - verified the real Windows backend warm-up path now builds `admin-api-service`, `gateway-service`, and `portal-api-service` successfully under the managed short target directory `bin/.sdkwork-target-vs2022`
  - added startup-script contract coverage for the new runtime-home override and safe file-write helper, while explicitly skipping the runtime PowerShell spawn probe when Node child-process execution is blocked with `EPERM` in the current sandbox

## Unreleased - Step 06 Admin Portal Test Contract Hardening

- Date: 2026-04-08
- Type: patch
- Highlights:
  - hardened `apps/sdkwork-router-admin/tests/admin-i18n-coverage.test.mjs` so the Step 06 admin i18n proof now resolves readable pnpm package entries and no longer assumes the app-local `typescript` runtime is directly readable in the current Windows sandbox
  - restored the portal redeem workspace to a layered contract by keeping the detail-table slot `portal-redeem-reward-history-table` while also exposing the higher-level wrapper slot `portal-redeem-history-table`, removing a real conflict between the active workspace-polish and product-polish suites
  - moved the redeemed / rolled-back history summary onto shared portal i18n keys and added the matching `zh-CN` translation so the new commercial history evidence line no longer falls back to raw English
  - verified the Step 06 admin and portal frontend proof suites again with the documented `node --test --experimental-test-isolation=none` runner path

## v0.1.11 - Router Startup Recovery and Gateway Media Boundary Cleanup

- Date: 2026-04-08
- Type: patch
- Highlights:
  - restored the Windows `start-dev.ps1` startup path by introducing a shared Vite runtime wrapper and readable-package fallback resolution for the admin and portal apps, so the router dev stack can boot again under the current pnpm workspace layout
  - removed the legacy music and video local fallback surface from `crates/sdkwork-api-app-gateway/src/relay_files_uploads.rs`, keeping media fallback ownership in the canonical `relay_music_video.rs` module instead of duplicating it inside the files/uploads boundary
  - added executable source-boundary regression coverage for the gateway media split in both Rust and Node, and repaired the internal extension-discovery helper visibility needed to compile and run those gateway unit tests cleanly
  - trimmed a set of low-risk unused imports and test-only internal re-exports across config, runtime, commerce, and admin interface crates to reduce startup warning noise without changing business behavior
  - reconnected `sdkwork-api-storage-sqlite` routing decision log serialization to the shared `sqlite_support` codecs, removing a duplicate private implementation from `routing_store.rs` and adding regression coverage for that shared-boundary rule
  - removed currently unconsumed Stripe internal result and webhook metadata fields from `sdkwork-api-app-commerce`, eliminating the remaining payment-provider dead-code warnings while preserving the live refund, reconciliation, and webhook processing paths

## v0.1.10 - Short Core Step Prompt

- Date: 2026-04-07
- Type: patch
- Highlights:
  - compressed `docs/prompts/反复执行Step指令.md` again into a shorter core prompt while preserving the closure, release, and dependency-truth gates
  - reduced the execution loop to a single fixed sequence so repeated runs keep the same self-thinking and self-correction order
  - kept the hard rules for `00-04` serial control, `13` final closure, `11-13` `Release-Truth Lane`, `8.3 / 8.6 / 91 / 95 / 97 / 98`, and mandatory `Unreleased` carry-forward behavior

## v0.1.9 - Prompt Final Tightening

- Date: 2026-04-07
- Type: patch
- Highlights:
  - tightened `docs/prompts/反复执行Step指令.md` again so the prompt stays concise while keeping the hard closure logic intact
  - made document-system self-repair explicit, so repeated execution first fixes `docs/step` / `docs/架构` / `docs/review` / `docs/release` when they no longer support fact-based continuation
  - clarified the batch-first then per-step deep-verification flow to preserve fast iteration without weakening dependency, release, or validation gates

## Unreleased - Step 06 Release Blocker and Dependency Sync Audit

- Date: 2026-04-07
- Status: blocked / unpublished
- Type: hold
- Highlights:
  - formalized that no new `main` commit, `git push`, or GitHub release is allowed until `sdkwork-api-router` remote access is verifiable and the dependent repositories `sdkwork-core`, `sdkwork-ui`, `sdkwork-appbase`, and `sdkwork-im-sdk` are clean and synchronized
  - closed the previous release-workflow gap where frontend installs could start before GitHub-backed sibling repositories were materialized in CI
  - expanded `scripts/release/materialize-external-deps.mjs` from a single-repository helper into a governed release-dependency materializer for `sdkwork-core`, `sdkwork-ui`, `sdkwork-appbase`, and `sdkwork-im-sdk`, while preserving local relative-path development dependencies
  - added a tested `scripts/release/verify-release-sync.mjs` audit script so the repository-sync gate for `sdkwork-api-router`, `sdkwork-core`, `sdkwork-ui`, `sdkwork-appbase`, and `sdkwork-im-sdk` is now executable instead of documentation-only
  - replayed the release-governance checks in the current sandbox and confirmed that default `node --test` hits `spawn EPERM`, the documented `--experimental-test-isolation=none` path passes, and the live sync audit still reports `command-exec-blocked`
  - added `scripts/release/run-release-governance-checks.mjs` plus contract coverage so the documented release test chain and live sync audit can be executed through one stable repository entry point
  - added in-process release governance contract helpers so the governance runner now keeps contract checks green in the current sandbox while preserving the live sync audit as the only blocking lane
  - repaired the operator-facing `scripts/release/verify-release-sync.mjs --format text` path, which previously crashed on an undefined variable instead of printing the release-sync block report
  - documented the stable release-governance entry points and sandbox-safe verification commands directly in `docs/release/README.md`
  - added an external-release-dependency coverage audit inside `scripts/release/materialize-external-deps.mjs`, so admin / portal sibling references are now scanned and proven to be fully covered by declared GitHub materialization specs
  - locked the current release-app external sibling dependency surface to `sdkwork-ui` through workflow contracts, preventing silent introduction of new unmanaged sibling repositories into the release build graph
  - added `scripts/release/compute-release-window-snapshot.mjs` plus test coverage so the release ledger can recompute its latest tag baseline, commit delta, and workspace-size snapshot instead of relying on permanently hand-maintained counts
  - inserted `Run release governance gate` into both release jobs so CI now executes governance checks after GitHub materialization and before native/web dependency installation
  - resolved release refs per repository through `SDKWORK_API_ROUTER_GIT_REF`, `SDKWORK_CORE_GIT_REF`, `SDKWORK_UI_GIT_REF`, `SDKWORK_APPBASE_GIT_REF`, and `SDKWORK_IM_SDK_GIT_REF`
  - taught `verify-release-sync.mjs` to validate detached tag-like main-repository refs by resolving `ls-remote origin <expectedRef>` and accepting peeled tag output when local `HEAD` matches the remote release tag object
  - hardened `assertReleaseWorkflowContracts()` so CI contract checks now fail if materialization or governance steps stop wiring the governed sibling refs, or if the governance step stops receiving `SDKWORK_API_ROUTER_GIT_REF`
  - expanded `run-release-governance-checks.mjs` so the fixed governance sequence now also executes `release-window-snapshot.test.mjs`, not only sync-audit and workflow contracts
  - added `release-window-snapshot-contracts.mjs` so `EPERM` fallback mode can still prove release-window snapshot behavior in-process when Node child execution is blocked
  - promoted `compute-release-window-snapshot.mjs --format json` into the actual governance runner sequence, so release-window facts are now part of the live release-truth gate instead of being only test-backed
  - changed `compute-release-window-snapshot.mjs` to return structured `command-exec-blocked` results instead of crashing with a raw stack trace when Git child execution is denied
  - added top-level `blocked`, `passingIds`, `blockedIds`, and `failingIds` summary fields to `scripts/release/run-release-governance-checks.mjs`, so blocked release-truth lanes are now distinguishable from real failing lanes without parsing nested payloads manually
  - refined the governance text report to print `PASS` / `BLOCK` / `FAIL`, instead of collapsing every non-passing lane into the same state
  - marked the current `v0.1.1` to `v0.1.6` note set as a provisional release window that must be merged into the next successful real GitHub release if published release history cannot be proven
  - captured the current pending release scope on 2026-04-07 as `16` commits since local tag `release-2026-03-28-8`, `16` release-governance tests passing, and `631` working-tree entries in the current workspace snapshot

This section is a release-hold ledger, not a published version. It must be folded into the next verified successful GitHub release window.

## v0.1.8 - Concise Step Loop Prompt

- Date: 2026-04-07
- Type: patch
- Highlights:
  - compressed `docs/prompts/反复执行Step指令.md` into a shorter, clearer, repeatable execution prompt while preserving the project-specific closure rules
  - kept the same hard logic for serial backbone control, ready-set batch implementation, per-step deep verification, wave acceptance, release-truth gating, and iterative changelog updates
  - reduced redundant phrasing so the prompt is easier to reuse repeatedly without losing self-reflection, blocker-clearing, and commercialization-oriented convergence behavior

## Unreleased - Windows Rust Workspace Verification Stabilization

- Date: 2026-04-08
- Type: patch
- Highlights:
  - patched workspace `zip 3.0.0` through `vendor/zip-3.0.0` so the vendored Swagger UI build path no longer routes `deflate` through `zlib-rs` on Windows
  - added regression coverage that guards both the local `zip` patch and the Rust verification matrix contract for Windows target-dir handling
  - extended `scripts/check-rust-verification-matrix.mjs` with a `workspace` group and switched the script to the repository-managed short `CARGO_TARGET_DIR` flow instead of forcing `RUSTFLAGS='-C debuginfo=0'`
  - verified `cargo check --workspace -j 1` and `node scripts/check-rust-verification-matrix.mjs --group workspace` both pass on Windows when routed through the short-target strategy
  - exposed a manual `windows-latest` `workspace` lane in `.github/workflows/rust-verification.yml`, so hosted CI can now collect the same full-workspace evidence without expanding the default PR matrix

## v0.1.7 - Step Batch Execution Prompt Hardening

- Date: 2026-04-07
- Type: patch
- Highlights:
  - rewrote `docs/prompts/反复执行Step指令.md` into a project-specific repeatable execution prompt aligned with `docs/step/00-13`, `docs/step/90-98`, `docs/架构/130-142`, and `docs/release/*`
  - formalized a two-phase execution loop for each ready set: batch code implementation first, then step-by-step testing, validation, architecture writeback, and wave acceptance
  - made serial boundaries explicit for `00-04` and `13`, while clarifying that parallelism is limited to unlocked in-step lanes and a dedicated `Release-Truth Lane`
  - required every meaningful iteration to update `/docs/release` with dated, versioned, professional changelog records rather than deferring release notes to the end

## v0.1.6 - Admin Extension Runtime Recovery and Repository Hygiene

- Date: 2026-04-07
- Type: patch
- Highlights:
  - repaired the remaining `sdkwork-api-interface-admin` Step 06 extension-runtime verification blockers by restoring native mock fixture linkage and serializing environment-sensitive discovery tests
  - added repository ignore coverage for local `target-*`, `tmp`, and `bin/.sdkwork-target` build outputs so pre-commit workspace hygiene no longer depends on manual cleanup
  - confirmed green verification for the full admin SQLite control-plane suite and the admin frontend commercial/i18n source checks under a sandbox-safe Node test mode

## v0.1.5 - Step 06 Rust Verification Recovery

- Date: 2026-04-07
- Type: patch
- Highlights:
  - repaired the remaining Step 06 Rust verification blockers across the portal interface, admin interface, shared runtime rollout module, and portal service manifest wiring
  - restored green split-package verification for both `admin-service` and `portal-service` through the repository-owned matrix runner
  - converted the current blocker profile from concrete compile failures to non-blocking warning debt, allowing Step 06 to continue from verification recovery into control-plane capability closure

## v0.1.4 - Admin I18n Recovery and Step 06 Verification

- Date: 2026-04-07
- Type: patch
- Highlights:
  - repaired the admin translation source chain so the `zh-CN` catalog loads again under the real module graph
  - added `i18nTranslationsRecovery.ts` to backfill 158 missing payment/control-plane translation keys used by the current admin source tree
  - restored the green admin verification set for commercial API surface, commercial workbench, and admin i18n coverage
  - confirmed the remaining Step 06 service-level blocker is the pre-existing `sdkwork-api-domain-billing` compile break, not the admin i18n recovery slice

本文件记录 `sdkwork-api-router` 的累计版本变更，按版本倒序维护。

## v0.1.3 - Prompt 目标函数与收敛边界增强

- 日期：2026-04-07
- 状态：已完成
- 类型：Patch
- 说明：
  - 继续增强 `docs/prompts/反复执行Step指令.md`，加入统一目标函数、候选动作评分、决策账本、反摆动和策略冻结规则。
  - 为母提示词补充“当前阶段完美收敛 / 持续优化”的边界判定，避免无休止结构性翻修。
  - 增加商业化放行证据包约束，要求协议兼容、双运行态、性能、安全、交付运维与商业化运营证据同步完备。
  - 同步增强 `docs/release/README.md`，让 changelog 记录覆盖优先级选择依据、模式切换触发器与决策记忆。

## v0.1.2 - Prompt 收敛控制增强

- 日期：2026-04-07
- 状态：已完成
- 类型：Patch
- 说明：
  - 继续增强 `docs/prompts/反复执行Step指令.md`，加入执行模式切换、反回归保真、自我挑战式否证、完美目标最终判定等控制逻辑。
  - 强化每轮输出对执行模式、回归修复、商业化完备度提升的说明要求。
  - 增强 `docs/release/README.md`，使 release 记录同时覆盖执行模式与反回归信息。
  - 使母提示词更接近“长期运行的执行操作系统”，而不是单纯的连续任务提示词。

## v0.1.1 - 持续执行 Prompt 强化

- 日期：2026-04-07
- 状态：已完成
- 类型：Patch
- 说明：
  - 强化了 `docs/prompts/反复执行Step指令.md` 的自我思考、自我纠偏、自我降级、自我收敛逻辑。
  - 增加了连续无实质进展时的阻塞清除与收敛修正机制。
  - 增加了商业化完备度判定、量化收敛度输出要求、最小交付单位、持续优化阶段输出要求。
  - 补强了 `docs/release/README.md` 中与持续执行 Prompt 相关的 release / version / changelog 规则。
  - 使母提示词更适合被重复输入并持续推进到商业化交付目标。

## v0.1.0 - 初始化基线

- 日期：待后续迭代更新
- 状态：初始化占位版本
- 说明：用于建立 `/docs/release` 目录与 changelog 管理基线，后续每轮迭代必须按版本规则更新本文件与独立迭代变更日志。

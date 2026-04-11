# Coupon And Growth Marketing System Implementation Plan

> Status: historical implementation program. It was superseded by the coupon-first marketing architecture and the 2026-04-10 full legacy coupon exit.

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** upgrade `sdkwork-api-router` from a single-code coupon campaign model to a production-grade coupon and growth marketing system with template, batch, code pool, claim, redemption, referral, attribution, and admin or portal operations support.

**Architecture:** keep the current commercial and plugin-first architecture, but move coupon behavior out of the current lightweight campaign table into a canonical marketing kernel. Separate coupon definition, code issuance, redemption evidence, and financial fulfillment while preserving compatibility reads for current admin and portal surfaces during migration.

**Tech Stack:** Rust, Axum, sqlx, SQLite, PostgreSQL, React, TypeScript, pnpm, cargo test

---

### Task 1: Freeze the current coupon model as a compatibility layer

**Files:**
- Modify: historical legacy coupon domain compatibility layer
- Modify: historical legacy coupon app compatibility layer
- Modify: `docs/superpowers/specs/2026-04-04-coupon-and-growth-marketing-system-design.md`
- Modify: `docs/superpowers/specs/2026-04-04-global-commercial-system-design.md`

- [ ] Mark the current `CouponCampaign` shape as compatibility-era and stop treating it as the long-term source of truth.
- [ ] Preserve existing list and basic CRUD behavior only as a migration shim.
- [ ] Add tests that lock down compatibility behavior while new marketing kernels are introduced.
- [ ] Run: historical legacy coupon compatibility package tests
- [ ] Checkpoint commit: `git commit -m "docs: freeze coupon compatibility layer"`

### Task 2: Introduce canonical marketing domain records

**Files:**
- Create: `crates/sdkwork-api-domain-marketing/Cargo.toml`
- Create: `crates/sdkwork-api-domain-marketing/src/lib.rs`
- Create: `crates/sdkwork-api-domain-marketing/tests/coupon_template_records.rs`
- Modify: `Cargo.toml`

- [ ] Define records for coupon template, benefit rule, campaign, code batch, coupon code, claim, redemption, referral program, referral invite, and attribution touch.
- [ ] Define lifecycle enums for template status, batch status, code status, claim status, redemption status, and referral status.
- [ ] Encode benefit kinds, distribution kinds, stacking policies, and exclusivity groups as explicit types rather than freeform strings.
- [ ] Run: `cargo test -p sdkwork-api-domain-marketing -- --nocapture`
- [ ] Checkpoint commit: `git commit -m "feat: add canonical marketing domain model"`

### Task 3: Add marketing storage facets and schema

**Files:**
- Modify: `crates/sdkwork-api-storage-core/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-postgres/src/lib.rs`
- Create: `crates/sdkwork-api-storage-sqlite/tests/marketing_roundtrip.rs`
- Create: `crates/sdkwork-api-storage-postgres/tests/marketing_roundtrip.rs`

- [ ] Add a dedicated `MarketingStore` facet for templates, campaigns, batches, codes, claims, redemptions, referrals, and attribution touches.
- [ ] Add hashed code lookup plus masked code display fields to the schema so raw-code handling stays controlled.
- [ ] Add indexes for template, campaign, batch, code status, code lookup hash, subject ownership, and redemption lineage.
- [ ] Run: `cargo test -p sdkwork-api-storage-sqlite -p sdkwork-api-storage-postgres marketing_roundtrip -- --nocapture`
- [ ] Checkpoint commit: `git commit -m "feat: add marketing storage schema"`

### Task 4: Implement the marketing application kernel

**Files:**
- Create: `crates/sdkwork-api-app-marketing/Cargo.toml`
- Create: `crates/sdkwork-api-app-marketing/src/lib.rs`
- Create: `crates/sdkwork-api-app-marketing/tests/code_issue_and_redeem.rs`
- Modify: `crates/sdkwork-api-app-commerce/src/lib.rs`
- Modify: `crates/sdkwork-api-app-finance/src/lib.rs`

- [ ] Implement template validation, campaign window enforcement, batch issuance, secure code lookup, claim, redeem, void, and expiry processing.
- [ ] Implement quote-time validation for discount coupons and fulfillment-time benefit issuance for grant coupons.
- [ ] Make redemption idempotent and finance-safe so duplicate submit or callback replay cannot double-apply subsidy or grants.
- [ ] Run: `cargo test -p sdkwork-api-app-marketing code_issue_and_redeem -- --nocapture`
- [ ] Checkpoint commit: `git commit -m "feat: add marketing application kernel"`

### Task 5: Add admin marketing APIs

**Files:**
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Create: `crates/sdkwork-api-interface-admin/tests/marketing_routes.rs`
- Modify: `docs/api-reference/admin-api.md`

- [ ] Add admin APIs for coupon templates, campaigns, code batches, code inventory, redemptions, referral programs, and budget views.
- [ ] Add batch export or masked-preview flows that do not leak raw codes casually in the control plane.
- [ ] Add operator actions for void, block, archive, replay fulfillment, and inspect redemption lineage.
- [ ] Run: `cargo test -p sdkwork-api-interface-admin marketing_routes -- --nocapture`
- [ ] Checkpoint commit: `git commit -m "feat: add admin marketing APIs"`

### Task 6: Add portal marketing APIs

**Files:**
- Modify: `crates/sdkwork-api-interface-portal/src/lib.rs`
- Create: `crates/sdkwork-api-interface-portal/tests/marketing_routes.rs`
- Modify: `docs/api-reference/portal-api.md`

- [ ] Add portal APIs for claim, redeem, list my coupons, referral program status, and reward history.
- [ ] Make quote and checkout APIs return explicit coupon validation, subsidy, and eligibility diagnostics.
- [ ] Keep user-scoped and workspace-scoped marketing data separated so invite, reward, and benefit history remain attributable.
- [ ] Run: `cargo test -p sdkwork-api-interface-portal marketing_routes -- --nocapture`
- [ ] Checkpoint commit: `git commit -m "feat: add portal marketing APIs"`

### Task 7: Upgrade the admin React marketing control plane

**Files:**
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-coupons/src/index.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-admin-api/src/index.ts`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-types/src/index.ts`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/routeManifest.ts`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-marketing-campaigns/src/index.tsx`
- Create: `apps/sdkwork-router-admin/tests/admin-marketing-control-plane.test.mjs`

- [ ] Replace the current single-table coupon page with template, batch, code, and redemption-oriented views.
- [ ] Add admin types for template, batch, code, claim, redemption, referral, and budget records.
- [ ] Surface masked code search, at-risk campaign budgets, redemption backlog, and fraud flags in the control plane.
- [ ] Run: `pnpm --dir apps/sdkwork-router-admin test`
- [ ] Checkpoint commit: `git commit -m "feat: upgrade admin marketing control plane"`

### Task 8: Upgrade the portal redeem and growth experience

**Files:**
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-credits/src/pages/index.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/pages/index.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-portal-api/src/index.ts`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-types/src/index.ts`
- Create: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-marketing/src/index.tsx`
- Create: `apps/sdkwork-router-portal/tests/portal-marketing-growth.test.mjs`

- [ ] Replace the current simple redeem experience with claim, redeem, invite, and reward-history flows backed by canonical marketing APIs.
- [ ] Show eligible promotions and applied subsidy evidence in checkout and order history.
- [ ] Keep invite growth visible but subordinate to finance-safe reward activation.
- [ ] Run: `pnpm --dir apps/sdkwork-router-portal test`
- [ ] Checkpoint commit: `git commit -m "feat: upgrade portal marketing experience"`

### Task 9: Connect marketing to finance and benefit issuance

**Files:**
- Modify: `crates/sdkwork-api-app-billing/src/lib.rs`
- Modify: `crates/sdkwork-api-app-finance/src/lib.rs`
- Modify: `crates/sdkwork-api-app-commerce/src/lib.rs`
- Create: `crates/sdkwork-api-app-marketing/tests/marketing_finance_closure.rs`

- [ ] Route discount coupons into order pricing adjustments before payment finalization.
- [ ] Route grant coupons into benefit-lot issuance with immutable redemption and journal evidence.
- [ ] Ensure referral rewards and invite rewards obey the same finance-safe fulfillment model.
- [ ] Run: `cargo test -p sdkwork-api-app-marketing marketing_finance_closure -- --nocapture`
- [ ] Checkpoint commit: `git commit -m "feat: close marketing and finance loop"`

### Task 10: Add attribution, analytics, and anti-abuse posture

**Files:**
- Modify: `crates/sdkwork-api-app-marketing/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-overview/src/view-model.ts`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-operations/src/index.tsx`
- Create: `docs/superpowers/specs/2026-04-04-marketing-attribution-and-budget-design.md`

- [ ] Capture source channel, referral source, partner source, and campaign touch data alongside claim and redemption records.
- [ ] Surface conversion, subsidy burn, suspicious redemption velocity, and invite abuse signals in admin.
- [ ] Keep advanced experimentation and warehouse-grade analytics out of the first slice unless required by active launch scope.
- [ ] Run: `cargo test -p sdkwork-api-app-marketing -- --nocapture`
- [ ] Checkpoint commit: `git commit -m "feat: add marketing attribution and abuse signals"`

## Execution Order

The best delivery order is:

1. freeze the old coupon model as compatibility
2. add canonical marketing domains and storage
3. land the marketing application kernel
4. expose admin and portal APIs
5. upgrade admin and portal packages
6. close finance and benefit issuance
7. add attribution, analytics, and anti-abuse views

## Exit Criteria

This plan is complete only when all of the following are true:

- one coupon template can issue many codes
- unique codes, vanity codes, and invite codes all have canonical lifecycle records
- redemptions are immutable and idempotent
- grant and discount coupons both integrate safely with finance and billing
- admin can manage templates, batches, codes, and redemptions
- portal can claim, redeem, and inspect rewards from canonical marketing APIs

# Routing Policy Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace placeholder catalog-only routing with persisted routing policies that drive admin simulation and real gateway provider selection.

**Architecture:** Add a first-class routing policy aggregate in the domain layer, persist it through the storage abstraction, expose create/list APIs in the admin control plane, and make `simulate_route_with_store` apply deterministic policy ordering before falling back to the current built-in default route. Policies remain intentionally simple in this batch: exact capability match, wildcard-capable model pattern, numeric priority, ordered provider preference, and optional default provider. This gives the gateway a stable extensibility point without prematurely adding weighted load balancing, health scoring, or region-aware routing.

**Tech Stack:** Rust, Axum, serde, sqlx, existing domain/application/storage/interface gateway crates

---

### Task 1: Add failing tests for routing policy behavior

**Files:**
- Modify: `crates/sdkwork-api-domain-routing/tests/routing_decision_rules.rs`
- Modify: `crates/sdkwork-api-app-routing/tests/simulate_route.rs`
- Modify: `crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes.rs`
- Modify: `crates/sdkwork-api-storage-postgres/tests/integration_postgres.rs`
- Create: `crates/sdkwork-api-storage-sqlite/tests/routing_policies.rs`
- Create: `crates/sdkwork-api-app-gateway/tests/routing_policy_dispatch.rs`

**Step 1: Write the failing tests**

Add tests that prove:

- routing policies match exact and wildcard model patterns deterministically
- policy priority and provider ordering override lexicographic candidate selection
- SQLite persists routing policy metadata plus ordered providers
- PostgreSQL persists routing policy metadata plus ordered providers when `SDKWORK_TEST_POSTGRES_URL` is available
- `/admin/routing/policies` supports create and list with authenticated access
- `/admin/routing/simulations` returns the provider selected by the matching policy
- real gateway relay uses the same policy-selected provider path

**Step 2: Run focused tests to verify they fail**

Run:

- `cargo test -p sdkwork-api-domain-routing --test routing_decision_rules -q`
- `cargo test -p sdkwork-api-app-routing --test simulate_route -q`
- `cargo test -p sdkwork-api-storage-sqlite --test routing_policies -q`
- `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes -q`
- `cargo test -p sdkwork-api-app-gateway --test routing_policy_dispatch -q`

Expected: FAIL because the domain has no routing policy model, storage has no persistence API, admin policies endpoint is placeholder-only, and app routing still sorts candidates lexicographically.

### Task 2: Add routing policy domain and storage support

**Files:**
- Modify: `crates/sdkwork-api-domain-routing/src/lib.rs`
- Modify: `crates/sdkwork-api-domain-routing/Cargo.toml`
- Modify: `crates/sdkwork-api-storage-core/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-postgres/src/lib.rs`

**Step 1: Define the aggregate**

Add a `RoutingPolicy` type with:

- `policy_id`
- `capability`
- `model_pattern`
- `enabled`
- `priority`
- `ordered_provider_ids`
- `default_provider_id`

Also add deterministic helpers for:

- policy matching
- policy precedence ordering
- ranking available providers according to the policy

**Step 2: Persist the aggregate**

Extend `AdminStore` with routing policy persistence methods and add the backing tables:

- `routing_policies`
- `routing_policy_providers`

Store ordered provider preference explicitly so the runtime can replay the same order across SQLite and PostgreSQL.

**Step 3: Run focused tests**

Run:

- `cargo test -p sdkwork-api-domain-routing --test routing_decision_rules -q`
- `cargo test -p sdkwork-api-storage-sqlite --test routing_policies -q`
- `cargo test -p sdkwork-api-storage-postgres --test integration_postgres -q`

Expected: PASS, with PostgreSQL assertions executing only when `SDKWORK_TEST_POSTGRES_URL` is set.

### Task 3: Make application routing use persisted policies

**Files:**
- Modify: `crates/sdkwork-api-app-routing/src/lib.rs`
- Modify: `crates/sdkwork-api-app-routing/Cargo.toml`

**Step 1: Add application helpers**

Expose helpers to create, persist, and list routing policies through the application layer so the admin interface stays thin.

**Step 2: Upgrade route simulation**

Update `simulate_route_with_store` to:

- load catalog model candidates as before
- load persisted routing policies
- select the highest-precedence matching policy
- rank candidate providers using the policy order
- allow policy-only routing when there is no catalog model candidate but the policy declares providers
- preserve the existing built-in fallback when no usable policy or candidates exist

**Step 3: Run focused tests**

Run:

- `cargo test -p sdkwork-api-app-routing --test simulate_route -q`
- `cargo test -p sdkwork-api-app-gateway --test routing_policy_dispatch -q`

Expected: PASS

### Task 4: Expose routing policy management in the admin API

**Files:**
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/Cargo.toml`
- Modify: `crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes.rs`

**Step 1: Add authenticated endpoints**

Implement:

- `GET /admin/routing/policies`
- `POST /admin/routing/policies`

Return real JSON payloads backed by the new application-layer helpers.

**Step 2: Enrich routing simulation output**

Return the matched policy identifier in simulation responses so the control plane can explain why a provider was chosen.

**Step 3: Run focused tests**

Run:

- `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes -q`

Expected: PASS

### Task 5: Update docs and verify the whole workspace

**Files:**
- Modify: `README.md`
- Modify: `docs/architecture/runtime-modes.md`

**Step 1: Update capability docs**

Document that routing policies are now persisted and applied by both simulation and real gateway relay, and call out what still remains out of scope for this batch.

**Step 2: Run full verification**

Run:

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo test --workspace -q`

Expected: PASS

**Step 3: Commit**

```bash
git add docs/plans/2026-03-14-routing-policy-implementation.md README.md docs/architecture/runtime-modes.md crates/sdkwork-api-domain-routing crates/sdkwork-api-app-routing crates/sdkwork-api-storage-core crates/sdkwork-api-storage-sqlite crates/sdkwork-api-storage-postgres crates/sdkwork-api-interface-admin crates/sdkwork-api-app-gateway
git commit -m "feat: add persisted routing policies"
git push
```

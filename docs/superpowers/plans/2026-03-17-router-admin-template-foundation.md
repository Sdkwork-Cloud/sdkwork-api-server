# Router Admin Template Foundation Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn `apps/sdkwork-router-admin` into a reusable, domain-modeled super-admin template while preserving the current business module vocabulary and aligning the frontend with the real admin control plane.

**Architecture:** Keep the current top-level business modules readable, add missing foundation and governance packages, then split monolithic frontend shell and transport logic into module-owned boundaries. Expand product surfaces in the modules that already own the relevant backend capabilities instead of renaming the entire product model.

**Tech Stack:** Rust, Axum, React 19, TypeScript, pnpm, Vite, Tauri, Node test runner

---

## Chunk 1: Codify The Target Template Structure

### Task 1: Strengthen admin workspace architecture tests

**Files:**
- Modify: `apps/sdkwork-router-admin/tests/admin-architecture.test.mjs`
- Test: `apps/sdkwork-router-admin/tests/admin-architecture.test.mjs`

- [ ] **Step 1: Write failing architecture tests for missing foundation and governance packages**

Add assertions for:
- `sdkwork-router-admin-i18n`
- `sdkwork-router-admin-audit`
- per-package `tsconfig.json`
- per-package `vite.config.ts`
- required internal directories for business packages

- [ ] **Step 2: Run the architecture test to verify it fails**

Run:

```bash
node --test tests/admin-architecture.test.mjs
```

Expected:
- FAIL because the missing packages or missing files are not present yet

- [ ] **Step 3: Implement the minimal package scaffolding**

Create the missing packages and package-level config files. Add required source
directories for business modules while keeping the current public entry points
stable.

- [ ] **Step 4: Run the architecture test to verify it passes**

Run:

```bash
node --test tests/admin-architecture.test.mjs
```

Expected:
- PASS

## Chunk 2: Split Frontend Transport And Shell Foundations

### Task 2: Refactor the frontend transport layer into route-family modules

**Files:**
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-admin-api/src/index.ts`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-admin-api/src/client/index.ts`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-admin-api/src/auth/index.ts`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-admin-api/src/users/index.ts`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-admin-api/src/tenants/index.ts`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-admin-api/src/coupons/index.ts`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-admin-api/src/catalog/index.ts`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-admin-api/src/traffic/index.ts`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-admin-api/src/operations/index.ts`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-admin-api/src/audit/index.ts`

- [ ] **Step 1: Write a failing test for modular admin-api exports**

Extend the architecture test to assert that the route-family files exist and the
public `index.ts` re-exports from them.

- [ ] **Step 2: Run the test to verify it fails**

Run:

```bash
node --test tests/admin-architecture.test.mjs
```

Expected:
- FAIL due to missing route-family files

- [ ] **Step 3: Implement the modular transport split**

Move shared HTTP client logic into `client/`, then relocate route-specific API
functions into domain folders and keep the public API stable through
`src/index.ts`.

- [ ] **Step 4: Run the admin frontend typecheck**

Run:

```bash
pnpm typecheck
```

Workdir:

```bash
apps/sdkwork-router-admin
```

Expected:
- PASS

### Task 3: Reduce core-shell ownership to layout and orchestration

**Files:**
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/index.tsx`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/navigation/index.ts`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/session/index.ts`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/workspace/index.ts`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/layout/index.tsx`

- [ ] **Step 1: Write a failing architecture test for split core responsibilities**

Add checks asserting that navigation/session/workspace modules exist and the
root core entry point composes them instead of carrying everything inline.

- [ ] **Step 2: Run the test to verify it fails**

Run:

```bash
node --test tests/admin-architecture.test.mjs
```

Expected:
- FAIL

- [ ] **Step 3: Implement the split**

Move route normalization, session bootstrap, and workspace refresh orchestration
out of the core page shell file into focused modules while preserving behavior.

- [ ] **Step 4: Run frontend typecheck**

Run:

```bash
pnpm typecheck
```

Workdir:

```bash
apps/sdkwork-router-admin
```

Expected:
- PASS

## Chunk 3: Add Audit As A First-Class Admin Module

### Task 4: Add audit package and route-level UI scaffolding

**Files:**
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-types/src/index.ts`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/routes.ts`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/index.tsx`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-audit/package.json`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-audit/tsconfig.json`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-audit/vite.config.ts`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-audit/src/index.tsx`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-audit/src/pages/index.tsx`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-audit/src/types/index.ts`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-audit/src/components/index.tsx`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-audit/src/repository/index.ts`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-audit/src/services/index.ts`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-audit/src/domain/index.ts`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-audit/src/store/index.ts`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-audit/src/hooks/index.ts`

- [ ] **Step 1: Write a failing test for the new audit route and package**

Add assertions requiring:
- `audit` route key
- audit package existence
- audit page exports

- [ ] **Step 2: Run the test to verify it fails**

Run:

```bash
node --test tests/admin-architecture.test.mjs
```

Expected:
- FAIL

- [ ] **Step 3: Implement the audit module scaffold**

Create a first version of the audit page that explains current evidence sources
and prepares the UI contract for operator events, resource history, and runtime
change history.

- [ ] **Step 4: Run frontend typecheck**

Run:

```bash
pnpm typecheck
```

Workdir:

```bash
apps/sdkwork-router-admin
```

Expected:
- PASS

## Chunk 4: Align Frontend Operations And Traffic With Existing Backend Capabilities

### Task 5: Surface backend rollout and policy capabilities in the owning modules

**Files:**
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-types/src/index.ts`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-admin-api/src/operations/index.ts`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-admin-api/src/traffic/index.ts`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-operations/src/index.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-traffic/src/index.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/index.tsx`

- [ ] **Step 1: Write failing tests for missing rollout and policy surfaces**

Extend architecture and page-behavior tests to assert:
- operations exposes extension packages/installations/instances
- operations exposes runtime rollouts and config rollouts
- traffic exposes quota policy posture

- [ ] **Step 2: Run the tests to verify they fail**

Run:

```bash
node --test tests/admin-architecture.test.mjs
```

Expected:
- FAIL

- [ ] **Step 3: Implement minimal UI and API coverage**

Expose the already-existing backend endpoints in the owning frontend modules and
display them using current workspace styling.

- [ ] **Step 4: Run frontend typecheck and targeted backend admin tests**

Run:

```bash
pnpm typecheck
cargo test -p sdkwork-api-interface-admin -q
```

Expected:
- `pnpm typecheck`: PASS
- backend suite: same or better than baseline; existing native-dynamic fixture
  failures may remain unless intentionally addressed in this chunk

## Chunk 5: Harden Admin Bootstrap Posture

### Task 6: Remove visible default credentials from the browser app and docs

**Files:**
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-auth/src/index.tsx`
- Modify: `docs/api-reference/admin-api.md`
- Modify: `README.md`
- Modify: `crates/sdkwork-api-app-identity/src/lib.rs`
- Test: `crates/sdkwork-api-app-identity/tests/admin_identity.rs`

- [ ] **Step 1: Write failing tests for safer bootstrap behavior**

Add or extend tests so that bootstrap mode becomes explicit and UI no longer
renders fixed credentials by default.

- [ ] **Step 2: Run the relevant tests to verify they fail**

Run:

```bash
cargo test -p sdkwork-api-app-identity -q
pnpm typecheck
```

Expected:
- identity tests fail until bootstrap behavior is updated

- [ ] **Step 3: Implement safer bootstrap handling**

Keep local-first onboarding support, but require explicit development posture
for generated bootstrap credentials and remove hard-coded credential display from
steady-state UI.

- [ ] **Step 4: Re-run verification**

Run:

```bash
cargo test -p sdkwork-api-app-identity -q
cargo test -p sdkwork-api-interface-admin -q
pnpm typecheck
```

Expected:
- passing identity tests
- admin interface suite no worse than baseline
- frontend typecheck PASS

## Chunk 6: Completion Verification

### Task 7: Verify the foundation migration end to end

**Files:**
- Review-only task

- [ ] **Step 1: Run admin frontend architecture test**

Run:

```bash
node --test tests/admin-architecture.test.mjs
```

- [ ] **Step 2: Run admin frontend typecheck**

Run:

```bash
pnpm typecheck
```

Workdir:

```bash
apps/sdkwork-router-admin
```

- [ ] **Step 3: Run backend admin interface tests**

Run:

```bash
cargo test -p sdkwork-api-interface-admin -q
```

- [ ] **Step 4: Capture known baseline exceptions**

Document any remaining pre-existing native-dynamic fixture failures with exact
test names and confirm no new regressions were introduced.

- [ ] **Step 5: Commit**

```bash
git add docs/superpowers/specs docs/superpowers/plans apps/sdkwork-router-admin crates/sdkwork-api-app-identity docs/api-reference/admin-api.md README.md
git commit -m "feat: establish router admin template foundation"
```

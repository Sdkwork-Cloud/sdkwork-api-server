# Router Admin Product Redesign Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Redesign `apps/sdkwork-router-admin` so `Overview` behaves like an operator cockpit and the business routes behave like minimal, consistent workbenches.

**Architecture:** Keep the existing shell and settings-center architecture intact, then tighten the product layer by updating shared admin primitives, reducing page clutter, converging on a single workbench pattern, and redesigning `Overview` around risk, trend, and action prioritization. Most changes should live inside the page packages plus `sdkwork-router-admin-commons` and the shell stylesheet.

**Tech Stack:** React 19, TypeScript, react-router-dom, Zustand, Lucide React, Motion, CSS

---

### Task 1: Lock The Product Redesign Contract In Tests

**Files:**
- Modify: `apps/sdkwork-router-admin/tests/admin-product-experience.test.mjs`
- Modify: `apps/sdkwork-router-admin/tests/admin-shell-parity.test.mjs`

**Step 1: Write the failing test**

Add assertions that require:

- `Overview` to expose cockpit-oriented copy like risk, trend, and action framing
- `Users`, `Tenants`, `Catalog`, `Traffic`, and `Operations` to expose a single workbench-oriented main task
- removal or reduction of standing guidance language that conflicts with the new product model

**Step 2: Run test to verify it fails**

Run: `pnpm --dir apps/sdkwork-router-admin exec node --test tests/admin-product-experience.test.mjs tests/admin-shell-parity.test.mjs`
Expected: FAIL because the current pages still expose broader multi-surface layouts and explanatory copy.

**Step 3: Do not implement yet**

Stop after the failures are confirmed.

### Task 2: Strengthen Shared Workbench Primitives

**Files:**
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-commons/src/index.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-shell/src/styles/index.css`
- Test: `apps/sdkwork-router-admin/tests/admin-product-experience.test.mjs`

**Step 1: Write the failing test**

Require shared primitives that support the new pattern:

- a tighter toolbar/control-bar composition
- cleaner surface hierarchy
- reusable compact metric and table framing

**Step 2: Run test to verify it fails**

Run: `pnpm --dir apps/sdkwork-router-admin exec node --test tests/admin-product-experience.test.mjs`
Expected: FAIL because the existing shared components are still tuned for the older stacked layout style.

**Step 3: Write minimal implementation**

Add or refine shared helpers so pages can consistently render:

```tsx
<PageToolbar title="..." detail="..." actions={...}>
  <div className="adminx-control-bar">...</div>
</PageToolbar>
```

Keep the API small. Do not add a large component framework.

**Step 4: Run test to verify it passes**

Run: `pnpm --dir apps/sdkwork-router-admin exec node --test tests/admin-product-experience.test.mjs`
Expected: PASS for the shared workbench expectations.

**Step 5: Commit**

```bash
git add apps/sdkwork-router-admin/packages/sdkwork-router-admin-commons/src/index.tsx apps/sdkwork-router-admin/packages/sdkwork-router-admin-shell/src/styles/index.css apps/sdkwork-router-admin/tests/admin-product-experience.test.mjs
git commit -m "feat: add shared workbench primitives for router admin"
```

### Task 3: Redesign Overview Into An Operator Cockpit

**Files:**
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-overview/src/index.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-shell/src/styles/index.css`
- Test: `apps/sdkwork-router-admin/tests/admin-product-experience.test.mjs`

**Step 1: Write the failing test**

Require `Overview` to expose cockpit-oriented sections such as:

- risk queue
- trend or hotspot framing
- action-oriented operator priorities

**Step 2: Run test to verify it fails**

Run: `pnpm --dir apps/sdkwork-router-admin exec node --test tests/admin-product-experience.test.mjs`
Expected: FAIL because the page still reads like a broad summary grid.

**Step 3: Write minimal implementation**

Rebuild the page around a tighter cockpit hierarchy, for example:

```tsx
<Surface title="Priority risks" detail="...">...</Surface>
<Surface title="Trend watch" detail="...">...</Surface>
<Surface title="Operator queue" detail="...">...</Surface>
```

Keep navigation hooks intact and use the live snapshot as-is.

**Step 4: Run test to verify it passes**

Run: `pnpm --dir apps/sdkwork-router-admin exec node --test tests/admin-product-experience.test.mjs`
Expected: PASS

**Step 5: Commit**

```bash
git add apps/sdkwork-router-admin/packages/sdkwork-router-admin-overview/src/index.tsx apps/sdkwork-router-admin/packages/sdkwork-router-admin-shell/src/styles/index.css apps/sdkwork-router-admin/tests/admin-product-experience.test.mjs
git commit -m "feat: redesign router admin overview as operator cockpit"
```

### Task 4: Converge Users And Tenants On The Workbench Pattern

**Files:**
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-users/src/index.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-tenants/src/index.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-shell/src/styles/index.css`
- Test: `apps/sdkwork-router-admin/tests/admin-product-experience.test.mjs`

**Step 1: Write the failing test**

Require:

- fewer standing guidance cards
- a single main table or registry emphasis
- a stable control bar above the dominant surface

**Step 2: Run test to verify it fails**

Run: `pnpm --dir apps/sdkwork-router-admin exec node --test tests/admin-product-experience.test.mjs`
Expected: FAIL against the current multi-surface versions.

**Step 3: Write minimal implementation**

For `Users`:

```tsx
<PageToolbar ...>
  <div className="adminx-control-bar">search, status, type, primary CTA</div>
</PageToolbar>
<Surface title="Identity registry">one dominant table</Surface>
```

For `Tenants`:

```tsx
<PageToolbar ...>
  <div className="adminx-control-bar">workspace filters and primary CTA</div>
</PageToolbar>
<Surface title="Workspace registry">tenant/project dominant view</Surface>
```

Preserve dialogs and destructive-action safeguards.

**Step 4: Run test to verify it passes**

Run: `pnpm --dir apps/sdkwork-router-admin exec node --test tests/admin-product-experience.test.mjs`
Expected: PASS

**Step 5: Commit**

```bash
git add apps/sdkwork-router-admin/packages/sdkwork-router-admin-users/src/index.tsx apps/sdkwork-router-admin/packages/sdkwork-router-admin-tenants/src/index.tsx apps/sdkwork-router-admin/packages/sdkwork-router-admin-shell/src/styles/index.css apps/sdkwork-router-admin/tests/admin-product-experience.test.mjs
git commit -m "feat: simplify router admin user and tenant workbenches"
```

### Task 5: Converge Catalog, Traffic, And Operations On The Workbench Pattern

**Files:**
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-catalog/src/index.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-traffic/src/index.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-operations/src/index.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-shell/src/styles/index.css`
- Test: `apps/sdkwork-router-admin/tests/admin-product-experience.test.mjs`

**Step 1: Write the failing test**

Require:

- one dominant working surface per page
- reduced standing instructional copy
- tighter action hierarchy

**Step 2: Run test to verify it fails**

Run: `pnpm --dir apps/sdkwork-router-admin exec node --test tests/admin-product-experience.test.mjs`
Expected: FAIL because these pages still keep too many equal-weight sections open.

**Step 3: Write minimal implementation**

For `Catalog`, make provider readiness the center of gravity.

For `Traffic`, keep one active result surface at a time:

```tsx
{mode === 'usage' ? <Surface title="Usage results">...</Surface> : null}
{mode === 'routing' ? <Surface title="Routing results">...</Surface> : null}
```

For `Operations`, make runtime posture primary and move latest reload summary into the header area.

**Step 4: Run test to verify it passes**

Run: `pnpm --dir apps/sdkwork-router-admin exec node --test tests/admin-product-experience.test.mjs`
Expected: PASS

**Step 5: Commit**

```bash
git add apps/sdkwork-router-admin/packages/sdkwork-router-admin-catalog/src/index.tsx apps/sdkwork-router-admin/packages/sdkwork-router-admin-traffic/src/index.tsx apps/sdkwork-router-admin/packages/sdkwork-router-admin-operations/src/index.tsx apps/sdkwork-router-admin/packages/sdkwork-router-admin-shell/src/styles/index.css apps/sdkwork-router-admin/tests/admin-product-experience.test.mjs
git commit -m "feat: simplify router admin catalog traffic and operations workbenches"
```

### Task 6: Final Visual Tightening And Verification

**Files:**
- Modify any files required by verification

**Step 1: Run all admin tests**

Run: `pnpm --dir apps/sdkwork-router-admin exec node --test tests/*.mjs`
Expected: PASS

**Step 2: Run typecheck**

Run: `pnpm --dir apps/sdkwork-router-admin typecheck`
Expected: PASS

**Step 3: Run production build**

Run: `pnpm --dir apps/sdkwork-router-admin build`
Expected: PASS

**Step 4: Re-check the redesign standard**

Confirm:

- `Overview` reads like a cockpit
- business pages each have one clear main job
- standing guidance clutter is reduced
- action hierarchy is consistent
- the dominant data surface is visually obvious on every page

**Step 5: Commit**

```bash
git add apps/sdkwork-router-admin
git commit -m "feat: redesign router admin product surfaces"
```

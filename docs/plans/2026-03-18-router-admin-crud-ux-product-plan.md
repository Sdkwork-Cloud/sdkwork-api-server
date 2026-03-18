# Router Admin CRUD UX Product Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Refactor the admin management pages so Users, Tenants, and Coupons behave like focused backend workbenches with modal-driven create and edit flows.

**Architecture:** Keep the current admin shell and route structure, but extend the shared admin UI primitives with dialog-capable components and move CRUD mutation flows out of inline page forms. Each target page keeps tables and filters as the primary surface, while create and edit happen inside focused modals with reusable form fields and footer actions.

**Tech Stack:** React 19, TypeScript, Vite, existing admin shell/theme CSS, Radix Dialog primitives

---

### Task 1: Lock the CRUD UX rules in tests

**Files:**
- Create: `apps/sdkwork-router-admin/tests/admin-crud-ux.test.mjs`
- Modify: `apps/sdkwork-router-admin/tests/admin-shell-parity.test.mjs`

**Step 1: Write the failing test**

Require:

- `Users` page uses dialog or modal flows for operator and portal creation
- `Tenants` page uses dialog or modal flows for tenant, project, and gateway key issuance
- `Coupons` page uses dialog or modal flows for coupon creation/editing
- these pages no longer use always-visible inline submit forms as the primary create pattern
- admin commons exposes dialog primitives

**Step 2: Run the tests to verify they fail**

Run:

```powershell
node --test tests/admin-shell-parity.test.mjs tests/admin-crud-ux.test.mjs
```

Expected:

- FAIL because the current pages still embed create/edit forms inline

### Task 2: Add shared modal primitives to admin commons

**Files:**
- Modify: `apps/sdkwork-router-admin/package.json`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-commons/src/index.tsx`
- Modify: `apps/sdkwork-router-admin/src/theme.css`

**Step 1: Write the smallest implementation needed**

Add:

- dialog primitives
- form field helpers
- compact toolbar wrapper if needed
- modal styling aligned to the admin shell

**Step 2: Run the targeted tests**

Run:

```powershell
node --test tests/admin-shell-parity.test.mjs tests/admin-crud-ux.test.mjs
```

Expected:

- the shared dialog primitive checks pass
- page-level CRUD checks still fail until page refactors land

### Task 3: Refactor Users into a registry-first workbench

**Files:**
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-users/src/index.tsx`

**Step 1: Write the failing page assertion if needed**

Strengthen the users-page test so it expects:

- `Create operator` and `Create portal user` actions
- modal content for operator and portal user editing
- filters remain visible outside mutation flows

**Step 2: Implement**

Move create/edit into dialogs, keep:

- hero
- KPI strip
- filter surface
- operator roster
- portal roster

**Step 3: Re-run tests**

Run:

```powershell
node --test tests/admin-crud-ux.test.mjs
```

Expected:

- users CRUD UX assertions pass

### Task 4: Refactor Tenants into a workspace registry workbench

**Files:**
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-tenants/src/index.tsx`

**Step 1: Write or strengthen the failing test**

Require:

- modal-driven `New tenant`, `New project`, `Issue gateway key`
- no large inline create forms living in the page body

**Step 2: Implement**

Keep registry tables primary and use dialogs for:

- tenant create/edit
- project create/edit
- key issuance

**Step 3: Re-run tests**

Run:

```powershell
node --test tests/admin-crud-ux.test.mjs
```

Expected:

- tenant CRUD UX assertions pass

### Task 5: Refactor Coupons into a campaign roster workbench

**Files:**
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-coupons/src/index.tsx`

**Step 1: Write or strengthen the failing test**

Require:

- `Create coupon` CTA
- coupon create/edit modal
- filters and roster remain primary

**Step 2: Implement**

Move coupon editing into a single modal and keep:

- KPI strip
- filter toolbar
- roster table
- compact campaign guidance card

**Step 3: Re-run tests**

Run:

```powershell
node --test tests/admin-crud-ux.test.mjs
```

Expected:

- coupon CRUD UX assertions pass

### Task 6: Full admin verification

**Files:**
- Modify: any touched admin files required to clear verification failures

**Step 1: Run all admin tests**

Run:

```powershell
node --test tests/*.mjs
```

Expected:

- PASS

**Step 2: Run typecheck and build**

Run:

```powershell
pnpm --dir apps/sdkwork-router-admin typecheck
pnpm --dir apps/sdkwork-router-admin build
```

Expected:

- PASS

**Step 3: Fix the smallest remaining issues**

Only adjust the code needed to restore:

- CRUD interaction clarity
- backend visual hierarchy
- modal quality
- regression safety

**Step 4: Re-run all verification**

Run:

```powershell
node --test tests/*.mjs
pnpm --dir apps/sdkwork-router-admin typecheck
pnpm --dir apps/sdkwork-router-admin build
```

Expected:

- PASS with clean exit codes

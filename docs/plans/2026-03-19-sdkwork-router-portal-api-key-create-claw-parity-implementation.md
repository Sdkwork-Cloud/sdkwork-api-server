# SDKWork Router Portal API Key Create Flow Claw Parity Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Rebuild the Portal API key creation flow so it matches Claw Studio's Api Router manager structure while supporting Portal-specific environment keys.

**Architecture:** Mirror Claw's create-form hierarchy in the Portal UI, then extend the Portal API create contract so the front end can support both generated and custom plaintext keys. Keep the persisted key model write-only and continue returning plaintext only from the create response.

**Tech Stack:** React 19, Portal commons components, Tailwind 4 utilities, Axum portal interface, Rust identity/application crates, SQLite/Postgres admin-store backends.

---

### Task 1: Lock create-flow parity with failing tests

**Files:**
- Modify: `apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- Modify: `crates/sdkwork-api-interface-portal/tests/portal_api_keys.rs`

**Step 1: Write the failing test**

- Assert the Portal create dialog now includes:
  - a dedicated create-form component
  - `Gateway key mode`
  - `System generated`
  - `Custom key`
  - `Portal managed`
- Add a Portal interface test that posts a custom key payload and verifies:
  - create succeeds
  - list results stay write-only
  - persisted metadata remains correct

**Step 2: Run test to verify it fails**

Run: `node --test tests/portal-product-polish.test.mjs`

Expected: FAIL

Run: `cargo test -p sdkwork-api-interface-portal portal_api_keys`

Expected: FAIL

**Step 3: Write minimal implementation**

- Only implement enough contract and UI changes to satisfy the new create-flow assertions.

**Step 4: Run test to verify it passes**

Run: `node --test tests/portal-product-polish.test.mjs`

Expected: PASS

Run: `cargo test -p sdkwork-api-interface-portal portal_api_keys`

Expected: PASS

### Task 2: Extend the Portal API create contract for custom keys

**Files:**
- Modify: `crates/sdkwork-api-interface-portal/src/lib.rs`
- Modify: `crates/sdkwork-api-app-identity/src/lib.rs`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-portal-api/src/index.ts`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-api-keys/src/repository/index.ts`

**Step 1: Write the failing test**

- Use the interface test from Task 1 as the red phase.

**Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-interface-portal portal_api_keys`

Expected: FAIL

**Step 3: Write minimal implementation**

- Add optional plaintext key input to the Portal create request.
- Support generated and custom key creation in the identity layer.
- Preserve existing `label` and `expires_at_ms` behavior.

**Step 4: Run test to verify it passes**

Run: `cargo test -p sdkwork-api-interface-portal portal_api_keys`

Expected: PASS

### Task 3: Rebuild the Portal create dialog to mirror Claw form structure

**Files:**
- Create: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-api-keys/src/components/PortalApiKeyCreateForm.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-api-keys/src/components/PortalApiKeyDialogs.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-api-keys/src/components/PortalApiKeyTable.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-api-keys/src/components/index.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-api-keys/src/types/index.ts`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-api-keys/src/services/index.ts`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-api-keys/src/pages/index.tsx`

**Step 1: Write the failing test**

- Use the Portal product polish test from Task 1 as the red phase.

**Step 2: Run test to verify it fails**

Run: `node --test tests/portal-product-polish.test.mjs`

Expected: FAIL

**Step 3: Write minimal implementation**

- Create a Claw-style create-form component.
- Add key mode state and custom key input handling.
- Align create modal layout, helper text, and source copy with Claw patterns.
- Update table/source wording to stay truthful for both generated and custom keys.

**Step 4: Run test to verify it passes**

Run: `node --test tests/portal-product-polish.test.mjs`

Expected: PASS

### Task 4: Verify end-to-end

**Files:**
- No direct file changes required

**Step 1: Run focused Portal UI tests**

Run: `node --test tests/portal-theme-config.test.mjs tests/portal-product-polish.test.mjs`

Expected: PASS

**Step 2: Run backend tests**

Run: `cargo test -p sdkwork-api-interface-portal portal_api_keys`

Expected: PASS

**Step 3: Run typecheck and build**

Run: `pnpm typecheck`

Expected: PASS

Run: `pnpm build`

Expected: PASS

**Step 4: Run a visual smoke check**

- Launch Portal locally.
- Open the API Keys page.
- Open the create dialog.
- Verify the form hierarchy and styling now read like a Claw Studio sibling instead of a Portal-specific variant.

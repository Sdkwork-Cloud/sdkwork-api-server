# Public Portal and Cross-Platform Console Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a public self-service portal with registration, login, workspace summary, and API key issuance, plus package-bounded portal frontend modules and cross-platform startup documentation.

**Architecture:** Keep admin, gateway, and portal as separate interface boundaries. Extend the current identity and storage layers additively for portal accounts, expose a dedicated `/portal/*` router and standalone service, then compose new portal packages into the existing React shell so the same UI works in browser and Tauri.

**Tech Stack:** Rust, Axum, SQLx, serde, jsonwebtoken, React, TypeScript, Vite, pnpm, Tauri

---

### Task 1: Add failing backend tests for the public portal surface

**Files:**
- Create: `crates/sdkwork-api-interface-portal/tests/portal_auth.rs`
- Create: `crates/sdkwork-api-interface-portal/tests/portal_api_keys.rs`
- Create: `crates/sdkwork-api-interface-portal/src/lib.rs`
- Modify: `Cargo.toml`

**Step 1: Write the failing tests**

Cover:

- `POST /portal/auth/register`
- `POST /portal/auth/login`
- `GET /portal/auth/me`
- `GET /portal/workspace`
- `GET /portal/api-keys`
- `POST /portal/api-keys`

Include:

- duplicate email rejection
- invalid password rejection
- authenticated workspace scoping
- plaintext API key returned only on create

**Step 2: Run tests to verify they fail**

Run:

- `cargo test -p sdkwork-api-interface-portal --test portal_auth -q`
- `cargo test -p sdkwork-api-interface-portal --test portal_api_keys -q`

Expected: FAIL because the portal crate and routes do not exist yet.

### Task 2: Extend identity domain, storage contract, and persistence

**Files:**
- Modify: `crates/sdkwork-api-domain-identity/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-core/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-postgres/src/lib.rs`
- Create: `crates/sdkwork-api-app-identity/tests/portal_identity.rs`
- Modify: `crates/sdkwork-api-app-identity/src/lib.rs`

**Step 1: Add failing application-layer tests**

Test:

- portal user registration
- portal login verification
- portal user lookup
- portal-scoped API key listing

**Step 2: Run tests to verify they fail**

Run:

- `cargo test -p sdkwork-api-app-identity --test portal_identity -q`

Expected: FAIL because portal account types and store methods do not exist yet.

**Step 3: Add additive identity model and store methods**

Add:

- `PortalUserRecord`
- portal JWT claims
- password hash and salt helpers
- storage contract methods for create or find portal users

Extend `identity_users` into a real persisted portal account table with:

- email uniqueness
- password hash and salt
- display name
- workspace tenant or project ownership
- active flag
- created timestamp

**Step 4: Implement app-layer portal identity flows**

Add:

- register portal user and default workspace
- login portal user
- verify portal JWT
- list portal user API keys
- create portal user API key

**Step 5: Run tests to verify they pass**

Run:

- `cargo test -p sdkwork-api-app-identity --test portal_identity -q`

Expected: PASS

### Task 3: Implement the dedicated portal interface and standalone service

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/sdkwork-api-interface-portal/Cargo.toml`
- Create: `crates/sdkwork-api-interface-portal/src/lib.rs`
- Create: `services/portal-api-service/Cargo.toml`
- Create: `services/portal-api-service/src/main.rs`
- Modify: `crates/sdkwork-api-config/src/lib.rs`

**Step 1: Wire the portal router**

Implement:

- portal auth extractor
- register and login handlers
- authenticated `me`, workspace, and API key handlers
- HTTP metrics and tracing middleware alignment with existing services

**Step 2: Add standalone service startup**

Mirror the current admin and gateway service structure:

- shared config loading
- storage dialect selection
- tracing init
- bind listener
- serve portal router

**Step 3: Add portal bind configuration**

Document and implement:

- `SDKWORK_PORTAL_BIND`

**Step 4: Run targeted tests**

Run:

- `cargo test -p sdkwork-api-interface-portal --test portal_auth -q`
- `cargo test -p sdkwork-api-interface-portal --test portal_api_keys -q`

Expected: PASS

### Task 4: Build package-bounded portal frontend modules

**Files:**
- Create: `console/packages/sdkwork-api-portal-sdk/package.json`
- Create: `console/packages/sdkwork-api-portal-sdk/tsconfig.json`
- Create: `console/packages/sdkwork-api-portal-sdk/src/index.ts`
- Create: `console/packages/sdkwork-api-portal-auth/package.json`
- Create: `console/packages/sdkwork-api-portal-auth/tsconfig.json`
- Create: `console/packages/sdkwork-api-portal-auth/src/index.tsx`
- Create: `console/packages/sdkwork-api-portal-user/package.json`
- Create: `console/packages/sdkwork-api-portal-user/tsconfig.json`
- Create: `console/packages/sdkwork-api-portal-user/src/index.tsx`
- Modify: `console/packages/sdkwork-api-types/src/index.ts`
- Modify: `console/package.json`
- Modify: `console/src/App.tsx`
- Modify: `console/src/App.css`

**Step 1: Add typed portal SDK support**

Implement:

- register
- login
- get current session user
- get workspace summary
- list API keys
- create API key

**Step 2: Add portal auth and portal user packages**

Implement:

- registration page
- login page
- session persistence
- dashboard page
- API key creation UI
- workspace summary UI

**Step 3: Recompose the root app shell**

Make the shell support:

- browser access
- Tauri access
- hash-route navigation for portal and admin experiences

**Step 4: Run frontend verification**

Run:

- `pnpm --dir console -r typecheck`
- `pnpm --dir console build`

Expected: PASS

### Task 5: Improve desktop and browser workflow plus documentation

**Files:**
- Modify: `console/package.json`
- Modify: `console/vite.config.ts`
- Modify: `console/src-tauri/tauri.conf.json`
- Modify: `README.md`
- Modify: `README.zh-CN.md`
- Modify: `docs/architecture/runtime-modes.md`
- Modify: `docs/api/compatibility-matrix.md`

**Step 1: Improve shared browser and desktop startup scripts**

Add scripts and docs for:

- browser-only console startup
- desktop-only startup
- desktop plus browser shared development workflow
- preview or build workflow across Windows, Linux, and macOS

**Step 2: Update runtime documentation**

Explain:

- admin, gateway, and portal roles
- browser access during desktop development
- platform-specific shell examples
- standalone three-service startup

**Step 3: Run full verification**

Run:

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo test --workspace -q -j 1`
- `pnpm --dir console -r typecheck`
- `pnpm --dir console build`
- `$env:CARGO_BUILD_JOBS='1'; cargo clippy --workspace --all-targets -- -D warnings`

Expected: all commands exit `0`

### Task 6: Review, commit, and push

**Files:**
- Review: all touched files

**Step 1: Run a final diff review**

Check:

- portal routes are isolated from admin routes
- workspace scoping is enforced
- browser and Tauri shell composition remain intact
- docs match actual startup commands

**Step 2: Commit**

```bash
git add .
git commit -m "feat: add public portal and cross-platform console flows"
```

**Step 3: Push**

```bash
git push
```

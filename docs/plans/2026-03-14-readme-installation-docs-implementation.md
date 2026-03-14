# README And Installation Docs Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Rewrite the root README into a complete installation and startup guide, add a Chinese mirror document, and verify that all documented commands match the repository.

**Architecture:** Keep `README.md` as the default English entrypoint for first-run operators, with runnable flows for SQLite, PostgreSQL, console, and Tauri startup. Add `README.zh-CN.md` as a mirrored operations guide so the repository has a bilingual primary entrypoint without duplicating lower-level design content elsewhere.

**Tech Stack:** Markdown, Rust workspace, Axum, pnpm, Vite, Tauri

---

### Task 1: Capture the current runnable startup surface

**Files:**
- Inspect: `README.md`
- Inspect: `services/gateway-service/src/main.rs`
- Inspect: `services/admin-api-service/src/main.rs`
- Inspect: `crates/sdkwork-api-config/src/lib.rs`
- Inspect: `console/package.json`
- Inspect: `console/src-tauri/tauri.conf.json`

**Step 1: Verify service entrypoints**

Confirm:

- standalone HTTP binaries are `gateway-service` and `admin-api-service`
- default binds are `127.0.0.1:8080` and `127.0.0.1:8081`
- supported standalone storage dialects are SQLite and PostgreSQL
- console web entrypoint is `pnpm --dir console dev`
- Tauri entrypoint is `cargo tauri dev` from `console/` if the CLI is available

**Step 2: Note documentation constraints**

Document the current reality:

- root README must be English by default
- Chinese version must be added
- commands must reflect the current repo, not aspirational tooling

### Task 2: Rewrite the English README

**Files:**
- Modify: `README.md`

**Step 1: Reframe the README structure**

Add sections for:

- overview
- repository layout
- prerequisites
- quick start with SQLite
- quick start with PostgreSQL
- standalone startup
- console web usage
- Tauri usage
- environment variables
- verification
- capability snapshot
- limitations

**Step 2: Keep commands runnable**

Use concrete command blocks such as:

```bash
cargo run -p admin-api-service
cargo run -p gateway-service
pnpm --dir console install
pnpm --dir console dev
```

**Step 3: Link the Chinese mirror**

Add a short language switch near the top.

### Task 3: Add the Chinese mirror document

**Files:**
- Create: `README.zh-CN.md`

**Step 1: Mirror the English operational structure**

Translate the runnable operational content faithfully:

- prerequisites
- startup flows
- runtime configuration
- verification
- limitations

**Step 2: Link back to the English original**

Add a short language switch near the top.

### Task 4: Verify command and path accuracy

**Files:**
- Verify: `README.md`
- Verify: `README.zh-CN.md`

**Step 1: Run validation commands**

Run:

- `pnpm --dir console -r typecheck`
- `cargo fmt --all --check`

Expected:

- exit code `0`

**Step 2: Inspect git diff and status**

Run:

- `git status --short`

Expected:

- only documentation changes intended for this batch remain before commit

### Task 5: Commit and push

**Files:**
- Modify: repository worktree from previous tasks

**Step 1: Commit**

```bash
git add README.md README.zh-CN.md docs/plans/2026-03-14-readme-installation-docs-design.md docs/plans/2026-03-14-readme-installation-docs-implementation.md
git commit -m "docs: expand installation and startup guides"
```

**Step 2: Push**

```bash
git push origin feature/bootstrap-workspace-skeleton
```

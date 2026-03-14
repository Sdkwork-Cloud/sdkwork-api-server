# VitePress Operations Documentation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a VitePress and TypeScript documentation site under `docs/`, then update both README files so source startup, release builds, and cross-platform operations are easier to follow.

**Architecture:** Keep `docs/` as the single documentation root. Add a lightweight docs runtime with VitePress config, create curated bilingual operational pages, and reduce the README files to durable quick-start and navigation content that points into the docs site.

**Tech Stack:** VitePress, TypeScript, pnpm, Markdown

---

### Task 1: Add failing verification for the docs site

**Files:**
- Create: `docs/package.json`
- Create: `docs/tsconfig.json`
- Create: `docs/.vitepress/config.ts`

**Step 1: Verify docs build currently fails**

Run:

```powershell
pnpm --dir docs build
```

Expected: fail because `docs/package.json` does not exist yet

### Task 2: Create the VitePress runtime

**Files:**
- Create: `docs/package.json`
- Create: `docs/tsconfig.json`
- Create: `docs/.vitepress/config.ts`

**Step 1: Add the minimal docs runtime**

Implement:

- VitePress package metadata
- TypeScript config
- bilingual locale-aware `config.ts`
- curated nav and sidebar definitions

**Step 2: Verify docs config can be typechecked and built**

Run:

```powershell
pnpm --dir docs install
pnpm --dir docs typecheck
pnpm --dir docs build
```

Expected: all commands exit `0`

### Task 3: Write curated bilingual docs pages

**Files:**
- Create: `docs/index.md`
- Create: `docs/getting-started/installation.md`
- Create: `docs/getting-started/source-development.md`
- Create: `docs/getting-started/release-builds.md`
- Create: `docs/getting-started/runtime-modes.md`
- Create: `docs/getting-started/public-portal.md`
- Create: `docs/operations/configuration.md`
- Create: `docs/operations/health-and-metrics.md`
- Create: `docs/reference/api-compatibility.md`
- Create: `docs/reference/repository-layout.md`
- Create: `docs/zh/index.md`
- Create: `docs/zh/getting-started/installation.md`
- Create: `docs/zh/getting-started/source-development.md`
- Create: `docs/zh/getting-started/release-builds.md`
- Create: `docs/zh/getting-started/runtime-modes.md`
- Create: `docs/zh/getting-started/public-portal.md`
- Create: `docs/zh/operations/configuration.md`
- Create: `docs/zh/operations/health-and-metrics.md`
- Create: `docs/zh/reference/api-compatibility.md`
- Create: `docs/zh/reference/repository-layout.md`

**Step 1: Add the docs content**

Cover:

- Windows, Linux, macOS prerequisites
- source startup
- release builds and release binary startup
- browser and Tauri usage
- portal API key flow
- configuration and observability

**Step 2: Rebuild docs**

Run:

```powershell
pnpm --dir docs build
```

Expected: PASS

### Task 4: Tighten the README files around the docs site

**Files:**
- Modify: `README.md`
- Modify: `README.zh-CN.md`

**Step 1: Reframe README as quick-start plus docs gateway**

Add:

- docs site commands
- direct links to install, source, and release guides
- shorter but stronger startup matrices

**Step 2: Verify README references**

Run:

```powershell
rg -n "pnpm --dir docs|docs/getting-started|docs/zh/getting-started|start-workspace|release" README.md README.zh-CN.md
```

Expected: both README files reference the docs site and cross-platform source or release flows

### Task 5: Re-run verification

**Files:**
- Review: README files, docs runtime, docs pages

**Step 1: Run fresh verification**

Run:

```powershell
pnpm --dir docs typecheck
pnpm --dir docs build
cargo fmt --all --check
cargo test --workspace -q -j 1
pnpm --dir console -r typecheck
pnpm --dir console build
```

Expected: all commands exit `0`

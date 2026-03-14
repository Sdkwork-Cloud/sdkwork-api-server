# VitePress Operations Documentation Design

**Date:** 2026-03-14

**Status:** Approved by the user's standing instruction to proceed autonomously without waiting for interactive checkpoints

## Context

The repository already has:

- a detailed English `README.md`
- a detailed Chinese `README.zh-CN.md`
- architecture and API reference markdown under `docs/architecture` and `docs/api`
- many historical plan documents under `docs/plans`

What is still missing is an actual documentation product surface:

- no docs site runtime exists yet
- no structured guide for release builds versus source startup exists in `/docs/`
- no curated navigation separates "start here" content from deep architecture notes
- the README files still carry too much operational detail that belongs in a docs system

## Goal

Add a VitePress and TypeScript documentation site inside the existing `docs/` directory, then tighten both README files so they work as concise entry points into a fuller bilingual operations guide.

## Approaches

### Option A: Keep using README files only

Pros:

- no new tooling
- lowest immediate effort

Cons:

- poor discoverability for deeper topics
- hard to maintain bilingual parity at scale
- weak information architecture once release and source workflows diverge

### Option B: Create a VitePress site inside the existing `docs/` directory

Pros:

- preserves current docs content and paths
- introduces typed config and local preview with minimal tooling
- gives room for curated install, startup, release, and runtime guides
- works cleanly with bilingual docs under root plus `zh/`

Cons:

- adds one more Node-based package to the repository
- requires nav and sidebar curation so the plans archive does not dominate the UX

### Option C: Create a separate `website/` or `docs-site/` project

Pros:

- clean separation between source markdown and site runtime

Cons:

- duplicates documentation roots
- makes links between README, existing docs, and plans more awkward
- unnecessary churn for the current repository shape

## Recommendation

Choose **Option B**.

The repository already treats `docs/` as the documentation home. The right move is to turn that directory into a first-class VitePress site without relocating the accumulated architecture and plan records.

## Target Documentation Architecture

### Site Runtime

Add:

- `docs/package.json`
- `docs/tsconfig.json`
- `docs/.vitepress/config.ts`

Scripts:

- `pnpm --dir docs install`
- `pnpm --dir docs dev`
- `pnpm --dir docs build`
- `pnpm --dir docs preview`
- `pnpm --dir docs typecheck`

### Information Architecture

English will live at the root of the docs site:

- `docs/index.md`
- `docs/getting-started/installation.md`
- `docs/getting-started/source-development.md`
- `docs/getting-started/release-builds.md`
- `docs/getting-started/runtime-modes.md`
- `docs/getting-started/public-portal.md`
- `docs/operations/configuration.md`
- `docs/operations/health-and-metrics.md`
- `docs/reference/api-compatibility.md`
- `docs/reference/repository-layout.md`

Chinese will mirror the operational subset under:

- `docs/zh/index.md`
- `docs/zh/getting-started/...`
- `docs/zh/operations/...`
- `docs/zh/reference/...`

Existing deep technical markdown such as:

- `docs/api/compatibility-matrix.md`
- `docs/architecture/runtime-modes.md`
- `docs/plans/*`

will remain in place and be linked as supporting materials rather than promoted to the primary onboarding path.

### README Role

The README files should become:

- concise project overview
- platform and runtime summary
- one-command quick start
- source and release path summary
- links into the VitePress docs

They should stop trying to carry every operational detail inline.

## Scope of Documentation Content

The new docs site should explicitly cover:

- prerequisites for Windows, Linux, and macOS
- source-based startup for:
  - full stack browser mode
  - backend-only mode
  - browser console only
  - Tauri desktop mode
- release build flow for:
  - Rust HTTP services
  - browser console assets
  - optional Tauri desktop packaging
- runtime environment variables
- health and metrics endpoints
- public portal registration and API key usage
- repository and package layout

## Non-Goals

This batch should not:

- translate every historical plan document into Chinese
- replace existing architecture notes wholesale
- introduce a root monorepo package just for docs
- add a custom VitePress theme when the default theme is sufficient

## Testing Strategy

Use minimal but real verification:

1. prove the docs site is missing by running `pnpm --dir docs build` before scaffolding
2. add the VitePress runtime and docs pages
3. build the docs site successfully
4. grep README links and commands to ensure they point at real docs entry points
5. re-run the existing repository verification baseline so docs changes do not hide regressions

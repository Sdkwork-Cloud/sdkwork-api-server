# README And Installation Docs Design

**Date:** 2026-03-14

**Status:** Approved by the user's standing instruction to continue autonomously without pausing for approval checkpoints

## Goal

Turn the repository README from an architecture-heavy status report into an operator-facing runbook that explains how to install, configure, start, and verify the system in both standalone server mode and embedded desktop-oriented mode.

## Current Problem

The existing README explains the architecture and feature surface well, but it is not yet a complete entrypoint for a new operator.

The main gaps are:

- prerequisites are not clearly stated
- quick-start commands are not grouped into runnable flows
- standalone service startup is described conceptually but not as one concrete sequence
- SQLite and PostgreSQL startup paths are not separated cleanly
- console startup expectations are not explicit
- Tauri embedded mode exists in code but is under-documented
- there is no Chinese mirror document for the same operational content

## Options Considered

### Option A: Keep README short and move all operational content into `docs/`

Pros:

- keeps the root README compact
- avoids a very long landing page

Cons:

- weak first-run experience
- users must discover secondary docs before they can start the project

### Option B: Make README the full operational handbook and add a Chinese mirror

Pros:

- best onboarding experience
- one obvious entrypoint for installation and startup
- aligns with the user's request for a default English README plus Chinese version

Cons:

- longer root README
- requires careful structure so it stays scannable

### Option C: Add a minimal quick start only and leave deeper operational details unchanged

Pros:

- lowest edit cost
- small diff

Cons:

- still leaves important ambiguity around runtime modes, console usage, and configuration
- does not satisfy the request for complete documentation

## Recommendation

Use **Option B**.

The repository has reached the point where a new contributor or operator needs a trustworthy runbook more than another architecture summary. The README should become the default operational entrypoint, while the deeper design docs remain in `docs/`.

## Documentation Shape

The root README should be reorganized into these sections:

1. project overview
2. repository layout
3. prerequisites
4. quick start with SQLite
5. quick start with PostgreSQL
6. standalone service startup
7. console web startup
8. Tauri embedded startup
9. runtime configuration and environment variables
10. verification commands
11. current capability snapshot
12. known limitations
13. links to deeper docs

The root README should stay in English.

A Chinese mirror should be added as:

- `README.zh-CN.md`

The English README should link to the Chinese version near the top, and the Chinese document should link back to the English original.

## Scope

This batch will:

- rewrite `README.md` around runnable installation and startup flows
- add a Chinese mirror document
- preserve high-value architecture and capability notes, but compress them behind operational guidance
- verify that all documented commands and file references match the current codebase

This batch will not:

- redesign the service architecture
- add new runtime features outside documentation alignment
- add packaging or deployment automation beyond documenting the existing flows

## Verification Strategy

The documentation update should be validated by checking:

- referenced service and workspace paths exist
- documented commands match current binaries and package scripts
- console typecheck still passes after any related documentation-facing config changes
- git worktree is clean after commit


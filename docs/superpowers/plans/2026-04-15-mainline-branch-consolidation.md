# Mainline Branch Consolidation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** preserve the current dirty workspace, consolidate all branch-resident code into `main`, verify the integrated state, and push `main` to `origin`.

**Architecture:** this repository currently has one active local branch (`main`), one historical remote feature branch already absorbed by `main`, and a large dirty working tree carrying uncommitted integration work. The safe consolidation path is to snapshot the dirty working tree onto a dedicated branch, perform the real integration on an isolated `main` worktree, prove that the only remaining remote feature branch is already merged, then verify and push `main`.

**Tech Stack:** Git branches/worktrees, GitHub remote, Node-based governance verification, repository-native workflow and product verification commands.

---

## File Map

- Create: `docs/superpowers/plans/2026-04-15-mainline-branch-consolidation.md`
- Modify: repository files only if conflict resolution or verification remediation is required during merge execution

### Task 1: Snapshot the dirty workspace safely

**Files:**
- Modify: none before the snapshot commit

- [ ] **Step 1: Confirm the exact branch and dirty workspace state**

Run: `git branch --show-current`
Expected: `main`

Run: `git status --short`
Expected: a large dirty workspace that must not be merged in-place

- [ ] **Step 2: Move the dirty workspace off `main` without changing file content**

Run: `git switch -c snapshot/mainline-consolidation-20260415`
Expected: current files remain unchanged and `main` becomes available for the isolated worktree

- [ ] **Step 3: Snapshot every current tracked and untracked change**

Run: `git add -A`
Expected: all current workspace changes become staged on the snapshot branch

Run: `git commit -m "snapshot: preserve workspace consolidation state"`
Expected: one snapshot commit preserving the entire dirty workspace

### Task 2: Create an isolated `main` worktree

**Files:**
- Modify: none

- [ ] **Step 1: Create a global worktree path to avoid touching repo-local ignore rules**

Run: `New-Item -ItemType Directory -Force "$env:USERPROFILE\\.config\\superpowers\\worktrees\\sdkwork-api-router" | Out-Null`
Expected: global worktree base directory exists

- [ ] **Step 2: Attach a clean worktree to `main`**

Run: `git worktree add "$env:USERPROFILE\\.config\\superpowers\\worktrees\\sdkwork-api-router\\mainline-merge-20260415" main`
Expected: a clean checkout of local `main`

- [ ] **Step 3: Reconfirm remote branch coverage inside the worktree**

Run: `git branch -a --format="%(refname:short)"`
Expected: local `main`, the new snapshot branch, `origin/main`, and `origin/feature/bootstrap-workspace-skeleton`

Run: `git merge-base --is-ancestor origin/feature/bootstrap-workspace-skeleton main`
Expected: exit code `0`, proving the remote feature branch is already absorbed by `main`

### Task 3: Integrate the preserved dirty workspace into `main`

**Files:**
- Modify: whichever files are touched by the snapshot branch if merge conflicts appear

- [ ] **Step 1: Merge the snapshot branch into `main` inside the isolated worktree**

Run: `git merge --no-ff snapshot/mainline-consolidation-20260415`
Expected: a merge commit or a conflict set that must be resolved before proceeding

- [ ] **Step 2: If conflicts appear, resolve them against architecture and security requirements**

Resolve only conflicted files.
Expected: merged content keeps the intended feature behavior, preserves existing governance checks, and does not regress auth, release, or runtime safety behavior.

- [ ] **Step 3: Confirm the worktree is conflict-free**

Run: `git status --short`
Expected: no unmerged paths remain

### Task 4: Verify the integrated `main`

**Files:**
- Modify: only if verification exposes real defects

- [ ] **Step 1: Re-run the release-governance workflow contract suite**

Run: `node --test scripts/release-governance-workflow.test.mjs`
Expected: PASS

- [ ] **Step 2: Re-run the release governance runner self-test**

Run: `node --test --experimental-test-isolation=none scripts/release/tests/release-governance-runner.test.mjs`
Expected: PASS

- [ ] **Step 3: Re-run release governance preflight**

Run: `node scripts/release/run-release-governance-checks.mjs --profile preflight --format json`
Expected: `"ok": true`

- [ ] **Step 4: Run the product verification surface if available**

Run: `node --test scripts/product-verification-workflow.test.mjs`
Expected: PASS if the file exists in the integrated tree

### Task 5: Push consolidated `main`

**Files:**
- Modify: none unless verification requires follow-up fixes

- [ ] **Step 1: Inspect the final graph and working tree**

Run: `git log --oneline --decorate --graph --max-count=20`
Expected: `main` contains the snapshot merge and any required conflict-resolution commit(s)

Run: `git status --short`
Expected: clean working tree

- [ ] **Step 2: Push `main` to the remote**

Run: `git push origin main`
Expected: remote `origin/main` advances to the verified consolidated state

# Cross-Platform Startup Polish Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add cross-platform startup helper scripts and tighten the README so Windows, Linux, macOS, browser, and Tauri flows are copy-pasteable and operational.

**Architecture:** Keep runtime topology unchanged: gateway, admin, and portal stay as independent services, while the React console remains browser-first and Tauri-hostable. Add thin OS-specific startup scripts under `scripts/` and document them as the preferred entry points.

**Tech Stack:** PowerShell, POSIX shell, Rust, Cargo, pnpm, Vite, Tauri

---

### Task 1: Add failing script verification commands

**Files:**
- Create: `scripts/dev/start-servers.ps1`
- Create: `scripts/dev/start-console.ps1`
- Create: `scripts/dev/start-stack.mjs`
- Create: `scripts/dev/start-console.mjs`

**Step 1: Verify the files do not exist yet**

Run:

```powershell
Test-Path scripts/dev/start-servers.ps1
Test-Path scripts/dev/start-console.ps1
Test-Path scripts/dev/start-stack.mjs
Test-Path scripts/dev/start-console.mjs
```

Expected: all `False`

**Step 2: Verify shell syntax check fails before implementation**

Run:

```powershell
node --check scripts/dev/start-stack.mjs
```

Expected: fail because the file does not exist yet

### Task 2: Implement minimal cross-platform helper scripts

**Files:**
- Create: `scripts/dev/start-servers.ps1`
- Create: `scripts/dev/start-console.ps1`
- Create: `scripts/dev/start-stack.mjs`
- Create: `scripts/dev/start-console.mjs`

**Step 1: Add PowerShell helpers**

Implement:

- one script to launch admin, gateway, and portal services in separate PowerShell windows
- one script to launch the browser console
- configurable database URL and bind override environment variables

**Step 2: Add portable Node helpers**

Implement:

- one script to launch admin, gateway, and portal in the current terminal with child-process lifecycle handling
- one script to launch the browser console or Tauri desktop flow
- dry-run output for both scripts so commands can be verified without launching long-running processes

**Step 3: Verify the scripts**

Run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-servers.ps1 -DryRun
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-console.ps1 -DryRun
node --check scripts/dev/start-stack.mjs
node --check scripts/dev/start-console.mjs
node scripts/dev/start-stack.mjs --dry-run
node scripts/dev/start-console.mjs --dry-run --tauri
```

Expected: all commands succeed

### Task 3: Polish README and Chinese README around the new entry points

**Files:**
- Modify: `README.md`
- Modify: `README.zh-CN.md`

**Step 1: Document preferred startup paths**

Add:

- quick-start matrix by OS
- recommended helper script entry points
- fallback raw commands
- browser + Tauri parallel workflow
- portal registration and API key self-service walkthrough

**Step 2: Document supported and intentionally unsupported areas**

Add:

- what is complete today
- what still remains roadmap-only
- how to choose browser-only vs browser + Tauri workflows

**Step 3: Review the docs against the actual scripts**

Run:

```powershell
rg -n "start-servers|start-console|start-stack|tauri:dev|#/portal/register|SDKWORK_PORTAL_BIND" README.md README.zh-CN.md
```

Expected: the operational entry points are all referenced

### Task 4: Re-run verification

**Files:**
- Review: modified scripts and docs

**Step 1: Run the full checks**

Run:

```powershell
cargo fmt --all --check
cargo test --workspace -q -j 1
pnpm --dir console -r typecheck
pnpm --dir console build
```

Expected: all commands exit `0`

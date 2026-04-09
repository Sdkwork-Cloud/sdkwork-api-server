# Portal Recharge Visual Overhaul Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Transform the portal recharge page into a premium decision-oriented purchase workspace without changing backend contracts.

**Architecture:** Keep the existing recharge repository and service boundaries intact, then recompose the page into a stronger three-part decision flow. Drive the redesign with test-first updates that lock the new structure, copy, and data-slot contract before updating the React page implementation.

**Tech Stack:** React 19, TypeScript, portal commons framework components, Tailwind utility classes, Node built-in test runner

---

### Task 1: Lock the New Page Structure in Tests

**Files:**
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\.worktrees\portal-recharge-visual-refresh\apps\sdkwork-router-portal\tests\portal-recharge-center.test.mjs`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\.worktrees\portal-recharge-visual-refresh\apps\sdkwork-router-portal\tests\portal-recharge-workflow-polish.test.mjs`

- [ ] **Step 1: Write failing assertions for the redesigned structure**

Add expectations for:

- `data-slot="portal-recharge-selection-hero"`
- `data-slot="portal-recharge-posture-strip"`
- `data-slot="portal-recharge-quote-note"`
- `data-slot="portal-recharge-history-header"`

- [ ] **Step 2: Run the targeted tests to verify RED**

Run: `node --test tests/portal-recharge-center.test.mjs tests/portal-recharge-workflow-polish.test.mjs`
Expected: FAIL because the new data slots and copy do not exist yet.

- [ ] **Step 3: Keep existing behavior checks that must survive**

Preserve assertions for:

- existing page/package route wiring
- current repository and services boundaries
- `portal-recharge-options`
- `portal-recharge-custom-form`
- `portal-recharge-quote-card`
- `portal-recharge-history-table`

- [ ] **Step 4: Re-run targeted tests to confirm failures remain about missing page structure only**

Run: `node --test tests/portal-recharge-center.test.mjs tests/portal-recharge-workflow-polish.test.mjs`
Expected: FAIL with missing new slot or copy assertions, not syntax or unrelated errors.

### Task 2: Refine Recharge View Models for Premium Presentation

**Files:**
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\.worktrees\portal-recharge-visual-refresh\apps\sdkwork-router-portal\packages\sdkwork-router-portal-recharge\src\services\index.ts`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\.worktrees\portal-recharge-visual-refresh\apps\sdkwork-router-portal\packages\sdkwork-router-portal-recharge\src\types\index.ts`

- [ ] **Step 1: Add any minimal derived presentation helpers required by the redesign**

Only add helpers if the page would otherwise duplicate logic for:

- balance posture labels
- pending order counts
- selected option state framing

- [ ] **Step 2: Keep service scope narrow**

Do not reintroduce deleted finance projection or unrelated billing dependencies.

- [ ] **Step 3: Run the service-focused recharge tests**

Run: `node --test tests/portal-recharge-finance-projection.test.mjs tests/portal-recharge-workflow-polish.test.mjs`
Expected: PASS or fail only on assertions that still require page changes.

### Task 3: Implement the Decision Studio Layout

**Files:**
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\.worktrees\portal-recharge-visual-refresh\apps\sdkwork-router-portal\packages\sdkwork-router-portal-recharge\src\pages\index.tsx`

- [ ] **Step 1: Add the selection hero and posture strip**

Implement a compact premium intro inside the selection card that surfaces:

- current balance
- pending payment count
- recommended amount cue

- [ ] **Step 2: Rework preset tiles and custom tile hierarchy**

Improve:

- typography scale for amount values
- active and recommended states
- custom tile composition
- mobile stacking behavior

- [ ] **Step 3: Strengthen the quote cockpit**

Add:

- stronger hero framing
- concise trust note
- better CTA emphasis

- [ ] **Step 4: Refine the recharge history header and footer framing**

Keep the same table but improve surrounding hierarchy and operations continuity.

- [ ] **Step 5: Run the targeted recharge tests to verify GREEN**

Run: `node --test tests/portal-recharge-center.test.mjs tests/portal-recharge-workflow-polish.test.mjs tests/portal-recharge-finance-projection.test.mjs`
Expected: PASS

### Task 4: Verify Portal Build Safety

**Files:**
- Modify: none expected unless typecheck reveals a real issue

- [ ] **Step 1: Run portal typecheck**

Run: `pnpm typecheck`
Expected: PASS

- [ ] **Step 2: Build the portal app if typecheck passes in reasonable time**

Run: `pnpm build`
Expected: PASS

- [ ] **Step 3: Inspect git diff for final review**

Run: `git diff -- apps/sdkwork-router-portal`
Expected: only design doc, plan doc, recharge page, and any intentional recharge tests/helpers changed.

- [ ] **Step 4: Commit the work in focused commits**

Suggested sequence:

1. `git add apps/sdkwork-router-portal/docs/superpowers/specs/2026-04-09-portal-recharge-visual-overhaul-design.md`
2. `git commit -m "docs: define portal recharge visual overhaul"`
3. `git add apps/sdkwork-router-portal/docs/superpowers/plans/2026-04-09-portal-recharge-visual-overhaul.md apps/sdkwork-router-portal/packages/sdkwork-router-portal-recharge/src/pages/index.tsx apps/sdkwork-router-portal/packages/sdkwork-router-portal-recharge/src/services/index.ts apps/sdkwork-router-portal/packages/sdkwork-router-portal-recharge/src/types/index.ts apps/sdkwork-router-portal/tests/portal-recharge-center.test.mjs apps/sdkwork-router-portal/tests/portal-recharge-workflow-polish.test.mjs`
4. `git commit -m "feat: elevate portal recharge purchase experience"`

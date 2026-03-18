# Router Admin Shell Polish Design

## Goal

Tighten `apps/sdkwork-router-admin` so the already-aligned shell and settings center feel materially closer to `claw-studio` in visual hierarchy, sidebar behavior, account controls, and settings-stage presentation.

## Confirmed Direction

This is a polish pass, not another architecture rewrite.

The shell package structure, theme manager ownership, and settings-center split already exist. The remaining work is to close interaction and visual gaps without disturbing admin business modules.

## Gaps To Close

### Header

The current header is structurally aligned but still reads heavier and flatter than `claw-studio`.

We want:

- a lighter frosted surface
- a stronger centered workspace focus
- tighter action density
- better synchronization between route metadata, theme state, and shell status

### Sidebar

The current sidebar has the right shape, collapse behavior, and bottom identity region, but it still lacks the richer control language from `claw-studio`.

We want:

- animated active-route affordances
- a true account control anchored at the bottom
- a popover for profile actions instead of only static footer buttons
- a secondary bottom action that still exposes settings entry clearly
- collapsed and expanded states that remain visually balanced

### Settings Center

The settings center already follows the left-nav plus right-panel pattern, but it still needs more product framing and stage polish.

We want:

- a left-nav summary card that surfaces live shell posture
- denser, more tactile nav buttons
- a right-side stage wrapper for the active section
- clearer preview language tying theme, sidebar, and canvas together
- stronger continuity between left-nav summary, active panel header, and preview cards

## Recommended Approach

Keep the existing architecture and refine three shared surfaces together:

1. Shell header and main canvas atmosphere
2. Sidebar navigation and account affordances
3. Settings navigation, stage framing, and preview cards

This keeps the live admin routes stable while making the shell feel intentionally designed rather than merely theme-matched.

## Completion Standard

This polish pass is complete only when:

- sidebar account controls behave like a real claw-style control surface
- active navigation states feel animated and obvious
- settings center exposes a left-nav summary plus a distinct right detail stage
- theme, sidebar, and content-canvas previews tell one coherent story
- tests, typecheck, and build still pass after the refinement

# SDKWork Router Portal Wide Workspace Layout Design

## Goal

Remove the global page-top shell summary treatment from `sdkwork-router-portal` and reshape the authenticated workspace into a wider, more direct right-side content area that opens on actual work instead of decorative summary cards.

## User Feedback Incorporated

- The top `Credentials`/summary card treatment still feels inconsistent with the desired product.
- The global `ShellStatus` block is visually noisy and wastes first-screen space.
- Portal pages should maximize usable content width on the right side.

## Problem Statement

The current Portal shell still uses a dashboard-like content rhythm:

- shell-level status card at the top of every authenticated page
- page-level status rows and metric-card strips before primary work areas
- a constrained main container width that limits usable workspace area

This makes the product feel more like a reporting surface than a Claw-style working console.

## Design Decision

Use a wide workspace-first layout across all authenticated Portal pages.

### Global Shell

- Remove `ShellStatus` from `MainLayout`.
- Stop reserving a page-top banner zone in the authenticated shell.
- Expand the right content container to full available width.
- Keep the sidebar fixed on the left and the page content as the primary work canvas on the right.

### Page Rhythm

- Each page should begin with its first actionable or information-dense work surface.
- Remove top-of-page status rows that summarize obvious information already visible elsewhere.
- Remove top metric-card strips that delay access to the real task.
- Move any needed state or actions into local surface headers, toolbars, or inline notice rows.

### API Keys

- Remove the page-top credentials summary card.
- Start directly with the manager toolbar and table.
- Keep one-time plaintext handling, but render it as a narrow inline notice rather than a decorative hero card.

### Dashboard

- Remove the global status strip and top metric grid.
- Start directly with `Traffic overview` and supporting operational panels.
- Keep quick actions and routing posture visible, but as side/work panels instead of a preamble.

### Routing / Usage / Credits / Billing / User / Account

- Remove top status strips and top metric-card grids.
- Keep tabs only where they still organize meaningful work.
- Move actions into the relevant surfaces instead of page-wide summary headers.

## Layout Contract

- Main content area should no longer use a fixed `max-w-[1600px]` style cap.
- Use full-width inner content with responsive padding.
- Prefer:
  - `w-full`
  - `min-h-full`
  - moderate horizontal padding
  - direct grid splits only when content needs them

## Testing Strategy

- Add layout-contract tests that assert:
  - `MainLayout` no longer renders `ShellStatus`
  - the main content wrapper no longer uses the fixed max width
  - key Portal pages no longer render the top `portalx-status-row`
  - API Keys no longer render the top credentials/status card copy

## Implementation Notes

- This is a structural UX correction, not a backend change.
- Existing data-loading and repository logic should remain intact.
- Changes should focus on hierarchy, spacing, and page composition.

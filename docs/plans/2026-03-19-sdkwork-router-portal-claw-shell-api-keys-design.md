# SDKWork Router Portal Claw Shell API Keys Design

## Goal

Make `apps/sdkwork-router-portal` feel like a first-party sibling of `claw-studio` by aligning the shell, config center, theme behavior, sidebar interaction, and API Key management with the same visual grammar and interaction model.

## Scope

- Keep the existing Portal route map and data contracts intact.
- Preserve the current right-side content area layout and collapsible sidebar behavior.
- Rebuild the config center as a `claw-studio Settings` inspired shell with left navigation and right content.
- Rebuild Portal API Key management to mirror the `api-router` global API key manager interaction model.
- Support flexible API key creation through:
  - preset environments
  - custom environment entry
  - lifecycle presets
  - optional explicit expiry
  - richer creation guidance and usage guidance

## Constraints

- The current Portal backend only supports create/list/status/delete for API keys.
- No stored source/group/notes schema exists for Portal API keys today.
- Avoid storage schema expansion unless it is required to ship a clearly better result.

## Product Decisions

### Shell

- Keep the current authenticated shell route structure.
- Keep sidebar collapse/expand and drag resize behavior.
- Tighten the shell contract so provider setup, theme state, and config center behavior resemble `claw-studio`.
- Leave the right side as the main content viewport.

### Config Center

- Replace the current simple dialog sections with a settings shell:
  - left rail for search and tab navigation
  - right content pane for tab content
- Tabs:
  - `appearance`
  - `navigation`
  - `workspace`
- Preserve Portal-specific content, but style and structure it like `claw-studio Settings`.
- Add a live shell preview panel so theme color and mode changes are easier to validate.

### API Key Management

- Replace the current page-first layout with a manager shell:
  - top action bar
  - search
  - environment filter
  - manager table
  - create dialog
  - usage dialog
- Keep Portal-specific governance cards where they still add value, but subordinate them to the manager interaction.
- Use flexible creation via:
  - recommended environments (`live`, `staging`, `test`)
  - custom environment field
  - lifecycle presets (`never`, `30 days`, `90 days`, `custom`)
- No fake edit mutation will be introduced because the backend cannot persist it.
- Instead, expose `Usage method`, `Revoke/Restore`, `Delete`, and one-time plaintext handling strongly and clearly.

## Technical Design

### Portal Core

- `AppProviders` will be brought closer to the `claw-studio` provider shape while staying within Portal dependencies.
- `ConfigCenter` will become an internal settings workspace with local tab state and search.
- `usePortalShellStore` will gain any small helper actions needed for resetting or previewing shell preferences.

### Portal API Keys

- Add dedicated UI components for:
  - manager toolbar
  - key table
  - create dialog
  - usage dialog
- Keep repository functions small and stable.
- Expand page-level service logic to derive:
  - filtered rows
  - environment options
  - lifecycle summaries
  - quick usage snippets

## Error Handling

- Keep all API failure copy routed through `portalErrorMessage`.
- Show page status text for non-blocking feedback.
- Keep modal validation local and immediate.

## Testing

- Add or update string-contract tests for:
  - config center shell parity
  - API key manager parity
  - custom environment and lifecycle creation affordances
- Run Portal node tests and relevant app build/typecheck checks.


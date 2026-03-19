# SDKWork Router Portal Claw Dashboard And Theme Parity Design

**Date:** 2026-03-19

## Goal

Bring `apps/sdkwork-router-portal` to the same visual family as `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\claw-studio`, with the strongest parity on:

- dashboard information architecture
- shell chrome
- theme tokens and theme switching
- settings/config experience
- sidebar display, collapse, expand, and resize behavior

The right side must remain the Portal content canvas, but it should look and feel like a sibling product inside the same design system.

## Scope Chosen

Proceed with **dashboard + shell + config center + theme contract parity** as the primary implementation scope.

This is the highest-value slice because:

- the user explicitly called out dashboard, theme, config center, and sidebar parity
- the current Portal shell is already partially aligned, so finishing parity here gives the most visible product lift
- it avoids dragging unrelated business routes into a much riskier full-app rewrite while the workspace already contains in-flight edits

## Product Decisions

### Dashboard

The Portal dashboard should stop reading like a custom operational overview and instead adopt a claw-style analytics workbench layout:

- a top summary band using claw-style KPI cards
- chart sections with claw-style section headers and glass surfaces
- tabbed or segmented workbench surfaces for recent requests, routing evidence, module posture, and next actions
- tables and chips styled with the same density, rounding, and contrast language as claw-studio

Portal business semantics stay Portal-owned:

- routing posture
- recent requests
- provider and model demand
- quota, billing, and module posture

### Shell

The shell contract should stay visually synchronized with claw-studio:

- glass titlebar
- dark rail/sidebar with raw gradient chrome
- centered workspace context in the header
- sidebar collapse/expand button on the rail edge
- drag resize with the same width envelope

Portal-specific workspace identity stays in the centered header slot and footer dock, not in a separate decorative top rail card.

### Config Center

The config center remains a Portal-owned modal workspace, but it should feel closer to claw-studio General Settings:

- left navigation with search
- card-based content sections on the right
- theme mode and theme color controls matching claw defaults
- navigation settings with sidebar behavior and visible route controls
- workspace summary and reset affordances
- a live preview panel that uses the same shell geometry as the actual Portal shell

### Theme Contract

Theme behavior must remain equivalent to claw-studio:

- `themeMode`: `light | dark | system`
- `themeColor`: `lobster | tech-blue | green-tech | zinc | violet | rose`
- default mode: `system`
- default color: `lobster`
- root-level `data-theme` switching
- root-level `.dark` switching

Theme tokens must drive:

- app background
- sidebar chrome
- surfaces
- charts
- dialog/config surfaces
- hover and active states

## Architecture

### Files Likely To Change

- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-dashboard/src/pages/index.tsx`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-dashboard/src/components/index.tsx`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-dashboard/src/types/index.ts`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-dashboard/src/services/index.ts`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components/ConfigCenter.tsx`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components/AppHeader.tsx`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components/Sidebar.tsx`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-commons/src/index.tsx`
- `apps/sdkwork-router-portal/src/theme.css`
- `apps/sdkwork-router-portal/tests/*.test.mjs`

### What Must Stay Stable

- Portal auth flow
- Portal route map and route ownership
- persisted shell preferences
- sidebar collapse/expand/resize persistence
- right-side content ownership
- current API and repository contracts

## Testing Strategy

Use test-first contract updates before implementation.

The tests should assert:

- the dashboard now follows a claw-style analytics workbench hierarchy
- the dashboard uses claw-style summary-card and section-header structures
- the config center continues to expose claw-equivalent theme controls and navigation settings
- the sidebar still supports parity widths, click collapse/expand, and resize
- the theme contract remains synchronized across shell, dashboard, and config surfaces

## Completion Standard

This work is complete only when:

- the dashboard reads like the same product family as claw-studio
- theme switching stays consistent across shell, dashboard, and config center
- the sidebar still behaves correctly in expanded, collapsed, and resized states
- the config center remains coherent and theme-aware
- tests, typecheck, and build pass with fresh verification

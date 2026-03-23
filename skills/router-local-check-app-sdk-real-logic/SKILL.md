---
name: router-local-check-app-sdk-real-logic
description: Guides router-local-check console flows onto generator-backed app SDK boundaries while preserving native Rust services. Use when integrating or repairing apps/sdkwork-api-router-local-check console-facing business modules so they consume spring-ai-plus-app-api instead of handwritten console HTTP, or when a missing contract must be closed end to end without collapsing the native router architecture.
---

# Router Local Check App SDK Real Logic

## Overview

Drive `apps/sdkwork-api-router-local-check` to one split architecture:

`console app / shared console package -> generator-backed app SDK boundary -> spring-ai-plus-app-api`

Keep native Rust router, gateway, provider, storage, and extension-host work on their original service boundaries. Console-facing business capability should use the generated app-SDK standard. If a method is missing, close the backend/OpenAPI/generator gap first, then return and delete the workaround.

Treat every round as a recursive closure loop: self-review the touched app or client code, decide whether the next fix belongs in app or frontend code, backend or service code, or generator inputs, regenerate the SDK when contracts move, then review again until no higher-value gap remains.

## Progressive Loading

- Start with this file only.
- Load `references/architecture-map.md` only when deciding whether work belongs to console-client or native-service boundaries.
- Load `../../../SDK_INTEGRATION_STANDARD.md` only when console SDK lifecycle or token rules matter.
- Load `../../ARCHITECT.md` only when workspace ownership or crate boundaries are unclear.
- Load `references/verification.md` only before closing the round.

## Hard Rules

- Use `spring-ai-plus-app-api` as the single contract source for console-facing app business capability.
- Console-facing SDK packages such as `sdkwork-api-portal-sdk` or `sdkwork-api-admin-sdk` must stay generator-backed, not handwritten request layers.
- Keep native Rust router work on native boundaries. Do not force `crates/sdkwork-api-*`, provider integrations, routing, gateway, or storage internals through the app SDK.
- Replace console-side business HTTP with the generated SDK boundary. Do not add raw `fetch`, ad hoc client helpers, manual auth headers, mock branches, or console-local contract forks.
- Never hand-edit generated SDK output. Fix backend or generator inputs, then regenerate.
- Any table, column, index, migration, or storage schema change requires user confirmation first.

## Default Loop

1. Classify the target as console-client, native-service, or mixed.
2. Audit the touched console package or Rust crate for raw HTTP, duplicated DTOs, manual headers, stale client shortcuts, or boundary leakage.
3. Verify the real generated SDK export and the shared console boundary.
4. If the target is native Rust service logic, keep it native and do not invent an app-sdk hop.
5. If the target is console-facing and the method exists, refactor to the SDK path and delete the bypass.
6. If the console method is missing, close the gap in `spring-ai-plus-app-api` and backend modules, regenerate the SDK package, then finish integration.
7. If gap closure or storage evolution needs any schema change, stop and ask the user before touching structure.
8. Self-review the touched path. If a better next fix still belongs in app or frontend code, backend or service code, generator inputs, or adjacent cleanup, keep iterating instead of stopping at the first pass.
9. Run verification, then rescan adjacent console packages and one extra global pass.

## Red Flags

- raw `fetch(` or manual HTTP helpers in `console/` app business code
- manual `Authorization` or `Access-Token` assignment in console packages
- handwritten SDK packages that drift from the generator standard
- frontend-only DTO shims hiding a missing backend contract
- any unapproved migration, DDL, or storage schema edit

## Completion Bar

- Console-facing business modules use the generator-backed SDK boundary.
- Native router, gateway, provider, and storage logic still stay on the correct service boundaries.
- No raw HTTP, manual header, mock bypass, or temporary fallback remains in console-facing code.
- Missing contracts are closed in backend/OpenAPI/generator inputs, and no schema change happened without approval.
- Relevant cargo and console verification pass.

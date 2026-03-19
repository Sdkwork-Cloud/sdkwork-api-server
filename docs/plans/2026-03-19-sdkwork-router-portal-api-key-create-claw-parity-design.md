# SDKWork Router Portal API Key Create Flow Claw Parity Design

## Goal

Make the Portal API key creation experience feel like a true Claw Studio sibling by aligning the create entry, dialog hierarchy, form semantics, and post-create handoff with the Claw Api Router key manager.

## User Direction Incorporated

- New API key creation must fully reference Claw Studio.
- The UI, interaction rhythm, styles, and theme behavior must stay consistent with Claw.
- The result should preserve Portal-specific project and environment constraints without looking like a different product.

## Current Gap

The current Portal API key page already shares the broad shell language, but the create flow is still a simplified variant:

- the create dialog is Portal-specific instead of structurally matching Claw's key form
- creation guidance is separated into environment and lifecycle cards rather than one Claw-like form stack
- the table and usage handoff still describe the key as purely Portal-issued instead of a Portal-managed credential surface
- the current flow cannot express Claw's generated-versus-custom key creation model

## Design Decision

Adopt Claw's create-form hierarchy as the primary pattern, then map Portal's workspace API key rules into that hierarchy.

### Create Dialog

- Keep the same large glass modal treatment already used across the Portal shell.
- Replace the current two-section environment/lifecycle layout with a Claw-style form:
  - key label
  - environment boundary selector
  - key mode selector
  - generated/custom key body
  - expiry control
- Use the same visual language as `UnifiedApiKeyForm`:
  - dense two-column grid
  - card-style mode selector
  - inline helper copy beneath each field
  - system-generated info card when generation mode is active

### Functional Parity

- Support two key modes in Portal create flow:
  - system-generated
  - custom key
- Reuse the existing Portal backend path for `label` and `expires_at_ms`.
- Extend the Portal create contract to accept an optional plaintext key when custom mode is chosen.
- Continue storing only the hashed key in persisted records.
- Continue returning plaintext only in the creation response.

### Portal-Specific Mapping

- Claw's `group` field becomes Portal's `environment boundary`.
- Claw's `generate/custom` selector remains conceptually the same.
- Claw's date-based expiry control maps to Portal expiry posture.
- Portal does not add Claw model-mapping association into this create step because the underlying workspace gateway key model does not use it.

### Table / Post-Create Handoff

- Shift source language from `Portal issued` to `Portal managed` so both generated and custom modes remain truthful.
- Keep usage guidance and one-time plaintext copy behavior, but ensure the create flow and usage handoff feel like one connected manager.

## Testing Strategy

- Tighten Portal UI tests so the create dialog must expose:
  - a Claw-style create form component boundary
  - an explicit key mode selector
  - both generated and custom key creation paths
  - a Portal-managed source label in the table
- Add backend tests that prove Portal create endpoints accept and persist metadata for custom keys without exposing plaintext in list results.

## Non-Goals

- No model-mapping association for Portal gateway keys in this iteration.
- No new sidebar, theme, or global shell redesign beyond what already landed.

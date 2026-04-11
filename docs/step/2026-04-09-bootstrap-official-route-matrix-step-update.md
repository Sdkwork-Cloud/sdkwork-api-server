# Bootstrap Official Route Matrix Step Update

## What changed

- added `data/updates/2026-04-global-official-route-matrix.json`
- added grouped seed data for:
  - `data/routing/2026-04-global-official-route-matrix.json`
  - `data/api-key-groups/2026-04-global-official-route-matrix.json`
  - `data/observability/2026-04-global-official-route-matrix.json`
  - `data/billing/2026-04-global-official-route-matrix.json`
- wired the new update pack into both:
  - `data/profiles/prod.json`
  - `data/profiles/dev.json`

## Architecture decision

The new official route matrix keeps a strict separation between:

- `channel`: canonical model inventor or vendor
- `provider`: official or proxy execution entry
- `provider-model`: explicit declaration of which canonical models a provider can actually execute

Because proxy providers do not cover every model under every channel, the official route profiles added in this step deliberately default to official providers only. This keeps bootstrap routing correct by construction and avoids hidden invalid fallback chains for models that do not have a declared `provider-model` mapping.

Proxy and local providers remain first-class, but they are surfaced through the existing catalog and provider-model coverage:

- `provider-openrouter-main`
- `provider-siliconflow-main`
- `provider-ollama-local`

That means admin can safely govern:

- which models exist under a channel
- which models each proxy provider actually exposes
- which price rows belong to official pricing vs proxy pricing
- which route profiles are valid for a given execution posture

## Operational outcome

After this update pack is applied, dev and prod bootstrap both gain:

- per-channel official live API key groups
- per-channel official routing profiles
- routing simulation snapshots for newly seeded official channels
- billing samples for official execution paths
- provider health coverage for all seeded official channels

This improves install-time readiness without introducing secrets, duplicate records, or routing references that cannot be resolved through the declared provider-model graph.

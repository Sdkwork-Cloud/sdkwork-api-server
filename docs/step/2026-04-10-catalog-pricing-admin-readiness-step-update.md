# Catalog Pricing Admin Readiness Step Update

## Completed

- Enforced the catalog guardrail that `model-price` cannot exist unless the matching `provider-model` already exists.
- Made provider-scoped model creation land the full canonical relation: `channel-model`, `provider-model`, and the default `model-price`.
- Upgraded admin catalog management so operators can curate which canonical models each provider supports, with provider-model metadata and pricing coverage visible in one place.
- Added an additive `/data` update pack for catalog pricing readiness instead of mutating baseline bundles.
- Refined friendly pricing posture metadata so official, proxy, and local price rows are explainable in admin and safe to extend in later update packs.

## Why This Design

- `channel` stays the canonical inventor surface and `channel-model` stays the canonical publication record.
- `provider` stays the executable upstream, while `provider-model` represents the provider-supported subset the operator actually enables.
- `model-price` stays provider-scoped, so official pricing and proxy pricing can coexist without collapsing routing, catalog, and commercial billing into one table.
- Additive update packs keep bootstrap idempotent and update-safe: repeated startup rewrites the same stable keys instead of creating duplicates or forcing baseline rewrites.
- The new pack only refines default route posture, pricing readability, and provider-supported model coverage, so it remains high-cohesion and low-coupling with the importer.

## Verified

- `cargo test -p sdkwork-api-app-catalog --test model_price_guardrails -- --nocapture`
- `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes create_model_price_requires_provider_model_support -- --nocapture`
- `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes create_provider_syncs_supported_models_and_exposes_provider_model_registry -- --nocapture`
- `node --test apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/providerCatalog.test.ts`
- `node --test apps/sdkwork-router-admin/tests/admin-catalog-pricing-contract.test.mjs`
- `pnpm --dir apps/sdkwork-router-admin typecheck`

## Next

- Run bootstrap-level verification against repository `/data` so the new update pack is proven in both app runtime and product runtime startup flows.
- Keep future catalog expansion additive: add new update packs that extend `provider-model`, `model-price`, and `routing` bundles without rewriting baseline files.

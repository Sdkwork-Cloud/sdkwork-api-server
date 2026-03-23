# Router Local Check Architecture Map

## Stack

- Rust workspace for router, gateway, storage, provider, and extension-host services
- pnpm console workspace for admin and portal shells
- optional Tauri shell for the console

## Standard Console Remote Path

Use this path for console-facing app business capability backed by `spring-ai-plus-app-api`:

`console app / shared console package -> generator-backed app SDK boundary -> spring-ai-plus-app-api`

Existing packages such as `sdkwork-api-portal-sdk` or `sdkwork-api-admin-sdk` should act as generator-backed boundaries, not handwritten client layers.

## Native And Service Path

Keep these concerns on their original boundaries:

- `crates/sdkwork-api-*` runtime, routing, provider, and extension infrastructure
- `services/*` Rust services and gateway composition
- storage engines and native observability or secret-management layers
- provider-specific outbound HTTP that belongs to native router service logic

Native router capability should stay native even while adjacent console modules move to the generated SDK.

## Replace Or Remove

- raw HTTP in `console/` business packages
- duplicate DTO mapping that only exists to hide a missing SDK method
- handwritten console SDK packages that drift from the generator output
- manual auth header assignment in console service layers

## Contract Closure Rule

If a console-facing package needs a method that the generated SDK does not expose:

1. Fix the contract in `spring-ai-plus-app-api` and required backend modules.
2. Regenerate the SDK package from the repository-standard generator flow.
3. Reconnect the console package through the shared SDK boundary.
4. Delete the temporary bypass.

If that backend or storage work would touch schema, migration, or DB layout, pause and ask the user first.

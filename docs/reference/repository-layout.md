# Repository Layout

## Top-Level Structure

```text
.
|-- crates/
|-- services/
|-- console/
|-- docs/
|-- scripts/
|-- Cargo.toml
|-- README.md
`-- README.zh-CN.md
```

## Backend Layers

- `crates/sdkwork-api-interface-*`
  - HTTP and interface boundaries
- `crates/sdkwork-api-app-*`
  - application or service layer
- `crates/sdkwork-api-domain-*`
  - domain models and policies
- `crates/sdkwork-api-storage-*`
  - repository and persistence implementations

## Standalone Services

- `services/gateway-service`
- `services/admin-api-service`
- `services/portal-api-service`

## Frontend Layers

- `console/src/`
  - shell composition only
- `console/packages/`
  - reusable business modules
- `console/src-tauri/`
  - desktop-native host integration

## Docs and Operational Assets

- `docs/`
  - VitePress docs site plus deep technical references
- `docs/plans/`
  - historical design and implementation records
- `scripts/dev/`
  - cross-platform startup helpers

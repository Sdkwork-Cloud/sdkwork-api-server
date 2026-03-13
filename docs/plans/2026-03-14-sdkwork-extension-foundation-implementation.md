# SDKWork Extension Foundation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Introduce a real extension host foundation for channels and proxy providers, harden gateway tenancy and admin authentication, and upgrade the catalog model so the current OpenAI-compatible gateway can evolve into a pluggable production platform.

**Architecture:** The implementation starts with additive foundations. First add extension contracts and an extension host, then migrate the current built-in provider adapters onto that host. In parallel, remove hardcoded data-plane tenancy, harden admin authentication, and upgrade catalog storage so providers can bind to multiple channels and models can carry capability-rich metadata.

**Tech Stack:** Rust, Axum, Tokio, sqlx, reqwest, serde, jsonwebtoken, SQLite, PostgreSQL

---

### Task 1: Add extension core contracts

**Files:**
- Create: `crates/sdkwork-api-extension-core/Cargo.toml`
- Create: `crates/sdkwork-api-extension-core/src/lib.rs`
- Create: `crates/sdkwork-api-extension-core/tests/manifest_contract.rs`
- Modify: `Cargo.toml`

**Step 1: Write the failing test**

```rust
use sdkwork_api_extension_core::{
    CapabilityDescriptor, CompatibilityLevel, ExtensionKind, ExtensionManifest,
    ExtensionRuntime,
};

#[test]
fn manifest_tracks_kind_runtime_and_capabilities() {
    let manifest = ExtensionManifest::new(
        "sdkwork.provider.openrouter",
        ExtensionKind::Provider,
        "0.1.0",
        ExtensionRuntime::Builtin,
    )
    .with_capability(CapabilityDescriptor::new(
        "responses.create",
        CompatibilityLevel::Relay,
    ))
    .with_channel_binding("sdkwork.channel.openai");

    assert_eq!(manifest.id, "sdkwork.provider.openrouter");
    assert_eq!(manifest.kind, ExtensionKind::Provider);
    assert_eq!(manifest.runtime, ExtensionRuntime::Builtin);
    assert_eq!(manifest.capabilities[0].operation, "responses.create");
    assert_eq!(manifest.capabilities[0].compatibility, CompatibilityLevel::Relay);
    assert_eq!(manifest.channel_bindings, vec!["sdkwork.channel.openai"]);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-extension-core --test manifest_contract -q`

Expected: FAIL because the crate does not exist yet.

**Step 3: Write minimal implementation**

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExtensionKind {
    Channel,
    Provider,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExtensionRuntime {
    Builtin,
    NativeDynamic,
    Connector,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompatibilityLevel {
    Native,
    Relay,
    Translated,
    Emulated,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityDescriptor {
    pub operation: String,
    pub compatibility: CompatibilityLevel,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtensionManifest {
    pub api_version: String,
    pub id: String,
    pub kind: ExtensionKind,
    pub version: String,
    pub runtime: ExtensionRuntime,
    pub channel_bindings: Vec<String>,
    pub capabilities: Vec<CapabilityDescriptor>,
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p sdkwork-api-extension-core --test manifest_contract -q`

Expected: PASS

**Step 5: Commit**

```bash
git add Cargo.toml crates/sdkwork-api-extension-core
git commit -m "feat: add extension core contracts"
```

### Task 2: Add extension host and built-in loader

**Files:**
- Create: `crates/sdkwork-api-extension-host/Cargo.toml`
- Create: `crates/sdkwork-api-extension-host/src/lib.rs`
- Create: `crates/sdkwork-api-extension-host/tests/builtin_host.rs`
- Modify: `Cargo.toml`
- Modify: `crates/sdkwork-api-provider-core/src/lib.rs`

**Step 1: Write the failing test**

```rust
use sdkwork_api_extension_core::{ExtensionKind, ExtensionManifest, ExtensionRuntime};
use sdkwork_api_extension_host::{BuiltinExtensionFactory, ExtensionHost};

#[test]
fn host_registers_and_resolves_builtin_provider_extensions() {
    let mut host = ExtensionHost::new();
    host.register_builtin(BuiltinExtensionFactory::new(
        ExtensionManifest::new(
            "sdkwork.provider.openai.official",
            ExtensionKind::Provider,
            "0.1.0",
            ExtensionRuntime::Builtin,
        ),
    ));

    let manifest = host
        .manifest("sdkwork.provider.openai.official")
        .expect("manifest");

    assert_eq!(manifest.id, "sdkwork.provider.openai.official");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-extension-host --test builtin_host -q`

Expected: FAIL because the host crate does not exist yet.

**Step 3: Write minimal implementation**

```rust
#[derive(Default)]
pub struct ExtensionHost {
    manifests: HashMap<String, ExtensionManifest>,
}

impl ExtensionHost {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_builtin(&mut self, factory: BuiltinExtensionFactory) {
        self.manifests
            .insert(factory.manifest.id.clone(), factory.manifest);
    }

    pub fn manifest(&self, id: &str) -> Option<&ExtensionManifest> {
        self.manifests.get(id)
    }
}
```

**Step 4: Bridge provider execution adapters into the host**

Add an additive wrapper in `crates/sdkwork-api-provider-core/src/lib.rs` so the host can keep using the current `ProviderExecutionAdapter` trait while the runtime architecture changes underneath it.

```rust
pub struct ProviderExecutionDescriptor {
    pub extension_id: String,
    pub adapter_kind: String,
    pub base_url: String,
}
```

**Step 5: Run tests to verify host registration**

Run:

- `cargo test -p sdkwork-api-extension-host --test builtin_host -q`
- `cargo test -p sdkwork-api-provider-core -q`

Expected: PASS

**Step 6: Commit**

```bash
git add Cargo.toml crates/sdkwork-api-extension-host crates/sdkwork-api-provider-core
git commit -m "feat: add extension host foundation"
```

### Task 3: Migrate gateway dispatch from static provider registry to extension host

**Files:**
- Modify: `crates/sdkwork-api-app-gateway/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-http/src/lib.rs`
- Create: `crates/sdkwork-api-app-gateway/tests/extension_dispatch.rs`

**Step 1: Write the failing test**

```rust
use sdkwork_api_app_gateway::builtin_extension_host;

#[test]
fn builtin_host_registers_current_provider_extensions() {
    let host = builtin_extension_host();
    assert!(host.manifest("sdkwork.provider.openai.official").is_some());
    assert!(host.manifest("sdkwork.provider.openrouter").is_some());
    assert!(host.manifest("sdkwork.provider.ollama").is_some());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-app-gateway --test extension_dispatch -q`

Expected: FAIL because the gateway still uses `default_provider_registry()`.

**Step 3: Replace the registry constructor with a built-in host**

Refactor:

- replace `default_provider_registry()` with `builtin_extension_host()`
- resolve provider execution adapters through the host
- keep request execution behavior unchanged for current built-in providers

```rust
fn builtin_extension_host() -> ExtensionHost {
    let mut host = ExtensionHost::new();
    host.register_builtin(openai_builtin_extension());
    host.register_builtin(openrouter_builtin_extension());
    host.register_builtin(ollama_builtin_extension());
    host
}
```

**Step 4: Run the gateway relay tests**

Run:

- `cargo test -p sdkwork-api-app-gateway --test extension_dispatch -q`
- `cargo test -p sdkwork-api-provider-openai -q`
- `cargo test -p sdkwork-api-interface-http --test chat_route -q`
- `cargo test -p sdkwork-api-interface-http --test responses_route -q`

Expected: PASS with no behavior regression for existing OpenAI-compatible relays.

**Step 5: Commit**

```bash
git add crates/sdkwork-api-app-gateway/src/lib.rs crates/sdkwork-api-interface-http/src/lib.rs crates/sdkwork-api-app-gateway/tests/extension_dispatch.rs
git commit -m "refactor: route provider dispatch through extension host"
```

### Task 4: Add real gateway API key request context extraction

**Files:**
- Modify: `crates/sdkwork-api-storage-core/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-postgres/src/lib.rs`
- Modify: `crates/sdkwork-api-app-identity/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-http/src/lib.rs`
- Create: `crates/sdkwork-api-interface-http/tests/gateway_auth_context.rs`

**Step 1: Write the failing test**

```rust
#[tokio::test]
async fn gateway_request_uses_api_key_context_instead_of_hardcoded_tenant() {
    let response = call_models_with_bearer("skw_live_demo");
    assert_eq!(response.status(), StatusCode::OK);

    let body = read_json(response).await;
    assert_eq!(body["data"][0]["owned_by"], "project-live");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-interface-http --test gateway_auth_context -q`

Expected: FAIL because the handlers still use `"tenant-1"` and `"project-1"`.

**Step 3: Add store lookup and request context types**

Add an additive lookup to `AdminStore` and storage implementations:

```rust
async fn find_gateway_api_key(&self, hashed_key: &str) -> Result<Option<GatewayApiKeyRecord>>;
```

Add a request context in `sdkwork-api-app-identity`:

```rust
pub struct GatewayRequestContext {
    pub tenant_id: String,
    pub project_id: String,
    pub environment: String,
}
```

**Step 4: Add an Axum extractor or middleware**

The HTTP interface should:

- read `Authorization: Bearer <gateway_key>`
- hash the presented value
- resolve the gateway API key record from the store
- reject missing or inactive keys
- attach the derived tenant and project to request handling

**Step 5: Replace hardcoded tenancy in handlers**

All stateful gateway handlers should read from the extracted request context instead of literals.

**Step 6: Run focused tests**

Run:

- `cargo test -p sdkwork-api-interface-http --test gateway_auth_context -q`
- `cargo test -p sdkwork-api-interface-http --test chat_route -q`
- `cargo test -p sdkwork-api-interface-http --test embeddings_route -q`

Expected: PASS

**Step 7: Commit**

```bash
git add crates/sdkwork-api-storage-core/src/lib.rs crates/sdkwork-api-storage-sqlite/src/lib.rs crates/sdkwork-api-storage-postgres/src/lib.rs crates/sdkwork-api-app-identity/src/lib.rs crates/sdkwork-api-interface-http/src/lib.rs crates/sdkwork-api-interface-http/tests/gateway_auth_context.rs
git commit -m "feat: derive gateway request context from api keys"
```

### Task 5: Harden admin JWTs and guard admin routes

**Files:**
- Modify: `Cargo.toml`
- Modify: `crates/sdkwork-api-app-identity/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Create: `crates/sdkwork-api-interface-admin/tests/admin_auth_guard.rs`

**Step 1: Write the failing test**

```rust
#[tokio::test]
async fn admin_routes_require_valid_bearer_token() {
    let unauthorized = get_admin_projects_without_token().await;
    assert_eq!(unauthorized.status(), StatusCode::UNAUTHORIZED);

    let authorized = get_admin_projects_with_valid_token().await;
    assert_eq!(authorized.status(), StatusCode::OK);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-interface-admin --test admin_auth_guard -q`

Expected: FAIL because admin routes are currently open and JWTs are unsigned.

**Step 3: Replace demo token format with signed JWTs**

Add `jsonwebtoken` to the workspace and implement:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub iss: String,
    pub aud: String,
    pub exp: usize,
    pub iat: usize,
}
```

Issue with `HS256` using a configured signing secret, and verify `iss`, `aud`, and `exp`.

**Step 4: Add an admin auth layer**

Guard all `/admin/*` routes except:

- `/admin/health`
- `/admin/auth/login`

**Step 5: Run focused tests**

Run:

- `cargo test -p sdkwork-api-app-identity -q`
- `cargo test -p sdkwork-api-interface-admin --test admin_auth_guard -q`
- `cargo test -p sdkwork-api-interface-admin --test auth_and_project_routes -q`

Expected: PASS

**Step 6: Commit**

```bash
git add Cargo.toml crates/sdkwork-api-app-identity/src/lib.rs crates/sdkwork-api-interface-admin/src/lib.rs crates/sdkwork-api-interface-admin/tests/admin_auth_guard.rs
git commit -m "feat: harden admin jwt auth"
```

### Task 6: Upgrade catalog modeling for multi-channel providers and capability-rich models

**Files:**
- Modify: `crates/sdkwork-api-domain-catalog/src/lib.rs`
- Modify: `crates/sdkwork-api-app-catalog/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-core/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-postgres/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Create: `crates/sdkwork-api-domain-catalog/tests/provider_channel_binding.rs`
- Create: `crates/sdkwork-api-storage-sqlite/tests/catalog_bindings.rs`
- Create: `crates/sdkwork-api-storage-postgres/tests/catalog_bindings.rs`

**Step 1: Write the failing tests**

```rust
use sdkwork_api_domain_catalog::{ModelCapability, ModelVariant, ProviderChannelBinding, ProxyProvider};

#[test]
fn provider_can_bind_to_multiple_channels() {
    let provider = ProxyProvider::new("provider-openrouter", "openrouter", "openai", "https://openrouter.ai/api/v1", "OpenRouter");
    let binding = ProviderChannelBinding::new(&provider.id, "sdkwork.channel.openai");
    assert_eq!(binding.channel_id, "sdkwork.channel.openai");
}

#[test]
fn model_variant_tracks_capabilities_and_streaming() {
    let model = ModelVariant::new("gpt-4.1", "provider-openai-official")
        .with_capability(ModelCapability::Responses)
        .with_capability(ModelCapability::ChatCompletions)
        .with_streaming(true);

    assert!(model.streaming);
    assert_eq!(model.capabilities.len(), 2);
}
```

**Step 2: Run tests to verify they fail**

Run:

- `cargo test -p sdkwork-api-domain-catalog --test provider_channel_binding -q`
- `cargo test -p sdkwork-api-storage-sqlite --test catalog_bindings -q`

Expected: FAIL because the domain and storage schema do not support these concepts yet.

**Step 3: Add additive domain and storage structures**

Upgrade the catalog model with:

- `ProviderChannelBinding`
- capability-rich `ModelVariant`
- additive storage tables for provider bindings and model metadata

Prefer additive schema changes instead of destructive replacements so existing tests and sample data can be migrated safely.

**Step 4: Update admin create/list APIs**

Admin APIs should accept and return:

- provider channel bindings
- model capabilities
- streaming support
- optional context or pricing metadata

**Step 5: Run focused catalog tests**

Run:

- `cargo test -p sdkwork-api-domain-catalog --test provider_channel_binding -q`
- `cargo test -p sdkwork-api-storage-sqlite --test catalog_bindings -q`
- `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes -q`

Expected: PASS

**Step 6: Commit**

```bash
git add crates/sdkwork-api-domain-catalog/src/lib.rs crates/sdkwork-api-app-catalog/src/lib.rs crates/sdkwork-api-storage-core/src/lib.rs crates/sdkwork-api-storage-sqlite/src/lib.rs crates/sdkwork-api-storage-postgres/src/lib.rs crates/sdkwork-api-interface-admin/src/lib.rs crates/sdkwork-api-domain-catalog/tests/provider_channel_binding.rs crates/sdkwork-api-storage-sqlite/tests/catalog_bindings.rs crates/sdkwork-api-storage-postgres/tests/catalog_bindings.rs
git commit -m "feat: enrich catalog with channel bindings and model metadata"
```

### Task 7: Add extension installation and instance configuration support

**Files:**
- Modify: `crates/sdkwork-api-domain-catalog/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-core/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-postgres/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Create: `crates/sdkwork-api-extension-host/tests/instance_mounting.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn installation_can_mount_multiple_instances_from_one_package() {
    let mut host = ExtensionHost::new();
    host.mount_instance("sdkwork.provider.openrouter", "provider-openrouter-main");
    host.mount_instance("sdkwork.provider.openrouter", "provider-openrouter-backup");

    assert_eq!(host.instances("sdkwork.provider.openrouter").len(), 2);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-extension-host --test instance_mounting -q`

Expected: FAIL because the host has no installation or instance model yet.

**Step 3: Add additive configuration entities**

Add:

- `ExtensionPackage`
- `ExtensionInstallation`
- `ExtensionInstance`

Persist enough data to support:

- enabled or disabled state
- entrypoint
- runtime mode
- instance config payload
- credential reference

**Step 4: Wire admin control-plane endpoints**

Expose list and create handlers for extension packages or instances through the admin API.

**Step 5: Run focused tests**

Run:

- `cargo test -p sdkwork-api-extension-host --test instance_mounting -q`
- `cargo test -p sdkwork-api-interface-admin -q`

Expected: PASS

**Step 6: Commit**

```bash
git add crates/sdkwork-api-domain-catalog/src/lib.rs crates/sdkwork-api-storage-core/src/lib.rs crates/sdkwork-api-storage-sqlite/src/lib.rs crates/sdkwork-api-storage-postgres/src/lib.rs crates/sdkwork-api-interface-admin/src/lib.rs crates/sdkwork-api-extension-host/tests/instance_mounting.rs
git commit -m "feat: add extension installation and instance config"
```

### Task 8: Reclassify compatibility coverage and document real implementation levels

**Files:**
- Modify: `docs/api/compatibility-matrix.md`
- Modify: `README.md`
- Create: `crates/sdkwork-api-extension-core/tests/compatibility_level.rs`

**Step 1: Write the failing test**

```rust
use sdkwork_api_extension_core::CompatibilityLevel;

#[test]
fn compatibility_levels_cover_gateway_truth_model() {
    let values = [
        CompatibilityLevel::Native,
        CompatibilityLevel::Relay,
        CompatibilityLevel::Translated,
        CompatibilityLevel::Emulated,
        CompatibilityLevel::Unsupported,
    ];

    assert_eq!(values.len(), 5);
}
```

**Step 2: Run test to verify it fails if the enum or documentation model is incomplete**

Run: `cargo test -p sdkwork-api-extension-core --test compatibility_level -q`

Expected: FAIL until the compatibility taxonomy is fully wired and documented.

**Step 3: Update documentation to reflect reality**

Classify each API family by actual behavior:

- `native`
- `relay`
- `translated`
- `emulated`
- `unsupported`

Do not describe placeholder local fallbacks as complete implementations.

**Step 4: Run documentation-adjacent verification**

Run:

- `cargo test -p sdkwork-api-extension-core --test compatibility_level -q`
- `cargo test --workspace -q`

Expected: PASS

**Step 5: Commit**

```bash
git add docs/api/compatibility-matrix.md README.md crates/sdkwork-api-extension-core/tests/compatibility_level.rs
git commit -m "docs: classify compatibility by execution level"
```

### Task 9: Full verification before broader API family expansion

**Files:**
- Modify: `README.md`
- Modify: `docs/architecture/runtime-modes.md`

**Step 1: Run workspace verification**

Run:

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo test --workspace -q`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `pnpm --dir console -r typecheck`
- `pnpm --dir console exec vite build`

Expected: PASS

**Step 2: Update final docs**

Document:

- built-in extension host availability
- gateway API key request context extraction
- admin JWT requirements
- new extension naming and lifecycle model

**Step 3: Commit**

```bash
git add README.md docs/architecture/runtime-modes.md
git commit -m "docs: update runtime and extension implementation status"
```

# Extension Loader Standardization Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Standardize extension package metadata and add configuration-driven load planning so provider and channel extensions can be mounted as pluggable runtime units.

**Architecture:** Build additively on the current `ExtensionManifest / ExtensionInstallation / ExtensionInstance` model. First enrich the manifest with packaging and schema metadata, then let `ExtensionHost` derive deterministic runtime load plans by combining manifest defaults, installation state, and instance overrides. Keep actual built-in provider execution intact while preparing connector and native-dynamic runtimes for future activation.

**Tech Stack:** Rust, serde, serde_json, Axum workspace crates, sqlx-backed config persistence

---

### Task 1: Add failing tests for extension standard metadata

**Files:**
- Create: `crates/sdkwork-api-extension-core/tests/extension_standard.rs`
- Modify: `crates/sdkwork-api-extension-core/src/lib.rs`

**Step 1: Write the failing test**

```rust
use sdkwork_api_extension_core::{ExtensionKind, ExtensionManifest, ExtensionRuntime};

#[test]
fn manifest_derives_distribution_and_crate_names_from_runtime_id() {
    let manifest = ExtensionManifest::new(
        "sdkwork.provider.openrouter",
        ExtensionKind::Provider,
        "0.1.0",
        ExtensionRuntime::Connector,
    )
    .with_display_name("OpenRouter")
    .with_entrypoint("bin/sdkwork-provider-openrouter")
    .with_config_schema("schemas/config.schema.json")
    .with_credential_schema("schemas/credential.schema.json");

    assert_eq!(manifest.distribution_name(), "sdkwork-provider-openrouter");
    assert_eq!(manifest.crate_name(), "sdkwork-api-ext-provider-openrouter");
    assert_eq!(manifest.display_name, "OpenRouter");
    assert_eq!(
        manifest.entrypoint.as_deref(),
        Some("bin/sdkwork-provider-openrouter")
    );
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-extension-core --test extension_standard -q`

Expected: FAIL because the manifest does not yet expose standardized package metadata or extra fields.

**Step 3: Write minimal implementation**

Additive manifest fields and helpers:

- `display_name`
- `entrypoint`
- `config_schema`
- `credential_schema`
- `distribution_name()`
- `crate_name()`

Keep the existing constructor working and derive package names from runtime IDs such as:

- `sdkwork.provider.openrouter` -> `sdkwork-provider-openrouter`
- `sdkwork.channel.openai` -> `sdkwork-channel-openai`

**Step 4: Run test to verify it passes**

Run: `cargo test -p sdkwork-api-extension-core --test extension_standard -q`

Expected: PASS

**Step 5: Commit**

```bash
git add crates/sdkwork-api-extension-core/src/lib.rs crates/sdkwork-api-extension-core/tests/extension_standard.rs
git commit -m "feat: standardize extension package metadata"
```

### Task 2: Add configuration-driven extension load planning

**Files:**
- Create: `crates/sdkwork-api-extension-host/tests/load_planning.rs`
- Modify: `crates/sdkwork-api-extension-host/src/lib.rs`

**Step 1: Write the failing test**

```rust
use sdkwork_api_extension_core::{
    ExtensionInstallation, ExtensionInstance, ExtensionKind, ExtensionManifest, ExtensionRuntime,
};
use sdkwork_api_extension_host::{BuiltinExtensionFactory, ExtensionHost};

#[test]
fn host_builds_runtime_load_plan_from_manifest_installation_and_instance() {
    let mut host = ExtensionHost::new();
    host.register_builtin(BuiltinExtensionFactory::new(
        ExtensionManifest::new(
            "sdkwork.provider.openrouter",
            ExtensionKind::Provider,
            "0.1.0",
            ExtensionRuntime::Connector,
        )
        .with_entrypoint("bin/default-openrouter")
        .with_config_schema("schemas/config.schema.json"),
    ));

    host.install(
        ExtensionInstallation::new(
            "openrouter-installation",
            "sdkwork.provider.openrouter",
            ExtensionRuntime::Connector,
        )
        .with_entrypoint("bin/sdkwork-provider-openrouter")
        .with_config(serde_json::json!({"timeout_secs": 30, "region": "global"})),
    )
    .unwrap();

    host.mount_instance(
        ExtensionInstance::new(
            "provider-openrouter-main",
            "openrouter-installation",
            "sdkwork.provider.openrouter",
        )
        .with_base_url("https://openrouter.ai/api/v1")
        .with_credential_ref("cred-openrouter")
        .with_config(serde_json::json!({"region": "us", "weight": 100})),
    )
    .unwrap();

    let plan = host
        .load_plan("provider-openrouter-main")
        .expect("plan should build");

    assert_eq!(plan.extension_id, "sdkwork.provider.openrouter");
    assert_eq!(plan.runtime, ExtensionRuntime::Connector);
    assert_eq!(
        plan.entrypoint.as_deref(),
        Some("bin/sdkwork-provider-openrouter")
    );
    assert_eq!(
        plan.base_url.as_deref(),
        Some("https://openrouter.ai/api/v1")
    );
    assert_eq!(plan.credential_ref.as_deref(), Some("cred-openrouter"));
    assert_eq!(plan.config["timeout_secs"], 30);
    assert_eq!(plan.config["region"], "us");
    assert_eq!(plan.config["weight"], 100);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-extension-host --test load_planning -q`

Expected: FAIL because the host does not yet expose a load-plan abstraction.

**Step 3: Write minimal implementation**

Add:

- `ExtensionLoadPlan`
- `ExtensionHost::load_plan(instance_id)`
- installation and instance indexing needed for lookup
- merged config behavior where instance config overrides installation config
- runtime validation for missing manifests or mismatched runtimes

Keep provider execution resolution unchanged for built-in adapters.

**Step 4: Run test to verify it passes**

Run: `cargo test -p sdkwork-api-extension-host --test load_planning -q`

Expected: PASS

**Step 5: Run focused regression tests**

Run:

- `cargo test -p sdkwork-api-extension-host -q`
- `cargo test -p sdkwork-api-app-gateway --test extension_dispatch -q`

Expected: PASS

**Step 6: Commit**

```bash
git add crates/sdkwork-api-extension-host/src/lib.rs crates/sdkwork-api-extension-host/tests/load_planning.rs
git commit -m "feat: add extension runtime load planning"
```

### Task 3: Document extension naming and load semantics

**Files:**
- Modify: `README.md`
- Modify: `docs/architecture/runtime-modes.md`

**Step 1: Update docs**

Document:

- runtime ID naming (`sdkwork.provider.*`, `sdkwork.channel.*`)
- distribution naming (`sdkwork-provider-*`, `sdkwork-channel-*`)
- Rust crate naming (`sdkwork-api-ext-provider-*`, `sdkwork-api-ext-channel-*`)
- config merge order: manifest defaults -> installation config -> instance config
- `connector` and `native_dynamic` runtimes as configuration-driven load plans even before executable loaders are wired

**Step 2: Run verification**

Run:

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo test --workspace -q`

Expected: PASS

**Step 3: Commit**

```bash
git add README.md docs/architecture/runtime-modes.md
git commit -m "docs: describe extension load standard"
```

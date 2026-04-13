use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use reqwest::Client;
use sdkwork_api_app_credential::CredentialSecretManager;
use sdkwork_api_app_runtime::{
    CommercialBillingReadKernel, build_admin_payment_store_handles_from_config,
};
use sdkwork_api_config::{CacheBackendKind, StandaloneConfigLoader};
use sdkwork_api_interface_http::GatewayApiState;
use sdkwork_api_product_runtime::{
    ProductRuntimeRole, ProductSiteDirs, RouterProductRuntime, RouterProductRuntimeOptions,
};
use sdkwork_api_storage_core::{AccountKernelStore, AdminStore, Reloadable};
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};

static TEMP_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

#[tokio::test]
async fn product_runtime_builder_wires_gateway_billing_handle_into_gateway_state() {
    let config_root = temp_root("gateway-billing-handle");
    let database_path = config_root.join("gateway-billing.db");
    let database_url = sqlite_url_for(database_path);
    let (_loader, mut config) = StandaloneConfigLoader::from_local_root_and_pairs(
        &config_root,
        [("SDKWORK_ALLOW_LOCAL_DEV_BOOTSTRAP", "true")],
    )
    .unwrap();
    config.database_url = database_url;

    let store_handles = build_admin_payment_store_handles_from_config(&config)
        .await
        .unwrap();
    let state =
        GatewayApiState::with_live_store_commercial_billing_payment_store_and_secret_manager_handle(
            Reloadable::new(store_handles.admin_store),
            Reloadable::new(store_handles.gateway_commercial_billing),
            Reloadable::new(store_handles.payment_store),
            Reloadable::new(CredentialSecretManager::new_with_legacy_master_keys(
                config.secret_backend,
                config.credential_master_key.clone(),
                config.credential_legacy_master_keys.clone(),
                config.secret_local_file.clone(),
                config.secret_keyring_service.clone(),
            )),
        );

    let _ = state.clone();
}

#[tokio::test]
async fn desktop_product_runtime_serves_static_sites_and_all_api_health_routes() {
    let config_root = temp_root("desktop-runtime-config");
    let admin_site_dir = temp_root("desktop-admin-site");
    let portal_site_dir = temp_root("desktop-portal-site");
    fs::write(
        admin_site_dir.join("index.html"),
        "<!doctype html><html><body>admin desktop site</body></html>",
    )
    .unwrap();
    fs::write(
        portal_site_dir.join("index.html"),
        "<!doctype html><html><body>portal desktop site</body></html>",
    )
    .unwrap();

    let (loader, config) = StandaloneConfigLoader::from_local_root_and_pairs(
        &config_root,
        [("SDKWORK_ALLOW_LOCAL_DEV_BOOTSTRAP", "true")],
    )
    .unwrap();

    let runtime = RouterProductRuntime::start(
        loader,
        config,
        RouterProductRuntimeOptions::desktop(ProductSiteDirs::new(
            &admin_site_dir,
            &portal_site_dir,
        )),
    )
    .await
    .unwrap();

    let base_url = runtime.public_base_url().unwrap().to_owned();
    let snapshot = runtime.snapshot();
    let client = http_client();

    assert_eq!(snapshot.mode, "desktop");
    assert_eq!(
        snapshot.roles,
        vec![
            "web".to_owned(),
            "gateway".to_owned(),
            "admin".to_owned(),
            "portal".to_owned()
        ]
    );
    assert_eq!(snapshot.public_base_url.as_deref(), Some(base_url.as_str()));
    assert!(snapshot
        .public_bind_addr
        .as_deref()
        .unwrap()
        .starts_with("127.0.0.1:"));
    assert!(snapshot
        .gateway_bind_addr
        .as_deref()
        .unwrap()
        .starts_with("127.0.0.1:"));
    assert!(snapshot
        .admin_bind_addr
        .as_deref()
        .unwrap()
        .starts_with("127.0.0.1:"));
    assert!(snapshot
        .portal_bind_addr
        .as_deref()
        .unwrap()
        .starts_with("127.0.0.1:"));

    assert_eq!(
        client
            .get(format!("{base_url}/api/admin/health"))
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap(),
        "ok"
    );
    assert_eq!(
        client
            .get(format!("{base_url}/api/portal/health"))
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap(),
        "ok"
    );
    assert_eq!(
        client
            .get(format!("{base_url}/api/v1/health"))
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap(),
        "ok"
    );
    assert!(client
        .get(format!("{base_url}/admin/"))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap()
        .contains("admin desktop site"));
    assert!(client
        .get(format!("{base_url}/portal/"))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap()
        .contains("portal desktop site"));
}

#[tokio::test]
async fn desktop_product_runtime_rejects_local_dev_defaults_without_explicit_dev_mode() {
    let config_root = temp_root("desktop-runtime-security");
    let admin_site_dir = temp_root("desktop-security-admin-site");
    let portal_site_dir = temp_root("desktop-security-portal-site");
    fs::write(admin_site_dir.join("index.html"), "admin").unwrap();
    fs::write(portal_site_dir.join("index.html"), "portal").unwrap();

    let (loader, config) = StandaloneConfigLoader::from_local_root_and_pairs(
        &config_root,
        std::iter::empty::<(&str, &str)>(),
    )
    .unwrap();

    let error = RouterProductRuntime::start(
        loader,
        config,
        RouterProductRuntimeOptions::desktop(ProductSiteDirs::new(
            &admin_site_dir,
            &portal_site_dir,
        )),
    )
    .await
    .err()
    .expect("runtime should reject insecure local-dev startup defaults");

    assert!(
        error
            .to_string()
            .contains("SDKWORK_ALLOW_LOCAL_DEV_BOOTSTRAP"),
        "{error}"
    );
}

#[tokio::test]
async fn desktop_product_runtime_does_not_bootstrap_default_users_without_explicit_dev_mode() {
    let config_root = temp_root("desktop-runtime-no-default-users");
    let admin_site_dir = temp_root("desktop-no-default-admin-site");
    let portal_site_dir = temp_root("desktop-no-default-portal-site");
    fs::write(admin_site_dir.join("index.html"), "admin").unwrap();
    fs::write(portal_site_dir.join("index.html"), "portal").unwrap();

    let (loader, config) = StandaloneConfigLoader::from_local_root_and_pairs(
        &config_root,
        [
            (
                "SDKWORK_ADMIN_JWT_SIGNING_SECRET",
                "prod-admin-jwt-secret-1234567890",
            ),
            (
                "SDKWORK_PORTAL_JWT_SIGNING_SECRET",
                "prod-portal-jwt-secret-1234567890",
            ),
            (
                "SDKWORK_CREDENTIAL_MASTER_KEY",
                "prod-master-key-1234567890",
            ),
        ],
    )
    .unwrap();

    let runtime = RouterProductRuntime::start(
        loader,
        config,
        RouterProductRuntimeOptions::desktop(ProductSiteDirs::new(
            &admin_site_dir,
            &portal_site_dir,
        )),
    )
    .await
    .unwrap();

    let base_url = runtime.public_base_url().unwrap().to_owned();
    let client = http_client();

    let admin_response = client
        .post(format!("{base_url}/api/admin/auth/login"))
        .header("content-type", "application/json")
        .body(r#"{"email":"admin@sdkwork.local","password":"ChangeMe123!"}"#)
        .send()
        .await
        .unwrap();
    assert_eq!(admin_response.status(), reqwest::StatusCode::UNAUTHORIZED);

    let portal_response = client
        .post(format!("{base_url}/api/portal/auth/login"))
        .header("content-type", "application/json")
        .body(r#"{"email":"portal@sdkwork.local","password":"ChangeMe123!"}"#)
        .send()
        .await
        .unwrap();
    assert_eq!(portal_response.status(), reqwest::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn server_product_runtime_rejects_web_role_without_required_api_upstreams() {
    let config_root = temp_root("server-runtime-config");
    let admin_site_dir = temp_root("server-admin-site");
    let portal_site_dir = temp_root("server-portal-site");
    fs::write(admin_site_dir.join("index.html"), "admin").unwrap();
    fs::write(portal_site_dir.join("index.html"), "portal").unwrap();

    let (loader, config) = StandaloneConfigLoader::from_local_root_and_pairs(
        &config_root,
        [("SDKWORK_ALLOW_LOCAL_DEV_BOOTSTRAP", "true")],
    )
    .unwrap();

    let error = RouterProductRuntime::start(
        loader,
        config,
        RouterProductRuntimeOptions::server(ProductSiteDirs::new(
            &admin_site_dir,
            &portal_site_dir,
        ))
        .with_roles([ProductRuntimeRole::Web]),
    )
    .await
    .err()
    .expect("web-only server runtime without API upstreams should fail");

    assert!(error.to_string().contains("gateway upstream"));
}

#[tokio::test]
async fn product_runtime_supports_redis_cache_backend_during_startup() {
    let config_root = temp_root("runtime-cache-backend");
    let (loader, mut config) = StandaloneConfigLoader::from_local_root_and_pairs(
        &config_root,
        [("SDKWORK_ALLOW_LOCAL_DEV_BOOTSTRAP", "true")],
    )
    .unwrap();
    let redis_server = MinimalRedisPingServer::start();
    config.cache_backend = CacheBackendKind::Redis;
    config.cache_url = Some(redis_server.url_with_db(0));

    let runtime = RouterProductRuntime::start(
        loader,
        config,
        RouterProductRuntimeOptions::desktop(ProductSiteDirs::new(
            config_root.join("unused-admin-site"),
            config_root.join("unused-portal-site"),
        ))
        .with_roles(Vec::<ProductRuntimeRole>::new()),
    )
    .await
    .unwrap();

    assert_eq!(runtime.snapshot().roles, Vec::<String>::new());
}

#[tokio::test]
async fn product_runtime_bootstraps_repository_default_data_pack() {
    let config_root = temp_root("product-runtime-bootstrap");
    let database_path = config_root.join("product-runtime-bootstrap.db");
    let database_url = sqlite_url_for(database_path.clone());
    let (loader, mut config) = StandaloneConfigLoader::from_local_root_and_pairs(
        &config_root,
        std::iter::empty::<(&str, &str)>(),
    )
    .unwrap();
    config.database_url = database_url;
    config.bootstrap_profile = "prod".to_owned();

    let runtime = RouterProductRuntime::start(
        loader,
        config,
        RouterProductRuntimeOptions::desktop(ProductSiteDirs::new(
            config_root.join("unused-admin-site"),
            config_root.join("unused-portal-site"),
        ))
        .with_roles(Vec::<ProductRuntimeRole>::new()),
    )
    .await
    .unwrap();
    drop(runtime);

    let pool = run_migrations(&sqlite_url_for(database_path))
        .await
        .unwrap();
    let store = SqliteAdminStore::new(pool);
    let channels = store.list_channels().await.unwrap();
    let providers = store.list_providers().await.unwrap();
    let official_provider_configs = store.list_official_provider_configs().await.unwrap();
    let payment_methods = store.list_payment_methods().await.unwrap();
    let extension_instances = store.list_extension_instances().await.unwrap();
    let routing_profiles = store.list_routing_profiles().await.unwrap();
    let api_key_groups = store.list_api_key_groups().await.unwrap();
    let rate_limit_policies = store.list_rate_limit_policies().await.unwrap();
    let admin_users = store.list_admin_users().await.unwrap();
    let portal_users = store.list_portal_users().await.unwrap();
    let gateway_api_keys = store.list_gateway_api_keys().await.unwrap();
    let compiled_routing_snapshots = store.list_compiled_routing_snapshots().await.unwrap();
    let routing_decision_logs = store.list_routing_decision_logs().await.unwrap();
    let provider_health_snapshots = store.list_provider_health_snapshots().await.unwrap();
    let service_runtime_nodes = store.list_service_runtime_nodes().await.unwrap();
    let extension_runtime_rollouts = store.list_extension_runtime_rollouts().await.unwrap();
    let extension_runtime_rollout_participants = store
        .list_extension_runtime_rollout_participants("rollout-global-openrouter-instance-refresh")
        .await
        .unwrap();
    let standalone_config_rollouts = store.list_standalone_config_rollouts().await.unwrap();
    let standalone_config_rollout_participants = store
        .list_standalone_config_rollout_participants("rollout-global-gateway-config-reload")
        .await
        .unwrap();
    let commerce_orders = store.list_commerce_orders().await.unwrap();
    let commerce_payment_attempts = store.list_commerce_payment_attempts().await.unwrap();
    let commerce_payment_events = store.list_commerce_payment_events().await.unwrap();
    let commerce_webhook_inbox = store.list_commerce_webhook_inbox_records().await.unwrap();
    let commerce_refunds = store.list_commerce_refunds().await.unwrap();
    let commerce_reconciliation_runs = store.list_commerce_reconciliation_runs().await.unwrap();
    let commerce_reconciliation_items = store
        .list_commerce_reconciliation_items("recon-run-global-growth-bootstrap")
        .await
        .unwrap();
    let provider_mix_reconciliation_items = store
        .list_commerce_reconciliation_items("recon-run-provider-mix-openrouter-2026")
        .await
        .unwrap();
    let official_direct_reconciliation_items = store
        .list_commerce_reconciliation_items("recon-run-global-official-openai-2026")
        .await
        .unwrap();
    let local_edge_reconciliation_items = store
        .list_commerce_reconciliation_items("recon-run-edge-local-ollama-2026")
        .await
        .unwrap();
    let billing_events = store.list_billing_events().await.unwrap();
    let async_jobs = store.list_async_jobs().await.unwrap();
    let async_job_attempts = store
        .list_async_job_attempts("job-global-growth-playbook")
        .await
        .unwrap();
    let async_job_assets = store
        .list_async_job_assets("job-global-growth-playbook")
        .await
        .unwrap();
    let async_job_callbacks = store
        .list_async_job_callbacks("job-global-growth-playbook")
        .await
        .unwrap();
    let official_direct_job_attempts = store
        .list_async_job_attempts("job-global-official-direct-capacity-plan")
        .await
        .unwrap();
    let official_direct_job_assets = store
        .list_async_job_assets("job-global-official-direct-capacity-plan")
        .await
        .unwrap();
    let official_direct_job_callbacks = store
        .list_async_job_callbacks("job-global-official-direct-capacity-plan")
        .await
        .unwrap();
    let local_edge_job_attempts = store
        .list_async_job_attempts("job-global-edge-local-capacity-audit")
        .await
        .unwrap();
    let local_edge_job_assets = store
        .list_async_job_assets("job-global-edge-local-capacity-audit")
        .await
        .unwrap();
    let local_edge_job_callbacks = store
        .list_async_job_callbacks("job-global-edge-local-capacity-audit")
        .await
        .unwrap();
    let enterprise_contract_job_attempts = store
        .list_async_job_attempts("job-global-enterprise-sow-draft")
        .await
        .unwrap();
    let enterprise_contract_job_assets = store
        .list_async_job_assets("job-global-enterprise-sow-draft")
        .await
        .unwrap();
    let enterprise_contract_job_callbacks = store
        .list_async_job_callbacks("job-global-enterprise-sow-draft")
        .await
        .unwrap();
    let pricing_plans = AccountKernelStore::list_pricing_plan_records(&store)
        .await
        .unwrap();
    let pricing_rates = AccountKernelStore::list_pricing_rate_records(&store)
        .await
        .unwrap();
    let coupon_templates = store.list_coupon_template_records().await.unwrap();
    let coupon_codes = store.list_coupon_code_records().await.unwrap();
    let marketing_campaigns = store.list_marketing_campaign_records().await.unwrap();
    let account_records = AccountKernelStore::list_account_records(&store)
        .await
        .unwrap();
    let account_benefit_lots = AccountKernelStore::list_account_benefit_lots(&store)
        .await
        .unwrap();
    let account_holds = AccountKernelStore::list_account_holds(&store)
        .await
        .unwrap();
    let account_hold_allocations = store.list_account_hold_allocations().await.unwrap();
    let account_ledger_entries = store.list_account_ledger_entry_records().await.unwrap();
    let account_ledger_allocations = store.list_account_ledger_allocations().await.unwrap();
    let request_meter_facts = store.list_request_meter_facts().await.unwrap();
    let request_meter_metrics = store.list_request_meter_metrics().await.unwrap();
    let request_settlements = AccountKernelStore::list_request_settlement_records(&store)
        .await
        .unwrap();
    let global_balance = CommercialBillingReadKernel::summarize_account_balance(
        &store,
        7001001,
        1710000500000,
    )
        .await
        .unwrap();
    let global_ledger_history = CommercialBillingReadKernel::list_account_ledger_history(
        &store,
        7001001,
    )
        .await
        .unwrap();
    let global_reconciliation_state = store
        .find_account_commerce_reconciliation_state(7001001, "project_global_default")
        .await
        .unwrap();

    assert!(providers
        .iter()
        .any(|provider| provider.id == "provider-openrouter-main"));
    assert!(providers
        .iter()
        .any(|provider| provider.id == "provider-openai-official"));
    assert!(channels.iter().any(|channel| channel.id == "ernie"));
    assert!(channels.iter().any(|channel| channel.id == "minimax"));
    assert!(providers
        .iter()
        .any(|provider| provider.id == "provider-siliconflow-main"));
    assert!(providers
        .iter()
        .any(|provider| provider.id == "provider-ernie-official"));
    assert!(providers
        .iter()
        .any(|provider| provider.id == "provider-minimax-official"));
    assert!(official_provider_configs
        .iter()
        .any(|record| record.provider_id == "provider-ernie-official"));
    assert!(official_provider_configs
        .iter()
        .any(|record| record.provider_id == "provider-minimax-official"));
    assert!(payment_methods
        .iter()
        .any(|record| record.payment_method_id == "payment-stripe-hosted"));
    assert!(extension_instances
        .iter()
        .any(|record| record.instance_id == "provider-openrouter-main"));
    assert!(extension_instances.iter().any(|record| {
        record.instance_id == "provider-anthropic-official"
            && record.installation_id == "installation-anthropic-official-builtin"
    }));
    assert!(extension_instances.iter().any(|record| {
        record.instance_id == "provider-gemini-official"
            && record.installation_id == "installation-gemini-official-builtin"
    }));
    assert!(extension_instances
        .iter()
        .any(|record| record.instance_id == "provider-siliconflow-main"));
    assert!(extension_instances.iter().any(|record| {
        record.instance_id == "provider-ernie-official"
            && record.installation_id == "installation-ernie-official-builtin"
    }));
    assert!(extension_instances.iter().any(|record| {
        record.instance_id == "provider-minimax-official"
            && record.installation_id == "installation-minimax-official-builtin"
    }));
    assert!(compiled_routing_snapshots
        .iter()
        .any(|snapshot| snapshot.snapshot_id == "snapshot-prod-global-balanced"));
    assert!(compiled_routing_snapshots
        .iter()
        .any(|snapshot| snapshot.snapshot_id == "snapshot-prod-openai-official"));
    assert!(compiled_routing_snapshots
        .iter()
        .any(|snapshot| snapshot.snapshot_id == "snapshot-prod-claude-official"));
    assert!(compiled_routing_snapshots
        .iter()
        .any(|snapshot| snapshot.snapshot_id == "snapshot-prod-deepseek-official"));
    assert!(compiled_routing_snapshots
        .iter()
        .any(|snapshot| snapshot.snapshot_id == "snapshot-prod-qwen-official"));
    assert!(compiled_routing_snapshots
        .iter()
        .any(|snapshot| snapshot.snapshot_id == "snapshot-prod-doubao-official"));
    assert!(routing_profiles
        .iter()
        .any(|profile| profile.profile_id == "profile-global-openai-official"));
    assert!(routing_profiles
        .iter()
        .any(|profile| profile.profile_id == "profile-global-claude-official"));
    assert!(routing_profiles
        .iter()
        .any(|profile| profile.profile_id == "profile-global-gemini-official"));
    assert!(routing_profiles
        .iter()
        .any(|profile| profile.profile_id == "profile-global-ernie-official"));
    assert!(routing_profiles
        .iter()
        .any(|profile| profile.profile_id == "profile-global-minimax-official"));
    assert!(routing_profiles
        .iter()
        .any(|profile| profile.profile_id == "profile-global-deepseek-official"));
    assert!(routing_profiles
        .iter()
        .any(|profile| profile.profile_id == "profile-global-qwen-official"));
    assert!(routing_profiles
        .iter()
        .any(|profile| profile.profile_id == "profile-global-doubao-official"));
    assert!(routing_profiles
        .iter()
        .any(|profile| profile.profile_id == "profile-global-xai-official"));
    assert!(routing_profiles
        .iter()
        .any(|profile| profile.profile_id == "profile-global-moonshot-official"));
    assert!(routing_profiles
        .iter()
        .any(|profile| profile.profile_id == "profile-global-zhipu-official"));
    assert!(routing_profiles
        .iter()
        .any(|profile| profile.profile_id == "profile-global-hunyuan-official"));
    assert!(routing_profiles
        .iter()
        .any(|profile| profile.profile_id == "profile-global-mistral-official"));
    assert!(routing_profiles
        .iter()
        .any(|profile| profile.profile_id == "profile-global-cohere-official"));
    assert!(routing_profiles
        .iter()
        .any(|profile| profile.profile_id == "profile-global-openrouter-main"));
    assert!(routing_profiles
        .iter()
        .any(|profile| profile.profile_id == "profile-global-siliconflow-main"));
    assert!(routing_profiles
        .iter()
        .any(|profile| profile.profile_id == "profile-global-ollama-local"));
    assert!(api_key_groups
        .iter()
        .any(|group| group.group_id == "group-official-deepseek-live"));
    assert!(api_key_groups
        .iter()
        .any(|group| group.group_id == "group-official-qwen-live"));
    assert!(api_key_groups
        .iter()
        .any(|group| group.group_id == "group-official-doubao-live"));
    assert!(api_key_groups
        .iter()
        .any(|group| group.group_id == "group-official-xai-live"));
    assert!(api_key_groups
        .iter()
        .any(|group| group.group_id == "group-official-moonshot-live"));
    assert!(api_key_groups
        .iter()
        .any(|group| group.group_id == "group-official-zhipu-live"));
    assert!(api_key_groups
        .iter()
        .any(|group| group.group_id == "group-official-hunyuan-live"));
    assert!(api_key_groups
        .iter()
        .any(|group| group.group_id == "group-official-mistral-live"));
    assert!(api_key_groups
        .iter()
        .any(|group| group.group_id == "group-official-cohere-live"));
    assert!(api_key_groups
        .iter()
        .any(|group| group.group_id == "group-official-openai-live"));
    assert!(api_key_groups
        .iter()
        .any(|group| group.group_id == "group-official-claude-live"));
    assert!(api_key_groups
        .iter()
        .any(|group| group.group_id == "group-official-gemini-live"));
    assert!(api_key_groups
        .iter()
        .any(|group| group.group_id == "group-proxy-openrouter-live"));
    assert!(api_key_groups
        .iter()
        .any(|group| group.group_id == "group-proxy-siliconflow-live"));
    assert!(api_key_groups
        .iter()
        .any(|group| group.group_id == "group-local-ollama-edge"));
    assert!(rate_limit_policies
        .iter()
        .any(|policy| policy.policy_id == "rate-limit-openrouter-main"));
    assert!(rate_limit_policies
        .iter()
        .any(|policy| policy.policy_id == "rate-limit-siliconflow-main"));
    assert!(rate_limit_policies
        .iter()
        .any(|policy| policy.policy_id == "rate-limit-ollama-local"));
    assert!(routing_decision_logs
        .iter()
        .any(|decision| decision.decision_id == "decision-prod-admin-sim-balanced"));
    assert!(routing_decision_logs
        .iter()
        .any(|decision| decision.decision_id == "decision-prod-openai-official"));
    assert!(routing_decision_logs
        .iter()
        .any(|decision| decision.decision_id == "decision-prod-claude-official"));
    assert!(routing_decision_logs
        .iter()
        .any(|decision| decision.decision_id == "decision-prod-deepseek-official"));
    assert!(routing_decision_logs
        .iter()
        .any(|decision| decision.decision_id == "decision-prod-qwen-official"));
    assert!(routing_decision_logs
        .iter()
        .any(|decision| decision.decision_id == "decision-prod-doubao-official"));
    assert!(routing_decision_logs
        .iter()
        .any(|decision| decision.decision_id == "decision-prod-asia-official"));
    assert!(compiled_routing_snapshots
        .iter()
        .any(|snapshot| snapshot.snapshot_id == "snapshot-prod-openrouter-main"));
    assert!(compiled_routing_snapshots
        .iter()
        .any(|snapshot| snapshot.snapshot_id == "snapshot-prod-siliconflow-main"));
    assert!(compiled_routing_snapshots
        .iter()
        .any(|snapshot| snapshot.snapshot_id == "snapshot-prod-ollama-local"));
    assert!(routing_decision_logs
        .iter()
        .any(|decision| decision.decision_id == "decision-prod-openrouter-main"));
    assert!(routing_decision_logs
        .iter()
        .any(|decision| decision.decision_id == "decision-prod-siliconflow-main"));
    assert!(routing_decision_logs
        .iter()
        .any(|decision| decision.decision_id == "decision-prod-ollama-local"));
    assert!(provider_health_snapshots.iter().any(|snapshot| {
        snapshot.provider_id == "provider-openai-official"
            && snapshot.instance_id.as_deref() == Some("provider-openai-official")
    }));
    assert!(provider_health_snapshots.iter().any(|snapshot| {
        snapshot.provider_id == "provider-anthropic-official"
            && snapshot.instance_id.as_deref() == Some("provider-anthropic-official")
    }));
    assert!(provider_health_snapshots.iter().any(|snapshot| {
        snapshot.provider_id == "provider-gemini-official"
            && snapshot.instance_id.as_deref() == Some("provider-gemini-official")
    }));
    assert!(provider_health_snapshots.iter().any(|snapshot| {
        snapshot.provider_id == "provider-ernie-official"
            && snapshot.instance_id.as_deref() == Some("provider-ernie-official")
    }));
    assert!(provider_health_snapshots.iter().any(|snapshot| {
        snapshot.provider_id == "provider-minimax-official"
            && snapshot.instance_id.as_deref() == Some("provider-minimax-official")
    }));
    assert!(service_runtime_nodes.iter().any(|node| {
        node.node_id == "node-global-gateway-sg-01"
            && node.service_kind == "gateway"
            && node.last_seen_at_ms >= node.started_at_ms
    }));
    assert!(service_runtime_nodes.iter().any(|node| {
        node.node_id == "node-global-admin-us-01"
            && node.service_kind == "admin"
            && node.last_seen_at_ms >= node.started_at_ms
    }));
    assert!(service_runtime_nodes.iter().any(|node| {
        node.node_id == "node-global-portal-eu-01"
            && node.service_kind == "portal"
            && node.last_seen_at_ms >= node.started_at_ms
    }));
    assert!(extension_runtime_rollouts.iter().any(|rollout| {
        rollout.rollout_id == "rollout-global-openrouter-instance-refresh"
            && rollout.scope == "instance"
            && rollout.requested_instance_id.as_deref() == Some("provider-openrouter-main")
            && rollout.resolved_extension_id.as_deref() == Some("sdkwork.provider.openrouter")
    }));
    assert!(extension_runtime_rollout_participants
        .iter()
        .any(|participant| {
            participant.rollout_id == "rollout-global-openrouter-instance-refresh"
                && participant.node_id == "node-global-gateway-sg-01"
                && participant.service_kind == "gateway"
                && participant.status == "succeeded"
        }));
    assert!(extension_runtime_rollout_participants
        .iter()
        .any(|participant| {
            participant.rollout_id == "rollout-global-openrouter-instance-refresh"
                && participant.node_id == "node-global-admin-us-01"
                && participant.service_kind == "admin"
                && participant.status == "pending"
        }));
    assert!(standalone_config_rollouts.iter().any(|rollout| {
        rollout.rollout_id == "rollout-global-gateway-config-reload"
            && rollout.requested_service_kind.as_deref() == Some("gateway")
    }));
    assert!(standalone_config_rollout_participants
        .iter()
        .any(|participant| {
            participant.rollout_id == "rollout-global-gateway-config-reload"
                && participant.node_id == "node-global-gateway-sg-01"
                && participant.service_kind == "gateway"
                && participant.status == "pending"
        }));
    assert!(commerce_orders
        .iter()
        .any(|order| order.order_id == "order-global-growth-bootstrap"));
    assert!(commerce_orders
        .iter()
        .any(|order| order.order_id == "order-provider-mix-openrouter-2026"));
    assert!(commerce_orders
        .iter()
        .any(|order| order.order_id == "order-china-direct-qwen-2026"));
    assert!(commerce_orders.iter().any(|order| {
        order.order_id == "order-global-official-openai-2026"
            && order.pricing_plan_id.as_deref() == Some("global-official-direct-retail")
    }));
    assert!(commerce_orders.iter().any(|order| {
        order.order_id == "order-edge-local-ollama-2026"
            && order.pricing_plan_id.as_deref() == Some("edge-local-commercial")
            && order.payment_method_id.as_deref() == Some("payment-bank-transfer-manual")
    }));
    assert!(commerce_orders.iter().any(|order| {
        order.order_id == "order-global-enterprise-contract-2026"
            && order.pricing_plan_id.as_deref() == Some("global-enterprise-commercial")
            && order.status == "pending_payment"
            && order.payment_method_id.as_deref() == Some("payment-bank-transfer-manual")
    }));
    assert!(commerce_payment_attempts
        .iter()
        .any(|attempt| attempt.payment_attempt_id == "attempt-global-growth-bootstrap"));
    assert!(commerce_payment_attempts
        .iter()
        .any(|attempt| attempt.payment_attempt_id == "attempt-provider-mix-openrouter-2026"));
    assert!(commerce_payment_attempts
        .iter()
        .any(|attempt| attempt.payment_attempt_id == "attempt-china-direct-qwen-2026"));
    assert!(commerce_payment_attempts.iter().any(|attempt| {
        attempt.payment_attempt_id == "attempt-global-official-openai-2026"
            && attempt.payment_method_id == "payment-stripe-hosted"
    }));
    assert!(commerce_payment_attempts.iter().any(|attempt| {
        attempt.payment_attempt_id == "attempt-edge-local-ollama-2026"
            && attempt.payment_method_id == "payment-bank-transfer-manual"
            && attempt.provider == "bank_transfer"
    }));
    assert!(commerce_payment_attempts.iter().any(|attempt| {
        attempt.payment_attempt_id == "attempt-global-enterprise-contract-2026"
            && attempt.payment_method_id == "payment-bank-transfer-manual"
            && attempt.status == "pending_review"
    }));
    assert!(commerce_payment_events
        .iter()
        .any(|event| event.payment_event_id == "payment-event-global-growth-bootstrap"));
    assert!(commerce_payment_events
        .iter()
        .any(|event| event.payment_event_id == "payment-event-provider-mix-openrouter-2026"));
    assert!(commerce_payment_events
        .iter()
        .any(|event| event.payment_event_id == "payment-event-china-direct-qwen-2026"));
    assert!(commerce_payment_events
        .iter()
        .any(|event| event.payment_event_id == "payment-event-global-official-openai-2026"));
    assert!(commerce_payment_events
        .iter()
        .any(|event| event.payment_event_id == "payment-event-edge-local-ollama-2026"));
    assert!(commerce_payment_events.iter().any(|event| {
        event.payment_event_id == "payment-event-global-enterprise-contract-2026"
            && event.processing_status.as_str() == "received"
    }));
    assert!(commerce_webhook_inbox
        .iter()
        .any(|record| record.webhook_inbox_id == "webhook-inbox-global-growth-bootstrap"));
    assert!(commerce_webhook_inbox
        .iter()
        .any(|record| record.webhook_inbox_id == "webhook-inbox-provider-mix-openrouter-2026"));
    assert!(commerce_webhook_inbox
        .iter()
        .any(|record| record.webhook_inbox_id == "webhook-inbox-china-direct-qwen-2026"));
    assert!(commerce_webhook_inbox
        .iter()
        .any(|record| record.webhook_inbox_id == "webhook-inbox-global-official-openai-2026"));
    assert!(commerce_refunds
        .iter()
        .any(|refund| refund.refund_id == "refund-global-growth-bootstrap"));
    assert!(commerce_refunds
        .iter()
        .any(|refund| refund.refund_id == "refund-provider-mix-openrouter-2026"));
    assert!(commerce_refunds
        .iter()
        .any(|refund| refund.refund_id == "refund-global-official-openai-2026"));
    assert!(commerce_reconciliation_runs
        .iter()
        .any(|run| run.reconciliation_run_id == "recon-run-global-growth-bootstrap"));
    assert!(commerce_reconciliation_runs
        .iter()
        .any(|run| run.reconciliation_run_id == "recon-run-provider-mix-openrouter-2026"));
    assert!(commerce_reconciliation_runs
        .iter()
        .any(|run| run.reconciliation_run_id == "recon-run-global-official-openai-2026"));
    assert!(commerce_reconciliation_runs
        .iter()
        .any(|run| run.reconciliation_run_id == "recon-run-edge-local-ollama-2026"));
    assert!(commerce_reconciliation_runs.iter().any(|run| {
        run.reconciliation_run_id == "recon-run-global-enterprise-contract-2026"
            && run.status == "running"
    }));
    assert!(commerce_reconciliation_items
        .iter()
        .any(|item| item.reconciliation_item_id == "recon-item-global-growth-bootstrap"));
    assert!(provider_mix_reconciliation_items
        .iter()
        .any(|item| item.reconciliation_item_id == "recon-item-provider-mix-openrouter-2026"));
    assert!(official_direct_reconciliation_items
        .iter()
        .any(|item| item.reconciliation_item_id == "recon-item-global-official-openai-2026"));
    assert!(local_edge_reconciliation_items
        .iter()
        .any(|item| item.reconciliation_item_id == "recon-item-edge-local-ollama-2026"));
    assert!(billing_events
        .iter()
        .any(|event| event.event_id == "billing-prod-global-balanced"));
    assert!(billing_events
        .iter()
        .any(|event| event.event_id == "billing-prod-openai-official"));
    assert!(billing_events
        .iter()
        .any(|event| event.event_id == "billing-prod-openai-official-direct-2026"));
    assert!(billing_events
        .iter()
        .any(|event| event.event_id == "billing-prod-claude-official"));
    assert!(billing_events
        .iter()
        .any(|event| event.event_id == "billing-prod-deepseek-official"));
    assert!(billing_events
        .iter()
        .any(|event| event.event_id == "billing-prod-qwen-official"));
    assert!(billing_events
        .iter()
        .any(|event| event.event_id == "billing-prod-doubao-official"));
    assert!(billing_events
        .iter()
        .any(|event| event.event_id == "billing-prod-asia-official"));
    assert!(billing_events
        .iter()
        .any(|event| event.event_id == "billing-prod-openrouter-main"));
    assert!(billing_events
        .iter()
        .any(|event| event.event_id == "billing-prod-siliconflow-main"));
    assert!(billing_events
        .iter()
        .any(|event| event.event_id == "billing-prod-ollama-local"));
    assert!(billing_events
        .iter()
        .any(|event| event.event_id == "billing-prod-ollama-local-edge-2026"));
    assert!(pricing_plans
        .iter()
        .any(|plan| plan.plan_code == "global-provider-mix-commercial"));
    assert!(pricing_plans
        .iter()
        .any(|plan| plan.plan_code == "china-direct-commercial"));
    assert!(pricing_plans
        .iter()
        .any(|plan| plan.plan_code == "edge-local-commercial"));
    assert!(pricing_plans
        .iter()
        .any(|plan| plan.plan_code == "global-official-direct-cost"));
    assert!(pricing_plans
        .iter()
        .any(|plan| plan.plan_code == "global-official-direct-retail"));
    assert!(pricing_plans
        .iter()
        .any(|plan| plan.plan_code == "china-official-direct-cost"));
    assert!(pricing_plans
        .iter()
        .any(|plan| plan.plan_code == "china-official-direct-retail"));
    assert!(pricing_plans
        .iter()
        .any(|plan| plan.plan_code == "global-marketplace-proxy-cost"));
    assert!(pricing_plans
        .iter()
        .any(|plan| plan.plan_code == "china-proxy-distribution-cost"));
    assert!(pricing_plans
        .iter()
        .any(|plan| plan.plan_code == "local-edge-infra-cost"));
    assert!(pricing_rates.iter().any(|rate| {
        rate.provider_code.as_deref() == Some("provider-openrouter-main")
            && rate.model_code.as_deref() == Some("gpt-4.1-mini")
    }));
    assert!(pricing_rates.iter().any(|rate| {
        rate.provider_code.as_deref() == Some("provider-siliconflow-main")
            && rate.model_code.as_deref() == Some("deepseek-v3")
    }));
    assert!(pricing_rates.iter().any(|rate| {
        rate.provider_code.as_deref() == Some("provider-ollama-local")
            && rate.model_code.as_deref() == Some("qwen2.5-14b-instruct")
    }));
    assert!(pricing_rates.iter().any(|rate| {
        rate.pricing_plan_id == 9106
            && rate.metric_code == "tokens.input"
            && rate.provider_code.is_none()
            && rate.model_code.is_none()
    }));
    assert!(pricing_rates.iter().any(|rate| {
        rate.pricing_plan_id == 9110
            && rate.metric_code == "tokens.output"
            && rate.provider_code.as_deref() == Some("provider-openrouter-main")
            && rate.model_code.is_none()
    }));
    assert!(pricing_rates.iter().any(|rate| {
        rate.pricing_plan_id == 9112
            && rate.metric_code == "tokens.input"
            && rate.provider_code.as_deref() == Some("provider-ollama-local")
            && rate.model_code.is_none()
    }));
    assert!(payment_methods
        .iter()
        .any(|method| method.payment_method_id == "payment-stripe-link"));
    assert!(payment_methods
        .iter()
        .any(|method| method.payment_method_id == "payment-alipay-mobile"));
    assert!(payment_methods
        .iter()
        .any(|method| method.payment_method_id == "payment-bank-transfer-manual"));
    assert!(coupon_templates
        .iter()
        .any(|template| template.template_key == "provider-mix-15"));
    assert!(coupon_templates
        .iter()
        .any(|template| template.template_key == "china-direct-888"));
    assert!(coupon_templates
        .iter()
        .any(|template| template.template_key == "official-direct-credit-100"));
    assert!(coupon_templates
        .iter()
        .any(|template| template.template_key == "edge-local-credit-20"));
    assert!(coupon_templates
        .iter()
        .any(|template| template.template_key == "enterprise-contract-credit-500"));
    assert!(coupon_codes
        .iter()
        .any(|code| code.code_value == "LAUNCH100"));
    assert!(marketing_campaigns
        .iter()
        .any(|campaign| campaign.marketing_campaign_id == "campaign-provider-mix-2026"));
    assert!(marketing_campaigns
        .iter()
        .any(|campaign| campaign.marketing_campaign_id == "campaign-china-direct-2026"));
    assert!(marketing_campaigns
        .iter()
        .any(|campaign| campaign.marketing_campaign_id == "campaign-official-direct-2026"));
    assert!(marketing_campaigns
        .iter()
        .any(|campaign| campaign.marketing_campaign_id == "campaign-edge-local-2026"));
    assert!(marketing_campaigns
        .iter()
        .any(|campaign| campaign.marketing_campaign_id == "campaign-enterprise-contract-2026"));
    assert!(async_jobs.iter().any(|job| {
        job.job_id == "job-global-growth-playbook"
            && job.provider_id.as_deref() == Some("provider-openai-official")
            && job.model_code.as_deref() == Some("gpt-4.1")
    }));
    assert!(async_jobs.iter().any(|job| {
        job.job_id == "job-global-openrouter-routing-brief"
            && job.provider_id.as_deref() == Some("provider-openrouter-main")
            && job.model_code.as_deref() == Some("openai/gpt-4.1-mini")
    }));
    assert!(async_jobs.iter().any(|job| {
        job.job_id == "job-global-siliconflow-translation-pack"
            && job.provider_id.as_deref() == Some("provider-siliconflow-main")
            && job.model_code.as_deref() == Some("deepseek-ai/DeepSeek-V3")
    }));
    assert!(async_jobs.iter().any(|job| {
        job.job_id == "job-global-ollama-guardrail-review"
            && job.provider_id.as_deref() == Some("provider-ollama-local")
            && job.model_code.as_deref() == Some("qwen2.5:14b")
    }));
    assert!(async_jobs.iter().any(|job| {
        job.job_id == "job-global-official-direct-capacity-plan"
            && job.provider_id.as_deref() == Some("provider-openai-official")
            && job.model_code.as_deref() == Some("gpt-4.1-mini")
    }));
    assert!(async_jobs.iter().any(|job| {
        job.job_id == "job-global-edge-local-capacity-audit"
            && job.provider_id.as_deref() == Some("provider-ollama-local")
            && job.model_code.as_deref() == Some("qwen2.5:14b")
    }));
    assert!(async_jobs.iter().any(|job| {
        job.job_id == "job-global-enterprise-sow-draft"
            && job.provider_id.as_deref() == Some("provider-anthropic-official")
            && job.model_code.as_deref() == Some("claude-3-7-sonnet")
    }));
    assert!(async_job_attempts.iter().any(|attempt| {
        attempt.attempt_id == 9801
            && attempt.job_id == "job-global-growth-playbook"
            && attempt.runtime_kind == "openai"
    }));
    assert!(official_direct_job_attempts.iter().any(|attempt| {
        attempt.job_id == "job-global-official-direct-capacity-plan"
            && attempt.runtime_kind == "openai"
    }));
    assert!(local_edge_job_attempts.iter().any(|attempt| {
        attempt.job_id == "job-global-edge-local-capacity-audit"
            && attempt.runtime_kind == "ollama"
    }));
    assert!(async_job_assets.iter().any(|asset| {
        asset.asset_id == "asset-global-growth-playbook-md"
            && asset.job_id == "job-global-growth-playbook"
    }));
    assert!(official_direct_job_assets.iter().any(|asset| {
        asset.job_id == "job-global-official-direct-capacity-plan"
    }));
    assert!(local_edge_job_assets.iter().any(|asset| {
        asset.job_id == "job-global-edge-local-capacity-audit"
    }));
    assert!(async_job_callbacks.iter().any(|callback| {
        callback.callback_id == 10801 && callback.job_id == "job-global-growth-playbook"
    }));
    assert!(official_direct_job_callbacks.iter().any(|callback| {
        callback.job_id == "job-global-official-direct-capacity-plan"
    }));
    assert!(local_edge_job_callbacks.iter().any(|callback| {
        callback.job_id == "job-global-edge-local-capacity-audit"
    }));
    assert!(enterprise_contract_job_attempts.iter().any(|attempt| {
        attempt.job_id == "job-global-enterprise-sow-draft"
            && attempt.runtime_kind == "anthropic"
    }));
    assert!(enterprise_contract_job_assets.iter().any(|asset| {
        asset.job_id == "job-global-enterprise-sow-draft"
    }));
    assert!(enterprise_contract_job_callbacks.iter().any(|callback| {
        callback.job_id == "job-global-enterprise-sow-draft"
    }));
    assert_eq!(account_records.len(), 1);
    assert!(account_records.iter().any(|account| {
        account.account_id == 7001001
            && account.currency_code == "USD"
    }));
    assert_eq!(account_benefit_lots.len(), 6);
    assert!(account_benefit_lots.iter().any(|lot| lot.lot_id == 8001003));
    assert!(account_benefit_lots.iter().any(|lot| lot.lot_id == 8001004));
    assert!(account_benefit_lots.iter().any(|lot| lot.lot_id == 8001005));
    assert!(account_benefit_lots.iter().any(|lot| lot.lot_id == 8001006));
    assert_eq!(account_holds.len(), 21);
    assert!(account_holds.iter().any(|hold| hold.hold_id == 8101002));
    assert!(account_holds.iter().any(|hold| hold.hold_id == 8101003));
    assert!(account_holds.iter().any(|hold| hold.hold_id == 8101007));
    assert!(account_holds.iter().any(|hold| hold.hold_id == 8101012));
    assert!(account_holds.iter().any(|hold| hold.hold_id == 8101013));
    assert!(account_holds.iter().any(|hold| hold.hold_id == 8101018));
    assert!(account_holds.iter().any(|hold| hold.hold_id == 8101019));
    assert!(account_holds.iter().any(|hold| hold.hold_id == 8101021));
    assert_eq!(account_hold_allocations.len(), 21);
    assert_eq!(account_ledger_entries.len(), 27);
    assert!(account_ledger_entries
        .iter()
        .any(|entry| entry.ledger_entry_id == 8201005));
    assert!(account_ledger_entries
        .iter()
        .any(|entry| entry.ledger_entry_id == 8201006));
    assert!(account_ledger_entries
        .iter()
        .any(|entry| entry.ledger_entry_id == 8201011));
    assert!(account_ledger_entries
        .iter()
        .any(|entry| entry.ledger_entry_id == 8201016));
    assert!(account_ledger_entries
        .iter()
        .any(|entry| entry.ledger_entry_id == 8201018));
    assert!(account_ledger_entries
        .iter()
        .any(|entry| entry.ledger_entry_id == 8201023));
    assert!(account_ledger_entries
        .iter()
        .any(|entry| entry.ledger_entry_id == 8201025));
    assert!(account_ledger_entries
        .iter()
        .any(|entry| entry.ledger_entry_id == 8201027));
    assert_eq!(account_ledger_allocations.len(), 27);
    assert_eq!(request_meter_facts.len(), 21);
    assert!(request_meter_facts.iter().any(|fact| fact.request_id == 610002));
    assert!(request_meter_facts.iter().any(|fact| fact.request_id == 610003));
    assert!(request_meter_facts.iter().any(|fact| fact.request_id == 610007));
    assert!(request_meter_facts.iter().any(|fact| fact.request_id == 610012));
    assert!(request_meter_facts.iter().any(|fact| fact.request_id == 610013));
    assert!(request_meter_facts.iter().any(|fact| fact.request_id == 610018));
    assert!(request_meter_facts.iter().any(|fact| fact.request_id == 610019));
    assert!(request_meter_facts.iter().any(|fact| fact.request_id == 610021));
    assert!(request_meter_facts.iter().any(|fact| {
        fact.request_id == 610002
            && fact.cost_pricing_plan_id == Some(9108)
            && fact.retail_pricing_plan_id == Some(9109)
    }));
    assert!(request_meter_facts.iter().any(|fact| {
        fact.request_id == 610013
            && fact.cost_pricing_plan_id == Some(9106)
            && fact.retail_pricing_plan_id == Some(9107)
    }));
    assert!(request_meter_facts.iter().any(|fact| {
        fact.request_id == 610016
            && fact.cost_pricing_plan_id == Some(9110)
            && fact.retail_pricing_plan_id == Some(9103)
    }));
    assert!(request_meter_facts.iter().any(|fact| {
        fact.request_id == 610017
            && fact.cost_pricing_plan_id == Some(9111)
            && fact.retail_pricing_plan_id == Some(9104)
    }));
    assert!(request_meter_facts.iter().any(|fact| {
        fact.request_id == 610018
            && fact.cost_pricing_plan_id == Some(9112)
            && fact.retail_pricing_plan_id == Some(9105)
    }));
    assert!(request_meter_facts.iter().any(|fact| {
        fact.request_id == 610019
            && fact.cost_pricing_plan_id == Some(9110)
            && fact.retail_pricing_plan_id == Some(9103)
    }));
    assert_eq!(request_meter_metrics.len(), 42);
    assert_eq!(request_settlements.len(), 21);
    assert!(request_settlements
        .iter()
        .any(|settlement| settlement.request_settlement_id == 8301002));
    assert!(request_settlements
        .iter()
        .any(|settlement| settlement.request_settlement_id == 8301003));
    assert!(request_settlements
        .iter()
        .any(|settlement| settlement.request_settlement_id == 8301004));
    assert!(request_settlements
        .iter()
        .any(|settlement| settlement.request_settlement_id == 8301012));
    assert!(request_settlements
        .iter()
        .any(|settlement| settlement.request_settlement_id == 8301013));
    assert!(request_settlements
        .iter()
        .any(|settlement| settlement.request_settlement_id == 8301018));
    assert!(request_settlements
        .iter()
        .any(|settlement| settlement.request_settlement_id == 8301019));
    assert!(request_settlements
        .iter()
        .any(|settlement| settlement.request_settlement_id == 8301021));
    assert_eq!(global_balance.account_id, 7001001);
    assert_eq!(global_balance.active_lot_count, 6);
    assert!((global_balance.available_balance - 22_709_600.0).abs() < f64::EPSILON);
    assert!((global_balance.consumed_balance - 290_400.0).abs() < f64::EPSILON);
    assert_eq!(global_ledger_history.len(), 27);
    assert_eq!(
        global_reconciliation_state
            .expect("global reconciliation state")
            .last_order_id,
        "order-global-enterprise-contract-2026"
    );
    assert!(admin_users.is_empty());
    assert!(portal_users.is_empty());
    assert!(gateway_api_keys.is_empty());
}

#[tokio::test]
async fn product_runtime_bootstraps_repository_dev_identity_seed_data() {
    let config_root = temp_root("product-runtime-bootstrap-dev-identities");
    let database_path = config_root.join("product-runtime-bootstrap-dev-identities.db");
    let database_url = sqlite_url_for(database_path.clone());
    let (loader, mut config) = StandaloneConfigLoader::from_local_root_and_pairs(
        &config_root,
        std::iter::empty::<(&str, &str)>(),
    )
    .unwrap();
    config.database_url = database_url;
    config.bootstrap_profile = "dev".to_owned();

    let runtime = RouterProductRuntime::start(
        loader,
        config,
        RouterProductRuntimeOptions::desktop(ProductSiteDirs::new(
            config_root.join("unused-admin-site"),
            config_root.join("unused-portal-site"),
        ))
        .with_roles(Vec::<ProductRuntimeRole>::new()),
    )
    .await
    .unwrap();
    drop(runtime);

    let pool = run_migrations(&sqlite_url_for(database_path))
        .await
        .unwrap();
    let store = SqliteAdminStore::new(pool);
    let tenants = store.list_tenants().await.unwrap();
    let projects = store.list_projects().await.unwrap();
    let admin_users = store.list_admin_users().await.unwrap();
    let portal_users = store.list_portal_users().await.unwrap();
    let gateway_api_keys = store.list_gateway_api_keys().await.unwrap();
    let api_key_groups = store.list_api_key_groups().await.unwrap();
    let quota_policies = store.list_quota_policies().await.unwrap();
    let routing_profiles = store.list_routing_profiles().await.unwrap();
    let compiled_routing_snapshots = store.list_compiled_routing_snapshots().await.unwrap();
    let routing_decision_logs = store.list_routing_decision_logs().await.unwrap();
    let provider_health_snapshots = store.list_provider_health_snapshots().await.unwrap();
    let service_runtime_nodes = store.list_service_runtime_nodes().await.unwrap();
    let extension_runtime_rollouts = store.list_extension_runtime_rollouts().await.unwrap();
    let extension_runtime_rollout_participants = store
        .list_extension_runtime_rollout_participants("rollout-global-openrouter-instance-refresh")
        .await
        .unwrap();
    let standalone_config_rollouts = store.list_standalone_config_rollouts().await.unwrap();
    let standalone_config_rollout_participants = store
        .list_standalone_config_rollout_participants("rollout-global-gateway-config-reload")
        .await
        .unwrap();
    let project_growth_lab_membership = store
        .find_project_membership("project_growth_lab")
        .await
        .unwrap()
        .expect("growth_lab project membership");
    let coupon_templates = store.list_coupon_template_records().await.unwrap();
    let marketing_campaigns = store.list_marketing_campaign_records().await.unwrap();
    let campaign_budgets = store.list_campaign_budget_records().await.unwrap();
    let coupon_codes = store.list_coupon_code_records().await.unwrap();
    let commerce_orders = store.list_commerce_orders().await.unwrap();
    let commerce_payment_attempts = store.list_commerce_payment_attempts().await.unwrap();
    let commerce_payment_events = store.list_commerce_payment_events().await.unwrap();
    let commerce_webhook_inbox = store.list_commerce_webhook_inbox_records().await.unwrap();
    let commerce_refunds = store.list_commerce_refunds().await.unwrap();
    let commerce_reconciliation_runs = store.list_commerce_reconciliation_runs().await.unwrap();
    let commerce_reconciliation_items = store
        .list_commerce_reconciliation_items("recon-run-local-sandbox-dev")
        .await
        .unwrap();
    let billing_events = store.list_billing_events().await.unwrap();
    let async_jobs = store.list_async_jobs().await.unwrap();
    let local_async_job_attempts = store
        .list_async_job_attempts("job-local-sandbox-eval")
        .await
        .unwrap();
    let growth_lab_async_job_attempts = store
        .list_async_job_attempts("job-growth-lab-minimax-brief")
        .await
        .unwrap();
    let growth_lab_async_job_assets = store
        .list_async_job_assets("job-growth-lab-minimax-brief")
        .await
        .unwrap();
    let growth_lab_async_job_callbacks = store
        .list_async_job_callbacks("job-growth-lab-minimax-brief")
        .await
        .unwrap();
    let partner_async_job_callbacks = store
        .list_async_job_callbacks("job-partner-gemini-brief")
        .await
        .unwrap();
    let account_records = AccountKernelStore::list_account_records(&store)
        .await
        .unwrap();
    let account_benefit_lots = AccountKernelStore::list_account_benefit_lots(&store)
        .await
        .unwrap();
    let account_holds = AccountKernelStore::list_account_holds(&store)
        .await
        .unwrap();
    let account_hold_allocations = store.list_account_hold_allocations().await.unwrap();
    let account_ledger_entries = store.list_account_ledger_entry_records().await.unwrap();
    let account_ledger_allocations = store.list_account_ledger_allocations().await.unwrap();
    let request_meter_facts = store.list_request_meter_facts().await.unwrap();
    let request_meter_metrics = store.list_request_meter_metrics().await.unwrap();
    let request_settlements = AccountKernelStore::list_request_settlement_records(&store)
        .await
        .unwrap();
    let local_demo_balance = CommercialBillingReadKernel::summarize_account_balance(
        &store,
        7002001,
        1710001700000,
    )
        .await
        .unwrap();
    let growth_lab_balance = CommercialBillingReadKernel::summarize_account_balance(
        &store,
        7002003,
        1710001700000,
    )
        .await
        .unwrap();
    let partner_ledger_history = CommercialBillingReadKernel::list_account_ledger_history(
        &store,
        7002002,
    )
        .await
        .unwrap();
    let local_demo_reconciliation_state = store
        .find_account_commerce_reconciliation_state(7002001, "project_local_demo")
        .await
        .unwrap();
    let partner_reconciliation_state = store
        .find_account_commerce_reconciliation_state(7002002, "project_partner_demo")
        .await
        .unwrap();
    let growth_lab_reconciliation_state = store
        .find_account_commerce_reconciliation_state(7002003, "project_growth_lab")
        .await
        .unwrap();

    assert!(admin_users
        .iter()
        .any(|user| user.email == "admin@sdkwork.local"));
    assert!(portal_users
        .iter()
        .any(|user| user.email == "portal@sdkwork.local"));
    assert!(portal_users
        .iter()
        .any(|user| user.email == "partner@sdkwork.local"));
    assert!(admin_users
        .iter()
        .any(|user| user.email == "growth.admin@sdkwork.local"));
    assert!(portal_users
        .iter()
        .any(|user| user.email == "growthlab@sdkwork.local"));
    assert!(gateway_api_keys.iter().any(|record| {
        record.api_key_group_id.as_deref() == Some("group-local-demo-live")
            && record.environment == "live"
    }));
    assert!(gateway_api_keys.iter().any(|record| {
        record.api_key_group_id.as_deref() == Some("group-growth-lab-sandbox")
            && record.environment == "sandbox"
    }));
    assert!(api_key_groups.iter().any(|group| {
        group.group_id == "group-growth-lab-sandbox"
            && group.default_routing_profile_id.as_deref()
                == Some("profile-growth-lab-asia-official")
    }));
    assert!(quota_policies
        .iter()
        .any(|policy| policy.policy_id == "quota-growth-lab-sandbox"));
    assert!(routing_profiles
        .iter()
        .any(|profile| profile.profile_id == "profile-global-openai-official"));
    assert!(routing_profiles
        .iter()
        .any(|profile| profile.profile_id == "profile-global-claude-official"));
    assert!(routing_profiles
        .iter()
        .any(|profile| profile.profile_id == "profile-global-gemini-official"));
    assert!(compiled_routing_snapshots
        .iter()
        .any(|snapshot| snapshot.snapshot_id == "snapshot-dev-local-sandbox"));
    assert!(compiled_routing_snapshots
        .iter()
        .any(|snapshot| snapshot.snapshot_id == "snapshot-dev-growth-lab"));
    assert!(routing_decision_logs
        .iter()
        .any(|decision| decision.decision_id == "decision-dev-local-sandbox"));
    assert!(routing_decision_logs
        .iter()
        .any(|decision| decision.decision_id == "decision-dev-growth-lab"));
    assert_eq!(project_growth_lab_membership.plan_id, "plan-growth-lab-apac");
    assert_eq!(project_growth_lab_membership.project_id, "project_growth_lab");
    assert!(provider_health_snapshots.iter().any(|snapshot| {
        snapshot.provider_id == "provider-ollama-local"
            && snapshot.instance_id.as_deref() == Some("provider-ollama-local")
    }));
    assert!(provider_health_snapshots.iter().any(|snapshot| {
        snapshot.provider_id == "provider-anthropic-official"
            && snapshot.instance_id.as_deref() == Some("provider-anthropic-official")
    }));
    assert!(provider_health_snapshots.iter().any(|snapshot| {
        snapshot.provider_id == "provider-gemini-official"
            && snapshot.instance_id.as_deref() == Some("provider-gemini-official")
    }));
    assert!(provider_health_snapshots.iter().any(|snapshot| {
        snapshot.provider_id == "provider-minimax-official"
            && snapshot.instance_id.as_deref() == Some("provider-minimax-official")
            && snapshot
                .message
                .as_deref()
                .is_some_and(|message| message.contains("growth lab"))
    }));
    assert!(service_runtime_nodes.iter().any(|node| {
        node.node_id == "node-global-gateway-sg-01"
            && node.service_kind == "gateway"
            && node.last_seen_at_ms >= node.started_at_ms
    }));
    assert!(service_runtime_nodes.iter().any(|node| {
        node.node_id == "node-global-admin-us-01"
            && node.service_kind == "admin"
            && node.last_seen_at_ms >= node.started_at_ms
    }));
    assert!(extension_runtime_rollouts.iter().any(|rollout| {
        rollout.rollout_id == "rollout-global-openrouter-instance-refresh"
            && rollout.scope == "instance"
            && rollout.requested_instance_id.as_deref() == Some("provider-openrouter-main")
    }));
    assert!(extension_runtime_rollout_participants
        .iter()
        .any(|participant| {
            participant.rollout_id == "rollout-global-openrouter-instance-refresh"
                && participant.node_id == "node-global-gateway-sg-01"
                && participant.service_kind == "gateway"
        }));
    assert!(standalone_config_rollouts.iter().any(|rollout| {
        rollout.rollout_id == "rollout-global-gateway-config-reload"
            && rollout.requested_service_kind.as_deref() == Some("gateway")
    }));
    assert!(standalone_config_rollout_participants
        .iter()
        .any(|participant| {
            participant.rollout_id == "rollout-global-gateway-config-reload"
                && participant.node_id == "node-global-gateway-sg-01"
                && participant.service_kind == "gateway"
        }));
    assert!(coupon_templates
        .iter()
        .any(|template| template.template_key == "growth-lab-apac-credit-30"));
    assert!(marketing_campaigns
        .iter()
        .any(|campaign| campaign.marketing_campaign_id == "campaign-growth-lab-apac-launch"));
    assert!(campaign_budgets
        .iter()
        .any(|budget| budget.campaign_budget_id == "budget-growth-lab-apac-launch"));
    assert!(coupon_codes.iter().any(|code| code.code_value == "LAUNCH100"));
    assert!(coupon_codes.iter().any(|code| code.code_value == "DEV50"));
    assert!(coupon_codes
        .iter()
        .any(|code| code.code_value == "PARTNER30"));
    assert!(coupon_codes
        .iter()
        .any(|code| code.code_value == "GLABAPAC3000"));
    assert!(commerce_orders
        .iter()
        .any(|order| order.order_id == "order-local-demo-sandbox-pack"));
    assert!(commerce_orders
        .iter()
        .any(|order| order.order_id == "order-partner-demo-growth"));
    assert!(commerce_orders
        .iter()
        .any(|order| order.order_id == "order-growth-lab-asia-launch"));
    assert!(projects
        .iter()
        .any(|project| project.id == "project_growth_lab"));
    assert!(tenants
        .iter()
        .any(|tenant| tenant.id == "tenant_growth_lab"));
    assert!(commerce_payment_attempts
        .iter()
        .any(|attempt| attempt.payment_attempt_id == "attempt-local-demo-sandbox"));
    assert!(commerce_payment_attempts
        .iter()
        .any(|attempt| attempt.payment_attempt_id == "attempt-partner-demo-growth"));
    assert!(commerce_payment_attempts
        .iter()
        .any(|attempt| attempt.payment_attempt_id == "attempt-growth-lab-asia-launch"));
    assert!(commerce_payment_events
        .iter()
        .any(|event| event.payment_event_id == "payment-event-local-demo-sandbox"));
    assert!(commerce_payment_events
        .iter()
        .any(|event| event.payment_event_id == "payment-event-growth-lab-asia-launch"));
    assert!(commerce_webhook_inbox
        .iter()
        .any(|record| record.webhook_inbox_id == "webhook-inbox-local-demo-sandbox"));
    assert!(commerce_webhook_inbox
        .iter()
        .any(|record| record.webhook_inbox_id == "webhook-inbox-growth-lab-asia-launch"));
    assert!(commerce_refunds
        .iter()
        .any(|refund| refund.refund_id == "refund-local-demo-sandbox"));
    assert!(commerce_refunds
        .iter()
        .any(|refund| refund.refund_id == "refund-growth-lab-asia-launch"));
    assert!(commerce_reconciliation_runs
        .iter()
        .any(|run| run.reconciliation_run_id == "recon-run-local-sandbox-dev"));
    assert!(commerce_reconciliation_runs
        .iter()
        .any(|run| run.reconciliation_run_id == "recon-run-growth-lab-apac"));
    assert!(commerce_reconciliation_items
        .iter()
        .any(|item| item.reconciliation_item_id == "recon-item-local-sandbox-dev"));
    assert!(billing_events
        .iter()
        .any(|event| event.event_id == "billing-dev-local-sandbox"));
    assert!(billing_events
        .iter()
        .any(|event| event.event_id == "billing-dev-partner-staging"));
    assert!(billing_events
        .iter()
        .any(|event| event.event_id == "billing-dev-growth-lab-asia"));
    assert!(async_jobs.iter().any(|job| {
        job.job_id == "job-local-sandbox-eval"
            && job.provider_id.as_deref() == Some("provider-ollama-local")
            && job.model_code.as_deref() == Some("llama3.2:latest")
    }));
    assert!(async_jobs.iter().any(|job| {
        job.job_id == "job-partner-gemini-brief"
            && job.provider_id.as_deref() == Some("provider-gemini-official")
            && job.model_code.as_deref() == Some("gemini-2.5-pro")
    }));
    assert!(async_jobs.iter().any(|job| {
        job.job_id == "job-growth-lab-minimax-brief"
            && job.provider_id.as_deref() == Some("provider-minimax-official")
            && job.model_code.as_deref() == Some("minimax-m1")
    }));
    assert!(local_async_job_attempts.iter().any(|attempt| {
        attempt.attempt_id == 9802
            && attempt.job_id == "job-local-sandbox-eval"
            && attempt.runtime_kind == "ollama"
    }));
    assert!(growth_lab_async_job_attempts.iter().any(|attempt| {
        attempt.attempt_id == 9804
            && attempt.job_id == "job-growth-lab-minimax-brief"
            && attempt.runtime_kind == "minimax"
    }));
    assert!(growth_lab_async_job_assets.iter().any(|asset| {
        asset.asset_id == "asset-growth-lab-minimax-brief-json"
            && asset.job_id == "job-growth-lab-minimax-brief"
    }));
    assert!(growth_lab_async_job_callbacks.iter().any(|callback| {
        callback.callback_id == 10806 && callback.job_id == "job-growth-lab-minimax-brief"
    }));
    assert!(partner_async_job_callbacks.iter().any(|callback| {
        callback.callback_id == 10803 && callback.job_id == "job-partner-gemini-brief"
    }));
    assert_eq!(account_records.len(), 4);
    assert!(account_records.iter().any(|account| account.account_id == 7001001));
    assert!(account_records.iter().any(|account| {
        account.account_id == 7002001
            && account.currency_code == "USD"
    }));
    assert!(account_records.iter().any(|account| account.account_id == 7002002));
    assert!(account_records.iter().any(|account| account.account_id == 7002003));
    assert_eq!(account_benefit_lots.len(), 11);
    assert!(account_benefit_lots.iter().any(|lot| lot.lot_id == 8001003));
    assert!(account_benefit_lots.iter().any(|lot| lot.lot_id == 8001004));
    assert!(account_benefit_lots.iter().any(|lot| lot.lot_id == 8001005));
    assert!(account_benefit_lots.iter().any(|lot| lot.lot_id == 8001006));
    assert_eq!(account_holds.len(), 24);
    assert!(account_holds.iter().any(|hold| hold.hold_id == 8101002));
    assert!(account_holds.iter().any(|hold| hold.hold_id == 8101003));
    assert!(account_holds.iter().any(|hold| hold.hold_id == 8101007));
    assert!(account_holds.iter().any(|hold| hold.hold_id == 8101012));
    assert!(account_holds.iter().any(|hold| hold.hold_id == 8101013));
    assert!(account_holds.iter().any(|hold| hold.hold_id == 8101018));
    assert!(account_holds.iter().any(|hold| hold.hold_id == 8101019));
    assert!(account_holds.iter().any(|hold| hold.hold_id == 8101021));
    assert_eq!(account_hold_allocations.len(), 24);
    assert_eq!(account_ledger_entries.len(), 35);
    assert!(account_ledger_entries
        .iter()
        .any(|entry| entry.ledger_entry_id == 8201005));
    assert!(account_ledger_entries
        .iter()
        .any(|entry| entry.ledger_entry_id == 8201006));
    assert!(account_ledger_entries
        .iter()
        .any(|entry| entry.ledger_entry_id == 8201011));
    assert!(account_ledger_entries
        .iter()
        .any(|entry| entry.ledger_entry_id == 8201016));
    assert!(account_ledger_entries
        .iter()
        .any(|entry| entry.ledger_entry_id == 8201018));
    assert!(account_ledger_entries
        .iter()
        .any(|entry| entry.ledger_entry_id == 8201023));
    assert!(account_ledger_entries
        .iter()
        .any(|entry| entry.ledger_entry_id == 8201025));
    assert!(account_ledger_entries
        .iter()
        .any(|entry| entry.ledger_entry_id == 8201027));
    assert_eq!(account_ledger_allocations.len(), 35);
    assert_eq!(request_meter_facts.len(), 24);
    assert!(request_meter_facts.iter().any(|fact| fact.request_id == 610002));
    assert!(request_meter_facts.iter().any(|fact| fact.request_id == 610003));
    assert!(request_meter_facts.iter().any(|fact| fact.request_id == 610007));
    assert!(request_meter_facts.iter().any(|fact| fact.request_id == 610012));
    assert!(request_meter_facts.iter().any(|fact| fact.request_id == 610013));
    assert!(request_meter_facts.iter().any(|fact| fact.request_id == 610018));
    assert!(request_meter_facts.iter().any(|fact| fact.request_id == 610019));
    assert!(request_meter_facts.iter().any(|fact| fact.request_id == 610021));
    assert!(request_meter_facts.iter().any(|fact| {
        fact.request_id == 620001
            && fact.cost_pricing_plan_id == Some(9112)
            && fact.retail_pricing_plan_id == Some(9105)
    }));
    assert!(request_meter_facts.iter().any(|fact| {
        fact.request_id == 620002
            && fact.cost_pricing_plan_id == Some(9106)
            && fact.retail_pricing_plan_id == Some(9107)
    }));
    assert!(request_meter_facts.iter().any(|fact| {
        fact.request_id == 620003
            && fact.cost_pricing_plan_id == Some(9108)
            && fact.retail_pricing_plan_id == Some(9109)
    }));
    assert_eq!(request_meter_metrics.len(), 48);
    assert_eq!(request_settlements.len(), 24);
    assert!(request_settlements
        .iter()
        .any(|settlement| settlement.request_settlement_id == 8301002));
    assert!(request_settlements
        .iter()
        .any(|settlement| settlement.request_settlement_id == 8301003));
    assert!(request_settlements
        .iter()
        .any(|settlement| settlement.request_settlement_id == 8301004));
    assert!(request_settlements
        .iter()
        .any(|settlement| settlement.request_settlement_id == 8301012));
    assert!(request_settlements
        .iter()
        .any(|settlement| settlement.request_settlement_id == 8301013));
    assert!(request_settlements
        .iter()
        .any(|settlement| settlement.request_settlement_id == 8301018));
    assert!(request_settlements
        .iter()
        .any(|settlement| settlement.request_settlement_id == 8301019));
    assert!(request_settlements
        .iter()
        .any(|settlement| settlement.request_settlement_id == 8301021));
    assert_eq!(local_demo_balance.active_lot_count, 2);
    assert!((local_demo_balance.available_balance - 3_497_320.0).abs() < f64::EPSILON);
    assert_eq!(growth_lab_balance.active_lot_count, 2);
    assert!((growth_lab_balance.available_balance - 9_197_140.0).abs() < f64::EPSILON);
    assert_eq!(partner_ledger_history.len(), 2);
    assert_eq!(
        local_demo_reconciliation_state
            .expect("local demo reconciliation state")
            .last_order_id,
        "order-local-demo-sandbox-pack"
    );
    assert_eq!(
        partner_reconciliation_state
            .expect("partner reconciliation state")
            .last_order_id,
        "order-partner-demo-growth"
    );
    assert_eq!(
        growth_lab_reconciliation_state
            .expect("growth lab reconciliation state")
            .last_order_id,
        "order-growth-lab-asia-launch"
    );
}

struct MinimalRedisPingServer {
    address: String,
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl MinimalRedisPingServer {
    fn start() -> Self {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let address = listener.local_addr().unwrap().to_string();
        let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let thread_stop = stop.clone();
        let thread = std::thread::spawn(move || {
            while !thread_stop.load(std::sync::atomic::Ordering::Relaxed) {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        stream.set_nonblocking(false).unwrap();
                        loop {
                            match read_minimal_resp_array(&mut stream) {
                                Ok(Some(command)) => match String::from_utf8_lossy(&command[0])
                                    .to_ascii_uppercase()
                                    .as_str()
                                {
                                    "PING" => {
                                        use std::io::Write;
                                        stream.write_all(b"+PONG\r\n").unwrap();
                                        stream.flush().unwrap();
                                    }
                                    "AUTH" | "SELECT" => {
                                        use std::io::Write;
                                        stream.write_all(b"+OK\r\n").unwrap();
                                        stream.flush().unwrap();
                                    }
                                    other => panic!("unexpected minimal redis command: {other}"),
                                },
                                Ok(None) => break,
                                Err(error)
                                    if matches!(
                                        error.kind(),
                                        std::io::ErrorKind::UnexpectedEof
                                            | std::io::ErrorKind::ConnectionReset
                                            | std::io::ErrorKind::TimedOut
                                    ) =>
                                {
                                    break
                                }
                                Err(error) => panic!("minimal redis server read failed: {error}"),
                            }
                        }
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(std::time::Duration::from_millis(10));
                    }
                    Err(error) => panic!("minimal redis accept failed: {error}"),
                }
            }
        });

        Self {
            address,
            stop,
            thread: Some(thread),
        }
    }

    fn url_with_db(&self, db: u32) -> String {
        format!("redis://{}/{db}", self.address)
    }
}

impl Drop for MinimalRedisPingServer {
    fn drop(&mut self) {
        self.stop.store(true, std::sync::atomic::Ordering::Relaxed);
        let _ = std::net::TcpStream::connect(&self.address);
        if let Some(thread) = self.thread.take() {
            thread.join().unwrap();
        }
    }
}

fn read_minimal_resp_array(
    stream: &mut std::net::TcpStream,
) -> std::io::Result<Option<Vec<Vec<u8>>>> {
    let mut marker = [0_u8; 1];
    match std::io::Read::read_exact(stream, &mut marker) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(error) => return Err(error),
    }
    assert_eq!(marker[0], b'*');
    let count = read_minimal_resp_line(stream)?.parse::<usize>().unwrap();
    let mut values = Vec::with_capacity(count);
    for _ in 0..count {
        let mut bulk_marker = [0_u8; 1];
        std::io::Read::read_exact(stream, &mut bulk_marker)?;
        assert_eq!(bulk_marker[0], b'$');
        let length = read_minimal_resp_line(stream)?.parse::<usize>().unwrap();
        let mut value = vec![0_u8; length];
        std::io::Read::read_exact(stream, &mut value)?;
        let mut crlf = [0_u8; 2];
        std::io::Read::read_exact(stream, &mut crlf)?;
        values.push(value);
    }
    Ok(Some(values))
}

fn read_minimal_resp_line(stream: &mut std::net::TcpStream) -> std::io::Result<String> {
    let mut bytes = Vec::new();
    loop {
        let mut byte = [0_u8; 1];
        std::io::Read::read_exact(stream, &mut byte)?;
        if byte[0] == b'\r' {
            let mut newline = [0_u8; 1];
            std::io::Read::read_exact(stream, &mut newline)?;
            assert_eq!(newline[0], b'\n');
            break;
        }
        bytes.push(byte[0]);
    }
    Ok(String::from_utf8(bytes).unwrap())
}

fn temp_root(label: &str) -> PathBuf {
    let unique = TEMP_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!("sdkwork-product-runtime-tests-{label}-{unique}"));
    if root.exists() {
        fs::remove_dir_all(&root).unwrap();
    }
    fs::create_dir_all(&root).unwrap();
    root
}

fn sqlite_url_for(path: PathBuf) -> String {
    let normalized = path.to_string_lossy().replace('\\', "/");
    if normalized.starts_with('/') {
        format!("sqlite://{normalized}")
    } else {
        format!("sqlite:///{normalized}")
    }
}

fn http_client() -> Client {
    Client::builder().build().unwrap()
}

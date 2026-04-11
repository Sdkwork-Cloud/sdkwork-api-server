use super::runtime_reload::{merge_applied_service_config, restart_required_changed_fields};
use super::*;
use sdkwork_api_app_credential::{
    list_official_provider_configs, persist_credential_with_secret_and_manager,
    resolve_provider_secret_with_fallback_and_manager,
};
use sdkwork_api_config::CacheBackendKind;
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static TEMP_BOOTSTRAP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[tokio::test]
async fn validate_secret_manager_for_store_checks_multiple_credentials() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    let manager = CredentialSecretManager::database_encrypted("runtime-test-master-key");

    persist_credential_with_secret_and_manager(
        &store,
        &manager,
        "tenant-1",
        "provider-a",
        "cred-a",
        "secret-a",
    )
    .await
    .unwrap();
    persist_credential_with_secret_and_manager(
        &store,
        &manager,
        "tenant-2",
        "provider-b",
        "cred-b",
        "secret-b",
    )
    .await
    .unwrap();

    validate_secret_manager_for_store(&store, &manager)
        .await
        .unwrap();
}

#[tokio::test]
async fn build_cache_runtime_from_config_returns_memory_cache_runtime() {
    let config = StandaloneConfig::default();

    let stores = build_cache_runtime_from_config(&config).await.unwrap();
    stores
        .cache_store()
        .put("routing", "selection", b"provider-a".to_vec(), None, &[])
        .await
        .unwrap();
    let cached = stores
        .cache_store()
        .get("routing", "selection")
        .await
        .unwrap()
        .expect("cached entry");

    assert_eq!(cached.value(), b"provider-a");
}

#[tokio::test]
async fn build_cache_runtime_from_config_builds_redis_cache_runtime() {
    let mut config = StandaloneConfig::default();
    config.cache_backend = CacheBackendKind::Redis;
    let server = MinimalRedisPingServer::start();
    config.cache_url = Some(server.url_with_db(4));

    build_cache_runtime_from_config(&config).await.unwrap();
}

#[tokio::test]
async fn build_admin_store_from_config_surfaces_supported_dialects_for_mysql() {
    let mut config = StandaloneConfig::default();
    config.database_url = "mysql://router:secret@localhost:3306/router".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!("mysql should remain unsupported until a real driver ships"),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("mysql"));
    assert!(error.contains("supported dialects"));
    assert!(error.contains("sqlite"));
    assert!(error.contains("postgres"));
}

#[tokio::test]
async fn build_admin_store_from_config_surfaces_supported_dialects_for_libsql() {
    let mut config = StandaloneConfig::default();
    config.database_url = "libsql://router.example.com".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!("libsql should remain unsupported until a real driver ships"),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("libsql"));
    assert!(error.contains("supported dialects"));
    assert!(error.contains("sqlite"));
    assert!(error.contains("postgres"));
}

#[tokio::test]
async fn build_admin_store_from_config_seeds_builtin_official_provider_access() {
    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.official_openai_enabled = true;
    config.official_openai_base_url = "https://api.openai.com/v1".to_owned();
    config.official_openai_api_key = "sk-bootstrap-openai".to_owned();

    let (store, _commercial_billing) =
        build_admin_store_and_commercial_billing_from_config(&config)
            .await
            .unwrap();
    let manager = build_secret_manager_from_config(&config);

    let provider = store
        .find_provider("provider-openai-official")
        .await
        .unwrap()
        .expect("official openai provider");
    assert_eq!(provider.channel_id, "openai");
    assert_eq!(provider.adapter_kind, "openai");
    assert_eq!(provider.protocol_kind(), "openai");
    assert_eq!(provider.base_url, "https://api.openai.com/v1");

    let configs = list_official_provider_configs(store.as_ref())
        .await
        .unwrap();
    let official = configs
        .iter()
        .find(|config| config.provider_id == "provider-openai-official")
        .expect("official openai config");
    assert!(official.enabled);
    assert_eq!(official.base_url, "https://api.openai.com/v1");

    let secret = resolve_provider_secret_with_fallback_and_manager(
        store.as_ref(),
        &manager,
        "tenant-without-own-key",
        "provider-openai-official",
    )
    .await
    .unwrap();
    assert_eq!(secret.as_deref(), Some("sk-bootstrap-openai"));
}

#[tokio::test]
async fn build_admin_store_from_config_seeds_starter_official_model_catalog() {
    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();

    let (store, _commercial_billing) =
        build_admin_store_and_commercial_billing_from_config(&config)
            .await
            .unwrap();
    let models = store.list_models().await.unwrap();
    let channel_models = store.list_channel_models().await.unwrap();
    let model_prices = store.list_model_prices().await.unwrap();

    assert!(models.iter().any(|model| {
        model.provider_id == "provider-openai-official" && model.external_name == "gpt-4.1"
    }));
    assert!(models.iter().any(|model| {
        model.provider_id == "provider-openai-official"
            && model.external_name == "text-embedding-3-small"
    }));
    assert!(models.iter().any(|model| {
        model.provider_id == "provider-anthropic-official"
            && model.external_name == "claude-3-7-sonnet"
    }));
    assert!(models.iter().any(|model| {
        model.provider_id == "provider-gemini-official" && model.external_name == "gemini-2.5-pro"
    }));
    assert!(models.iter().any(|model| {
        model.provider_id == "provider-openai-official"
            && model.external_name == "gpt-3.5-turbo-instruct"
    }));
    assert!(models.iter().any(|model| {
        model.provider_id == "provider-openai-official" && model.external_name == "gpt-image-1"
    }));
    assert!(models.iter().any(|model| {
        model.provider_id == "provider-openai-official"
            && model.external_name == "gpt-4o-mini-transcribe"
    }));
    assert!(models.iter().any(|model| {
        model.provider_id == "provider-openai-official" && model.external_name == "gpt-4o-mini-tts"
    }));
    assert!(models.iter().any(|model| {
        model.provider_id == "provider-openai-official"
            && model.external_name == "gpt-4o-realtime-preview"
    }));
    assert!(channel_models
        .iter()
        .any(|model| { model.channel_id == "openai" && model.model_id == "gpt-image-1" }));
    assert!(channel_models.iter().any(|model| {
        model.channel_id == "openai" && model.model_id == "gpt-4o-mini-transcribe"
    }));
    assert!(channel_models
        .iter()
        .any(|model| { model.channel_id == "openai" && model.model_id == "gpt-4o-mini-tts" }));
    assert!(channel_models.iter().any(|model| {
        model.channel_id == "openai" && model.model_id == "gpt-4o-realtime-preview"
    }));
    assert!(model_prices.iter().any(|price| {
        price.channel_id == "openai"
            && price.model_id == "gpt-image-1"
            && price.proxy_provider_id == "provider-openai-official"
    }));
    assert!(model_prices.iter().any(|price| {
        price.channel_id == "openai"
            && price.model_id == "gpt-4o-mini-transcribe"
            && price.proxy_provider_id == "provider-openai-official"
    }));
    assert!(model_prices.iter().any(|price| {
        price.channel_id == "openai"
            && price.model_id == "gpt-4o-mini-tts"
            && price.proxy_provider_id == "provider-openai-official"
    }));
    assert!(model_prices.iter().any(|price| {
        price.channel_id == "openai"
            && price.model_id == "gpt-4o-realtime-preview"
            && price.proxy_provider_id == "provider-openai-official"
    }));
}

#[tokio::test]
async fn build_admin_store_from_config_loads_bootstrap_profile_data_pack() {
    let bootstrap_root = temp_bootstrap_root("profile-pack");
    write_bootstrap_profile_pack(&bootstrap_root);
    let pack = crate::bootstrap_data::load_bootstrap_profile_pack(&bootstrap_root, "dev").unwrap();
    assert_eq!(pack.data.provider_accounts.len(), 3);
    assert_eq!(pack.data.accounts.len(), 1);
    assert_eq!(pack.data.account_benefit_lots.len(), 2);
    assert_eq!(pack.data.account_holds.len(), 1);
    assert_eq!(pack.data.account_hold_allocations.len(), 1);
    assert_eq!(pack.data.account_ledger_entries.len(), 3);
    assert_eq!(pack.data.account_ledger_allocations.len(), 3);
    assert_eq!(pack.data.request_settlements.len(), 1);
    assert_eq!(pack.data.request_meter_facts.len(), 1);
    assert_eq!(pack.data.request_meter_metrics.len(), 2);
    assert_eq!(pack.data.account_commerce_reconciliation_states.len(), 1);
    assert_eq!(pack.data.service_runtime_nodes.len(), 2);
    assert_eq!(pack.data.extension_runtime_rollouts.len(), 1);
    assert_eq!(pack.data.extension_runtime_rollout_participants.len(), 2);
    assert_eq!(pack.data.standalone_config_rollouts.len(), 1);
    assert_eq!(pack.data.standalone_config_rollout_participants.len(), 1);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let (store, commercial_billing) = build_admin_store_and_commercial_billing_from_config(&config)
        .await
        .unwrap();

    let providers = store.list_providers().await.unwrap();
    let provider_accounts = store.list_provider_accounts().await.unwrap();
    let channel_models = store.list_channel_models().await.unwrap();
    let policies = store.list_routing_policies().await.unwrap();
    let profiles = store.list_routing_profiles().await.unwrap();
    let api_key_groups = store.list_api_key_groups().await.unwrap();
    let quota_policies = store.list_quota_policies().await.unwrap();
    let rate_limit_policies = store.list_rate_limit_policies().await.unwrap();
    let payment_methods = store.list_payment_methods().await.unwrap();
    let extension_installations = store.list_extension_installations().await.unwrap();
    let extension_instances = store.list_extension_instances().await.unwrap();
    let admin_users = store.list_admin_users().await.unwrap();
    let portal_users = store.list_portal_users().await.unwrap();
    let gateway_api_keys = store.list_gateway_api_keys().await.unwrap();
    let compiled_routing_snapshots = store.list_compiled_routing_snapshots().await.unwrap();
    let routing_decision_logs = store.list_routing_decision_logs().await.unwrap();
    let provider_health_snapshots = store.list_provider_health_snapshots().await.unwrap();
    let service_runtime_nodes = store.list_service_runtime_nodes().await.unwrap();
    let extension_runtime_rollouts = store.list_extension_runtime_rollouts().await.unwrap();
    let extension_runtime_rollout_participants = store
        .list_extension_runtime_rollout_participants("rollout-extension-openrouter-refresh")
        .await
        .unwrap();
    let standalone_config_rollouts = store.list_standalone_config_rollouts().await.unwrap();
    let standalone_config_rollout_participants = store
        .list_standalone_config_rollout_participants("rollout-config-gateway-reload")
        .await
        .unwrap();
    let commerce_orders = store.list_commerce_orders().await.unwrap();
    let commerce_payment_attempts = store.list_commerce_payment_attempts().await.unwrap();
    let commerce_payment_events = store.list_commerce_payment_events().await.unwrap();
    let commerce_webhook_inbox = store.list_commerce_webhook_inbox_records().await.unwrap();
    let commerce_refunds = store.list_commerce_refunds().await.unwrap();
    let commerce_reconciliation_runs = store.list_commerce_reconciliation_runs().await.unwrap();
    let commerce_reconciliation_items = store
        .list_commerce_reconciliation_items("recon-run-local-demo-growth-2026")
        .await
        .unwrap();
    let billing_events = store.list_billing_events().await.unwrap();
    let async_jobs = store.list_async_jobs().await.unwrap();
    let async_job_attempts = store
        .list_async_job_attempts("job-local-demo-growth-brief")
        .await
        .unwrap();
    let async_job_assets = store
        .list_async_job_assets("job-local-demo-growth-brief")
        .await
        .unwrap();
    let async_job_callbacks = store
        .list_async_job_callbacks("job-local-demo-growth-brief")
        .await
        .unwrap();
    let pricing_plans = commercial_billing
        .list_pricing_plan_records()
        .await
        .unwrap();
    let pricing_rates = commercial_billing
        .list_pricing_rate_records()
        .await
        .unwrap();
    let coupon_templates = store.list_coupon_template_records().await.unwrap();
    let coupon_codes = store.list_coupon_code_records().await.unwrap();
    let marketing_campaigns = store.list_marketing_campaign_records().await.unwrap();
    let account_records = commercial_billing.list_account_records().await.unwrap();
    let account_benefit_lots = commercial_billing
        .list_account_benefit_lots()
        .await
        .unwrap();
    let account_holds = commercial_billing.list_account_holds().await.unwrap();
    let request_settlements = commercial_billing
        .list_request_settlement_records()
        .await
        .unwrap();
    let local_demo_balance = commercial_billing
        .summarize_account_balance(7001, 1710007000000)
        .await
        .unwrap();
    let local_demo_ledger = commercial_billing
        .list_account_ledger_history(7001)
        .await
        .unwrap();
    let local_demo_reconciliation = commercial_billing
        .find_account_commerce_reconciliation_state(7001, "project_local_demo")
        .await
        .unwrap();

    assert!(providers.iter().any(|provider| {
        provider.id == "provider-openrouter-main" && provider.channel_id == "openrouter"
    }));
    assert!(provider_accounts.iter().any(|account| {
        account.provider_account_id == "acct-openrouter-default"
            && account.provider_id == "provider-openrouter-main"
            && account.execution_instance_id == "provider-openrouter-main"
    }));
    assert!(provider_accounts.iter().any(|account| {
        account.provider_account_id == "acct-siliconflow-default"
            && account.provider_id == "provider-siliconflow-main"
            && account.execution_instance_id == "provider-siliconflow-main"
    }));
    assert!(provider_accounts.iter().any(|account| {
        account.provider_account_id == "acct-ollama-local-default"
            && account.provider_id == "provider-ollama-local"
            && account.execution_instance_id == "provider-ollama-local"
    }));
    assert!(providers.iter().any(|provider| {
        provider.id == "provider-siliconflow-main" && provider.channel_id == "siliconflow"
    }));
    assert!(providers.iter().any(|provider| {
        provider.id == "provider-ollama-local" && provider.channel_id == "ollama"
    }));
    assert!(channel_models
        .iter()
        .any(|record| { record.channel_id == "openrouter" && record.model_id == "deepseek-chat" }));
    assert!(channel_models.iter().any(|record| {
        record.channel_id == "siliconflow" && record.model_id == "qwen-plus-latest"
    }));
    assert!(profiles.iter().any(|profile| {
        profile.profile_id == "profile-global-balanced"
            && profile.project_id == "project_local_demo"
    }));
    assert!(policies.iter().any(|policy| {
        policy.policy_id == "policy-default-responses"
            && policy.default_provider_id.as_deref() == Some("provider-openrouter-main")
    }));
    assert!(api_key_groups.iter().any(|group| {
        group.group_id == "group-local-demo-live"
            && group.default_routing_profile_id.as_deref() == Some("profile-global-balanced")
    }));
    assert!(quota_policies.iter().any(|policy| {
        policy.policy_id == "quota-default-live" && policy.project_id == "project_local_demo"
    }));
    assert!(rate_limit_policies.iter().any(|policy| {
        policy.policy_id == "rate-limit-default-live" && policy.project_id == "project_local_demo"
    }));
    assert!(payment_methods.iter().any(|method| {
        method.payment_method_id == "payment-stripe-hosted" && method.provider == "stripe"
    }));
    assert!(extension_installations.iter().any(|installation| {
        installation.installation_id == "installation-openrouter-builtin"
            && installation.extension_id == "sdkwork.provider.openrouter"
    }));
    assert!(extension_installations.iter().any(|installation| {
        installation.installation_id == "installation-siliconflow-builtin"
            && installation.extension_id == "sdkwork.provider.siliconflow"
    }));
    assert!(extension_installations.iter().any(|installation| {
        installation.installation_id == "installation-ollama-builtin"
            && installation.extension_id == "sdkwork.provider.ollama"
    }));
    assert!(extension_instances.iter().any(|instance| {
        instance.instance_id == "provider-openrouter-main"
            && instance.installation_id == "installation-openrouter-builtin"
    }));
    assert!(extension_instances.iter().any(|instance| {
        instance.instance_id == "provider-siliconflow-main"
            && instance.installation_id == "installation-siliconflow-builtin"
    }));
    assert!(extension_instances.iter().any(|instance| {
        instance.instance_id == "provider-ollama-local"
            && instance.installation_id == "installation-ollama-builtin"
    }));
    assert!(admin_users
        .iter()
        .any(|user| { user.id == "admin_local_default" && user.email == "admin@sdkwork.local" }));
    assert!(portal_users.iter().any(|user| {
        user.id == "user_local_demo"
            && user.email == "portal@sdkwork.local"
            && user.workspace_project_id == "project_local_demo"
    }));
    assert!(gateway_api_keys.iter().any(|record| {
        record.hashed_key == "a19d2bf76318aa7f619d684271469bb383faf1cb5bd4c680088465cde9d0003b"
            && record.api_key_group_id.as_deref() == Some("group-local-demo-live")
            && record.environment == "live"
    }));
    assert!(gateway_api_keys.iter().any(|record| {
        record.hashed_key == "13072ae2c436e62116c61d76c68e7cc32a7a1e252a1d192490d6ac7cc92295eb"
            && record.api_key_group_id.as_deref() == Some("group-local-demo-sandbox")
            && record.environment == "sandbox"
    }));
    assert!(compiled_routing_snapshots.iter().any(|snapshot| {
        snapshot.snapshot_id == "snapshot-local-demo-live-responses"
            && snapshot.api_key_group_id.as_deref() == Some("group-local-demo-live")
            && snapshot.applied_routing_profile_id.as_deref() == Some("profile-global-balanced")
            && snapshot.default_provider_id.as_deref() == Some("provider-openrouter-main")
    }));
    assert!(routing_decision_logs.iter().any(|decision| {
        decision.decision_id == "decision-local-demo-live-responses"
            && decision.selected_provider_id == "provider-openrouter-main"
            && decision.compiled_routing_snapshot_id.as_deref()
                == Some("snapshot-local-demo-live-responses")
            && decision
                .assessments
                .iter()
                .any(|assessment| assessment.provider_id == "provider-siliconflow-main")
    }));
    assert!(provider_health_snapshots.iter().any(|snapshot| {
        snapshot.provider_id == "provider-ollama-local"
            && snapshot.instance_id.as_deref() == Some("provider-ollama-local")
            && snapshot.running
            && snapshot.healthy
    }));
    assert!(service_runtime_nodes.iter().any(|node| {
        node.node_id == "node-gateway-local-a"
            && node.service_kind == "gateway"
            && node.last_seen_at_ms >= node.started_at_ms
    }));
    assert!(service_runtime_nodes.iter().any(|node| {
        node.node_id == "node-admin-local-a"
            && node.service_kind == "admin"
            && node.last_seen_at_ms >= node.started_at_ms
    }));
    assert!(extension_runtime_rollouts.iter().any(|rollout| {
        rollout.rollout_id == "rollout-extension-openrouter-refresh"
            && rollout.scope == "instance"
            && rollout.requested_instance_id.as_deref() == Some("provider-openrouter-main")
            && rollout.resolved_extension_id.as_deref() == Some("sdkwork.provider.openrouter")
    }));
    assert!(extension_runtime_rollout_participants
        .iter()
        .any(|participant| {
            participant.rollout_id == "rollout-extension-openrouter-refresh"
                && participant.node_id == "node-gateway-local-a"
                && participant.service_kind == "gateway"
                && participant.status == "succeeded"
        }));
    assert!(extension_runtime_rollout_participants
        .iter()
        .any(|participant| {
            participant.rollout_id == "rollout-extension-openrouter-refresh"
                && participant.node_id == "node-admin-local-a"
                && participant.service_kind == "admin"
                && participant.status == "pending"
        }));
    assert!(standalone_config_rollouts.iter().any(|rollout| {
        rollout.rollout_id == "rollout-config-gateway-reload"
            && rollout.requested_service_kind.as_deref() == Some("gateway")
    }));
    assert!(standalone_config_rollout_participants
        .iter()
        .any(|participant| {
            participant.rollout_id == "rollout-config-gateway-reload"
                && participant.node_id == "node-gateway-local-a"
                && participant.service_kind == "gateway"
                && participant.status == "pending"
        }));
    assert!(commerce_orders.iter().any(|order| {
        order.order_id == "order-local-demo-growth-2026"
            && order.project_id == "project_local_demo"
            && order.user_id == "user_local_demo"
            && order.payment_method_id.as_deref() == Some("payment-stripe-hosted")
    }));
    assert!(commerce_payment_attempts.iter().any(|attempt| {
        attempt.payment_attempt_id == "attempt-local-demo-growth-2026"
            && attempt.order_id == "order-local-demo-growth-2026"
            && attempt.payment_method_id == "payment-stripe-hosted"
            && attempt.idempotency_key == "attempt:order-local-demo-growth-2026:1"
    }));
    assert!(commerce_payment_events.iter().any(|event| {
        event.payment_event_id == "payment-event-local-demo-growth-2026"
            && event.order_id == "order-local-demo-growth-2026"
            && event.dedupe_key == "stripe:evt_local_demo_growth_2026"
    }));
    assert!(commerce_webhook_inbox.iter().any(|record| {
        record.webhook_inbox_id == "webhook-inbox-local-demo-growth-2026"
            && record.payment_method_id.as_deref() == Some("payment-stripe-hosted")
            && record.dedupe_key == "stripe:evt_local_demo_growth_2026"
    }));
    assert!(commerce_refunds.iter().any(|refund| {
        refund.refund_id == "refund-local-demo-growth-2026"
            && refund.order_id == "order-local-demo-growth-2026"
            && refund.payment_attempt_id.as_deref() == Some("attempt-local-demo-growth-2026")
    }));
    assert!(commerce_reconciliation_runs.iter().any(|run| {
        run.reconciliation_run_id == "recon-run-local-demo-growth-2026"
            && run.payment_method_id.as_deref() == Some("payment-stripe-hosted")
    }));
    assert!(commerce_reconciliation_items.iter().any(|item| {
        item.reconciliation_item_id == "recon-item-local-demo-growth-2026"
            && item.order_id.as_deref() == Some("order-local-demo-growth-2026")
            && item.payment_attempt_id.as_deref() == Some("attempt-local-demo-growth-2026")
            && item.refund_id.as_deref() == Some("refund-local-demo-growth-2026")
    }));
    assert!(billing_events.iter().any(|event| {
        event.event_id == "billing-local-demo-growth-2026"
            && event.project_id == "project_local_demo"
            && event.provider_id == "provider-openrouter-main"
            && event.compiled_routing_snapshot_id.as_deref()
                == Some("snapshot-local-demo-live-responses")
    }));
    assert!(async_jobs.iter().any(|job| {
        job.job_id == "job-local-demo-growth-brief"
            && job.provider_id.as_deref() == Some("provider-openrouter-main")
            && job.model_code.as_deref() == Some("deepseek-chat")
    }));
    assert!(async_jobs.iter().any(|job| {
        job.job_id == "job-partner-sandbox-review"
            && job.provider_id.as_deref() == Some("provider-ollama-local")
            && job.model_code.as_deref() == Some("llama3.2:latest")
    }));
    assert!(async_job_attempts.iter().any(|attempt| {
        attempt.attempt_id == 8801
            && attempt.job_id == "job-local-demo-growth-brief"
            && attempt.runtime_kind == "openrouter"
    }));
    assert!(async_job_assets.iter().any(|asset| {
        asset.asset_id == "asset-local-demo-growth-brief-json"
            && asset.job_id == "job-local-demo-growth-brief"
    }));
    assert!(async_job_callbacks.iter().any(|callback| {
        callback.callback_id == 9901
            && callback.job_id == "job-local-demo-growth-brief"
            && callback.dedupe_key.as_deref()
                == Some("openrouter:or-job-local-demo-growth-brief:completed")
    }));
    assert!(pricing_plans
        .iter()
        .any(|plan| plan.plan_code == "global-default-commercial"));
    assert!(pricing_rates
        .iter()
        .any(|rate| rate.metric_code == "tokens.input"));
    assert!(coupon_templates
        .iter()
        .any(|template| template.template_key == "launch-credit-100"));
    assert!(coupon_codes
        .iter()
        .any(|code| code.code_value == "LAUNCH100"));
    assert!(marketing_campaigns
        .iter()
        .any(|campaign| campaign.marketing_campaign_id == "campaign-launch-q2"));
    assert_eq!(account_records.len(), 1);
    assert!(account_records.iter().any(|account| {
        account.account_id == 7001
            && account.account_type == sdkwork_api_domain_billing::AccountType::Primary
            && account.currency_code == "USD"
    }));
    assert_eq!(account_benefit_lots.len(), 2);
    assert!(account_benefit_lots.iter().any(|lot| {
        lot.lot_id == 8001
            && lot.account_id == 7001
            && (lot.remaining_quantity - 2_997_700.0).abs() < f64::EPSILON
    }));
    assert!(account_holds.iter().any(|hold| {
        hold.hold_id == 8101
            && hold.account_id == 7001
            && hold.status == sdkwork_api_domain_billing::AccountHoldStatus::PartiallyReleased
    }));
    assert!(request_settlements.iter().any(|settlement| {
        settlement.request_settlement_id == 8301
            && settlement.account_id == 7001
            && settlement.status
                == sdkwork_api_domain_billing::RequestSettlementStatus::PartiallyReleased
    }));
    assert_eq!(local_demo_balance.account_id, 7001);
    assert_eq!(local_demo_balance.active_lot_count, 2);
    assert!((local_demo_balance.available_balance - 3_247_700.0).abs() < f64::EPSILON);
    assert!((local_demo_balance.consumed_balance - 2_300.0).abs() < f64::EPSILON);
    assert_eq!(local_demo_ledger.len(), 3);
    assert_eq!(
        local_demo_reconciliation
            .expect("local demo reconciliation state")
            .last_order_id,
        "order-local-demo-growth-2026"
    );
}

#[tokio::test]
async fn build_admin_store_from_config_applies_bootstrap_profile_data_idempotently() {
    let bootstrap_root = temp_bootstrap_root("profile-pack-idempotent");
    write_bootstrap_profile_pack(&bootstrap_root);
    let database_path = bootstrap_root.join("bootstrap-idempotent.db");
    let database_url = sqlite_url_for(database_path);

    let mut config = StandaloneConfig::default();
    config.database_url = database_url;
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let (store, commercial_billing) = build_admin_store_and_commercial_billing_from_config(&config)
        .await
        .unwrap();
    drop(store);
    drop(commercial_billing);

    let (store, commercial_billing) = build_admin_store_and_commercial_billing_from_config(&config)
        .await
        .unwrap();

    let openrouter_count = store
        .list_providers()
        .await
        .unwrap()
        .into_iter()
        .filter(|provider| provider.id == "provider-openrouter-main")
        .count();
    let profile_count = store
        .list_routing_profiles()
        .await
        .unwrap()
        .into_iter()
        .filter(|profile| profile.profile_id == "profile-global-balanced")
        .count();
    let api_key_group_count = store
        .list_api_key_groups()
        .await
        .unwrap()
        .into_iter()
        .filter(|group| group.group_id == "group-local-demo-live")
        .count();
    let pricing_plan_count = commercial_billing
        .list_pricing_plan_records()
        .await
        .unwrap()
        .into_iter()
        .filter(|plan| plan.plan_code == "global-default-commercial")
        .count();
    let extension_installation_count = store
        .list_extension_installations()
        .await
        .unwrap()
        .into_iter()
        .filter(|installation| installation.installation_id == "installation-openrouter-builtin")
        .count();
    let extension_instance_count = store
        .list_extension_instances()
        .await
        .unwrap()
        .into_iter()
        .filter(|instance| instance.instance_id == "provider-openrouter-main")
        .count();
    let provider_account_count = store
        .list_provider_accounts()
        .await
        .unwrap()
        .into_iter()
        .filter(|account| account.provider_account_id == "acct-openrouter-default")
        .count();
    let admin_user_count = store
        .list_admin_users()
        .await
        .unwrap()
        .into_iter()
        .filter(|user| user.id == "admin_local_default")
        .count();
    let portal_user_count = store
        .list_portal_users()
        .await
        .unwrap()
        .into_iter()
        .filter(|user| user.id == "user_local_demo")
        .count();
    let gateway_api_key_count = store
        .list_gateway_api_keys()
        .await
        .unwrap()
        .into_iter()
        .filter(|record| {
            record.hashed_key == "a19d2bf76318aa7f619d684271469bb383faf1cb5bd4c680088465cde9d0003b"
        })
        .count();
    let compiled_snapshot_count = store
        .list_compiled_routing_snapshots()
        .await
        .unwrap()
        .into_iter()
        .filter(|snapshot| snapshot.snapshot_id == "snapshot-local-demo-live-responses")
        .count();
    let routing_decision_count = store
        .list_routing_decision_logs()
        .await
        .unwrap()
        .into_iter()
        .filter(|decision| decision.decision_id == "decision-local-demo-live-responses")
        .count();
    let provider_health_count = store
        .list_provider_health_snapshots()
        .await
        .unwrap()
        .into_iter()
        .filter(|snapshot| {
            snapshot.provider_id == "provider-ollama-local"
                && snapshot.instance_id.as_deref() == Some("provider-ollama-local")
        })
        .count();
    let service_runtime_node_count = store
        .list_service_runtime_nodes()
        .await
        .unwrap()
        .into_iter()
        .filter(|node| node.node_id == "node-gateway-local-a")
        .count();
    let extension_runtime_rollout_count = store
        .list_extension_runtime_rollouts()
        .await
        .unwrap()
        .into_iter()
        .filter(|rollout| rollout.rollout_id == "rollout-extension-openrouter-refresh")
        .count();
    let extension_runtime_rollout_participant_count = store
        .list_extension_runtime_rollout_participants("rollout-extension-openrouter-refresh")
        .await
        .unwrap()
        .into_iter()
        .filter(|participant| participant.node_id == "node-gateway-local-a")
        .count();
    let standalone_config_rollout_count = store
        .list_standalone_config_rollouts()
        .await
        .unwrap()
        .into_iter()
        .filter(|rollout| rollout.rollout_id == "rollout-config-gateway-reload")
        .count();
    let standalone_config_rollout_participant_count = store
        .list_standalone_config_rollout_participants("rollout-config-gateway-reload")
        .await
        .unwrap()
        .into_iter()
        .filter(|participant| participant.node_id == "node-gateway-local-a")
        .count();
    let commerce_order_count = store
        .list_commerce_orders()
        .await
        .unwrap()
        .into_iter()
        .filter(|order| order.order_id == "order-local-demo-growth-2026")
        .count();
    let commerce_payment_attempt_count = store
        .list_commerce_payment_attempts()
        .await
        .unwrap()
        .into_iter()
        .filter(|attempt| attempt.payment_attempt_id == "attempt-local-demo-growth-2026")
        .count();
    let commerce_payment_event_count = store
        .list_commerce_payment_events()
        .await
        .unwrap()
        .into_iter()
        .filter(|event| event.dedupe_key == "stripe:evt_local_demo_growth_2026")
        .count();
    let commerce_webhook_inbox_count = store
        .list_commerce_webhook_inbox_records()
        .await
        .unwrap()
        .into_iter()
        .filter(|record| record.webhook_inbox_id == "webhook-inbox-local-demo-growth-2026")
        .count();
    let commerce_refund_count = store
        .list_commerce_refunds()
        .await
        .unwrap()
        .into_iter()
        .filter(|refund| refund.refund_id == "refund-local-demo-growth-2026")
        .count();
    let commerce_reconciliation_run_count = store
        .list_commerce_reconciliation_runs()
        .await
        .unwrap()
        .into_iter()
        .filter(|run| run.reconciliation_run_id == "recon-run-local-demo-growth-2026")
        .count();
    let commerce_reconciliation_item_count = store
        .list_commerce_reconciliation_items("recon-run-local-demo-growth-2026")
        .await
        .unwrap()
        .into_iter()
        .filter(|item| item.reconciliation_item_id == "recon-item-local-demo-growth-2026")
        .count();
    let billing_event_count = store
        .list_billing_events()
        .await
        .unwrap()
        .into_iter()
        .filter(|event| event.event_id == "billing-local-demo-growth-2026")
        .count();
    let async_job_count = store
        .list_async_jobs()
        .await
        .unwrap()
        .into_iter()
        .filter(|job| job.job_id == "job-local-demo-growth-brief")
        .count();
    let async_job_attempt_count = store
        .list_async_job_attempts("job-local-demo-growth-brief")
        .await
        .unwrap()
        .into_iter()
        .filter(|attempt| attempt.attempt_id == 8801)
        .count();
    let async_job_asset_count = store
        .list_async_job_assets("job-local-demo-growth-brief")
        .await
        .unwrap()
        .into_iter()
        .filter(|asset| asset.asset_id == "asset-local-demo-growth-brief-json")
        .count();
    let async_job_callback_count = store
        .list_async_job_callbacks("job-local-demo-growth-brief")
        .await
        .unwrap()
        .into_iter()
        .filter(|callback| callback.callback_id == 9901)
        .count();

    assert_eq!(openrouter_count, 1);
    assert_eq!(profile_count, 1);
    assert_eq!(api_key_group_count, 1);
    assert_eq!(pricing_plan_count, 1);
    assert_eq!(extension_installation_count, 1);
    assert_eq!(extension_instance_count, 1);
    assert_eq!(provider_account_count, 1);
    assert_eq!(admin_user_count, 1);
    assert_eq!(portal_user_count, 1);
    assert_eq!(gateway_api_key_count, 1);
    assert_eq!(compiled_snapshot_count, 1);
    assert_eq!(routing_decision_count, 1);
    assert_eq!(provider_health_count, 1);
    assert_eq!(service_runtime_node_count, 1);
    assert_eq!(extension_runtime_rollout_count, 1);
    assert_eq!(extension_runtime_rollout_participant_count, 1);
    assert_eq!(standalone_config_rollout_count, 1);
    assert_eq!(standalone_config_rollout_participant_count, 1);
    assert_eq!(commerce_order_count, 1);
    assert_eq!(commerce_payment_attempt_count, 1);
    assert_eq!(commerce_payment_event_count, 1);
    assert_eq!(commerce_webhook_inbox_count, 1);
    assert_eq!(commerce_refund_count, 1);
    assert_eq!(commerce_reconciliation_run_count, 1);
    assert_eq!(commerce_reconciliation_item_count, 1);
    assert_eq!(billing_event_count, 1);
    assert_eq!(async_job_count, 1);
    assert_eq!(async_job_attempt_count, 1);
    assert_eq!(async_job_asset_count, 1);
    assert_eq!(async_job_callback_count, 1);
}

#[tokio::test]
async fn build_admin_store_from_config_applies_repository_dev_bootstrap_profile_concurrently() {
    let bootstrap_root = temp_bootstrap_root("repo-dev-concurrent-bootstrap");
    let database_path = bootstrap_root.join("repo-dev-concurrent-bootstrap.db");
    let database_url = sqlite_url_for(database_path);
    let pool = run_migrations(&database_url).await.unwrap();

    let mut config = StandaloneConfig::default();
    config.database_url = database_url;
    config.bootstrap_profile = "dev".to_owned();

    let barrier = Arc::new(tokio::sync::Barrier::new(3));
    let mut tasks = Vec::new();
    for _ in 0..3 {
        let barrier = barrier.clone();
        let config = config.clone();
        let store = Arc::new(SqliteAdminStore::new(pool.clone()));
        tasks.push(tokio::spawn(async move {
            barrier.wait().await;
            crate::bootstrap_data::bootstrap_repository_data_from_config(
                store.as_ref(),
                store.as_ref(),
                store.as_ref(),
                &config,
            )
            .await?;
            Ok::<Arc<SqliteAdminStore>, anyhow::Error>(store)
        }));
    }

    let mut stores = Vec::new();
    for task in tasks {
        let store = task.await.unwrap().unwrap();
        stores.push(store);
    }

    let store = stores.pop().unwrap();
    let admin_user_count = store
        .list_admin_users()
        .await
        .unwrap()
        .into_iter()
        .filter(|user| user.email == "admin@sdkwork.local")
        .count();
    let payment_attempt_count = store
        .list_commerce_payment_attempts()
        .await
        .unwrap()
        .into_iter()
        .filter(|attempt| attempt.idempotency_key == "attempt:order-local-demo-sandbox-pack:1")
        .count();
    let pricing_plan_count = store
        .list_pricing_plan_records()
        .await
        .unwrap()
        .into_iter()
        .filter(|plan| plan.plan_code == "global-default-commercial")
        .count();

    assert_eq!(admin_user_count, 1);
    assert_eq!(payment_attempt_count, 1);
    assert_eq!(pricing_plan_count, 1);
}

#[tokio::test]
async fn build_admin_store_from_config_loads_bootstrap_profile_update_packs_in_order() {
    let bootstrap_root = temp_bootstrap_root("profile-pack-updates");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("profiles").join("dev.json"),
        &serde_json::json!({
            "profile_id": "dev",
            "description": "development bootstrap pack with ordered updates",
            "channels": ["channels/default.json"],
            "providers": ["providers/default.json"],
            "official_provider_configs": ["official-provider-configs/default.json"],
            "provider_accounts": ["provider-accounts/default.json"],
            "models": ["models/default.json"],
            "channel_models": ["channel-models/default.json"],
            "model_prices": ["model-prices/default.json"],
            "tenants": ["tenants/default.json"],
            "projects": ["projects/default.json"],
            "identities": ["identities/default.json"],
            "extensions": ["extensions/default.json"],
            "service_runtime_nodes": ["service-runtime-nodes/default.json"],
            "extension_runtime_rollouts": ["extension-runtime-rollouts/default.json"],
            "standalone_config_rollouts": ["standalone-config-rollouts/default.json"],
            "routing": ["routing/default.json"],
            "api_key_groups": ["api-key-groups/default.json"],
            "observability": ["observability/default.json"],
            "quota_policies": ["quota-policies/default.json"],
            "pricing": ["pricing/default.json"],
            "accounts": ["accounts/default.json"],
            "payment_methods": ["payment-methods/default.json"],
            "marketing": ["marketing/default.json"],
            "commerce": ["commerce/default.json"],
            "billing": ["billing/default.json"],
            "jobs": ["jobs/default.json", "jobs/dev.json"],
            "updates": [
                "updates/catalog-expansion.json",
                "updates/route-rebalance.json"
            ]
        }),
    );
    write_json(
        &bootstrap_root
            .join("updates")
            .join("catalog-expansion.json"),
        &serde_json::json!({
            "update_id": "catalog-expansion",
            "release_version": "2026.04.01",
            "description": "adds more official channels and models",
            "channels": ["channels/catalog-expansion.json"],
            "providers": ["providers/catalog-expansion.json"],
            "official_provider_configs": ["official-provider-configs/catalog-expansion.json"],
            "provider_accounts": ["provider-accounts/catalog-expansion.json"],
            "extensions": ["extensions/catalog-expansion.json"],
            "models": ["models/catalog-expansion.json"],
            "channel_models": ["channel-models/catalog-expansion.json"],
            "model_prices": ["model-prices/catalog-expansion.json"],
            "provider_models": ["provider-models/catalog-expansion.json"],
            "routing": ["routing/catalog-expansion.json"]
        }),
    );
    write_json(
        &bootstrap_root.join("updates").join("route-rebalance.json"),
        &serde_json::json!({
            "update_id": "route-rebalance",
            "release_version": "2026.04.02",
            "description": "rebalances default routing after catalog expansion",
            "depends_on": ["catalog-expansion"],
            "providers": ["providers/route-rebalance.json"],
            "routing": ["routing/route-rebalance.json"]
        }),
    );
    write_json(
        &bootstrap_root
            .join("channels")
            .join("catalog-expansion.json"),
        &serde_json::json!([
            { "id": "ernie", "name": "Baidu ERNIE" },
            { "id": "minimax", "name": "MiniMax" }
        ]),
    );
    write_json(
        &bootstrap_root
            .join("providers")
            .join("catalog-expansion.json"),
        &serde_json::json!([
            {
                "id": "provider-ernie-official",
                "channel_id": "ernie",
                "extension_id": "sdkwork.provider.ernie",
                "adapter_kind": "openai-compatible",
                "protocol_kind": "openai",
                "base_url": "https://qianfan.baidubce.com/v2",
                "display_name": "ERNIE Official",
                "channel_bindings": [{ "provider_id": "provider-ernie-official", "channel_id": "ernie", "is_primary": true }]
            },
            {
                "id": "provider-minimax-official",
                "channel_id": "minimax",
                "extension_id": "sdkwork.provider.minimax",
                "adapter_kind": "openai-compatible",
                "protocol_kind": "openai",
                "base_url": "https://api.minimax.chat/v1",
                "display_name": "MiniMax Official",
                "channel_bindings": [{ "provider_id": "provider-minimax-official", "channel_id": "minimax", "is_primary": true }]
            }
        ]),
    );
    write_json(
        &bootstrap_root
            .join("official-provider-configs")
            .join("catalog-expansion.json"),
        &serde_json::json!([
            {
                "provider_id": "provider-ernie-official",
                "key_reference": "ernie-default",
                "base_url": "https://qianfan.baidubce.com/v2",
                "enabled": false,
                "created_at_ms": 1710000000000u64,
                "updated_at_ms": 1710000000000u64
            },
            {
                "provider_id": "provider-minimax-official",
                "key_reference": "minimax-default",
                "base_url": "https://api.minimax.chat/v1",
                "enabled": false,
                "created_at_ms": 1710000000000u64,
                "updated_at_ms": 1710000000000u64
            }
        ]),
    );
    write_json(
        &bootstrap_root
            .join("provider-accounts")
            .join("catalog-expansion.json"),
        &serde_json::json!([
            {
                "provider_account_id": "acct-provider-ernie-official-primary",
                "provider_id": "provider-ernie-official",
                "display_name": "ERNIE Official Primary",
                "account_kind": "official_api",
                "owner_scope": "platform",
                "owner_tenant_id": null,
                "execution_instance_id": "provider-ernie-official",
                "base_url_override": null,
                "region": "cn",
                "priority": 945,
                "weight": 100,
                "enabled": true,
                "routing_tags": ["official", "primary", "cn", "enterprise"],
                "health_score_hint": 0.985,
                "latency_ms_hint": 185,
                "cost_hint": 0.62,
                "success_rate_hint": 0.985,
                "throughput_hint": null,
                "max_concurrency": null,
                "daily_budget": null,
                "notes": "Catalog-expansion ERNIE bootstrap account"
            },
            {
                "provider_account_id": "acct-provider-minimax-official-primary",
                "provider_id": "provider-minimax-official",
                "display_name": "MiniMax Official Primary",
                "account_kind": "official_api",
                "owner_scope": "platform",
                "owner_tenant_id": null,
                "execution_instance_id": "provider-minimax-official",
                "base_url_override": null,
                "region": "cn",
                "priority": 905,
                "weight": 100,
                "enabled": true,
                "routing_tags": ["official", "primary", "cn", "reasoning"],
                "health_score_hint": 0.984,
                "latency_ms_hint": 195,
                "cost_hint": 0.78,
                "success_rate_hint": 0.984,
                "throughput_hint": null,
                "max_concurrency": null,
                "daily_budget": null,
                "notes": "Catalog-expansion MiniMax bootstrap account"
            }
        ]),
    );
    write_json(
        &bootstrap_root
            .join("extensions")
            .join("catalog-expansion.json"),
        &serde_json::json!({
            "installations": [
                {
                    "installation_id": "installation-ernie-official-builtin",
                    "extension_id": "sdkwork.provider.ernie",
                    "runtime": "builtin",
                    "enabled": true,
                    "entrypoint": null,
                    "config": {
                        "health_path": "/models",
                        "plugin_family": "ernie"
                    }
                },
                {
                    "installation_id": "installation-minimax-official-builtin",
                    "extension_id": "sdkwork.provider.minimax",
                    "runtime": "builtin",
                    "enabled": true,
                    "entrypoint": null,
                    "config": {
                        "health_path": "/models",
                        "plugin_family": "minimax"
                    }
                }
            ],
            "instances": [
                {
                    "instance_id": "provider-ernie-official",
                    "installation_id": "installation-ernie-official-builtin",
                    "extension_id": "sdkwork.provider.ernie",
                    "enabled": true,
                    "base_url": "https://qianfan.baidubce.com/v2",
                    "credential_ref": null,
                    "config": {
                        "routing_hint": "cn-official"
                    }
                },
                {
                    "instance_id": "provider-minimax-official",
                    "installation_id": "installation-minimax-official-builtin",
                    "extension_id": "sdkwork.provider.minimax",
                    "enabled": true,
                    "base_url": "https://api.minimax.chat/v1",
                    "credential_ref": null,
                    "config": {
                        "routing_hint": "cn-reasoning"
                    }
                }
            ]
        }),
    );
    write_json(
        &bootstrap_root.join("models").join("catalog-expansion.json"),
        &serde_json::json!([
            {
                "external_name": "ernie-4.5-turbo",
                "provider_id": "provider-ernie-official",
                "capabilities": ["responses", "chat_completions"],
                "streaming": true,
                "context_window": 128000
            },
            {
                "external_name": "minimax-m1",
                "provider_id": "provider-minimax-official",
                "capabilities": ["responses", "chat_completions"],
                "streaming": true,
                "context_window": 1000000
            }
        ]),
    );
    write_json(
        &bootstrap_root
            .join("provider-models")
            .join("catalog-expansion.json"),
        &serde_json::json!([
            {
                "proxy_provider_id": "provider-ernie-official",
                "channel_id": "ernie",
                "model_id": "ernie-4.5-turbo",
                "provider_model_id": "ernie-4.5-turbo",
                "provider_model_family": "ernie",
                "capabilities": ["responses", "chat_completions"],
                "streaming": true,
                "context_window": 128000,
                "max_output_tokens": 8192,
                "supports_prompt_caching": false,
                "supports_reasoning_usage": true,
                "supports_tool_usage_metrics": true,
                "is_default_route": true,
                "is_active": true
            },
            {
                "proxy_provider_id": "provider-minimax-official",
                "channel_id": "minimax",
                "model_id": "minimax-m1",
                "provider_model_id": "minimax-m1",
                "provider_model_family": "minimax",
                "capabilities": ["responses", "chat_completions"],
                "streaming": true,
                "context_window": 1000000,
                "max_output_tokens": 65536,
                "supports_prompt_caching": false,
                "supports_reasoning_usage": true,
                "supports_tool_usage_metrics": true,
                "is_default_route": true,
                "is_active": true
            }
        ]),
    );
    write_json(
        &bootstrap_root
            .join("channel-models")
            .join("catalog-expansion.json"),
        &serde_json::json!([
            {
                "channel_id": "ernie",
                "model_id": "ernie-4.5-turbo",
                "model_display_name": "ERNIE 4.5 Turbo",
                "capabilities": ["responses", "chat_completions"],
                "streaming": true,
                "context_window": 128000,
                "description": "ERNIE flagship default"
            },
            {
                "channel_id": "minimax",
                "model_id": "minimax-m1",
                "model_display_name": "MiniMax M1",
                "capabilities": ["responses", "chat_completions"],
                "streaming": true,
                "context_window": 1000000,
                "description": "MiniMax reasoning default"
            }
        ]),
    );
    write_json(
        &bootstrap_root
            .join("model-prices")
            .join("catalog-expansion.json"),
        &serde_json::json!([
            {
                "channel_id": "ernie",
                "model_id": "ernie-4.5-turbo",
                "proxy_provider_id": "provider-ernie-official",
                "currency_code": "USD",
                "price_unit": "per_1m_tokens",
                "input_price": 0.55,
                "output_price": 1.65,
                "cache_read_price": 0.0,
                "cache_write_price": 0.0,
                "request_price": 0.0,
                "is_active": true
            },
            {
                "channel_id": "minimax",
                "model_id": "minimax-m1",
                "proxy_provider_id": "provider-minimax-official",
                "currency_code": "USD",
                "price_unit": "per_1m_tokens",
                "input_price": 0.9,
                "output_price": 2.4,
                "cache_read_price": 0.0,
                "cache_write_price": 0.0,
                "request_price": 0.0,
                "is_active": true
            }
        ]),
    );
    write_json(
        &bootstrap_root
            .join("routing")
            .join("catalog-expansion.json"),
        &serde_json::json!({
            "profiles": [
                {
                    "profile_id": "profile-asia-official",
                    "tenant_id": "tenant_local_demo",
                    "project_id": "project_local_demo",
                    "name": "Asia Official",
                    "slug": "asia-official",
                    "description": "Asia-focused official providers",
                    "active": true,
                    "strategy": "deterministic_priority",
                    "ordered_provider_ids": [
                        "provider-ernie-official",
                        "provider-minimax-official",
                        "provider-siliconflow-main"
                    ],
                    "default_provider_id": "provider-ernie-official",
                    "max_cost": 4.2,
                    "max_latency_ms": 9000,
                    "require_healthy": false,
                    "preferred_region": "apac",
                    "created_at_ms": 1710000100000u64,
                    "updated_at_ms": 1710000100000u64
                }
            ],
            "policies": [],
            "project_preferences": []
        }),
    );
    write_json(
        &bootstrap_root
            .join("providers")
            .join("route-rebalance.json"),
        &serde_json::json!([
            {
                "id": "provider-minimax-official",
                "channel_id": "minimax",
                "extension_id": "sdkwork.provider.minimax",
                "adapter_kind": "openai-compatible",
                "protocol_kind": "openai",
                "base_url": "https://api.minimax.chat/v1/text",
                "display_name": "MiniMax Official Global",
                "channel_bindings": [{ "provider_id": "provider-minimax-official", "channel_id": "minimax", "is_primary": true }]
            }
        ]),
    );
    write_json(
        &bootstrap_root.join("routing").join("route-rebalance.json"),
        &serde_json::json!({
            "profiles": [
                {
                    "profile_id": "profile-global-balanced",
                    "tenant_id": "tenant_local_demo",
                    "project_id": "project_local_demo",
                    "name": "Global Balanced",
                    "slug": "global-balanced",
                    "description": "Balanced multi-provider routing with update overlay",
                    "active": true,
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-minimax-official",
                        "provider-openrouter-main",
                        "provider-siliconflow-main",
                        "provider-ollama-local"
                    ],
                    "default_provider_id": "provider-minimax-official",
                    "max_cost": 3.8,
                    "max_latency_ms": 7600,
                    "require_healthy": false,
                    "preferred_region": "global",
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000200000u64
                }
            ],
            "policies": [],
            "project_preferences": [
                {
                    "project_id": "project_local_demo",
                    "preset_id": "profile-global-balanced",
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-minimax-official",
                        "provider-openrouter-main",
                        "provider-siliconflow-main"
                    ],
                    "default_provider_id": "provider-minimax-official",
                    "max_cost": 3.8,
                    "max_latency_ms": 7600,
                    "require_healthy": false,
                    "preferred_region": "global",
                    "updated_at_ms": 1710000200000u64
                }
            ]
        }),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let store = build_admin_store_from_config(&config).await.unwrap();
    let channels = store.list_channels().await.unwrap();
    let providers = store.list_providers().await.unwrap();
    let provider_accounts = store.list_provider_accounts().await.unwrap();
    let official_provider_configs = store.list_official_provider_configs().await.unwrap();
    let channel_models = store.list_channel_models().await.unwrap();
    let routing_profiles = store.list_routing_profiles().await.unwrap();
    let project_preferences = store
        .find_project_routing_preferences("project_local_demo")
        .await
        .unwrap()
        .unwrap();

    assert!(channels.iter().any(|record| record.id == "ernie"));
    assert!(channels.iter().any(|record| record.id == "minimax"));
    assert!(providers.iter().any(|record| {
        record.id == "provider-ernie-official"
            && record.channel_id == "ernie"
            && record.base_url == "https://qianfan.baidubce.com/v2"
    }));
    assert!(providers.iter().any(|record| {
        record.id == "provider-minimax-official"
            && record.display_name == "MiniMax Official Global"
            && record.base_url == "https://api.minimax.chat/v1/text"
    }));
    assert!(official_provider_configs.iter().any(|record| {
        record.provider_id == "provider-minimax-official"
            && record.key_reference == "minimax-default"
    }));
    assert!(provider_accounts.iter().any(|record| {
        record.provider_account_id == "acct-provider-ernie-official-primary"
            && record.provider_id == "provider-ernie-official"
            && record.execution_instance_id == "provider-ernie-official"
            && record.enabled
    }));
    assert!(provider_accounts.iter().any(|record| {
        record.provider_account_id == "acct-provider-minimax-official-primary"
            && record.provider_id == "provider-minimax-official"
            && record.execution_instance_id == "provider-minimax-official"
            && record.enabled
    }));
    assert!(channel_models
        .iter()
        .any(|record| { record.channel_id == "ernie" && record.model_id == "ernie-4.5-turbo" }));
    assert!(channel_models
        .iter()
        .any(|record| { record.channel_id == "minimax" && record.model_id == "minimax-m1" }));
    assert!(routing_profiles.iter().any(|record| {
        record.profile_id == "profile-asia-official"
            && record.default_provider_id.as_deref() == Some("provider-ernie-official")
    }));
    assert!(routing_profiles.iter().any(|record| {
        record.profile_id == "profile-global-balanced"
            && record.default_provider_id.as_deref() == Some("provider-minimax-official")
            && record.ordered_provider_ids.first().map(String::as_str)
                == Some("provider-minimax-official")
    }));
    assert_eq!(
        project_preferences.default_provider_id.as_deref(),
        Some("provider-minimax-official")
    );
    assert_eq!(
        project_preferences
            .ordered_provider_ids
            .first()
            .map(String::as_str),
        Some("provider-minimax-official")
    );
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_profile_update_with_missing_dependency() {
    let bootstrap_root = temp_bootstrap_root("profile-pack-update-missing-dependency");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("profiles").join("dev.json"),
        &serde_json::json!({
            "profile_id": "dev",
            "description": "development bootstrap pack with invalid update dependency",
            "channels": ["channels/default.json"],
            "providers": ["providers/default.json"],
            "official_provider_configs": ["official-provider-configs/default.json"],
            "provider_accounts": ["provider-accounts/default.json"],
            "models": ["models/default.json"],
            "channel_models": ["channel-models/default.json"],
            "model_prices": ["model-prices/default.json"],
            "tenants": ["tenants/default.json"],
            "projects": ["projects/default.json"],
            "identities": ["identities/default.json"],
            "extensions": ["extensions/default.json"],
            "service_runtime_nodes": ["service-runtime-nodes/default.json"],
            "extension_runtime_rollouts": ["extension-runtime-rollouts/default.json"],
            "standalone_config_rollouts": ["standalone-config-rollouts/default.json"],
            "routing": ["routing/default.json"],
            "api_key_groups": ["api-key-groups/default.json"],
            "observability": ["observability/default.json"],
            "quota_policies": ["quota-policies/default.json"],
            "pricing": ["pricing/default.json"],
            "payment_methods": ["payment-methods/default.json"],
            "marketing": ["marketing/default.json"],
            "commerce": ["commerce/default.json"],
            "billing": ["billing/default.json"],
            "jobs": ["jobs/default.json", "jobs/dev.json"],
            "updates": ["updates/route-rebalance.json"]
        }),
    );
    write_json(
        &bootstrap_root.join("updates").join("route-rebalance.json"),
        &serde_json::json!({
            "update_id": "route-rebalance",
            "release_version": "2026.04.02",
            "description": "invalid because dependency is missing",
            "depends_on": ["catalog-expansion"]
        }),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!("bootstrap should reject update packs that reference missing dependencies"),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("catalog-expansion"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_discovers_repository_bootstrap_profile_data_pack() {
    let bootstrap_root = temp_bootstrap_root("repo-discovery");
    let database_path = bootstrap_root.join("repo-discovery.db");

    let mut config = StandaloneConfig::default();
    config.database_url = sqlite_url_for(database_path);
    config.bootstrap_profile = "prod".to_owned();

    let (store, commercial_billing) = build_admin_store_and_commercial_billing_from_config(&config)
        .await
        .unwrap();
    let providers = store.list_providers().await.unwrap();
    let provider_accounts = store.list_provider_accounts().await.unwrap();
    let provider_models = store.list_provider_models().await.unwrap();
    let model_prices = store.list_model_prices().await.unwrap();
    let routing_profiles = store.list_routing_profiles().await.unwrap();
    let api_key_groups = store.list_api_key_groups().await.unwrap();
    let compiled_routing_snapshots = store.list_compiled_routing_snapshots().await.unwrap();
    let routing_decision_logs = store.list_routing_decision_logs().await.unwrap();
    let billing_events = store.list_billing_events().await.unwrap();
    let extension_installations = store.list_extension_installations().await.unwrap();
    let extension_instances = store.list_extension_instances().await.unwrap();
    let account_holds = commercial_billing.list_account_holds().await.unwrap();
    let request_settlements = commercial_billing
        .list_request_settlement_records()
        .await
        .unwrap();
    let coupon_codes = store.list_coupon_code_records().await.unwrap();
    let pricing_plans = commercial_billing
        .list_pricing_plan_records()
        .await
        .unwrap();
    let pricing_rates = commercial_billing
        .list_pricing_rate_records()
        .await
        .unwrap();
    let global_ledger = commercial_billing
        .list_account_ledger_history(7001001)
        .await
        .unwrap();
    let global_balance = commercial_billing
        .summarize_account_balance(7001001, 1710000600000)
        .await
        .unwrap();

    assert!(providers
        .iter()
        .any(|provider| provider.id == "provider-openrouter-main"));
    assert!(providers.iter().any(|provider| {
        provider.id == "provider-openrouter-main" && provider.channel_id == "openai"
    }));
    assert!(providers
        .iter()
        .any(|provider| provider.id == "provider-openai-official"));
    assert!(providers
        .iter()
        .any(|provider| provider.id == "provider-hunyuan-official"));
    assert!(providers
        .iter()
        .any(|provider| provider.id == "provider-moonshot-official"));
    assert!(providers
        .iter()
        .any(|provider| provider.id == "provider-zhipu-official"));
    assert!(providers
        .iter()
        .any(|provider| provider.id == "provider-mistral-official"));
    assert!(providers
        .iter()
        .any(|provider| provider.id == "provider-cohere-official"));
    assert!(provider_accounts.iter().any(|account| {
        account.provider_account_id == "acct-provider-ernie-official-primary"
            && account.provider_id == "provider-ernie-official"
            && account.execution_instance_id == "provider-ernie-official"
            && account.enabled
    }));
    assert!(provider_accounts.iter().any(|account| {
        account.provider_account_id == "acct-provider-minimax-official-primary"
            && account.provider_id == "provider-minimax-official"
            && account.execution_instance_id == "provider-minimax-official"
            && account.enabled
    }));
    assert!(routing_profiles
        .iter()
        .any(|profile| profile.profile_id == "profile-global-balanced"));
    assert!(routing_profiles
        .iter()
        .any(|profile| profile.profile_id == "profile-global-official-ranked"));
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
        .any(|profile| profile.profile_id == "profile-global-china-ranked"));
    assert!(routing_profiles
        .iter()
        .any(|profile| profile.profile_id == "profile-global-openrouter-xai"));
    assert!(routing_profiles
        .iter()
        .any(|profile| profile.profile_id == "profile-global-openrouter-moonshot"));
    assert!(routing_profiles
        .iter()
        .any(|profile| profile.profile_id == "profile-global-openrouter-mistral"));
    assert!(api_key_groups
        .iter()
        .any(|group| group.group_id == "group-proxy-openrouter-xai-live"));
    assert!(api_key_groups
        .iter()
        .any(|group| group.group_id == "group-proxy-openrouter-moonshot-live"));
    assert!(api_key_groups
        .iter()
        .any(|group| group.group_id == "group-proxy-openrouter-mistral-live"));
    assert!(api_key_groups
        .iter()
        .any(|group| group.group_id == "group-official-ernie-live"));
    assert!(api_key_groups
        .iter()
        .any(|group| group.group_id == "group-official-minimax-live"));
    assert!(extension_installations.iter().any(|installation| {
        installation.installation_id == "installation-openai-official-builtin"
            && installation.extension_id == "sdkwork.provider.openai.official"
    }));
    assert!(extension_installations.iter().any(|installation| {
        installation.installation_id == "installation-anthropic-official-builtin"
            && installation.extension_id == "sdkwork.provider.anthropic"
    }));
    assert!(extension_installations.iter().any(|installation| {
        installation.installation_id == "installation-gemini-official-builtin"
            && installation.extension_id == "sdkwork.provider.gemini"
    }));
    assert!(extension_installations.iter().any(|installation| {
        installation.installation_id == "installation-deepseek-official-builtin"
            && installation.extension_id == "sdkwork.provider.deepseek"
    }));
    assert!(extension_installations.iter().any(|installation| {
        installation.installation_id == "installation-qwen-official-builtin"
            && installation.extension_id == "sdkwork.provider.qwen"
    }));
    assert!(extension_installations.iter().any(|installation| {
        installation.installation_id == "installation-ernie-official-builtin"
            && installation.extension_id == "sdkwork.provider.ernie"
    }));
    assert!(extension_installations.iter().any(|installation| {
        installation.installation_id == "installation-minimax-official-builtin"
            && installation.extension_id == "sdkwork.provider.minimax"
    }));
    assert!(extension_installations.iter().any(|installation| {
        installation.installation_id == "installation-siliconflow-builtin"
            && installation.extension_id == "sdkwork.provider.siliconflow"
    }));
    assert!(extension_installations.iter().any(|installation| {
        installation.installation_id == "installation-openrouter-builtin"
            && installation.extension_id == "sdkwork.provider.openrouter"
    }));
    assert!(extension_installations.iter().any(|installation| {
        installation.installation_id == "installation-ollama-builtin"
            && installation.extension_id == "sdkwork.provider.ollama"
    }));
    assert!(extension_instances.iter().any(|instance| {
        instance.instance_id == "provider-openai-official"
            && instance.installation_id == "installation-openai-official-builtin"
    }));
    assert!(extension_instances.iter().any(|instance| {
        instance.instance_id == "provider-anthropic-official"
            && instance.installation_id == "installation-anthropic-official-builtin"
    }));
    assert!(extension_instances.iter().any(|instance| {
        instance.instance_id == "provider-gemini-official"
            && instance.installation_id == "installation-gemini-official-builtin"
    }));
    assert!(extension_instances.iter().any(|instance| {
        instance.instance_id == "provider-deepseek-official"
            && instance.installation_id == "installation-deepseek-official-builtin"
    }));
    assert!(extension_instances.iter().any(|instance| {
        instance.instance_id == "provider-qwen-official"
            && instance.installation_id == "installation-qwen-official-builtin"
    }));
    assert!(extension_instances.iter().any(|instance| {
        instance.instance_id == "provider-ernie-official"
            && instance.installation_id == "installation-ernie-official-builtin"
    }));
    assert!(extension_instances.iter().any(|instance| {
        instance.instance_id == "provider-minimax-official"
            && instance.installation_id == "installation-minimax-official-builtin"
    }));
    assert!(extension_instances.iter().any(|instance| {
        instance.instance_id == "provider-siliconflow-main"
            && instance.installation_id == "installation-siliconflow-builtin"
    }));
    assert!(extension_instances.iter().any(|instance| {
        instance.instance_id == "provider-openrouter-main"
            && instance.installation_id == "installation-openrouter-builtin"
    }));
    assert!(extension_instances.iter().any(|instance| {
        instance.instance_id == "provider-ollama-local"
            && instance.installation_id == "installation-ollama-builtin"
    }));
    assert!(provider_models.iter().any(|record| {
        record.proxy_provider_id == "provider-gemini-official"
            && record.channel_id == "gemini"
            && record.model_id == "gemini-2.5-pro"
            && record.supports_prompt_caching
    }));
    assert!(provider_models.iter().any(|record| {
        record.proxy_provider_id == "provider-openrouter-main"
            && record.channel_id == "xai"
            && record.model_id == "grok-4"
    }));
    assert!(provider_models.iter().any(|record| {
        record.proxy_provider_id == "provider-openrouter-main"
            && record.channel_id == "moonshot"
            && record.model_id == "kimi-k2.5"
    }));
    assert!(provider_models.iter().any(|record| {
        record.proxy_provider_id == "provider-openrouter-main"
            && record.channel_id == "mistral"
            && record.model_id == "mistral-large-latest"
    }));
    let gemini_price = model_prices
        .iter()
        .find(|record| {
            record.channel_id == "gemini"
                && record.model_id == "gemini-2.5-pro"
                && record.proxy_provider_id == "provider-gemini-official"
        })
        .expect("gemini official pricing");
    assert_eq!(gemini_price.price_source_kind, "official");
    assert!(gemini_price.pricing_tiers.len() >= 2);
    assert!(gemini_price
        .billing_notes
        .as_deref()
        .is_some_and(|notes| notes.contains("prompt")));

    let openrouter_price = model_prices
        .iter()
        .find(|record| {
            record.channel_id == "openai"
                && record.model_id == "gpt-4.1"
                && record.proxy_provider_id == "provider-openrouter-main"
        })
        .expect("openrouter openai pricing");
    assert_eq!(openrouter_price.price_source_kind, "proxy");
    assert_eq!(openrouter_price.input_price, 2.0);
    assert_eq!(openrouter_price.output_price, 8.0);
    assert!(openrouter_price
        .billing_notes
        .as_deref()
        .is_some_and(|notes| notes.contains("pass-through")));
    assert!(model_prices.iter().any(|record| {
        record.channel_id == "xai"
            && record.model_id == "grok-4"
            && record.proxy_provider_id == "provider-openrouter-main"
            && record.price_source_kind == "proxy"
    }));
    assert!(model_prices.iter().any(|record| {
        record.channel_id == "moonshot"
            && record.model_id == "kimi-k2.5"
            && record.proxy_provider_id == "provider-openrouter-main"
            && record.price_source_kind == "proxy"
    }));
    assert!(model_prices.iter().any(|record| {
        record.channel_id == "mistral"
            && record.model_id == "mistral-large-latest"
            && record.proxy_provider_id == "provider-openrouter-main"
            && record.price_source_kind == "proxy"
    }));
    assert!(compiled_routing_snapshots
        .iter()
        .any(|snapshot| snapshot.snapshot_id == "snapshot-prod-openrouter-xai"));
    assert!(compiled_routing_snapshots
        .iter()
        .any(|snapshot| snapshot.snapshot_id == "snapshot-prod-openrouter-moonshot"));
    assert!(compiled_routing_snapshots
        .iter()
        .any(|snapshot| snapshot.snapshot_id == "snapshot-prod-openrouter-mistral"));
    assert!(compiled_routing_snapshots
        .iter()
        .any(|snapshot| snapshot.snapshot_id == "snapshot-prod-ernie-official"));
    assert!(compiled_routing_snapshots
        .iter()
        .any(|snapshot| snapshot.snapshot_id == "snapshot-prod-minimax-official"));
    assert!(routing_decision_logs
        .iter()
        .any(|decision| decision.decision_id == "decision-prod-openrouter-xai"));
    assert!(routing_decision_logs
        .iter()
        .any(|decision| decision.decision_id == "decision-prod-openrouter-moonshot"));
    assert!(routing_decision_logs
        .iter()
        .any(|decision| decision.decision_id == "decision-prod-openrouter-mistral"));
    assert!(routing_decision_logs
        .iter()
        .any(|decision| decision.decision_id == "decision-prod-ernie-official"));
    assert!(routing_decision_logs
        .iter()
        .any(|decision| decision.decision_id == "decision-prod-minimax-official"));
    assert!(billing_events
        .iter()
        .any(|event| event.event_id == "billing-prod-openrouter-xai"));
    assert!(billing_events
        .iter()
        .any(|event| event.event_id == "billing-prod-openrouter-moonshot"));
    assert!(billing_events
        .iter()
        .any(|event| event.event_id == "billing-prod-openrouter-mistral"));
    assert!(billing_events
        .iter()
        .any(|event| event.event_id == "billing-prod-ernie-official"));
    assert!(billing_events
        .iter()
        .any(|event| event.event_id == "billing-prod-minimax-official"));
    assert!(coupon_codes
        .iter()
        .any(|code| code.code_value == "LAUNCH100"));
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
        rate.pricing_plan_id == 9106
            && rate.metric_code == "tokens.input"
            && rate.provider_code.is_none()
            && rate.model_code.is_none()
    }));
    assert!(pricing_rates.iter().any(|rate| {
        rate.pricing_plan_id == 9103
            && rate.metric_code == "tokens.input"
            && rate.provider_code.as_deref() == Some("provider-openrouter-main")
            && rate.model_code.is_none()
    }));
    assert!(pricing_rates.iter().any(|rate| {
        rate.pricing_plan_id == 9111
            && rate.metric_code == "tokens.output"
            && rate.provider_code.as_deref() == Some("provider-siliconflow-main")
            && rate.model_code.is_none()
    }));
    assert!(pricing_rates.iter().any(|rate| {
        rate.pricing_plan_id == 9112
            && rate.metric_code == "tokens.input"
            && rate.provider_code.as_deref() == Some("provider-ollama-local")
            && rate.model_code.is_none()
    }));
    assert!(account_holds.iter().any(|hold| {
        hold.hold_id == 8101002
            && hold.request_id == 610002
            && hold.account_id == 7001001
            && hold.captured_quantity == 4200.0
    }));
    assert!(account_holds.iter().any(|hold| {
        hold.hold_id == 8101003
            && hold.request_id == 610003
            && hold.account_id == 7001001
            && hold.captured_quantity == 8400.0
    }));
    assert!(request_settlements.iter().any(|settlement| {
        settlement.request_settlement_id == 8301002
            && settlement.request_id == 610002
            && settlement.account_id == 7001001
    }));
    assert!(request_settlements.iter().any(|settlement| {
        settlement.request_settlement_id == 8301003
            && settlement.request_id == 610003
            && settlement.account_id == 7001001
    }));
    assert!(account_holds.iter().any(|hold| {
        hold.hold_id == 8101007
            && hold.request_id == 610007
            && hold.account_id == 7001001
            && hold.captured_quantity == 30000.0
    }));
    assert!(account_holds.iter().any(|hold| {
        hold.hold_id == 8101012
            && hold.request_id == 610012
            && hold.account_id == 7001001
            && hold.captured_quantity == 26000.0
    }));
    assert!(account_holds.iter().any(|hold| {
        hold.hold_id == 8101013
            && hold.request_id == 610013
            && hold.account_id == 7001001
            && hold.captured_quantity == 23000.0
    }));
    assert!(account_holds.iter().any(|hold| {
        hold.hold_id == 8101018
            && hold.request_id == 610018
            && hold.account_id == 7001001
            && hold.captured_quantity == 1600.0
    }));
    assert!(account_holds.iter().any(|hold| {
        hold.hold_id == 8101019
            && hold.request_id == 610019
            && hold.account_id == 7001001
            && hold.captured_quantity == 32400.0
    }));
    assert!(request_settlements.iter().any(|settlement| {
        settlement.request_settlement_id == 8301004
            && settlement.request_id == 610004
            && settlement.account_id == 7001001
    }));
    assert!(request_settlements.iter().any(|settlement| {
        settlement.request_settlement_id == 8301012
            && settlement.request_id == 610012
            && settlement.account_id == 7001001
    }));
    assert!(request_settlements.iter().any(|settlement| {
        settlement.request_settlement_id == 8301013
            && settlement.request_id == 610013
            && settlement.account_id == 7001001
    }));
    assert!(request_settlements.iter().any(|settlement| {
        settlement.request_settlement_id == 8301018
            && settlement.request_id == 610018
            && settlement.account_id == 7001001
    }));
    assert!(request_settlements.iter().any(|settlement| {
        settlement.request_settlement_id == 8301019
            && settlement.request_id == 610019
            && settlement.account_id == 7001001
    }));
    assert!(global_ledger.iter().any(|entry| {
        entry.entry.ledger_entry_id == 8201005
            && entry.entry.request_id == Some(610002)
            && entry.entry.hold_id == Some(8101002)
    }));
    assert!(global_ledger.iter().any(|entry| {
        entry.entry.ledger_entry_id == 8201006
            && entry.entry.request_id == Some(610003)
            && entry.entry.hold_id == Some(8101003)
    }));
    assert!(global_ledger.iter().any(|entry| {
        entry.entry.ledger_entry_id == 8201011
            && entry.entry.request_id == Some(610007)
            && entry.entry.hold_id == Some(8101007)
    }));
    assert!(global_ledger.iter().any(|entry| {
        entry.entry.ledger_entry_id == 8201016
            && entry.entry.request_id == Some(610012)
            && entry.entry.hold_id == Some(8101012)
    }));
    assert!(global_ledger.iter().any(|entry| {
        entry.entry.ledger_entry_id == 8201018
            && entry.entry.request_id == Some(610013)
            && entry.entry.hold_id == Some(8101013)
    }));
    assert!(global_ledger.iter().any(|entry| {
        entry.entry.ledger_entry_id == 8201023
            && entry.entry.request_id == Some(610018)
            && entry.entry.hold_id == Some(8101018)
    }));
    assert!(global_ledger.iter().any(|entry| {
        entry.entry.ledger_entry_id == 8201025
            && entry.entry.request_id == Some(610019)
            && entry.entry.hold_id == Some(8101019)
    }));
    assert_eq!(global_balance.active_lot_count, 6);
    assert!((global_balance.available_balance - 22_709_600.0).abs() < f64::EPSILON);
}

#[tokio::test]
async fn build_admin_store_from_config_discovers_repository_dev_bootstrap_profile_data_pack() {
    let bootstrap_root = temp_bootstrap_root("repo-dev-discovery");
    let database_path = bootstrap_root.join("repo-dev-discovery.db");

    let mut config = StandaloneConfig::default();
    config.database_url = sqlite_url_for(database_path);
    config.bootstrap_profile = "dev".to_owned();

    let (store, commercial_billing) = build_admin_store_and_commercial_billing_from_config(&config)
        .await
        .unwrap();
    let tenants = store.list_tenants().await.unwrap();
    let projects = store.list_projects().await.unwrap();
    let provider_accounts = store.list_provider_accounts().await.unwrap();
    let routing_profiles = store.list_routing_profiles().await.unwrap();
    let api_key_groups = store.list_api_key_groups().await.unwrap();
    let payment_methods = store.list_payment_methods().await.unwrap();
    let extension_installations = store.list_extension_installations().await.unwrap();
    let extension_instances = store.list_extension_instances().await.unwrap();
    let admin_users = store.list_admin_users().await.unwrap();
    let portal_users = store.list_portal_users().await.unwrap();
    let gateway_api_keys = store.list_gateway_api_keys().await.unwrap();
    let compiled_routing_snapshots = store.list_compiled_routing_snapshots().await.unwrap();
    let routing_decision_logs = store.list_routing_decision_logs().await.unwrap();
    let provider_health_snapshots = store.list_provider_health_snapshots().await.unwrap();
    let commerce_orders = store.list_commerce_orders().await.unwrap();
    let commerce_payment_events = store.list_commerce_payment_events().await.unwrap();
    let billing_events = store.list_billing_events().await.unwrap();
    let coupon_templates = store.list_coupon_template_records().await.unwrap();
    let coupon_codes = store.list_coupon_code_records().await.unwrap();
    let pricing_plans = commercial_billing
        .list_pricing_plan_records()
        .await
        .unwrap();

    assert!(tenants
        .iter()
        .any(|tenant| tenant.id == "tenant_local_demo"));
    assert!(projects
        .iter()
        .any(|project| project.id == "project_local_demo"));
    assert!(provider_accounts.iter().any(|account| {
        account.provider_account_id == "acct-provider-ernie-official-primary"
            && account.provider_id == "provider-ernie-official"
            && account.execution_instance_id == "provider-ernie-official"
            && account.enabled
    }));
    assert!(provider_accounts.iter().any(|account| {
        account.provider_account_id == "acct-provider-minimax-official-primary"
            && account.provider_id == "provider-minimax-official"
            && account.execution_instance_id == "provider-minimax-official"
            && account.enabled
    }));
    assert!(routing_profiles
        .iter()
        .any(|profile| profile.profile_id == "profile-dev-local-first"));
    assert!(routing_profiles
        .iter()
        .any(|profile| profile.profile_id == "profile-global-openai-official"));
    assert!(routing_profiles
        .iter()
        .any(|profile| profile.profile_id == "profile-global-claude-official"));
    assert!(routing_profiles
        .iter()
        .any(|profile| profile.profile_id == "profile-global-gemini-official"));
    assert!(coupon_codes.iter().any(|code| code.code_value == "LAUNCH100"));
    assert!(coupon_codes.iter().any(|code| code.code_value == "DEV50"));
    assert!(coupon_codes
        .iter()
        .any(|code| code.code_value == "PARTNER30"));
    assert!(coupon_codes
        .iter()
        .any(|code| code.code_value == "GLABAPAC3000"));
    assert!(api_key_groups.iter().any(|group| {
        group.group_id == "group-local-demo-sandbox"
            && group.default_routing_profile_id.as_deref() == Some("profile-dev-local-first")
    }));
    assert!(payment_methods
        .iter()
        .any(|method| method.payment_method_id == "payment-stripe-test"));
    assert!(payment_methods
        .iter()
        .any(|method| method.payment_method_id == "payment-bank-transfer-manual"));
    assert!(extension_installations
        .iter()
        .any(|installation| { installation.installation_id == "installation-openrouter-builtin" }));
    assert!(extension_installations.iter().any(|installation| {
        installation.installation_id == "installation-anthropic-official-builtin"
    }));
    assert!(extension_installations.iter().any(|installation| {
        installation.installation_id == "installation-gemini-official-builtin"
    }));
    assert!(extension_instances
        .iter()
        .any(|instance| { instance.instance_id == "provider-openrouter-main" }));
    assert!(extension_instances
        .iter()
        .any(|instance| { instance.instance_id == "provider-anthropic-official" }));
    assert!(extension_instances
        .iter()
        .any(|instance| { instance.instance_id == "provider-gemini-official" }));
    assert!(admin_users
        .iter()
        .any(|user| user.email == "admin@sdkwork.local"));
    assert!(portal_users
        .iter()
        .any(|user| user.email == "portal@sdkwork.local"));
    assert!(gateway_api_keys
        .iter()
        .any(|record| { record.api_key_group_id.as_deref() == Some("group-local-demo-live") }));
    assert!(compiled_routing_snapshots
        .iter()
        .any(|snapshot| snapshot.snapshot_id == "snapshot-dev-local-sandbox"));
    assert!(routing_decision_logs
        .iter()
        .any(|decision| decision.decision_id == "decision-dev-partner-staging"));
    assert!(provider_health_snapshots.iter().any(|snapshot| {
        snapshot.provider_id == "provider-ollama-local"
            && snapshot.instance_id.as_deref() == Some("provider-ollama-local")
    }));
    assert!(commerce_orders
        .iter()
        .any(|order| order.order_id == "order-local-demo-sandbox-pack"));
    assert!(commerce_orders
        .iter()
        .any(|order| order.order_id == "order-partner-demo-growth"));
    assert!(commerce_orders.iter().any(|order| {
        order.order_id == "order-global-official-openai-2026"
            && order.pricing_plan_id.as_deref() == Some("global-official-direct-retail")
    }));
    assert!(commerce_orders.iter().any(|order| {
        order.order_id == "order-edge-local-ollama-2026"
            && order.pricing_plan_id.as_deref() == Some("edge-local-commercial")
            && order.payment_method_id.as_deref() == Some("payment-bank-transfer-manual")
    }));
    assert!(commerce_payment_events
        .iter()
        .any(|event| event.payment_event_id == "payment-event-local-demo-sandbox"));
    assert!(commerce_payment_events
        .iter()
        .any(|event| event.payment_event_id == "payment-event-global-official-openai-2026"));
    assert!(commerce_payment_events
        .iter()
        .any(|event| event.payment_event_id == "payment-event-edge-local-ollama-2026"));
    assert!(billing_events
        .iter()
        .any(|event| event.event_id == "billing-dev-local-sandbox"));
    assert!(billing_events
        .iter()
        .any(|event| event.event_id == "billing-dev-partner-staging"));
    assert!(coupon_templates
        .iter()
        .any(|template| template.template_key == "dev-sandbox-credit-50"));
    assert!(coupon_templates
        .iter()
        .any(|template| template.template_key == "official-direct-credit-100"));
    assert!(coupon_templates
        .iter()
        .any(|template| template.template_key == "edge-local-credit-20"));
    assert!(pricing_plans
        .iter()
        .any(|plan| plan.plan_code == "global-default-commercial"));
    assert!(pricing_plans
        .iter()
        .any(|plan| plan.plan_code == "global-official-direct-cost"));
    assert!(pricing_plans
        .iter()
        .any(|plan| plan.plan_code == "china-official-direct-cost"));
    assert!(pricing_plans
        .iter()
        .any(|plan| plan.plan_code == "global-marketplace-proxy-cost"));
    assert!(pricing_plans
        .iter()
        .any(|plan| plan.plan_code == "local-edge-infra-cost"));
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_provider_channel_binding_without_active_provider_model_coverage(
) {
    let bootstrap_root = temp_bootstrap_root("profile-pack-invalid-provider-channel-binding-model");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("channels").join("default.json"),
        &serde_json::json!([
            { "id": "openrouter", "name": "OpenRouter" },
            { "id": "siliconflow", "name": "SiliconFlow" },
            { "id": "ollama", "name": "Ollama" },
            { "id": "xai", "name": "xAI" }
        ]),
    );
    write_json(
        &bootstrap_root.join("providers").join("default.json"),
        &serde_json::json!([
            {
                "id": "provider-openrouter-main",
                "channel_id": "openrouter",
                "extension_id": "sdkwork.provider.openrouter",
                "adapter_kind": "openrouter",
                "protocol_kind": "openai",
                "base_url": "https://openrouter.ai/api/v1",
                "display_name": "OpenRouter Main",
                "channel_bindings": [
                    { "provider_id": "provider-openrouter-main", "channel_id": "openrouter", "is_primary": true },
                    { "provider_id": "provider-openrouter-main", "channel_id": "xai", "is_primary": false }
                ]
            },
            {
                "id": "provider-siliconflow-main",
                "channel_id": "openrouter",
                "extension_id": "sdkwork.provider.siliconflow",
                "adapter_kind": "openai-compatible",
                "protocol_kind": "openai",
                "base_url": "https://api.siliconflow.cn/v1",
                "display_name": "SiliconFlow Main",
                "channel_bindings": [{ "provider_id": "provider-siliconflow-main", "channel_id": "siliconflow", "is_primary": true }]
            },
            {
                "id": "provider-ollama-local",
                "channel_id": "ollama",
                "extension_id": "sdkwork.provider.ollama",
                "adapter_kind": "ollama",
                "protocol_kind": "custom",
                "base_url": "http://127.0.0.1:11434",
                "display_name": "Ollama Local",
                "channel_bindings": [{ "provider_id": "provider-ollama-local", "channel_id": "ollama", "is_primary": true }]
            }
        ]),
    );
    write_json(
        &bootstrap_root.join("models").join("default.json"),
        &serde_json::json!([
            {
                "external_name": "gpt-4.1",
                "provider_id": "provider-openrouter-main",
                "capabilities": ["responses", "chat_completions"],
                "streaming": true,
                "context_window": 128000
            },
            {
                "external_name": "deepseek-chat",
                "provider_id": "provider-openrouter-main",
                "capabilities": ["responses", "chat_completions"],
                "streaming": true,
                "context_window": 65536
            },
            {
                "external_name": "grok-4",
                "provider_id": "provider-openrouter-main",
                "capabilities": ["responses", "chat_completions"],
                "streaming": true,
                "context_window": 131072
            },
            {
                "external_name": "qwen-plus-latest",
                "provider_id": "provider-siliconflow-main",
                "capabilities": ["responses", "chat_completions"],
                "streaming": true,
                "context_window": 131072
            },
            {
                "external_name": "llama3.2:latest",
                "provider_id": "provider-ollama-local",
                "capabilities": ["responses", "chat_completions"],
                "streaming": true,
                "context_window": 8192
            }
        ]),
    );
    write_json(
        &bootstrap_root.join("channel-models").join("default.json"),
        &serde_json::json!([
            {
                "channel_id": "openrouter",
                "model_id": "gpt-4.1",
                "model_display_name": "GPT-4.1",
                "capabilities": ["responses", "chat_completions"],
                "streaming": true,
                "context_window": 128000,
                "description": "OpenRouter GPT-4.1 default"
            },
            {
                "channel_id": "openrouter",
                "model_id": "deepseek-chat",
                "model_display_name": "DeepSeek Chat",
                "capabilities": ["responses", "chat_completions"],
                "streaming": true,
                "context_window": 65536,
                "description": "OpenRouter DeepSeek chat default"
            },
            {
                "channel_id": "xai",
                "model_id": "grok-4",
                "model_display_name": "Grok 4",
                "capabilities": ["responses", "chat_completions"],
                "streaming": true,
                "context_window": 131072,
                "description": "xAI coverage without provider-model backing"
            },
            {
                "channel_id": "siliconflow",
                "model_id": "qwen-plus-latest",
                "model_display_name": "Qwen Plus Latest",
                "capabilities": ["responses", "chat_completions"],
                "streaming": true,
                "context_window": 131072,
                "description": "SiliconFlow Qwen default"
            },
            {
                "channel_id": "ollama",
                "model_id": "llama3.2:latest",
                "model_display_name": "Llama 3.2 Latest",
                "capabilities": ["responses", "chat_completions"],
                "streaming": true,
                "context_window": 8192,
                "description": "Ollama local default"
            }
        ]),
    );
    write_json(
        &bootstrap_root.join("model-prices").join("default.json"),
        &serde_json::json!([
            {
                "channel_id": "openrouter",
                "model_id": "gpt-4.1",
                "proxy_provider_id": "provider-openrouter-main",
                "currency_code": "USD",
                "price_unit": "per_1m_tokens",
                "input_price": 2.0,
                "output_price": 8.0,
                "cache_read_price": 0.0,
                "cache_write_price": 0.0,
                "request_price": 0.0,
                "price_source_kind": "proxy",
                "is_active": true
            },
            {
                "channel_id": "openrouter",
                "model_id": "deepseek-chat",
                "proxy_provider_id": "provider-openrouter-main",
                "currency_code": "USD",
                "price_unit": "per_1m_tokens",
                "input_price": 0.27,
                "output_price": 1.1,
                "cache_read_price": 0.0,
                "cache_write_price": 0.0,
                "request_price": 0.0,
                "price_source_kind": "proxy",
                "is_active": true
            },
            {
                "channel_id": "xai",
                "model_id": "grok-4",
                "proxy_provider_id": "provider-openrouter-main",
                "currency_code": "USD",
                "price_unit": "per_1m_tokens",
                "input_price": 3.0,
                "output_price": 15.0,
                "cache_read_price": 0.0,
                "cache_write_price": 0.0,
                "request_price": 0.0,
                "price_source_kind": "proxy",
                "is_active": true
            },
            {
                "channel_id": "siliconflow",
                "model_id": "qwen-plus-latest",
                "proxy_provider_id": "provider-siliconflow-main",
                "currency_code": "USD",
                "price_unit": "per_1m_tokens",
                "input_price": 0.4,
                "output_price": 1.2,
                "cache_read_price": 0.0,
                "cache_write_price": 0.0,
                "request_price": 0.0,
                "price_source_kind": "proxy",
                "is_active": true
            }
        ]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!("bootstrap should reject non-primary provider channel bindings without provider-model coverage"),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("provider-openrouter-main"), "{error}");
    assert!(error.contains("xai"), "{error}");
    assert!(error.contains("provider model"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_provider_channel_binding_without_active_price_coverage(
) {
    let bootstrap_root = temp_bootstrap_root("profile-pack-invalid-provider-channel-binding-price");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("profiles").join("dev.json"),
        &serde_json::json!({
            "profile_id": "dev",
            "description": "development bootstrap pack",
            "channels": ["channels/default.json"],
            "providers": ["providers/default.json"],
            "official_provider_configs": ["official-provider-configs/default.json"],
            "provider_accounts": ["provider-accounts/default.json"],
            "models": ["models/default.json"],
            "channel_models": ["channel-models/default.json"],
            "provider_models": ["provider-models/default.json"],
            "model_prices": ["model-prices/default.json"],
            "tenants": ["tenants/default.json"],
            "projects": ["projects/default.json"],
            "identities": ["identities/default.json"],
            "extensions": ["extensions/default.json"],
            "service_runtime_nodes": ["service-runtime-nodes/default.json"],
            "extension_runtime_rollouts": ["extension-runtime-rollouts/default.json"],
            "standalone_config_rollouts": ["standalone-config-rollouts/default.json"],
            "routing": ["routing/default.json"],
            "api_key_groups": ["api-key-groups/default.json"],
            "observability": ["observability/default.json"],
            "quota_policies": ["quota-policies/default.json"],
            "pricing": ["pricing/default.json"],
            "accounts": ["accounts/default.json"],
            "account_benefit_lots": ["account-benefit-lots/default.json"],
            "account_holds": ["account-holds/default.json"],
            "account_ledger": ["account-ledger/default.json"],
            "request_metering": ["request-metering/default.json"],
            "request_settlements": ["request-settlements/default.json"],
            "account_reconciliation": ["account-reconciliation/default.json"],
            "payment_methods": ["payment-methods/default.json"],
            "marketing": ["marketing/default.json"],
            "commerce": ["commerce/default.json"],
            "billing": ["billing/default.json"],
            "jobs": ["jobs/default.json", "jobs/dev.json"]
        }),
    );
    write_json(
        &bootstrap_root.join("channels").join("default.json"),
        &serde_json::json!([
            { "id": "openrouter", "name": "OpenRouter" },
            { "id": "siliconflow", "name": "SiliconFlow" },
            { "id": "ollama", "name": "Ollama" },
            { "id": "xai", "name": "xAI" }
        ]),
    );
    write_json(
        &bootstrap_root.join("providers").join("default.json"),
        &serde_json::json!([
            {
                "id": "provider-openrouter-main",
                "channel_id": "openrouter",
                "extension_id": "sdkwork.provider.openrouter",
                "adapter_kind": "openrouter",
                "protocol_kind": "openai",
                "base_url": "https://openrouter.ai/api/v1",
                "display_name": "OpenRouter Main",
                "channel_bindings": [
                    { "provider_id": "provider-openrouter-main", "channel_id": "openrouter", "is_primary": true },
                    { "provider_id": "provider-openrouter-main", "channel_id": "xai", "is_primary": false }
                ]
            },
            {
                "id": "provider-siliconflow-main",
                "channel_id": "siliconflow",
                "extension_id": "sdkwork.provider.siliconflow",
                "adapter_kind": "openai-compatible",
                "protocol_kind": "openai",
                "base_url": "https://api.siliconflow.cn/v1",
                "display_name": "SiliconFlow Main",
                "channel_bindings": [{ "provider_id": "provider-siliconflow-main", "channel_id": "siliconflow", "is_primary": true }]
            },
            {
                "id": "provider-ollama-local",
                "channel_id": "ollama",
                "extension_id": "sdkwork.provider.ollama",
                "adapter_kind": "ollama",
                "protocol_kind": "custom",
                "base_url": "http://127.0.0.1:11434",
                "display_name": "Ollama Local",
                "channel_bindings": [{ "provider_id": "provider-ollama-local", "channel_id": "ollama", "is_primary": true }]
            }
        ]),
    );
    write_json(
        &bootstrap_root.join("channel-models").join("default.json"),
        &serde_json::json!([
            {
                "channel_id": "openrouter",
                "model_id": "gpt-4.1",
                "model_display_name": "GPT-4.1",
                "capabilities": ["responses", "chat_completions"],
                "streaming": true,
                "context_window": 128000,
                "description": "OpenRouter GPT-4.1 default"
            },
            {
                "channel_id": "openrouter",
                "model_id": "deepseek-chat",
                "model_display_name": "DeepSeek Chat",
                "capabilities": ["responses", "chat_completions"],
                "streaming": true,
                "context_window": 65536,
                "description": "OpenRouter DeepSeek chat default"
            },
            {
                "channel_id": "xai",
                "model_id": "grok-4",
                "model_display_name": "Grok 4",
                "capabilities": ["responses", "chat_completions"],
                "streaming": true,
                "context_window": 131072,
                "description": "xAI coverage without price backing"
            },
            {
                "channel_id": "siliconflow",
                "model_id": "qwen-plus-latest",
                "model_display_name": "Qwen Plus Latest",
                "capabilities": ["responses", "chat_completions"],
                "streaming": true,
                "context_window": 131072,
                "description": "SiliconFlow Qwen default"
            },
            {
                "channel_id": "ollama",
                "model_id": "llama3.2:latest",
                "model_display_name": "Llama 3.2 Latest",
                "capabilities": ["responses", "chat_completions"],
                "streaming": true,
                "context_window": 8192,
                "description": "Ollama local default"
            }
        ]),
    );
    write_json(
        &bootstrap_root.join("provider-models").join("default.json"),
        &serde_json::json!([
            {
                "proxy_provider_id": "provider-openrouter-main",
                "channel_id": "xai",
                "model_id": "grok-4",
                "provider_model_id": "x-ai/grok-4",
                "provider_model_family": "grok",
                "capabilities": ["responses", "chat_completions"],
                "streaming": true,
                "context_window": 131072,
                "max_output_tokens": 32768,
                "supports_prompt_caching": false,
                "supports_reasoning_usage": true,
                "supports_tool_usage_metrics": true,
                "is_default_route": true,
                "is_active": true
            }
        ]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject non-primary provider channel bindings without price coverage"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("provider-openrouter-main"), "{error}");
    assert!(error.contains("xai"), "{error}");
    assert!(error.contains("price"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_routing_profile_default_provider_without_enabled_account(
) {
    let bootstrap_root = temp_bootstrap_root("profile-pack-invalid-default-provider-account");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root
            .join("provider-accounts")
            .join("default.json"),
        &serde_json::json!([
            {
                "provider_account_id": "acct-openrouter-default",
                "provider_id": "provider-openrouter-main",
                "display_name": "OpenRouter Default",
                "account_kind": "api_key",
                "owner_scope": "platform",
                "owner_tenant_id": null,
                "execution_instance_id": "provider-openrouter-main",
                "base_url_override": "https://openrouter.ai/api/v1",
                "region": "global",
                "priority": 100,
                "weight": 10,
                "enabled": true,
                "routing_tags": ["default", "global"],
                "health_score_hint": null,
                "latency_ms_hint": null,
                "cost_hint": null,
                "success_rate_hint": null,
                "throughput_hint": null,
                "max_concurrency": null,
                "daily_budget": null,
                "notes": "bootstrap default account"
            }
        ]),
    );
    write_json(
        &bootstrap_root.join("routing").join("default.json"),
        &serde_json::json!({
            "profiles": [
                {
                    "profile_id": "profile-global-balanced",
                    "tenant_id": "tenant_local_demo",
                    "project_id": "project_local_demo",
                    "name": "Global Balanced",
                    "slug": "global-balanced",
                    "description": "Balanced multi-provider routing",
                    "active": true,
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-openrouter-main",
                        "provider-siliconflow-main",
                        "provider-ollama-local"
                    ],
                    "default_provider_id": "provider-siliconflow-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "preferred_region": "global",
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ],
            "policies": [
                {
                    "policy_id": "policy-default-responses",
                    "capability": "responses",
                    "model_pattern": "*",
                    "enabled": true,
                    "priority": 100,
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-openrouter-main",
                        "provider-siliconflow-main",
                        "provider-ollama-local"
                    ],
                    "default_provider_id": "provider-siliconflow-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "execution_failover_enabled": true,
                    "upstream_retry_max_attempts": 3,
                    "upstream_retry_base_delay_ms": 250,
                    "upstream_retry_max_delay_ms": 2000
                }
            ],
            "project_preferences": [
                {
                    "project_id": "project_local_demo",
                    "preset_id": "profile-global-balanced",
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-openrouter-main",
                        "provider-siliconflow-main"
                    ],
                    "default_provider_id": "provider-siliconflow-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "preferred_region": "global",
                    "updated_at_ms": 1710000000000u64
                }
            ]
        }),
    );
    write_json(
        &bootstrap_root.join("model-prices").join("default.json"),
        &model_prices_without_siliconflow_fixture(),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!("bootstrap should reject routing profiles whose default provider has no enabled account"),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("profile-global-balanced"), "{error}");
    assert!(error.contains("provider-siliconflow-main"), "{error}");
    assert!(error.contains("provider account"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_routing_candidates_without_enabled_account(
) {
    let bootstrap_root = temp_bootstrap_root("profile-pack-invalid-routed-provider-account");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root
            .join("provider-accounts")
            .join("default.json"),
        &serde_json::json!([
            {
                "provider_account_id": "acct-openrouter-default",
                "provider_id": "provider-openrouter-main",
                "display_name": "OpenRouter Default",
                "account_kind": "api_key",
                "owner_scope": "platform",
                "owner_tenant_id": null,
                "execution_instance_id": "provider-openrouter-main",
                "base_url_override": "https://openrouter.ai/api/v1",
                "region": "global",
                "priority": 100,
                "weight": 10,
                "enabled": true,
                "routing_tags": ["default", "global"],
                "health_score_hint": null,
                "latency_ms_hint": null,
                "cost_hint": null,
                "success_rate_hint": null,
                "throughput_hint": null,
                "max_concurrency": null,
                "daily_budget": null,
                "notes": "bootstrap default account"
            }
        ]),
    );
    write_json(
        &bootstrap_root.join("model-prices").join("default.json"),
        &model_prices_without_siliconflow_fixture(),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => {
            panic!("bootstrap should reject routed providers whose candidate list has no enabled account")
        }
        Err(error) => error.to_string(),
    };

    assert!(error.contains("profile-global-balanced"), "{error}");
    assert!(error.contains("provider-siliconflow-main"), "{error}");
    assert!(error.contains("provider account"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_project_preferences_preset_from_other_workspace() {
    let bootstrap_root = temp_bootstrap_root("profile-pack-invalid-project-preference-preset");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("tenants").join("default.json"),
        &serde_json::json!([
            { "id": "tenant_local_demo", "name": "Local Demo Workspace" },
            { "id": "tenant_other_demo", "name": "Other Demo Workspace" }
        ]),
    );
    write_json(
        &bootstrap_root.join("projects").join("default.json"),
        &serde_json::json!([
            { "tenant_id": "tenant_local_demo", "id": "project_local_demo", "name": "default" },
            { "tenant_id": "tenant_other_demo", "id": "project_other_demo", "name": "other" }
        ]),
    );
    write_json(
        &bootstrap_root.join("routing").join("default.json"),
        &serde_json::json!({
            "profiles": [
                {
                    "profile_id": "profile-global-balanced",
                    "tenant_id": "tenant_local_demo",
                    "project_id": "project_local_demo",
                    "name": "Global Balanced",
                    "slug": "global-balanced",
                    "description": "Balanced multi-provider routing",
                    "active": true,
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-openrouter-main",
                        "provider-siliconflow-main",
                        "provider-ollama-local"
                    ],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "preferred_region": "global",
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                },
                {
                    "profile_id": "profile-other-workspace",
                    "tenant_id": "tenant_other_demo",
                    "project_id": "project_other_demo",
                    "name": "Other Workspace",
                    "slug": "other-workspace",
                    "description": "Profile for another workspace",
                    "active": true,
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-openrouter-main",
                        "provider-siliconflow-main"
                    ],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 2.8,
                    "max_latency_ms": 7000,
                    "require_healthy": false,
                    "preferred_region": "global",
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ],
            "policies": [
                {
                    "policy_id": "policy-default-responses",
                    "capability": "responses",
                    "model_pattern": "*",
                    "enabled": true,
                    "priority": 100,
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-openai-official"
                    ],
                    "default_provider_id": "provider-openai-official",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "execution_failover_enabled": true,
                    "upstream_retry_max_attempts": 3,
                    "upstream_retry_base_delay_ms": 250,
                    "upstream_retry_max_delay_ms": 2000
                }
            ],
            "project_preferences": [
                {
                    "project_id": "project_local_demo",
                    "preset_id": "profile-other-workspace",
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-openrouter-main",
                        "provider-siliconflow-main"
                    ],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "preferred_region": "global",
                    "updated_at_ms": 1710000000000u64
                }
            ]
        }),
    );
    write_json(
        &bootstrap_root.join("observability").join("default.json"),
        &serde_json::json!({
            "compiled_routing_snapshots": [],
            "routing_decision_logs": [],
            "provider_health_snapshots": []
        }),
    );
    write_json(
        &bootstrap_root.join("observability").join("dev.json"),
        &serde_json::json!({
            "compiled_routing_snapshots": [],
            "routing_decision_logs": [],
            "provider_health_snapshots": []
        }),
    );
    write_json(
        &bootstrap_root.join("billing").join("default.json"),
        &billing_fixture_with_local_metering_support(vec![]),
    );
    write_json(
        &bootstrap_root.join("billing").join("dev.json"),
        &billing_fixture_with_local_metering_support(vec![]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => {
            panic!("bootstrap should reject project preferences preset from another workspace")
        }
        Err(error) => error.to_string(),
    };

    assert!(error.contains("project_local_demo"), "{error}");
    assert!(error.contains("profile-other-workspace"), "{error}");
    assert!(error.contains("workspace"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_enabled_routing_policy_without_any_capability_matched_active_model_price_coverage(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-routing-policy-capability-price-coverage");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("routing").join("default.json"),
        &serde_json::json!({
            "profiles": [
                {
                    "profile_id": "profile-global-balanced",
                    "tenant_id": "tenant_local_demo",
                    "project_id": "project_local_demo",
                    "name": "Global Balanced",
                    "slug": "global-balanced",
                    "description": "Balanced multi-provider routing",
                    "active": true,
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-ollama-local"
                    ],
                    "default_provider_id": "provider-ollama-local",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "preferred_region": "global",
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ],
            "policies": [
                {
                    "policy_id": "policy-invalid-responses-embeddings-only",
                    "capability": "responses",
                    "model_pattern": "nomic-embed-text-*",
                    "enabled": true,
                    "priority": 100,
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-ollama-local"
                    ],
                    "default_provider_id": "provider-ollama-local",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "execution_failover_enabled": true,
                    "upstream_retry_max_attempts": 3,
                    "upstream_retry_base_delay_ms": 250,
                    "upstream_retry_max_delay_ms": 2000
                }
            ],
            "project_preferences": [
                {
                    "project_id": "project_local_demo",
                    "preset_id": "profile-global-balanced",
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-ollama-local"
                    ],
                    "default_provider_id": "provider-ollama-local",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "preferred_region": "global",
                    "updated_at_ms": 1710000000000u64
                }
            ]
        }),
    );
    write_json(
        &bootstrap_root.join("observability").join("default.json"),
        &serde_json::json!({
            "compiled_routing_snapshots": [],
            "routing_decision_logs": [],
            "provider_health_snapshots": []
        }),
    );
    write_json(
        &bootstrap_root.join("billing").join("default.json"),
        &billing_fixture_with_local_metering_support(vec![]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject enabled routing policies without any capability-matched active model price coverage"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("policy-invalid-responses-embeddings-only"), "{error}");
    assert!(error.contains("responses"), "{error}");
    assert!(error.contains("nomic-embed-text-*"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_api_key_group_default_routing_profile_from_other_workspace(
) {
    let bootstrap_root = temp_bootstrap_root("profile-pack-invalid-api-key-group-profile");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("tenants").join("default.json"),
        &serde_json::json!([
            { "id": "tenant_local_demo", "name": "Local Demo Workspace" },
            { "id": "tenant_other_demo", "name": "Other Demo Workspace" }
        ]),
    );
    write_json(
        &bootstrap_root.join("projects").join("default.json"),
        &serde_json::json!([
            { "tenant_id": "tenant_local_demo", "id": "project_local_demo", "name": "default" },
            { "tenant_id": "tenant_other_demo", "id": "project_other_demo", "name": "other" }
        ]),
    );
    write_json(
        &bootstrap_root.join("routing").join("default.json"),
        &serde_json::json!({
            "profiles": [
                {
                    "profile_id": "profile-global-balanced",
                    "tenant_id": "tenant_local_demo",
                    "project_id": "project_local_demo",
                    "name": "Global Balanced",
                    "slug": "global-balanced",
                    "description": "Balanced multi-provider routing",
                    "active": true,
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-openrouter-main",
                        "provider-siliconflow-main",
                        "provider-ollama-local"
                    ],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "preferred_region": "global",
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                },
                {
                    "profile_id": "profile-other-workspace",
                    "tenant_id": "tenant_other_demo",
                    "project_id": "project_other_demo",
                    "name": "Other Workspace",
                    "slug": "other-workspace",
                    "description": "Profile for another workspace",
                    "active": true,
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-openrouter-main",
                        "provider-siliconflow-main"
                    ],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 2.8,
                    "max_latency_ms": 7000,
                    "require_healthy": false,
                    "preferred_region": "global",
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ],
            "policies": [
                {
                    "policy_id": "policy-default-responses",
                    "capability": "responses",
                    "model_pattern": "*",
                    "enabled": true,
                    "priority": 100,
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-openrouter-main",
                        "provider-siliconflow-main",
                        "provider-ollama-local"
                    ],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "execution_failover_enabled": true,
                    "upstream_retry_max_attempts": 3,
                    "upstream_retry_base_delay_ms": 250,
                    "upstream_retry_max_delay_ms": 2000
                }
            ],
            "project_preferences": [
                {
                    "project_id": "project_local_demo",
                    "preset_id": "profile-global-balanced",
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-openrouter-main",
                        "provider-siliconflow-main"
                    ],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "preferred_region": "global",
                    "updated_at_ms": 1710000000000u64
                }
            ]
        }),
    );
    write_json(
        &bootstrap_root.join("api-key-groups").join("default.json"),
        &serde_json::json!([
            {
                "group_id": "group-local-demo-live",
                "tenant_id": "tenant_local_demo",
                "project_id": "project_local_demo",
                "environment": "live",
                "name": "Local Demo Live",
                "slug": "local-demo-live",
                "description": "Default live traffic group for the local demo workspace",
                "color": "#0f766e",
                "default_capability_scope": "responses",
                "default_routing_profile_id": "profile-other-workspace",
                "default_accounting_mode": "platform_credit",
                "active": true,
                "created_at_ms": 1710000000000u64,
                "updated_at_ms": 1710000000000u64
            },
            {
                "group_id": "group-local-demo-sandbox",
                "tenant_id": "tenant_local_demo",
                "project_id": "project_local_demo",
                "environment": "sandbox",
                "name": "Local Demo Sandbox",
                "slug": "local-demo-sandbox",
                "description": "Sandbox traffic group for the local demo workspace",
                "color": "#1d4ed8",
                "default_capability_scope": "responses",
                "default_routing_profile_id": "profile-global-balanced",
                "default_accounting_mode": "platform_credit",
                "active": true,
                "created_at_ms": 1710000000000u64,
                "updated_at_ms": 1710000000000u64
            }
        ]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => {
            panic!("bootstrap should reject api key group default routing profile from another workspace")
        }
        Err(error) => error.to_string(),
    };

    assert!(error.contains("group-local-demo-live"), "{error}");
    assert!(error.contains("profile-other-workspace"), "{error}");
    assert!(error.contains("workspace"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_routing_profile_provider_with_only_foreign_tenant_account(
) {
    let bootstrap_root = temp_bootstrap_root("profile-pack-invalid-tenant-scoped-profile-account");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("profiles").join("dev.json"),
        &serde_json::json!({
            "profile_id": "dev",
            "description": "development bootstrap pack",
            "channels": ["channels/default.json"],
            "providers": ["providers/default.json"],
            "provider_accounts": ["provider-accounts/default.json"],
            "models": ["models/default.json"],
            "channel_models": ["channel-models/default.json"],
            "model_prices": ["model-prices/default.json"],
            "tenants": ["tenants/default.json"],
            "projects": ["projects/default.json"],
            "extensions": ["extensions/default.json"],
            "routing": ["routing/default.json"]
        }),
    );
    write_json(
        &bootstrap_root.join("tenants").join("default.json"),
        &serde_json::json!([
            { "id": "tenant_local_demo", "name": "Local Demo Workspace" },
            { "id": "tenant_other_demo", "name": "Other Demo Workspace" }
        ]),
    );
    write_json(
        &bootstrap_root.join("projects").join("default.json"),
        &serde_json::json!([
            { "tenant_id": "tenant_local_demo", "id": "project_local_demo", "name": "default" },
            { "tenant_id": "tenant_other_demo", "id": "project_other_demo", "name": "other" }
        ]),
    );
    write_json(
        &bootstrap_root
            .join("provider-accounts")
            .join("default.json"),
        &serde_json::json!([
            {
                "provider_account_id": "acct-openrouter-tenant-other",
                "provider_id": "provider-openrouter-main",
                "display_name": "OpenRouter Other Tenant",
                "account_kind": "api_key",
                "owner_scope": "tenant",
                "owner_tenant_id": "tenant_other_demo",
                "execution_instance_id": "provider-openrouter-main",
                "base_url_override": "https://openrouter.ai/api/v1",
                "region": "global",
                "priority": 100,
                "weight": 10,
                "enabled": true,
                "routing_tags": ["tenant", "other"],
                "health_score_hint": null,
                "latency_ms_hint": null,
                "cost_hint": null,
                "success_rate_hint": null,
                "throughput_hint": null,
                "max_concurrency": null,
                "daily_budget": null,
                "notes": "foreign tenant scoped account"
            }
        ]),
    );
    write_json(
        &bootstrap_root.join("routing").join("default.json"),
        &serde_json::json!({
            "profiles": [
                {
                    "profile_id": "profile-global-balanced",
                    "tenant_id": "tenant_local_demo",
                    "project_id": "project_local_demo",
                    "name": "Global Balanced",
                    "slug": "global-balanced",
                    "description": "Balanced multi-provider routing",
                    "active": true,
                    "strategy": "weighted_random",
                    "ordered_provider_ids": ["provider-openrouter-main"],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "preferred_region": "global",
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ],
            "policies": [],
            "project_preferences": []
        }),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject routing profile provider with only a foreign tenant account"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("profile-global-balanced"), "{error}");
    assert!(error.contains("provider-openrouter-main"), "{error}");
    assert!(error.contains("provider account"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_project_preferences_provider_with_only_foreign_tenant_account(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-tenant-scoped-project-preference-account");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("profiles").join("dev.json"),
        &serde_json::json!({
            "profile_id": "dev",
            "description": "development bootstrap pack",
            "channels": ["channels/default.json"],
            "providers": ["providers/default.json"],
            "provider_accounts": ["provider-accounts/default.json"],
            "models": ["models/default.json"],
            "channel_models": ["channel-models/default.json"],
            "model_prices": ["model-prices/default.json"],
            "tenants": ["tenants/default.json"],
            "projects": ["projects/default.json"],
            "extensions": ["extensions/default.json"],
            "routing": ["routing/default.json"]
        }),
    );
    write_json(
        &bootstrap_root.join("tenants").join("default.json"),
        &serde_json::json!([
            { "id": "tenant_local_demo", "name": "Local Demo Workspace" },
            { "id": "tenant_other_demo", "name": "Other Demo Workspace" }
        ]),
    );
    write_json(
        &bootstrap_root.join("projects").join("default.json"),
        &serde_json::json!([
            { "tenant_id": "tenant_local_demo", "id": "project_local_demo", "name": "default" },
            { "tenant_id": "tenant_other_demo", "id": "project_other_demo", "name": "other" }
        ]),
    );
    write_json(
        &bootstrap_root
            .join("provider-accounts")
            .join("default.json"),
        &serde_json::json!([
            {
                "provider_account_id": "acct-openrouter-tenant-other",
                "provider_id": "provider-openrouter-main",
                "display_name": "OpenRouter Other Tenant",
                "account_kind": "api_key",
                "owner_scope": "tenant",
                "owner_tenant_id": "tenant_other_demo",
                "execution_instance_id": "provider-openrouter-main",
                "base_url_override": "https://openrouter.ai/api/v1",
                "region": "global",
                "priority": 100,
                "weight": 10,
                "enabled": true,
                "routing_tags": ["tenant", "other"],
                "health_score_hint": null,
                "latency_ms_hint": null,
                "cost_hint": null,
                "success_rate_hint": null,
                "throughput_hint": null,
                "max_concurrency": null,
                "daily_budget": null,
                "notes": "foreign tenant scoped account"
            }
        ]),
    );
    write_json(
        &bootstrap_root.join("routing").join("default.json"),
        &serde_json::json!({
            "profiles": [],
            "policies": [],
            "project_preferences": [
                {
                    "project_id": "project_local_demo",
                    "preset_id": "",
                    "strategy": "weighted_random",
                    "ordered_provider_ids": ["provider-openrouter-main"],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "preferred_region": "global",
                    "updated_at_ms": 1710000000000u64
                }
            ]
        }),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!("bootstrap should reject project preferences provider with only a foreign tenant account"),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("project_local_demo"), "{error}");
    assert!(error.contains("provider-openrouter-main"), "{error}");
    assert!(error.contains("provider account"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_active_provider_model_without_active_price_coverage(
) {
    let bootstrap_root = temp_bootstrap_root("profile-pack-invalid-provider-model-price-coverage");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("profiles").join("dev.json"),
        &serde_json::json!({
            "profile_id": "dev",
            "description": "development bootstrap pack",
            "channels": ["channels/default.json"],
            "providers": ["providers/default.json"],
            "official_provider_configs": ["official-provider-configs/default.json"],
            "provider_accounts": ["provider-accounts/default.json"],
            "models": ["models/default.json"],
            "channel_models": ["channel-models/default.json"],
            "provider_models": ["provider-models/default.json"],
            "model_prices": ["model-prices/default.json"],
            "tenants": ["tenants/default.json"],
            "projects": ["projects/default.json"],
            "identities": ["identities/default.json"],
            "extensions": ["extensions/default.json"],
            "service_runtime_nodes": ["service-runtime-nodes/default.json"],
            "extension_runtime_rollouts": ["extension-runtime-rollouts/default.json"],
            "standalone_config_rollouts": ["standalone-config-rollouts/default.json"],
            "routing": ["routing/default.json"],
            "api_key_groups": ["api-key-groups/default.json"],
            "observability": ["observability/default.json"],
            "quota_policies": ["quota-policies/default.json"],
            "pricing": ["pricing/default.json"],
            "accounts": ["accounts/default.json"],
            "account_benefit_lots": ["account-benefit-lots/default.json"],
            "account_holds": ["account-holds/default.json"],
            "account_ledger": ["account-ledger/default.json"],
            "request_metering": ["request-metering/default.json"],
            "request_settlements": ["request-settlements/default.json"],
            "account_reconciliation": ["account-reconciliation/default.json"],
            "payment_methods": ["payment-methods/default.json"],
            "marketing": ["marketing/default.json"],
            "commerce": ["commerce/default.json"],
            "billing": ["billing/default.json"],
            "jobs": ["jobs/default.json", "jobs/dev.json"]
        }),
    );
    write_json(
        &bootstrap_root.join("provider-models").join("default.json"),
        &serde_json::json!([
            {
                "proxy_provider_id": "provider-openrouter-main",
                "channel_id": "openrouter",
                "model_id": "deepseek-chat",
                "provider_model_id": "openrouter/deepseek-chat",
                "provider_model_family": "deepseek",
                "capabilities": ["responses", "chat_completions"],
                "streaming": true,
                "context_window": 65536,
                "max_output_tokens": 8192,
                "supports_prompt_caching": false,
                "supports_reasoning_usage": true,
                "supports_tool_usage_metrics": true,
                "is_default_route": false,
                "is_active": true
            }
        ]),
    );
    write_json(
        &bootstrap_root.join("model-prices").join("default.json"),
        &serde_json::json!([
            {
                "channel_id": "openrouter",
                "model_id": "gpt-4.1",
                "proxy_provider_id": "provider-openrouter-main",
                "currency_code": "USD",
                "price_unit": "per_1m_tokens",
                "input_price": 2.0,
                "output_price": 8.0,
                "cache_read_price": 0.0,
                "cache_write_price": 0.0,
                "request_price": 0.0,
                "price_source_kind": "proxy",
                "is_active": true
            },
            {
                "channel_id": "siliconflow",
                "model_id": "qwen-plus-latest",
                "proxy_provider_id": "provider-siliconflow-main",
                "currency_code": "USD",
                "price_unit": "per_1m_tokens",
                "input_price": 0.4,
                "output_price": 1.2,
                "cache_read_price": 0.0,
                "cache_write_price": 0.0,
                "request_price": 0.0,
                "price_source_kind": "proxy",
                "is_active": true
            }
        ]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject active provider models without active model price coverage"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("provider-openrouter-main"), "{error}");
    assert!(error.contains("deepseek-chat"), "{error}");
    assert!(error.contains("price"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_model_price_with_invalid_source_kind() {
    let bootstrap_root = temp_bootstrap_root("profile-pack-invalid-model-price-source-kind");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("model-prices").join("default.json"),
        &serde_json::json!([
            {
                "channel_id": "openrouter",
                "model_id": "gpt-4.1",
                "proxy_provider_id": "provider-openrouter-main",
                "currency_code": "USD",
                "price_unit": "per_1m_tokens",
                "input_price": 2.0,
                "output_price": 8.0,
                "cache_read_price": 0.0,
                "cache_write_price": 0.0,
                "request_price": 0.0,
                "price_source_kind": "proxy",
                "is_active": true
            },
            {
                "channel_id": "openrouter",
                "model_id": "deepseek-chat",
                "proxy_provider_id": "provider-openrouter-main",
                "currency_code": "USD",
                "price_unit": "per_1m_tokens",
                "input_price": 0.27,
                "output_price": 1.1,
                "cache_read_price": 0.0,
                "cache_write_price": 0.0,
                "request_price": 0.0,
                "price_source_kind": "mystery",
                "is_active": true
            }
        ]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!("bootstrap should reject unsupported model price source kinds"),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("price_source_kind"), "{error}");
    assert!(error.contains("mystery"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_model_price_with_duplicate_pricing_tiers()
{
    let bootstrap_root = temp_bootstrap_root("profile-pack-invalid-model-price-duplicate-tiers");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("model-prices").join("default.json"),
        &serde_json::json!([
            {
                "channel_id": "openrouter",
                "model_id": "gpt-4.1",
                "proxy_provider_id": "provider-openrouter-main",
                "currency_code": "USD",
                "price_unit": "per_1m_tokens",
                "input_price": 2.0,
                "output_price": 8.0,
                "cache_read_price": 0.0,
                "cache_write_price": 0.0,
                "request_price": 0.0,
                "price_source_kind": "proxy",
                "is_active": true
            },
            {
                "channel_id": "openrouter",
                "model_id": "deepseek-chat",
                "proxy_provider_id": "provider-openrouter-main",
                "currency_code": "USD",
                "price_unit": "per_1m_tokens",
                "input_price": 0.27,
                "output_price": 1.1,
                "cache_read_price": 0.0,
                "cache_write_price": 0.0,
                "request_price": 0.0,
                "price_source_kind": "proxy",
                "billing_notes": "duplicate tiers should be rejected",
                "pricing_tiers": [
                    {
                        "tier_id": "default",
                        "display_name": "Default",
                        "condition_kind": "default",
                        "currency_code": "USD",
                        "price_unit": "per_1m_tokens",
                        "input_price": 0.27,
                        "output_price": 1.1,
                        "cache_read_price": 0.0,
                        "cache_write_price": 0.0,
                        "request_price": 0.0
                    },
                    {
                        "tier_id": "default",
                        "display_name": "Cached",
                        "condition_kind": "cache",
                        "currency_code": "USD",
                        "price_unit": "per_1m_tokens",
                        "input_price": 0.05,
                        "output_price": 1.1,
                        "cache_read_price": 0.0,
                        "cache_write_price": 0.0,
                        "request_price": 0.0
                    }
                ],
                "is_active": true
            }
        ]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!("bootstrap should reject duplicate model price tiers"),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("pricing_tiers"), "{error}");
    assert!(error.contains("default"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_model_price_with_negative_pricing_tier_value(
) {
    let bootstrap_root = temp_bootstrap_root("profile-pack-invalid-model-price-negative-tier");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("model-prices").join("default.json"),
        &serde_json::json!([
            {
                "channel_id": "openrouter",
                "model_id": "gpt-4.1",
                "proxy_provider_id": "provider-openrouter-main",
                "currency_code": "USD",
                "price_unit": "per_1m_tokens",
                "input_price": 2.0,
                "output_price": 8.0,
                "cache_read_price": 0.0,
                "cache_write_price": 0.0,
                "request_price": 0.0,
                "price_source_kind": "proxy",
                "is_active": true
            },
            {
                "channel_id": "openrouter",
                "model_id": "deepseek-chat",
                "proxy_provider_id": "provider-openrouter-main",
                "currency_code": "USD",
                "price_unit": "per_1m_tokens",
                "input_price": 0.27,
                "output_price": 1.1,
                "cache_read_price": 0.0,
                "cache_write_price": 0.0,
                "request_price": 0.0,
                "price_source_kind": "proxy",
                "billing_notes": "negative tiers should be rejected",
                "pricing_tiers": [
                    {
                        "tier_id": "default",
                        "display_name": "Default",
                        "condition_kind": "default",
                        "currency_code": "USD",
                        "price_unit": "per_1m_tokens",
                        "input_price": -0.01,
                        "output_price": 1.1,
                        "cache_read_price": 0.0,
                        "cache_write_price": 0.0,
                        "request_price": 0.0
                    }
                ],
                "is_active": true
            }
        ]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!("bootstrap should reject negative pricing tier values"),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("pricing_tiers"), "{error}");
    assert!(error.contains("default"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_model_price_channel_outside_provider_bindings(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-model-price-provider-channel-binding");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("models").join("default.json"),
        &serde_json::json!([
            {
                "external_name": "gpt-4.1",
                "provider_id": "provider-openrouter-main",
                "capabilities": ["responses", "chat_completions"],
                "streaming": true,
                "context_window": 128000
            },
            {
                "external_name": "deepseek-chat",
                "provider_id": "provider-openrouter-main",
                "capabilities": ["responses", "chat_completions"],
                "streaming": true,
                "context_window": 65536
            },
            {
                "external_name": "qwen-plus-latest",
                "provider_id": "provider-openrouter-main",
                "capabilities": ["responses", "chat_completions"],
                "streaming": true,
                "context_window": 131072
            },
            {
                "external_name": "qwen-plus-latest",
                "provider_id": "provider-siliconflow-main",
                "capabilities": ["responses", "chat_completions"],
                "streaming": true,
                "context_window": 131072
            },
            {
                "external_name": "llama3.2:latest",
                "provider_id": "provider-ollama-local",
                "capabilities": ["responses", "chat_completions"],
                "streaming": true,
                "context_window": 8192
            }
        ]),
    );
    write_json(
        &bootstrap_root.join("model-prices").join("default.json"),
        &serde_json::json!([
            {
                "channel_id": "openrouter",
                "model_id": "gpt-4.1",
                "proxy_provider_id": "provider-openrouter-main",
                "currency_code": "USD",
                "price_unit": "per_1m_tokens",
                "input_price": 2.0,
                "output_price": 8.0,
                "cache_read_price": 0.0,
                "cache_write_price": 0.0,
                "request_price": 0.0,
                "price_source_kind": "proxy",
                "is_active": true
            },
            {
                "channel_id": "siliconflow",
                "model_id": "qwen-plus-latest",
                "proxy_provider_id": "provider-openrouter-main",
                "currency_code": "USD",
                "price_unit": "per_1m_tokens",
                "input_price": 0.4,
                "output_price": 1.2,
                "cache_read_price": 0.0,
                "cache_write_price": 0.0,
                "request_price": 0.0,
                "price_source_kind": "proxy",
                "is_active": true
            }
        ]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject active model prices whose provider does not bind the declared channel"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("provider-openrouter-main"), "{error}");
    assert!(error.contains("siliconflow"), "{error}");
    assert!(error.contains("channel binding"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_active_model_price_without_executable_provider_account(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-model-price-without-executable-account");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root
            .join("provider-accounts")
            .join("default.json"),
        &provider_accounts_without_siliconflow_fixture(),
    );
    write_json(
        &bootstrap_root.join("routing").join("default.json"),
        &serde_json::json!({
            "profiles": [
                {
                    "profile_id": "profile-global-balanced",
                    "tenant_id": "tenant_local_demo",
                    "project_id": "project_local_demo",
                    "name": "Global Balanced",
                    "slug": "global-balanced",
                    "description": "Balanced multi-provider routing",
                    "active": true,
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-openrouter-main",
                        "provider-siliconflow-main"
                    ],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "preferred_region": "global",
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ],
            "policies": [
                {
                    "policy_id": "policy-default-responses",
                    "capability": "responses",
                    "model_pattern": "*",
                    "enabled": true,
                    "priority": 100,
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-openrouter-main",
                        "provider-siliconflow-main"
                    ],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "execution_failover_enabled": true,
                    "upstream_retry_max_attempts": 3,
                    "upstream_retry_base_delay_ms": 250,
                    "upstream_retry_max_delay_ms": 2000
                }
            ],
            "project_preferences": [
                {
                    "project_id": "project_local_demo",
                    "preset_id": "profile-global-balanced",
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-openrouter-main",
                        "provider-siliconflow-main"
                    ],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "preferred_region": "global",
                    "updated_at_ms": 1710000000000u64
                }
            ]
        }),
    );
    write_json(
        &bootstrap_root.join("observability").join("default.json"),
        &serde_json::json!({
            "compiled_routing_snapshots": [],
            "routing_decision_logs": [],
            "provider_health_snapshots": []
        }),
    );
    write_json(
        &bootstrap_root.join("billing").join("default.json"),
        &billing_fixture_with_local_metering_support(vec![]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject active model prices whose provider has no executable account"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("provider-siliconflow-main"), "{error}");
    assert!(error.contains("qwen-plus-latest"), "{error}");
    assert!(error.contains("provider account"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_extension_instance_without_installation() {
    let bootstrap_root = temp_bootstrap_root("profile-pack-invalid-extension");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root
            .join("provider-accounts")
            .join("default.json"),
        &serde_json::json!([
            {
                "provider_account_id": "acct-openrouter-default",
                "provider_id": "provider-openrouter-main",
                "display_name": "OpenRouter Default",
                "account_kind": "api_key",
                "owner_scope": "platform",
                "owner_tenant_id": null,
                "execution_instance_id": "provider-openrouter-main",
                "base_url_override": "https://openrouter.ai/api/v1",
                "region": "global",
                "priority": 100,
                "weight": 10,
                "enabled": true,
                "routing_tags": ["default", "global"],
                "health_score_hint": null,
                "latency_ms_hint": null,
                "cost_hint": null,
                "success_rate_hint": null,
                "throughput_hint": null,
                "max_concurrency": null,
                "daily_budget": null,
                "notes": "bootstrap default account"
            }
        ]),
    );
    write_json(
        &bootstrap_root.join("profiles").join("dev.json"),
        &serde_json::json!({
            "profile_id": "dev",
            "description": "development bootstrap pack",
            "channels": ["channels/default.json"],
            "providers": ["providers/default.json"],
            "official_provider_configs": ["official-provider-configs/default.json"],
            "provider_accounts": ["provider-accounts/default.json"],
            "models": ["models/default.json"],
            "channel_models": ["channel-models/default.json"],
            "model_prices": ["model-prices/default.json"],
            "tenants": ["tenants/default.json"],
            "projects": ["projects/default.json"],
            "extensions": ["extensions/invalid-missing-installation.json"],
            "routing": ["routing/default.json"],
            "api_key_groups": ["api-key-groups/default.json"],
            "quota_policies": ["quota-policies/default.json"],
            "pricing": ["pricing/default.json"],
            "payment_methods": ["payment-methods/default.json"],
            "marketing": ["marketing/default.json"]
        }),
    );
    write_json(
        &bootstrap_root
            .join("extensions")
            .join("invalid-missing-installation.json"),
        &serde_json::json!({
            "instances": [
                {
                    "instance_id": "provider-openrouter-main",
                    "installation_id": "installation-missing",
                    "extension_id": "sdkwork.provider.openrouter",
                    "enabled": true,
                    "base_url": "https://openrouter.ai/api/v1",
                    "credential_ref": null,
                    "config": { "health_path": "/v1/models" }
                }
            ]
        }),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!("bootstrap should reject orphan extension instance"),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("installation-missing"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_enabled_provider_account_with_disabled_extension_instance(
) {
    let bootstrap_root = temp_bootstrap_root("profile-pack-disabled-account-instance");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("profiles").join("dev.json"),
        &serde_json::json!({
            "profile_id": "dev",
            "description": "development bootstrap pack",
            "channels": ["channels/default.json"],
            "providers": ["providers/default.json"],
            "official_provider_configs": ["official-provider-configs/default.json"],
            "provider_accounts": ["provider-accounts/default.json"],
            "models": ["models/default.json"],
            "channel_models": ["channel-models/default.json"],
            "model_prices": ["model-prices/default.json"],
            "tenants": ["tenants/default.json"],
            "projects": ["projects/default.json"],
            "extensions": ["extensions/disabled-instance.json"],
            "routing": ["routing/default.json"],
            "api_key_groups": ["api-key-groups/default.json"],
            "quota_policies": ["quota-policies/default.json"],
            "pricing": ["pricing/default.json"],
            "payment_methods": ["payment-methods/default.json"],
            "marketing": ["marketing/default.json"]
        }),
    );
    write_json(
        &bootstrap_root
            .join("provider-accounts")
            .join("default.json"),
        &serde_json::json!([
            {
                "provider_account_id": "acct-openrouter-default",
                "provider_id": "provider-openrouter-main",
                "display_name": "OpenRouter Default",
                "account_kind": "api_key",
                "owner_scope": "platform",
                "owner_tenant_id": null,
                "execution_instance_id": "provider-openrouter-main",
                "base_url_override": "https://openrouter.ai/api/v1",
                "region": "global",
                "priority": 100,
                "weight": 10,
                "enabled": true,
                "routing_tags": ["default", "global"],
                "health_score_hint": null,
                "latency_ms_hint": null,
                "cost_hint": null,
                "success_rate_hint": null,
                "throughput_hint": null,
                "max_concurrency": null,
                "daily_budget": null,
                "notes": "bootstrap default account"
            }
        ]),
    );
    write_json(
        &bootstrap_root.join("routing").join("default.json"),
        &serde_json::json!({
            "profiles": [
                {
                    "profile_id": "profile-global-balanced",
                    "tenant_id": "tenant_local_demo",
                    "project_id": "project_local_demo",
                    "name": "Global Balanced",
                    "slug": "global-balanced",
                    "description": "Balanced multi-provider routing",
                    "active": true,
                    "strategy": "weighted_random",
                    "ordered_provider_ids": ["provider-openrouter-main"],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "preferred_region": "global",
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ],
            "policies": [
                {
                    "policy_id": "policy-default-responses",
                    "capability": "responses",
                    "model_pattern": "*",
                    "enabled": true,
                    "priority": 100,
                    "strategy": "weighted_random",
                    "ordered_provider_ids": ["provider-openrouter-main"],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "execution_failover_enabled": true,
                    "upstream_retry_max_attempts": 3,
                    "upstream_retry_base_delay_ms": 250,
                    "upstream_retry_max_delay_ms": 2000
                }
            ],
            "project_preferences": [
                {
                    "project_id": "project_local_demo",
                    "preset_id": "profile-global-balanced",
                    "strategy": "weighted_random",
                    "ordered_provider_ids": ["provider-openrouter-main"],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "preferred_region": "global",
                    "updated_at_ms": 1710000000000u64
                }
            ]
        }),
    );
    write_json(
        &bootstrap_root
            .join("extensions")
            .join("disabled-instance.json"),
        &serde_json::json!({
            "installations": [
                {
                    "installation_id": "installation-openrouter-builtin",
                    "extension_id": "sdkwork.provider.openrouter",
                    "runtime": "builtin",
                    "enabled": true,
                    "entrypoint": null,
                    "config": {
                        "health_path": "/v1/models",
                        "plugin_family": "openrouter"
                    }
                }
            ],
            "instances": [
                {
                    "instance_id": "provider-openrouter-main",
                    "installation_id": "installation-openrouter-builtin",
                    "extension_id": "sdkwork.provider.openrouter",
                    "enabled": false,
                    "base_url": "https://openrouter.ai/api/v1",
                    "credential_ref": null,
                    "config": {
                        "routing_hint": "global-balanced"
                    }
                }
            ]
        }),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => {
            panic!("bootstrap should reject enabled provider account bound to disabled instance")
        }
        Err(error) => error.to_string(),
    };

    assert!(error.contains("acct-openrouter-default"), "{error}");
    assert!(error.contains("provider-openrouter-main"), "{error}");
    assert!(error.contains("disabled"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_enabled_provider_account_with_disabled_installation()
{
    let bootstrap_root = temp_bootstrap_root("profile-pack-disabled-account-installation");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("profiles").join("dev.json"),
        &serde_json::json!({
            "profile_id": "dev",
            "description": "development bootstrap pack",
            "channels": ["channels/default.json"],
            "providers": ["providers/default.json"],
            "official_provider_configs": ["official-provider-configs/default.json"],
            "provider_accounts": ["provider-accounts/default.json"],
            "models": ["models/default.json"],
            "channel_models": ["channel-models/default.json"],
            "model_prices": ["model-prices/default.json"],
            "tenants": ["tenants/default.json"],
            "projects": ["projects/default.json"],
            "extensions": ["extensions/disabled-installation.json"],
            "routing": ["routing/default.json"],
            "api_key_groups": ["api-key-groups/default.json"],
            "quota_policies": ["quota-policies/default.json"],
            "pricing": ["pricing/default.json"],
            "payment_methods": ["payment-methods/default.json"],
            "marketing": ["marketing/default.json"]
        }),
    );
    write_json(
        &bootstrap_root
            .join("provider-accounts")
            .join("default.json"),
        &serde_json::json!([
            {
                "provider_account_id": "acct-openrouter-default",
                "provider_id": "provider-openrouter-main",
                "display_name": "OpenRouter Default",
                "account_kind": "api_key",
                "owner_scope": "platform",
                "owner_tenant_id": null,
                "execution_instance_id": "provider-openrouter-main",
                "base_url_override": "https://openrouter.ai/api/v1",
                "region": "global",
                "priority": 100,
                "weight": 10,
                "enabled": true,
                "routing_tags": ["default", "global"],
                "health_score_hint": null,
                "latency_ms_hint": null,
                "cost_hint": null,
                "success_rate_hint": null,
                "throughput_hint": null,
                "max_concurrency": null,
                "daily_budget": null,
                "notes": "bootstrap default account"
            }
        ]),
    );
    write_json(
        &bootstrap_root.join("routing").join("default.json"),
        &serde_json::json!({
            "profiles": [
                {
                    "profile_id": "profile-global-balanced",
                    "tenant_id": "tenant_local_demo",
                    "project_id": "project_local_demo",
                    "name": "Global Balanced",
                    "slug": "global-balanced",
                    "description": "Balanced multi-provider routing",
                    "active": true,
                    "strategy": "weighted_random",
                    "ordered_provider_ids": ["provider-openrouter-main"],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "preferred_region": "global",
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ],
            "policies": [
                {
                    "policy_id": "policy-default-responses",
                    "capability": "responses",
                    "model_pattern": "*",
                    "enabled": true,
                    "priority": 100,
                    "strategy": "weighted_random",
                    "ordered_provider_ids": ["provider-openrouter-main"],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "execution_failover_enabled": true,
                    "upstream_retry_max_attempts": 3,
                    "upstream_retry_base_delay_ms": 250,
                    "upstream_retry_max_delay_ms": 2000
                }
            ],
            "project_preferences": [
                {
                    "project_id": "project_local_demo",
                    "preset_id": "profile-global-balanced",
                    "strategy": "weighted_random",
                    "ordered_provider_ids": ["provider-openrouter-main"],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "preferred_region": "global",
                    "updated_at_ms": 1710000000000u64
                }
            ]
        }),
    );
    write_json(
        &bootstrap_root
            .join("extensions")
            .join("disabled-installation.json"),
        &serde_json::json!({
            "installations": [
                {
                    "installation_id": "installation-openrouter-builtin",
                    "extension_id": "sdkwork.provider.openrouter",
                    "runtime": "builtin",
                    "enabled": false,
                    "entrypoint": null,
                    "config": {
                        "health_path": "/v1/models",
                        "plugin_family": "openrouter"
                    }
                }
            ],
            "instances": [
                {
                    "instance_id": "provider-openrouter-main",
                    "installation_id": "installation-openrouter-builtin",
                    "extension_id": "sdkwork.provider.openrouter",
                    "enabled": true,
                    "base_url": "https://openrouter.ai/api/v1",
                    "credential_ref": null,
                    "config": {
                        "routing_hint": "global-balanced"
                    }
                }
            ]
        }),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject enabled provider account bound to disabled installation"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("acct-openrouter-default"), "{error}");
    assert!(error.contains("installation-openrouter-builtin"), "{error}");
    assert!(error.contains("disabled"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_extension_runtime_rollout_participant_with_unknown_node(
) {
    let bootstrap_root = temp_bootstrap_root("profile-pack-invalid-runtime-governance");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("profiles").join("dev.json"),
        &serde_json::json!({
            "profile_id": "dev",
            "description": "development bootstrap pack",
            "channels": ["channels/default.json"],
            "providers": ["providers/default.json"],
            "official_provider_configs": ["official-provider-configs/default.json"],
            "provider_accounts": ["provider-accounts/default.json"],
            "extensions": ["extensions/default.json"],
            "service_runtime_nodes": ["service-runtime-nodes/default.json"],
            "extension_runtime_rollouts": ["extension-runtime-rollouts/invalid-missing-node.json"]
        }),
    );
    write_json(
        &bootstrap_root
            .join("extension-runtime-rollouts")
            .join("invalid-missing-node.json"),
        &serde_json::json!({
            "rollouts": [
                {
                    "rollout_id": "rollout-invalid-missing-node",
                    "scope": "instance",
                    "requested_extension_id": null,
                    "requested_instance_id": "provider-openrouter-main",
                    "resolved_extension_id": "sdkwork.provider.openrouter",
                    "created_by": "admin_local_default",
                    "created_at_ms": 1710002100000u64,
                    "deadline_at_ms": 1710002400000u64
                }
            ],
            "participants": [
                {
                    "rollout_id": "rollout-invalid-missing-node",
                    "node_id": "node-missing",
                    "service_kind": "gateway",
                    "status": "pending",
                    "message": "This participant points to a missing runtime node.",
                    "updated_at_ms": 1710002120000u64
                }
            ]
        }),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => {
            panic!("bootstrap should reject rollout participants that reference missing nodes")
        }
        Err(error) => error.to_string(),
    };

    assert!(error.contains("node-missing"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_extension_runtime_rollout_instance_without_executable_provider_account_binding(
) {
    let bootstrap_root = temp_bootstrap_root("profile-pack-invalid-runtime-rollout-provider-binding");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root
            .join("provider-accounts")
            .join("default.json"),
        &provider_accounts_without_siliconflow_fixture(),
    );
    write_json(
        &bootstrap_root.join("routing").join("default.json"),
        &serde_json::json!({
            "profiles": [
                {
                    "profile_id": "profile-global-balanced",
                    "tenant_id": "tenant_local_demo",
                    "project_id": "project_local_demo",
                    "name": "Global Balanced",
                    "slug": "global-balanced",
                    "description": "Balanced multi-provider routing",
                    "active": true,
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-openrouter-main",
                        "provider-siliconflow-main"
                    ],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "preferred_region": "global",
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ],
            "policies": [
                {
                    "policy_id": "policy-default-responses",
                    "capability": "responses",
                    "model_pattern": "*",
                    "enabled": true,
                    "priority": 100,
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-openrouter-main",
                        "provider-siliconflow-main"
                    ],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "execution_failover_enabled": true,
                    "upstream_retry_max_attempts": 3,
                    "upstream_retry_base_delay_ms": 250,
                    "upstream_retry_max_delay_ms": 2000
                }
            ],
            "project_preferences": [
                {
                    "project_id": "project_local_demo",
                    "preset_id": "profile-global-balanced",
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-openrouter-main",
                        "provider-siliconflow-main"
                    ],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "preferred_region": "global",
                    "updated_at_ms": 1710000000000u64
                }
            ]
        }),
    );
    write_json(
        &bootstrap_root.join("observability").join("default.json"),
        &serde_json::json!({
            "compiled_routing_snapshots": [],
            "routing_decision_logs": [],
            "provider_health_snapshots": []
        }),
    );
    write_json(
        &bootstrap_root.join("billing").join("default.json"),
        &billing_fixture_with_local_metering_support(vec![]),
    );
    write_json(
        &bootstrap_root.join("model-prices").join("default.json"),
        &model_prices_without_siliconflow_fixture(),
    );
    write_json(
        &bootstrap_root
            .join("extension-runtime-rollouts")
            .join("default.json"),
        &serde_json::json!({
            "rollouts": [
                {
                    "rollout_id": "rollout-invalid-siliconflow-without-provider-binding",
                    "scope": "instance",
                    "requested_extension_id": null,
                    "requested_instance_id": "provider-siliconflow-main",
                    "resolved_extension_id": "sdkwork.provider.siliconflow",
                    "created_by": "admin_local_default",
                    "created_at_ms": 1710002100000u64,
                    "deadline_at_ms": 1710002400000u64
                }
            ],
            "participants": []
        }),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject instance-scoped extension rollouts without executable provider-account binding"
        ),
        Err(error) => error.to_string(),
    };

    assert!(
        error.contains("rollout-invalid-siliconflow-without-provider-binding"),
        "{error}"
    );
    assert!(error.contains("provider-siliconflow-main"), "{error}");
    assert!(error.contains("provider account"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_gateway_api_key_with_mismatched_group() {
    let bootstrap_root = temp_bootstrap_root("profile-pack-invalid-identity");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("profiles").join("dev.json"),
        &serde_json::json!({
            "profile_id": "dev",
            "description": "development bootstrap pack",
            "channels": ["channels/default.json"],
            "providers": ["providers/default.json"],
            "official_provider_configs": ["official-provider-configs/default.json"],
            "provider_accounts": ["provider-accounts/default.json"],
            "models": ["models/default.json"],
            "channel_models": ["channel-models/default.json"],
            "model_prices": ["model-prices/default.json"],
            "tenants": ["tenants/default.json"],
            "projects": ["projects/default.json"],
            "identities": ["identities/invalid-mismatched-group.json"],
            "extensions": ["extensions/default.json"],
            "routing": ["routing/default.json"],
            "api_key_groups": ["api-key-groups/default.json"],
            "quota_policies": ["quota-policies/default.json"],
            "pricing": ["pricing/default.json"],
            "payment_methods": ["payment-methods/default.json"],
            "marketing": ["marketing/default.json"]
        }),
    );
    write_json(
        &bootstrap_root
            .join("identities")
            .join("invalid-mismatched-group.json"),
        &serde_json::json!({
            "admin_users": [],
            "portal_users": [],
            "gateway_api_keys": [
                {
                    "tenant_id": "tenant_local_demo",
                    "project_id": "project_local_demo",
                    "environment": "sandbox",
                    "hashed_key": "79aac191d0637853cca289ebd01e2ab6949fa915da1bc809ff8349971f3fc71f",
                    "api_key_group_id": "group-local-demo-live",
                    "raw_key": "skw_partner_demo_staging_2026",
                    "label": "Broken group assignment",
                    "notes": "Should fail because the group environment does not match",
                    "created_at_ms": 1710003000000u64,
                    "last_used_at_ms": null,
                    "expires_at_ms": null,
                    "active": true
                }
            ]
        }),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!("bootstrap should reject mismatched gateway api key group"),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("group-local-demo-live"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_observability_decision_with_unknown_provider(
) {
    let bootstrap_root = temp_bootstrap_root("profile-pack-invalid-observability");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("profiles").join("dev.json"),
        &serde_json::json!({
            "profile_id": "dev",
            "description": "development bootstrap pack",
            "channels": ["channels/default.json"],
            "providers": ["providers/default.json"],
            "official_provider_configs": ["official-provider-configs/default.json"],
            "provider_accounts": ["provider-accounts/default.json"],
            "models": ["models/default.json"],
            "channel_models": ["channel-models/default.json"],
            "model_prices": ["model-prices/default.json"],
            "tenants": ["tenants/default.json"],
            "projects": ["projects/default.json"],
            "identities": ["identities/default.json"],
            "extensions": ["extensions/default.json"],
            "service_runtime_nodes": ["service-runtime-nodes/default.json"],
            "extension_runtime_rollouts": ["extension-runtime-rollouts/default.json"],
            "standalone_config_rollouts": ["standalone-config-rollouts/default.json"],
            "routing": ["routing/default.json"],
            "api_key_groups": ["api-key-groups/default.json"],
            "observability": ["observability/invalid-missing-provider.json"],
            "quota_policies": ["quota-policies/default.json"],
            "pricing": ["pricing/default.json"],
            "payment_methods": ["payment-methods/default.json"],
            "marketing": ["marketing/default.json"]
        }),
    );
    write_json(
        &bootstrap_root
            .join("observability")
            .join("invalid-missing-provider.json"),
        &serde_json::json!({
            "compiled_routing_snapshots": [],
            "routing_decision_logs": [
                {
                    "decision_id": "decision-invalid-provider",
                    "decision_source": "gateway",
                    "tenant_id": "tenant_local_demo",
                    "project_id": "project_local_demo",
                    "api_key_group_id": "group-local-demo-live",
                    "capability": "responses",
                    "route_key": "gpt-4.1",
                    "selected_provider_id": "provider-missing",
                    "matched_policy_id": "policy-default-responses",
                    "applied_routing_profile_id": "profile-global-balanced",
                    "compiled_routing_snapshot_id": null,
                    "strategy": "weighted_random",
                    "selection_seed": 7u64,
                    "selection_reason": "broken provider reference",
                    "fallback_reason": null,
                    "requested_region": "global",
                    "slo_applied": false,
                    "slo_degraded": false,
                    "created_at_ms": 1710006000000u64,
                    "assessments": [
                        {
                            "provider_id": "provider-missing",
                            "available": false,
                            "health": "unknown",
                            "policy_rank": 1,
                            "reasons": ["provider not registered"]
                        }
                    ]
                }
            ],
            "provider_health_snapshots": []
        }),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!("bootstrap should reject routing decision logs with unknown providers"),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("provider-missing"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_provider_health_snapshot_without_executable_provider_account_binding(
) {
    let bootstrap_root = temp_bootstrap_root("profile-pack-invalid-provider-health-binding");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root
            .join("provider-accounts")
            .join("default.json"),
        &provider_accounts_without_siliconflow_fixture(),
    );
    write_json(
        &bootstrap_root.join("routing").join("default.json"),
        &routing_fixture_without_siliconflow_candidates(),
    );
    write_json(
        &bootstrap_root.join("observability").join("default.json"),
        &serde_json::json!({
            "compiled_routing_snapshots": [],
            "routing_decision_logs": [],
            "provider_health_snapshots": [
                {
                    "provider_id": "provider-siliconflow-main",
                    "extension_id": "sdkwork.provider.siliconflow",
                    "runtime": "builtin",
                    "observed_at_ms": 1710003002000u64,
                    "instance_id": "provider-siliconflow-main",
                    "running": true,
                    "healthy": true,
                    "message": "Should fail because no executable provider account binds this instance."
                }
            ]
        }),
    );
    write_json(
        &bootstrap_root.join("billing").join("default.json"),
        &billing_fixture_with_local_metering_support(vec![]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject provider health snapshots without executable provider-account binding"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("provider-siliconflow-main"), "{error}");
    assert!(error.contains("provider account"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_builtin_provider_health_snapshot_without_instance_id(
) {
    let error = bootstrap_error_from_profile_pack_override(
        "profile-pack-invalid-provider-health-builtin-missing-instance",
        &serde_json::json!({
            "compiled_routing_snapshots": [],
            "routing_decision_logs": [],
            "provider_health_snapshots": [
                {
                    "provider_id": "provider-openrouter-main",
                    "extension_id": "sdkwork.provider.openrouter",
                    "runtime": "builtin",
                    "observed_at_ms": 1710003002000u64,
                    "instance_id": null,
                    "running": true,
                    "healthy": true,
                    "message": "builtin runtime omitted its bound instance"
                }
            ]
        }),
        &billing_fixture(vec![]),
    )
    .await;

    assert!(error.contains("provider-openrouter-main"), "{error}");
    assert!(error.contains("builtin"), "{error}");
    assert!(error.contains("instance"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_passthrough_provider_health_snapshot_with_instance_id(
) {
    let error = bootstrap_error_from_profile_pack_override(
        "profile-pack-invalid-provider-health-passthrough-instance",
        &serde_json::json!({
            "compiled_routing_snapshots": [],
            "routing_decision_logs": [],
            "provider_health_snapshots": [
                {
                    "provider_id": "provider-openrouter-main",
                    "extension_id": "sdkwork.provider.openrouter",
                    "runtime": "passthrough",
                    "observed_at_ms": 1710003002000u64,
                    "instance_id": "provider-openrouter-main",
                    "running": true,
                    "healthy": true,
                    "message": "passthrough runtime should not bind an executable instance"
                }
            ]
        }),
        &billing_fixture(vec![]),
    )
    .await;

    assert!(error.contains("provider-openrouter-main"), "{error}");
    assert!(error.contains("passthrough"), "{error}");
    assert!(error.contains("instance"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_healthy_provider_health_snapshot_that_is_not_running(
) {
    let error = bootstrap_error_from_profile_pack_override(
        "profile-pack-invalid-provider-health-healthy-not-running",
        &serde_json::json!({
            "compiled_routing_snapshots": [],
            "routing_decision_logs": [],
            "provider_health_snapshots": [
                {
                    "provider_id": "provider-openrouter-main",
                    "extension_id": "sdkwork.provider.openrouter",
                    "runtime": "builtin",
                    "observed_at_ms": 1710003002000u64,
                    "instance_id": "provider-openrouter-main",
                    "running": false,
                    "healthy": true,
                    "message": "healthy snapshots must represent a running runtime"
                }
            ]
        }),
        &billing_fixture(vec![]),
    )
    .await;

    assert!(error.contains("provider-openrouter-main"), "{error}");
    assert!(error.contains("healthy"), "{error}");
    assert!(error.contains("running"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_healthy_provider_health_snapshot_without_message(
) {
    let error = bootstrap_error_from_profile_pack_override(
        "profile-pack-invalid-provider-health-healthy-missing-message",
        &serde_json::json!({
            "compiled_routing_snapshots": [],
            "routing_decision_logs": [],
            "provider_health_snapshots": [
                {
                    "provider_id": "provider-openrouter-main",
                    "extension_id": "sdkwork.provider.openrouter",
                    "runtime": "builtin",
                    "observed_at_ms": 1710003002000u64,
                    "instance_id": "provider-openrouter-main",
                    "running": true,
                    "healthy": true,
                    "message": null
                }
            ]
        }),
        &billing_fixture(vec![]),
    )
    .await;

    assert!(error.contains("provider-openrouter-main"), "{error}");
    assert!(error.contains("healthy"), "{error}");
    assert!(error.contains("message"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_commerce_payment_event_with_unknown_order()
{
    let bootstrap_root = temp_bootstrap_root("profile-pack-invalid-commerce");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("profiles").join("dev.json"),
        &serde_json::json!({
            "profile_id": "dev",
            "description": "development bootstrap pack",
            "channels": ["channels/default.json"],
            "providers": ["providers/default.json"],
            "official_provider_configs": ["official-provider-configs/default.json"],
            "provider_accounts": ["provider-accounts/default.json"],
            "models": ["models/default.json"],
            "channel_models": ["channel-models/default.json"],
            "model_prices": ["model-prices/default.json"],
            "tenants": ["tenants/default.json"],
            "projects": ["projects/default.json"],
            "identities": ["identities/default.json"],
            "extensions": ["extensions/default.json"],
            "service_runtime_nodes": ["service-runtime-nodes/default.json"],
            "extension_runtime_rollouts": ["extension-runtime-rollouts/default.json"],
            "standalone_config_rollouts": ["standalone-config-rollouts/default.json"],
            "routing": ["routing/default.json"],
            "api_key_groups": ["api-key-groups/default.json"],
            "observability": ["observability/default.json"],
            "quota_policies": ["quota-policies/default.json"],
            "pricing": ["pricing/default.json"],
            "payment_methods": ["payment-methods/default.json"],
            "marketing": ["marketing/default.json"],
            "commerce": ["commerce/invalid-missing-order.json"]
        }),
    );
    write_json(
        &bootstrap_root
            .join("commerce")
            .join("invalid-missing-order.json"),
        &serde_json::json!({
            "orders": [],
            "payment_events": [
                {
                    "payment_event_id": "payment-event-invalid-order",
                    "order_id": "order-missing",
                    "project_id": "project_local_demo",
                    "user_id": "user_local_demo",
                    "provider": "stripe",
                    "provider_event_id": "evt_invalid_order",
                    "dedupe_key": "stripe:evt_invalid_order",
                    "event_type": "checkout.session.completed",
                    "payload_json": "{\"id\":\"evt_invalid_order\"}",
                    "processing_status": "processed",
                    "processing_message": "should fail because order is missing",
                    "received_at_ms": 1710007000000u64,
                    "processed_at_ms": 1710007000500u64,
                    "order_status_after": "fulfilled"
                }
            ]
        }),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!("bootstrap should reject payment events that reference unknown orders"),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("order-missing"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_commerce_order_with_coupon_code_outside_marketing_campaign_template(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-commerce-order-coupon-campaign-template");
    write_bootstrap_profile_pack(&bootstrap_root);

    let marketing_path = bootstrap_root.join("marketing").join("default.json");
    let mut marketing =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&marketing_path).unwrap())
            .unwrap();
    marketing["coupon_templates"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({
            "coupon_template_id": "template-campaign-mismatch-20",
            "template_key": "campaign-mismatch-20",
            "display_name": "Campaign Mismatch 20",
            "status": "active",
            "distribution_kind": "shared_code",
            "benefit": {
                "benefit_kind": "percentage_off",
                "discount_percent": 20,
                "discount_amount_minor": null,
                "grant_units": null,
                "currency_code": "USD",
                "max_discount_minor": 20000u64
            },
            "restriction": {
                "subject_scope": "project",
                "min_order_amount_minor": 0u64,
                "first_order_only": false,
                "new_customer_only": false,
                "exclusive_group": "campaign-mismatch",
                "stacking_policy": "best_of_group",
                "max_redemptions_per_subject": 2u64,
                "eligible_target_kinds": ["subscription_plan", "recharge_pack"]
            },
            "created_at_ms": 1710000000000u64,
            "updated_at_ms": 1710000000000u64
        }));
    marketing["marketing_campaigns"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({
            "marketing_campaign_id": "campaign-coupon-mismatch",
            "coupon_template_id": "template-campaign-mismatch-20",
            "display_name": "Campaign Coupon Mismatch",
            "status": "active",
            "start_at_ms": 1710000000000u64,
            "end_at_ms": 1767225600000u64,
            "created_at_ms": 1710000000000u64,
            "updated_at_ms": 1710000000000u64
        }));
    write_json(&marketing_path, &marketing);

    let commerce_path = bootstrap_root.join("commerce").join("default.json");
    let mut commerce =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&commerce_path).unwrap())
            .unwrap();
    commerce["orders"]
        .as_array_mut()
        .unwrap()
        .first_mut()
        .unwrap()["marketing_campaign_id"] = serde_json::json!("campaign-coupon-mismatch");
    write_json(&commerce_path, &commerce);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject orders whose applied coupon code resolves to a different template than the linked marketing campaign"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("order-local-demo-growth-2026"), "{error}");
    assert!(error.contains("campaign-coupon-mismatch"), "{error}");
    assert!(error.contains("LAUNCH100"), "{error}");
    assert!(error.contains("template"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_commerce_order_with_subsidy_mismatched_price_delta(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-commerce-order-subsidy-price-delta");
    write_bootstrap_profile_pack(&bootstrap_root);

    let commerce_path = bootstrap_root.join("commerce").join("default.json");
    let mut commerce =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&commerce_path).unwrap())
            .unwrap();
    commerce["orders"]
        .as_array_mut()
        .unwrap()
        .first_mut()
        .unwrap()["subsidy_amount_minor"] = serde_json::json!(9999u64);
    write_json(&commerce_path, &commerce);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject commerce orders whose subsidy amount does not match list minus payable price"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("order-local-demo-growth-2026"), "{error}");
    assert!(error.contains("subsidy"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_commerce_order_with_refundable_amount_exceeding_payable(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-commerce-order-refundable-over-payable");
    write_bootstrap_profile_pack(&bootstrap_root);

    let commerce_path = bootstrap_root.join("commerce").join("default.json");
    let mut commerce =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&commerce_path).unwrap())
            .unwrap();
    commerce["orders"]
        .as_array_mut()
        .unwrap()
        .first_mut()
        .unwrap()["refundable_amount_minor"] = serde_json::json!(10000u64);
    write_json(&commerce_path, &commerce);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject commerce orders whose refundable amount exceeds payable price"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("order-local-demo-growth-2026"), "{error}");
    assert!(error.contains("refundable"), "{error}");
    assert!(error.contains("payable"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_commerce_order_with_refunded_amount_exceeding_refundable(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-commerce-order-refunded-over-refundable");
    write_bootstrap_profile_pack(&bootstrap_root);

    let commerce_path = bootstrap_root.join("commerce").join("default.json");
    let mut commerce =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&commerce_path).unwrap())
            .unwrap();
    let order = commerce["orders"].as_array_mut().unwrap().first_mut().unwrap();
    order["refundable_amount_minor"] = serde_json::json!(5000u64);
    order["refunded_amount_minor"] = serde_json::json!(5001u64);
    write_json(&commerce_path, &commerce);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject commerce orders whose refunded amount exceeds refundable amount"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("order-local-demo-growth-2026"), "{error}");
    assert!(error.contains("refunded"), "{error}");
    assert!(error.contains("refundable"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_commerce_order_with_latest_payment_attempt_amount_mismatched_payable(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-commerce-order-latest-attempt-amount");
    write_bootstrap_profile_pack(&bootstrap_root);

    let commerce_path = bootstrap_root.join("commerce").join("default.json");
    let mut commerce =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&commerce_path).unwrap())
            .unwrap();
    commerce["payment_attempts"]
        .as_array_mut()
        .unwrap()
        .first_mut()
        .unwrap()["amount_minor"] = serde_json::json!(9800u64);
    commerce["payment_attempts"]
        .as_array_mut()
        .unwrap()
        .first_mut()
        .unwrap()["captured_amount_minor"] = serde_json::json!(9800u64);
    write_json(&commerce_path, &commerce);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject commerce orders whose latest payment attempt amount drifts from payable price"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("order-local-demo-growth-2026"), "{error}");
    assert!(error.contains("latest payment attempt"), "{error}");
    assert!(error.contains("payable"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_commerce_order_with_refunded_amount_mismatched_linked_refunds(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-commerce-order-refund-sum-mismatch");
    write_bootstrap_profile_pack(&bootstrap_root);

    let commerce_path = bootstrap_root.join("commerce").join("default.json");
    let mut commerce =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&commerce_path).unwrap())
            .unwrap();
    commerce["orders"]
        .as_array_mut()
        .unwrap()
        .first_mut()
        .unwrap()["refunded_amount_minor"] = serde_json::json!(1001u64);
    write_json(&commerce_path, &commerce);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject commerce orders whose refunded amount drifts from linked refunds"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("order-local-demo-growth-2026"), "{error}");
    assert!(error.contains("linked refunds"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_commerce_payment_attempt_with_refunded_amount_mismatched_linked_refunds(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-commerce-payment-attempt-refund-sum-mismatch");
    write_bootstrap_profile_pack(&bootstrap_root);

    let commerce_path = bootstrap_root.join("commerce").join("default.json");
    let mut commerce =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&commerce_path).unwrap())
            .unwrap();
    commerce["payment_attempts"]
        .as_array_mut()
        .unwrap()
        .first_mut()
        .unwrap()["refunded_amount_minor"] = serde_json::json!(999u64);
    write_json(&commerce_path, &commerce);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject payment attempts whose refunded amount drifts from linked refunds"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("attempt-local-demo-growth-2026"), "{error}");
    assert!(error.contains("linked refunds"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_commerce_payment_attempt_with_succeeded_status_missing_full_capture(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-commerce-payment-attempt-succeeded-capture");
    write_bootstrap_profile_pack(&bootstrap_root);

    let commerce_path = bootstrap_root.join("commerce").join("default.json");
    let mut commerce =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&commerce_path).unwrap())
            .unwrap();
    commerce["payment_attempts"]
        .as_array_mut()
        .unwrap()
        .first_mut()
        .unwrap()["captured_amount_minor"] = serde_json::json!(9800u64);
    write_json(&commerce_path, &commerce);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject succeeded payment attempts that are not fully captured"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("attempt-local-demo-growth-2026"), "{error}");
    assert!(error.contains("succeeded"), "{error}");
    assert!(error.contains("capture"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_commerce_payment_attempt_with_succeeded_status_missing_completed_at(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-commerce-payment-attempt-succeeded-completion");
    write_bootstrap_profile_pack(&bootstrap_root);

    let commerce_path = bootstrap_root.join("commerce").join("default.json");
    let mut commerce =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&commerce_path).unwrap())
            .unwrap();
    commerce["payment_attempts"]
        .as_array_mut()
        .unwrap()
        .first_mut()
        .unwrap()["completed_at_ms"] = serde_json::Value::Null;
    write_json(&commerce_path, &commerce);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject succeeded payment attempts without a completion timestamp"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("attempt-local-demo-growth-2026"), "{error}");
    assert!(error.contains("succeeded"), "{error}");
    assert!(error.contains("completed"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_commerce_payment_event_with_processed_at_earlier_than_received_at(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-commerce-payment-event-processed-before-received");
    write_bootstrap_profile_pack(&bootstrap_root);

    let commerce_path = bootstrap_root.join("commerce").join("default.json");
    let mut commerce =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&commerce_path).unwrap())
            .unwrap();
    commerce["payment_events"]
        .as_array_mut()
        .unwrap()
        .first_mut()
        .unwrap()["processed_at_ms"] = serde_json::json!(1710005000500u64);
    write_json(&commerce_path, &commerce);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject payment events whose processed_at_ms is earlier than received_at_ms"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("payment-event-local-demo-growth-2026"), "{error}");
    assert!(error.contains("processed"), "{error}");
    assert!(error.contains("received"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_commerce_payment_event_with_processed_status_missing_processed_at(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-commerce-payment-event-processed-missing-ts");
    write_bootstrap_profile_pack(&bootstrap_root);

    let commerce_path = bootstrap_root.join("commerce").join("default.json");
    let mut commerce =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&commerce_path).unwrap())
            .unwrap();
    commerce["payment_events"]
        .as_array_mut()
        .unwrap()
        .first_mut()
        .unwrap()["processed_at_ms"] = serde_json::Value::Null;
    write_json(&commerce_path, &commerce);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject processed payment events that omit processed_at_ms"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("payment-event-local-demo-growth-2026"), "{error}");
    assert!(error.contains("processed"), "{error}");
    assert!(error.contains("processed_at_ms"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_commerce_webhook_inbox_with_dedupe_key_mismatched_linked_payment_event(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-commerce-webhook-linked-payment-event-dedupe");
    write_bootstrap_profile_pack(&bootstrap_root);

    let commerce_path = bootstrap_root.join("commerce").join("default.json");
    let mut commerce =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&commerce_path).unwrap())
            .unwrap();
    commerce["webhook_inbox_records"]
        .as_array_mut()
        .unwrap()
        .first_mut()
        .unwrap()["dedupe_key"] = serde_json::json!("stripe:evt_local_demo_growth_2026:drift");
    write_json(&commerce_path, &commerce);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject webhook inbox records whose dedupe key drifts from linked payment events"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("webhook-inbox-local-demo-growth-2026"), "{error}");
    assert!(error.contains("payment-event-local-demo-growth-2026"), "{error}");
    assert!(error.contains("dedupe"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_commerce_webhook_inbox_with_processed_status_missing_processed_at(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-commerce-webhook-processed-missing-ts");
    write_bootstrap_profile_pack(&bootstrap_root);

    let commerce_path = bootstrap_root.join("commerce").join("default.json");
    let mut commerce =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&commerce_path).unwrap())
            .unwrap();
    commerce["webhook_inbox_records"]
        .as_array_mut()
        .unwrap()
        .first_mut()
        .unwrap()["processed_at_ms"] = serde_json::Value::Null;
    write_json(&commerce_path, &commerce);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject processed webhook inbox records that omit processed_at_ms"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("webhook-inbox-local-demo-growth-2026"), "{error}");
    assert!(error.contains("processed"), "{error}");
    assert!(error.contains("processed_at_ms"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_commerce_webhook_inbox_with_processed_at_earlier_than_last_received_at(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-commerce-webhook-processed-before-last-received");
    write_bootstrap_profile_pack(&bootstrap_root);

    let commerce_path = bootstrap_root.join("commerce").join("default.json");
    let mut commerce =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&commerce_path).unwrap())
            .unwrap();
    commerce["webhook_inbox_records"]
        .as_array_mut()
        .unwrap()
        .first_mut()
        .unwrap()["processed_at_ms"] = serde_json::json!(1710005000590u64);
    write_json(&commerce_path, &commerce);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject webhook inbox records whose processed_at_ms is earlier than last_received_at_ms"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("webhook-inbox-local-demo-growth-2026"), "{error}");
    assert!(error.contains("processed"), "{error}");
    assert!(error.contains("last_received"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_commerce_payment_event_with_provider_mismatched_order_payment_method(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-commerce-payment-event-order-payment-method-provider");
    write_bootstrap_profile_pack(&bootstrap_root);

    let commerce_path = bootstrap_root.join("commerce").join("default.json");
    let mut commerce =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&commerce_path).unwrap())
            .unwrap();
    let order = commerce["orders"].as_array_mut().unwrap().first_mut().unwrap();
    order["latest_payment_attempt_id"] = serde_json::Value::Null;
    let payment_event = commerce["payment_events"]
        .as_array_mut()
        .unwrap()
        .first_mut()
        .unwrap();
    payment_event["provider"] = serde_json::json!("bank_transfer");
    write_json(&commerce_path, &commerce);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject payment events whose provider drifts from the linked order payment method"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("payment-event-local-demo-growth-2026"), "{error}");
    assert!(error.contains("order-local-demo-growth-2026"), "{error}");
    assert!(error.contains("payment method"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_commerce_payment_event_with_provider_mismatched_latest_order_attempt(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-commerce-payment-event-latest-attempt-provider");
    write_bootstrap_profile_pack(&bootstrap_root);

    let commerce_path = bootstrap_root.join("commerce").join("default.json");
    let mut commerce =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&commerce_path).unwrap())
            .unwrap();
    let order = commerce["orders"].as_array_mut().unwrap().first_mut().unwrap();
    order["payment_method_id"] = serde_json::Value::Null;
    let payment_event = commerce["payment_events"]
        .as_array_mut()
        .unwrap()
        .first_mut()
        .unwrap();
    payment_event["provider"] = serde_json::json!("bank_transfer");
    write_json(&commerce_path, &commerce);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject payment events whose provider drifts from the linked order latest payment attempt"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("payment-event-local-demo-growth-2026"), "{error}");
    assert!(error.contains("order-local-demo-growth-2026"), "{error}");
    assert!(error.contains("latest payment attempt"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_commerce_webhook_inbox_with_provider_mismatched_linked_payment_event(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-commerce-webhook-linked-payment-event-provider");
    write_bootstrap_profile_pack(&bootstrap_root);

    let commerce_path = bootstrap_root.join("commerce").join("default.json");
    let mut commerce =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&commerce_path).unwrap())
            .unwrap();
    let webhook = commerce["webhook_inbox_records"]
        .as_array_mut()
        .unwrap()
        .first_mut()
        .unwrap();
    webhook["payment_method_id"] = serde_json::Value::Null;
    webhook["provider"] = serde_json::json!("bank_transfer");
    write_json(&commerce_path, &commerce);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject webhook inbox records whose provider drifts from linked payment events sharing provider_event_id"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("webhook-inbox-local-demo-growth-2026"), "{error}");
    assert!(error.contains("payment-event-local-demo-growth-2026"), "{error}");
    assert!(error.contains("provider"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_commerce_refund_with_payment_method_provider_mismatch_without_payment_attempt(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-commerce-refund-payment-method-provider");
    write_bootstrap_profile_pack(&bootstrap_root);

    let commerce_path = bootstrap_root.join("commerce").join("default.json");
    let mut commerce =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&commerce_path).unwrap())
            .unwrap();
    let refund = commerce["refunds"].as_array_mut().unwrap().first_mut().unwrap();
    refund["payment_attempt_id"] = serde_json::Value::Null;
    refund["provider"] = serde_json::json!("bank_transfer");
    commerce["payment_attempts"]
        .as_array_mut()
        .unwrap()
        .first_mut()
        .unwrap()["refunded_amount_minor"] = serde_json::json!(0u64);
    commerce["reconciliation_runs"] = serde_json::json!([]);
    commerce["reconciliation_items"] = serde_json::json!([]);
    write_json(&commerce_path, &commerce);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject refunds whose payment method provider drifts when no payment attempt is linked"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("refund-local-demo-growth-2026"), "{error}");
    assert!(error.contains("payment-stripe-hosted"), "{error}");
    assert!(error.contains("payment method"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_commerce_payment_attempt_with_currency_mismatched_order(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-commerce-payment-attempt-currency");
    write_bootstrap_profile_pack(&bootstrap_root);

    let commerce_path = bootstrap_root.join("commerce").join("default.json");
    let mut commerce =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&commerce_path).unwrap())
            .unwrap();
    let attempt = commerce["payment_attempts"]
        .as_array_mut()
        .unwrap()
        .first_mut()
        .unwrap();
    attempt["currency_code"] = serde_json::json!("CNY");
    write_json(&commerce_path, &commerce);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject payment attempts whose currency drifts from the linked order"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("attempt-local-demo-growth-2026"), "{error}");
    assert!(error.contains("order-local-demo-growth-2026"), "{error}");
    assert!(error.contains("currency"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_commerce_refund_with_currency_mismatched_order(
) {
    let bootstrap_root = temp_bootstrap_root("profile-pack-invalid-commerce-refund-currency");
    write_bootstrap_profile_pack(&bootstrap_root);

    let commerce_path = bootstrap_root.join("commerce").join("default.json");
    let mut commerce =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&commerce_path).unwrap())
            .unwrap();
    let refund = commerce["refunds"].as_array_mut().unwrap().first_mut().unwrap();
    refund["currency_code"] = serde_json::json!("CNY");
    write_json(&commerce_path, &commerce);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!("bootstrap should reject refunds whose currency drifts from the linked order"),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("refund-local-demo-growth-2026"), "{error}");
    assert!(error.contains("order-local-demo-growth-2026"), "{error}");
    assert!(error.contains("currency"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_commerce_refund_with_succeeded_status_missing_completed_at(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-commerce-refund-succeeded-missing-completion");
    write_bootstrap_profile_pack(&bootstrap_root);

    let commerce_path = bootstrap_root.join("commerce").join("default.json");
    let mut commerce =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&commerce_path).unwrap())
            .unwrap();
    commerce["refunds"]
        .as_array_mut()
        .unwrap()
        .first_mut()
        .unwrap()["completed_at_ms"] = serde_json::Value::Null;
    write_json(&commerce_path, &commerce);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject succeeded refunds that omit completed_at_ms"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("refund-local-demo-growth-2026"), "{error}");
    assert!(error.contains("succeeded"), "{error}");
    assert!(error.contains("completed_at_ms"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_commerce_reconciliation_run_with_completed_status_missing_completed_at(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-commerce-reconciliation-run-completed-missing-ts");
    write_bootstrap_profile_pack(&bootstrap_root);

    let commerce_path = bootstrap_root.join("commerce").join("default.json");
    let mut commerce =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&commerce_path).unwrap())
            .unwrap();
    commerce["reconciliation_runs"]
        .as_array_mut()
        .unwrap()
        .first_mut()
        .unwrap()["completed_at_ms"] = serde_json::Value::Null;
    write_json(&commerce_path, &commerce);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject completed reconciliation runs that omit completed_at_ms"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("recon-run-local-demo-growth-2026"), "{error}");
    assert!(error.contains("completed"), "{error}");
    assert!(error.contains("completed_at_ms"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_commerce_webhook_inbox_with_payment_method_mismatched_linked_order(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-commerce-webhook-linked-order-payment-method");
    write_bootstrap_profile_pack(&bootstrap_root);

    let payment_methods_path = bootstrap_root.join("payment-methods").join("default.json");
    let mut payment_methods = serde_json::from_str::<serde_json::Value>(
        &fs::read_to_string(&payment_methods_path).unwrap(),
    )
    .unwrap();
    payment_methods["payment_methods"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({
            "payment_method_id": "payment-stripe-billing-portal",
            "display_name": "Stripe Billing Portal",
            "description": "Alternative Stripe payment method for portal billing flows",
            "provider": "stripe",
            "channel": "billing_portal",
            "mode": "live",
            "enabled": true,
            "sort_order": 11,
            "capability_codes": ["checkout", "refund"],
            "supported_currency_codes": ["USD", "EUR"],
            "supported_country_codes": ["US", "DE", "SG"],
            "supported_order_kinds": ["subscription_plan", "recharge_pack", "custom_recharge"],
            "callback_strategy": "webhook_signed",
            "webhook_path": "/api/portal/commerce/webhooks/stripe",
            "webhook_tolerance_seconds": 300u64,
            "replay_window_seconds": 300u64,
            "max_retry_count": 8u32,
            "config_json": "{\"provider\":\"stripe\",\"mode\":\"billing_portal\"}",
            "created_at_ms": 1710000000000u64,
            "updated_at_ms": 1710000000000u64
        }));
    write_json(&payment_methods_path, &payment_methods);

    let commerce_path = bootstrap_root.join("commerce").join("default.json");
    let mut commerce =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&commerce_path).unwrap())
            .unwrap();
    let webhook = commerce["webhook_inbox_records"]
        .as_array_mut()
        .unwrap()
        .first_mut()
        .unwrap();
    webhook["payment_method_id"] = serde_json::json!("payment-stripe-billing-portal");
    write_json(&commerce_path, &commerce);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject webhook inbox records whose payment method drifts from the linked payment event order"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("webhook-inbox-local-demo-growth-2026"), "{error}");
    assert!(error.contains("order-local-demo-growth-2026"), "{error}");
    assert!(error.contains("payment method"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_billing_event_with_missing_snapshot() {
    let bootstrap_root = temp_bootstrap_root("profile-pack-invalid-billing");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("profiles").join("dev.json"),
        &serde_json::json!({
            "profile_id": "dev",
            "description": "development bootstrap pack",
            "channels": ["channels/default.json"],
            "providers": ["providers/default.json"],
            "official_provider_configs": ["official-provider-configs/default.json"],
            "provider_accounts": ["provider-accounts/default.json"],
            "models": ["models/default.json"],
            "channel_models": ["channel-models/default.json"],
            "model_prices": ["model-prices/default.json"],
            "tenants": ["tenants/default.json"],
            "projects": ["projects/default.json"],
            "identities": ["identities/default.json"],
            "extensions": ["extensions/default.json"],
            "routing": ["routing/default.json"],
            "api_key_groups": ["api-key-groups/default.json"],
            "observability": ["observability/default.json"],
            "quota_policies": ["quota-policies/default.json"],
            "pricing": ["pricing/default.json"],
            "payment_methods": ["payment-methods/default.json"],
            "marketing": ["marketing/default.json"],
            "commerce": ["commerce/default.json"],
            "billing": ["billing/invalid-missing-snapshot.json"]
        }),
    );
    write_json(
        &bootstrap_root
            .join("billing")
            .join("invalid-missing-snapshot.json"),
        &serde_json::json!({
            "billing_events": [
                {
                    "event_id": "billing-invalid-missing-snapshot",
                    "tenant_id": "tenant_local_demo",
                    "project_id": "project_local_demo",
                    "api_key_group_id": "group-local-demo-live",
                    "capability": "responses",
                    "route_key": "gpt-4.1",
                    "usage_model": "gpt-4.1",
                    "provider_id": "provider-openrouter-main",
                    "accounting_mode": "platform_credit",
                    "operation_kind": "request",
                    "modality": "text",
                    "api_key_hash": "a19d2bf76318aa7f619d684271469bb383faf1cb5bd4c680088465cde9d0003b",
                    "channel_id": "openrouter",
                    "reference_id": "req_invalid_snapshot",
                    "latency_ms": 420u64,
                    "units": 1u64,
                    "request_count": 1u64,
                    "input_tokens": 1200u64,
                    "output_tokens": 400u64,
                    "total_tokens": 1600u64,
                    "cache_read_tokens": 0u64,
                    "cache_write_tokens": 0u64,
                    "image_count": 0u64,
                    "audio_seconds": 0.0,
                    "video_seconds": 0.0,
                    "music_seconds": 0.0,
                    "upstream_cost": 0.27,
                    "customer_charge": 0.59,
                    "applied_routing_profile_id": "profile-global-balanced",
                    "compiled_routing_snapshot_id": "snapshot-missing",
                    "fallback_reason": null,
                    "created_at_ms": 1710008000000u64
                }
            ]
        }),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject billing events that reference unknown routing snapshots"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("snapshot-missing"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_billing_route_key_without_channel_model(
) {
    let bootstrap_root = temp_bootstrap_root("profile-pack-invalid-billing-route-key");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("billing").join("default.json"),
        &billing_fixture_with_local_metering_support(vec![serde_json::json!({
            "event_id": "billing-invalid-missing-channel-model",
            "tenant_id": "tenant_local_demo",
            "project_id": "project_local_demo",
            "api_key_group_id": "group-local-demo-live",
            "capability": "responses",
            "route_key": "model-missing",
            "usage_model": "deepseek-chat",
            "provider_id": "provider-openrouter-main",
            "accounting_mode": "platform_credit",
            "operation_kind": "request",
            "modality": "text",
            "api_key_hash": "a19d2bf76318aa7f619d684271469bb383faf1cb5bd4c680088465cde9d0003b",
            "channel_id": "openrouter",
            "reference_id": "req_invalid_channel_model",
            "latency_ms": 420u64,
            "units": 1u64,
            "request_count": 1u64,
            "input_tokens": 1200u64,
            "output_tokens": 400u64,
            "total_tokens": 1600u64,
            "cache_read_tokens": 0u64,
            "cache_write_tokens": 0u64,
            "image_count": 0u64,
            "audio_seconds": 0.0,
            "video_seconds": 0.0,
            "music_seconds": 0.0,
            "upstream_cost": 0.27,
            "customer_charge": 0.59,
            "applied_routing_profile_id": "profile-global-balanced",
            "compiled_routing_snapshot_id": null,
            "fallback_reason": null,
            "created_at_ms": 1710008000000u64
        })]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject billing events whose route_key is missing from the channel catalog"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("billing_events.route_key"), "{error}");
    assert!(error.contains("openrouter::model-missing"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_billing_usage_model_without_provider_mapping(
) {
    let bootstrap_root = temp_bootstrap_root("profile-pack-invalid-billing-usage-model");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("billing").join("default.json"),
        &billing_fixture_with_local_metering_support(vec![serde_json::json!({
            "event_id": "billing-invalid-usage-model-provider",
            "tenant_id": "tenant_local_demo",
            "project_id": "project_local_demo",
            "api_key_group_id": "group-local-demo-live",
            "capability": "responses",
            "route_key": "deepseek-chat",
            "usage_model": "qwen-plus-latest",
            "provider_id": "provider-openrouter-main",
            "accounting_mode": "platform_credit",
            "operation_kind": "request",
            "modality": "text",
            "api_key_hash": "a19d2bf76318aa7f619d684271469bb383faf1cb5bd4c680088465cde9d0003b",
            "channel_id": "openrouter",
            "reference_id": "req_invalid_usage_model_provider",
            "latency_ms": 420u64,
            "units": 1u64,
            "request_count": 1u64,
            "input_tokens": 1200u64,
            "output_tokens": 400u64,
            "total_tokens": 1600u64,
            "cache_read_tokens": 0u64,
            "cache_write_tokens": 0u64,
            "image_count": 0u64,
            "audio_seconds": 0.0,
            "video_seconds": 0.0,
            "music_seconds": 0.0,
            "upstream_cost": 0.27,
            "customer_charge": 0.59,
            "applied_routing_profile_id": "profile-global-balanced",
            "compiled_routing_snapshot_id": null,
            "fallback_reason": null,
            "created_at_ms": 1710008000000u64
        })]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject billing events whose usage_model is not mapped for the provider route"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("billing-invalid-usage-model-provider"), "{error}");
    assert!(error.contains("qwen-plus-latest"), "{error}");
    assert!(error.contains("provider-openrouter-main"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_billing_api_key_hash_mismatched_gateway_key_group(
) {
    let bootstrap_root = temp_bootstrap_root("profile-pack-invalid-billing-api-key-group");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("billing").join("default.json"),
        &billing_fixture_with_local_metering_support(vec![serde_json::json!({
            "event_id": "billing-invalid-api-key-group",
            "tenant_id": "tenant_local_demo",
            "project_id": "project_local_demo",
            "api_key_group_id": "group-local-demo-sandbox",
            "capability": "responses",
            "route_key": "gpt-4.1",
            "usage_model": "gpt-4.1",
            "provider_id": "provider-openrouter-main",
            "accounting_mode": "platform_credit",
            "operation_kind": "request",
            "modality": "text",
            "api_key_hash": "a19d2bf76318aa7f619d684271469bb383faf1cb5bd4c680088465cde9d0003b",
            "channel_id": "openrouter",
            "reference_id": "req_invalid_billing_api_key_group",
            "latency_ms": 420u64,
            "units": 1u64,
            "request_count": 1u64,
            "input_tokens": 1200u64,
            "output_tokens": 400u64,
            "total_tokens": 1600u64,
            "cache_read_tokens": 0u64,
            "cache_write_tokens": 0u64,
            "image_count": 0u64,
            "audio_seconds": 0.0,
            "video_seconds": 0.0,
            "music_seconds": 0.0,
            "upstream_cost": 0.27,
            "customer_charge": 0.59,
            "applied_routing_profile_id": "profile-global-balanced",
            "compiled_routing_snapshot_id": null,
            "fallback_reason": null,
            "created_at_ms": 1710008000000u64
        })]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject billing events whose api_key_hash resolves to a gateway key from another api key group"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("billing-invalid-api-key-group"), "{error}");
    assert!(error.contains("api_key_hash"), "{error}");
    assert!(error.contains("group-local-demo-sandbox"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_decision_selected_provider_outside_snapshot(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-decision-selected-provider-snapshot");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("observability").join("default.json"),
        &serde_json::json!({
            "compiled_routing_snapshots": [
                {
                    "snapshot_id": "snapshot-invalid-decision-provider",
                    "tenant_id": "tenant_local_demo",
                    "project_id": "project_local_demo",
                    "api_key_group_id": "group-local-demo-live",
                    "capability": "responses",
                    "route_key": "gpt-4.1",
                    "matched_policy_id": "policy-default-responses",
                    "project_routing_preferences_project_id": "project_local_demo",
                    "applied_routing_profile_id": "profile-global-balanced",
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-openrouter-main",
                        "provider-siliconflow-main"
                    ],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000u64,
                    "require_healthy": false,
                    "preferred_region": "global",
                    "created_at_ms": 1710003000000u64,
                    "updated_at_ms": 1710003000500u64
                }
            ],
            "routing_decision_logs": [
                {
                    "decision_id": "decision-invalid-snapshot-provider",
                    "decision_source": "gateway",
                    "tenant_id": "tenant_local_demo",
                    "project_id": "project_local_demo",
                    "api_key_group_id": "group-local-demo-live",
                    "capability": "responses",
                    "route_key": "gpt-4.1",
                    "selected_provider_id": "provider-ollama-local",
                    "matched_policy_id": "policy-default-responses",
                    "applied_routing_profile_id": "profile-global-balanced",
                    "compiled_routing_snapshot_id": "snapshot-invalid-decision-provider",
                    "strategy": "weighted_random",
                    "selection_seed": 17u64,
                    "selection_reason": "invalid selected provider should be rejected",
                    "fallback_reason": null,
                    "requested_region": "global",
                    "slo_applied": false,
                    "slo_degraded": false,
                    "created_at_ms": 1710003001000u64,
                    "assessments": [
                        {
                            "provider_id": "provider-openrouter-main",
                            "available": true,
                            "health": "healthy",
                            "policy_rank": 0,
                            "weight": 60u64,
                            "cost": 0.27,
                            "latency_ms": 540u64,
                            "region": "global",
                            "region_match": true,
                            "reasons": ["primary route"]
                        }
                    ]
                }
            ],
            "provider_health_snapshots": []
        }),
    );
    write_json(
        &bootstrap_root.join("billing").join("default.json"),
        &billing_fixture_with_local_metering_support(vec![]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject routing decision logs whose selected provider is outside the compiled snapshot"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("decision-invalid-snapshot-provider"), "{error}");
    assert!(error.contains("provider-ollama-local"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_decision_assessment_provider_outside_snapshot(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-decision-assessment-provider-snapshot");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("observability").join("default.json"),
        &serde_json::json!({
            "compiled_routing_snapshots": [
                {
                    "snapshot_id": "snapshot-invalid-assessment-provider",
                    "tenant_id": "tenant_local_demo",
                    "project_id": "project_local_demo",
                    "api_key_group_id": "group-local-demo-live",
                    "capability": "responses",
                    "route_key": "gpt-4.1",
                    "matched_policy_id": "policy-default-responses",
                    "project_routing_preferences_project_id": "project_local_demo",
                    "applied_routing_profile_id": "profile-global-balanced",
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-openrouter-main",
                        "provider-siliconflow-main"
                    ],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000u64,
                    "require_healthy": false,
                    "preferred_region": "global",
                    "created_at_ms": 1710003000000u64,
                    "updated_at_ms": 1710003000500u64
                }
            ],
            "routing_decision_logs": [
                {
                    "decision_id": "decision-invalid-snapshot-assessment",
                    "decision_source": "gateway",
                    "tenant_id": "tenant_local_demo",
                    "project_id": "project_local_demo",
                    "api_key_group_id": "group-local-demo-live",
                    "capability": "responses",
                    "route_key": "gpt-4.1",
                    "selected_provider_id": "provider-openrouter-main",
                    "matched_policy_id": "policy-default-responses",
                    "applied_routing_profile_id": "profile-global-balanced",
                    "compiled_routing_snapshot_id": "snapshot-invalid-assessment-provider",
                    "strategy": "weighted_random",
                    "selection_seed": 17u64,
                    "selection_reason": "invalid assessment provider should be rejected",
                    "fallback_reason": null,
                    "requested_region": "global",
                    "slo_applied": false,
                    "slo_degraded": false,
                    "created_at_ms": 1710003001000u64,
                    "assessments": [
                        {
                            "provider_id": "provider-openrouter-main",
                            "available": true,
                            "health": "healthy",
                            "policy_rank": 0,
                            "weight": 60u64,
                            "cost": 0.27,
                            "latency_ms": 540u64,
                            "region": "global",
                            "region_match": true,
                            "reasons": ["primary route"]
                        },
                        {
                            "provider_id": "provider-ollama-local",
                            "available": true,
                            "health": "healthy",
                            "policy_rank": 1,
                            "weight": 40u64,
                            "cost": 0.0,
                            "latency_ms": 180u64,
                            "region": "local",
                            "region_match": false,
                            "reasons": ["invalid fallback"]
                        }
                    ]
                }
            ],
            "provider_health_snapshots": []
        }),
    );
    write_json(
        &bootstrap_root.join("billing").join("default.json"),
        &billing_fixture_with_local_metering_support(vec![]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject routing decision logs whose assessed providers are outside the compiled snapshot"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("decision-invalid-snapshot-assessment"), "{error}");
    assert!(error.contains("provider-ollama-local"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_billing_provider_outside_snapshot() {
    let bootstrap_root = temp_bootstrap_root("profile-pack-invalid-billing-provider-snapshot");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("observability").join("default.json"),
        &serde_json::json!({
            "compiled_routing_snapshots": [
                {
                    "snapshot_id": "snapshot-invalid-billing-provider",
                    "tenant_id": "tenant_local_demo",
                    "project_id": "project_local_demo",
                    "api_key_group_id": "group-local-demo-live",
                    "capability": "responses",
                    "route_key": "gpt-4.1",
                    "matched_policy_id": "policy-default-responses",
                    "project_routing_preferences_project_id": "project_local_demo",
                    "applied_routing_profile_id": "profile-global-balanced",
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-openrouter-main",
                        "provider-siliconflow-main"
                    ],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000u64,
                    "require_healthy": false,
                    "preferred_region": "global",
                    "created_at_ms": 1710003000000u64,
                    "updated_at_ms": 1710003000500u64
                }
            ],
            "routing_decision_logs": [],
            "provider_health_snapshots": []
        }),
    );
    write_json(
        &bootstrap_root.join("billing").join("default.json"),
        &billing_fixture_with_local_metering_support(vec![serde_json::json!({
            "event_id": "billing-invalid-snapshot-provider",
            "tenant_id": "tenant_local_demo",
            "project_id": "project_local_demo",
            "api_key_group_id": "group-local-demo-live",
            "capability": "responses",
            "route_key": "gpt-4.1",
            "usage_model": "gpt-4.1",
            "provider_id": "provider-openai-official",
            "accounting_mode": "platform_credit",
            "operation_kind": "request",
            "modality": "text",
            "api_key_hash": "a19d2bf76318aa7f619d684271469bb383faf1cb5bd4c680088465cde9d0003b",
            "channel_id": "openai",
            "reference_id": "req_invalid_billing_provider",
            "latency_ms": 420u64,
            "units": 1u64,
            "request_count": 1u64,
            "input_tokens": 1200u64,
            "output_tokens": 400u64,
            "total_tokens": 1600u64,
            "cache_read_tokens": 0u64,
            "cache_write_tokens": 0u64,
            "image_count": 0u64,
            "audio_seconds": 0.0,
            "video_seconds": 0.0,
            "music_seconds": 0.0,
            "upstream_cost": 0.0,
            "customer_charge": 0.59,
            "applied_routing_profile_id": "profile-global-balanced",
            "compiled_routing_snapshot_id": "snapshot-invalid-billing-provider",
            "fallback_reason": null,
            "created_at_ms": 1710008000000u64
        })]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject billing events whose provider is outside the compiled snapshot"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("billing-invalid-snapshot-provider"), "{error}");
    assert!(error.contains("provider-openai-official"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_billing_channel_outside_provider_bindings()
{
    let bootstrap_root = temp_bootstrap_root("profile-pack-invalid-billing-channel-provider");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("billing").join("default.json"),
        &billing_fixture_with_local_metering_support(vec![serde_json::json!({
            "event_id": "billing-invalid-provider-channel",
            "tenant_id": "tenant_local_demo",
            "project_id": "project_local_demo",
            "api_key_group_id": "group-local-demo-live",
            "capability": "responses",
            "route_key": "gpt-4.1",
            "usage_model": "gpt-4.1",
            "provider_id": "provider-openrouter-main",
            "accounting_mode": "platform_credit",
            "operation_kind": "request",
            "modality": "text",
            "api_key_hash": "a19d2bf76318aa7f619d684271469bb383faf1cb5bd4c680088465cde9d0003b",
            "channel_id": "ollama",
            "reference_id": "req_invalid_provider_channel",
            "latency_ms": 420u64,
            "units": 1u64,
            "request_count": 1u64,
            "input_tokens": 1200u64,
            "output_tokens": 400u64,
            "total_tokens": 1600u64,
            "cache_read_tokens": 0u64,
            "cache_write_tokens": 0u64,
            "image_count": 0u64,
            "audio_seconds": 0.0,
            "video_seconds": 0.0,
            "music_seconds": 0.0,
            "upstream_cost": 0.27,
            "customer_charge": 0.59,
            "applied_routing_profile_id": "profile-global-balanced",
            "compiled_routing_snapshot_id": "snapshot-local-demo-live-responses",
            "fallback_reason": null,
            "created_at_ms": 1710008000000u64
        })]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject billing events whose channel is not bound to the provider"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("billing-invalid-provider-channel"), "{error}");
    assert!(error.contains("provider-openrouter-main"), "{error}");
    assert!(error.contains("ollama"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_billing_provider_with_only_foreign_tenant_account(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-billing-foreign-tenant-provider-account");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("tenants").join("default.json"),
        &serde_json::json!([
            { "id": "tenant_local_demo", "name": "Local Demo Workspace" },
            { "id": "tenant_other_demo", "name": "Other Demo Workspace" }
        ]),
    );
    write_json(
        &bootstrap_root.join("routing").join("default.json"),
        &serde_json::json!({
            "profiles": [
                {
                    "profile_id": "profile-global-balanced",
                    "tenant_id": "tenant_local_demo",
                    "project_id": "project_local_demo",
                    "name": "Global Balanced",
                    "slug": "global-balanced",
                    "description": "Balanced multi-provider routing",
                    "active": true,
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-openrouter-main",
                        "provider-ollama-local"
                    ],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "preferred_region": "global",
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ],
            "policies": [
                {
                    "policy_id": "policy-default-responses",
                    "capability": "responses",
                    "model_pattern": "*",
                    "enabled": true,
                    "priority": 100,
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-openrouter-main",
                        "provider-ollama-local"
                    ],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "execution_failover_enabled": true,
                    "upstream_retry_max_attempts": 3,
                    "upstream_retry_base_delay_ms": 250,
                    "upstream_retry_max_delay_ms": 2000
                }
            ],
            "project_preferences": [
                {
                    "project_id": "project_local_demo",
                    "preset_id": "profile-global-balanced",
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-openrouter-main",
                        "provider-ollama-local"
                    ],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "preferred_region": "global",
                    "updated_at_ms": 1710000000000u64
                }
            ]
        }),
    );
    write_json(
        &bootstrap_root
            .join("provider-accounts")
            .join("default.json"),
        &serde_json::json!([
            {
                "provider_account_id": "acct-openrouter-default",
                "provider_id": "provider-openrouter-main",
                "display_name": "OpenRouter Default",
                "account_kind": "api_key",
                "owner_scope": "platform",
                "owner_tenant_id": null,
                "execution_instance_id": "provider-openrouter-main",
                "base_url_override": "https://openrouter.ai/api/v1",
                "region": "global",
                "priority": 100,
                "weight": 10,
                "enabled": true,
                "routing_tags": ["default", "global"],
                "health_score_hint": null,
                "latency_ms_hint": null,
                "cost_hint": null,
                "success_rate_hint": null,
                "throughput_hint": null,
                "max_concurrency": null,
                "daily_budget": null,
                "notes": "bootstrap default account"
            },
            {
                "provider_account_id": "acct-ollama-local-default",
                "provider_id": "provider-ollama-local",
                "display_name": "Ollama Local Default",
                "account_kind": "runtime_instance",
                "owner_scope": "platform",
                "owner_tenant_id": null,
                "execution_instance_id": "provider-ollama-local",
                "base_url_override": "http://127.0.0.1:11434",
                "region": "local",
                "priority": 90,
                "weight": 5,
                "enabled": true,
                "routing_tags": ["default", "local"],
                "health_score_hint": null,
                "latency_ms_hint": 35,
                "cost_hint": 0.0,
                "success_rate_hint": null,
                "throughput_hint": null,
                "max_concurrency": null,
                "daily_budget": null,
                "notes": "bootstrap local account"
            },
            {
                "provider_account_id": "acct-siliconflow-tenant-other",
                "provider_id": "provider-siliconflow-main",
                "display_name": "SiliconFlow Other Tenant",
                "account_kind": "api_key",
                "owner_scope": "tenant",
                "owner_tenant_id": "tenant_other_demo",
                "execution_instance_id": "provider-siliconflow-main",
                "base_url_override": "https://api.siliconflow.cn/v1",
                "region": "cn",
                "priority": 95,
                "weight": 8,
                "enabled": true,
                "routing_tags": ["tenant", "other"],
                "health_score_hint": null,
                "latency_ms_hint": null,
                "cost_hint": null,
                "success_rate_hint": null,
                "throughput_hint": null,
                "max_concurrency": null,
                "daily_budget": null,
                "notes": "foreign tenant scoped account"
            }
        ]),
    );
    write_json(
        &bootstrap_root.join("observability").join("default.json"),
        &serde_json::json!({
            "compiled_routing_snapshots": [],
            "routing_decision_logs": [],
            "provider_health_snapshots": []
        }),
    );
    write_json(
        &bootstrap_root.join("billing").join("default.json"),
        &billing_fixture_with_local_metering_support(vec![serde_json::json!({
            "event_id": "billing-invalid-foreign-tenant-provider-account",
            "tenant_id": "tenant_local_demo",
            "project_id": "project_local_demo",
            "api_key_group_id": null,
            "capability": "responses",
            "route_key": "qwen-plus-latest",
            "usage_model": "qwen-plus-latest",
            "provider_id": "provider-siliconflow-main",
            "accounting_mode": "platform_credit",
            "operation_kind": "request",
            "modality": "text",
            "api_key_hash": null,
            "channel_id": "siliconflow",
            "reference_id": "req_invalid_foreign_tenant_provider_account",
            "latency_ms": 420u64,
            "units": 1u64,
            "request_count": 1u64,
            "input_tokens": 1200u64,
            "output_tokens": 400u64,
            "total_tokens": 1600u64,
            "cache_read_tokens": 0u64,
            "cache_write_tokens": 0u64,
            "image_count": 0u64,
            "audio_seconds": 0.0,
            "video_seconds": 0.0,
            "music_seconds": 0.0,
            "upstream_cost": 0.27,
            "customer_charge": 0.59,
            "applied_routing_profile_id": null,
            "compiled_routing_snapshot_id": null,
            "fallback_reason": null,
            "created_at_ms": 1710008000000u64
        })]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject billing events whose provider only has a foreign tenant account"
        ),
        Err(error) => error.to_string(),
    };

    assert!(
        error.contains("billing-invalid-foreign-tenant-provider-account"),
        "{error}"
    );
    assert!(error.contains("provider-siliconflow-main"), "{error}");
    assert!(error.contains("provider account"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_compiled_snapshot_provider_with_only_foreign_tenant_account(
) {
    let snapshot_id = "snapshot-invalid-foreign-tenant-provider-account";
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-snapshot-foreign-tenant-provider-account");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("tenants").join("default.json"),
        &serde_json::json!([
            { "id": "tenant_local_demo", "name": "Local Demo Workspace" },
            { "id": "tenant_other_demo", "name": "Other Demo Workspace" }
        ]),
    );
    write_json(
        &bootstrap_root
            .join("provider-accounts")
            .join("default.json"),
        &foreign_tenant_siliconflow_provider_accounts_fixture(),
    );
    write_json(
        &bootstrap_root.join("routing").join("default.json"),
        &routing_fixture_without_siliconflow_candidates(),
    );
    let mut snapshot = compiled_routing_snapshot_fixture(snapshot_id);
    snapshot["applied_routing_profile_id"] = serde_json::Value::Null;
    snapshot["ordered_provider_ids"] = serde_json::json!([
        "provider-openrouter-main",
        "provider-siliconflow-main"
    ]);
    write_json(
        &bootstrap_root.join("observability").join("default.json"),
        &observability_fixture(snapshot, vec![]),
    );
    write_json(
        &bootstrap_root.join("billing").join("default.json"),
        &billing_fixture_with_local_metering_support(vec![]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject compiled routing snapshot providers whose tenant only has a foreign tenant account"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains(snapshot_id), "{error}");
    assert!(error.contains("provider-siliconflow-main"), "{error}");
    assert!(error.contains("provider account"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_compiled_snapshot_provider_outside_applied_routing_profile(
) {
    let snapshot_id = "snapshot-invalid-profile-provider";
    let bootstrap_root = temp_bootstrap_root("profile-pack-invalid-snapshot-profile-provider");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("routing").join("default.json"),
        &serde_json::json!({
            "profiles": [
                {
                    "profile_id": "profile-global-balanced",
                    "tenant_id": "tenant_local_demo",
                    "project_id": "project_local_demo",
                    "name": "Global Balanced",
                    "slug": "global-balanced",
                    "description": "Balanced multi-provider routing",
                    "active": true,
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-openrouter-main",
                        "provider-ollama-local"
                    ],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "preferred_region": "global",
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ],
            "policies": [
                {
                    "policy_id": "policy-default-responses",
                    "capability": "responses",
                    "model_pattern": "qwen-plus-*",
                    "enabled": true,
                    "priority": 100,
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-siliconflow-main"
                    ],
                    "default_provider_id": "provider-siliconflow-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "execution_failover_enabled": true,
                    "upstream_retry_max_attempts": 3,
                    "upstream_retry_base_delay_ms": 250,
                    "upstream_retry_max_delay_ms": 2000
                }
            ],
            "project_preferences": [
                {
                    "project_id": "project_local_demo",
                    "preset_id": "profile-global-balanced",
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-openrouter-main",
                        "provider-ollama-local"
                    ],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "preferred_region": "global",
                    "updated_at_ms": 1710000000000u64
                }
            ]
        }),
    );
    let mut snapshot = compiled_routing_snapshot_fixture(snapshot_id);
    snapshot["route_key"] = serde_json::json!("qwen-plus-latest");
    snapshot["ordered_provider_ids"] = serde_json::json!(["provider-siliconflow-main"]);
    snapshot["default_provider_id"] = serde_json::json!("provider-siliconflow-main");
    write_json(
        &bootstrap_root.join("observability").join("default.json"),
        &observability_fixture(snapshot, vec![]),
    );
    write_json(
        &bootstrap_root.join("billing").join("default.json"),
        &billing_fixture_with_local_metering_support(vec![]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject compiled routing snapshots whose provider is outside the applied routing profile"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains(snapshot_id), "{error}");
    assert!(error.contains("provider-siliconflow-main"), "{error}");
    assert!(error.contains("profile-global-balanced"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_compiled_snapshot_default_provider_without_active_price_coverage(
) {
    let snapshot_id = "snapshot-invalid-default-provider-price-coverage";
    let mut snapshot = compiled_routing_snapshot_fixture(snapshot_id);
    snapshot["default_provider_id"] = serde_json::json!("provider-siliconflow-main");

    let error = bootstrap_error_from_profile_pack_override(
        "profile-pack-invalid-snapshot-default-provider-price-coverage",
        &observability_fixture(snapshot, vec![]),
        &billing_fixture(vec![]),
    )
    .await;

    assert!(error.contains(snapshot_id), "{error}");
    assert!(error.contains("provider-siliconflow-main"), "{error}");
    assert!(error.contains("gpt-4.1"), "{error}");
    assert!(error.contains("price"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_compiled_snapshot_updated_before_created(
) {
    let snapshot_id = "snapshot-invalid-updated-before-created";
    let mut snapshot = compiled_routing_snapshot_fixture(snapshot_id);
    snapshot["updated_at_ms"] = serde_json::json!(1710002999999u64);

    let error = bootstrap_error_from_profile_pack_override(
        "profile-pack-invalid-snapshot-updated-before-created",
        &observability_fixture(snapshot, vec![]),
        &billing_fixture(vec![]),
    )
    .await;

    assert!(error.contains(snapshot_id), "{error}");
    assert!(error.contains("updated_at_ms"), "{error}");
    assert!(error.contains("created_at_ms"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_deterministic_snapshot_default_provider_not_first(
) {
    let snapshot_id = "snapshot-invalid-deterministic-default-order";
    let mut snapshot = compiled_routing_snapshot_fixture(snapshot_id);
    snapshot["strategy"] = serde_json::json!("deterministic_priority");
    snapshot["ordered_provider_ids"] = serde_json::json!([
        "provider-siliconflow-main",
        "provider-openrouter-main",
        "provider-ollama-local"
    ]);
    snapshot["default_provider_id"] = serde_json::json!("provider-openrouter-main");

    let error = bootstrap_error_from_profile_pack_override(
        "profile-pack-invalid-snapshot-deterministic-default-order",
        &observability_fixture(snapshot, vec![]),
        &billing_fixture(vec![]),
    )
    .await;

    assert!(error.contains(snapshot_id), "{error}");
    assert!(error.contains("deterministic_priority"), "{error}");
    assert!(error.contains("default_provider_id"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_compiled_snapshot_with_disabled_matched_policy(
) {
    let snapshot_id = "snapshot-invalid-disabled-matched-policy";
    let bootstrap_root = temp_bootstrap_root("profile-pack-invalid-snapshot-disabled-policy");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("routing").join("default.json"),
        &serde_json::json!({
            "profiles": [
                {
                    "profile_id": "profile-global-balanced",
                    "tenant_id": "tenant_local_demo",
                    "project_id": "project_local_demo",
                    "name": "Global Balanced",
                    "slug": "global-balanced",
                    "description": "Balanced multi-provider routing",
                    "active": true,
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-openrouter-main",
                        "provider-siliconflow-main",
                        "provider-ollama-local"
                    ],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "preferred_region": "global",
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ],
            "policies": [
                {
                    "policy_id": "policy-default-responses",
                    "capability": "responses",
                    "model_pattern": "*",
                    "enabled": false,
                    "priority": 100,
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-openrouter-main",
                        "provider-siliconflow-main",
                        "provider-ollama-local"
                    ],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "execution_failover_enabled": true,
                    "upstream_retry_max_attempts": 3,
                    "upstream_retry_base_delay_ms": 250,
                    "upstream_retry_max_delay_ms": 2000
                }
            ],
            "project_preferences": [
                {
                    "project_id": "project_local_demo",
                    "preset_id": "profile-global-balanced",
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-openrouter-main",
                        "provider-siliconflow-main",
                        "provider-ollama-local"
                    ],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "preferred_region": "global",
                    "updated_at_ms": 1710000000000u64
                }
            ]
        }),
    );
    write_json(
        &bootstrap_root.join("observability").join("default.json"),
        &observability_fixture(compiled_routing_snapshot_fixture(snapshot_id), vec![]),
    );
    write_json(
        &bootstrap_root.join("billing").join("default.json"),
        &billing_fixture_with_local_metering_support(vec![]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject compiled routing snapshots whose matched policy is disabled"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains(snapshot_id), "{error}");
    assert!(error.contains("policy-default-responses"), "{error}");
    assert!(error.contains("disabled"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_compiled_snapshot_with_mismatched_matched_policy(
) {
    let snapshot_id = "snapshot-invalid-mismatched-matched-policy";
    let bootstrap_root = temp_bootstrap_root("profile-pack-invalid-snapshot-mismatched-policy");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("routing").join("default.json"),
        &serde_json::json!({
            "profiles": [
                {
                    "profile_id": "profile-global-balanced",
                    "tenant_id": "tenant_local_demo",
                    "project_id": "project_local_demo",
                    "name": "Global Balanced",
                    "slug": "global-balanced",
                    "description": "Balanced multi-provider routing",
                    "active": true,
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-openrouter-main",
                        "provider-siliconflow-main",
                        "provider-ollama-local"
                    ],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "preferred_region": "global",
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ],
            "policies": [
                {
                    "policy_id": "policy-default-responses",
                    "capability": "responses",
                    "model_pattern": "deepseek-*",
                    "enabled": true,
                    "priority": 100,
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-openrouter-main"
                    ],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "execution_failover_enabled": true,
                    "upstream_retry_max_attempts": 3,
                    "upstream_retry_base_delay_ms": 250,
                    "upstream_retry_max_delay_ms": 2000
                }
            ],
            "project_preferences": [
                {
                    "project_id": "project_local_demo",
                    "preset_id": "profile-global-balanced",
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-openrouter-main",
                        "provider-siliconflow-main",
                        "provider-ollama-local"
                    ],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "preferred_region": "global",
                    "updated_at_ms": 1710000000000u64
                }
            ]
        }),
    );
    write_json(
        &bootstrap_root.join("observability").join("default.json"),
        &observability_fixture(compiled_routing_snapshot_fixture(snapshot_id), vec![]),
    );
    write_json(
        &bootstrap_root.join("billing").join("default.json"),
        &billing_fixture_with_local_metering_support(vec![]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject compiled routing snapshots whose matched policy does not match the snapshot capability and route key"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains(snapshot_id), "{error}");
    assert!(error.contains("policy-default-responses"), "{error}");
    assert!(error.contains("deepseek-*"), "{error}");
    assert!(error.contains("gpt-4.1"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_decision_selected_provider_with_only_foreign_tenant_account(
) {
    let decision_id = "decision-invalid-foreign-tenant-selected-provider";
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-decision-foreign-tenant-selected-provider");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("tenants").join("default.json"),
        &serde_json::json!([
            { "id": "tenant_local_demo", "name": "Local Demo Workspace" },
            { "id": "tenant_other_demo", "name": "Other Demo Workspace" }
        ]),
    );
    write_json(
        &bootstrap_root
            .join("provider-accounts")
            .join("default.json"),
        &foreign_tenant_siliconflow_provider_accounts_fixture(),
    );
    write_json(
        &bootstrap_root.join("routing").join("default.json"),
        &routing_fixture_without_siliconflow_candidates(),
    );
    let mut decision = routing_decision_log_fixture(decision_id, "snapshot-unused");
    decision["compiled_routing_snapshot_id"] = serde_json::Value::Null;
    decision["selected_provider_id"] = serde_json::json!("provider-siliconflow-main");
    decision["assessments"] = serde_json::json!([
        {
            "provider_id": "provider-siliconflow-main",
            "available": true,
            "health": "healthy",
            "policy_rank": 0,
            "weight": 100u64,
            "cost": 0.31,
            "latency_ms": 610u64,
            "region": "cn",
            "region_match": false,
            "reasons": ["selected provider should be rejected before this assessment matters"]
        }
    ]);
    write_json(
        &bootstrap_root.join("observability").join("default.json"),
        &serde_json::json!({
            "compiled_routing_snapshots": [],
            "routing_decision_logs": [decision],
            "provider_health_snapshots": []
        }),
    );
    write_json(
        &bootstrap_root.join("billing").join("default.json"),
        &billing_fixture_with_local_metering_support(vec![]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject routing decision selected providers whose tenant only has a foreign tenant account"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains(decision_id), "{error}");
    assert!(error.contains("provider-siliconflow-main"), "{error}");
    assert!(error.contains("provider account"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_decision_selected_provider_without_active_price_coverage(
) {
    let snapshot_id = "snapshot-selected-provider-price-coverage";
    let decision_id = "decision-invalid-selected-provider-price-coverage";
    let mut decision = routing_decision_log_fixture(decision_id, snapshot_id);
    decision["selected_provider_id"] = serde_json::json!("provider-siliconflow-main");
    decision["assessments"] = serde_json::json!([
        {
            "provider_id": "provider-siliconflow-main",
            "available": true,
            "health": "healthy",
            "policy_rank": 0,
            "weight": 100u64,
            "cost": 0.31,
            "latency_ms": 610u64,
            "region": "cn",
            "region_match": false,
            "reasons": ["selected provider lacks active route pricing"]
        }
    ]);

    let error = bootstrap_error_from_profile_pack_override(
        "profile-pack-invalid-decision-selected-provider-price-coverage",
        &observability_fixture(compiled_routing_snapshot_fixture(snapshot_id), vec![decision]),
        &billing_fixture(vec![]),
    )
    .await;

    assert!(error.contains(decision_id), "{error}");
    assert!(error.contains("provider-siliconflow-main"), "{error}");
    assert!(error.contains("gpt-4.1"), "{error}");
    assert!(error.contains("price"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_decision_assessment_provider_with_only_foreign_tenant_account(
) {
    let decision_id = "decision-invalid-foreign-tenant-assessment-provider";
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-decision-foreign-tenant-assessment-provider");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("tenants").join("default.json"),
        &serde_json::json!([
            { "id": "tenant_local_demo", "name": "Local Demo Workspace" },
            { "id": "tenant_other_demo", "name": "Other Demo Workspace" }
        ]),
    );
    write_json(
        &bootstrap_root
            .join("provider-accounts")
            .join("default.json"),
        &foreign_tenant_siliconflow_provider_accounts_fixture(),
    );
    write_json(
        &bootstrap_root.join("routing").join("default.json"),
        &routing_fixture_without_siliconflow_candidates(),
    );
    let mut decision = routing_decision_log_fixture(decision_id, "snapshot-unused");
    decision["compiled_routing_snapshot_id"] = serde_json::Value::Null;
    decision["assessments"] = serde_json::json!([
        {
            "provider_id": "provider-openrouter-main",
            "available": true,
            "health": "healthy",
            "policy_rank": 0,
            "weight": 60u64,
            "cost": 0.27,
            "latency_ms": 540u64,
            "region": "global",
            "region_match": true,
            "reasons": ["primary route"]
        },
        {
            "provider_id": "provider-siliconflow-main",
            "available": true,
            "health": "healthy",
            "policy_rank": 1,
            "weight": 40u64,
            "cost": 0.31,
            "latency_ms": 610u64,
            "region": "cn",
            "region_match": false,
            "reasons": ["foreign tenant provider should be rejected"]
        }
    ]);
    write_json(
        &bootstrap_root.join("observability").join("default.json"),
        &serde_json::json!({
            "compiled_routing_snapshots": [],
            "routing_decision_logs": [decision],
            "provider_health_snapshots": []
        }),
    );
    write_json(
        &bootstrap_root.join("billing").join("default.json"),
        &billing_fixture_with_local_metering_support(vec![]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject routing decision assessment providers whose tenant only has a foreign tenant account"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains(decision_id), "{error}");
    assert!(error.contains("provider-siliconflow-main"), "{error}");
    assert!(error.contains("provider account"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_decision_capability_mismatched_snapshot()
{
    let snapshot_id = "snapshot-invalid-decision-capability";
    let decision_id = "decision-invalid-snapshot-capability";
    let mut snapshot = compiled_routing_snapshot_fixture(snapshot_id);
    snapshot["matched_policy_id"] = serde_json::Value::Null;
    let mut decision = routing_decision_log_fixture(decision_id, snapshot_id);
    decision["capability"] = serde_json::json!("chat_completions");
    decision["matched_policy_id"] = serde_json::Value::Null;

    let error = bootstrap_error_from_profile_pack_override(
        "profile-pack-invalid-decision-capability-snapshot",
        &observability_fixture(snapshot, vec![decision]),
        &billing_fixture(vec![]),
    )
    .await;

    assert!(error.contains(decision_id), "{error}");
    assert!(error.contains("capability"), "{error}");
    assert!(error.contains(snapshot_id), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_decision_route_key_mismatched_snapshot()
{
    let snapshot_id = "snapshot-invalid-decision-route-key";
    let decision_id = "decision-invalid-snapshot-route-key";
    let mut decision = routing_decision_log_fixture(decision_id, snapshot_id);
    decision["route_key"] = serde_json::json!("deepseek-chat");

    let error = bootstrap_error_from_profile_pack_override(
        "profile-pack-invalid-decision-route-key-snapshot",
        &observability_fixture(
            compiled_routing_snapshot_fixture(snapshot_id),
            vec![decision],
        ),
        &billing_fixture(vec![]),
    )
    .await;

    assert!(error.contains(decision_id), "{error}");
    assert!(error.contains("route_key"), "{error}");
    assert!(error.contains(snapshot_id), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_decision_strategy_mismatched_snapshot() {
    let snapshot_id = "snapshot-invalid-decision-strategy";
    let decision_id = "decision-invalid-snapshot-strategy";
    let mut decision = routing_decision_log_fixture(decision_id, snapshot_id);
    decision["strategy"] = serde_json::json!("latency_first");

    let error = bootstrap_error_from_profile_pack_override(
        "profile-pack-invalid-decision-strategy-snapshot",
        &observability_fixture(
            compiled_routing_snapshot_fixture(snapshot_id),
            vec![decision],
        ),
        &billing_fixture(vec![]),
    )
    .await;

    assert!(error.contains(decision_id), "{error}");
    assert!(error.contains("strategy"), "{error}");
    assert!(error.contains(snapshot_id), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_decision_policy_mismatched_snapshot() {
    let snapshot_id = "snapshot-invalid-decision-policy";
    let decision_id = "decision-invalid-snapshot-policy";
    let mut decision = routing_decision_log_fixture(decision_id, snapshot_id);
    decision["matched_policy_id"] = serde_json::Value::Null;

    let error = bootstrap_error_from_profile_pack_override(
        "profile-pack-invalid-decision-policy-snapshot",
        &observability_fixture(
            compiled_routing_snapshot_fixture(snapshot_id),
            vec![decision],
        ),
        &billing_fixture(vec![]),
    )
    .await;

    assert!(error.contains(decision_id), "{error}");
    assert!(error.contains("matched_policy_id"), "{error}");
    assert!(error.contains(snapshot_id), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_decision_with_disabled_matched_policy() {
    let decision_id = "decision-invalid-disabled-matched-policy";
    let bootstrap_root = temp_bootstrap_root("profile-pack-invalid-decision-disabled-policy");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("routing").join("default.json"),
        &serde_json::json!({
            "profiles": [
                {
                    "profile_id": "profile-global-balanced",
                    "tenant_id": "tenant_local_demo",
                    "project_id": "project_local_demo",
                    "name": "Global Balanced",
                    "slug": "global-balanced",
                    "description": "Balanced multi-provider routing",
                    "active": true,
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-openrouter-main"
                    ],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "preferred_region": "global",
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ],
            "policies": [
                {
                    "policy_id": "policy-default-responses",
                    "capability": "responses",
                    "model_pattern": "*",
                    "enabled": false,
                    "priority": 100,
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-openrouter-main"
                    ],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "execution_failover_enabled": true,
                    "upstream_retry_max_attempts": 3,
                    "upstream_retry_base_delay_ms": 250,
                    "upstream_retry_max_delay_ms": 2000
                }
            ],
            "project_preferences": [
                {
                    "project_id": "project_local_demo",
                    "preset_id": "profile-global-balanced",
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-openrouter-main"
                    ],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "preferred_region": "global",
                    "updated_at_ms": 1710000000000u64
                }
            ]
        }),
    );
    let mut decision = routing_decision_log_fixture(decision_id, "snapshot-unused");
    decision["compiled_routing_snapshot_id"] = serde_json::Value::Null;
    write_json(
        &bootstrap_root.join("observability").join("default.json"),
        &serde_json::json!({
            "compiled_routing_snapshots": [],
            "routing_decision_logs": [decision],
            "provider_health_snapshots": []
        }),
    );
    write_json(
        &bootstrap_root.join("billing").join("default.json"),
        &billing_fixture_with_local_metering_support(vec![]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject routing decision logs whose matched policy is disabled"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains(decision_id), "{error}");
    assert!(error.contains("policy-default-responses"), "{error}");
    assert!(error.contains("disabled"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_decision_with_mismatched_matched_policy()
{
    let decision_id = "decision-invalid-mismatched-matched-policy";
    let bootstrap_root = temp_bootstrap_root("profile-pack-invalid-decision-mismatched-policy");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("routing").join("default.json"),
        &serde_json::json!({
            "profiles": [
                {
                    "profile_id": "profile-global-balanced",
                    "tenant_id": "tenant_local_demo",
                    "project_id": "project_local_demo",
                    "name": "Global Balanced",
                    "slug": "global-balanced",
                    "description": "Balanced multi-provider routing",
                    "active": true,
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-openrouter-main"
                    ],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "preferred_region": "global",
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ],
            "policies": [
                {
                    "policy_id": "policy-default-responses",
                    "capability": "responses",
                    "model_pattern": "deepseek-*",
                    "enabled": true,
                    "priority": 100,
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-openrouter-main"
                    ],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "execution_failover_enabled": true,
                    "upstream_retry_max_attempts": 3,
                    "upstream_retry_base_delay_ms": 250,
                    "upstream_retry_max_delay_ms": 2000
                }
            ],
            "project_preferences": [
                {
                    "project_id": "project_local_demo",
                    "preset_id": "profile-global-balanced",
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-openrouter-main"
                    ],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "preferred_region": "global",
                    "updated_at_ms": 1710000000000u64
                }
            ]
        }),
    );
    let mut decision = routing_decision_log_fixture(decision_id, "snapshot-unused");
    decision["compiled_routing_snapshot_id"] = serde_json::Value::Null;
    write_json(
        &bootstrap_root.join("observability").join("default.json"),
        &serde_json::json!({
            "compiled_routing_snapshots": [],
            "routing_decision_logs": [decision],
            "provider_health_snapshots": []
        }),
    );
    write_json(
        &bootstrap_root.join("billing").join("default.json"),
        &billing_fixture_with_local_metering_support(vec![]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject routing decision logs whose matched policy does not match the decision capability and route key"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains(decision_id), "{error}");
    assert!(error.contains("policy-default-responses"), "{error}");
    assert!(error.contains("deepseek-*"), "{error}");
    assert!(error.contains("gpt-4.1"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_decision_provider_outside_applied_routing_profile(
) {
    let decision_id = "decision-invalid-profile-provider";
    let bootstrap_root = temp_bootstrap_root("profile-pack-invalid-decision-profile-provider");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("routing").join("default.json"),
        &serde_json::json!({
            "profiles": [
                {
                    "profile_id": "profile-global-balanced",
                    "tenant_id": "tenant_local_demo",
                    "project_id": "project_local_demo",
                    "name": "Global Balanced",
                    "slug": "global-balanced",
                    "description": "Balanced multi-provider routing",
                    "active": true,
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-openrouter-main",
                        "provider-ollama-local"
                    ],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "preferred_region": "global",
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ],
            "policies": [
                {
                    "policy_id": "policy-default-responses",
                    "capability": "responses",
                    "model_pattern": "*",
                    "enabled": true,
                    "priority": 100,
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-openrouter-main",
                        "provider-ollama-local"
                    ],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "execution_failover_enabled": true,
                    "upstream_retry_max_attempts": 3,
                    "upstream_retry_base_delay_ms": 250,
                    "upstream_retry_max_delay_ms": 2000
                }
            ],
            "project_preferences": [
                {
                    "project_id": "project_local_demo",
                    "preset_id": "profile-global-balanced",
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-openrouter-main",
                        "provider-ollama-local"
                    ],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "preferred_region": "global",
                    "updated_at_ms": 1710000000000u64
                }
            ]
        }),
    );
    let mut decision = routing_decision_log_fixture(decision_id, "snapshot-unused");
    decision["compiled_routing_snapshot_id"] = serde_json::Value::Null;
    decision["route_key"] = serde_json::json!("qwen-plus-latest");
    decision["selected_provider_id"] = serde_json::json!("provider-siliconflow-main");
    decision["assessments"] = serde_json::json!([
        {
            "provider_id": "provider-siliconflow-main",
            "available": true,
            "health": "healthy",
            "policy_rank": 0,
            "weight": 100u64,
            "cost": 0.31,
            "latency_ms": 610u64,
            "region": "cn",
            "region_match": false,
            "reasons": ["provider should be rejected before assessment relevance matters"]
        }
    ]);
    write_json(
        &bootstrap_root.join("observability").join("default.json"),
        &serde_json::json!({
            "compiled_routing_snapshots": [],
            "routing_decision_logs": [decision],
            "provider_health_snapshots": []
        }),
    );
    write_json(
        &bootstrap_root.join("billing").join("default.json"),
        &billing_fixture_with_local_metering_support(vec![]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject routing decision logs whose provider is outside the applied routing profile"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains(decision_id), "{error}");
    assert!(error.contains("provider-siliconflow-main"), "{error}");
    assert!(error.contains("profile-global-balanced"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_decision_assessment_provider_outside_applied_routing_profile(
) {
    let decision_id = "decision-invalid-profile-assessment-provider";
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-decision-profile-assessment-provider");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("routing").join("default.json"),
        &serde_json::json!({
            "profiles": [
                {
                    "profile_id": "profile-global-balanced",
                    "tenant_id": "tenant_local_demo",
                    "project_id": "project_local_demo",
                    "name": "Global Balanced",
                    "slug": "global-balanced",
                    "description": "Balanced multi-provider routing",
                    "active": true,
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-openrouter-main",
                        "provider-ollama-local"
                    ],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "preferred_region": "global",
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ],
            "policies": [
                {
                    "policy_id": "policy-default-responses",
                    "capability": "responses",
                    "model_pattern": "*",
                    "enabled": true,
                    "priority": 100,
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-openrouter-main",
                        "provider-ollama-local"
                    ],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "execution_failover_enabled": true,
                    "upstream_retry_max_attempts": 3,
                    "upstream_retry_base_delay_ms": 250,
                    "upstream_retry_max_delay_ms": 2000
                }
            ],
            "project_preferences": [
                {
                    "project_id": "project_local_demo",
                    "preset_id": "profile-global-balanced",
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-openrouter-main",
                        "provider-ollama-local"
                    ],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "preferred_region": "global",
                    "updated_at_ms": 1710000000000u64
                }
            ]
        }),
    );
    let mut decision = routing_decision_log_fixture(decision_id, "snapshot-unused");
    decision["compiled_routing_snapshot_id"] = serde_json::Value::Null;
    decision["selected_provider_id"] = serde_json::json!("provider-openrouter-main");
    decision["assessments"] = serde_json::json!([
        {
            "provider_id": "provider-openrouter-main",
            "available": true,
            "health": "healthy",
            "policy_rank": 0,
            "weight": 60u64,
            "cost": 0.27,
            "latency_ms": 540u64,
            "region": "global",
            "region_match": true,
            "reasons": ["primary route"]
        },
        {
            "provider_id": "provider-siliconflow-main",
            "available": true,
            "health": "healthy",
            "policy_rank": 1,
            "weight": 40u64,
            "cost": 0.19,
            "latency_ms": 610u64,
            "region": "cn",
            "region_match": false,
            "reasons": ["assessment provider should be rejected outside profile"]
        }
    ]);
    write_json(
        &bootstrap_root.join("observability").join("default.json"),
        &serde_json::json!({
            "compiled_routing_snapshots": [],
            "routing_decision_logs": [decision],
            "provider_health_snapshots": []
        }),
    );
    write_json(
        &bootstrap_root.join("billing").join("default.json"),
        &billing_fixture_with_local_metering_support(vec![]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject routing decision assessments whose provider is outside the applied routing profile"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains(decision_id), "{error}");
    assert!(error.contains("provider-siliconflow-main"), "{error}");
    assert!(error.contains("profile-global-balanced"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_decision_without_selected_provider_assessment(
) {
    let snapshot_id = "snapshot-invalid-decision-selected-assessment";
    let decision_id = "decision-missing-selected-assessment";
    let mut decision = routing_decision_log_fixture(decision_id, snapshot_id);
    decision["assessments"] = serde_json::json!([
        {
            "provider_id": "provider-siliconflow-main",
            "available": true,
            "health": "healthy",
            "policy_rank": 1,
            "weight": 40u64,
            "cost": 0.19,
            "latency_ms": 610u64,
            "region": "cn",
            "region_match": false,
            "reasons": ["missing selected provider evidence"]
        }
    ]);

    let error = bootstrap_error_from_profile_pack_override(
        "profile-pack-invalid-decision-missing-selected-assessment",
        &observability_fixture(
            compiled_routing_snapshot_fixture(snapshot_id),
            vec![decision],
        ),
        &billing_fixture(vec![]),
    )
    .await;

    assert!(error.contains(decision_id), "{error}");
    assert!(error.contains("provider-openrouter-main"), "{error}");
    assert!(error.contains("assessment"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_decision_with_unavailable_selected_provider_assessment(
) {
    let snapshot_id = "snapshot-invalid-decision-selected-unavailable";
    let decision_id = "decision-selected-provider-unavailable";
    let mut decision = routing_decision_log_fixture(decision_id, snapshot_id);
    decision["assessments"] = serde_json::json!([
        {
            "provider_id": "provider-openrouter-main",
            "available": false,
            "health": "healthy",
            "policy_rank": 0,
            "weight": 60u64,
            "cost": 0.27,
            "latency_ms": 540u64,
            "region": "global",
            "region_match": true,
            "reasons": ["selected provider was not actually executable"]
        }
    ]);

    let error = bootstrap_error_from_profile_pack_override(
        "profile-pack-invalid-decision-selected-unavailable",
        &observability_fixture(
            compiled_routing_snapshot_fixture(snapshot_id),
            vec![decision],
        ),
        &billing_fixture(vec![]),
    )
    .await;

    assert!(error.contains(decision_id), "{error}");
    assert!(error.contains("provider-openrouter-main"), "{error}");
    assert!(error.contains("available"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_decision_with_unhealthy_selected_provider_when_snapshot_requires_healthy(
) {
    let snapshot_id = "snapshot-invalid-decision-selected-unhealthy";
    let decision_id = "decision-selected-provider-unhealthy";
    let mut snapshot = compiled_routing_snapshot_fixture(snapshot_id);
    snapshot["require_healthy"] = serde_json::json!(true);
    let mut decision = routing_decision_log_fixture(decision_id, snapshot_id);
    decision["assessments"] = serde_json::json!([
        {
            "provider_id": "provider-openrouter-main",
            "available": true,
            "health": "unknown",
            "policy_rank": 0,
            "weight": 60u64,
            "cost": 0.27,
            "latency_ms": 540u64,
            "region": "global",
            "region_match": true,
            "reasons": ["healthy-only route should reject non-healthy evidence"]
        }
    ]);

    let error = bootstrap_error_from_profile_pack_override(
        "profile-pack-invalid-decision-selected-unhealthy",
        &observability_fixture(snapshot, vec![decision]),
        &billing_fixture(vec![]),
    )
    .await;

    assert!(error.contains(decision_id), "{error}");
    assert!(error.contains("provider-openrouter-main"), "{error}");
    assert!(error.contains("healthy"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_billing_capability_mismatched_snapshot()
{
    let snapshot_id = "snapshot-invalid-billing-capability";
    let event_id = "billing-invalid-snapshot-capability";
    let mut event = billing_event_fixture(event_id, snapshot_id);
    event["capability"] = serde_json::json!("chat_completions");

    let error = bootstrap_error_from_profile_pack_override(
        "profile-pack-invalid-billing-capability-snapshot",
        &observability_fixture(compiled_routing_snapshot_fixture(snapshot_id), vec![]),
        &billing_fixture(vec![event]),
    )
    .await;

    assert!(error.contains(event_id), "{error}");
    assert!(error.contains("capability"), "{error}");
    assert!(error.contains(snapshot_id), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_billing_route_key_mismatched_snapshot() {
    let snapshot_id = "snapshot-invalid-billing-route-key";
    let event_id = "billing-invalid-snapshot-route-key";
    let mut event = billing_event_fixture(event_id, snapshot_id);
    event["route_key"] = serde_json::json!("deepseek-chat");
    event["usage_model"] = serde_json::json!("deepseek-chat");

    let error = bootstrap_error_from_profile_pack_override(
        "profile-pack-invalid-billing-route-key-snapshot",
        &observability_fixture(compiled_routing_snapshot_fixture(snapshot_id), vec![]),
        &billing_fixture(vec![event]),
    )
    .await;

    assert!(error.contains(event_id), "{error}");
    assert!(error.contains("route_key"), "{error}");
    assert!(error.contains(snapshot_id), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_billing_profile_mismatched_snapshot() {
    let snapshot_id = "snapshot-invalid-billing-profile";
    let event_id = "billing-invalid-snapshot-profile";
    let mut event = billing_event_fixture(event_id, snapshot_id);
    event["applied_routing_profile_id"] = serde_json::Value::Null;

    let error = bootstrap_error_from_profile_pack_override(
        "profile-pack-invalid-billing-profile-snapshot",
        &observability_fixture(compiled_routing_snapshot_fixture(snapshot_id), vec![]),
        &billing_fixture(vec![event]),
    )
    .await;

    assert!(error.contains(event_id), "{error}");
    assert!(error.contains("applied_routing_profile_id"), "{error}");
    assert!(error.contains(snapshot_id), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_billing_provider_outside_applied_routing_profile(
) {
    let bootstrap_root = temp_bootstrap_root("profile-pack-invalid-billing-profile-provider");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("routing").join("default.json"),
        &serde_json::json!({
            "profiles": [
                {
                    "profile_id": "profile-global-balanced",
                    "tenant_id": "tenant_local_demo",
                    "project_id": "project_local_demo",
                    "name": "Global Balanced",
                    "slug": "global-balanced",
                    "description": "Balanced multi-provider routing",
                    "active": true,
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-openrouter-main",
                        "provider-ollama-local"
                    ],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "preferred_region": "global",
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ],
            "policies": [
                {
                    "policy_id": "policy-default-responses",
                    "capability": "responses",
                    "model_pattern": "*",
                    "enabled": true,
                    "priority": 100,
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-openrouter-main",
                        "provider-ollama-local"
                    ],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "execution_failover_enabled": true,
                    "upstream_retry_max_attempts": 3,
                    "upstream_retry_base_delay_ms": 250,
                    "upstream_retry_max_delay_ms": 2000
                }
            ],
            "project_preferences": [
                {
                    "project_id": "project_local_demo",
                    "preset_id": "profile-global-balanced",
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-openrouter-main",
                        "provider-ollama-local"
                    ],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "preferred_region": "global",
                    "updated_at_ms": 1710000000000u64
                }
            ]
        }),
    );
    write_json(
        &bootstrap_root.join("observability").join("default.json"),
        &serde_json::json!({
            "compiled_routing_snapshots": [],
            "routing_decision_logs": [],
            "provider_health_snapshots": []
        }),
    );
    write_json(
        &bootstrap_root.join("billing").join("default.json"),
        &billing_fixture_with_local_metering_support(vec![serde_json::json!({
            "event_id": "billing-invalid-profile-provider",
            "tenant_id": "tenant_local_demo",
            "project_id": "project_local_demo",
            "api_key_group_id": "group-local-demo-live",
            "capability": "responses",
            "route_key": "qwen-plus-latest",
            "usage_model": "qwen-plus-latest",
            "provider_id": "provider-siliconflow-main",
            "accounting_mode": "platform_credit",
            "operation_kind": "request",
            "modality": "text",
            "api_key_hash": "a19d2bf76318aa7f619d684271469bb383faf1cb5bd4c680088465cde9d0003b",
            "channel_id": "siliconflow",
            "reference_id": "req_invalid_profile_provider",
            "latency_ms": 420u64,
            "units": 1u64,
            "request_count": 1u64,
            "input_tokens": 1200u64,
            "output_tokens": 400u64,
            "total_tokens": 1600u64,
            "cache_read_tokens": 0u64,
            "cache_write_tokens": 0u64,
            "image_count": 0u64,
            "audio_seconds": 0.0,
            "video_seconds": 0.0,
            "music_seconds": 0.0,
            "upstream_cost": 0.27,
            "customer_charge": 0.59,
            "applied_routing_profile_id": "profile-global-balanced",
            "compiled_routing_snapshot_id": null,
            "fallback_reason": null,
            "created_at_ms": 1710008000000u64
        })]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject billing events whose provider is outside the applied routing profile"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("billing-invalid-profile-provider"), "{error}");
    assert!(error.contains("provider-siliconflow-main"), "{error}");
    assert!(error.contains("profile-global-balanced"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_request_meter_fact_with_inactive_cost_pricing_plan(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-request-meter-inactive-cost-pricing-plan");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("pricing").join("default.json"),
        &serde_json::json!({
            "plans": [
                {
                    "pricing_plan_id": 9101u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "plan_code": "global-default-commercial",
                    "plan_version": 1u64,
                    "display_name": "Global Default Commercial",
                    "currency_code": "USD",
                    "credit_unit_code": "credit",
                    "status": "archived",
                    "effective_from_ms": 1710000000000u64,
                    "effective_to_ms": null,
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ],
            "rates": [
                {
                    "pricing_rate_id": 9201u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "pricing_plan_id": 9101u64,
                    "metric_code": "tokens.input",
                    "capability_code": "responses",
                    "model_code": null,
                    "provider_code": null,
                    "charge_unit": "1k_tokens",
                    "pricing_method": "per_unit",
                    "quantity_step": 1.0,
                    "unit_price": 0.002,
                    "display_price_unit": "$ / 1K input tokens",
                    "minimum_billable_quantity": 0.0,
                    "minimum_charge": 0.0,
                    "rounding_increment": 1.0,
                    "rounding_mode": "none",
                    "included_quantity": 0.0,
                    "priority": 10u64,
                    "notes": "default responses input pricing",
                    "status": "active",
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ]
        }),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject request meter facts whose cost_pricing_plan_id is not active"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("6001"), "{error}");
    assert!(error.contains("cost pricing plan"), "{error}");
    assert!(error.contains("9101"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_request_meter_fact_with_inactive_retail_pricing_plan(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-request-meter-inactive-retail-pricing-plan");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("pricing").join("default.json"),
        &serde_json::json!({
            "plans": [
                {
                    "pricing_plan_id": 9101u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "plan_code": "global-default-commercial",
                    "plan_version": 1u64,
                    "display_name": "Global Default Commercial",
                    "currency_code": "USD",
                    "credit_unit_code": "credit",
                    "status": "archived",
                    "effective_from_ms": 1710000000000u64,
                    "effective_to_ms": null,
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ],
            "rates": [
                {
                    "pricing_rate_id": 9201u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "pricing_plan_id": 9101u64,
                    "metric_code": "tokens.input",
                    "capability_code": "responses",
                    "model_code": null,
                    "provider_code": null,
                    "charge_unit": "1k_tokens",
                    "pricing_method": "per_unit",
                    "quantity_step": 1.0,
                    "unit_price": 0.002,
                    "display_price_unit": "$ / 1K input tokens",
                    "minimum_billable_quantity": 0.0,
                    "minimum_charge": 0.0,
                    "rounding_increment": 1.0,
                    "rounding_mode": "none",
                    "included_quantity": 0.0,
                    "priority": 10u64,
                    "notes": "default responses input pricing",
                    "status": "active",
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ]
        }),
    );
    write_json(
        &bootstrap_root.join("request-metering").join("default.json"),
        &serde_json::json!({
            "facts": [
                {
                    "request_id": 6001u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "user_id": 9001u64,
                    "account_id": 7001u64,
                    "api_key_id": 10001u64,
                    "api_key_hash": "a19d2bf76318aa7f619d684271469bb383faf1cb5bd4c680088465cde9d0003b",
                    "auth_type": "api_key",
                    "jwt_subject": null,
                    "platform": "portal",
                    "owner": "user_local_demo",
                    "request_trace_id": "trace-local-demo-growth-2026",
                    "gateway_request_ref": null,
                    "upstream_request_ref": null,
                    "protocol_family": "openai",
                    "capability_code": "responses",
                    "channel_code": "siliconflow",
                    "model_code": "qwen-plus-latest",
                    "provider_code": "provider-siliconflow-main",
                    "request_status": "succeeded",
                    "usage_capture_status": "captured",
                    "cost_pricing_plan_id": null,
                    "retail_pricing_plan_id": 9101u64,
                    "estimated_credit_hold": 2400.0,
                    "actual_credit_charge": 2300.0,
                    "actual_provider_cost": 0.27,
                    "started_at_ms": 1710005500000u64,
                    "finished_at_ms": 1710005500900u64,
                    "created_at_ms": 1710005500000u64,
                    "updated_at_ms": 1710005500900u64
                }
            ],
            "metrics": [
                {
                    "request_metric_id": 7001001u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "request_id": 6001u64,
                    "metric_code": "token.input",
                    "quantity": 1800.0,
                    "provider_field": "prompt_tokens",
                    "source_kind": "provider",
                    "capture_stage": "final",
                    "is_billable": true,
                    "captured_at_ms": 1710005500850u64
                },
                {
                    "request_metric_id": 7001002u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "request_id": 6001u64,
                    "metric_code": "token.output",
                    "quantity": 600.0,
                    "provider_field": "completion_tokens",
                    "source_kind": "provider",
                    "capture_stage": "final",
                    "is_billable": true,
                    "captured_at_ms": 1710005500850u64
                }
            ]
        }),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject request meter facts whose retail_pricing_plan_id is not active"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("6001"), "{error}");
    assert!(error.contains("retail pricing plan"), "{error}");
    assert!(error.contains("9101"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_request_meter_fact_with_non_shared_cross_workspace_pricing_plan(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-request-meter-cross-workspace-pricing-plan");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("pricing").join("default.json"),
        &serde_json::json!({
            "plans": [
                {
                    "pricing_plan_id": 9101u64,
                    "tenant_id": 1002u64,
                    "organization_id": 2002u64,
                    "plan_code": "global-default-commercial",
                    "plan_version": 1u64,
                    "display_name": "Global Default Commercial",
                    "currency_code": "USD",
                    "credit_unit_code": "credit",
                    "status": "active",
                    "ownership_scope": "workspace",
                    "effective_from_ms": 1710000000000u64,
                    "effective_to_ms": null,
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ],
            "rates": [
                {
                    "pricing_rate_id": 9201u64,
                    "tenant_id": 1002u64,
                    "organization_id": 2002u64,
                    "pricing_plan_id": 9101u64,
                    "metric_code": "tokens.input",
                    "capability_code": "responses",
                    "model_code": null,
                    "provider_code": null,
                    "charge_unit": "1k_tokens",
                    "pricing_method": "per_unit",
                    "quantity_step": 1.0,
                    "unit_price": 0.002,
                    "display_price_unit": "$ / 1K input tokens",
                    "minimum_billable_quantity": 0.0,
                    "minimum_charge": 0.0,
                    "rounding_increment": 1.0,
                    "rounding_mode": "none",
                    "included_quantity": 0.0,
                    "priority": 10u64,
                    "notes": "cross-workspace pricing plan should be rejected for non-shared request lineage",
                    "status": "active",
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ]
        }),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject request meter facts whose pricing plan crosses workspace ownership without shared scope"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("6001"), "{error}");
    assert!(error.contains("9101"), "{error}");
    assert!(error.contains("pricing plan"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_allows_bootstrap_request_meter_fact_with_platform_shared_cross_workspace_pricing_plan(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-valid-request-meter-shared-pricing-plan");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("pricing").join("default.json"),
        &serde_json::json!({
            "plans": [
                {
                    "pricing_plan_id": 9101u64,
                    "tenant_id": 1002u64,
                    "organization_id": 2002u64,
                    "plan_code": "global-default-commercial",
                    "plan_version": 1u64,
                    "display_name": "Global Default Commercial",
                    "currency_code": "USD",
                    "credit_unit_code": "credit",
                    "status": "active",
                    "ownership_scope": "platform_shared",
                    "effective_from_ms": 1710000000000u64,
                    "effective_to_ms": null,
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ],
            "rates": [
                {
                    "pricing_rate_id": 9201u64,
                    "tenant_id": 1002u64,
                    "organization_id": 2002u64,
                    "pricing_plan_id": 9101u64,
                    "metric_code": "tokens.input",
                    "capability_code": "responses",
                    "model_code": null,
                    "provider_code": null,
                    "charge_unit": "1k_tokens",
                    "pricing_method": "per_unit",
                    "quantity_step": 1.0,
                    "unit_price": 0.002,
                    "display_price_unit": "$ / 1K input tokens",
                    "minimum_billable_quantity": 0.0,
                    "minimum_charge": 0.0,
                    "rounding_increment": 1.0,
                    "rounding_mode": "none",
                    "included_quantity": 0.0,
                    "priority": 10u64,
                    "notes": "shared pricing plan should remain allowed across workspaces",
                    "status": "active",
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ]
        }),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    build_admin_store_from_config(&config).await.unwrap();
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_pricing_rate_with_missing_provider_code(
) {
    let bootstrap_root = temp_bootstrap_root("profile-pack-invalid-pricing-rate-provider");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("pricing").join("default.json"),
        &serde_json::json!({
            "plans": [
                {
                    "pricing_plan_id": 9101u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "plan_code": "global-default-commercial",
                    "plan_version": 1u64,
                    "display_name": "Global Default Commercial",
                    "currency_code": "USD",
                    "credit_unit_code": "credit",
                    "status": "active",
                    "effective_from_ms": 1710000000000u64,
                    "effective_to_ms": null,
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ],
            "rates": [
                {
                    "pricing_rate_id": 9201u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "pricing_plan_id": 9101u64,
                    "metric_code": "tokens.input",
                    "capability_code": "responses",
                    "model_code": "gpt-4.1",
                    "provider_code": "provider-missing",
                    "charge_unit": "1k_tokens",
                    "pricing_method": "per_unit",
                    "quantity_step": 1.0,
                    "unit_price": 0.002,
                    "display_price_unit": "$ / 1K input tokens",
                    "minimum_billable_quantity": 0.0,
                    "minimum_charge": 0.0,
                    "rounding_increment": 1.0,
                    "rounding_mode": "none",
                    "included_quantity": 0.0,
                    "priority": 10u64,
                    "notes": "provider-missing responses input pricing",
                    "status": "active",
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ]
        }),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!("bootstrap should reject pricing rates whose provider_code is missing"),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("9201"), "{error}");
    assert!(error.contains("provider-missing"), "{error}");
    assert!(error.contains("provider_code"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_pricing_rate_with_mismatched_parent_plan_workspace(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-pricing-rate-parent-plan-workspace");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("pricing").join("default.json"),
        &serde_json::json!({
            "plans": [
                {
                    "pricing_plan_id": 9101u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "plan_code": "global-default-commercial",
                    "plan_version": 1u64,
                    "display_name": "Global Default Commercial",
                    "currency_code": "USD",
                    "credit_unit_code": "credit",
                    "status": "active",
                    "effective_from_ms": 1710000000000u64,
                    "effective_to_ms": null,
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ],
            "rates": [
                {
                    "pricing_rate_id": 9201u64,
                    "tenant_id": 1002u64,
                    "organization_id": 2002u64,
                    "pricing_plan_id": 9101u64,
                    "metric_code": "tokens.input",
                    "capability_code": null,
                    "model_code": null,
                    "provider_code": null,
                    "charge_unit": "1k_tokens",
                    "pricing_method": "per_unit",
                    "quantity_step": 1.0,
                    "unit_price": 0.002,
                    "display_price_unit": "$ / 1K input tokens",
                    "minimum_billable_quantity": 0.0,
                    "minimum_charge": 0.0,
                    "rounding_increment": 1.0,
                    "rounding_mode": "none",
                    "included_quantity": 0.0,
                    "priority": 10u64,
                    "notes": "invalid cross-workspace pricing rate",
                    "status": "draft",
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ]
        }),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject pricing rates whose tenant or organization ownership drifts from the parent pricing plan"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("9201"), "{error}");
    assert!(error.contains("9101"), "{error}");
    assert!(error.contains("tenant/organization"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_pricing_rate_with_model_not_available_on_provider(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-pricing-rate-provider-model");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("pricing").join("default.json"),
        &serde_json::json!({
            "plans": [
                {
                    "pricing_plan_id": 9101u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "plan_code": "global-default-commercial",
                    "plan_version": 1u64,
                    "display_name": "Global Default Commercial",
                    "currency_code": "USD",
                    "credit_unit_code": "credit",
                    "status": "active",
                    "effective_from_ms": 1710000000000u64,
                    "effective_to_ms": null,
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ],
            "rates": [
                {
                    "pricing_rate_id": 9201u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "pricing_plan_id": 9101u64,
                    "metric_code": "tokens.input",
                    "capability_code": "responses",
                    "model_code": "missing-commercial-model",
                    "provider_code": "provider-openrouter-main",
                    "charge_unit": "1k_tokens",
                    "pricing_method": "per_unit",
                    "quantity_step": 1.0,
                    "unit_price": 0.002,
                    "display_price_unit": "$ / 1K input tokens",
                    "minimum_billable_quantity": 0.0,
                    "minimum_charge": 0.0,
                    "rounding_increment": 1.0,
                    "rounding_mode": "none",
                    "included_quantity": 0.0,
                    "priority": 10u64,
                    "notes": "invalid provider model pricing",
                    "status": "active",
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ]
        }),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject pricing rates whose model_code is not available on the provider"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("9201"), "{error}");
    assert!(error.contains("missing-commercial-model"), "{error}");
    assert!(error.contains("provider-openrouter-main"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_pricing_rate_with_capability_not_supported_by_provider_model(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-pricing-rate-provider-capability");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("pricing").join("default.json"),
        &serde_json::json!({
            "plans": [
                {
                    "pricing_plan_id": 9101u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "plan_code": "global-default-commercial",
                    "plan_version": 1u64,
                    "display_name": "Global Default Commercial",
                    "currency_code": "USD",
                    "credit_unit_code": "credit",
                    "status": "active",
                    "effective_from_ms": 1710000000000u64,
                    "effective_to_ms": null,
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ],
            "rates": [
                {
                    "pricing_rate_id": 9201u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "pricing_plan_id": 9101u64,
                    "metric_code": "tokens.input",
                    "capability_code": "embeddings",
                    "model_code": "gpt-4.1",
                    "provider_code": "provider-openrouter-main",
                    "charge_unit": "1k_tokens",
                    "pricing_method": "per_unit",
                    "quantity_step": 1.0,
                    "unit_price": 0.002,
                    "display_price_unit": "$ / 1K input tokens",
                    "minimum_billable_quantity": 0.0,
                    "minimum_charge": 0.0,
                    "rounding_increment": 1.0,
                    "rounding_mode": "none",
                    "included_quantity": 0.0,
                    "priority": 10u64,
                    "notes": "invalid provider model capability pricing",
                    "status": "active",
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ]
        }),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject pricing rates whose capability_code is not supported by the provider model"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("9201"), "{error}");
    assert!(error.contains("provider-openrouter-main"), "{error}");
    assert!(error.contains("gpt-4.1"), "{error}");
    assert!(error.contains("embeddings"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_pricing_rate_with_model_code_without_provider_code(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-pricing-rate-model-without-provider");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("pricing").join("default.json"),
        &serde_json::json!({
            "plans": [
                {
                    "pricing_plan_id": 9101u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "plan_code": "global-default-commercial",
                    "plan_version": 1u64,
                    "display_name": "Global Default Commercial",
                    "currency_code": "USD",
                    "credit_unit_code": "credit",
                    "status": "active",
                    "effective_from_ms": 1710000000000u64,
                    "effective_to_ms": null,
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ],
            "rates": [
                {
                    "pricing_rate_id": 9201u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "pricing_plan_id": 9101u64,
                    "metric_code": "tokens.input",
                    "capability_code": "responses",
                    "model_code": "gpt-4.1",
                    "provider_code": null,
                    "charge_unit": "1k_tokens",
                    "pricing_method": "per_unit",
                    "quantity_step": 1.0,
                    "unit_price": 0.002,
                    "display_price_unit": "$ / 1K input tokens",
                    "minimum_billable_quantity": 0.0,
                    "minimum_charge": 0.0,
                    "rounding_increment": 1.0,
                    "rounding_mode": "none",
                    "included_quantity": 0.0,
                    "priority": 10u64,
                    "notes": "invalid model-only pricing",
                    "status": "active",
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ]
        }),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject pricing rates whose model_code is set without provider_code"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("9201"), "{error}");
    assert!(error.contains("model_code"), "{error}");
    assert!(error.contains("provider_code"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_active_pricing_rate_with_inactive_parent_plan(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-pricing-rate-inactive-parent-plan");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("pricing").join("default.json"),
        &serde_json::json!({
            "plans": [
                {
                    "pricing_plan_id": 9101u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "plan_code": "global-default-commercial",
                    "plan_version": 1u64,
                    "display_name": "Global Default Commercial",
                    "currency_code": "USD",
                    "credit_unit_code": "credit",
                    "status": "archived",
                    "effective_from_ms": 1710000000000u64,
                    "effective_to_ms": null,
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ],
            "rates": [
                {
                    "pricing_rate_id": 9201u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "pricing_plan_id": 9101u64,
                    "metric_code": "tokens.input",
                    "capability_code": "responses",
                    "model_code": null,
                    "provider_code": null,
                    "charge_unit": "1k_tokens",
                    "pricing_method": "per_unit",
                    "quantity_step": 1.0,
                    "unit_price": 0.002,
                    "display_price_unit": "$ / 1K input tokens",
                    "minimum_billable_quantity": 0.0,
                    "minimum_charge": 0.0,
                    "rounding_increment": 1.0,
                    "rounding_mode": "none",
                    "included_quantity": 0.0,
                    "priority": 10u64,
                    "notes": "invalid active pricing rate on archived plan",
                    "status": "active",
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ]
        }),
    );
    write_json(
        &bootstrap_root.join("request-metering").join("default.json"),
        &serde_json::json!({
            "facts": [
                {
                    "request_id": 6001u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "user_id": 9001u64,
                    "account_id": 7001u64,
                    "api_key_id": 10001u64,
                    "api_key_hash": "a19d2bf76318aa7f619d684271469bb383faf1cb5bd4c680088465cde9d0003b",
                    "auth_type": "api_key",
                    "jwt_subject": null,
                    "platform": "portal",
                    "owner": "user_local_demo",
                    "request_trace_id": "trace-local-demo-growth-2026",
                    "gateway_request_ref": null,
                    "upstream_request_ref": null,
                    "protocol_family": "openai",
                    "capability_code": "responses",
                    "channel_code": "openrouter",
                    "model_code": "gpt-4.1",
                    "provider_code": "provider-openrouter-main",
                    "request_status": "succeeded",
                    "usage_capture_status": "captured",
                    "cost_pricing_plan_id": null,
                    "retail_pricing_plan_id": null,
                    "estimated_credit_hold": 2400.0,
                    "actual_credit_charge": 2300.0,
                    "actual_provider_cost": 0.27,
                    "started_at_ms": 1710005500000u64,
                    "finished_at_ms": 1710005500900u64,
                    "created_at_ms": 1710005500000u64,
                    "updated_at_ms": 1710005500900u64
                }
            ],
            "metrics": [
                {
                    "request_metric_id": 7001001u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "request_id": 6001u64,
                    "metric_code": "token.input",
                    "quantity": 1800.0,
                    "provider_field": "prompt_tokens",
                    "source_kind": "provider",
                    "capture_stage": "final",
                    "is_billable": true,
                    "captured_at_ms": 1710005500850u64
                },
                {
                    "request_metric_id": 7001002u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "request_id": 6001u64,
                    "metric_code": "token.output",
                    "quantity": 600.0,
                    "provider_field": "completion_tokens",
                    "source_kind": "provider",
                    "capture_stage": "final",
                    "is_billable": true,
                    "captured_at_ms": 1710005500850u64
                }
            ]
        }),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject active pricing rates whose parent pricing plan is not active"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("9201"), "{error}");
    assert!(error.contains("active pricing rate"), "{error}");
    assert!(error.contains("9101"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_pricing_plan_with_effective_to_before_effective_from(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-pricing-plan-effective-window");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("pricing").join("default.json"),
        &serde_json::json!({
            "plans": [
                {
                    "pricing_plan_id": 9101u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "plan_code": "global-default-commercial",
                    "plan_version": 1u64,
                    "display_name": "Global Default Commercial",
                    "currency_code": "USD",
                    "credit_unit_code": "credit",
                    "status": "active",
                    "effective_from_ms": 1710000000000u64,
                    "effective_to_ms": 1709999999000u64,
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ],
            "rates": [
                {
                    "pricing_rate_id": 9201u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "pricing_plan_id": 9101u64,
                    "metric_code": "tokens.input",
                    "capability_code": "responses",
                    "model_code": null,
                    "provider_code": null,
                    "charge_unit": "1k_tokens",
                    "pricing_method": "per_unit",
                    "quantity_step": 1.0,
                    "unit_price": 0.002,
                    "display_price_unit": "$ / 1K input tokens",
                    "minimum_billable_quantity": 0.0,
                    "minimum_charge": 0.0,
                    "rounding_increment": 1.0,
                    "rounding_mode": "none",
                    "included_quantity": 0.0,
                    "priority": 10u64,
                    "notes": "invalid pricing plan window",
                    "status": "active",
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ]
        }),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject pricing plans whose effective_to_ms is earlier than effective_from_ms"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("9101"), "{error}");
    assert!(error.contains("effective_to_ms"), "{error}");
    assert!(error.contains("effective_from_ms"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_active_pricing_rate_without_active_model_price_coverage(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-pricing-rate-model-price-coverage");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("pricing").join("default.json"),
        &serde_json::json!({
            "plans": [
                {
                    "pricing_plan_id": 9101u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "plan_code": "global-default-commercial",
                    "plan_version": 1u64,
                    "display_name": "Global Default Commercial",
                    "currency_code": "USD",
                    "credit_unit_code": "credit",
                    "status": "active",
                    "effective_from_ms": 1710000000000u64,
                    "effective_to_ms": null,
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ],
            "rates": [
                {
                    "pricing_rate_id": 9201u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "pricing_plan_id": 9101u64,
                    "metric_code": "tokens.input",
                    "capability_code": "responses",
                    "model_code": "deepseek-chat",
                    "provider_code": "provider-openrouter-main",
                    "charge_unit": "1k_tokens",
                    "pricing_method": "per_unit",
                    "quantity_step": 1.0,
                    "unit_price": 0.002,
                    "display_price_unit": "$ / 1K input tokens",
                    "minimum_billable_quantity": 0.0,
                    "minimum_charge": 0.0,
                    "rounding_increment": 1.0,
                    "rounding_mode": "none",
                    "included_quantity": 0.0,
                    "priority": 10u64,
                    "notes": "invalid commercial rate without model price coverage",
                    "status": "active",
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ]
        }),
    );
    write_json(
        &bootstrap_root.join("model-prices").join("default.json"),
        &serde_json::json!([
            {
                "channel_id": "openrouter",
                "model_id": "gpt-4.1",
                "proxy_provider_id": "provider-openrouter-main",
                "currency_code": "USD",
                "price_unit": "per_1m_tokens",
                "input_price": 6.0,
                "output_price": 18.0,
                "cache_read_price": 0.0,
                "cache_write_price": 0.0,
                "request_price": 0.0,
                "price_source_kind": "proxy",
                "is_active": true
            },
            {
                "channel_id": "siliconflow",
                "model_id": "qwen-plus-latest",
                "proxy_provider_id": "provider-siliconflow-main",
                "currency_code": "USD",
                "price_unit": "per_1m_tokens",
                "input_price": 0.4,
                "output_price": 1.2,
                "cache_read_price": 0.0,
                "cache_write_price": 0.0,
                "request_price": 0.0,
                "price_source_kind": "proxy",
                "is_active": true
            },
            {
                "channel_id": "ollama",
                "model_id": "llama3.2:latest",
                "proxy_provider_id": "provider-ollama-local",
                "currency_code": "USD",
                "price_unit": "per_1m_tokens",
                "input_price": 0.0,
                "output_price": 0.0,
                "cache_read_price": 0.0,
                "cache_write_price": 0.0,
                "request_price": 0.0,
                "price_source_kind": "local",
                "is_active": true
            }
        ]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject active pricing rates whose provider/model pair has no active model price coverage"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("9201"), "{error}");
    assert!(error.contains("deepseek-chat"), "{error}");
    assert!(error.contains("provider-openrouter-main"), "{error}");
    assert!(error.contains("active model price coverage"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_active_provider_rate_without_any_active_model_price_coverage(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-provider-rate-model-price-coverage");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("pricing").join("default.json"),
        &serde_json::json!({
            "plans": [
                {
                    "pricing_plan_id": 9101u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "plan_code": "global-default-commercial",
                    "plan_version": 1u64,
                    "display_name": "Global Default Commercial",
                    "currency_code": "USD",
                    "credit_unit_code": "credit",
                    "status": "active",
                    "effective_from_ms": 1710000000000u64,
                    "effective_to_ms": null,
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ],
            "rates": [
                {
                    "pricing_rate_id": 9201u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "pricing_plan_id": 9101u64,
                    "metric_code": "tokens.input",
                    "capability_code": "responses",
                    "model_code": null,
                    "provider_code": "provider-siliconflow-main",
                    "charge_unit": "1k_tokens",
                    "pricing_method": "per_unit",
                    "quantity_step": 1.0,
                    "unit_price": 0.002,
                    "display_price_unit": "$ / 1K input tokens",
                    "minimum_billable_quantity": 0.0,
                    "minimum_charge": 0.0,
                    "rounding_increment": 1.0,
                    "rounding_mode": "none",
                    "included_quantity": 0.0,
                    "priority": 10u64,
                    "notes": "invalid provider fallback rate without model price coverage",
                    "status": "active",
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ]
        }),
    );
    write_json(
        &bootstrap_root.join("model-prices").join("default.json"),
        &serde_json::json!([
            {
                "channel_id": "openrouter",
                "model_id": "gpt-4.1",
                "proxy_provider_id": "provider-openrouter-main",
                "currency_code": "USD",
                "price_unit": "per_1m_tokens",
                "input_price": 6.0,
                "output_price": 18.0,
                "cache_read_price": 0.0,
                "cache_write_price": 0.0,
                "request_price": 0.0,
                "price_source_kind": "proxy",
                "is_active": true
            },
            {
                "channel_id": "ollama",
                "model_id": "llama3.2:latest",
                "proxy_provider_id": "provider-ollama-local",
                "currency_code": "USD",
                "price_unit": "per_1m_tokens",
                "input_price": 0.0,
                "output_price": 0.0,
                "cache_read_price": 0.0,
                "cache_write_price": 0.0,
                "request_price": 0.0,
                "price_source_kind": "local",
                "is_active": true
            }
        ]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject active provider fallback rates when the provider has no active model price coverage"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("9201"), "{error}");
    assert!(error.contains("provider-siliconflow-main"), "{error}");
    assert!(error.contains("active model price coverage"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_active_provider_rate_without_capability_matched_active_model_price_coverage(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-provider-rate-capability-price-coverage");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("pricing").join("default.json"),
        &serde_json::json!({
            "plans": [
                {
                    "pricing_plan_id": 9101u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "plan_code": "global-default-commercial",
                    "plan_version": 1u64,
                    "display_name": "Global Default Commercial",
                    "currency_code": "USD",
                    "credit_unit_code": "credit",
                    "status": "active",
                    "effective_from_ms": 1710000000000u64,
                    "effective_to_ms": null,
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ],
            "rates": [
                {
                    "pricing_rate_id": 9201u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "pricing_plan_id": 9101u64,
                    "metric_code": "tokens.input",
                    "capability_code": "embeddings",
                    "model_code": null,
                    "provider_code": "provider-openrouter-main",
                    "charge_unit": "1k_tokens",
                    "pricing_method": "per_unit",
                    "quantity_step": 1.0,
                    "unit_price": 0.0002,
                    "display_price_unit": "$ / 1K embedding input tokens",
                    "minimum_billable_quantity": 0.0,
                    "minimum_charge": 0.0,
                    "rounding_increment": 1.0,
                    "rounding_mode": "none",
                    "included_quantity": 0.0,
                    "priority": 10u64,
                    "notes": "invalid provider fallback rate without capability-matched priced model",
                    "status": "active",
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ]
        }),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject active provider pricing rates without capability-matched active model price coverage"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("9201"), "{error}");
    assert!(error.contains("provider-openrouter-main"), "{error}");
    assert!(error.contains("embeddings"), "{error}");
    assert!(error.contains("price"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_active_provider_rate_without_executable_provider_account(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-provider-rate-without-executable-account");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root
            .join("provider-accounts")
            .join("default.json"),
        &provider_accounts_without_siliconflow_fixture(),
    );
    write_json(
        &bootstrap_root.join("routing").join("default.json"),
        &routing_fixture_without_siliconflow_candidates(),
    );
    write_json(
        &bootstrap_root.join("observability").join("default.json"),
        &serde_json::json!({
            "compiled_routing_snapshots": [],
            "routing_decision_logs": [],
            "provider_health_snapshots": []
        }),
    );
    write_json(
        &bootstrap_root.join("billing").join("default.json"),
        &billing_fixture_with_local_metering_support(vec![]),
    );
    write_json(
        &bootstrap_root.join("model-prices").join("default.json"),
        &model_prices_without_siliconflow_fixture(),
    );
    write_json(
        &bootstrap_root.join("pricing").join("default.json"),
        &serde_json::json!({
            "plans": [
                {
                    "pricing_plan_id": 9101u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "plan_code": "global-default-commercial",
                    "plan_version": 1u64,
                    "display_name": "Global Default Commercial",
                    "currency_code": "USD",
                    "credit_unit_code": "credit",
                    "status": "active",
                    "effective_from_ms": 1710000000000u64,
                    "effective_to_ms": null,
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ],
            "rates": [
                {
                    "pricing_rate_id": 9201u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "pricing_plan_id": 9101u64,
                    "metric_code": "tokens.input",
                    "capability_code": "responses",
                    "model_code": null,
                    "provider_code": "provider-siliconflow-main",
                    "charge_unit": "1k_tokens",
                    "pricing_method": "per_unit",
                    "quantity_step": 1.0,
                    "unit_price": 0.002,
                    "display_price_unit": "$ / 1K input tokens",
                    "minimum_billable_quantity": 0.0,
                    "minimum_charge": 0.0,
                    "rounding_increment": 1.0,
                    "rounding_mode": "none",
                    "included_quantity": 0.0,
                    "priority": 10u64,
                    "notes": "invalid provider rate without executable account",
                    "status": "active",
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ]
        }),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject active provider pricing rates whose provider has no executable account"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("9201"), "{error}");
    assert!(error.contains("provider-siliconflow-main"), "{error}");
    assert!(error.contains("provider account"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_request_settlement_without_request_meter_fact(
) {
    let bootstrap_root = temp_bootstrap_root("profile-pack-invalid-request-settlement-missing-fact");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_single_account_hold_fixture(
        &bootstrap_root,
        6002u64,
        "partially_released",
        2400.0,
        2300.0,
        100.0,
        1710005500000u64,
        1710005500900u64,
    );
    write_json(
        &bootstrap_root.join("request-metering").join("default.json"),
        &serde_json::json!({
            "facts": [
                {
                    "request_id": 6002u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "user_id": 9001u64,
                    "account_id": 7001u64,
                    "api_key_id": 10001u64,
                    "api_key_hash": "a19d2bf76318aa7f619d684271469bb383faf1cb5bd4c680088465cde9d0003b",
                    "auth_type": "api_key",
                    "jwt_subject": null,
                    "platform": "portal",
                    "owner": "user_local_demo",
                    "request_trace_id": "trace-local-demo-growth-2026-shadow",
                    "gateway_request_ref": null,
                    "upstream_request_ref": null,
                    "protocol_family": "openai",
                    "capability_code": "responses",
                    "channel_code": "openrouter",
                    "model_code": "gpt-4.1",
                    "provider_code": "provider-openrouter-main",
                    "request_status": "succeeded",
                    "usage_capture_status": "captured",
                    "cost_pricing_plan_id": 9101u64,
                    "retail_pricing_plan_id": 9101u64,
                    "estimated_credit_hold": 2400.0,
                    "actual_credit_charge": 2300.0,
                    "actual_provider_cost": 0.27,
                    "started_at_ms": 1710005500000u64,
                    "finished_at_ms": 1710005500900u64,
                    "created_at_ms": 1710005500000u64,
                    "updated_at_ms": 1710005500900u64
                }
            ],
            "metrics": []
        }),
    );
    write_json(
        &bootstrap_root.join("request-settlements").join("default.json"),
        &serde_json::json!([
            {
                "request_settlement_id": 8301u64,
                "tenant_id": 1001u64,
                "organization_id": 2001u64,
                "request_id": 6001u64,
                "account_id": 7001u64,
                "user_id": 9001u64,
                "hold_id": null,
                "status": "partially_released",
                "estimated_credit_hold": 2400.0,
                "released_credit_amount": 100.0,
                "captured_credit_amount": 2300.0,
                "provider_cost_amount": 0.27,
                "retail_charge_amount": 0.69,
                "shortfall_amount": 0.0,
                "refunded_amount": 0.0,
                "settled_at_ms": 1710005500900u64,
                "created_at_ms": 1710005500000u64,
                "updated_at_ms": 1710005500900u64
            }
        ]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject request settlements that do not reference a request meter fact"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("8301"), "{error}");
    assert!(error.contains("6001"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_request_meter_metric_with_mismatched_parent_ownership(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-request-meter-metric-ownership");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("request-metering").join("default.json"),
        &serde_json::json!({
            "facts": [
                {
                    "request_id": 6001u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "user_id": 9001u64,
                    "account_id": 7001u64,
                    "api_key_id": 10001u64,
                    "api_key_hash": "a19d2bf76318aa7f619d684271469bb383faf1cb5bd4c680088465cde9d0003b",
                    "auth_type": "api_key",
                    "jwt_subject": null,
                    "platform": "portal",
                    "owner": "user_local_demo",
                    "request_trace_id": "trace-local-demo-growth-2026",
                    "gateway_request_ref": null,
                    "upstream_request_ref": null,
                    "protocol_family": "openai",
                    "capability_code": "responses",
                    "channel_code": "openrouter",
                    "model_code": "gpt-4.1",
                    "provider_code": "provider-openrouter-main",
                    "request_status": "succeeded",
                    "usage_capture_status": "captured",
                    "cost_pricing_plan_id": 9101u64,
                    "retail_pricing_plan_id": 9101u64,
                    "estimated_credit_hold": 2400.0,
                    "actual_credit_charge": 2300.0,
                    "actual_provider_cost": 0.27,
                    "started_at_ms": 1710005500000u64,
                    "finished_at_ms": 1710005500900u64,
                    "created_at_ms": 1710005500000u64,
                    "updated_at_ms": 1710005500900u64
                }
            ],
            "metrics": [
                {
                    "request_metric_id": 7001001u64,
                    "tenant_id": 1999u64,
                    "organization_id": 2999u64,
                    "request_id": 6001u64,
                    "metric_code": "token.input",
                    "quantity": 1800.0,
                    "provider_field": "prompt_tokens",
                    "source_kind": "provider",
                    "capture_stage": "final",
                    "is_billable": true,
                    "captured_at_ms": 1710005500850u64
                },
                {
                    "request_metric_id": 7001002u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "request_id": 6001u64,
                    "metric_code": "token.output",
                    "quantity": 600.0,
                    "provider_field": "completion_tokens",
                    "source_kind": "provider",
                    "capture_stage": "final",
                    "is_billable": true,
                    "captured_at_ms": 1710005500850u64
                }
            ]
        }),
    );
    write_json(
        &bootstrap_root.join("request-settlements").join("default.json"),
        &serde_json::json!([]),
    );
    write_json(
        &bootstrap_root.join("request-settlements").join("default.json"),
        &serde_json::json!([]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject request meter metrics whose tenant or organization drifts from the parent request meter fact"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("7001001"), "{error}");
    assert!(error.contains("tenant"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_request_meter_metric_capture_stage_mismatched_parent_fact(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-request-meter-metric-stage");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("request-metering").join("default.json"),
        &serde_json::json!({
            "facts": [
                {
                    "request_id": 6001u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "user_id": 9001u64,
                    "account_id": 7001u64,
                    "api_key_id": 10001u64,
                    "api_key_hash": "a19d2bf76318aa7f619d684271469bb383faf1cb5bd4c680088465cde9d0003b",
                    "auth_type": "api_key",
                    "jwt_subject": null,
                    "platform": "portal",
                    "owner": "user_local_demo",
                    "request_trace_id": "trace-local-demo-growth-2026",
                    "gateway_request_ref": null,
                    "upstream_request_ref": null,
                    "protocol_family": "openai",
                    "capability_code": "responses",
                    "channel_code": "openrouter",
                    "model_code": "gpt-4.1",
                    "provider_code": "provider-openrouter-main",
                    "request_status": "succeeded",
                    "usage_capture_status": "captured",
                    "cost_pricing_plan_id": 9101u64,
                    "retail_pricing_plan_id": 9101u64,
                    "estimated_credit_hold": 2400.0,
                    "actual_credit_charge": 2300.0,
                    "actual_provider_cost": 0.27,
                    "started_at_ms": 1710005500000u64,
                    "finished_at_ms": 1710005500900u64,
                    "created_at_ms": 1710005500000u64,
                    "updated_at_ms": 1710005500900u64
                }
            ],
            "metrics": [
                {
                    "request_metric_id": 7001001u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "request_id": 6001u64,
                    "metric_code": "token.input",
                    "quantity": 1800.0,
                    "provider_field": "prompt_tokens",
                    "source_kind": "provider",
                    "capture_stage": "estimate",
                    "is_billable": true,
                    "captured_at_ms": 1710005500850u64
                },
                {
                    "request_metric_id": 7001002u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "request_id": 6001u64,
                    "metric_code": "token.output",
                    "quantity": 600.0,
                    "provider_field": "completion_tokens",
                    "source_kind": "provider",
                    "capture_stage": "final",
                    "is_billable": true,
                    "captured_at_ms": 1710005500850u64
                }
            ]
        }),
    );
    write_json(
        &bootstrap_root.join("request-settlements").join("default.json"),
        &serde_json::json!([]),
    );
    write_json(
        &bootstrap_root.join("request-settlements").join("default.json"),
        &serde_json::json!([]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject request meter metrics whose capture_stage drifts from the parent request meter fact usage_capture_status"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("7001001"), "{error}");
    assert!(error.contains("capture_stage"), "{error}");
    assert!(error.contains("captured"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_request_meter_metric_captured_after_parent_finish(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-request-meter-metric-finished-window");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("request-metering").join("default.json"),
        &serde_json::json!({
            "facts": [
                {
                    "request_id": 6001u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "user_id": 9001u64,
                    "account_id": 7001u64,
                    "api_key_id": 10001u64,
                    "api_key_hash": "a19d2bf76318aa7f619d684271469bb383faf1cb5bd4c680088465cde9d0003b",
                    "auth_type": "api_key",
                    "jwt_subject": null,
                    "platform": "portal",
                    "owner": "user_local_demo",
                    "request_trace_id": "trace-local-demo-growth-2026",
                    "gateway_request_ref": null,
                    "upstream_request_ref": null,
                    "protocol_family": "openai",
                    "capability_code": "responses",
                    "channel_code": "openrouter",
                    "model_code": "gpt-4.1",
                    "provider_code": "provider-openrouter-main",
                    "request_status": "succeeded",
                    "usage_capture_status": "captured",
                    "cost_pricing_plan_id": 9101u64,
                    "retail_pricing_plan_id": 9101u64,
                    "estimated_credit_hold": 2400.0,
                    "actual_credit_charge": 2300.0,
                    "actual_provider_cost": 0.27,
                    "started_at_ms": 1710005500000u64,
                    "finished_at_ms": 1710005500900u64,
                    "created_at_ms": 1710005500000u64,
                    "updated_at_ms": 1710005500900u64
                }
            ],
            "metrics": [
                {
                    "request_metric_id": 7001001u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "request_id": 6001u64,
                    "metric_code": "token.input",
                    "quantity": 1800.0,
                    "provider_field": "prompt_tokens",
                    "source_kind": "provider",
                    "capture_stage": "final",
                    "is_billable": true,
                    "captured_at_ms": 1710005500950u64
                }
            ]
        }),
    );
    write_json(
        &bootstrap_root.join("request-settlements").join("default.json"),
        &serde_json::json!([]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject request meter metrics captured after the parent request finished"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("7001001"), "{error}");
    assert!(error.contains("finished_at_ms"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_request_meter_metric_captured_after_parent_update(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-request-meter-metric-updated-window");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("request-metering").join("default.json"),
        &serde_json::json!({
            "facts": [
                {
                    "request_id": 6001u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "user_id": 9001u64,
                    "account_id": 7001u64,
                    "api_key_id": 10001u64,
                    "api_key_hash": "a19d2bf76318aa7f619d684271469bb383faf1cb5bd4c680088465cde9d0003b",
                    "auth_type": "api_key",
                    "jwt_subject": null,
                    "platform": "portal",
                    "owner": "user_local_demo",
                    "request_trace_id": "trace-local-demo-growth-2026",
                    "gateway_request_ref": null,
                    "upstream_request_ref": null,
                    "protocol_family": "openai",
                    "capability_code": "responses",
                    "channel_code": "openrouter",
                    "model_code": "gpt-4.1",
                    "provider_code": "provider-openrouter-main",
                    "request_status": "succeeded",
                    "usage_capture_status": "estimated",
                    "cost_pricing_plan_id": 9101u64,
                    "retail_pricing_plan_id": 9101u64,
                    "estimated_credit_hold": 2400.0,
                    "actual_credit_charge": null,
                    "actual_provider_cost": null,
                    "started_at_ms": 1710005500000u64,
                    "finished_at_ms": null,
                    "created_at_ms": 1710005500000u64,
                    "updated_at_ms": 1710005500600u64
                }
            ],
            "metrics": [
                {
                    "request_metric_id": 7001001u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "request_id": 6001u64,
                    "metric_code": "token.input",
                    "quantity": 1800.0,
                    "provider_field": "prompt_tokens",
                    "source_kind": "provider",
                    "capture_stage": "estimate",
                    "is_billable": true,
                    "captured_at_ms": 1710005500700u64
                }
            ]
        }),
    );
    write_json(
        &bootstrap_root.join("request-settlements").join("default.json"),
        &serde_json::json!([]),
    );
    write_single_account_hold_fixture(
        &bootstrap_root,
        6001u64,
        "held",
        2400.0,
        0.0,
        0.0,
        1710005500000u64,
        1710005500600u64,
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject request meter metrics captured after the parent request was last updated"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("7001001"), "{error}");
    assert!(error.contains("updated_at_ms"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_captured_request_meter_fact_without_actual_accounting(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-request-meter-fact-missing-actual-accounting");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("request-metering").join("default.json"),
        &serde_json::json!({
            "facts": [
                {
                    "request_id": 6001u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "user_id": 9001u64,
                    "account_id": 7001u64,
                    "api_key_id": 10001u64,
                    "api_key_hash": "a19d2bf76318aa7f619d684271469bb383faf1cb5bd4c680088465cde9d0003b",
                    "auth_type": "api_key",
                    "jwt_subject": null,
                    "platform": "portal",
                    "owner": "user_local_demo",
                    "request_trace_id": "trace-local-demo-growth-2026",
                    "gateway_request_ref": null,
                    "upstream_request_ref": null,
                    "protocol_family": "openai",
                    "capability_code": "responses",
                    "channel_code": "openrouter",
                    "model_code": "gpt-4.1",
                    "provider_code": "provider-openrouter-main",
                    "request_status": "succeeded",
                    "usage_capture_status": "captured",
                    "cost_pricing_plan_id": 9101u64,
                    "retail_pricing_plan_id": 9101u64,
                    "estimated_credit_hold": 2400.0,
                    "actual_credit_charge": null,
                    "actual_provider_cost": null,
                    "started_at_ms": 1710005500000u64,
                    "finished_at_ms": 1710005500900u64,
                    "created_at_ms": 1710005500000u64,
                    "updated_at_ms": 1710005500900u64
                }
            ],
            "metrics": []
        }),
    );
    write_json(
        &bootstrap_root.join("request-settlements").join("default.json"),
        &serde_json::json!([]),
    );
    write_json(
        &bootstrap_root.join("request-settlements").join("default.json"),
        &serde_json::json!([]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject captured request meter facts that do not contain actual accounting values"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("6001"), "{error}");
    assert!(error.contains("captured"), "{error}");
    assert!(error.contains("actual_credit_charge"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_estimated_request_meter_fact_with_actual_accounting(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-request-meter-fact-estimated-actual-accounting");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("request-metering").join("default.json"),
        &serde_json::json!({
            "facts": [
                {
                    "request_id": 6001u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "user_id": 9001u64,
                    "account_id": 7001u64,
                    "api_key_id": 10001u64,
                    "api_key_hash": "a19d2bf76318aa7f619d684271469bb383faf1cb5bd4c680088465cde9d0003b",
                    "auth_type": "api_key",
                    "jwt_subject": null,
                    "platform": "portal",
                    "owner": "user_local_demo",
                    "request_trace_id": "trace-local-demo-growth-2026",
                    "gateway_request_ref": null,
                    "upstream_request_ref": null,
                    "protocol_family": "openai",
                    "capability_code": "responses",
                    "channel_code": "openrouter",
                    "model_code": "gpt-4.1",
                    "provider_code": "provider-openrouter-main",
                    "request_status": "succeeded",
                    "usage_capture_status": "estimated",
                    "cost_pricing_plan_id": 9101u64,
                    "retail_pricing_plan_id": 9101u64,
                    "estimated_credit_hold": 2400.0,
                    "actual_credit_charge": 2300.0,
                    "actual_provider_cost": 0.27,
                    "started_at_ms": 1710005500000u64,
                    "finished_at_ms": 1710005500900u64,
                    "created_at_ms": 1710005500000u64,
                    "updated_at_ms": 1710005500900u64
                }
            ],
            "metrics": []
        }),
    );
    write_json(
        &bootstrap_root.join("request-settlements").join("default.json"),
        &serde_json::json!([]),
    );
    write_single_account_hold_fixture(
        &bootstrap_root,
        6001u64,
        "held",
        2400.0,
        0.0,
        0.0,
        1710005500000u64,
        1710005500900u64,
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject estimated request meter facts that already contain actual accounting values"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("6001"), "{error}");
    assert!(error.contains("estimated"), "{error}");
    assert!(error.contains("actual_credit_charge"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_request_settlement_captured_amount_mismatched_request_meter_fact(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-request-settlement-captured-drift");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("request-settlements").join("default.json"),
        &serde_json::json!([
            {
                "request_settlement_id": 8301u64,
                "tenant_id": 1001u64,
                "organization_id": 2001u64,
                "request_id": 6001u64,
                "account_id": 7001u64,
                "user_id": 9001u64,
                "hold_id": 8101u64,
                "status": "refunded",
                "estimated_credit_hold": 2400.0,
                "released_credit_amount": 100.0,
                "captured_credit_amount": 2200.0,
                "provider_cost_amount": 0.27,
                "retail_charge_amount": 0.69,
                "shortfall_amount": 0.0,
                "refunded_amount": 0.0,
                "settled_at_ms": 1710005500900u64,
                "created_at_ms": 1710005500000u64,
                "updated_at_ms": 1710005500900u64
            }
        ]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject request settlements whose captured amount drifts from the request meter fact"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("8301"), "{error}");
    assert!(error.contains("captured_credit_amount"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_request_settlement_released_amount_mismatched_hold(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-request-settlement-released-hold-drift");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("request-settlements").join("default.json"),
        &serde_json::json!([
            {
                "request_settlement_id": 8301u64,
                "tenant_id": 1001u64,
                "organization_id": 2001u64,
                "request_id": 6001u64,
                "account_id": 7001u64,
                "user_id": 9001u64,
                "hold_id": 8101u64,
                "status": "partially_released",
                "estimated_credit_hold": 2400.0,
                "released_credit_amount": 90.0,
                "captured_credit_amount": 2300.0,
                "provider_cost_amount": 0.27,
                "retail_charge_amount": 0.69,
                "shortfall_amount": 0.0,
                "refunded_amount": 0.0,
                "settled_at_ms": 1710005500900u64,
                "created_at_ms": 1710005500000u64,
                "updated_at_ms": 1710005500900u64
            }
        ]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject request settlements whose released amount drifts from the linked hold"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("8301"), "{error}");
    assert!(error.contains("released_credit_amount"), "{error}");
    assert!(error.contains("8101"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_request_settlement_estimated_hold_mismatched_hold(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-request-settlement-estimated-hold-drift");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("account-holds").join("default.json"),
        &serde_json::json!({
            "holds": [
                {
                    "hold_id": 8101u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "account_id": 7001u64,
                    "user_id": 9001u64,
                    "request_id": 6001u64,
                    "status": "failed",
                    "estimated_quantity": 2500.0,
                    "captured_quantity": 2300.0,
                    "released_quantity": 100.0,
                    "expires_at_ms": 1710006100000u64,
                    "created_at_ms": 1710005500000u64,
                    "updated_at_ms": 1710005500900u64
                }
            ],
            "allocations": [
                {
                    "hold_allocation_id": 8401u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "hold_id": 8101u64,
                    "lot_id": 8001u64,
                    "allocated_quantity": 2500.0,
                    "captured_quantity": 2300.0,
                    "released_quantity": 100.0,
                    "created_at_ms": 1710005500000u64,
                    "updated_at_ms": 1710005500900u64
                }
            ]
        }),
    );
    let metering_path = bootstrap_root.join("request-metering").join("default.json");
    let mut metering =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&metering_path).unwrap())
            .unwrap();
    metering["facts"][0]["estimated_credit_hold"] = serde_json::json!(2500.0);
    write_json(&metering_path, &metering);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject request settlements whose estimated hold drifts from the linked hold"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("8301"), "{error}");
    assert!(error.contains("estimated_credit_hold"), "{error}");
    assert!(error.contains("8101"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_account_reconciliation_with_mismatched_account_ownership(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-account-reconciliation-account-ownership");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("account-reconciliation").join("default.json"),
        &serde_json::json!([
            {
                "tenant_id": 1999u64,
                "organization_id": 2999u64,
                "account_id": 7001u64,
                "project_id": "project_local_demo",
                "last_order_updated_at_ms": 1710005000500u64,
                "last_order_created_at_ms": 1710005000000u64,
                "last_order_id": "order-local-demo-growth-2026",
                "updated_at_ms": 1710005300700u64
            }
        ]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject account reconciliation states whose tenant or organization drifts from the linked account"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("7001::project_local_demo"), "{error}");
    assert!(error.contains("account"), "{error}");
    assert!(error.contains("tenant"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_account_reconciliation_with_mismatched_last_order_timestamps(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-account-reconciliation-order-timestamps");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("account-reconciliation").join("default.json"),
        &serde_json::json!([
            {
                "tenant_id": 1001u64,
                "organization_id": 2001u64,
                "account_id": 7001u64,
                "project_id": "project_local_demo",
                "last_order_updated_at_ms": 1710005000600u64,
                "last_order_created_at_ms": 1710005000000u64,
                "last_order_id": "order-local-demo-growth-2026",
                "updated_at_ms": 1710005300700u64
            }
        ]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject account reconciliation states whose last order timestamps drift from the linked order"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("7001::project_local_demo"), "{error}");
    assert!(error.contains("order-local-demo-growth-2026"), "{error}");
    assert!(error.contains("last_order_updated_at_ms"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_request_settlement_with_captured_and_released_exceeding_estimated_hold(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-request-settlement-captured-released-over-estimated");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("request-settlements").join("default.json"),
        &serde_json::json!([
            {
                "request_settlement_id": 8301u64,
                "tenant_id": 1001u64,
                "organization_id": 2001u64,
                "request_id": 6001u64,
                "account_id": 7001u64,
                "user_id": 9001u64,
                "hold_id": null,
                "status": "refunded",
                "estimated_credit_hold": 2400.0,
                "released_credit_amount": 150.0,
                "captured_credit_amount": 2300.0,
                "provider_cost_amount": 0.27,
                "retail_charge_amount": 0.69,
                "shortfall_amount": 0.0,
                "refunded_amount": 0.0,
                "settled_at_ms": 1710005500900u64,
                "created_at_ms": 1710005500000u64,
                "updated_at_ms": 1710005500900u64
            }
        ]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject request settlements whose captured and released totals exceed the estimated hold"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("8301"), "{error}");
    assert!(error.contains("captured"), "{error}");
    assert!(error.contains("estimated"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_request_settlement_with_refunded_amount_exceeding_captured_amount(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-request-settlement-refund-over-capture");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("request-settlements").join("default.json"),
        &serde_json::json!([
            {
                "request_settlement_id": 8301u64,
                "tenant_id": 1001u64,
                "organization_id": 2001u64,
                "request_id": 6001u64,
                "account_id": 7001u64,
                "user_id": 9001u64,
                "hold_id": null,
                "status": "refunded",
                "estimated_credit_hold": 2400.0,
                "released_credit_amount": 100.0,
                "captured_credit_amount": 2300.0,
                "provider_cost_amount": 0.27,
                "retail_charge_amount": 0.69,
                "shortfall_amount": 0.0,
                "refunded_amount": 2300.1,
                "settled_at_ms": 1710005500900u64,
                "created_at_ms": 1710005500000u64,
                "updated_at_ms": 1710005500900u64
            }
        ]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject request settlements whose refunded amount exceeds the captured amount"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("8301"), "{error}");
    assert!(error.contains("refunded"), "{error}");
    assert!(error.contains("captured"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_request_settlement_created_before_request_started(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-request-settlement-created-before-request-started");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("request-settlements").join("default.json"),
        &serde_json::json!([
            {
                "request_settlement_id": 8301u64,
                "tenant_id": 1001u64,
                "organization_id": 2001u64,
                "request_id": 6001u64,
                "account_id": 7001u64,
                "user_id": 9001u64,
                "hold_id": null,
                "status": "partially_released",
                "estimated_credit_hold": 2400.0,
                "released_credit_amount": 100.0,
                "captured_credit_amount": 2300.0,
                "provider_cost_amount": 0.27,
                "retail_charge_amount": 0.69,
                "shortfall_amount": 0.0,
                "refunded_amount": 0.0,
                "settled_at_ms": 1710005500900u64,
                "created_at_ms": 1709999999000u64,
                "updated_at_ms": 1710005500900u64
            }
        ]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject request settlements created before the linked request meter fact started"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("8301"), "{error}");
    assert!(error.contains("created_at_ms"), "{error}");
    assert!(error.contains("started_at_ms"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_request_settlement_settled_before_request_finished(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-request-settlement-settled-before-request-finished");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("request-settlements").join("default.json"),
        &serde_json::json!([
            {
                "request_settlement_id": 8301u64,
                "tenant_id": 1001u64,
                "organization_id": 2001u64,
                "request_id": 6001u64,
                "account_id": 7001u64,
                "user_id": 9001u64,
                "hold_id": null,
                "status": "partially_released",
                "estimated_credit_hold": 2400.0,
                "released_credit_amount": 100.0,
                "captured_credit_amount": 2300.0,
                "provider_cost_amount": 0.27,
                "retail_charge_amount": 0.69,
                "shortfall_amount": 0.0,
                "refunded_amount": 0.0,
                "settled_at_ms": 1710005500800u64,
                "created_at_ms": 1710005500000u64,
                "updated_at_ms": 1710005500900u64
            }
        ]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject request settlements settled before the linked request meter fact finished"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("8301"), "{error}");
    assert!(error.contains("settled_at_ms"), "{error}");
    assert!(error.contains("finished_at_ms"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_request_settlement_created_before_hold_created(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-request-settlement-created-before-hold-created");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("account-holds").join("default.json"),
        &serde_json::json!({
            "holds": [
                {
                    "hold_id": 8101u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "account_id": 7001u64,
                    "user_id": 9001u64,
                    "request_id": 6001u64,
                    "status": "failed",
                    "estimated_quantity": 2400.0,
                    "captured_quantity": 2300.0,
                    "released_quantity": 100.0,
                    "expires_at_ms": 1710006100000u64,
                    "created_at_ms": 1710005500100u64,
                    "updated_at_ms": 1710005500900u64
                }
            ],
            "allocations": [
                {
                    "hold_allocation_id": 8401u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "hold_id": 8101u64,
                    "lot_id": 8001u64,
                    "allocated_quantity": 2400.0,
                    "captured_quantity": 2300.0,
                    "released_quantity": 100.0,
                    "created_at_ms": 1710005500100u64,
                    "updated_at_ms": 1710005500900u64
                }
            ]
        }),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject request settlements created before the linked hold was created"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("8301"), "{error}");
    assert!(error.contains("created_at_ms"), "{error}");
    assert!(error.contains("hold"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_request_settlement_updated_before_hold_updated(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-request-settlement-updated-before-hold-updated");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("account-holds").join("default.json"),
        &serde_json::json!({
            "holds": [
                {
                    "hold_id": 8101u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "account_id": 7001u64,
                    "user_id": 9001u64,
                    "request_id": 6001u64,
                    "status": "partially_released",
                    "estimated_quantity": 2400.0,
                    "captured_quantity": 2300.0,
                    "released_quantity": 100.0,
                    "expires_at_ms": 1710006100000u64,
                    "created_at_ms": 1710005500000u64,
                    "updated_at_ms": 1710005500950u64
                }
            ],
            "allocations": [
                {
                    "hold_allocation_id": 8401u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "hold_id": 8101u64,
                    "lot_id": 8001u64,
                    "allocated_quantity": 2400.0,
                    "captured_quantity": 2300.0,
                    "released_quantity": 100.0,
                    "created_at_ms": 1710005500000u64,
                    "updated_at_ms": 1710005500950u64
                }
            ]
        }),
    );
    write_json(
        &bootstrap_root.join("request-settlements").join("default.json"),
        &serde_json::json!([
            {
                "request_settlement_id": 8301u64,
                "tenant_id": 1001u64,
                "organization_id": 2001u64,
                "request_id": 6001u64,
                "account_id": 7001u64,
                "user_id": 9001u64,
                "hold_id": 8101u64,
                "status": "refunded",
                "estimated_credit_hold": 2400.0,
                "released_credit_amount": 100.0,
                "captured_credit_amount": 2300.0,
                "provider_cost_amount": 0.27,
                "retail_charge_amount": 0.69,
                "shortfall_amount": 0.0,
                "refunded_amount": 0.0,
                "settled_at_ms": 1710005501000u64,
                "created_at_ms": 1710005500000u64,
                "updated_at_ms": 1710005500900u64
            }
        ]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject request settlements updated before the linked hold finished updating"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("8301"), "{error}");
    assert!(error.contains("updated_at_ms"), "{error}");
    assert!(error.contains("hold"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_request_settlement_settled_before_hold_updated(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-request-settlement-settled-before-hold-updated");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("account-holds").join("default.json"),
        &serde_json::json!({
            "holds": [
                {
                    "hold_id": 8101u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "account_id": 7001u64,
                    "user_id": 9001u64,
                    "request_id": 6001u64,
                    "status": "partially_released",
                    "estimated_quantity": 2400.0,
                    "captured_quantity": 2300.0,
                    "released_quantity": 100.0,
                    "expires_at_ms": 1710006100000u64,
                    "created_at_ms": 1710005500000u64,
                    "updated_at_ms": 1710005501000u64
                }
            ],
            "allocations": [
                {
                    "hold_allocation_id": 8401u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "hold_id": 8101u64,
                    "lot_id": 8001u64,
                    "allocated_quantity": 2400.0,
                    "captured_quantity": 2300.0,
                    "released_quantity": 100.0,
                    "created_at_ms": 1710005500000u64,
                    "updated_at_ms": 1710005501000u64
                }
            ]
        }),
    );
    write_json(
        &bootstrap_root.join("request-settlements").join("default.json"),
        &serde_json::json!([
            {
                "request_settlement_id": 8301u64,
                "tenant_id": 1001u64,
                "organization_id": 2001u64,
                "request_id": 6001u64,
                "account_id": 7001u64,
                "user_id": 9001u64,
                "hold_id": 8101u64,
                "status": "partially_released",
                "estimated_credit_hold": 2400.0,
                "released_credit_amount": 100.0,
                "captured_credit_amount": 2300.0,
                "provider_cost_amount": 0.27,
                "retail_charge_amount": 0.69,
                "shortfall_amount": 0.0,
                "refunded_amount": 0.0,
                "settled_at_ms": 1710005500950u64,
                "created_at_ms": 1710005500000u64,
                "updated_at_ms": 1710005501000u64
            }
        ]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject request settlements settled before the linked hold finished updating"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("8301"), "{error}");
    assert!(error.contains("settled_at_ms"), "{error}");
    assert!(error.contains("hold"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_pending_request_settlement_with_settled_timestamp(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-request-settlement-pending-with-settled-timestamp");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("request-metering").join("default.json"),
        &serde_json::json!({
            "facts": [
                {
                    "request_id": 6001u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "user_id": 9001u64,
                    "account_id": 7001u64,
                    "api_key_id": 10001u64,
                    "api_key_hash": "a19d2bf76318aa7f619d684271469bb383faf1cb5bd4c680088465cde9d0003b",
                    "auth_type": "api_key",
                    "jwt_subject": null,
                    "platform": "portal",
                    "owner": "user_local_demo",
                    "request_trace_id": "trace-local-demo-growth-2026",
                    "gateway_request_ref": null,
                    "upstream_request_ref": null,
                    "protocol_family": "openai",
                    "capability_code": "responses",
                    "channel_code": "openrouter",
                    "model_code": "gpt-4.1",
                    "provider_code": "provider-openrouter-main",
                    "request_status": "succeeded",
                    "usage_capture_status": "estimated",
                    "cost_pricing_plan_id": 9101u64,
                    "retail_pricing_plan_id": 9101u64,
                    "estimated_credit_hold": 2400.0,
                    "actual_credit_charge": null,
                    "actual_provider_cost": null,
                    "started_at_ms": 1710005500000u64,
                    "finished_at_ms": null,
                    "created_at_ms": 1710005500000u64,
                    "updated_at_ms": 1710005500900u64
                }
            ],
            "metrics": []
        }),
    );
    write_json(
        &bootstrap_root.join("request-settlements").join("default.json"),
        &serde_json::json!([
            {
                "request_settlement_id": 8301u64,
                "tenant_id": 1001u64,
                "organization_id": 2001u64,
                "request_id": 6001u64,
                "account_id": 7001u64,
                "user_id": 9001u64,
                "hold_id": null,
                "status": "pending",
                "estimated_credit_hold": 2400.0,
                "released_credit_amount": 0.0,
                "captured_credit_amount": 0.0,
                "provider_cost_amount": 0.0,
                "retail_charge_amount": 0.0,
                "shortfall_amount": 0.0,
                "refunded_amount": 0.0,
                "settled_at_ms": 1710005500900u64,
                "created_at_ms": 1710005500000u64,
                "updated_at_ms": 1710005500900u64
            }
        ]),
    );
    write_single_account_hold_fixture(
        &bootstrap_root,
        6001u64,
        "held",
        2400.0,
        0.0,
        0.0,
        1710005500000u64,
        1710005500900u64,
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject pending request settlements that already carry settled_at_ms"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("8301"), "{error}");
    assert!(error.contains("pending"), "{error}");
    assert!(error.contains("settled_at_ms"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_pending_request_settlement_with_realized_accounting(
) {
    let bootstrap_root = temp_bootstrap_root(
        "profile-pack-invalid-request-settlement-pending-with-realized-accounting",
    );
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("request-metering").join("default.json"),
        &serde_json::json!({
            "facts": [
                {
                    "request_id": 6001u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "user_id": 9001u64,
                    "account_id": 7001u64,
                    "api_key_id": 10001u64,
                    "api_key_hash": "a19d2bf76318aa7f619d684271469bb383faf1cb5bd4c680088465cde9d0003b",
                    "auth_type": "api_key",
                    "jwt_subject": null,
                    "platform": "portal",
                    "owner": "user_local_demo",
                    "request_trace_id": "trace-local-demo-growth-2026",
                    "gateway_request_ref": null,
                    "upstream_request_ref": null,
                    "protocol_family": "openai",
                    "capability_code": "responses",
                    "channel_code": "openrouter",
                    "model_code": "gpt-4.1",
                    "provider_code": "provider-openrouter-main",
                    "request_status": "succeeded",
                    "usage_capture_status": "estimated",
                    "cost_pricing_plan_id": 9101u64,
                    "retail_pricing_plan_id": 9101u64,
                    "estimated_credit_hold": 2400.0,
                    "actual_credit_charge": null,
                    "actual_provider_cost": null,
                    "started_at_ms": 1710005500000u64,
                    "finished_at_ms": null,
                    "created_at_ms": 1710005500000u64,
                    "updated_at_ms": 1710005500900u64
                }
            ],
            "metrics": []
        }),
    );
    write_json(
        &bootstrap_root.join("request-settlements").join("default.json"),
        &serde_json::json!([
            {
                "request_settlement_id": 8301u64,
                "tenant_id": 1001u64,
                "organization_id": 2001u64,
                "request_id": 6001u64,
                "account_id": 7001u64,
                "user_id": 9001u64,
                "hold_id": null,
                "status": "pending",
                "estimated_credit_hold": 2400.0,
                "released_credit_amount": 0.0,
                "captured_credit_amount": 0.0,
                "provider_cost_amount": 0.0,
                "retail_charge_amount": 0.0,
                "shortfall_amount": 1.0,
                "refunded_amount": 0.0,
                "settled_at_ms": 0,
                "created_at_ms": 1710005500000u64,
                "updated_at_ms": 1710005500900u64
            }
        ]),
    );
    write_single_account_hold_fixture(
        &bootstrap_root,
        6001u64,
        "held",
        2400.0,
        0.0,
        0.0,
        1710005500000u64,
        1710005500900u64,
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject pending request settlements that already carry realized accounting values"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("8301"), "{error}");
    assert!(error.contains("pending"), "{error}");
    assert!(error.contains("accounting"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_non_pending_request_settlement_without_settled_timestamp(
) {
    let bootstrap_root = temp_bootstrap_root(
        "profile-pack-invalid-request-settlement-non-pending-without-settled-timestamp",
    );
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("request-settlements").join("default.json"),
        &serde_json::json!([
            {
                "request_settlement_id": 8301u64,
                "tenant_id": 1001u64,
                "organization_id": 2001u64,
                "request_id": 6001u64,
                "account_id": 7001u64,
                "user_id": 9001u64,
                "hold_id": null,
                "status": "partially_released",
                "estimated_credit_hold": 2400.0,
                "released_credit_amount": 100.0,
                "captured_credit_amount": 2300.0,
                "provider_cost_amount": 0.27,
                "retail_charge_amount": 0.69,
                "shortfall_amount": 0.0,
                "refunded_amount": 0.0,
                "settled_at_ms": 0,
                "created_at_ms": 1710005500000u64,
                "updated_at_ms": 1710005500900u64
            }
        ]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject non-pending request settlements that omit settled_at_ms"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("8301"), "{error}");
    assert!(error.contains("status"), "{error}");
    assert!(error.contains("settled_at_ms"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_pending_request_settlement_with_captured_request_fact(
) {
    let bootstrap_root = temp_bootstrap_root(
        "profile-pack-invalid-request-settlement-pending-with-captured-request-fact",
    );
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("request-metering").join("default.json"),
        &serde_json::json!({
            "facts": [
                {
                    "request_id": 6001u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "user_id": 9001u64,
                    "account_id": 7001u64,
                    "api_key_id": 10001u64,
                    "api_key_hash": "a19d2bf76318aa7f619d684271469bb383faf1cb5bd4c680088465cde9d0003b",
                    "auth_type": "api_key",
                    "jwt_subject": null,
                    "platform": "portal",
                    "owner": "user_local_demo",
                    "request_trace_id": "trace-local-demo-growth-2026",
                    "gateway_request_ref": null,
                    "upstream_request_ref": null,
                    "protocol_family": "openai",
                    "capability_code": "responses",
                    "channel_code": "openrouter",
                    "model_code": "gpt-4.1",
                    "provider_code": "provider-openrouter-main",
                    "request_status": "succeeded",
                    "usage_capture_status": "captured",
                    "cost_pricing_plan_id": 9101u64,
                    "retail_pricing_plan_id": 9101u64,
                    "estimated_credit_hold": 2400.0,
                    "actual_credit_charge": 0.0,
                    "actual_provider_cost": 0.0,
                    "started_at_ms": 1710005500000u64,
                    "finished_at_ms": 1710005500900u64,
                    "created_at_ms": 1710005500000u64,
                    "updated_at_ms": 1710005500900u64
                }
            ],
            "metrics": []
        }),
    );
    write_json(
        &bootstrap_root.join("request-settlements").join("default.json"),
        &serde_json::json!([
            {
                "request_settlement_id": 8301u64,
                "tenant_id": 1001u64,
                "organization_id": 2001u64,
                "request_id": 6001u64,
                "account_id": 7001u64,
                "user_id": 9001u64,
                "hold_id": null,
                "status": "pending",
                "estimated_credit_hold": 2400.0,
                "released_credit_amount": 0.0,
                "captured_credit_amount": 0.0,
                "provider_cost_amount": 0.0,
                "retail_charge_amount": 0.0,
                "shortfall_amount": 0.0,
                "refunded_amount": 0.0,
                "settled_at_ms": 0,
                "created_at_ms": 1710005500000u64,
                "updated_at_ms": 1710005500900u64
            }
        ]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject pending request settlements linked to captured request meter facts"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("8301"), "{error}");
    assert!(error.contains("pending"), "{error}");
    assert!(error.contains("usage_capture_status"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_completed_request_settlement_with_estimated_request_fact(
) {
    let bootstrap_root = temp_bootstrap_root(
        "profile-pack-invalid-request-settlement-completed-with-estimated-request-fact",
    );
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("request-settlements").join("default.json"),
        &serde_json::json!([
            {
                "request_settlement_id": 8301u64,
                "tenant_id": 1001u64,
                "organization_id": 2001u64,
                "request_id": 6001u64,
                "account_id": 7001u64,
                "user_id": 9001u64,
                "hold_id": null,
                "status": "captured",
                "estimated_credit_hold": 0.0,
                "released_credit_amount": 0.0,
                "captured_credit_amount": 0.0,
                "provider_cost_amount": 0.0,
                "retail_charge_amount": 0.0,
                "shortfall_amount": 0.0,
                "refunded_amount": 0.0,
                "settled_at_ms": 1710005500900u64,
                "created_at_ms": 1710005500000u64,
                "updated_at_ms": 1710005500900u64
            }
        ]),
    );
    write_json(
        &bootstrap_root.join("request-metering").join("default.json"),
        &serde_json::json!({
            "facts": [
                {
                    "request_id": 6001u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "user_id": 9001u64,
                    "account_id": 7001u64,
                    "api_key_id": 10001u64,
                    "api_key_hash": "a19d2bf76318aa7f619d684271469bb383faf1cb5bd4c680088465cde9d0003b",
                    "auth_type": "api_key",
                    "jwt_subject": null,
                    "platform": "portal",
                    "owner": "user_local_demo",
                    "request_trace_id": "trace-local-demo-growth-2026",
                    "gateway_request_ref": null,
                    "upstream_request_ref": null,
                    "protocol_family": "openai",
                    "capability_code": "responses",
                    "channel_code": "openrouter",
                    "model_code": "gpt-4.1",
                    "provider_code": "provider-openrouter-main",
                    "request_status": "succeeded",
                    "usage_capture_status": "estimated",
                    "cost_pricing_plan_id": 9101u64,
                    "retail_pricing_plan_id": 9101u64,
                    "estimated_credit_hold": 0.0,
                    "actual_credit_charge": null,
                    "actual_provider_cost": null,
                    "started_at_ms": 1710005500000u64,
                    "finished_at_ms": null,
                    "created_at_ms": 1710005500000u64,
                    "updated_at_ms": 1710005500900u64
                }
            ],
            "metrics": []
        }),
    );
    write_single_account_hold_fixture(
        &bootstrap_root,
        6001u64,
        "held",
        0.0,
        0.0,
        0.0,
        1710005500000u64,
        1710005500900u64,
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject completed request settlements linked to estimated request meter facts"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("8301"), "{error}");
    assert!(error.contains("captured"), "{error}");
    assert!(error.contains("usage_capture_status"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_partially_released_request_settlement_without_released_amount(
) {
    let bootstrap_root = temp_bootstrap_root(
        "profile-pack-invalid-request-settlement-partially-released-without-release",
    );
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("request-settlements").join("default.json"),
        &serde_json::json!([
            {
                "request_settlement_id": 8301u64,
                "tenant_id": 1001u64,
                "organization_id": 2001u64,
                "request_id": 6001u64,
                "account_id": 7001u64,
                "user_id": 9001u64,
                "hold_id": null,
                "status": "partially_released",
                "estimated_credit_hold": 100.0,
                "released_credit_amount": 0.0,
                "captured_credit_amount": 100.0,
                "provider_cost_amount": 0.0,
                "retail_charge_amount": 0.0,
                "shortfall_amount": 0.0,
                "refunded_amount": 0.0,
                "settled_at_ms": 1710005500900u64,
                "created_at_ms": 1710005500000u64,
                "updated_at_ms": 1710005500900u64
            }
        ]),
    );
    write_json(
        &bootstrap_root.join("request-metering").join("default.json"),
        &serde_json::json!({
            "facts": [
                {
                    "request_id": 6001u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "user_id": 9001u64,
                    "account_id": 7001u64,
                    "api_key_id": 10001u64,
                    "api_key_hash": "a19d2bf76318aa7f619d684271469bb383faf1cb5bd4c680088465cde9d0003b",
                    "auth_type": "api_key",
                    "jwt_subject": null,
                    "platform": "portal",
                    "owner": "user_local_demo",
                    "request_trace_id": "trace-local-demo-growth-2026",
                    "gateway_request_ref": null,
                    "upstream_request_ref": null,
                    "protocol_family": "openai",
                    "capability_code": "responses",
                    "channel_code": "openrouter",
                    "model_code": "gpt-4.1",
                    "provider_code": "provider-openrouter-main",
                    "request_status": "succeeded",
                    "usage_capture_status": "captured",
                    "cost_pricing_plan_id": 9101u64,
                    "retail_pricing_plan_id": 9101u64,
                    "estimated_credit_hold": 100.0,
                    "actual_credit_charge": 100.0,
                    "actual_provider_cost": 0.0,
                    "started_at_ms": 1710005500000u64,
                    "finished_at_ms": 1710005500900u64,
                    "created_at_ms": 1710005500000u64,
                    "updated_at_ms": 1710005500900u64
                }
            ],
            "metrics": []
        }),
    );
    write_single_account_hold_fixture(
        &bootstrap_root,
        6001u64,
        "partially_released",
        100.0,
        50.0,
        50.0,
        1710005500000u64,
        1710005500900u64,
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject partially_released request settlements without released credit"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("8301"), "{error}");
    assert!(error.contains("partially_released"), "{error}");
    assert!(error.contains("released"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_partially_released_request_settlement_without_captured_amount(
) {
    let bootstrap_root = temp_bootstrap_root(
        "profile-pack-invalid-request-settlement-partially-released-without-capture",
    );
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("request-settlements").join("default.json"),
        &serde_json::json!([
            {
                "request_settlement_id": 8301u64,
                "tenant_id": 1001u64,
                "organization_id": 2001u64,
                "request_id": 6001u64,
                "account_id": 7001u64,
                "user_id": 9001u64,
                "hold_id": null,
                "status": "partially_released",
                "estimated_credit_hold": 100.0,
                "released_credit_amount": 100.0,
                "captured_credit_amount": 0.0,
                "provider_cost_amount": 0.0,
                "retail_charge_amount": 0.0,
                "shortfall_amount": 0.0,
                "refunded_amount": 0.0,
                "settled_at_ms": 1710005500900u64,
                "created_at_ms": 1710005500000u64,
                "updated_at_ms": 1710005500900u64
            }
        ]),
    );
    write_json(
        &bootstrap_root.join("request-metering").join("default.json"),
        &serde_json::json!({
            "facts": [
                {
                    "request_id": 6001u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "user_id": 9001u64,
                    "account_id": 7001u64,
                    "api_key_id": 10001u64,
                    "api_key_hash": "a19d2bf76318aa7f619d684271469bb383faf1cb5bd4c680088465cde9d0003b",
                    "auth_type": "api_key",
                    "jwt_subject": null,
                    "platform": "portal",
                    "owner": "user_local_demo",
                    "request_trace_id": "trace-local-demo-growth-2026",
                    "gateway_request_ref": null,
                    "upstream_request_ref": null,
                    "protocol_family": "openai",
                    "capability_code": "responses",
                    "channel_code": "openrouter",
                    "model_code": "gpt-4.1",
                    "provider_code": "provider-openrouter-main",
                    "request_status": "succeeded",
                    "usage_capture_status": "captured",
                    "cost_pricing_plan_id": 9101u64,
                    "retail_pricing_plan_id": 9101u64,
                    "estimated_credit_hold": 100.0,
                    "actual_credit_charge": 0.0,
                    "actual_provider_cost": 0.0,
                    "started_at_ms": 1710005500000u64,
                    "finished_at_ms": 1710005500900u64,
                    "created_at_ms": 1710005500000u64,
                    "updated_at_ms": 1710005500900u64
                }
            ],
            "metrics": []
        }),
    );
    write_single_account_hold_fixture(
        &bootstrap_root,
        6001u64,
        "partially_released",
        100.0,
        50.0,
        50.0,
        1710005500000u64,
        1710005500900u64,
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject partially_released request settlements without captured credit"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("8301"), "{error}");
    assert!(error.contains("partially_released"), "{error}");
    assert!(error.contains("captured"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_partially_released_request_settlement_with_refunded_amount(
) {
    let bootstrap_root = temp_bootstrap_root(
        "profile-pack-invalid-request-settlement-partially-released-with-refund",
    );
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("request-settlements").join("default.json"),
        &serde_json::json!([
            {
                "request_settlement_id": 8301u64,
                "tenant_id": 1001u64,
                "organization_id": 2001u64,
                "request_id": 6001u64,
                "account_id": 7001u64,
                "user_id": 9001u64,
                "hold_id": null,
                "status": "partially_released",
                "estimated_credit_hold": 100.0,
                "released_credit_amount": 50.0,
                "captured_credit_amount": 50.0,
                "provider_cost_amount": 0.0,
                "retail_charge_amount": 0.0,
                "shortfall_amount": 0.0,
                "refunded_amount": 10.0,
                "settled_at_ms": 1710005500900u64,
                "created_at_ms": 1710005500000u64,
                "updated_at_ms": 1710005500900u64
            }
        ]),
    );
    write_json(
        &bootstrap_root.join("request-metering").join("default.json"),
        &serde_json::json!({
            "facts": [
                {
                    "request_id": 6001u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "user_id": 9001u64,
                    "account_id": 7001u64,
                    "api_key_id": 10001u64,
                    "api_key_hash": "a19d2bf76318aa7f619d684271469bb383faf1cb5bd4c680088465cde9d0003b",
                    "auth_type": "api_key",
                    "jwt_subject": null,
                    "platform": "portal",
                    "owner": "user_local_demo",
                    "request_trace_id": "trace-local-demo-growth-2026",
                    "gateway_request_ref": null,
                    "upstream_request_ref": null,
                    "protocol_family": "openai",
                    "capability_code": "responses",
                    "channel_code": "openrouter",
                    "model_code": "gpt-4.1",
                    "provider_code": "provider-openrouter-main",
                    "request_status": "succeeded",
                    "usage_capture_status": "captured",
                    "cost_pricing_plan_id": 9101u64,
                    "retail_pricing_plan_id": 9101u64,
                    "estimated_credit_hold": 100.0,
                    "actual_credit_charge": 50.0,
                    "actual_provider_cost": 0.0,
                    "started_at_ms": 1710005500000u64,
                    "finished_at_ms": 1710005500900u64,
                    "created_at_ms": 1710005500000u64,
                    "updated_at_ms": 1710005500900u64
                }
            ],
            "metrics": []
        }),
    );
    write_single_account_hold_fixture(
        &bootstrap_root,
        6001u64,
        "partially_released",
        100.0,
        50.0,
        50.0,
        1710005500000u64,
        1710005500900u64,
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject partially_released request settlements that already carry refunded credit"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("8301"), "{error}");
    assert!(error.contains("partially_released"), "{error}");
    assert!(error.contains("refunded"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_pending_request_settlement_with_non_held_hold_status(
) {
    let bootstrap_root = temp_bootstrap_root(
        "profile-pack-invalid-request-settlement-pending-with-non-held-hold-status",
    );
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("account-holds").join("default.json"),
        &serde_json::json!({
            "holds": [
                {
                    "hold_id": 8101u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "account_id": 7001u64,
                    "user_id": 9001u64,
                    "request_id": 6001u64,
                    "status": "failed",
                    "estimated_quantity": 2400.0,
                    "captured_quantity": 0.0,
                    "released_quantity": 0.0,
                    "expires_at_ms": 1710006100000u64,
                    "created_at_ms": 1710005500000u64,
                    "updated_at_ms": 1710005500900u64
                }
            ],
            "allocations": [
                {
                    "hold_allocation_id": 8401u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "hold_id": 8101u64,
                    "lot_id": 8001u64,
                    "allocated_quantity": 2400.0,
                    "captured_quantity": 0.0,
                    "released_quantity": 0.0,
                    "created_at_ms": 1710005500000u64,
                    "updated_at_ms": 1710005500900u64
                }
            ]
        }),
    );
    write_json(
        &bootstrap_root.join("request-metering").join("default.json"),
        &serde_json::json!({
            "facts": [
                {
                    "request_id": 6001u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "user_id": 9001u64,
                    "account_id": 7001u64,
                    "api_key_id": 10001u64,
                    "api_key_hash": "a19d2bf76318aa7f619d684271469bb383faf1cb5bd4c680088465cde9d0003b",
                    "auth_type": "api_key",
                    "jwt_subject": null,
                    "platform": "portal",
                    "owner": "user_local_demo",
                    "request_trace_id": "trace-local-demo-growth-2026",
                    "gateway_request_ref": null,
                    "upstream_request_ref": null,
                    "protocol_family": "openai",
                    "capability_code": "responses",
                    "channel_code": "openrouter",
                    "model_code": "gpt-4.1",
                    "provider_code": "provider-openrouter-main",
                    "request_status": "succeeded",
                    "usage_capture_status": "estimated",
                    "cost_pricing_plan_id": 9101u64,
                    "retail_pricing_plan_id": 9101u64,
                    "estimated_credit_hold": 2400.0,
                    "actual_credit_charge": null,
                    "actual_provider_cost": null,
                    "started_at_ms": 1710005500000u64,
                    "finished_at_ms": null,
                    "created_at_ms": 1710005500000u64,
                    "updated_at_ms": 1710005500900u64
                }
            ],
            "metrics": []
        }),
    );
    write_json(
        &bootstrap_root.join("request-settlements").join("default.json"),
        &serde_json::json!([
            {
                "request_settlement_id": 8301u64,
                "tenant_id": 1001u64,
                "organization_id": 2001u64,
                "request_id": 6001u64,
                "account_id": 7001u64,
                "user_id": 9001u64,
                "hold_id": 8101u64,
                "status": "pending",
                "estimated_credit_hold": 2400.0,
                "released_credit_amount": 0.0,
                "captured_credit_amount": 0.0,
                "provider_cost_amount": 0.0,
                "retail_charge_amount": 0.0,
                "shortfall_amount": 0.0,
                "refunded_amount": 0.0,
                "settled_at_ms": 0,
                "created_at_ms": 1710005500000u64,
                "updated_at_ms": 1710005500900u64
            }
        ]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject pending request settlements whose linked hold is not held"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("8301"), "{error}");
    assert!(error.contains("pending"), "{error}");
    assert!(error.contains("hold"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_partially_released_request_settlement_with_non_partial_hold_status(
) {
    let bootstrap_root = temp_bootstrap_root(
        "profile-pack-invalid-request-settlement-partial-with-non-partial-hold-status",
    );
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("account-holds").join("default.json"),
        &serde_json::json!({
            "holds": [
                {
                    "hold_id": 8101u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "account_id": 7001u64,
                    "user_id": 9001u64,
                    "request_id": 6001u64,
                    "status": "captured",
                    "estimated_quantity": 2400.0,
                    "captured_quantity": 2300.0,
                    "released_quantity": 100.0,
                    "expires_at_ms": 1710006100000u64,
                    "created_at_ms": 1710005500000u64,
                    "updated_at_ms": 1710005500900u64
                }
            ],
            "allocations": [
                {
                    "hold_allocation_id": 8401u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "hold_id": 8101u64,
                    "lot_id": 8001u64,
                    "allocated_quantity": 2400.0,
                    "captured_quantity": 2300.0,
                    "released_quantity": 100.0,
                    "created_at_ms": 1710005500000u64,
                    "updated_at_ms": 1710005500900u64
                }
            ]
        }),
    );
    write_json(
        &bootstrap_root.join("request-metering").join("default.json"),
        &serde_json::json!({
            "facts": [
                {
                    "request_id": 6001u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "user_id": 9001u64,
                    "account_id": 7001u64,
                    "api_key_id": 10001u64,
                    "api_key_hash": "a19d2bf76318aa7f619d684271469bb383faf1cb5bd4c680088465cde9d0003b",
                    "auth_type": "api_key",
                    "jwt_subject": null,
                    "platform": "portal",
                    "owner": "user_local_demo",
                    "request_trace_id": "trace-local-demo-growth-2026",
                    "gateway_request_ref": null,
                    "upstream_request_ref": null,
                    "protocol_family": "openai",
                    "capability_code": "responses",
                    "channel_code": "openrouter",
                    "model_code": "gpt-4.1",
                    "provider_code": "provider-openrouter-main",
                    "request_status": "succeeded",
                    "usage_capture_status": "captured",
                    "cost_pricing_plan_id": 9101u64,
                    "retail_pricing_plan_id": 9101u64,
                    "estimated_credit_hold": 2400.0,
                    "actual_credit_charge": 2300.0,
                    "actual_provider_cost": 0.27,
                    "started_at_ms": 1710005500000u64,
                    "finished_at_ms": 1710005500900u64,
                    "created_at_ms": 1710005500000u64,
                    "updated_at_ms": 1710005500900u64
                }
            ],
            "metrics": []
        }),
    );
    write_json(
        &bootstrap_root.join("request-settlements").join("default.json"),
        &serde_json::json!([
            {
                "request_settlement_id": 8301u64,
                "tenant_id": 1001u64,
                "organization_id": 2001u64,
                "request_id": 6001u64,
                "account_id": 7001u64,
                "user_id": 9001u64,
                "hold_id": 8101u64,
                "status": "partially_released",
                "estimated_credit_hold": 2400.0,
                "released_credit_amount": 100.0,
                "captured_credit_amount": 2300.0,
                "provider_cost_amount": 0.27,
                "retail_charge_amount": 0.69,
                "shortfall_amount": 0.0,
                "refunded_amount": 0.0,
                "settled_at_ms": 1710005500900u64,
                "created_at_ms": 1710005500000u64,
                "updated_at_ms": 1710005500900u64
            }
        ]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject partially_released request settlements whose linked hold is not partially_released"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("8301"), "{error}");
    assert!(error.contains("partially_released"), "{error}");
    assert!(error.contains("hold"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_held_account_hold_with_realized_quantity()
{
    let bootstrap_root = temp_bootstrap_root("profile-pack-invalid-held-account-hold-realized");
    write_bootstrap_profile_pack(&bootstrap_root);

    let holds_path = bootstrap_root.join("account-holds").join("default.json");
    let mut holds =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&holds_path).unwrap())
            .unwrap();
    holds["holds"].as_array_mut().unwrap().push(serde_json::json!({
        "hold_id": 8102u64,
        "tenant_id": 1001u64,
        "organization_id": 2001u64,
        "account_id": 7001u64,
        "user_id": 9001u64,
        "request_id": 6002u64,
        "status": "held",
        "estimated_quantity": 1200.0,
        "captured_quantity": 1000.0,
        "released_quantity": 0.0,
        "expires_at_ms": 1710006200000u64,
        "created_at_ms": 1710005600000u64,
        "updated_at_ms": 1710005600500u64
    }));
    holds["allocations"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({
            "hold_allocation_id": 8402u64,
            "tenant_id": 1001u64,
            "organization_id": 2001u64,
            "hold_id": 8102u64,
            "lot_id": 8001u64,
            "allocated_quantity": 1200.0,
            "captured_quantity": 1000.0,
            "released_quantity": 0.0,
            "created_at_ms": 1710005600000u64,
            "updated_at_ms": 1710005600500u64
        }));
    write_json(&holds_path, &holds);
    write_json(
        &bootstrap_root.join("request-metering").join("default.json"),
        &serde_json::json!({
            "facts": [
                {
                    "request_id": 6001u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "user_id": 9001u64,
                    "account_id": 7001u64,
                    "api_key_id": 10001u64,
                    "api_key_hash": "a19d2bf76318aa7f619d684271469bb383faf1cb5bd4c680088465cde9d0003b",
                    "auth_type": "api_key",
                    "jwt_subject": null,
                    "platform": "portal",
                    "owner": "user_local_demo",
                    "request_trace_id": "trace-local-demo-growth-2026",
                    "gateway_request_ref": null,
                    "upstream_request_ref": null,
                    "protocol_family": "openai",
                    "capability_code": "responses",
                    "channel_code": "openrouter",
                    "model_code": "gpt-4.1",
                    "provider_code": "provider-openrouter-main",
                    "request_status": "succeeded",
                    "usage_capture_status": "captured",
                    "cost_pricing_plan_id": 9101u64,
                    "retail_pricing_plan_id": 9101u64,
                    "estimated_credit_hold": 2400.0,
                    "actual_credit_charge": 2300.0,
                    "actual_provider_cost": 0.27,
                    "started_at_ms": 1710005500000u64,
                    "finished_at_ms": 1710005500900u64,
                    "created_at_ms": 1710005500000u64,
                    "updated_at_ms": 1710005500900u64
                },
                {
                    "request_id": 6002u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "user_id": 9001u64,
                    "account_id": 7001u64,
                    "api_key_id": 10001u64,
                    "api_key_hash": "a19d2bf76318aa7f619d684271469bb383faf1cb5bd4c680088465cde9d0003b",
                    "auth_type": "api_key",
                    "jwt_subject": null,
                    "platform": "portal",
                    "owner": "user_local_demo",
                    "request_trace_id": "trace-local-demo-hold-posture-6002",
                    "gateway_request_ref": null,
                    "upstream_request_ref": null,
                    "protocol_family": "openai",
                    "capability_code": "responses",
                    "channel_code": "openrouter",
                    "model_code": "gpt-4.1",
                    "provider_code": "provider-openrouter-main",
                    "request_status": "succeeded",
                    "usage_capture_status": "estimated",
                    "cost_pricing_plan_id": 9101u64,
                    "retail_pricing_plan_id": 9101u64,
                    "estimated_credit_hold": 1200.0,
                    "actual_credit_charge": null,
                    "actual_provider_cost": null,
                    "started_at_ms": 1710005600000u64,
                    "finished_at_ms": null,
                    "created_at_ms": 1710005600000u64,
                    "updated_at_ms": 1710005600500u64
                }
            ],
            "metrics": [
                {
                    "request_metric_id": 7001001u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "request_id": 6001u64,
                    "metric_code": "token.input",
                    "quantity": 1800.0,
                    "provider_field": "prompt_tokens",
                    "source_kind": "provider",
                    "capture_stage": "final",
                    "is_billable": true,
                    "captured_at_ms": 1710005500850u64
                },
                {
                    "request_metric_id": 7001002u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "request_id": 6001u64,
                    "metric_code": "token.output",
                    "quantity": 600.0,
                    "provider_field": "completion_tokens",
                    "source_kind": "provider",
                    "capture_stage": "final",
                    "is_billable": true,
                    "captured_at_ms": 1710005500850u64
                }
            ]
        }),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => {
            panic!("bootstrap should reject held account holds that already carry realized quantity")
        }
        Err(error) => error.to_string(),
    };

    assert!(error.contains("8102"), "{error}");
    assert!(error.contains("held"), "{error}");
    assert!(error.contains("realized"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_partially_released_account_hold_without_captured_quantity(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-partially-released-account-hold-no-capture");
    write_bootstrap_profile_pack(&bootstrap_root);

    let holds_path = bootstrap_root.join("account-holds").join("default.json");
    let mut holds =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&holds_path).unwrap())
            .unwrap();
    holds["holds"].as_array_mut().unwrap().push(serde_json::json!({
        "hold_id": 8102u64,
        "tenant_id": 1001u64,
        "organization_id": 2001u64,
        "account_id": 7001u64,
        "user_id": 9001u64,
        "request_id": 6002u64,
        "status": "partially_released",
        "estimated_quantity": 1200.0,
        "captured_quantity": 0.0,
        "released_quantity": 200.0,
        "expires_at_ms": 1710006200000u64,
        "created_at_ms": 1710005600000u64,
        "updated_at_ms": 1710005600500u64
    }));
    holds["allocations"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({
            "hold_allocation_id": 8402u64,
            "tenant_id": 1001u64,
            "organization_id": 2001u64,
            "hold_id": 8102u64,
            "lot_id": 8001u64,
            "allocated_quantity": 1200.0,
            "captured_quantity": 0.0,
            "released_quantity": 200.0,
            "created_at_ms": 1710005600000u64,
            "updated_at_ms": 1710005600500u64
        }));
    write_json(&holds_path, &holds);
    write_json(
        &bootstrap_root.join("request-metering").join("default.json"),
        &serde_json::json!({
            "facts": [
                {
                    "request_id": 6001u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "user_id": 9001u64,
                    "account_id": 7001u64,
                    "api_key_id": 10001u64,
                    "api_key_hash": "a19d2bf76318aa7f619d684271469bb383faf1cb5bd4c680088465cde9d0003b",
                    "auth_type": "api_key",
                    "jwt_subject": null,
                    "platform": "portal",
                    "owner": "user_local_demo",
                    "request_trace_id": "trace-local-demo-growth-2026",
                    "gateway_request_ref": "req_local_demo_growth_2026",
                    "upstream_request_ref": "or-local-demo-growth-2026",
                    "protocol_family": "openai",
                    "capability_code": "responses",
                    "channel_code": "openrouter",
                    "model_code": "gpt-4.1",
                    "provider_code": "provider-openrouter-main",
                    "request_status": "succeeded",
                    "usage_capture_status": "captured",
                    "cost_pricing_plan_id": 9101u64,
                    "retail_pricing_plan_id": 9101u64,
                    "estimated_credit_hold": 2400.0,
                    "actual_credit_charge": 2300.0,
                    "actual_provider_cost": 0.27,
                    "started_at_ms": 1710005500000u64,
                    "finished_at_ms": 1710005500900u64,
                    "created_at_ms": 1710005500000u64,
                    "updated_at_ms": 1710005500900u64
                },
                {
                    "request_id": 6002u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "user_id": 9001u64,
                    "account_id": 7001u64,
                    "api_key_id": 10001u64,
                    "api_key_hash": "a19d2bf76318aa7f619d684271469bb383faf1cb5bd4c680088465cde9d0003b",
                    "auth_type": "api_key",
                    "jwt_subject": null,
                    "platform": "portal",
                    "owner": "user_local_demo",
                    "request_trace_id": "trace-local-demo-hold-posture-6002",
                    "gateway_request_ref": null,
                    "upstream_request_ref": null,
                    "protocol_family": "openai",
                    "capability_code": "responses",
                    "channel_code": "openrouter",
                    "model_code": "gpt-4.1",
                    "provider_code": "provider-openrouter-main",
                    "request_status": "succeeded",
                    "usage_capture_status": "captured",
                    "cost_pricing_plan_id": 9101u64,
                    "retail_pricing_plan_id": 9101u64,
                    "estimated_credit_hold": 1200.0,
                    "actual_credit_charge": 1000.0,
                    "actual_provider_cost": 0.12,
                    "started_at_ms": 1710005600000u64,
                    "finished_at_ms": 1710005600500u64,
                    "created_at_ms": 1710005600000u64,
                    "updated_at_ms": 1710005600500u64
                }
            ],
            "metrics": [
                {
                    "request_metric_id": 7001001u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "request_id": 6001u64,
                    "metric_code": "token.input",
                    "quantity": 1800.0,
                    "provider_field": "prompt_tokens",
                    "source_kind": "provider",
                    "capture_stage": "final",
                    "is_billable": true,
                    "captured_at_ms": 1710005500850u64
                },
                {
                    "request_metric_id": 7001002u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "request_id": 6001u64,
                    "metric_code": "token.output",
                    "quantity": 600.0,
                    "provider_field": "completion_tokens",
                    "source_kind": "provider",
                    "capture_stage": "final",
                    "is_billable": true,
                    "captured_at_ms": 1710005500850u64
                }
            ]
        }),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => {
            panic!("bootstrap should reject partially_released account holds without captured quantity")
        }
        Err(error) => error.to_string(),
    };

    assert!(error.contains("8102"), "{error}");
    assert!(error.contains("partially_released"), "{error}");
    assert!(error.contains("captured"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_partially_released_account_hold_without_released_quantity(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-partially-released-account-hold-no-release");
    write_bootstrap_profile_pack(&bootstrap_root);

    let holds_path = bootstrap_root.join("account-holds").join("default.json");
    let mut holds =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&holds_path).unwrap())
            .unwrap();
    holds["holds"].as_array_mut().unwrap().push(serde_json::json!({
        "hold_id": 8102u64,
        "tenant_id": 1001u64,
        "organization_id": 2001u64,
        "account_id": 7001u64,
        "user_id": 9001u64,
        "request_id": 6002u64,
        "status": "partially_released",
        "estimated_quantity": 1200.0,
        "captured_quantity": 1000.0,
        "released_quantity": 0.0,
        "expires_at_ms": 1710006200000u64,
        "created_at_ms": 1710005600000u64,
        "updated_at_ms": 1710005600500u64
    }));
    holds["allocations"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({
            "hold_allocation_id": 8402u64,
            "tenant_id": 1001u64,
            "organization_id": 2001u64,
            "hold_id": 8102u64,
            "lot_id": 8001u64,
            "allocated_quantity": 1200.0,
            "captured_quantity": 1000.0,
            "released_quantity": 0.0,
            "created_at_ms": 1710005600000u64,
            "updated_at_ms": 1710005600500u64
        }));
    write_json(&holds_path, &holds);
    write_json(
        &bootstrap_root.join("request-metering").join("default.json"),
        &serde_json::json!({
            "facts": [
                {
                    "request_id": 6001u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "user_id": 9001u64,
                    "account_id": 7001u64,
                    "api_key_id": 10001u64,
                    "api_key_hash": "a19d2bf76318aa7f619d684271469bb383faf1cb5bd4c680088465cde9d0003b",
                    "auth_type": "api_key",
                    "jwt_subject": null,
                    "platform": "portal",
                    "owner": "user_local_demo",
                    "request_trace_id": "trace-local-demo-growth-2026",
                    "gateway_request_ref": "req_local_demo_growth_2026",
                    "upstream_request_ref": "or-local-demo-growth-2026",
                    "protocol_family": "openai",
                    "capability_code": "responses",
                    "channel_code": "openrouter",
                    "model_code": "gpt-4.1",
                    "provider_code": "provider-openrouter-main",
                    "request_status": "succeeded",
                    "usage_capture_status": "captured",
                    "cost_pricing_plan_id": 9101u64,
                    "retail_pricing_plan_id": 9101u64,
                    "estimated_credit_hold": 2400.0,
                    "actual_credit_charge": 2300.0,
                    "actual_provider_cost": 0.27,
                    "started_at_ms": 1710005500000u64,
                    "finished_at_ms": 1710005500900u64,
                    "created_at_ms": 1710005500000u64,
                    "updated_at_ms": 1710005500900u64
                },
                {
                    "request_id": 6002u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "user_id": 9001u64,
                    "account_id": 7001u64,
                    "api_key_id": 10001u64,
                    "api_key_hash": "a19d2bf76318aa7f619d684271469bb383faf1cb5bd4c680088465cde9d0003b",
                    "auth_type": "api_key",
                    "jwt_subject": null,
                    "platform": "portal",
                    "owner": "user_local_demo",
                    "request_trace_id": "trace-local-demo-hold-posture-6002",
                    "gateway_request_ref": null,
                    "upstream_request_ref": null,
                    "protocol_family": "openai",
                    "capability_code": "responses",
                    "channel_code": "openrouter",
                    "model_code": "gpt-4.1",
                    "provider_code": "provider-openrouter-main",
                    "request_status": "succeeded",
                    "usage_capture_status": "captured",
                    "cost_pricing_plan_id": 9101u64,
                    "retail_pricing_plan_id": 9101u64,
                    "estimated_credit_hold": 1200.0,
                    "actual_credit_charge": 1000.0,
                    "actual_provider_cost": 0.12,
                    "started_at_ms": 1710005600000u64,
                    "finished_at_ms": 1710005600500u64,
                    "created_at_ms": 1710005600000u64,
                    "updated_at_ms": 1710005600500u64
                }
            ],
            "metrics": [
                {
                    "request_metric_id": 7001001u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "request_id": 6001u64,
                    "metric_code": "token.input",
                    "quantity": 1800.0,
                    "provider_field": "prompt_tokens",
                    "source_kind": "provider",
                    "capture_stage": "final",
                    "is_billable": true,
                    "captured_at_ms": 1710005500850u64
                },
                {
                    "request_metric_id": 7001002u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "request_id": 6001u64,
                    "metric_code": "token.output",
                    "quantity": 600.0,
                    "provider_field": "completion_tokens",
                    "source_kind": "provider",
                    "capture_stage": "final",
                    "is_billable": true,
                    "captured_at_ms": 1710005500850u64
                }
            ]
        }),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => {
            panic!("bootstrap should reject partially_released account holds without released quantity")
        }
        Err(error) => error.to_string(),
    };

    assert!(error.contains("8102"), "{error}");
    assert!(error.contains("partially_released"), "{error}");
    assert!(error.contains("released"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_account_hold_without_request_meter_fact() {
    let bootstrap_root = temp_bootstrap_root("profile-pack-invalid-account-hold-missing-fact");
    write_bootstrap_profile_pack(&bootstrap_root);

    let metering_path = bootstrap_root.join("request-metering").join("default.json");
    let mut metering =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&metering_path).unwrap())
            .unwrap();
    metering["facts"] = serde_json::json!([]);
    write_json(&metering_path, &metering);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!("bootstrap should reject account holds without a request meter fact"),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("8101"), "{error}");
    assert!(error.contains("request meter fact"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_account_hold_with_mismatched_request_meter_fact_ownership(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-account-hold-request-fact-ownership");
    write_bootstrap_profile_pack(&bootstrap_root);

    let metering_path = bootstrap_root.join("request-metering").join("default.json");
    let mut metering =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&metering_path).unwrap())
            .unwrap();
    metering["facts"][0]["tenant_id"] = serde_json::json!(9999u64);
    write_json(&metering_path, &metering);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject account holds whose request meter fact ownership drifts"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("8101"), "{error}");
    assert!(error.contains("ownership"), "{error}");
    assert!(error.contains("request meter fact"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_account_hold_with_estimated_quantity_mismatched_request_meter_fact(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-account-hold-request-fact-estimated-drift");
    write_bootstrap_profile_pack(&bootstrap_root);

    let holds_path = bootstrap_root.join("account-holds").join("default.json");
    let mut holds =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&holds_path).unwrap())
            .unwrap();
    holds["holds"][0]["estimated_quantity"] = serde_json::json!(2500.0);
    holds["allocations"][0]["allocated_quantity"] = serde_json::json!(2500.0);
    write_json(&holds_path, &holds);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject account holds whose estimated quantity drifts from request metering"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("8101"), "{error}");
    assert!(error.contains("estimated"), "{error}");
    assert!(error.contains("request meter fact"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_held_account_hold_with_captured_request_meter_fact(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-held-account-hold-captured-request-fact");
    write_bootstrap_profile_pack(&bootstrap_root);

    let holds_path = bootstrap_root.join("account-holds").join("default.json");
    let mut holds =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&holds_path).unwrap())
            .unwrap();
    holds["holds"][0]["status"] = serde_json::json!("held");
    holds["holds"][0]["captured_quantity"] = serde_json::json!(0.0);
    holds["holds"][0]["released_quantity"] = serde_json::json!(0.0);
    holds["allocations"][0]["captured_quantity"] = serde_json::json!(0.0);
    holds["allocations"][0]["released_quantity"] = serde_json::json!(0.0);
    write_json(&holds_path, &holds);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject held account holds linked to captured request meter facts"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("8101"), "{error}");
    assert!(error.contains("held"), "{error}");
    assert!(error.contains("estimated"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_partially_released_account_hold_with_estimated_request_meter_fact(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-partial-account-hold-estimated-request-fact");
    write_bootstrap_profile_pack(&bootstrap_root);

    let metering_path = bootstrap_root.join("request-metering").join("default.json");
    let mut metering =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&metering_path).unwrap())
            .unwrap();
    metering["facts"][0]["usage_capture_status"] = serde_json::json!("estimated");
    metering["facts"][0]["actual_credit_charge"] = serde_json::Value::Null;
    metering["facts"][0]["actual_provider_cost"] = serde_json::Value::Null;
    metering["facts"][0]["finished_at_ms"] = serde_json::Value::Null;
    write_json(&metering_path, &metering);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject partially_released account holds linked to estimated request meter facts"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("8101"), "{error}");
    assert!(error.contains("partially_released"), "{error}");
    assert!(error.contains("captured"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_account_hold_created_before_request_started(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-account-hold-created-before-request-started");
    write_bootstrap_profile_pack(&bootstrap_root);

    let holds_path = bootstrap_root.join("account-holds").join("default.json");
    let mut holds =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&holds_path).unwrap())
            .unwrap();
    holds["holds"][0]["created_at_ms"] = serde_json::json!(1710005499000u64);
    write_json(&holds_path, &holds);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject account holds created before the linked request started"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("8101"), "{error}");
    assert!(error.contains("created_at_ms"), "{error}");
    assert!(error.contains("started_at_ms"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_partially_released_account_hold_updated_before_request_finished(
) {
    let bootstrap_root = temp_bootstrap_root(
        "profile-pack-invalid-partial-account-hold-updated-before-request-finished",
    );
    write_bootstrap_profile_pack(&bootstrap_root);

    let holds_path = bootstrap_root.join("account-holds").join("default.json");
    let mut holds =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&holds_path).unwrap())
            .unwrap();
    holds["holds"][0]["updated_at_ms"] = serde_json::json!(1710005500800u64);
    write_json(&holds_path, &holds);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject partially_released account holds updated before request metering finished"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("8101"), "{error}");
    assert!(error.contains("updated_at_ms"), "{error}");
    assert!(error.contains("finished_at_ms"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_commerce_reconciliation_item_with_payment_attempt_outside_run_provider_context(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-commerce-reconciliation-item-attempt-provider-context");
    write_bootstrap_profile_pack(&bootstrap_root);

    let payment_methods_path = bootstrap_root.join("payment-methods").join("default.json");
    let mut payment_methods = serde_json::from_str::<serde_json::Value>(
        &fs::read_to_string(&payment_methods_path).unwrap(),
    )
    .unwrap();
    payment_methods["payment_methods"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({
            "payment_method_id": "payment-bank-transfer-manual",
            "display_name": "Enterprise Bank Transfer",
            "description": "Manual settlement for enterprise invoicing",
            "provider": "bank_transfer",
            "channel": "manual_review",
            "mode": "live",
            "enabled": true,
            "sort_order": 90,
            "capability_codes": ["checkout", "manual_review", "invoice"],
            "supported_currency_codes": ["USD", "EUR", "CNY"],
            "supported_country_codes": [],
            "supported_order_kinds": ["enterprise_contract", "custom_recharge"],
            "callback_strategy": "manual_audit",
            "webhook_path": null,
            "webhook_tolerance_seconds": 300u64,
            "replay_window_seconds": 300u64,
            "max_retry_count": 3u32,
            "config_json": "{\"provider\":\"bank_transfer\",\"approval\":\"manual\"}",
            "created_at_ms": 1710000000000u64,
            "updated_at_ms": 1710000000000u64
        }));
    write_json(&payment_methods_path, &payment_methods);

    let commerce_path = bootstrap_root.join("commerce").join("default.json");
    let mut commerce =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&commerce_path).unwrap())
            .unwrap();
    let reconciliation_run = commerce["reconciliation_runs"]
        .as_array_mut()
        .unwrap()
        .first_mut()
        .unwrap();
    reconciliation_run["provider"] = serde_json::json!("bank_transfer");
    reconciliation_run["payment_method_id"] = serde_json::json!("payment-bank-transfer-manual");
    write_json(&commerce_path, &commerce);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject commerce reconciliation items whose payment attempt drifts from the linked run provider context"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("recon-item-local-demo-growth-2026"), "{error}");
    assert!(error.contains("attempt-local-demo-growth-2026"), "{error}");
    assert!(error.contains("payment attempt"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_commerce_reconciliation_item_with_refund_outside_run_provider_context(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-commerce-reconciliation-item-refund-provider-context");
    write_bootstrap_profile_pack(&bootstrap_root);

    let payment_methods_path = bootstrap_root.join("payment-methods").join("default.json");
    let mut payment_methods = serde_json::from_str::<serde_json::Value>(
        &fs::read_to_string(&payment_methods_path).unwrap(),
    )
    .unwrap();
    payment_methods["payment_methods"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({
            "payment_method_id": "payment-bank-transfer-manual",
            "display_name": "Enterprise Bank Transfer",
            "description": "Manual settlement for enterprise invoicing",
            "provider": "bank_transfer",
            "channel": "manual_review",
            "mode": "live",
            "enabled": true,
            "sort_order": 90,
            "capability_codes": ["checkout", "manual_review", "invoice"],
            "supported_currency_codes": ["USD", "EUR", "CNY"],
            "supported_country_codes": [],
            "supported_order_kinds": ["enterprise_contract", "custom_recharge"],
            "callback_strategy": "manual_audit",
            "webhook_path": null,
            "webhook_tolerance_seconds": 300u64,
            "replay_window_seconds": 300u64,
            "max_retry_count": 3u32,
            "config_json": "{\"provider\":\"bank_transfer\",\"approval\":\"manual\"}",
            "created_at_ms": 1710000000000u64,
            "updated_at_ms": 1710000000000u64
        }));
    write_json(&payment_methods_path, &payment_methods);

    let commerce_path = bootstrap_root.join("commerce").join("default.json");
    let mut commerce =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&commerce_path).unwrap())
            .unwrap();
    let reconciliation_run = commerce["reconciliation_runs"]
        .as_array_mut()
        .unwrap()
        .first_mut()
        .unwrap();
    reconciliation_run["provider"] = serde_json::json!("bank_transfer");
    reconciliation_run["payment_method_id"] = serde_json::json!("payment-bank-transfer-manual");
    let reconciliation_item = commerce["reconciliation_items"]
        .as_array_mut()
        .unwrap()
        .first_mut()
        .unwrap();
    reconciliation_item["payment_attempt_id"] = serde_json::Value::Null;
    write_json(&commerce_path, &commerce);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject commerce reconciliation items whose refund drifts from the linked run provider context"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("recon-item-local-demo-growth-2026"), "{error}");
    assert!(error.contains("refund-local-demo-growth-2026"), "{error}");
    assert!(error.contains("refund"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_commerce_reconciliation_item_with_payment_attempt_outside_run_scope(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-commerce-reconciliation-item-attempt-scope");
    write_bootstrap_profile_pack(&bootstrap_root);

    let commerce_path = bootstrap_root.join("commerce").join("default.json");
    let mut commerce =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&commerce_path).unwrap())
            .unwrap();
    commerce["payment_attempts"]
        .as_array_mut()
        .unwrap()
        .first_mut()
        .unwrap()["updated_at_ms"] = serde_json::json!(1710005401000u64);
    write_json(&commerce_path, &commerce);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject reconciliation items whose linked payment attempt falls outside the run scope"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("recon-item-local-demo-growth-2026"), "{error}");
    assert!(error.contains("attempt-local-demo-growth-2026"), "{error}");
    assert!(error.contains("scope"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_commerce_reconciliation_item_with_refund_outside_run_scope(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-commerce-reconciliation-item-refund-scope");
    write_bootstrap_profile_pack(&bootstrap_root);

    let commerce_path = bootstrap_root.join("commerce").join("default.json");
    let mut commerce =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&commerce_path).unwrap())
            .unwrap();
    commerce["refunds"]
        .as_array_mut()
        .unwrap()
        .first_mut()
        .unwrap()["created_at_ms"] = serde_json::json!(1710004700000u64);
    write_json(&commerce_path, &commerce);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject reconciliation items whose linked refund falls outside the run scope"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("recon-item-local-demo-growth-2026"), "{error}");
    assert!(error.contains("refund-local-demo-growth-2026"), "{error}");
    assert!(error.contains("scope"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_commerce_reconciliation_item_with_external_reference_missing_payment_event(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-commerce-reconciliation-item-external-ref-missing-event");
    write_bootstrap_profile_pack(&bootstrap_root);

    let commerce_path = bootstrap_root.join("commerce").join("default.json");
    let mut commerce =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&commerce_path).unwrap())
            .unwrap();
    commerce["reconciliation_items"]
        .as_array_mut()
        .unwrap()
        .first_mut()
        .unwrap()["external_reference"] = serde_json::json!("stripe:evt_missing_local_demo_growth");
    write_json(&commerce_path, &commerce);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject reconciliation items whose external_reference does not resolve to a payment event"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("recon-item-local-demo-growth-2026"), "{error}");
    assert!(error.contains("external_reference"), "{error}");
    assert!(error.contains("payment event"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_commerce_reconciliation_item_with_external_reference_payment_event_outside_run_scope(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-commerce-reconciliation-item-external-ref-scope");
    write_bootstrap_profile_pack(&bootstrap_root);

    let commerce_path = bootstrap_root.join("commerce").join("default.json");
    let mut commerce =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&commerce_path).unwrap())
            .unwrap();
    let payment_event = commerce["payment_events"]
        .as_array_mut()
        .unwrap()
        .first_mut()
        .unwrap();
    payment_event["received_at_ms"] = serde_json::json!(1710005401000u64);
    payment_event["processed_at_ms"] = serde_json::json!(1710005401200u64);
    write_json(&commerce_path, &commerce);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject reconciliation items whose external_reference payment event falls outside the run scope"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("recon-item-local-demo-growth-2026"), "{error}");
    assert!(error.contains("external_reference"), "{error}");
    assert!(error.contains("scope"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_request_meter_fact_without_billing_event()
{
    let bootstrap_root = temp_bootstrap_root("profile-pack-invalid-request-meter-fact-missing-billing");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("billing").join("default.json"),
        &serde_json::json!({
            "billing_events": []
        }),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject request meter facts whose gateway_request_ref does not resolve to billing evidence"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("6001"), "{error}");
    assert!(error.contains("req_local_demo_growth_2026"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_request_meter_fact_without_executable_provider_account(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-request-meter-fact-provider-account");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("routing").join("default.json"),
        &serde_json::json!({
            "profiles": [
                {
                    "profile_id": "profile-global-balanced",
                    "tenant_id": "tenant_local_demo",
                    "project_id": "project_local_demo",
                    "name": "Global Balanced",
                    "slug": "global-balanced",
                    "description": "Balanced multi-provider routing",
                    "active": true,
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-openrouter-main",
                        "provider-ollama-local"
                    ],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "preferred_region": "global",
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ],
            "policies": [
                {
                    "policy_id": "policy-default-responses",
                    "capability": "responses",
                    "model_pattern": "*",
                    "enabled": true,
                    "priority": 100,
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-openrouter-main",
                        "provider-ollama-local"
                    ],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "execution_failover_enabled": true,
                    "upstream_retry_max_attempts": 3,
                    "upstream_retry_base_delay_ms": 250,
                    "upstream_retry_max_delay_ms": 2000
                }
            ],
            "project_preferences": [
                {
                    "project_id": "project_local_demo",
                    "preset_id": "profile-global-balanced",
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-openrouter-main",
                        "provider-ollama-local"
                    ],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "preferred_region": "global",
                    "updated_at_ms": 1710000000000u64
                }
            ]
        }),
    );
    write_json(
        &bootstrap_root
            .join("provider-accounts")
            .join("default.json"),
        &serde_json::json!([
            {
                "provider_account_id": "acct-openrouter-default",
                "provider_id": "provider-openrouter-main",
                "display_name": "OpenRouter Default",
                "account_kind": "api_key",
                "owner_scope": "platform",
                "owner_tenant_id": null,
                "execution_instance_id": "provider-openrouter-main",
                "base_url_override": "https://openrouter.ai/api/v1",
                "region": "global",
                "priority": 100,
                "weight": 10,
                "enabled": true,
                "routing_tags": ["default", "global"],
                "health_score_hint": null,
                "latency_ms_hint": null,
                "cost_hint": null,
                "success_rate_hint": null,
                "throughput_hint": null,
                "max_concurrency": null,
                "daily_budget": null,
                "notes": "bootstrap default account"
            },
            {
                "provider_account_id": "acct-ollama-local-default",
                "provider_id": "provider-ollama-local",
                "display_name": "Ollama Local Default",
                "account_kind": "runtime_instance",
                "owner_scope": "platform",
                "owner_tenant_id": null,
                "execution_instance_id": "provider-ollama-local",
                "base_url_override": "http://127.0.0.1:11434",
                "region": "local",
                "priority": 90,
                "weight": 5,
                "enabled": true,
                "routing_tags": ["default", "local"],
                "health_score_hint": null,
                "latency_ms_hint": 35,
                "cost_hint": 0.0,
                "success_rate_hint": null,
                "throughput_hint": null,
                "max_concurrency": null,
                "daily_budget": null,
                "notes": "bootstrap local account"
            }
        ]),
    );
    write_json(
        &bootstrap_root.join("billing").join("default.json"),
        &serde_json::json!({
            "billing_events": []
        }),
    );
    write_json(
        &bootstrap_root.join("request-metering").join("default.json"),
        &serde_json::json!({
            "facts": [
                {
                    "request_id": 6001u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "user_id": 9001u64,
                    "account_id": 7001u64,
                    "api_key_id": 10001u64,
                    "api_key_hash": "a19d2bf76318aa7f619d684271469bb383faf1cb5bd4c680088465cde9d0003b",
                    "auth_type": "api_key",
                    "jwt_subject": null,
                    "platform": "portal",
                    "owner": "user_local_demo",
                    "request_trace_id": "trace-local-demo-growth-2026",
                    "gateway_request_ref": null,
                    "upstream_request_ref": null,
                    "protocol_family": "openai",
                    "capability_code": "responses",
                    "channel_code": "openrouter",
                    "model_code": "gpt-4.1",
                    "provider_code": "provider-openrouter-main",
                    "request_status": "succeeded",
                    "usage_capture_status": "captured",
                    "cost_pricing_plan_id": 9101u64,
                    "retail_pricing_plan_id": 9101u64,
                    "estimated_credit_hold": 2400.0,
                    "actual_credit_charge": 2300.0,
                    "actual_provider_cost": 0.27,
                    "started_at_ms": 1710005500000u64,
                    "finished_at_ms": 1710005500900u64,
                    "created_at_ms": 1710005500000u64,
                    "updated_at_ms": 1710005500900u64
                },
                {
                    "request_id": 6009u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "user_id": 9001u64,
                    "account_id": 7001u64,
                    "api_key_id": null,
                    "api_key_hash": null,
                    "auth_type": "workspace_session",
                    "jwt_subject": null,
                    "platform": "portal",
                    "owner": "user_local_demo",
                    "request_trace_id": "trace-local-siliconflow-without-account",
                    "gateway_request_ref": null,
                    "upstream_request_ref": "siliconflow-local-no-account",
                    "protocol_family": "openai",
                    "capability_code": "responses",
                    "channel_code": "siliconflow",
                    "model_code": "qwen-plus-latest",
                    "provider_code": "provider-siliconflow-main",
                    "request_status": "pending",
                    "usage_capture_status": "pending",
                    "cost_pricing_plan_id": null,
                    "retail_pricing_plan_id": null,
                    "estimated_credit_hold": 0.0,
                    "actual_credit_charge": null,
                    "actual_provider_cost": null,
                    "started_at_ms": 1710005510000u64,
                    "finished_at_ms": null,
                    "created_at_ms": 1710005510000u64,
                    "updated_at_ms": 1710005510100u64
                }
            ],
            "metrics": []
        }),
    );
    write_json(
        &bootstrap_root.join("request-settlements").join("default.json"),
        &serde_json::json!([]),
    );
    write_single_account_hold_fixture(
        &bootstrap_root,
        6001u64,
        "partially_released",
        2400.0,
        2300.0,
        100.0,
        1710005500000u64,
        1710005500900u64,
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject request meter facts whose provider has no executable provider account"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("6009"), "{error}");
    assert!(error.contains("provider-siliconflow-main"), "{error}");
    assert!(error.contains("provider account"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_request_meter_fact_provider_context_mismatched_billing_event(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-request-meter-fact-provider-context");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("request-metering").join("default.json"),
        &serde_json::json!({
            "facts": [
                {
                    "request_id": 6001u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "user_id": 9001u64,
                    "account_id": 7001u64,
                    "api_key_id": 10001u64,
                    "api_key_hash": "a19d2bf76318aa7f619d684271469bb383faf1cb5bd4c680088465cde9d0003b",
                    "auth_type": "api_key",
                    "jwt_subject": null,
                    "platform": "portal",
                    "owner": "user_local_demo",
                    "request_trace_id": "trace-local-demo-growth-2026",
                    "gateway_request_ref": "req_local_demo_growth_2026",
                    "upstream_request_ref": "sf-local-demo-growth-2026",
                    "protocol_family": "openai",
                    "capability_code": "responses",
                    "channel_code": "siliconflow",
                    "model_code": "qwen-plus-latest",
                    "provider_code": "provider-siliconflow-main",
                    "request_status": "succeeded",
                    "usage_capture_status": "captured",
                    "cost_pricing_plan_id": 9101u64,
                    "retail_pricing_plan_id": 9101u64,
                    "estimated_credit_hold": 2400.0,
                    "actual_credit_charge": 2300.0,
                    "actual_provider_cost": 0.27,
                    "started_at_ms": 1710005500000u64,
                    "finished_at_ms": 1710005500900u64,
                    "created_at_ms": 1710005500000u64,
                    "updated_at_ms": 1710005500900u64
                }
            ],
            "metrics": [
                {
                    "request_metric_id": 7001001u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "request_id": 6001u64,
                    "metric_code": "token.input",
                    "quantity": 1800.0,
                    "provider_field": "prompt_tokens",
                    "source_kind": "provider",
                    "capture_stage": "final",
                    "is_billable": true,
                    "captured_at_ms": 1710005500850u64
                },
                {
                    "request_metric_id": 7001002u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "request_id": 6001u64,
                    "metric_code": "token.output",
                    "quantity": 600.0,
                    "provider_field": "completion_tokens",
                    "source_kind": "provider",
                    "capture_stage": "final",
                    "is_billable": true,
                    "captured_at_ms": 1710005500850u64
                }
            ]
        }),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject request meter facts whose provider or channel context drifts from billing evidence"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("6001"), "{error}");
    assert!(error.contains("provider-siliconflow-main"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_request_meter_fact_capability_mismatched_billing_event(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-request-meter-fact-capability");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("request-metering").join("default.json"),
        &serde_json::json!({
            "facts": [
                {
                    "request_id": 6001u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "user_id": 9001u64,
                    "account_id": 7001u64,
                    "api_key_id": 10001u64,
                    "api_key_hash": "a19d2bf76318aa7f619d684271469bb383faf1cb5bd4c680088465cde9d0003b",
                    "auth_type": "api_key",
                    "jwt_subject": null,
                    "platform": "portal",
                    "owner": "user_local_demo",
                    "request_trace_id": "trace-local-demo-growth-2026",
                    "gateway_request_ref": "req_local_demo_growth_2026",
                    "upstream_request_ref": "or-local-demo-growth-2026",
                    "protocol_family": "openai",
                    "capability_code": "chat_completions",
                    "channel_code": "openrouter",
                    "model_code": "deepseek-chat",
                    "provider_code": "provider-openrouter-main",
                    "request_status": "succeeded",
                    "usage_capture_status": "captured",
                    "cost_pricing_plan_id": 9101u64,
                    "retail_pricing_plan_id": 9101u64,
                    "estimated_credit_hold": 2400.0,
                    "actual_credit_charge": 2300.0,
                    "actual_provider_cost": 0.27,
                    "started_at_ms": 1710005500000u64,
                    "finished_at_ms": 1710005500900u64,
                    "created_at_ms": 1710005500000u64,
                    "updated_at_ms": 1710005500900u64
                }
            ],
            "metrics": [
                {
                    "request_metric_id": 7001001u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "request_id": 6001u64,
                    "metric_code": "token.input",
                    "quantity": 1800.0,
                    "provider_field": "prompt_tokens",
                    "source_kind": "provider",
                    "capture_stage": "final",
                    "is_billable": true,
                    "captured_at_ms": 1710005500850u64
                },
                {
                    "request_metric_id": 7001002u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "request_id": 6001u64,
                    "metric_code": "token.output",
                    "quantity": 600.0,
                    "provider_field": "completion_tokens",
                    "source_kind": "provider",
                    "capture_stage": "final",
                    "is_billable": true,
                    "captured_at_ms": 1710005500850u64
                }
            ]
        }),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject request meter facts whose capability drifts from billing evidence"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("6001"), "{error}");
    assert!(error.contains("chat_completions"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_request_meter_fact_provider_cost_mismatched_billing_event(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-request-meter-fact-provider-cost");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("request-metering").join("default.json"),
        &serde_json::json!({
            "facts": [
                {
                    "request_id": 6001u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "user_id": 9001u64,
                    "account_id": 7001u64,
                    "api_key_id": 10001u64,
                    "api_key_hash": "a19d2bf76318aa7f619d684271469bb383faf1cb5bd4c680088465cde9d0003b",
                    "auth_type": "api_key",
                    "jwt_subject": null,
                    "platform": "portal",
                    "owner": "user_local_demo",
                    "request_trace_id": "trace-local-demo-growth-2026",
                    "gateway_request_ref": "req_local_demo_growth_2026",
                    "upstream_request_ref": "or-local-demo-growth-2026",
                    "protocol_family": "openai",
                    "capability_code": "responses",
                    "channel_code": "openrouter",
                    "model_code": "gpt-4.1",
                    "provider_code": "provider-openrouter-main",
                    "request_status": "succeeded",
                    "usage_capture_status": "captured",
                    "cost_pricing_plan_id": 9101u64,
                    "retail_pricing_plan_id": 9101u64,
                    "estimated_credit_hold": 2400.0,
                    "actual_credit_charge": 2300.0,
                    "actual_provider_cost": 0.28,
                    "started_at_ms": 1710005500000u64,
                    "finished_at_ms": 1710005500900u64,
                    "created_at_ms": 1710005500000u64,
                    "updated_at_ms": 1710005500900u64
                }
            ],
            "metrics": [
                {
                    "request_metric_id": 7001001u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "request_id": 6001u64,
                    "metric_code": "token.input",
                    "quantity": 1800.0,
                    "provider_field": "prompt_tokens",
                    "source_kind": "provider",
                    "capture_stage": "final",
                    "is_billable": true,
                    "captured_at_ms": 1710005500850u64
                },
                {
                    "request_metric_id": 7001002u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "request_id": 6001u64,
                    "metric_code": "token.output",
                    "quantity": 600.0,
                    "provider_field": "completion_tokens",
                    "source_kind": "provider",
                    "capture_stage": "final",
                    "is_billable": true,
                    "captured_at_ms": 1710005500850u64
                }
            ]
        }),
    );
    write_json(
        &bootstrap_root.join("request-settlements").join("default.json"),
        &serde_json::json!([
            {
                "request_settlement_id": 8301u64,
                "tenant_id": 1001u64,
                "organization_id": 2001u64,
                "request_id": 6001u64,
                "account_id": 7001u64,
                "user_id": 9001u64,
                "hold_id": 8101u64,
                "status": "partially_released",
                "estimated_credit_hold": 2400.0,
                "released_credit_amount": 100.0,
                "captured_credit_amount": 2300.0,
                "provider_cost_amount": 0.28,
                "retail_charge_amount": 0.69,
                "shortfall_amount": 0.0,
                "refunded_amount": 0.0,
                "settled_at_ms": 1710005500900u64,
                "created_at_ms": 1710005500000u64,
                "updated_at_ms": 1710005500900u64
            }
        ]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject request meter facts whose actual provider cost drifts from billing evidence"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("6001"), "{error}");
    assert!(error.contains("actual_provider_cost"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_request_meter_fact_model_code_mismatched_billing_route_key(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-request-meter-fact-route-key");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("request-metering").join("default.json"),
        &serde_json::json!({
            "facts": [
                {
                    "request_id": 6001u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "user_id": 9001u64,
                    "account_id": 7001u64,
                    "api_key_id": 10001u64,
                    "api_key_hash": "a19d2bf76318aa7f619d684271469bb383faf1cb5bd4c680088465cde9d0003b",
                    "auth_type": "api_key",
                    "jwt_subject": null,
                    "platform": "portal",
                    "owner": "user_local_demo",
                    "request_trace_id": "trace-local-demo-growth-2026",
                    "gateway_request_ref": "req_local_demo_growth_2026",
                    "upstream_request_ref": "or-local-demo-growth-2026",
                    "protocol_family": "openai",
                    "capability_code": "responses",
                    "channel_code": "openrouter",
                    "model_code": "deepseek-chat",
                    "provider_code": "provider-openrouter-main",
                    "request_status": "succeeded",
                    "usage_capture_status": "captured",
                    "cost_pricing_plan_id": 9101u64,
                    "retail_pricing_plan_id": 9101u64,
                    "estimated_credit_hold": 2400.0,
                    "actual_credit_charge": 2300.0,
                    "actual_provider_cost": 0.27,
                    "started_at_ms": 1710005500000u64,
                    "finished_at_ms": 1710005500900u64,
                    "created_at_ms": 1710005500000u64,
                    "updated_at_ms": 1710005500900u64
                }
            ],
            "metrics": [
                {
                    "request_metric_id": 7001001u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "request_id": 6001u64,
                    "metric_code": "token.input",
                    "quantity": 1800.0,
                    "provider_field": "prompt_tokens",
                    "source_kind": "provider",
                    "capture_stage": "final",
                    "is_billable": true,
                    "captured_at_ms": 1710005500850u64
                },
                {
                    "request_metric_id": 7001002u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "request_id": 6001u64,
                    "metric_code": "token.output",
                    "quantity": 600.0,
                    "provider_field": "completion_tokens",
                    "source_kind": "provider",
                    "capture_stage": "final",
                    "is_billable": true,
                    "captured_at_ms": 1710005500850u64
                }
            ]
        }),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject request meter facts whose model_code drifts from billing route_key"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("6001"), "{error}");
    assert!(error.contains("model_code"), "{error}");
    assert!(error.contains("route_key"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_request_meter_fact_api_key_hash_mismatched_billing_event(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-request-meter-fact-api-key-hash");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("request-metering").join("default.json"),
        &serde_json::json!({
            "facts": [
                {
                    "request_id": 6001u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "user_id": 9001u64,
                    "account_id": 7001u64,
                    "api_key_id": 10001u64,
                    "api_key_hash": "13072ae2c436e62116c61d76c68e7cc32a7a1e252a1d192490d6ac7cc92295eb",
                    "auth_type": "api_key",
                    "jwt_subject": null,
                    "platform": "portal",
                    "owner": "user_local_demo",
                    "request_trace_id": "trace-local-demo-growth-2026",
                    "gateway_request_ref": "req_local_demo_growth_2026",
                    "upstream_request_ref": "or-local-demo-growth-2026",
                    "protocol_family": "openai",
                    "capability_code": "responses",
                    "channel_code": "openrouter",
                    "model_code": "gpt-4.1",
                    "provider_code": "provider-openrouter-main",
                    "request_status": "succeeded",
                    "usage_capture_status": "captured",
                    "cost_pricing_plan_id": 9101u64,
                    "retail_pricing_plan_id": 9101u64,
                    "estimated_credit_hold": 2400.0,
                    "actual_credit_charge": 2300.0,
                    "actual_provider_cost": 0.27,
                    "started_at_ms": 1710005500000u64,
                    "finished_at_ms": 1710005500900u64,
                    "created_at_ms": 1710005500000u64,
                    "updated_at_ms": 1710005500900u64
                }
            ],
            "metrics": [
                {
                    "request_metric_id": 7001001u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "request_id": 6001u64,
                    "metric_code": "token.input",
                    "quantity": 1800.0,
                    "provider_field": "prompt_tokens",
                    "source_kind": "provider",
                    "capture_stage": "final",
                    "is_billable": true,
                    "captured_at_ms": 1710005500850u64
                },
                {
                    "request_metric_id": 7001002u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "request_id": 6001u64,
                    "metric_code": "token.output",
                    "quantity": 600.0,
                    "provider_field": "completion_tokens",
                    "source_kind": "provider",
                    "capture_stage": "final",
                    "is_billable": true,
                    "captured_at_ms": 1710005500850u64
                }
            ]
        }),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject request meter facts whose api_key_hash drifts from billing evidence"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("6001"), "{error}");
    assert!(error.contains("api_key_hash"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_request_meter_fact_without_token_input_metric_for_billing_tokens(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-request-meter-fact-missing-input-metric");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("request-metering").join("default.json"),
        &serde_json::json!({
            "facts": [
                {
                    "request_id": 6001u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "user_id": 9001u64,
                    "account_id": 7001u64,
                    "api_key_id": 10001u64,
                    "api_key_hash": "a19d2bf76318aa7f619d684271469bb383faf1cb5bd4c680088465cde9d0003b",
                    "auth_type": "api_key",
                    "jwt_subject": null,
                    "platform": "portal",
                    "owner": "user_local_demo",
                    "request_trace_id": "trace-local-demo-growth-2026",
                    "gateway_request_ref": "req_local_demo_growth_2026",
                    "upstream_request_ref": "or-local-demo-growth-2026",
                    "protocol_family": "openai",
                    "capability_code": "responses",
                    "channel_code": "openrouter",
                    "model_code": "gpt-4.1",
                    "provider_code": "provider-openrouter-main",
                    "request_status": "succeeded",
                    "usage_capture_status": "captured",
                    "cost_pricing_plan_id": 9101u64,
                    "retail_pricing_plan_id": 9101u64,
                    "estimated_credit_hold": 2400.0,
                    "actual_credit_charge": 2300.0,
                    "actual_provider_cost": 0.27,
                    "started_at_ms": 1710005500000u64,
                    "finished_at_ms": 1710005500900u64,
                    "created_at_ms": 1710005500000u64,
                    "updated_at_ms": 1710005500900u64
                }
            ],
            "metrics": [
                {
                    "request_metric_id": 7001002u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "request_id": 6001u64,
                    "metric_code": "token.output",
                    "quantity": 600.0,
                    "provider_field": "completion_tokens",
                    "source_kind": "provider",
                    "capture_stage": "final",
                    "is_billable": true,
                    "captured_at_ms": 1710005500850u64
                }
            ]
        }),
    );
    write_json(
        &bootstrap_root.join("request-settlements").join("default.json"),
        &serde_json::json!([]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject request metering when billing input_tokens exist but token.input metrics are missing"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("6001"), "{error}");
    assert!(error.contains("token.input"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_request_meter_fact_token_output_mismatched_billing_event(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-request-meter-fact-token-output-drift");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("request-metering").join("default.json"),
        &serde_json::json!({
            "facts": [
                {
                    "request_id": 6001u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "user_id": 9001u64,
                    "account_id": 7001u64,
                    "api_key_id": 10001u64,
                    "api_key_hash": "a19d2bf76318aa7f619d684271469bb383faf1cb5bd4c680088465cde9d0003b",
                    "auth_type": "api_key",
                    "jwt_subject": null,
                    "platform": "portal",
                    "owner": "user_local_demo",
                    "request_trace_id": "trace-local-demo-growth-2026",
                    "gateway_request_ref": "req_local_demo_growth_2026",
                    "upstream_request_ref": "or-local-demo-growth-2026",
                    "protocol_family": "openai",
                    "capability_code": "responses",
                    "channel_code": "openrouter",
                    "model_code": "gpt-4.1",
                    "provider_code": "provider-openrouter-main",
                    "request_status": "succeeded",
                    "usage_capture_status": "captured",
                    "cost_pricing_plan_id": 9101u64,
                    "retail_pricing_plan_id": 9101u64,
                    "estimated_credit_hold": 2400.0,
                    "actual_credit_charge": 2300.0,
                    "actual_provider_cost": 0.27,
                    "started_at_ms": 1710005500000u64,
                    "finished_at_ms": 1710005500900u64,
                    "created_at_ms": 1710005500000u64,
                    "updated_at_ms": 1710005500900u64
                }
            ],
            "metrics": [
                {
                    "request_metric_id": 7001001u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "request_id": 6001u64,
                    "metric_code": "token.input",
                    "quantity": 1800.0,
                    "provider_field": "prompt_tokens",
                    "source_kind": "provider",
                    "capture_stage": "final",
                    "is_billable": true,
                    "captured_at_ms": 1710005500850u64
                },
                {
                    "request_metric_id": 7001002u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "request_id": 6001u64,
                    "metric_code": "token.output",
                    "quantity": 599.0,
                    "provider_field": "completion_tokens",
                    "source_kind": "provider",
                    "capture_stage": "final",
                    "is_billable": true,
                    "captured_at_ms": 1710005500850u64
                }
            ]
        }),
    );
    write_json(
        &bootstrap_root.join("request-settlements").join("default.json"),
        &serde_json::json!([]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject request metering when token output usage drifts from billing evidence"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("6001"), "{error}");
    assert!(error.contains("token.output"), "{error}");
    assert!(error.contains("599"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_request_meter_fact_with_capability_not_supported_by_channel_model(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-request-meter-capability");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("billing").join("default.json"),
        &serde_json::json!({
            "billing_events": [
                {
                    "event_id": "billing-local-demo-growth-2026",
                    "tenant_id": "tenant_local_demo",
                    "project_id": "project_local_demo",
                    "api_key_group_id": "group-local-demo-live",
                    "capability": "embeddings",
                    "route_key": "gpt-4.1",
                    "usage_model": "gpt-4.1",
                    "provider_id": "provider-openrouter-main",
                    "accounting_mode": "platform_credit",
                    "operation_kind": "request",
                    "modality": "text",
                    "api_key_hash": "a19d2bf76318aa7f619d684271469bb383faf1cb5bd4c680088465cde9d0003b",
                    "channel_id": "openrouter",
                    "reference_id": "req_local_demo_growth_2026",
                    "latency_ms": 540u64,
                    "units": 1u64,
                    "request_count": 1u64,
                    "input_tokens": 1800u64,
                    "output_tokens": 600u64,
                    "total_tokens": 2400u64,
                    "cache_read_tokens": 0u64,
                    "cache_write_tokens": 0u64,
                    "image_count": 0u64,
                    "audio_seconds": 0.0,
                    "video_seconds": 0.0,
                    "music_seconds": 0.0,
                    "upstream_cost": 0.27,
                    "customer_charge": 0.69,
                    "applied_routing_profile_id": "profile-global-balanced",
                    "compiled_routing_snapshot_id": "snapshot-local-demo-live-responses",
                    "fallback_reason": null,
                    "created_at_ms": 1710005500000u64
                }
            ]
        }),
    );
    write_json(
        &bootstrap_root.join("observability").join("default.json"),
        &serde_json::json!({
            "compiled_routing_snapshots": [
                {
                    "snapshot_id": "snapshot-local-demo-live-responses",
                    "tenant_id": "tenant_local_demo",
                    "project_id": "project_local_demo",
                    "api_key_group_id": "group-local-demo-live",
                    "capability": "embeddings",
                    "route_key": "gpt-4.1",
                    "matched_policy_id": "policy-default-responses",
                    "project_routing_preferences_project_id": "project_local_demo",
                    "applied_routing_profile_id": "profile-global-balanced",
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-openrouter-main",
                        "provider-siliconflow-main",
                        "provider-ollama-local"
                    ],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000u64,
                    "require_healthy": false,
                    "preferred_region": "global",
                    "created_at_ms": 1710003000000u64,
                    "updated_at_ms": 1710003000500u64
                }
            ],
            "routing_decision_logs": [],
            "provider_health_snapshots": []
        }),
    );
    write_json(
        &bootstrap_root.join("request-metering").join("default.json"),
        &serde_json::json!({
            "facts": [
                {
                    "request_id": 6001u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "user_id": 9001u64,
                    "account_id": 7001u64,
                    "api_key_id": 10001u64,
                    "api_key_hash": "a19d2bf76318aa7f619d684271469bb383faf1cb5bd4c680088465cde9d0003b",
                    "auth_type": "api_key",
                    "jwt_subject": null,
                    "platform": "portal",
                    "owner": "user_local_demo",
                    "request_trace_id": "trace-local-demo-growth-2026",
                    "gateway_request_ref": "req_local_demo_growth_2026",
                    "upstream_request_ref": "or-local-demo-growth-2026",
                    "protocol_family": "openai",
                    "capability_code": "embeddings",
                    "channel_code": "openrouter",
                    "model_code": "gpt-4.1",
                    "provider_code": "provider-openrouter-main",
                    "request_status": "succeeded",
                    "usage_capture_status": "captured",
                    "cost_pricing_plan_id": 9101u64,
                    "retail_pricing_plan_id": 9101u64,
                    "estimated_credit_hold": 2400.0,
                    "actual_credit_charge": 2300.0,
                    "actual_provider_cost": 0.27,
                    "started_at_ms": 1710005500000u64,
                    "finished_at_ms": 1710005500900u64,
                    "created_at_ms": 1710005500000u64,
                    "updated_at_ms": 1710005500900u64
                }
            ],
            "metrics": [
                {
                    "request_metric_id": 7001001u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "request_id": 6001u64,
                    "metric_code": "token.input",
                    "quantity": 1800.0,
                    "provider_field": "prompt_tokens",
                    "source_kind": "provider",
                    "capture_stage": "final",
                    "is_billable": true,
                    "captured_at_ms": 1710005500850u64
                },
                {
                    "request_metric_id": 7001002u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "request_id": 6001u64,
                    "metric_code": "token.output",
                    "quantity": 600.0,
                    "provider_field": "completion_tokens",
                    "source_kind": "provider",
                    "capture_stage": "final",
                    "is_billable": true,
                    "captured_at_ms": 1710005500850u64
                }
            ]
        }),
    );
    write_json(
        &bootstrap_root.join("request-settlements").join("default.json"),
        &serde_json::json!([]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject request metering facts whose declared capability is not supported by the channel model"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("6001"), "{error}");
    assert!(error.contains("gpt-4.1"), "{error}");
    assert!(error.contains("embeddings"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_request_meter_fact_with_capability_not_supported_by_provider_model(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-request-meter-provider-capability");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("models").join("default.json"),
        &serde_json::json!([
            {
                "external_name": "gpt-4.1",
                "provider_id": "provider-openrouter-main",
                "capabilities": ["embeddings"],
                "streaming": true,
                "context_window": 128000
            },
            {
                "external_name": "deepseek-chat",
                "provider_id": "provider-openrouter-main",
                "capabilities": ["responses", "chat_completions"],
                "streaming": true,
                "context_window": 65536
            },
            {
                "external_name": "qwen-plus-latest",
                "provider_id": "provider-siliconflow-main",
                "capabilities": ["responses", "chat_completions"],
                "streaming": true,
                "context_window": 131072
            },
            {
                "external_name": "llama3.2:latest",
                "provider_id": "provider-ollama-local",
                "capabilities": ["responses", "chat_completions"],
                "streaming": true,
                "context_window": 8192
            }
        ]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject request metering facts whose declared capability is not supported by the provider model"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("6001"), "{error}");
    assert!(error.contains("gpt-4.1"), "{error}");
    assert!(error.contains("responses"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_request_meter_fact_without_active_model_price_coverage(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-request-meter-price-coverage");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("model-prices").join("default.json"),
        &serde_json::json!([
            {
                "channel_id": "siliconflow",
                "model_id": "qwen-plus-latest",
                "proxy_provider_id": "provider-siliconflow-main",
                "currency_code": "USD",
                "price_unit": "per_1m_tokens",
                "input_price": 0.4,
                "output_price": 1.2,
                "cache_read_price": 0.0,
                "cache_write_price": 0.0,
                "request_price": 0.0,
                "price_source_kind": "proxy",
                "is_active": true
            }
        ]),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject request metering facts without active model price coverage"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("6001"), "{error}");
    assert!(error.contains("provider-openrouter-main"), "{error}");
    assert!(error.contains("gpt-4.1"), "{error}");
    assert!(error.contains("price"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_billing_event_without_active_model_price_coverage(
) {
    let bootstrap_root = temp_bootstrap_root("profile-pack-invalid-billing-price-coverage");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("request-metering").join("default.json"),
        &serde_json::json!({
            "facts": [
                {
                    "request_id": 6002u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "user_id": 9001u64,
                    "account_id": 7001u64,
                    "api_key_id": 10001u64,
                    "api_key_hash": "a19d2bf76318aa7f619d684271469bb383faf1cb5bd4c680088465cde9d0003b",
                    "auth_type": "api_key",
                    "jwt_subject": null,
                    "platform": "portal",
                    "owner": "user_local_demo",
                    "request_trace_id": "trace-local-demo-billing-shadow-6002",
                    "gateway_request_ref": null,
                    "upstream_request_ref": null,
                    "protocol_family": "openai",
                    "capability_code": "responses",
                    "channel_code": "siliconflow",
                    "model_code": "qwen-plus-latest",
                    "provider_code": "provider-siliconflow-main",
                    "request_status": "pending",
                    "usage_capture_status": "pending",
                    "cost_pricing_plan_id": 9101u64,
                    "retail_pricing_plan_id": 9101u64,
                    "estimated_credit_hold": 0.0,
                    "actual_credit_charge": null,
                    "actual_provider_cost": null,
                    "started_at_ms": 1710005510000u64,
                    "finished_at_ms": null,
                    "created_at_ms": 1710005510000u64,
                    "updated_at_ms": 1710005510100u64
                }
            ],
            "metrics": []
        }),
    );
    write_json(
        &bootstrap_root.join("request-settlements").join("default.json"),
        &serde_json::json!([]),
    );
    write_json(
        &bootstrap_root.join("observability").join("default.json"),
        &serde_json::json!({
            "compiled_routing_snapshots": [],
            "routing_decision_logs": [],
            "provider_health_snapshots": []
        }),
    );
    write_json(
        &bootstrap_root.join("billing").join("default.json"),
        &billing_fixture_with_local_metering_support(vec![]),
    );
    write_json(
        &bootstrap_root.join("model-prices").join("default.json"),
        &serde_json::json!([
            {
                "channel_id": "siliconflow",
                "model_id": "qwen-plus-latest",
                "proxy_provider_id": "provider-siliconflow-main",
                "currency_code": "USD",
                "price_unit": "per_1m_tokens",
                "input_price": 0.4,
                "output_price": 1.2,
                "cache_read_price": 0.0,
                "cache_write_price": 0.0,
                "request_price": 0.0,
                "price_source_kind": "proxy",
                "is_active": true
            }
        ]),
    );
    write_single_account_hold_fixture(
        &bootstrap_root,
        6002u64,
        "failed",
        0.0,
        0.0,
        0.0,
        1710005510000u64,
        1710005510100u64,
    );
    let account_ledger_path = bootstrap_root.join("account-ledger").join("default.json");
    let mut account_ledger =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&account_ledger_path).unwrap())
            .unwrap();
    account_ledger["entries"][2]["request_id"] = serde_json::json!(6002u64);
    write_json(&account_ledger_path, &account_ledger);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject billing events without active model price coverage"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("billing-local-demo-metering-support"), "{error}");
    assert!(error.contains("provider-openrouter-main"), "{error}");
    assert!(error.contains("gpt-4.1"), "{error}");
    assert!(error.contains("price"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_async_job_without_executable_provider_account(
) {
    let bootstrap_root = temp_bootstrap_root("profile-pack-invalid-async-job-provider-account");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("routing").join("default.json"),
        &serde_json::json!({
            "profiles": [
                {
                    "profile_id": "profile-global-balanced",
                    "tenant_id": "tenant_local_demo",
                    "project_id": "project_local_demo",
                    "name": "Global Balanced",
                    "slug": "global-balanced",
                    "description": "Balanced multi-provider routing",
                    "active": true,
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-openrouter-main",
                        "provider-ollama-local"
                    ],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "preferred_region": "global",
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ],
            "policies": [
                {
                    "policy_id": "policy-default-responses",
                    "capability": "responses",
                    "model_pattern": "*",
                    "enabled": true,
                    "priority": 100,
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-openrouter-main",
                        "provider-ollama-local"
                    ],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "execution_failover_enabled": true,
                    "upstream_retry_max_attempts": 3,
                    "upstream_retry_base_delay_ms": 250,
                    "upstream_retry_max_delay_ms": 2000
                }
            ],
            "project_preferences": [
                {
                    "project_id": "project_local_demo",
                    "preset_id": "profile-global-balanced",
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-openrouter-main",
                        "provider-ollama-local"
                    ],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "preferred_region": "global",
                    "updated_at_ms": 1710000000000u64
                }
            ]
        }),
    );
    write_json(
        &bootstrap_root
            .join("provider-accounts")
            .join("default.json"),
        &serde_json::json!([
            {
                "provider_account_id": "acct-openrouter-default",
                "provider_id": "provider-openrouter-main",
                "display_name": "OpenRouter Default",
                "account_kind": "api_key",
                "owner_scope": "platform",
                "owner_tenant_id": null,
                "execution_instance_id": "provider-openrouter-main",
                "base_url_override": "https://openrouter.ai/api/v1",
                "region": "global",
                "priority": 100,
                "weight": 10,
                "enabled": true,
                "routing_tags": ["default", "global"],
                "health_score_hint": null,
                "latency_ms_hint": null,
                "cost_hint": null,
                "success_rate_hint": null,
                "throughput_hint": null,
                "max_concurrency": null,
                "daily_budget": null,
                "notes": "bootstrap default account"
            },
            {
                "provider_account_id": "acct-ollama-local-default",
                "provider_id": "provider-ollama-local",
                "display_name": "Ollama Local Default",
                "account_kind": "runtime_instance",
                "owner_scope": "platform",
                "owner_tenant_id": null,
                "execution_instance_id": "provider-ollama-local",
                "base_url_override": "http://127.0.0.1:11434",
                "region": "local",
                "priority": 90,
                "weight": 5,
                "enabled": true,
                "routing_tags": ["default", "local"],
                "health_score_hint": null,
                "latency_ms_hint": 35,
                "cost_hint": 0.0,
                "success_rate_hint": null,
                "throughput_hint": null,
                "max_concurrency": null,
                "daily_budget": null,
                "notes": "bootstrap local account"
            }
        ]),
    );
    write_json(
        &bootstrap_root.join("observability").join("default.json"),
        &serde_json::json!({
            "compiled_routing_snapshots": [],
            "routing_decision_logs": [],
            "provider_health_snapshots": []
        }),
    );
    write_json(
        &bootstrap_root.join("billing").join("default.json"),
        &billing_fixture_with_local_metering_support(vec![]),
    );
    write_json(
        &bootstrap_root.join("model-prices").join("default.json"),
        &model_prices_without_siliconflow_fixture(),
    );
    write_json(
        &bootstrap_root.join("jobs").join("default.json"),
        &serde_json::json!({
            "jobs": [
                {
                    "job_id": "job-invalid-siliconflow-without-account",
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "user_id": 9001u64,
                    "account_id": 7001u64,
                    "request_id": null,
                    "provider_id": "provider-siliconflow-main",
                    "model_code": "qwen-plus-latest",
                    "capability_code": "responses",
                    "modality": "text",
                    "operation_kind": "draft_generate",
                    "status": "queued",
                    "external_job_id": "sf-job-without-account",
                    "idempotency_key": "job:invalid:siliconflow-without-account",
                    "callback_url": "https://portal.sdkwork.local/api/jobs/callbacks/invalid",
                    "input_summary": "Should fail because provider has no executable account",
                    "progress_percent": 0u64,
                    "error_code": null,
                    "error_message": null,
                    "created_at_ms": 1710005600000u64,
                    "updated_at_ms": 1710005600100u64,
                    "started_at_ms": null,
                    "completed_at_ms": null
                }
            ],
            "attempts": [],
            "assets": [],
            "callbacks": []
        }),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject async jobs whose provider has no executable provider account"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("job-invalid-siliconflow-without-account"), "{error}");
    assert!(error.contains("provider-siliconflow-main"), "{error}");
    assert!(error.contains("provider account"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_async_job_with_missing_account() {
    let bootstrap_root = temp_bootstrap_root("profile-pack-invalid-async-job-account");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("jobs").join("default.json"),
        &serde_json::json!({
            "jobs": [
                {
                    "job_id": "job-invalid-missing-account",
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "user_id": 3001u64,
                    "account_id": 4999u64,
                    "request_id": null,
                    "provider_id": "provider-openrouter-main",
                    "model_code": "deepseek-chat",
                    "capability_code": "responses",
                    "modality": "text",
                    "operation_kind": "draft_generate",
                    "status": "running",
                    "external_job_id": "or-job-missing-account",
                    "idempotency_key": "job:invalid:missing-account",
                    "callback_url": "https://portal.sdkwork.local/api/jobs/callbacks/invalid-account",
                    "input_summary": "Should fail because async job account is missing",
                    "progress_percent": 0u64,
                    "error_code": null,
                    "error_message": null,
                    "created_at_ms": 1710005600000u64,
                    "updated_at_ms": 1710005600100u64,
                    "started_at_ms": null,
                    "completed_at_ms": null
                }
            ],
            "attempts": [],
            "assets": [],
            "callbacks": []
        }),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!("bootstrap should reject async jobs that reference missing accounts"),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("job-invalid-missing-account"), "{error}");
    assert!(error.contains("4999"), "{error}");
    assert!(error.contains("async_jobs.account_id"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_async_job_attempt_with_mismatched_external_job_id(
) {
    let bootstrap_root = temp_bootstrap_root("profile-pack-invalid-async-job-attempt-external-id");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("jobs").join("default.json"),
        &serde_json::json!({
            "jobs": [
                {
                    "job_id": "job-invalid-attempt-external-id",
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "user_id": 9001u64,
                    "account_id": 7001u64,
                    "request_id": null,
                    "provider_id": "provider-openrouter-main",
                    "model_code": "deepseek-chat",
                    "capability_code": "responses",
                    "modality": "text",
                    "operation_kind": "draft_generate",
                    "status": "succeeded",
                    "external_job_id": "or-job-parent",
                    "idempotency_key": "job:invalid:attempt-external-id",
                    "callback_url": "https://portal.sdkwork.local/api/jobs/callbacks/invalid-attempt",
                    "input_summary": "Should fail because attempt external job id drifts from parent job",
                    "progress_percent": 100u64,
                    "error_code": null,
                    "error_message": null,
                    "created_at_ms": 1710005600000u64,
                    "updated_at_ms": 1710005600500u64,
                    "started_at_ms": 1710005600100u64,
                    "completed_at_ms": 1710005600500u64
                }
            ],
            "attempts": [
                {
                    "attempt_id": 9911u64,
                    "job_id": "job-invalid-attempt-external-id",
                    "attempt_number": 1u64,
                    "status": "succeeded",
                    "runtime_kind": "openrouter",
                    "endpoint": "https://openrouter.ai/api/v1/responses",
                    "external_job_id": "or-job-other",
                    "claimed_at_ms": 1710005600120u64,
                    "finished_at_ms": 1710005600480u64,
                    "error_message": null,
                    "created_at_ms": 1710005600000u64,
                    "updated_at_ms": 1710005600480u64
                }
            ],
            "assets": [],
            "callbacks": []
        }),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject async job attempts whose external job id drifts from the parent job"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("9911"), "{error}");
    assert!(error.contains("external job id"), "{error}");
    assert!(error.contains("job-invalid-attempt-external-id"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_async_job_callback_with_mismatched_payload_job_id(
) {
    let bootstrap_root = temp_bootstrap_root("profile-pack-invalid-async-job-callback-payload");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("jobs").join("default.json"),
        &serde_json::json!({
            "jobs": [
                {
                    "job_id": "job-invalid-callback-payload",
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "user_id": 9001u64,
                    "account_id": 7001u64,
                    "request_id": null,
                    "provider_id": "provider-openrouter-main",
                    "model_code": "deepseek-chat",
                    "capability_code": "responses",
                    "modality": "text",
                    "operation_kind": "draft_generate",
                    "status": "succeeded",
                    "external_job_id": "or-job-callback-parent",
                    "idempotency_key": "job:invalid:callback-payload",
                    "callback_url": "https://portal.sdkwork.local/api/jobs/callbacks/invalid-payload",
                    "input_summary": "Should fail because callback payload points at another job id",
                    "progress_percent": 100u64,
                    "error_code": null,
                    "error_message": null,
                    "created_at_ms": 1710005600000u64,
                    "updated_at_ms": 1710005600500u64,
                    "started_at_ms": 1710005600100u64,
                    "completed_at_ms": 1710005600500u64
                }
            ],
            "attempts": [],
            "assets": [],
            "callbacks": [
                {
                    "callback_id": 9912u64,
                    "job_id": "job-invalid-callback-payload",
                    "event_type": "job.completed",
                    "dedupe_key": "openrouter:or-job-callback-parent:completed",
                    "payload_json": "{\"job_id\":\"job-someone-else\",\"status\":\"succeeded\"}",
                    "status": "processed",
                    "received_at_ms": 1710005600510u64,
                    "processed_at_ms": 1710005600530u64
                }
            ]
        }),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject async job callbacks whose payload job id drifts from the parent job"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("9912"), "{error}");
    assert!(error.contains("payload"), "{error}");
    assert!(error.contains("job-invalid-callback-payload"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_async_job_attempt_created_before_parent_job(
) {
    let bootstrap_root = temp_bootstrap_root("profile-pack-invalid-async-job-attempt-timeline");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("jobs").join("default.json"),
        &serde_json::json!({
            "jobs": [
                {
                    "job_id": "job-invalid-attempt-timeline",
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "user_id": 9001u64,
                    "account_id": 7001u64,
                    "request_id": null,
                    "provider_id": "provider-openrouter-main",
                    "model_code": "deepseek-chat",
                    "capability_code": "responses",
                    "modality": "text",
                    "operation_kind": "draft_generate",
                    "status": "queued",
                    "external_job_id": "or-job-timeline",
                    "idempotency_key": "job:invalid:attempt-timeline",
                    "callback_url": "https://portal.sdkwork.local/api/jobs/callbacks/invalid-timeline",
                    "input_summary": "Should fail because attempt appears before parent job creation",
                    "progress_percent": 0u64,
                    "error_code": null,
                    "error_message": null,
                    "created_at_ms": 1710005600000u64,
                    "updated_at_ms": 1710005600100u64,
                    "started_at_ms": 1710005600020u64,
                    "completed_at_ms": null
                }
            ],
            "attempts": [
                {
                    "attempt_id": 9913u64,
                    "job_id": "job-invalid-attempt-timeline",
                    "attempt_number": 1u64,
                    "status": "running",
                    "runtime_kind": "openrouter",
                    "endpoint": "https://openrouter.ai/api/v1/responses",
                    "external_job_id": "or-job-timeline",
                    "claimed_at_ms": 1710005600030u64,
                    "finished_at_ms": null,
                    "error_message": null,
                    "created_at_ms": 1710005599990u64,
                    "updated_at_ms": 1710005600030u64
                }
            ],
            "assets": [],
            "callbacks": []
        }),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject async job attempts created before the parent job exists"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("9913"), "{error}");
    assert!(error.contains("created_at_ms"), "{error}");
    assert!(error.contains("parent job"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_async_job_with_succeeded_status_missing_started_at(
) {
    let bootstrap_root = temp_bootstrap_root("profile-pack-invalid-async-job-succeeded-started");
    write_bootstrap_profile_pack(&bootstrap_root);

    let jobs_path = bootstrap_root.join("jobs").join("default.json");
    let mut jobs =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&jobs_path).unwrap())
            .unwrap();
    jobs["jobs"]
        .as_array_mut()
        .unwrap()
        .first_mut()
        .unwrap()["started_at_ms"] = serde_json::Value::Null;
    write_json(&jobs_path, &jobs);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!("bootstrap should reject succeeded async jobs without started_at_ms"),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("job-local-demo-growth-brief"), "{error}");
    assert!(error.contains("succeeded"), "{error}");
    assert!(error.contains("started_at_ms"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_async_job_with_succeeded_status_missing_completed_at(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-async-job-succeeded-completed");
    write_bootstrap_profile_pack(&bootstrap_root);

    let jobs_path = bootstrap_root.join("jobs").join("default.json");
    let mut jobs =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&jobs_path).unwrap())
            .unwrap();
    jobs["jobs"]
        .as_array_mut()
        .unwrap()
        .first_mut()
        .unwrap()["completed_at_ms"] = serde_json::Value::Null;
    write_json(&jobs_path, &jobs);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!("bootstrap should reject succeeded async jobs without completed_at_ms"),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("job-local-demo-growth-brief"), "{error}");
    assert!(error.contains("succeeded"), "{error}");
    assert!(error.contains("completed_at_ms"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_async_job_attempt_with_succeeded_status_missing_claimed_at(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-async-job-attempt-succeeded-claimed");
    write_bootstrap_profile_pack(&bootstrap_root);

    let jobs_path = bootstrap_root.join("jobs").join("default.json");
    let mut jobs =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&jobs_path).unwrap())
            .unwrap();
    jobs["attempts"]
        .as_array_mut()
        .unwrap()
        .first_mut()
        .unwrap()["claimed_at_ms"] = serde_json::Value::Null;
    write_json(&jobs_path, &jobs);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!("bootstrap should reject succeeded async job attempts without claimed_at_ms"),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("8801"), "{error}");
    assert!(error.contains("succeeded"), "{error}");
    assert!(error.contains("claimed_at_ms"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_async_job_attempt_with_succeeded_status_missing_finished_at(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-async-job-attempt-succeeded-finished");
    write_bootstrap_profile_pack(&bootstrap_root);

    let jobs_path = bootstrap_root.join("jobs").join("default.json");
    let mut jobs =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&jobs_path).unwrap())
            .unwrap();
    jobs["attempts"]
        .as_array_mut()
        .unwrap()
        .first_mut()
        .unwrap()["finished_at_ms"] = serde_json::Value::Null;
    write_json(&jobs_path, &jobs);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!("bootstrap should reject succeeded async job attempts without finished_at_ms"),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("8801"), "{error}");
    assert!(error.contains("succeeded"), "{error}");
    assert!(error.contains("finished_at_ms"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_async_job_callback_with_processed_status_missing_processed_at(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-async-job-callback-processed-ts");
    write_bootstrap_profile_pack(&bootstrap_root);

    let jobs_path = bootstrap_root.join("jobs").join("default.json");
    let mut jobs =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&jobs_path).unwrap())
            .unwrap();
    jobs["callbacks"]
        .as_array_mut()
        .unwrap()
        .first_mut()
        .unwrap()["processed_at_ms"] = serde_json::Value::Null;
    write_json(&jobs_path, &jobs);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!("bootstrap should reject processed async job callbacks without processed_at_ms"),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("9901"), "{error}");
    assert!(error.contains("processed"), "{error}");
    assert!(error.contains("processed_at_ms"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_async_job_attempt_claimed_before_parent_job_started(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-async-job-attempt-before-job-started");
    write_bootstrap_profile_pack(&bootstrap_root);

    let jobs_path = bootstrap_root.join("jobs").join("default.json");
    let mut jobs =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&jobs_path).unwrap())
            .unwrap();
    jobs["attempts"]
        .as_array_mut()
        .unwrap()
        .first_mut()
        .unwrap()["claimed_at_ms"] = serde_json::json!(1710005600050u64);
    write_json(&jobs_path, &jobs);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject async job attempts claimed before the parent job started"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("8801"), "{error}");
    assert!(error.contains("claimed_at_ms"), "{error}");
    assert!(error.contains("started_at_ms"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_async_job_attempt_finished_after_parent_job_completed(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-async-job-attempt-after-job-completed");
    write_bootstrap_profile_pack(&bootstrap_root);

    let jobs_path = bootstrap_root.join("jobs").join("default.json");
    let mut jobs =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&jobs_path).unwrap())
            .unwrap();
    jobs["attempts"]
        .as_array_mut()
        .unwrap()
        .first_mut()
        .unwrap()["finished_at_ms"] = serde_json::json!(1710005600510u64);
    jobs["attempts"]
        .as_array_mut()
        .unwrap()
        .first_mut()
        .unwrap()["updated_at_ms"] = serde_json::json!(1710005600510u64);
    write_json(&jobs_path, &jobs);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject async job attempts finished after the parent job completed"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("8801"), "{error}");
    assert!(error.contains("finished_at_ms"), "{error}");
    assert!(error.contains("completed_at_ms"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_async_job_asset_created_before_parent_job_created(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-async-job-asset-before-job-created");
    write_bootstrap_profile_pack(&bootstrap_root);

    let jobs_path = bootstrap_root.join("jobs").join("default.json");
    let mut jobs =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&jobs_path).unwrap())
            .unwrap();
    jobs["assets"]
        .as_array_mut()
        .unwrap()
        .first_mut()
        .unwrap()["created_at_ms"] = serde_json::json!(1710005599990u64);
    write_json(&jobs_path, &jobs);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject async job assets created before the parent job exists"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("asset-local-demo-growth-brief-json"), "{error}");
    assert!(error.contains("created_at_ms"), "{error}");
    assert!(error.contains("job-local-demo-growth-brief"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_async_job_asset_storage_key_outside_parent_tenant_scope(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-async-job-asset-tenant-storage-scope");
    write_bootstrap_profile_pack(&bootstrap_root);

    let jobs_path = bootstrap_root.join("jobs").join("default.json");
    let mut jobs =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&jobs_path).unwrap())
            .unwrap();
    jobs["assets"]
        .as_array_mut()
        .unwrap()
        .first_mut()
        .unwrap()["storage_key"] =
        serde_json::json!("tenant-9999/jobs/job-local-demo-growth-brief/output.json");
    write_json(&jobs_path, &jobs);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject async job assets whose storage key drifts outside the parent tenant scope"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("asset-local-demo-growth-brief-json"), "{error}");
    assert!(error.contains("storage_key"), "{error}");
    assert!(error.contains("tenant-1001"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_async_job_callback_received_before_parent_job_created(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-async-job-callback-before-job-created");
    write_bootstrap_profile_pack(&bootstrap_root);

    let jobs_path = bootstrap_root.join("jobs").join("default.json");
    let mut jobs =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&jobs_path).unwrap())
            .unwrap();
    jobs["callbacks"]
        .as_array_mut()
        .unwrap()
        .first_mut()
        .unwrap()["received_at_ms"] = serde_json::json!(1710005599990u64);
    jobs["callbacks"]
        .as_array_mut()
        .unwrap()
        .first_mut()
        .unwrap()["processed_at_ms"] = serde_json::json!(1710005600000u64);
    write_json(&jobs_path, &jobs);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject async job callbacks received before the parent job exists"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("9901"), "{error}");
    assert!(error.contains("received_at_ms"), "{error}");
    assert!(error.contains("job-local-demo-growth-brief"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_async_job_completed_callback_received_before_parent_job_completed(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-async-job-callback-before-job-completed");
    write_bootstrap_profile_pack(&bootstrap_root);

    let jobs_path = bootstrap_root.join("jobs").join("default.json");
    let mut jobs =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&jobs_path).unwrap())
            .unwrap();
    jobs["callbacks"]
        .as_array_mut()
        .unwrap()
        .first_mut()
        .unwrap()["received_at_ms"] = serde_json::json!(1710005600490u64);
    jobs["callbacks"]
        .as_array_mut()
        .unwrap()
        .first_mut()
        .unwrap()["processed_at_ms"] = serde_json::json!(1710005600530u64);
    write_json(&jobs_path, &jobs);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject completed async job callbacks received before the parent job completed"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("9901"), "{error}");
    assert!(error.contains("job.completed"), "{error}");
    assert!(error.contains("completed_at_ms"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_async_job_callback_with_mismatched_payload_status(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-async-job-callback-payload-status");
    write_bootstrap_profile_pack(&bootstrap_root);

    let jobs_path = bootstrap_root.join("jobs").join("default.json");
    let mut jobs =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&jobs_path).unwrap())
            .unwrap();
    jobs["callbacks"]
        .as_array_mut()
        .unwrap()
        .first_mut()
        .unwrap()["payload_json"] =
        serde_json::json!("{\"job_id\":\"job-local-demo-growth-brief\",\"status\":\"failed\"}");
    write_json(&jobs_path, &jobs);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject async job callbacks whose payload status drifts from the parent job"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("9901"), "{error}");
    assert!(error.contains("status"), "{error}");
    assert!(error.contains("job-local-demo-growth-brief"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_async_job_callback_with_mismatched_payload_provider(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-async-job-callback-payload-provider");
    write_bootstrap_profile_pack(&bootstrap_root);

    let jobs_path = bootstrap_root.join("jobs").join("default.json");
    let mut jobs =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&jobs_path).unwrap())
            .unwrap();
    jobs["callbacks"]
        .as_array_mut()
        .unwrap()
        .first_mut()
        .unwrap()["payload_json"] = serde_json::json!(
        "{\"job_id\":\"job-local-demo-growth-brief\",\"status\":\"succeeded\",\"provider\":\"gemini\"}"
    );
    write_json(&jobs_path, &jobs);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject async job callbacks whose payload provider drifts from the parent job"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("9901"), "{error}");
    assert!(error.contains("provider"), "{error}");
    assert!(error.contains("openrouter"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_async_job_attempt_with_runtime_kind_mismatched_parent_provider(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-async-job-attempt-runtime-kind-provider");
    write_bootstrap_profile_pack(&bootstrap_root);

    let jobs_path = bootstrap_root.join("jobs").join("default.json");
    let mut jobs =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&jobs_path).unwrap())
            .unwrap();
    jobs["attempts"]
        .as_array_mut()
        .unwrap()
        .first_mut()
        .unwrap()["runtime_kind"] = serde_json::json!("gemini");
    write_json(&jobs_path, &jobs);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject async job attempts whose runtime_kind drifts from the parent provider"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("8801"), "{error}");
    assert!(error.contains("runtime_kind"), "{error}");
    assert!(error.contains("provider-openrouter-main"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_async_job_asset_with_mismatched_mime_type_for_asset_kind(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-async-job-asset-mime-kind");
    write_bootstrap_profile_pack(&bootstrap_root);

    let jobs_path = bootstrap_root.join("jobs").join("default.json");
    let mut jobs =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&jobs_path).unwrap())
            .unwrap();
    jobs["assets"]
        .as_array_mut()
        .unwrap()
        .first_mut()
        .unwrap()["mime_type"] = serde_json::json!("text/markdown");
    write_json(&jobs_path, &jobs);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject async job assets whose mime_type does not match the declared asset_kind"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("asset-local-demo-growth-brief-json"), "{error}");
    assert!(error.contains("mime_type"), "{error}");
    assert!(error.contains("text/markdown"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_async_job_asset_with_storage_extension_mismatched_asset_kind(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-async-job-asset-storage-extension");
    write_bootstrap_profile_pack(&bootstrap_root);

    let jobs_path = bootstrap_root.join("jobs").join("default.json");
    let mut jobs =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&jobs_path).unwrap())
            .unwrap();
    jobs["assets"]
        .as_array_mut()
        .unwrap()
        .first_mut()
        .unwrap()["storage_key"] =
        serde_json::json!("tenant-1001/jobs/job-local-demo-growth-brief/output.md");
    write_json(&jobs_path, &jobs);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject async job assets whose storage extension does not match asset_kind"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("asset-local-demo-growth-brief-json"), "{error}");
    assert!(error.contains("storage_key"), "{error}");
    assert!(error.contains("output.md"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_async_job_asset_with_download_leaf_mismatched_storage_key(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-async-job-asset-download-leaf");
    write_bootstrap_profile_pack(&bootstrap_root);

    let jobs_path = bootstrap_root.join("jobs").join("default.json");
    let mut jobs =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&jobs_path).unwrap())
            .unwrap();
    jobs["assets"]
        .as_array_mut()
        .unwrap()
        .first_mut()
        .unwrap()["download_url"] =
        serde_json::json!("https://cdn.sdkwork.local/jobs/job-local-demo-growth-brief/other.md");
    write_json(&jobs_path, &jobs);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject async job assets whose download url leaf drifts from storage_key"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("asset-local-demo-growth-brief-json"), "{error}");
    assert!(error.contains("download_url"), "{error}");
    assert!(error.contains("job-local-demo-growth-brief"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_async_job_completed_callback_with_non_completed_event_type(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-async-job-callback-event-type");
    write_bootstrap_profile_pack(&bootstrap_root);

    let jobs_path = bootstrap_root.join("jobs").join("default.json");
    let mut jobs =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&jobs_path).unwrap())
            .unwrap();
    jobs["callbacks"]
        .as_array_mut()
        .unwrap()
        .first_mut()
        .unwrap()["event_type"] = serde_json::json!("job.progress");
    write_json(&jobs_path, &jobs);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject completed async job callbacks whose event_type is not job.completed"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("9901"), "{error}");
    assert!(error.contains("event_type"), "{error}");
    assert!(error.contains("job.completed"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_async_job_completed_callback_with_non_completed_dedupe_suffix(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-async-job-callback-dedupe-suffix");
    write_bootstrap_profile_pack(&bootstrap_root);

    let jobs_path = bootstrap_root.join("jobs").join("default.json");
    let mut jobs =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&jobs_path).unwrap())
            .unwrap();
    jobs["callbacks"]
        .as_array_mut()
        .unwrap()
        .first_mut()
        .unwrap()["dedupe_key"] =
        serde_json::json!("openrouter:or-job-local-demo-growth-brief:progress");
    write_json(&jobs_path, &jobs);

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject completed async job callbacks whose dedupe suffix is not completed"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("9901"), "{error}");
    assert!(error.contains("dedupe_key"), "{error}");
    assert!(error.contains("completed"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_async_job_with_capability_not_supported_by_model(
) {
    let bootstrap_root =
        temp_bootstrap_root("profile-pack-invalid-async-job-model-capability");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("jobs").join("default.json"),
        &serde_json::json!({
            "jobs": [
                {
                    "job_id": "job-invalid-model-capability",
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "user_id": 9001u64,
                    "account_id": 7001u64,
                    "request_id": null,
                    "provider_id": "provider-openrouter-main",
                    "model_code": "deepseek-chat",
                    "capability_code": "embeddings",
                    "modality": "text",
                    "operation_kind": "draft_generate",
                    "status": "queued",
                    "external_job_id": "or-job-invalid-model-capability",
                    "idempotency_key": "job:invalid:model-capability",
                    "callback_url": "https://portal.sdkwork.local/api/jobs/callbacks/invalid-model-capability",
                    "input_summary": "Should fail because the selected model does not support the declared capability",
                    "progress_percent": 0u64,
                    "error_code": null,
                    "error_message": null,
                    "created_at_ms": 1710005600000u64,
                    "updated_at_ms": 1710005600100u64,
                    "started_at_ms": null,
                    "completed_at_ms": null
                }
            ],
            "attempts": [],
            "assets": [],
            "callbacks": []
        }),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!(
            "bootstrap should reject async jobs whose declared capability is not supported by the provider model"
        ),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("job-invalid-model-capability"), "{error}");
    assert!(error.contains("deepseek-chat"), "{error}");
    assert!(error.contains("embeddings"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_job_attempt_with_missing_job() {
    let bootstrap_root = temp_bootstrap_root("profile-pack-invalid-jobs");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("profiles").join("dev.json"),
        &serde_json::json!({
            "profile_id": "dev",
            "description": "development bootstrap pack",
            "channels": ["channels/default.json"],
            "providers": ["providers/default.json"],
            "official_provider_configs": ["official-provider-configs/default.json"],
            "provider_accounts": ["provider-accounts/default.json"],
            "models": ["models/default.json"],
            "channel_models": ["channel-models/default.json"],
            "model_prices": ["model-prices/default.json"],
            "tenants": ["tenants/default.json"],
            "projects": ["projects/default.json"],
            "identities": ["identities/default.json"],
            "extensions": ["extensions/default.json"],
            "routing": ["routing/default.json"],
            "api_key_groups": ["api-key-groups/default.json"],
            "observability": ["observability/default.json"],
            "quota_policies": ["quota-policies/default.json"],
            "pricing": ["pricing/default.json"],
            "payment_methods": ["payment-methods/default.json"],
            "marketing": ["marketing/default.json"],
            "commerce": ["commerce/default.json"],
            "billing": ["billing/default.json"],
            "jobs": ["jobs/invalid-missing-job.json"]
        }),
    );
    write_json(
        &bootstrap_root.join("jobs").join("invalid-missing-job.json"),
        &serde_json::json!({
            "jobs": [],
            "attempts": [
                {
                    "attempt_id": 9991u64,
                    "job_id": "job-missing",
                    "attempt_number": 1u64,
                    "status": "failed",
                    "runtime_kind": "openrouter",
                    "endpoint": "https://openrouter.ai/api/v1/responses",
                    "external_job_id": "or-job-missing",
                    "claimed_at_ms": 1710009000100u64,
                    "finished_at_ms": 1710009000200u64,
                    "error_message": "should fail because the parent job is missing",
                    "created_at_ms": 1710009000000u64,
                    "updated_at_ms": 1710009000200u64
                }
            ],
            "assets": [],
            "callbacks": []
        }),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!("bootstrap should reject job attempts that reference unknown jobs"),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("job-missing"), "{error}");
}

#[tokio::test]
async fn build_admin_store_from_config_rejects_bootstrap_commerce_refund_with_unknown_payment_attempt(
) {
    let bootstrap_root = temp_bootstrap_root("profile-pack-invalid-commerce-refund");
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("profiles").join("dev.json"),
        &serde_json::json!({
            "profile_id": "dev",
            "description": "development bootstrap pack",
            "channels": ["channels/default.json"],
            "providers": ["providers/default.json"],
            "official_provider_configs": ["official-provider-configs/default.json"],
            "provider_accounts": ["provider-accounts/default.json"],
            "models": ["models/default.json"],
            "channel_models": ["channel-models/default.json"],
            "model_prices": ["model-prices/default.json"],
            "tenants": ["tenants/default.json"],
            "projects": ["projects/default.json"],
            "identities": ["identities/default.json"],
            "extensions": ["extensions/default.json"],
            "routing": ["routing/default.json"],
            "api_key_groups": ["api-key-groups/default.json"],
            "observability": ["observability/default.json"],
            "quota_policies": ["quota-policies/default.json"],
            "pricing": ["pricing/default.json"],
            "payment_methods": ["payment-methods/default.json"],
            "marketing": ["marketing/default.json"],
            "commerce": ["commerce/invalid-missing-payment-attempt.json"],
            "billing": ["billing/default.json"],
            "jobs": ["jobs/default.json", "jobs/dev.json"]
        }),
    );
    write_json(
        &bootstrap_root
            .join("commerce")
            .join("invalid-missing-payment-attempt.json"),
        &serde_json::json!({
            "orders": [
                {
                    "order_id": "order-local-demo-growth-2026",
                    "project_id": "project_local_demo",
                    "user_id": "user_local_demo",
                    "target_kind": "subscription_plan",
                    "target_id": "plan-local-growth",
                    "target_name": "Local Demo Growth",
                    "list_price_cents": 19900u64,
                    "payable_price_cents": 9900u64,
                    "list_price_label": "$199 / month",
                    "payable_price_label": "$99 / month",
                    "granted_units": 3000000u64,
                    "bonus_units": 250000u64,
                    "currency_code": "USD",
                    "pricing_plan_id": "global-default-commercial",
                    "pricing_plan_version": 1u64,
                    "pricing_snapshot_json": "{\"plan_code\":\"global-default-commercial\",\"billing_interval\":\"monthly\"}",
                    "applied_coupon_code": "LAUNCH100",
                    "coupon_reservation_id": null,
                    "coupon_redemption_id": null,
                    "marketing_campaign_id": "campaign-launch-q2",
                    "subsidy_amount_minor": 10000u64,
                    "payment_method_id": "payment-stripe-hosted",
                    "latest_payment_attempt_id": null,
                    "status": "fulfilled",
                    "settlement_status": "settled",
                    "source": "bootstrap",
                    "refundable_amount_minor": 9900u64,
                    "refunded_amount_minor": 1000u64,
                    "created_at_ms": 1710005000000u64,
                    "updated_at_ms": 1710005000500u64
                }
            ],
            "payment_events": [],
            "payment_attempts": [],
            "webhook_inbox_records": [],
            "refunds": [
                {
                    "refund_id": "refund-invalid-missing-attempt",
                    "order_id": "order-local-demo-growth-2026",
                    "payment_attempt_id": "attempt-missing",
                    "payment_method_id": "payment-stripe-hosted",
                    "provider": "stripe",
                    "provider_refund_id": null,
                    "idempotency_key": "refund:invalid:missing-attempt",
                    "reason": "should fail because payment attempt is missing",
                    "status": "requested",
                    "amount_minor": 1000u64,
                    "currency_code": "USD",
                    "request_payload_json": "{\"amount_minor\":1000}",
                    "response_payload_json": "{}",
                    "created_at_ms": 1710008000000u64,
                    "updated_at_ms": 1710008000000u64,
                    "completed_at_ms": null
                }
            ],
            "reconciliation_runs": [],
            "reconciliation_items": []
        }),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    let error = match build_admin_store_from_config(&config).await {
        Ok(_) => panic!("bootstrap should reject refunds that reference unknown payment attempts"),
        Err(error) => error.to_string(),
    };

    assert!(error.contains("attempt-missing"), "{error}");
}

#[test]
fn restart_required_changed_fields_include_cache_backend_for_gateway_runtime() {
    let current = StandaloneConfig::default();
    let next = StandaloneConfig {
        cache_backend: CacheBackendKind::Redis,
        cache_url: Some("redis://127.0.0.1:6379/8".to_owned()),
        ..current.clone()
    };

    let changed = restart_required_changed_fields(StandaloneServiceKind::Gateway, &current, &next);

    assert!(changed.contains(&"cache_backend"));
    assert!(changed.contains(&"cache_url"));
}

#[test]
fn merge_applied_service_config_keeps_gateway_cache_backend_on_restart_required_changes() {
    let current = StandaloneConfig::default();
    let next = StandaloneConfig {
        gateway_bind: "127.0.0.1:19090".to_owned(),
        cache_backend: CacheBackendKind::Redis,
        cache_url: Some("redis://127.0.0.1:6379/9".to_owned()),
        ..current.clone()
    };

    let applied = merge_applied_service_config(StandaloneServiceKind::Gateway, &current, &next);

    assert_eq!(applied.gateway_bind, "127.0.0.1:19090");
    assert_eq!(applied.cache_backend, CacheBackendKind::Memory);
    assert_eq!(applied.cache_url, None);
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
                                        stream.write_all(b"+PONG\r\n").unwrap();
                                        stream.flush().unwrap();
                                    }
                                    "GET" => {
                                        stream.write_all(b"$-1\r\n").unwrap();
                                        stream.flush().unwrap();
                                    }
                                    "AUTH" | "SELECT" => {
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
                                    break;
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

fn compiled_routing_snapshot_fixture(snapshot_id: &str) -> serde_json::Value {
    serde_json::json!({
        "snapshot_id": snapshot_id,
        "tenant_id": "tenant_local_demo",
        "project_id": "project_local_demo",
        "api_key_group_id": "group-local-demo-live",
        "capability": "responses",
        "route_key": "gpt-4.1",
        "matched_policy_id": "policy-default-responses",
        "project_routing_preferences_project_id": "project_local_demo",
        "applied_routing_profile_id": "profile-global-balanced",
        "strategy": "weighted_random",
        "ordered_provider_ids": [
            "provider-openrouter-main",
            "provider-siliconflow-main",
            "provider-ollama-local"
        ],
        "default_provider_id": "provider-openrouter-main",
        "max_cost": 3.5,
        "max_latency_ms": 8000u64,
        "require_healthy": false,
        "preferred_region": "global",
        "created_at_ms": 1710003000000u64,
        "updated_at_ms": 1710003000500u64
    })
}

fn routing_decision_log_fixture(decision_id: &str, snapshot_id: &str) -> serde_json::Value {
    serde_json::json!({
        "decision_id": decision_id,
        "decision_source": "gateway",
        "tenant_id": "tenant_local_demo",
        "project_id": "project_local_demo",
        "api_key_group_id": "group-local-demo-live",
        "capability": "responses",
        "route_key": "gpt-4.1",
        "selected_provider_id": "provider-openrouter-main",
        "matched_policy_id": "policy-default-responses",
        "applied_routing_profile_id": "profile-global-balanced",
        "compiled_routing_snapshot_id": snapshot_id,
        "strategy": "weighted_random",
        "selection_seed": 17u64,
        "selection_reason": "snapshot alignment test fixture",
        "fallback_reason": null,
        "requested_region": "global",
        "slo_applied": false,
        "slo_degraded": false,
        "created_at_ms": 1710003001000u64,
        "assessments": [
            {
                "provider_id": "provider-openrouter-main",
                "available": true,
                "health": "healthy",
                "policy_rank": 0,
                "weight": 60u64,
                "cost": 0.27,
                "latency_ms": 540u64,
                "region": "global",
                "region_match": true,
                "reasons": ["snapshot fixture provider"]
            }
        ]
    })
}

fn billing_event_fixture(event_id: &str, snapshot_id: &str) -> serde_json::Value {
    serde_json::json!({
        "event_id": event_id,
        "tenant_id": "tenant_local_demo",
        "project_id": "project_local_demo",
        "api_key_group_id": "group-local-demo-live",
        "capability": "responses",
        "route_key": "gpt-4.1",
        "usage_model": "gpt-4.1",
        "provider_id": "provider-openrouter-main",
        "accounting_mode": "platform_credit",
        "operation_kind": "request",
        "modality": "text",
        "api_key_hash": "a19d2bf76318aa7f619d684271469bb383faf1cb5bd4c680088465cde9d0003b",
        "channel_id": "openrouter",
        "reference_id": "req_snapshot_alignment_fixture",
        "latency_ms": 540u64,
        "units": 1u64,
        "request_count": 1u64,
        "input_tokens": 1800u64,
        "output_tokens": 600u64,
        "total_tokens": 2400u64,
        "cache_read_tokens": 0u64,
        "cache_write_tokens": 0u64,
        "image_count": 0u64,
        "audio_seconds": 0.0,
        "video_seconds": 0.0,
        "music_seconds": 0.0,
        "upstream_cost": 0.27,
        "customer_charge": 0.69,
        "applied_routing_profile_id": "profile-global-balanced",
        "compiled_routing_snapshot_id": snapshot_id,
        "fallback_reason": null,
        "created_at_ms": 1710005500000u64
    })
}

fn foreign_tenant_siliconflow_provider_accounts_fixture() -> serde_json::Value {
    serde_json::json!([
        {
            "provider_account_id": "acct-openrouter-default",
            "provider_id": "provider-openrouter-main",
            "display_name": "OpenRouter Default",
            "account_kind": "api_key",
            "owner_scope": "platform",
            "owner_tenant_id": null,
            "execution_instance_id": "provider-openrouter-main",
            "base_url_override": "https://openrouter.ai/api/v1",
            "region": "global",
            "priority": 100,
            "weight": 10,
            "enabled": true,
            "routing_tags": ["default", "global"],
            "health_score_hint": null,
            "latency_ms_hint": null,
            "cost_hint": null,
            "success_rate_hint": null,
            "throughput_hint": null,
            "max_concurrency": null,
            "daily_budget": null,
            "notes": "bootstrap default account"
        },
        {
            "provider_account_id": "acct-ollama-local-default",
            "provider_id": "provider-ollama-local",
            "display_name": "Ollama Local Default",
            "account_kind": "runtime_instance",
            "owner_scope": "platform",
            "owner_tenant_id": null,
            "execution_instance_id": "provider-ollama-local",
            "base_url_override": "http://127.0.0.1:11434",
            "region": "local",
            "priority": 90,
            "weight": 5,
            "enabled": true,
            "routing_tags": ["default", "local"],
            "health_score_hint": null,
            "latency_ms_hint": 35,
            "cost_hint": 0.0,
            "success_rate_hint": null,
            "throughput_hint": null,
            "max_concurrency": null,
            "daily_budget": null,
            "notes": "bootstrap local account"
        },
        {
            "provider_account_id": "acct-siliconflow-tenant-other",
            "provider_id": "provider-siliconflow-main",
            "display_name": "SiliconFlow Other Tenant",
            "account_kind": "api_key",
            "owner_scope": "tenant",
            "owner_tenant_id": "tenant_other_demo",
            "execution_instance_id": "provider-siliconflow-main",
            "base_url_override": "https://api.siliconflow.cn/v1",
            "region": "cn",
            "priority": 95,
            "weight": 8,
            "enabled": true,
            "routing_tags": ["tenant", "other"],
            "health_score_hint": null,
            "latency_ms_hint": null,
            "cost_hint": null,
            "success_rate_hint": null,
            "throughput_hint": null,
            "max_concurrency": null,
            "daily_budget": null,
            "notes": "foreign tenant scoped account"
        }
    ])
}

fn provider_accounts_without_siliconflow_fixture() -> serde_json::Value {
    serde_json::json!([
        {
            "provider_account_id": "acct-openrouter-default",
            "provider_id": "provider-openrouter-main",
            "display_name": "OpenRouter Default",
            "account_kind": "api_key",
            "owner_scope": "platform",
            "owner_tenant_id": null,
            "execution_instance_id": "provider-openrouter-main",
            "base_url_override": "https://openrouter.ai/api/v1",
            "region": "global",
            "priority": 100,
            "weight": 10,
            "enabled": true,
            "routing_tags": ["default", "global"],
            "health_score_hint": null,
            "latency_ms_hint": null,
            "cost_hint": null,
            "success_rate_hint": null,
            "throughput_hint": null,
            "max_concurrency": null,
            "daily_budget": null,
            "notes": "bootstrap default account"
        },
        {
            "provider_account_id": "acct-ollama-local-default",
            "provider_id": "provider-ollama-local",
            "display_name": "Ollama Local Default",
            "account_kind": "runtime_instance",
            "owner_scope": "platform",
            "owner_tenant_id": null,
            "execution_instance_id": "provider-ollama-local",
            "base_url_override": "http://127.0.0.1:11434",
            "region": "local",
            "priority": 90,
            "weight": 5,
            "enabled": true,
            "routing_tags": ["default", "local"],
            "health_score_hint": null,
            "latency_ms_hint": 35,
            "cost_hint": 0.0,
            "success_rate_hint": null,
            "throughput_hint": null,
            "max_concurrency": null,
            "daily_budget": null,
            "notes": "bootstrap local account"
        }
    ])
}

fn model_prices_without_siliconflow_fixture() -> serde_json::Value {
    serde_json::json!([
        {
            "channel_id": "openrouter",
            "model_id": "gpt-4.1",
            "proxy_provider_id": "provider-openrouter-main",
            "currency_code": "USD",
            "price_unit": "per_1m_tokens",
            "input_price": 2.0,
            "output_price": 8.0,
            "cache_read_price": 0.0,
            "cache_write_price": 0.0,
            "request_price": 0.0,
            "price_source_kind": "proxy",
            "is_active": true
        },
        {
            "channel_id": "openrouter",
            "model_id": "deepseek-chat",
            "proxy_provider_id": "provider-openrouter-main",
            "currency_code": "USD",
            "price_unit": "per_1m_tokens",
            "input_price": 0.27,
            "output_price": 1.1,
            "cache_read_price": 0.0,
            "cache_write_price": 0.0,
            "request_price": 0.0,
            "price_source_kind": "proxy",
            "is_active": true
        }
    ])
}

fn routing_fixture_without_siliconflow_candidates() -> serde_json::Value {
    serde_json::json!({
        "profiles": [
            {
                "profile_id": "profile-global-balanced",
                "tenant_id": "tenant_local_demo",
                "project_id": "project_local_demo",
                "name": "Global Balanced",
                "slug": "global-balanced",
                "description": "Balanced multi-provider routing",
                "active": true,
                "strategy": "weighted_random",
                "ordered_provider_ids": [
                    "provider-openrouter-main",
                    "provider-ollama-local"
                ],
                "default_provider_id": "provider-openrouter-main",
                "max_cost": 3.5,
                "max_latency_ms": 8000,
                "require_healthy": false,
                "preferred_region": "global",
                "created_at_ms": 1710000000000u64,
                "updated_at_ms": 1710000000000u64
            }
        ],
        "policies": [
            {
                "policy_id": "policy-default-responses",
                "capability": "responses",
                "model_pattern": "*",
                "enabled": true,
                "priority": 100,
                "strategy": "weighted_random",
                "ordered_provider_ids": [
                    "provider-openrouter-main",
                    "provider-ollama-local"
                ],
                "default_provider_id": "provider-openrouter-main",
                "max_cost": 3.5,
                "max_latency_ms": 8000,
                "require_healthy": false,
                "execution_failover_enabled": true,
                "upstream_retry_max_attempts": 3,
                "upstream_retry_base_delay_ms": 250,
                "upstream_retry_max_delay_ms": 2000
            }
        ],
        "project_preferences": [
            {
                "project_id": "project_local_demo",
                "preset_id": "profile-global-balanced",
                "strategy": "weighted_random",
                "ordered_provider_ids": [
                    "provider-openrouter-main",
                    "provider-ollama-local"
                ],
                "default_provider_id": "provider-openrouter-main",
                "max_cost": 3.5,
                "max_latency_ms": 8000,
                "require_healthy": false,
                "preferred_region": "global",
                "updated_at_ms": 1710000000000u64
            }
        ]
    })
}

fn local_metering_support_billing_event() -> serde_json::Value {
    let mut event = billing_event_fixture(
        "billing-local-demo-metering-support",
        "snapshot-local-demo-live-responses",
    );
    event["reference_id"] = serde_json::json!("req_local_demo_growth_2026");
    event["compiled_routing_snapshot_id"] = serde_json::Value::Null;
    event
}

fn observability_fixture(
    snapshot: serde_json::Value,
    decision_logs: Vec<serde_json::Value>,
) -> serde_json::Value {
    serde_json::json!({
        "compiled_routing_snapshots": [snapshot],
        "routing_decision_logs": decision_logs,
        "provider_health_snapshots": []
    })
}

fn billing_fixture(events: Vec<serde_json::Value>) -> serde_json::Value {
    serde_json::json!({
        "billing_events": events
    })
}

fn billing_fixture_with_local_metering_support(
    mut events: Vec<serde_json::Value>,
) -> serde_json::Value {
    if !events.iter().any(|event| {
        event.get("reference_id") == Some(&serde_json::json!("req_local_demo_growth_2026"))
    }) {
        events.push(local_metering_support_billing_event());
    }
    billing_fixture(events)
}

async fn bootstrap_error_from_profile_pack_override(
    label: &str,
    observability: &serde_json::Value,
    billing: &serde_json::Value,
) -> String {
    let bootstrap_root = temp_bootstrap_root(label);
    write_bootstrap_profile_pack(&bootstrap_root);
    write_json(
        &bootstrap_root.join("observability").join("default.json"),
        observability,
    );
    let billing_events = billing
        .get("billing_events")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .expect("billing fixture must provide a billing_events array");
    write_json(
        &bootstrap_root.join("billing").join("default.json"),
        &billing_fixture_with_local_metering_support(billing_events),
    );

    let mut config = StandaloneConfig::default();
    config.database_url = "sqlite::memory:".to_owned();
    config.bootstrap_data_dir = Some(bootstrap_root.to_string_lossy().into_owned());
    config.bootstrap_profile = "dev".to_owned();

    match build_admin_store_from_config(&config).await {
        Ok(_) => panic!("bootstrap override fixture should fail validation"),
        Err(error) => error.to_string(),
    }
}

fn write_single_account_hold_fixture(
    root: &PathBuf,
    request_id: u64,
    status: &str,
    estimated_quantity: f64,
    captured_quantity: f64,
    released_quantity: f64,
    created_at_ms: u64,
    updated_at_ms: u64,
) {
    write_json(
        &root.join("account-holds").join("default.json"),
        &serde_json::json!({
            "holds": [
                {
                    "hold_id": 8101u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "account_id": 7001u64,
                    "user_id": 9001u64,
                    "request_id": request_id,
                    "status": status,
                    "estimated_quantity": estimated_quantity,
                    "captured_quantity": captured_quantity,
                    "released_quantity": released_quantity,
                    "expires_at_ms": 1710006100000u64,
                    "created_at_ms": created_at_ms,
                    "updated_at_ms": updated_at_ms
                }
            ],
            "allocations": [
                {
                    "hold_allocation_id": 8401u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "hold_id": 8101u64,
                    "lot_id": 8001u64,
                    "allocated_quantity": estimated_quantity,
                    "captured_quantity": captured_quantity,
                    "released_quantity": released_quantity,
                    "created_at_ms": created_at_ms,
                    "updated_at_ms": updated_at_ms
                }
            ]
        }),
    );
}

fn temp_bootstrap_root(label: &str) -> PathBuf {
    let unique = TEMP_BOOTSTRAP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!("sdkwork-bootstrap-tests-{label}-{unique}"));
    if root.exists() {
        fs::remove_dir_all(&root).unwrap();
    }
    fs::create_dir_all(&root).unwrap();
    root
}

fn write_bootstrap_profile_pack(root: &PathBuf) {
    write_json(
        &root.join("profiles").join("dev.json"),
        &serde_json::json!({
            "profile_id": "dev",
            "description": "development bootstrap pack",
            "channels": ["channels/default.json"],
            "providers": ["providers/default.json"],
            "official_provider_configs": ["official-provider-configs/default.json"],
            "provider_accounts": ["provider-accounts/default.json"],
            "models": ["models/default.json"],
            "channel_models": ["channel-models/default.json"],
            "model_prices": ["model-prices/default.json"],
            "tenants": ["tenants/default.json"],
            "projects": ["projects/default.json"],
            "identities": ["identities/default.json"],
            "extensions": ["extensions/default.json"],
            "service_runtime_nodes": ["service-runtime-nodes/default.json"],
            "extension_runtime_rollouts": ["extension-runtime-rollouts/default.json"],
            "standalone_config_rollouts": ["standalone-config-rollouts/default.json"],
            "routing": ["routing/default.json"],
            "api_key_groups": ["api-key-groups/default.json"],
            "observability": ["observability/default.json"],
            "quota_policies": ["quota-policies/default.json"],
            "pricing": ["pricing/default.json"],
            "accounts": ["accounts/default.json"],
            "account_benefit_lots": ["account-benefit-lots/default.json"],
            "account_holds": ["account-holds/default.json"],
            "account_ledger": ["account-ledger/default.json"],
            "request_metering": ["request-metering/default.json"],
            "request_settlements": ["request-settlements/default.json"],
            "account_reconciliation": ["account-reconciliation/default.json"],
            "payment_methods": ["payment-methods/default.json"],
            "marketing": ["marketing/default.json"],
            "commerce": ["commerce/default.json"],
            "billing": ["billing/default.json"],
            "jobs": ["jobs/default.json", "jobs/dev.json"]
        }),
    );
    write_json(
        &root.join("channels").join("default.json"),
        &serde_json::json!([
            { "id": "openrouter", "name": "OpenRouter" },
            { "id": "siliconflow", "name": "SiliconFlow" },
            { "id": "ollama", "name": "Ollama" }
        ]),
    );
    write_json(
        &root.join("providers").join("default.json"),
        &serde_json::json!([
            {
                "id": "provider-openrouter-main",
                "channel_id": "openrouter",
                "extension_id": "sdkwork.provider.openrouter",
                "adapter_kind": "openrouter",
                "protocol_kind": "openai",
                "base_url": "https://openrouter.ai/api/v1",
                "display_name": "OpenRouter Main",
                "channel_bindings": [{ "provider_id": "provider-openrouter-main", "channel_id": "openrouter", "is_primary": true }]
            },
            {
                "id": "provider-siliconflow-main",
                "channel_id": "siliconflow",
                "extension_id": "sdkwork.provider.siliconflow",
                "adapter_kind": "openai-compatible",
                "protocol_kind": "openai",
                "base_url": "https://api.siliconflow.cn/v1",
                "display_name": "SiliconFlow Main",
                "channel_bindings": [{ "provider_id": "provider-siliconflow-main", "channel_id": "siliconflow", "is_primary": true }]
            },
            {
                "id": "provider-ollama-local",
                "channel_id": "ollama",
                "extension_id": "sdkwork.provider.ollama",
                "adapter_kind": "ollama",
                "protocol_kind": "custom",
                "base_url": "http://127.0.0.1:11434",
                "display_name": "Ollama Local",
                "channel_bindings": [{ "provider_id": "provider-ollama-local", "channel_id": "ollama", "is_primary": true }]
            }
        ]),
    );
    write_json(
        &root.join("extensions").join("default.json"),
        &serde_json::json!({
            "installations": [
                {
                    "installation_id": "installation-openrouter-builtin",
                    "extension_id": "sdkwork.provider.openrouter",
                    "runtime": "builtin",
                    "enabled": true,
                    "entrypoint": null,
                    "config": {
                        "health_path": "/v1/models",
                        "plugin_family": "openrouter"
                    }
                },
                {
                    "installation_id": "installation-siliconflow-builtin",
                    "extension_id": "sdkwork.provider.siliconflow",
                    "runtime": "builtin",
                    "enabled": true,
                    "entrypoint": null,
                    "config": {
                        "health_path": "/v1/models",
                        "plugin_family": "siliconflow"
                    }
                },
                {
                    "installation_id": "installation-ollama-builtin",
                    "extension_id": "sdkwork.provider.ollama",
                    "runtime": "builtin",
                    "enabled": true,
                    "entrypoint": null,
                    "config": {
                        "health_path": "/v1/models",
                        "plugin_family": "ollama"
                    }
                }
            ],
            "instances": [
                {
                    "instance_id": "provider-openrouter-main",
                    "installation_id": "installation-openrouter-builtin",
                    "extension_id": "sdkwork.provider.openrouter",
                    "enabled": true,
                    "base_url": "https://openrouter.ai/api/v1",
                    "credential_ref": null,
                    "config": {
                        "routing_hint": "global-balanced"
                    }
                },
                {
                    "instance_id": "provider-siliconflow-main",
                    "installation_id": "installation-siliconflow-builtin",
                    "extension_id": "sdkwork.provider.siliconflow",
                    "enabled": true,
                    "base_url": "https://api.siliconflow.cn/v1",
                    "credential_ref": null,
                    "config": {
                        "routing_hint": "cn-balanced"
                    }
                },
                {
                    "instance_id": "provider-ollama-local",
                    "installation_id": "installation-ollama-builtin",
                    "extension_id": "sdkwork.provider.ollama",
                    "enabled": true,
                    "base_url": "http://127.0.0.1:11434",
                    "credential_ref": null,
                    "config": {
                        "routing_hint": "local-first"
                    }
                }
            ]
        }),
    );
    write_json(
        &root.join("official-provider-configs").join("default.json"),
        &serde_json::json!([
            {
                "provider_id": "provider-openrouter-main",
                "key_reference": "openrouter-default",
                "base_url": "https://openrouter.ai/api/v1",
                "enabled": false,
                "created_at_ms": 0,
                "updated_at_ms": 0
            }
        ]),
    );
    write_json(
        &root.join("provider-accounts").join("default.json"),
        &serde_json::json!([
            {
                "provider_account_id": "acct-openrouter-default",
                "provider_id": "provider-openrouter-main",
                "display_name": "OpenRouter Default",
                "account_kind": "api_key",
                "owner_scope": "platform",
                "owner_tenant_id": null,
                "execution_instance_id": "provider-openrouter-main",
                "base_url_override": "https://openrouter.ai/api/v1",
                "region": "global",
                "priority": 100,
                "weight": 10,
                "enabled": true,
                "routing_tags": ["default", "global"],
                "health_score_hint": null,
                "latency_ms_hint": null,
                "cost_hint": null,
                "success_rate_hint": null,
                "throughput_hint": null,
                "max_concurrency": null,
                "daily_budget": null,
                "notes": "bootstrap default account"
            },
            {
                "provider_account_id": "acct-siliconflow-default",
                "provider_id": "provider-siliconflow-main",
                "display_name": "SiliconFlow Default",
                "account_kind": "api_key",
                "owner_scope": "platform",
                "owner_tenant_id": null,
                "execution_instance_id": "provider-siliconflow-main",
                "base_url_override": "https://api.siliconflow.cn/v1",
                "region": "cn",
                "priority": 95,
                "weight": 8,
                "enabled": true,
                "routing_tags": ["default", "cn"],
                "health_score_hint": null,
                "latency_ms_hint": null,
                "cost_hint": null,
                "success_rate_hint": null,
                "throughput_hint": null,
                "max_concurrency": null,
                "daily_budget": null,
                "notes": "bootstrap siliconflow account"
            },
            {
                "provider_account_id": "acct-ollama-local-default",
                "provider_id": "provider-ollama-local",
                "display_name": "Ollama Local Default",
                "account_kind": "runtime_instance",
                "owner_scope": "platform",
                "owner_tenant_id": null,
                "execution_instance_id": "provider-ollama-local",
                "base_url_override": "http://127.0.0.1:11434",
                "region": "local",
                "priority": 90,
                "weight": 5,
                "enabled": true,
                "routing_tags": ["default", "local"],
                "health_score_hint": null,
                "latency_ms_hint": 35,
                "cost_hint": 0.0,
                "success_rate_hint": null,
                "throughput_hint": null,
                "max_concurrency": null,
                "daily_budget": null,
                "notes": "bootstrap local account"
            }
        ]),
    );
    write_json(
        &root.join("service-runtime-nodes").join("default.json"),
        &serde_json::json!([
            {
                "node_id": "node-gateway-local-a",
                "service_kind": "gateway",
                "started_at_ms": 1710000900000u64,
                "last_seen_at_ms": 1710001900000u64
            },
            {
                "node_id": "node-admin-local-a",
                "service_kind": "admin",
                "started_at_ms": 1710000950000u64,
                "last_seen_at_ms": 1710001950000u64
            }
        ]),
    );
    write_json(
        &root.join("extension-runtime-rollouts").join("default.json"),
        &serde_json::json!({
            "rollouts": [
                {
                    "rollout_id": "rollout-extension-openrouter-refresh",
                    "scope": "instance",
                    "requested_extension_id": null,
                    "requested_instance_id": "provider-openrouter-main",
                    "resolved_extension_id": "sdkwork.provider.openrouter",
                    "created_by": "admin_local_default",
                    "created_at_ms": 1710002100000u64,
                    "deadline_at_ms": 1710002400000u64
                }
            ],
            "participants": [
                {
                    "rollout_id": "rollout-extension-openrouter-refresh",
                    "node_id": "node-gateway-local-a",
                    "service_kind": "gateway",
                    "status": "succeeded",
                    "message": "Gateway runtime already reloaded the OpenRouter instance mapping.",
                    "updated_at_ms": 1710002150000u64
                },
                {
                    "rollout_id": "rollout-extension-openrouter-refresh",
                    "node_id": "node-admin-local-a",
                    "service_kind": "admin",
                    "status": "pending",
                    "message": "Admin rollout observer is waiting for the next supervision pass.",
                    "updated_at_ms": 1710002120000u64
                }
            ]
        }),
    );
    write_json(
        &root.join("standalone-config-rollouts").join("default.json"),
        &serde_json::json!({
            "rollouts": [
                {
                    "rollout_id": "rollout-config-gateway-reload",
                    "requested_service_kind": "gateway",
                    "created_by": "admin_local_default",
                    "created_at_ms": 1710002600000u64,
                    "deadline_at_ms": 1710002900000u64
                }
            ],
            "participants": [
                {
                    "rollout_id": "rollout-config-gateway-reload",
                    "node_id": "node-gateway-local-a",
                    "service_kind": "gateway",
                    "status": "pending",
                    "message": "Gateway nodes will pick up the next config bundle on the next poll.",
                    "updated_at_ms": 1710002610000u64
                }
            ]
        }),
    );
    write_json(
        &root.join("models").join("default.json"),
        &serde_json::json!([
            {
                "external_name": "gpt-4.1",
                "provider_id": "provider-openrouter-main",
                "capabilities": ["responses", "chat_completions"],
                "streaming": true,
                "context_window": 128000
            },
            {
                "external_name": "deepseek-chat",
                "provider_id": "provider-openrouter-main",
                "capabilities": ["responses", "chat_completions"],
                "streaming": true,
                "context_window": 65536
            },
            {
                "external_name": "qwen-plus-latest",
                "provider_id": "provider-siliconflow-main",
                "capabilities": ["responses", "chat_completions"],
                "streaming": true,
                "context_window": 131072
            },
            {
                "external_name": "llama3.2:latest",
                "provider_id": "provider-ollama-local",
                "capabilities": ["responses", "chat_completions"],
                "streaming": true,
                "context_window": 8192
            }
        ]),
    );
    write_json(
        &root.join("channel-models").join("default.json"),
        &serde_json::json!([
            {
                "channel_id": "openrouter",
                "model_id": "gpt-4.1",
                "model_display_name": "GPT-4.1",
                "capabilities": ["responses", "chat_completions"],
                "streaming": true,
                "context_window": 128000,
                "description": "OpenRouter GPT-4.1 default"
            },
            {
                "channel_id": "openrouter",
                "model_id": "deepseek-chat",
                "model_display_name": "DeepSeek Chat",
                "capabilities": ["responses", "chat_completions"],
                "streaming": true,
                "context_window": 65536,
                "description": "OpenRouter DeepSeek chat default"
            },
            {
                "channel_id": "siliconflow",
                "model_id": "qwen-plus-latest",
                "model_display_name": "Qwen Plus Latest",
                "capabilities": ["responses", "chat_completions"],
                "streaming": true,
                "context_window": 131072,
                "description": "SiliconFlow Qwen default"
            },
            {
                "channel_id": "ollama",
                "model_id": "llama3.2:latest",
                "model_display_name": "Llama 3.2 Latest",
                "capabilities": ["responses", "chat_completions"],
                "streaming": true,
                "context_window": 8192,
                "description": "Ollama local default"
            }
        ]),
    );
    write_json(
        &root.join("model-prices").join("default.json"),
        &serde_json::json!([
            {
                "channel_id": "openrouter",
                "model_id": "gpt-4.1",
                "proxy_provider_id": "provider-openrouter-main",
                "currency_code": "USD",
                "price_unit": "per_1m_tokens",
                "input_price": 2.0,
                "output_price": 8.0,
                "cache_read_price": 0.0,
                "cache_write_price": 0.0,
                "request_price": 0.0,
                "price_source_kind": "proxy",
                "is_active": true
            },
            {
                "channel_id": "openrouter",
                "model_id": "deepseek-chat",
                "proxy_provider_id": "provider-openrouter-main",
                "currency_code": "USD",
                "price_unit": "per_1m_tokens",
                "input_price": 0.27,
                "output_price": 1.1,
                "cache_read_price": 0.0,
                "cache_write_price": 0.0,
                "request_price": 0.0,
                "price_source_kind": "proxy",
                "is_active": true
            },
            {
                "channel_id": "siliconflow",
                "model_id": "qwen-plus-latest",
                "proxy_provider_id": "provider-siliconflow-main",
                "currency_code": "USD",
                "price_unit": "per_1m_tokens",
                "input_price": 0.4,
                "output_price": 1.2,
                "cache_read_price": 0.0,
                "cache_write_price": 0.0,
                "request_price": 0.0,
                "price_source_kind": "proxy",
                "is_active": true
            }
        ]),
    );
    write_json(
        &root.join("tenants").join("default.json"),
        &serde_json::json!([
            { "id": "tenant_local_demo", "name": "Local Demo Workspace" }
        ]),
    );
    write_json(
        &root.join("projects").join("default.json"),
        &serde_json::json!([
            { "tenant_id": "tenant_local_demo", "id": "project_local_demo", "name": "default" }
        ]),
    );
    write_json(
        &root.join("identities").join("default.json"),
        &serde_json::json!({
            "admin_users": [
                {
                    "id": "admin_local_default",
                    "email": "admin@sdkwork.local",
                    "display_name": "Admin Operator",
                    "password_salt": "c2Rrd29ya0FkbWluU2VlZA",
                    "password_hash": "$argon2id$v=19$m=19456,t=2,p=1$c2Rrd29ya0FkbWluU2VlZA$Qqn9CGRiZIU7JwoSPHszrNY459YXH9M3WSwuNnJSwxM",
                    "active": true,
                    "created_at_ms": 1710000000000u64
                }
            ],
            "portal_users": [
                {
                    "id": "user_local_demo",
                    "email": "portal@sdkwork.local",
                    "display_name": "Portal Demo",
                    "password_salt": "c2Rrd29ya1BvcnRhbFNlZWQ",
                    "password_hash": "$argon2id$v=19$m=19456,t=2,p=1$c2Rrd29ya1BvcnRhbFNlZWQ$A2N73CiDSUv+hpFm7j0p3Jx6SJv2+JEpqUbqUdhSRgU",
                    "workspace_tenant_id": "tenant_local_demo",
                    "workspace_project_id": "project_local_demo",
                    "active": true,
                    "created_at_ms": 1710000000000u64
                }
            ],
            "gateway_api_keys": [
                {
                    "tenant_id": "tenant_local_demo",
                    "project_id": "project_local_demo",
                    "environment": "live",
                    "hashed_key": "a19d2bf76318aa7f619d684271469bb383faf1cb5bd4c680088465cde9d0003b",
                    "api_key_group_id": "group-local-demo-live",
                    "raw_key": "skw_live_local_demo_2026",
                    "label": "Local Demo Live Key",
                    "notes": "Bootstrap live key for local demo workspace",
                    "created_at_ms": 1710002000000u64,
                    "last_used_at_ms": null,
                    "expires_at_ms": null,
                    "active": true
                },
                {
                    "tenant_id": "tenant_local_demo",
                    "project_id": "project_local_demo",
                    "environment": "sandbox",
                    "hashed_key": "13072ae2c436e62116c61d76c68e7cc32a7a1e252a1d192490d6ac7cc92295eb",
                    "api_key_group_id": "group-local-demo-sandbox",
                    "raw_key": "skw_sandbox_local_demo_2026",
                    "label": "Local Demo Sandbox Key",
                    "notes": "Bootstrap sandbox key for local demo workspace",
                    "created_at_ms": 1710002005000u64,
                    "last_used_at_ms": null,
                    "expires_at_ms": null,
                    "active": true
                }
            ]
        }),
    );
    write_json(
        &root.join("routing").join("default.json"),
        &serde_json::json!({
            "profiles": [
                {
                    "profile_id": "profile-global-balanced",
                    "tenant_id": "tenant_local_demo",
                    "project_id": "project_local_demo",
                    "name": "Global Balanced",
                    "slug": "global-balanced",
                    "description": "Balanced multi-provider routing",
                    "active": true,
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-openrouter-main",
                        "provider-siliconflow-main",
                        "provider-ollama-local"
                    ],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "preferred_region": "global",
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ],
            "policies": [
                {
                    "policy_id": "policy-default-responses",
                    "capability": "responses",
                    "model_pattern": "*",
                    "enabled": true,
                    "priority": 100,
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-openrouter-main",
                        "provider-siliconflow-main",
                        "provider-ollama-local"
                    ],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "execution_failover_enabled": true,
                    "upstream_retry_max_attempts": 3,
                    "upstream_retry_base_delay_ms": 250,
                    "upstream_retry_max_delay_ms": 2000
                }
            ],
            "project_preferences": [
                {
                    "project_id": "project_local_demo",
                    "preset_id": "profile-global-balanced",
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-openrouter-main",
                        "provider-siliconflow-main"
                    ],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000,
                    "require_healthy": false,
                    "preferred_region": "global",
                    "updated_at_ms": 1710000000000u64
                }
            ]
        }),
    );
    write_json(
        &root.join("api-key-groups").join("default.json"),
        &serde_json::json!([
            {
                "group_id": "group-local-demo-live",
                "tenant_id": "tenant_local_demo",
                "project_id": "project_local_demo",
                "environment": "live",
                "name": "Local Demo Live",
                "slug": "local-demo-live",
                "description": "Default live traffic group for the local demo workspace",
                "color": "#0f766e",
                "default_capability_scope": "responses",
                "default_routing_profile_id": "profile-global-balanced",
                "default_accounting_mode": "platform_credit",
                "active": true,
                "created_at_ms": 1710000000000u64,
                "updated_at_ms": 1710000000000u64
            },
            {
                "group_id": "group-local-demo-sandbox",
                "tenant_id": "tenant_local_demo",
                "project_id": "project_local_demo",
                "environment": "sandbox",
                "name": "Local Demo Sandbox",
                "slug": "local-demo-sandbox",
                "description": "Low-risk sandbox traffic group for the local demo workspace",
                "color": "#1d4ed8",
                "default_capability_scope": "responses",
                "default_routing_profile_id": "profile-global-balanced",
                "default_accounting_mode": "platform_credit",
                "active": true,
                "created_at_ms": 1710000000000u64,
                "updated_at_ms": 1710000000000u64
            }
        ]),
    );
    write_json(
        &root.join("observability").join("default.json"),
        &serde_json::json!({
            "compiled_routing_snapshots": [
                {
                    "snapshot_id": "snapshot-local-demo-live-responses",
                    "tenant_id": "tenant_local_demo",
                    "project_id": "project_local_demo",
                    "api_key_group_id": "group-local-demo-live",
                    "capability": "responses",
                    "route_key": "gpt-4.1",
                    "matched_policy_id": "policy-default-responses",
                    "project_routing_preferences_project_id": "project_local_demo",
                    "applied_routing_profile_id": "profile-global-balanced",
                    "strategy": "weighted_random",
                    "ordered_provider_ids": [
                        "provider-openrouter-main",
                        "provider-siliconflow-main",
                        "provider-ollama-local"
                    ],
                    "default_provider_id": "provider-openrouter-main",
                    "max_cost": 3.5,
                    "max_latency_ms": 8000u64,
                    "require_healthy": false,
                    "preferred_region": "global",
                    "created_at_ms": 1710003000000u64,
                    "updated_at_ms": 1710003000500u64
                }
            ],
            "routing_decision_logs": [
                {
                    "decision_id": "decision-local-demo-live-responses",
                    "decision_source": "gateway",
                    "tenant_id": "tenant_local_demo",
                    "project_id": "project_local_demo",
                    "api_key_group_id": "group-local-demo-live",
                    "capability": "responses",
                    "route_key": "gpt-4.1",
                    "selected_provider_id": "provider-openrouter-main",
                    "matched_policy_id": "policy-default-responses",
                    "applied_routing_profile_id": "profile-global-balanced",
                    "compiled_routing_snapshot_id": "snapshot-local-demo-live-responses",
                    "strategy": "weighted_random",
                    "selection_seed": 17u64,
                    "selection_reason": "balanced profile selected the first healthy frontier pool candidate",
                    "fallback_reason": null,
                    "requested_region": "global",
                    "slo_applied": false,
                    "slo_degraded": false,
                    "created_at_ms": 1710003001000u64,
                    "assessments": [
                        {
                            "provider_id": "provider-openrouter-main",
                            "available": true,
                            "health": "healthy",
                            "policy_rank": 0,
                            "weight": 60u64,
                            "cost": 0.27,
                            "latency_ms": 540u64,
                            "region": "global",
                            "region_match": true,
                            "reasons": ["broad model marketplace coverage"]
                        },
                        {
                            "provider_id": "provider-siliconflow-main",
                            "available": true,
                            "health": "healthy",
                            "policy_rank": 1,
                            "weight": 30u64,
                            "cost": 0.4,
                            "latency_ms": 620u64,
                            "region": "cn",
                            "region_match": false,
                            "reasons": ["cost-effective cn fallback"]
                        },
                        {
                            "provider_id": "provider-ollama-local",
                            "available": true,
                            "health": "healthy",
                            "policy_rank": 2,
                            "weight": 10u64,
                            "cost": 0.0,
                            "latency_ms": 180u64,
                            "region": "local",
                            "region_match": false,
                            "reasons": ["local privacy-first fallback"]
                        }
                    ]
                }
            ],
            "provider_health_snapshots": [
                {
                    "provider_id": "provider-openrouter-main",
                    "extension_id": "sdkwork.provider.openrouter",
                    "runtime": "builtin",
                    "observed_at_ms": 1710003002000u64,
                    "instance_id": "provider-openrouter-main",
                    "running": true,
                    "healthy": true,
                    "message": "marketplace relay healthy"
                },
                {
                    "provider_id": "provider-siliconflow-main",
                    "extension_id": "sdkwork.provider.siliconflow",
                    "runtime": "builtin",
                    "observed_at_ms": 1710003002500u64,
                    "instance_id": "provider-siliconflow-main",
                    "running": true,
                    "healthy": true,
                    "message": "cn relay healthy"
                },
                {
                    "provider_id": "provider-ollama-local",
                    "extension_id": "sdkwork.provider.ollama",
                    "runtime": "builtin",
                    "observed_at_ms": 1710003003000u64,
                    "instance_id": "provider-ollama-local",
                    "running": true,
                    "healthy": true,
                    "message": "local daemon ready"
                }
            ]
        }),
    );
    write_json(
        &root.join("quota-policies").join("default.json"),
        &serde_json::json!({
            "quota_policies": [
                {
                    "policy_id": "quota-default-live",
                    "project_id": "project_local_demo",
                    "max_units": 5000000u64,
                    "enabled": true
                }
            ],
            "rate_limit_policies": [
                {
                    "policy_id": "rate-limit-default-live",
                    "project_id": "project_local_demo",
                    "api_key_hash": null,
                    "route_key": "responses",
                    "model_name": null,
                    "requests_per_window": 120u64,
                    "window_seconds": 60u64,
                    "burst_requests": 180u64,
                    "enabled": true,
                    "notes": "default live burst",
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ]
        }),
    );
    write_json(
        &root.join("pricing").join("default.json"),
        &serde_json::json!({
            "plans": [
                {
                    "pricing_plan_id": 9101u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "plan_code": "global-default-commercial",
                    "plan_version": 1u64,
                    "display_name": "Global Default Commercial",
                    "currency_code": "USD",
                    "credit_unit_code": "credit",
                    "status": "active",
                    "effective_from_ms": 1710000000000u64,
                    "effective_to_ms": null,
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ],
            "rates": [
                {
                    "pricing_rate_id": 9201u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "pricing_plan_id": 9101u64,
                    "metric_code": "tokens.input",
                    "capability_code": "responses",
                    "model_code": null,
                    "provider_code": null,
                    "charge_unit": "1k_tokens",
                    "pricing_method": "per_unit",
                    "quantity_step": 1.0,
                    "unit_price": 0.002,
                    "display_price_unit": "$ / 1K input tokens",
                    "minimum_billable_quantity": 0.0,
                    "minimum_charge": 0.0,
                    "rounding_increment": 1.0,
                    "rounding_mode": "none",
                    "included_quantity": 0.0,
                    "priority": 10u64,
                    "notes": "default responses input pricing",
                    "status": "active",
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ]
        }),
    );
    write_json(
        &root.join("accounts").join("default.json"),
        &serde_json::json!([
            {
                "account_id": 7001u64,
                "tenant_id": 1001u64,
                "organization_id": 2001u64,
                "user_id": 9001u64,
                "account_type": "primary",
                "currency_code": "USD",
                "credit_unit_code": "credit",
                "status": "active",
                "allow_overdraft": false,
                "overdraft_limit": 0.0,
                "created_at_ms": 1710004000000u64,
                "updated_at_ms": 1710004000100u64
            }
        ]),
    );
    write_json(
        &root.join("account-benefit-lots").join("default.json"),
        &serde_json::json!([
            {
                "lot_id": 8001u64,
                "tenant_id": 1001u64,
                "organization_id": 2001u64,
                "account_id": 7001u64,
                "user_id": 9001u64,
                "benefit_type": "cash_credit",
                "source_type": "grant",
                "source_id": null,
                "scope_json": "{\"project_id\":\"project_local_demo\",\"route_profile\":\"profile-global-balanced\"}",
                "original_quantity": 3000000.0,
                "remaining_quantity": 2997700.0,
                "held_quantity": 0.0,
                "priority": 10,
                "acquired_unit_cost": 0.0,
                "issued_at_ms": 1710005000000u64,
                "expires_at_ms": null,
                "status": "active",
                "created_at_ms": 1710005000000u64,
                "updated_at_ms": 1710006500000u64
            },
            {
                "lot_id": 8002u64,
                "tenant_id": 1001u64,
                "organization_id": 2001u64,
                "account_id": 7001u64,
                "user_id": 9001u64,
                "benefit_type": "promo_credit",
                "source_type": "coupon",
                "source_id": 1001u64,
                "scope_json": "{\"campaign\":\"campaign-launch-q2\"}",
                "original_quantity": 250000.0,
                "remaining_quantity": 250000.0,
                "held_quantity": 0.0,
                "priority": 20,
                "acquired_unit_cost": 0.0,
                "issued_at_ms": 1710005000100u64,
                "expires_at_ms": 1767225600000u64,
                "status": "active",
                "created_at_ms": 1710005000100u64,
                "updated_at_ms": 1710006500000u64
            }
        ]),
    );
    write_json(
        &root.join("account-holds").join("default.json"),
        &serde_json::json!({
            "holds": [
                {
                    "hold_id": 8101u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "account_id": 7001u64,
                    "user_id": 9001u64,
                    "request_id": 6001u64,
                    "status": "partially_released",
                    "estimated_quantity": 2400.0,
                    "captured_quantity": 2300.0,
                    "released_quantity": 100.0,
                    "expires_at_ms": 1710006100000u64,
                    "created_at_ms": 1710005500000u64,
                    "updated_at_ms": 1710005500900u64
                }
            ],
            "allocations": [
                {
                    "hold_allocation_id": 8401u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "hold_id": 8101u64,
                    "lot_id": 8001u64,
                    "allocated_quantity": 2400.0,
                    "captured_quantity": 2300.0,
                    "released_quantity": 100.0,
                    "created_at_ms": 1710005500000u64,
                    "updated_at_ms": 1710005500900u64
                }
            ]
        }),
    );
    write_json(
        &root.join("account-ledger").join("default.json"),
        &serde_json::json!({
            "entries": [
                {
                    "ledger_entry_id": 8201u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "account_id": 7001u64,
                    "user_id": 9001u64,
                    "request_id": null,
                    "hold_id": null,
                    "entry_type": "grant_issue",
                    "benefit_type": "cash_credit",
                    "quantity": 3000000.0,
                    "amount": 0.0,
                    "created_at_ms": 1710005000000u64
                },
                {
                    "ledger_entry_id": 8202u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "account_id": 7001u64,
                    "user_id": 9001u64,
                    "request_id": null,
                    "hold_id": null,
                    "entry_type": "grant_issue",
                    "benefit_type": "promo_credit",
                    "quantity": 250000.0,
                    "amount": 0.0,
                    "created_at_ms": 1710005000100u64
                },
                {
                    "ledger_entry_id": 8203u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "account_id": 7001u64,
                    "user_id": 9001u64,
                    "request_id": 6001u64,
                    "hold_id": 8101u64,
                    "entry_type": "settlement_capture",
                    "benefit_type": "cash_credit",
                    "quantity": -2300.0,
                    "amount": -0.69,
                    "created_at_ms": 1710005500900u64
                }
            ],
            "allocations": [
                {
                    "ledger_allocation_id": 8501u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "ledger_entry_id": 8201u64,
                    "lot_id": 8001u64,
                    "quantity_delta": 3000000.0,
                    "created_at_ms": 1710005000000u64
                },
                {
                    "ledger_allocation_id": 8502u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "ledger_entry_id": 8202u64,
                    "lot_id": 8002u64,
                    "quantity_delta": 250000.0,
                    "created_at_ms": 1710005000100u64
                },
                {
                    "ledger_allocation_id": 8503u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "ledger_entry_id": 8203u64,
                    "lot_id": 8001u64,
                    "quantity_delta": -2300.0,
                    "created_at_ms": 1710005500900u64
                }
            ]
        }),
    );
    write_json(
        &root.join("request-metering").join("default.json"),
        &serde_json::json!({
            "facts": [
                {
                    "request_id": 6001u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "user_id": 9001u64,
                    "account_id": 7001u64,
                    "api_key_id": 10001u64,
                    "api_key_hash": "a19d2bf76318aa7f619d684271469bb383faf1cb5bd4c680088465cde9d0003b",
                    "auth_type": "api_key",
                    "jwt_subject": null,
                    "platform": "portal",
                    "owner": "user_local_demo",
                    "request_trace_id": "trace-local-demo-growth-2026",
                    "gateway_request_ref": "req_local_demo_growth_2026",
                    "upstream_request_ref": "or-local-demo-growth-2026",
                    "protocol_family": "openai",
                    "capability_code": "responses",
                    "channel_code": "openrouter",
                    "model_code": "gpt-4.1",
                    "provider_code": "provider-openrouter-main",
                    "request_status": "succeeded",
                    "usage_capture_status": "captured",
                    "cost_pricing_plan_id": 9101u64,
                    "retail_pricing_plan_id": 9101u64,
                    "estimated_credit_hold": 2400.0,
                    "actual_credit_charge": 2300.0,
                    "actual_provider_cost": 0.27,
                    "started_at_ms": 1710005500000u64,
                    "finished_at_ms": 1710005500900u64,
                    "created_at_ms": 1710005500000u64,
                    "updated_at_ms": 1710005500900u64
                }
            ],
            "metrics": [
                {
                    "request_metric_id": 7001001u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "request_id": 6001u64,
                    "metric_code": "token.input",
                    "quantity": 1800.0,
                    "provider_field": "prompt_tokens",
                    "source_kind": "provider",
                    "capture_stage": "final",
                    "is_billable": true,
                    "captured_at_ms": 1710005500850u64
                },
                {
                    "request_metric_id": 7001002u64,
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "request_id": 6001u64,
                    "metric_code": "token.output",
                    "quantity": 600.0,
                    "provider_field": "completion_tokens",
                    "source_kind": "provider",
                    "capture_stage": "final",
                    "is_billable": true,
                    "captured_at_ms": 1710005500850u64
                }
            ]
        }),
    );
    write_json(
        &root.join("request-settlements").join("default.json"),
        &serde_json::json!([
            {
                "request_settlement_id": 8301u64,
                "tenant_id": 1001u64,
                "organization_id": 2001u64,
                "request_id": 6001u64,
                "account_id": 7001u64,
                "user_id": 9001u64,
                "hold_id": 8101u64,
                "status": "partially_released",
                "estimated_credit_hold": 2400.0,
                "released_credit_amount": 100.0,
                "captured_credit_amount": 2300.0,
                "provider_cost_amount": 0.27,
                "retail_charge_amount": 0.69,
                "shortfall_amount": 0.0,
                "refunded_amount": 0.0,
                "settled_at_ms": 1710005500900u64,
                "created_at_ms": 1710005500000u64,
                "updated_at_ms": 1710005500900u64
            }
        ]),
    );
    write_json(
        &root.join("account-reconciliation").join("default.json"),
        &serde_json::json!([
            {
                "tenant_id": 1001u64,
                "organization_id": 2001u64,
                "account_id": 7001u64,
                "project_id": "project_local_demo",
                "last_order_updated_at_ms": 1710005000500u64,
                "last_order_created_at_ms": 1710005000000u64,
                "last_order_id": "order-local-demo-growth-2026",
                "updated_at_ms": 1710005300700u64
            }
        ]),
    );
    write_json(
        &root.join("payment-methods").join("default.json"),
        &serde_json::json!({
            "payment_methods": [
                {
                    "payment_method_id": "payment-stripe-hosted",
                    "display_name": "Stripe Hosted Checkout",
                    "description": "Global card and wallet checkout",
                    "provider": "stripe",
                    "channel": "hosted_checkout",
                    "mode": "live",
                    "enabled": true,
                    "sort_order": 10,
                    "capability_codes": ["checkout", "refund", "recommended"],
                    "supported_currency_codes": ["USD", "EUR"],
                    "supported_country_codes": ["US", "DE", "SG"],
                    "supported_order_kinds": ["subscription_plan", "recharge_pack", "custom_recharge"],
                    "callback_strategy": "webhook_signed",
                    "webhook_path": "/api/portal/commerce/webhooks/stripe",
                    "webhook_tolerance_seconds": 300u64,
                    "replay_window_seconds": 300u64,
                    "max_retry_count": 8u32,
                    "config_json": "{\"provider\":\"stripe\"}",
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ],
            "credential_bindings": [],
            "project_memberships": []
        }),
    );
    write_json(
        &root.join("marketing").join("default.json"),
        &serde_json::json!({
            "coupon_templates": [
                {
                    "coupon_template_id": "template-launch-credit-100",
                    "template_key": "launch-credit-100",
                    "display_name": "Launch Credit 100",
                    "status": "active",
                    "distribution_kind": "shared_code",
                    "benefit": {
                        "benefit_kind": "fixed_amount_off",
                        "discount_percent": null,
                        "discount_amount_minor": 10000u64,
                        "grant_units": null,
                        "currency_code": "USD",
                        "max_discount_minor": 10000u64
                    },
                    "restriction": {
                        "subject_scope": "project",
                        "min_order_amount_minor": 10000u64,
                        "first_order_only": true,
                        "new_customer_only": true,
                        "exclusive_group": "launch",
                        "stacking_policy": "exclusive",
                        "max_redemptions_per_subject": 1u64,
                        "eligible_target_kinds": ["subscription_plan", "recharge_pack"]
                    },
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ],
            "marketing_campaigns": [
                {
                    "marketing_campaign_id": "campaign-launch-q2",
                    "coupon_template_id": "template-launch-credit-100",
                    "display_name": "Launch Campaign Q2",
                    "status": "active",
                    "start_at_ms": 1710000000000u64,
                    "end_at_ms": 1767225600000u64,
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ],
            "campaign_budgets": [
                {
                    "campaign_budget_id": "budget-launch-q2",
                    "marketing_campaign_id": "campaign-launch-q2",
                    "status": "active",
                    "total_budget_minor": 5000000u64,
                    "reserved_budget_minor": 0u64,
                    "consumed_budget_minor": 0u64,
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ],
            "coupon_codes": [
                {
                    "coupon_code_id": "coupon-code-launch-credit-100",
                    "coupon_template_id": "template-launch-credit-100",
                    "code_value": "LAUNCH100",
                    "status": "available",
                    "claimed_subject_scope": null,
                    "claimed_subject_id": null,
                    "expires_at_ms": 1767225600000u64,
                    "created_at_ms": 1710000000000u64,
                    "updated_at_ms": 1710000000000u64
                }
            ]
        }),
    );
    write_json(
        &root.join("commerce").join("default.json"),
        &serde_json::json!({
            "orders": [
                {
                    "order_id": "order-local-demo-growth-2026",
                    "project_id": "project_local_demo",
                    "user_id": "user_local_demo",
                    "target_kind": "subscription_plan",
                    "target_id": "plan-local-growth",
                    "target_name": "Local Demo Growth",
                    "list_price_cents": 19900u64,
                    "payable_price_cents": 9900u64,
                    "list_price_label": "$199 / month",
                    "payable_price_label": "$99 / month",
                    "granted_units": 3000000u64,
                    "bonus_units": 250000u64,
                    "currency_code": "USD",
                    "pricing_plan_id": "global-default-commercial",
                    "pricing_plan_version": 1u64,
                    "pricing_snapshot_json": "{\"plan_code\":\"global-default-commercial\",\"billing_interval\":\"monthly\"}",
                    "applied_coupon_code": "LAUNCH100",
                    "coupon_reservation_id": null,
                    "coupon_redemption_id": null,
                    "marketing_campaign_id": "campaign-launch-q2",
                    "subsidy_amount_minor": 10000u64,
                    "payment_method_id": "payment-stripe-hosted",
                    "latest_payment_attempt_id": "attempt-local-demo-growth-2026",
                    "status": "fulfilled",
                    "settlement_status": "settled",
                    "source": "bootstrap",
                    "refundable_amount_minor": 9900u64,
                    "refunded_amount_minor": 1000u64,
                    "created_at_ms": 1710005000000u64,
                    "updated_at_ms": 1710005000500u64
                }
            ],
            "payment_attempts": [
                {
                    "payment_attempt_id": "attempt-local-demo-growth-2026",
                    "order_id": "order-local-demo-growth-2026",
                    "project_id": "project_local_demo",
                    "user_id": "user_local_demo",
                    "payment_method_id": "payment-stripe-hosted",
                    "provider": "stripe",
                    "channel": "hosted_checkout",
                    "status": "succeeded",
                    "idempotency_key": "attempt:order-local-demo-growth-2026:1",
                    "attempt_sequence": 1u32,
                    "amount_minor": 9900u64,
                    "currency_code": "USD",
                    "captured_amount_minor": 9900u64,
                    "refunded_amount_minor": 1000u64,
                    "provider_payment_intent_id": "pi_local_demo_growth_2026",
                    "provider_checkout_session_id": "cs_local_demo_growth_2026",
                    "provider_reference": "stripe-payment-reference-local-demo-growth-2026",
                    "checkout_url": "https://checkout.stripe.test/pay/cs_local_demo_growth_2026",
                    "qr_code_payload": null,
                    "request_payload_json": "{\"order_id\":\"order-local-demo-growth-2026\",\"amount_minor\":9900}",
                    "response_payload_json": "{\"payment_intent\":\"pi_local_demo_growth_2026\",\"status\":\"succeeded\"}",
                    "error_code": null,
                    "error_message": null,
                    "initiated_at_ms": 1710005000200u64,
                    "expires_at_ms": 1710005600200u64,
                    "completed_at_ms": 1710005000800u64,
                    "updated_at_ms": 1710005000900u64
                }
            ],
            "payment_events": [
                {
                    "payment_event_id": "payment-event-local-demo-growth-2026",
                    "order_id": "order-local-demo-growth-2026",
                    "project_id": "project_local_demo",
                    "user_id": "user_local_demo",
                    "provider": "stripe",
                    "provider_event_id": "evt_local_demo_growth_2026",
                    "dedupe_key": "stripe:evt_local_demo_growth_2026",
                    "event_type": "checkout.session.completed",
                    "payload_json": "{\"id\":\"evt_local_demo_growth_2026\",\"mode\":\"subscription\"}",
                    "processing_status": "processed",
                    "processing_message": "bootstrap payment settled",
                    "received_at_ms": 1710005000600u64,
                    "processed_at_ms": 1710005000900u64,
                    "order_status_after": "fulfilled"
                }
            ],
            "webhook_inbox_records": [
                {
                    "webhook_inbox_id": "webhook-inbox-local-demo-growth-2026",
                    "provider": "stripe",
                    "payment_method_id": "payment-stripe-hosted",
                    "provider_event_id": "evt_local_demo_growth_2026",
                    "dedupe_key": "stripe:evt_local_demo_growth_2026",
                    "signature_header": "t=1710005000,v1=testsig",
                    "payload_json": "{\"id\":\"evt_local_demo_growth_2026\",\"type\":\"checkout.session.completed\"}",
                    "processing_status": "processed",
                    "retry_count": 0u32,
                    "max_retry_count": 8u32,
                    "last_error_message": null,
                    "next_retry_at_ms": null,
                    "first_received_at_ms": 1710005000550u64,
                    "last_received_at_ms": 1710005000600u64,
                    "processed_at_ms": 1710005000900u64
                }
            ],
            "refunds": [
                {
                    "refund_id": "refund-local-demo-growth-2026",
                    "order_id": "order-local-demo-growth-2026",
                    "payment_attempt_id": "attempt-local-demo-growth-2026",
                    "payment_method_id": "payment-stripe-hosted",
                    "provider": "stripe",
                    "provider_refund_id": "re_local_demo_growth_2026",
                    "idempotency_key": "refund:order-local-demo-growth-2026:1",
                    "reason": "customer_goodwill_credit",
                    "status": "succeeded",
                    "amount_minor": 1000u64,
                    "currency_code": "USD",
                    "request_payload_json": "{\"amount_minor\":1000,\"reason\":\"customer_goodwill_credit\"}",
                    "response_payload_json": "{\"refund_id\":\"re_local_demo_growth_2026\",\"status\":\"succeeded\"}",
                    "created_at_ms": 1710005200000u64,
                    "updated_at_ms": 1710005200400u64,
                    "completed_at_ms": 1710005200400u64
                }
            ],
            "reconciliation_runs": [
                {
                    "reconciliation_run_id": "recon-run-local-demo-growth-2026",
                    "provider": "stripe",
                    "payment_method_id": "payment-stripe-hosted",
                    "scope_started_at_ms": 1710004800000u64,
                    "scope_ended_at_ms": 1710005400000u64,
                    "status": "completed",
                    "summary_json": "{\"matched\":1,\"refunds\":1,\"discrepancies\":1}",
                    "created_at_ms": 1710005300000u64,
                    "updated_at_ms": 1710005300500u64,
                    "completed_at_ms": 1710005300500u64
                }
            ],
            "reconciliation_items": [
                {
                    "reconciliation_item_id": "recon-item-local-demo-growth-2026",
                    "reconciliation_run_id": "recon-run-local-demo-growth-2026",
                    "order_id": "order-local-demo-growth-2026",
                    "payment_attempt_id": "attempt-local-demo-growth-2026",
                    "refund_id": "refund-local-demo-growth-2026",
                    "external_reference": "stripe:evt_local_demo_growth_2026",
                    "discrepancy_type": "manual_refund_review",
                    "status": "open",
                    "expected_amount_minor": 1000i64,
                    "provider_amount_minor": 1000i64,
                    "detail_json": "{\"note\":\"refund requires operator review despite matching provider totals\"}",
                    "created_at_ms": 1710005300600u64,
                    "updated_at_ms": 1710005300600u64
                }
            ]
        }),
    );
    write_json(
        &root.join("billing").join("default.json"),
        &serde_json::json!({
            "billing_events": [
                {
                    "event_id": "billing-local-demo-growth-2026",
                    "tenant_id": "tenant_local_demo",
                    "project_id": "project_local_demo",
                    "api_key_group_id": "group-local-demo-live",
                    "capability": "responses",
                    "route_key": "gpt-4.1",
                    "usage_model": "gpt-4.1",
                    "provider_id": "provider-openrouter-main",
                    "accounting_mode": "platform_credit",
                    "operation_kind": "request",
                    "modality": "text",
                    "api_key_hash": "a19d2bf76318aa7f619d684271469bb383faf1cb5bd4c680088465cde9d0003b",
                    "channel_id": "openrouter",
                    "reference_id": "req_local_demo_growth_2026",
                    "latency_ms": 540u64,
                    "units": 1u64,
                    "request_count": 1u64,
                    "input_tokens": 1800u64,
                    "output_tokens": 600u64,
                    "total_tokens": 2400u64,
                    "cache_read_tokens": 0u64,
                    "cache_write_tokens": 0u64,
                    "image_count": 0u64,
                    "audio_seconds": 0.0,
                    "video_seconds": 0.0,
                    "music_seconds": 0.0,
                    "upstream_cost": 0.27,
                    "customer_charge": 0.69,
                    "applied_routing_profile_id": "profile-global-balanced",
                    "compiled_routing_snapshot_id": "snapshot-local-demo-live-responses",
                    "fallback_reason": null,
                    "created_at_ms": 1710005500000u64
                }
            ]
        }),
    );
    write_json(
        &root.join("jobs").join("default.json"),
        &serde_json::json!({
            "jobs": [
                {
                    "job_id": "job-local-demo-growth-brief",
                    "tenant_id": 1001u64,
                    "organization_id": 2001u64,
                    "user_id": 9001u64,
                    "account_id": 7001u64,
                    "request_id": 5001u64,
                    "provider_id": "provider-openrouter-main",
                    "model_code": "deepseek-chat",
                    "capability_code": "responses",
                    "modality": "text",
                    "operation_kind": "draft_generate",
                    "status": "succeeded",
                    "external_job_id": "or-job-local-demo-growth-brief",
                    "idempotency_key": "job:local-demo:growth-brief",
                    "callback_url": "https://portal.sdkwork.local/api/jobs/callbacks/local-demo",
                    "input_summary": "Generate a commercial growth brief for the local demo workspace",
                    "progress_percent": 100u64,
                    "error_code": null,
                    "error_message": null,
                    "created_at_ms": 1710005600000u64,
                    "updated_at_ms": 1710005600500u64,
                    "started_at_ms": 1710005600100u64,
                    "completed_at_ms": 1710005600500u64
                }
            ],
            "attempts": [
                {
                    "attempt_id": 8801u64,
                    "job_id": "job-local-demo-growth-brief",
                    "attempt_number": 1u64,
                    "status": "succeeded",
                    "runtime_kind": "openrouter",
                    "endpoint": "https://openrouter.ai/api/v1/responses",
                    "external_job_id": "or-job-local-demo-growth-brief",
                    "claimed_at_ms": 1710005600120u64,
                    "finished_at_ms": 1710005600480u64,
                    "error_message": null,
                    "created_at_ms": 1710005600000u64,
                    "updated_at_ms": 1710005600480u64
                }
            ],
            "assets": [
                {
                    "asset_id": "asset-local-demo-growth-brief-json",
                    "job_id": "job-local-demo-growth-brief",
                    "asset_kind": "json",
                    "storage_key": "tenant-1001/jobs/job-local-demo-growth-brief/output.json",
                    "download_url": "https://cdn.sdkwork.local/jobs/job-local-demo-growth-brief/output.json",
                    "mime_type": "application/json",
                    "size_bytes": 2048u64,
                    "checksum_sha256": "3edc1c0a6f2ff4f9d3f8f6a77658fb3b7fb3154d2f2df2fe7c8357e4fd16f9ce",
                    "created_at_ms": 1710005600490u64
                }
            ],
            "callbacks": [
                {
                    "callback_id": 9901u64,
                    "job_id": "job-local-demo-growth-brief",
                    "event_type": "job.completed",
                    "dedupe_key": "openrouter:or-job-local-demo-growth-brief:completed",
                    "payload_json": "{\"job_id\":\"job-local-demo-growth-brief\",\"status\":\"succeeded\"}",
                    "status": "processed",
                    "received_at_ms": 1710005600510u64,
                    "processed_at_ms": 1710005600530u64
                }
            ]
        }),
    );
    write_json(
        &root.join("jobs").join("dev.json"),
        &serde_json::json!({
            "jobs": [
                {
                    "job_id": "job-partner-sandbox-review",
                    "tenant_id": 1002u64,
                    "organization_id": 2002u64,
                    "user_id": 3002u64,
                    "account_id": null,
                    "request_id": 5002u64,
                    "provider_id": "provider-ollama-local",
                    "model_code": "llama3.2:latest",
                    "capability_code": "responses",
                    "modality": "text",
                    "operation_kind": "qa_review",
                    "status": "succeeded",
                    "external_job_id": "ollama-job-partner-sandbox-review",
                    "idempotency_key": "job:partner-sandbox:review",
                    "callback_url": "http://127.0.0.1:3000/api/dev/jobs/callbacks/partner",
                    "input_summary": "Run partner sandbox readiness review through the local-first dev stack",
                    "progress_percent": 100u64,
                    "error_code": null,
                    "error_message": null,
                    "created_at_ms": 1710006600000u64,
                    "updated_at_ms": 1710006600600u64,
                    "started_at_ms": 1710006600200u64,
                    "completed_at_ms": 1710006600600u64
                }
            ],
            "attempts": [
                {
                    "attempt_id": 8802u64,
                    "job_id": "job-partner-sandbox-review",
                    "attempt_number": 1u64,
                    "status": "succeeded",
                    "runtime_kind": "ollama",
                    "endpoint": "http://127.0.0.1:11434/api/generate",
                    "external_job_id": "ollama-job-partner-sandbox-review",
                    "claimed_at_ms": 1710006600220u64,
                    "finished_at_ms": 1710006600580u64,
                    "error_message": null,
                    "created_at_ms": 1710006600000u64,
                    "updated_at_ms": 1710006600580u64
                }
            ],
            "assets": [
                {
                    "asset_id": "asset-partner-sandbox-review-md",
                    "job_id": "job-partner-sandbox-review",
                    "asset_kind": "markdown",
                    "storage_key": "tenant-1002/jobs/job-partner-sandbox-review/report.md",
                    "download_url": "http://127.0.0.1:3000/artifacts/job-partner-sandbox-review/report.md",
                    "mime_type": "text/markdown",
                    "size_bytes": 1536u64,
                    "checksum_sha256": "96a758f2c50a94c24173f4e4bc52e13e98e374834f7f4952ed8f4b0603f42cfe",
                    "created_at_ms": 1710006600590u64
                }
            ],
            "callbacks": [
                {
                    "callback_id": 9902u64,
                    "job_id": "job-partner-sandbox-review",
                    "event_type": "job.completed",
                    "dedupe_key": "ollama:job-partner-sandbox-review:completed",
                    "payload_json": "{\"job_id\":\"job-partner-sandbox-review\",\"status\":\"succeeded\"}",
                    "status": "processed",
                    "received_at_ms": 1710006600610u64,
                    "processed_at_ms": 1710006600650u64
                }
            ]
        }),
    );
}

fn write_json(path: &PathBuf, value: &serde_json::Value) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

fn sqlite_url_for(path: PathBuf) -> String {
    let normalized = path.to_string_lossy().replace('\\', "/");
    if normalized.starts_with('/') {
        format!("sqlite://{normalized}")
    } else {
        format!("sqlite:///{normalized}")
    }
}

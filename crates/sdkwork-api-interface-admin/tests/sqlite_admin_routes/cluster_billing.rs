use super::support::*;

#[tokio::test]
async fn cluster_runtime_rollout_creation_snapshots_active_nodes() {
    let pool = memory_pool().await;
    let store = sdkwork_api_storage_sqlite::SqliteAdminStore::new(pool.clone());
    let now_ms = unix_timestamp_ms();
    store
        .upsert_service_runtime_node(&ServiceRuntimeNodeRecord::new(
            "gateway-node-a",
            "gateway",
            now_ms - 1_000,
        ))
        .await
        .unwrap();
    store
        .upsert_service_runtime_node(&ServiceRuntimeNodeRecord::new(
            "admin-node-a",
            "admin",
            now_ms - 1_000,
        ))
        .await
        .unwrap();
    store
        .upsert_service_runtime_node(
            &ServiceRuntimeNodeRecord::new("stale-gateway-node", "gateway", 0)
                .with_last_seen_at_ms(0),
        )
        .await
        .unwrap();

    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let create = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/extensions/runtime-rollouts")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "extension_id": FIXTURE_EXTENSION_ID,
                        "timeout_secs": 30,
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create.status(), StatusCode::CREATED);
    let create_json = read_json(create).await;
    assert_eq!(create_json["status"], "pending");
    assert_eq!(create_json["scope"], "extension");
    assert_eq!(create_json["requested_extension_id"], FIXTURE_EXTENSION_ID);
    assert_eq!(create_json["participant_count"], 2);
    assert_eq!(create_json["participants"].as_array().unwrap().len(), 2);
    assert_eq!(create_json["participants"][0]["node_id"], "admin-node-a");
    assert_eq!(create_json["participants"][1]["node_id"], "gateway-node-a");

    let rollout_id = create_json["rollout_id"].as_str().unwrap();
    let list = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/extensions/runtime-rollouts")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(list.status(), StatusCode::OK);
    let list_json = read_json(list).await;
    assert_eq!(list_json[0]["rollout_id"], rollout_id);
    assert_eq!(list_json[0]["participant_count"], 2);

    let get = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/admin/extensions/runtime-rollouts/{rollout_id}"))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(get.status(), StatusCode::OK);
    let get_json = read_json(get).await;
    assert_eq!(get_json["rollout_id"], rollout_id);
    assert_eq!(get_json["participant_count"], 2);
}

#[serial(extension_env)]
#[tokio::test]
async fn cluster_runtime_config_rollout_creation_snapshots_active_nodes() {
    let pool = memory_pool().await;
    let store = sdkwork_api_storage_sqlite::SqliteAdminStore::new(pool.clone());
    let now_ms = unix_timestamp_ms();
    store
        .upsert_service_runtime_node(&ServiceRuntimeNodeRecord::new(
            "gateway-node-a",
            "gateway",
            now_ms - 1_000,
        ))
        .await
        .unwrap();
    store
        .upsert_service_runtime_node(&ServiceRuntimeNodeRecord::new(
            "admin-node-a",
            "admin",
            now_ms - 1_000,
        ))
        .await
        .unwrap();
    store
        .upsert_service_runtime_node(&ServiceRuntimeNodeRecord::new(
            "portal-node-a",
            "portal",
            now_ms - 1_000,
        ))
        .await
        .unwrap();
    store
        .upsert_service_runtime_node(
            &ServiceRuntimeNodeRecord::new("stale-portal-node", "portal", 0)
                .with_last_seen_at_ms(0),
        )
        .await
        .unwrap();

    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let create = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/runtime-config/rollouts")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "service_kind": "portal",
                        "timeout_secs": 30,
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create.status(), StatusCode::CREATED);
    let create_json = read_json(create).await;
    assert_eq!(create_json["status"], "pending");
    assert_eq!(create_json["requested_service_kind"], "portal");
    assert_eq!(create_json["participant_count"], 1);
    assert_eq!(create_json["participants"].as_array().unwrap().len(), 1);
    assert_eq!(create_json["participants"][0]["node_id"], "portal-node-a");
    assert_eq!(create_json["participants"][0]["service_kind"], "portal");

    let rollout_id = create_json["rollout_id"].as_str().unwrap();
    let list = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/runtime-config/rollouts")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(list.status(), StatusCode::OK);
    let list_json = read_json(list).await;
    assert_eq!(list_json[0]["rollout_id"], rollout_id);
    assert_eq!(list_json[0]["participant_count"], 1);

    let get = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/admin/runtime-config/rollouts/{rollout_id}"))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(get.status(), StatusCode::OK);
    let get_json = read_json(get).await;
    assert_eq!(get_json["rollout_id"], rollout_id);
    assert_eq!(get_json["participant_count"], 1);
}

#[serial(extension_env)]
#[tokio::test]
async fn list_provider_health_snapshots_from_admin_api() {
    let pool = memory_pool().await;
    let store = sdkwork_api_storage_sqlite::SqliteAdminStore::new(pool.clone());
    store
        .insert_provider_health_snapshot(
            &sdkwork_api_domain_routing::ProviderHealthSnapshot::new(
                "provider-openai-official",
                "sdkwork.provider.openai.official",
                "builtin",
                1234,
            )
            .with_running(true)
            .with_healthy(true)
            .with_message("healthy"),
        )
        .await
        .unwrap();

    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/routing/health-snapshots")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json.as_array().unwrap().len(), 1);
    assert_eq!(json[0]["provider_id"], "provider-openai-official");
    assert_eq!(json[0]["healthy"], true);
    assert_eq!(json[0]["message"], "healthy");
}

#[serial(extension_env)]
#[tokio::test]
async fn usage_summary_from_admin_api_reports_grouped_counts() {
    let pool = memory_pool().await;
    let store = sdkwork_api_storage_sqlite::SqliteAdminStore::new(pool.clone());
    store
        .insert_usage_record(&sdkwork_api_domain_usage::UsageRecord::new(
            "project-1",
            "gpt-4.1",
            "provider-openai",
        ))
        .await
        .unwrap();
    store
        .insert_usage_record(&sdkwork_api_domain_usage::UsageRecord::new(
            "project-1",
            "gpt-4.1",
            "provider-openai",
        ))
        .await
        .unwrap();
    store
        .insert_usage_record(&sdkwork_api_domain_usage::UsageRecord::new(
            "project-2",
            "text-embedding-3-large",
            "provider-openrouter",
        ))
        .await
        .unwrap();

    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/usage/summary")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["total_requests"], 3);
    assert_eq!(json["project_count"], 2);
    assert_eq!(json["provider_count"], 2);
    assert_eq!(json["projects"][0]["project_id"], "project-1");
    assert_eq!(json["projects"][0]["request_count"], 2);
    assert_eq!(json["providers"][0]["provider"], "provider-openai");
    assert_eq!(json["providers"][0]["request_count"], 2);
    assert_eq!(json["models"][0]["model"], "gpt-4.1");
    assert_eq!(json["models"][0]["request_count"], 2);
}

#[serial(extension_env)]
#[tokio::test]
async fn billing_summary_from_admin_api_reports_quota_posture() {
    let pool = memory_pool().await;
    let store = sdkwork_api_storage_sqlite::SqliteAdminStore::new(pool.clone());
    store
        .insert_ledger_entry(&sdkwork_api_domain_billing::LedgerEntry::new(
            "project-1",
            70,
            0.70,
        ))
        .await
        .unwrap();
    store
        .insert_ledger_entry(&sdkwork_api_domain_billing::LedgerEntry::new(
            "project-1",
            40,
            0.40,
        ))
        .await
        .unwrap();
    store
        .insert_quota_policy(&sdkwork_api_domain_billing::QuotaPolicy::new(
            "quota-project-1",
            "project-1",
            100,
        ))
        .await
        .unwrap();

    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/billing/summary")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["total_entries"], 2);
    assert_eq!(json["total_units"], 110);
    assert_eq!(json["active_quota_policy_count"], 1);
    assert_eq!(json["exhausted_project_count"], 1);
    assert_eq!(json["projects"][0]["project_id"], "project-1");
    assert_eq!(json["projects"][0]["entry_count"], 2);
    assert_eq!(json["projects"][0]["used_units"], 110);
    assert_eq!(json["projects"][0]["quota_policy_id"], "quota-project-1");
    assert_eq!(json["projects"][0]["remaining_units"], 0);
    assert_eq!(json["projects"][0]["exhausted"], true);
}

#[serial(extension_env)]
#[tokio::test]
async fn billing_events_from_admin_api_report_group_and_capability_aggregates() {
    let pool = memory_pool().await;
    let store = sdkwork_api_storage_sqlite::SqliteAdminStore::new(pool.clone());
    store
        .insert_billing_event(
            &sdkwork_api_domain_billing::BillingEventRecord::new(
                "evt_1",
                "tenant-1",
                "project-1",
                "responses",
                "gpt-4.1",
                "gpt-4.1",
                "provider-openrouter",
                sdkwork_api_domain_billing::BillingAccountingMode::PlatformCredit,
                100,
            )
            .with_api_key_group_id("group-blue")
            .with_operation("responses.create", "text")
            .with_request_facts(Some("key-live"), Some("openai"), Some("resp_1"), Some(650))
            .with_units(240)
            .with_token_usage(120, 80, 200)
            .with_financials(0.42, 0.89)
            .with_routing_evidence(Some("route-profile-1"), Some("snapshot-1"), None),
        )
        .await
        .unwrap();
    store
        .insert_billing_event(
            &sdkwork_api_domain_billing::BillingEventRecord::new(
                "evt_2",
                "tenant-1",
                "project-1",
                "images",
                "gpt-image-1",
                "gpt-image-1",
                "provider-openai",
                sdkwork_api_domain_billing::BillingAccountingMode::PlatformCredit,
                200,
            )
            .with_api_key_group_id("group-blue")
            .with_operation("images.generate", "image")
            .with_request_facts(Some("key-live"), Some("openai"), Some("img_1"), Some(900))
            .with_units(40)
            .with_request_count(1)
            .with_media_usage(2, 0.0, 0.0, 0.0)
            .with_financials(0.80, 1.50)
            .with_routing_evidence(
                Some("route-profile-1"),
                Some("snapshot-2"),
                Some("provider_capacity"),
            ),
        )
        .await
        .unwrap();
    store
        .insert_billing_event(
            &sdkwork_api_domain_billing::BillingEventRecord::new(
                "evt_3",
                "tenant-1",
                "project-2",
                "audio",
                "gpt-4o-mini-transcribe",
                "gpt-4o-mini-transcribe",
                "provider-byok",
                sdkwork_api_domain_billing::BillingAccountingMode::Byok,
                300,
            )
            .with_operation("audio.transcriptions.create", "audio")
            .with_request_facts(Some("key-byok"), Some("openai"), Some("aud_1"), Some(1200))
            .with_units(60)
            .with_request_count(2)
            .with_media_usage(0, 35.0, 0.0, 0.0)
            .with_financials(0.0, 0.0),
        )
        .await
        .unwrap();

    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let events_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/billing/events")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(events_response.status(), StatusCode::OK);
    let events_json = read_json(events_response).await;
    assert_eq!(events_json.as_array().unwrap().len(), 3);
    assert_eq!(events_json[0]["event_id"], "evt_3");
    assert_eq!(events_json[1]["event_id"], "evt_2");
    assert_eq!(events_json[2]["event_id"], "evt_1");

    let summary_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/billing/events/summary")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(summary_response.status(), StatusCode::OK);
    let summary_json = read_json(summary_response).await;
    assert_eq!(summary_json["total_events"], 3);
    assert_eq!(summary_json["project_count"], 2);
    assert_eq!(summary_json["group_count"], 2);
    assert_eq!(summary_json["capability_count"], 3);
    assert_eq!(summary_json["total_request_count"], 4);
    assert_eq!(summary_json["total_units"], 340);
    assert_eq!(summary_json["total_tokens"], 200);
    assert_eq!(summary_json["total_image_count"], 2);
    assert_eq!(summary_json["total_audio_seconds"], 35.0);
    assert_eq!(summary_json["groups"][0]["api_key_group_id"], "group-blue");
    assert_eq!(summary_json["groups"][0]["event_count"], 2);
    assert_eq!(summary_json["capabilities"][0]["capability"], "audio");
    assert_eq!(summary_json["capabilities"][1]["capability"], "images");
    assert_eq!(summary_json["capabilities"][2]["capability"], "responses");
    assert_eq!(
        summary_json["accounting_modes"][0]["accounting_mode"],
        "platform_credit"
    );
    assert_eq!(
        summary_json["accounting_modes"][1]["accounting_mode"],
        "byok"
    );
}

#[serial(extension_env)]
#[tokio::test]
async fn create_and_list_quota_policies_from_admin_api() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let create = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/billing/quota-policies")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"policy_id\":\"quota-project-1\",\"project_id\":\"project-1\",\"max_units\":1000,\"enabled\":true}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create.status(), StatusCode::CREATED);

    let list = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/billing/quota-policies")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(list.status(), StatusCode::OK);
    let json = read_json(list).await;
    assert_eq!(json.as_array().unwrap().len(), 1);
    assert_eq!(json[0]["policy_id"], "quota-project-1");
    assert_eq!(json[0]["project_id"], "project-1");
    assert_eq!(json[0]["max_units"], 1000);
}


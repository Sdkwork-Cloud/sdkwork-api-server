use super::support::*;

#[tokio::test]
async fn routing_simulation_uses_catalog_models() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    create_provider_fixture(
        app.clone(),
        &token,
        r#"{"id":"provider-openrouter","channel_id":"openrouter","default_plugin_family":"openrouter","base_url":"https://openrouter.ai/api/v1","display_name":"OpenRouter","channel_bindings":[{"channel_id":"openrouter","is_primary":true}]}"#,
    )
    .await;
    create_provider_fixture(
        app.clone(),
        &token,
        r#"{"id":"provider-openai-official","channel_id":"openai","adapter_kind":"openai","base_url":"https://api.openai.com","display_name":"OpenAI Official","channel_bindings":[{"channel_id":"openai","is_primary":true}]}"#,
    )
    .await;

    let create_openrouter = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"external_name\":\"gpt-4.1\",\"provider_id\":\"provider-openrouter\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_openrouter.status(), StatusCode::CREATED);

    let create_openai = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"external_name\":\"gpt-4.1\",\"provider_id\":\"provider-openai-official\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_openai.status(), StatusCode::CREATED);

    let simulate = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/simulations")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"capability\":\"chat_completion\",\"model\":\"gpt-4.1\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(simulate.status(), StatusCode::OK);
    let simulation_json = read_json(simulate).await;
    assert_eq!(
        simulation_json["selected_provider_id"],
        "provider-openai-official"
    );
    assert_eq!(
        simulation_json["candidate_ids"].as_array().unwrap().len(),
        2
    );
    assert_eq!(simulation_json["strategy"], "deterministic_priority");
    assert!(simulation_json["selection_reason"].as_str().is_some());
    assert_eq!(simulation_json["assessments"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn create_and_list_routing_policies() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let create = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"policy_id\":\"policy-gpt-4-1\",\"capability\":\"chat_completion\",\"model_pattern\":\"gpt-4.1\",\"enabled\":true,\"priority\":100,\"strategy\":\"slo_aware\",\"ordered_provider_ids\":[\"provider-openrouter\",\"provider-openai-official\"],\"default_provider_id\":\"provider-openai-official\",\"max_cost\":0.25,\"max_latency_ms\":200,\"require_healthy\":true,\"execution_failover_enabled\":false,\"upstream_retry_max_attempts\":1,\"upstream_retry_base_delay_ms\":125,\"upstream_retry_max_delay_ms\":2500}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create.status(), StatusCode::CREATED);
    let created_json = read_json(create).await;
    assert_eq!(created_json["policy_id"], "policy-gpt-4-1");
    assert_eq!(created_json["priority"], 100);
    assert_eq!(created_json["strategy"], "slo_aware");
    assert_eq!(created_json["max_cost"], 0.25);
    assert_eq!(created_json["max_latency_ms"], 200);
    assert_eq!(created_json["require_healthy"], true);
    assert_eq!(created_json["execution_failover_enabled"], false);
    assert_eq!(created_json["upstream_retry_max_attempts"], 1);
    assert_eq!(created_json["upstream_retry_base_delay_ms"], 125);
    assert_eq!(created_json["upstream_retry_max_delay_ms"], 2500);

    let list = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(list.status(), StatusCode::OK);
    let list_json = read_json(list).await;
    assert_eq!(list_json[0]["policy_id"], "policy-gpt-4-1");
    assert_eq!(
        list_json[0]["ordered_provider_ids"][0],
        "provider-openrouter"
    );
    assert_eq!(
        list_json[0]["default_provider_id"],
        "provider-openai-official"
    );
    assert_eq!(list_json[0]["strategy"], "slo_aware");
    assert_eq!(list_json[0]["max_cost"], 0.25);
    assert_eq!(list_json[0]["max_latency_ms"], 200);
    assert_eq!(list_json[0]["require_healthy"], true);
    assert_eq!(list_json[0]["execution_failover_enabled"], false);
    assert_eq!(list_json[0]["upstream_retry_max_attempts"], 1);
    assert_eq!(list_json[0]["upstream_retry_base_delay_ms"], 125);
    assert_eq!(list_json[0]["upstream_retry_max_delay_ms"], 2500);
}
#[tokio::test]
async fn create_and_list_routing_profiles_and_apply_them_in_simulation() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let create_profile = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/profiles")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"profile_id\":\"profile-priority\",\"tenant_id\":\"tenant-1\",\"project_id\":\"project-1\",\"name\":\"Priority Live\",\"slug\":\"priority-live\",\"strategy\":\"geo_affinity\",\"ordered_provider_ids\":[\"provider-openrouter\",\"provider-openai-official\"],\"default_provider_id\":\"provider-openrouter\",\"preferred_region\":\"us-east\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_profile.status(), StatusCode::CREATED);
    let created_profile_json = read_json(create_profile).await;
    assert_eq!(created_profile_json["profile_id"], "profile-priority");
    assert_eq!(created_profile_json["project_id"], "project-1");
    assert_eq!(created_profile_json["strategy"], "geo_affinity");
    assert_eq!(created_profile_json["preferred_region"], "us-east");

    let list_profiles = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/routing/profiles")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(list_profiles.status(), StatusCode::OK);
    let profiles_json = read_json(list_profiles).await;
    assert_eq!(profiles_json[0]["profile_id"], "profile-priority");

    let create_group = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/api-key-groups")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"tenant_id\":\"tenant-1\",\"project_id\":\"project-1\",\"environment\":\"live\",\"name\":\"Production Keys\",\"slug\":\"production-keys\",\"default_routing_profile_id\":\"profile-priority\",\"default_accounting_mode\":\"byok\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_group.status(), StatusCode::CREATED);
    let created_group_json = read_json(create_group).await;
    let group_id = created_group_json["group_id"].as_str().unwrap().to_owned();
    assert_eq!(
        created_group_json["default_routing_profile_id"],
        "profile-priority"
    );
    assert_eq!(created_group_json["default_accounting_mode"], "byok");

    create_provider_fixture(
        app.clone(),
        &token,
        r#"{"id":"provider-openrouter","channel_id":"openrouter","default_plugin_family":"openrouter","base_url":"https://openrouter.ai/api/v1","display_name":"OpenRouter","channel_bindings":[{"channel_id":"openrouter","is_primary":true}]}"#,
    )
    .await;
    create_provider_fixture(
        app.clone(),
        &token,
        r#"{"id":"provider-openai-official","channel_id":"openai","adapter_kind":"openai","base_url":"https://api.openai.com","display_name":"OpenAI Official","channel_bindings":[{"channel_id":"openai","is_primary":true}]}"#,
    )
    .await;

    let create_openrouter_model = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"external_name\":\"gpt-4.1\",\"provider_id\":\"provider-openrouter\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_openrouter_model.status(), StatusCode::CREATED);

    let create_openai_model = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"external_name\":\"gpt-4.1\",\"provider_id\":\"provider-openai-official\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_openai_model.status(), StatusCode::CREATED);

    let simulate = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/simulations")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"capability\":\"chat_completion\",\"model\":\"gpt-4.1\",\"tenant_id\":\"tenant-1\",\"project_id\":\"project-1\",\"api_key_group_id\":\"{group_id}\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(simulate.status(), StatusCode::OK);
    let simulation_json = read_json(simulate).await;
    assert_eq!(
        simulation_json["selected_provider_id"],
        "provider-openrouter"
    );
    assert_eq!(
        simulation_json["applied_routing_profile_id"],
        "profile-priority"
    );
    assert!(simulation_json["compiled_routing_snapshot_id"]
        .as_str()
        .is_some());
    assert_eq!(simulation_json["requested_region"], "us-east");
    assert_eq!(simulation_json["strategy"], "geo_affinity");
    assert_eq!(
        simulation_json["selected_candidate"]["provider_id"],
        "provider-openrouter"
    );
    assert_eq!(
        simulation_json["rejected_candidates"][0]["provider_id"],
        "provider-openai-official"
    );

    let list_snapshots = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/routing/snapshots")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(list_snapshots.status(), StatusCode::OK);
    let snapshots_json = read_json(list_snapshots).await;
    assert_eq!(
        snapshots_json[0]["snapshot_id"],
        simulation_json["compiled_routing_snapshot_id"]
    );
    assert_eq!(
        snapshots_json[0]["applied_routing_profile_id"],
        "profile-priority"
    );
    assert_eq!(snapshots_json[0]["strategy"], "geo_affinity");
    assert_eq!(
        snapshots_json[0]["ordered_provider_ids"][0],
        "provider-openrouter"
    );
}

#[serial(extension_env)]
#[tokio::test]
async fn routing_simulation_persists_decision_log_and_lists_it() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"external_name\":\"gpt-4.1\",\"provider_id\":\"provider-openrouter\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let simulation = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/simulations")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"capability\":\"chat_completion\",\"model\":\"gpt-4.1\",\"selection_seed\":7}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(simulation.status(), StatusCode::OK);

    let logs = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/routing/decision-logs")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(logs.status(), StatusCode::OK);
    let logs_json = read_json(logs).await;
    assert_eq!(logs_json.as_array().unwrap().len(), 1);
    assert_eq!(logs_json[0]["decision_source"], "admin_simulation");
    assert_eq!(logs_json[0]["capability"], "chat_completion");
    assert_eq!(logs_json[0]["route_key"], "gpt-4.1");
    assert_eq!(logs_json[0]["selection_seed"], 7);
}

#[serial(extension_env)]
#[tokio::test]
async fn routing_simulation_reports_policy_selected_provider() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    create_provider_fixture(
        app.clone(),
        &token,
        r#"{"id":"provider-openrouter","channel_id":"openrouter","default_plugin_family":"openrouter","base_url":"https://openrouter.ai/api/v1","display_name":"OpenRouter","channel_bindings":[{"channel_id":"openrouter","is_primary":true}]}"#,
    )
    .await;
    create_provider_fixture(
        app.clone(),
        &token,
        r#"{"id":"provider-openai-official","channel_id":"openai","adapter_kind":"openai","base_url":"https://api.openai.com","display_name":"OpenAI Official","channel_bindings":[{"channel_id":"openai","is_primary":true}]}"#,
    )
    .await;

    let create_openrouter = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"external_name\":\"gpt-4.1\",\"provider_id\":\"provider-openrouter\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_openrouter.status(), StatusCode::CREATED);

    let create_openai = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"external_name\":\"gpt-4.1\",\"provider_id\":\"provider-openai-official\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_openai.status(), StatusCode::CREATED);

    let create_policy = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"policy_id\":\"policy-gpt-4-1\",\"capability\":\"chat_completion\",\"model_pattern\":\"gpt-4.1\",\"enabled\":true,\"priority\":100,\"strategy\":\"weighted_random\",\"ordered_provider_ids\":[\"provider-openrouter\",\"provider-openai-official\"],\"default_provider_id\":\"provider-openai-official\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_policy.status(), StatusCode::CREATED);

    let simulate = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/simulations")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"capability\":\"chat_completion\",\"model\":\"gpt-4.1\",\"selection_seed\":11}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(simulate.status(), StatusCode::OK);
    let simulation_json = read_json(simulate).await;
    assert_eq!(
        simulation_json["selected_provider_id"],
        "provider-openrouter"
    );
    assert_eq!(simulation_json["matched_policy_id"], "policy-gpt-4-1");
    assert_eq!(simulation_json["strategy"], "weighted_random");
    assert_eq!(simulation_json["selection_seed"], 11);
    assert!(simulation_json["selection_reason"].as_str().is_some());
    assert_eq!(
        simulation_json["assessments"][0]["provider_id"],
        "provider-openrouter"
    );
    assert!(simulation_json["assessments"][0]["reasons"]
        .as_array()
        .is_some());
}

#[serial(extension_env)]
#[tokio::test]
async fn routing_simulation_accepts_requested_region_and_persists_logs() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let create_channel = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/channels")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"id\":\"geo-openai\",\"name\":\"Geo OpenAI\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_channel.status(), StatusCode::CREATED);

    let create_eu_provider = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"id\":\"provider-eu-west\",\"channel_id\":\"geo-openai\",\"extension_id\":\"sdkwork.provider.openai.official\",\"adapter_kind\":\"openai\",\"base_url\":\"https://eu-west.example/v1\",\"display_name\":\"EU West Provider\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_eu_provider.status(), StatusCode::CREATED);

    let create_us_provider = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"id\":\"provider-us-east\",\"channel_id\":\"geo-openai\",\"extension_id\":\"sdkwork.provider.openai.official\",\"adapter_kind\":\"openai\",\"base_url\":\"https://us-east.example/v1\",\"display_name\":\"US East Provider\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_us_provider.status(), StatusCode::CREATED);

    let create_eu_model = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"external_name\":\"gpt-4.1\",\"provider_id\":\"provider-eu-west\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_eu_model.status(), StatusCode::CREATED);

    let create_us_model = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"external_name\":\"gpt-4.1\",\"provider_id\":\"provider-us-east\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_us_model.status(), StatusCode::CREATED);

    let openrouter_installation = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/extensions/installations")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"installation_id\":\"geo-eu-installation\",\"extension_id\":\"sdkwork.provider.openai.official\",\"runtime\":\"builtin\",\"enabled\":true,\"entrypoint\":null,\"config\":{}}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(openrouter_installation.status(), StatusCode::CREATED);

    let openai_installation = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/extensions/installations")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"installation_id\":\"geo-us-installation\",\"extension_id\":\"sdkwork.provider.openai.official\",\"runtime\":\"builtin\",\"enabled\":true,\"entrypoint\":null,\"config\":{}}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(openai_installation.status(), StatusCode::CREATED);

    let openrouter_instance = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/extensions/instances")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"instance_id\":\"provider-eu-west\",\"installation_id\":\"geo-eu-installation\",\"extension_id\":\"sdkwork.provider.openai.official\",\"enabled\":true,\"base_url\":\"https://eu-west.example/v1\",\"credential_ref\":null,\"config\":{\"region\":\"eu-west\"}}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(openrouter_instance.status(), StatusCode::CREATED);

    let openai_instance = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/extensions/instances")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"instance_id\":\"provider-us-east\",\"installation_id\":\"geo-us-installation\",\"extension_id\":\"sdkwork.provider.openai.official\",\"enabled\":true,\"base_url\":\"https://us-east.example/v1\",\"credential_ref\":null,\"config\":{\"routing\":{\"region\":\"us-east\"}}}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(openai_instance.status(), StatusCode::CREATED);

    let create_policy = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"policy_id\":\"policy-geo-affinity\",\"capability\":\"chat_completion\",\"model_pattern\":\"gpt-4.1\",\"enabled\":true,\"priority\":100,\"strategy\":\"geo_affinity\",\"ordered_provider_ids\":[\"provider-eu-west\",\"provider-us-east\"]}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_policy.status(), StatusCode::CREATED);

    let simulation = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/simulations")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"capability\":\"chat_completion\",\"model\":\"gpt-4.1\",\"requested_region\":\"us-east\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(simulation.status(), StatusCode::OK);
    let simulation_json = read_json(simulation).await;
    assert_eq!(simulation_json["selected_provider_id"], "provider-us-east");
    assert_eq!(simulation_json["strategy"], "geo_affinity");
    assert_eq!(simulation_json["requested_region"], "us-east");
    assert_eq!(simulation_json["assessments"][0]["region"], "eu-west");
    assert_eq!(simulation_json["assessments"][0]["region_match"], false);
    assert_eq!(simulation_json["assessments"][1]["region"], "us-east");
    assert_eq!(simulation_json["assessments"][1]["region_match"], true);
    assert!(simulation_json["fallback_reason"].is_null());

    let logs = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/routing/decision-logs")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(logs.status(), StatusCode::OK);
    let logs_json = read_json(logs).await;
    assert_eq!(logs_json[0]["requested_region"], "us-east");

    let degraded_simulation = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/simulations")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"capability\":\"chat_completion\",\"model\":\"gpt-4.1\",\"requested_region\":\"ap-south\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(degraded_simulation.status(), StatusCode::OK);
    let degraded_simulation_json = read_json(degraded_simulation).await;
    assert_eq!(
        degraded_simulation_json["selected_provider_id"],
        "provider-eu-west"
    );
    assert_eq!(
        degraded_simulation_json["selected_candidate"]["provider_id"],
        "provider-eu-west"
    );
    assert_eq!(
        degraded_simulation_json["rejected_candidates"][0]["provider_id"],
        "provider-us-east"
    );
    assert!(degraded_simulation_json["fallback_reason"]
        .as_str()
        .unwrap()
        .contains("no candidate matched requested region ap-south"));
}

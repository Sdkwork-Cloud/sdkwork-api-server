use super::*;

#[tokio::test]
async fn standalone_runtime_supervision_auto_activates_due_planned_pricing_versions_in_background()
{
    let config_root = temp_root("runtime-pricing-lifecycle-activation");
    let database_path = config_root.join("pricing.db");
    let database_url = sqlite_url_for_path(&database_path);
    write_admin_pricing_runtime_config(&config_root, &database_url, "admin-secret-initial", 1);

    let (loader, initial_config) = StandaloneConfigLoader::from_local_root_and_pairs(
        &config_root,
        std::iter::empty::<(&str, &str)>(),
    )
    .unwrap();
    initial_config.apply_to_process_env();

    let pool = run_migrations(&database_url).await.unwrap();
    let store = Arc::new(SqliteAdminStore::new(pool));

    let active_plan = PricingPlanRecord::new(9101, 1001, 2002, "retail-pro", 1)
        .with_display_name("Retail Pro")
        .with_currency_code("USD")
        .with_credit_unit_code("credit")
        .with_status("active")
        .with_effective_from_ms(1)
        .with_created_at_ms(15)
        .with_updated_at_ms(15);
    let due_planned_plan = PricingPlanRecord::new(9102, 1001, 2002, "retail-pro", 2)
        .with_display_name("Retail Pro v2")
        .with_currency_code("USD")
        .with_credit_unit_code("credit")
        .with_status("planned")
        .with_effective_from_ms(2)
        .with_created_at_ms(16)
        .with_updated_at_ms(16);
    let future_planned_plan = PricingPlanRecord::new(9103, 1001, 2002, "retail-pro", 3)
        .with_display_name("Retail Pro Future")
        .with_currency_code("USD")
        .with_credit_unit_code("credit")
        .with_status("planned")
        .with_effective_from_ms(4_102_444_800_000)
        .with_created_at_ms(17)
        .with_updated_at_ms(17);
    let active_rate = PricingRateRecord::new(9201, 1001, 2002, 9101, "token.input")
        .with_charge_unit("input_token")
        .with_pricing_method("per_unit")
        .with_quantity_step(1000000.0)
        .with_unit_price(2.5)
        .with_display_price_unit("USD / 1M input tokens")
        .with_rounding_increment(1.0)
        .with_rounding_mode("ceil")
        .with_status("active")
        .with_created_at_ms(18)
        .with_updated_at_ms(18);
    let due_planned_rate = PricingRateRecord::new(9202, 1001, 2002, 9102, "token.input")
        .with_charge_unit("input_token")
        .with_pricing_method("per_unit")
        .with_quantity_step(1000000.0)
        .with_unit_price(2.8)
        .with_display_price_unit("USD / 1M input tokens")
        .with_rounding_increment(1.0)
        .with_rounding_mode("ceil")
        .with_status("planned")
        .with_created_at_ms(19)
        .with_updated_at_ms(19);
    let future_planned_rate = PricingRateRecord::new(9203, 1001, 2002, 9103, "token.input")
        .with_charge_unit("input_token")
        .with_pricing_method("per_unit")
        .with_quantity_step(1000000.0)
        .with_unit_price(3.1)
        .with_display_price_unit("USD / 1M input tokens")
        .with_rounding_increment(1.0)
        .with_rounding_mode("ceil")
        .with_status("planned")
        .with_created_at_ms(20)
        .with_updated_at_ms(20);

    store
        .insert_pricing_plan_record(&active_plan)
        .await
        .unwrap();
    store
        .insert_pricing_plan_record(&due_planned_plan)
        .await
        .unwrap();
    store
        .insert_pricing_plan_record(&future_planned_plan)
        .await
        .unwrap();
    store
        .insert_pricing_rate_record(&active_rate)
        .await
        .unwrap();
    store
        .insert_pricing_rate_record(&due_planned_rate)
        .await
        .unwrap();
    store
        .insert_pricing_rate_record(&future_planned_rate)
        .await
        .unwrap();

    let live_store = Reloadable::new(store.clone() as Arc<dyn AdminStore>);
    let live_commercial_billing = Reloadable::new(
        store.clone() as Arc<dyn sdkwork_api_app_billing::CommercialBillingAdminKernel>
    );
    let live_admin_jwt = Reloadable::new("admin-secret-initial".to_owned());
    let supervision = start_standalone_runtime_supervision(
        StandaloneServiceKind::Admin,
        loader,
        initial_config,
        StandaloneServiceReloadHandles::admin(live_store, live_admin_jwt)
            .with_live_commercial_billing(live_commercial_billing),
    );

    wait_for_pricing_plan_status(store.as_ref(), 9101, "archived").await;
    wait_for_pricing_plan_status(store.as_ref(), 9102, "active").await;
    wait_for_pricing_plan_status(store.as_ref(), 9103, "planned").await;
    wait_for_pricing_rate_status(store.as_ref(), 9201, "archived").await;
    wait_for_pricing_rate_status(store.as_ref(), 9202, "active").await;
    wait_for_pricing_rate_status(store.as_ref(), 9203, "planned").await;

    drop(supervision);
    cleanup_dir(&config_root);
}
#[tokio::test]
async fn standalone_runtime_supervision_reloads_secret_manager_after_config_file_change() {
    let config_root = temp_root("runtime-secret-manager-reload");
    let initial_secret_file = config_root.join("secrets-initial.json");
    let rotated_secret_file = config_root.join("secrets-rotated.json");
    write_gateway_secret_manager_runtime_config(
        &config_root,
        &initial_secret_file,
        "initial-master-key",
        &[],
    );

    let (loader, initial_config) = StandaloneConfigLoader::from_local_root_and_pairs(
        &config_root,
        std::iter::empty::<(&str, &str)>(),
    )
    .unwrap();
    initial_config.apply_to_process_env();

    let live_store = Reloadable::new(empty_store().await);
    let live_secret_manager =
        Reloadable::new(CredentialSecretManager::new_with_legacy_master_keys(
            sdkwork_api_secret_core::SecretBackendKind::LocalEncryptedFile,
            "initial-master-key",
            Vec::new(),
            &initial_secret_file,
            "sdkwork-api-server",
        ));
    persist_credential_with_secret_and_manager(
        live_store.snapshot().as_ref(),
        &live_secret_manager.snapshot(),
        "tenant-1",
        "provider-openai-official",
        "cred-openai",
        "sk-upstream-openai",
    )
    .await
    .unwrap();

    let supervision = start_standalone_runtime_supervision(
        StandaloneServiceKind::Gateway,
        loader,
        initial_config,
        StandaloneServiceReloadHandles::gateway(live_store.clone())
            .with_secret_manager(live_secret_manager.clone()),
    );

    write_gateway_secret_manager_runtime_config(
        &config_root,
        &rotated_secret_file,
        "rotated-master-key",
        &["initial-master-key"],
    );

    wait_for_secret_manager_master_key(&live_secret_manager, "rotated-master-key").await;
    let resolved = resolve_provider_secret_with_manager(
        live_store.snapshot().as_ref(),
        &live_secret_manager.snapshot(),
        "tenant-1",
        "provider-openai-official",
    )
    .await
    .unwrap();
    assert_eq!(resolved.as_deref(), Some("sk-upstream-openai"));

    drop(supervision);
    cleanup_dir(&config_root);
}

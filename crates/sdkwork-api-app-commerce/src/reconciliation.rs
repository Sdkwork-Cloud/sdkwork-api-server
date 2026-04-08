use crate::constants::COMMERCE_PAYMENT_PROVIDER_STRIPE;
use crate::error::{CommerceError, CommerceResult};
use crate::payment_provider::resolve_payment_method_secret_bundle;
use crate::payment_provider::stripe;
use crate::types::AdminCommerceReconciliationRunCreateRequest;
use reqwest::Client;
use sdkwork_api_app_credential::CredentialSecretManager;
use sdkwork_api_domain_commerce::{
    CommercePaymentAttemptRecord, CommerceReconciliationItemRecord,
    CommerceReconciliationRunRecord, CommerceRefundRecord, CommerceWebhookDeliveryAttemptRecord,
    CommerceWebhookInboxRecord,
};
use sdkwork_api_storage_core::AdminStore;
use std::collections::BTreeMap;

pub async fn create_admin_commerce_reconciliation_run(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    request: &AdminCommerceReconciliationRunCreateRequest,
) -> CommerceResult<CommerceReconciliationRunRecord> {
    let provider = request.provider.trim().to_ascii_lowercase();
    if provider.is_empty() {
        return Err(CommerceError::InvalidInput(
            "provider is required".to_owned(),
        ));
    }
    if request.scope_started_at_ms > request.scope_ended_at_ms {
        return Err(CommerceError::InvalidInput(
            "scope_started_at_ms must be less than or equal to scope_ended_at_ms".to_owned(),
        ));
    }
    if provider != COMMERCE_PAYMENT_PROVIDER_STRIPE {
        return Err(CommerceError::InvalidInput(format!(
            "reconciliation is not yet wired for provider {}",
            request.provider
        )));
    }

    let created_at_ms = crate::current_time_ms()?;
    let reconciliation_run_id = crate::generate_entity_id("commerce_reconciliation_run")?;
    let mut run = CommerceReconciliationRunRecord::new(
        reconciliation_run_id,
        provider.clone(),
        request.scope_started_at_ms,
        request.scope_ended_at_ms,
        created_at_ms,
    )
    .with_payment_method_id_option(request.payment_method_id.clone())
    .with_status("running")
    .with_updated_at_ms(created_at_ms);
    run = store
        .insert_commerce_reconciliation_run(&run)
        .await
        .map_err(CommerceError::from)?;

    let mut discrepancy_count = 0u64;
    let mut checked_attempts = 0u64;
    let mut checked_refunds = 0u64;
    let mut secret_cache = BTreeMap::<String, String>::new();
    let client = Client::new();

    let attempts = store
        .list_commerce_payment_attempts()
        .await
        .map_err(CommerceError::from)?
        .into_iter()
        .filter(|attempt| attempt.provider == provider)
        .filter(|attempt| {
            request
                .payment_method_id
                .as_deref()
                .map(|payment_method_id| attempt.payment_method_id == payment_method_id)
                .unwrap_or(true)
        })
        .filter(|attempt| {
            attempt.initiated_at_ms >= request.scope_started_at_ms
                && attempt.initiated_at_ms <= request.scope_ended_at_ms
        })
        .collect::<Vec<_>>();
    for attempt in attempts {
        checked_attempts = checked_attempts.saturating_add(1);
        reconcile_attempt(
            store,
            secret_manager,
            &client,
            &run,
            &attempt,
            &mut secret_cache,
            &mut discrepancy_count,
        )
        .await?;
    }

    let refunds = store
        .list_commerce_refunds()
        .await
        .map_err(CommerceError::from)?
        .into_iter()
        .filter(|refund| refund.provider == provider)
        .filter(|refund| {
            request
                .payment_method_id
                .as_deref()
                .map(|payment_method_id| {
                    refund.payment_method_id.as_deref() == Some(payment_method_id)
                })
                .unwrap_or(true)
        })
        .filter(|refund| {
            refund.created_at_ms >= request.scope_started_at_ms
                && refund.created_at_ms <= request.scope_ended_at_ms
        })
        .collect::<Vec<_>>();
    for refund in refunds {
        checked_refunds = checked_refunds.saturating_add(1);
        reconcile_refund(
            store,
            secret_manager,
            &client,
            &run,
            &refund,
            &mut secret_cache,
            &mut discrepancy_count,
        )
        .await?;
    }

    run = run
        .with_status(if discrepancy_count == 0 {
            "completed"
        } else {
            "completed_with_diff"
        })
        .with_summary_json(
            serde_json::json!({
                "checked_attempts": checked_attempts,
                "checked_refunds": checked_refunds,
                "discrepancy_count": discrepancy_count,
            })
            .to_string(),
        )
        .with_updated_at_ms(crate::current_time_ms()?)
        .with_completed_at_ms_option(Some(crate::current_time_ms()?));
    store
        .insert_commerce_reconciliation_run(&run)
        .await
        .map_err(CommerceError::from)
}

pub async fn list_admin_commerce_webhook_inbox(
    store: &dyn AdminStore,
) -> CommerceResult<Vec<CommerceWebhookInboxRecord>> {
    let mut records = store
        .list_commerce_webhook_inbox_records()
        .await
        .map_err(CommerceError::from)?;
    records.sort_by(|left, right| {
        right
            .last_received_at_ms
            .cmp(&left.last_received_at_ms)
            .then_with(|| right.webhook_inbox_id.cmp(&left.webhook_inbox_id))
    });
    Ok(records)
}

pub async fn list_admin_commerce_webhook_delivery_attempts(
    store: &dyn AdminStore,
    webhook_inbox_id: &str,
) -> CommerceResult<Vec<CommerceWebhookDeliveryAttemptRecord>> {
    store
        .list_commerce_webhook_delivery_attempts(webhook_inbox_id)
        .await
        .map_err(CommerceError::from)
}

pub async fn list_admin_commerce_reconciliation_runs(
    store: &dyn AdminStore,
) -> CommerceResult<Vec<CommerceReconciliationRunRecord>> {
    let mut runs = store
        .list_commerce_reconciliation_runs()
        .await
        .map_err(CommerceError::from)?;
    runs.sort_by(|left, right| {
        right
            .created_at_ms
            .cmp(&left.created_at_ms)
            .then_with(|| right.reconciliation_run_id.cmp(&left.reconciliation_run_id))
    });
    Ok(runs)
}

pub async fn list_admin_commerce_reconciliation_items(
    store: &dyn AdminStore,
    reconciliation_run_id: &str,
) -> CommerceResult<Vec<CommerceReconciliationItemRecord>> {
    let mut items = store
        .list_commerce_reconciliation_items(reconciliation_run_id)
        .await
        .map_err(CommerceError::from)?;
    items.sort_by(|left, right| {
        right.created_at_ms.cmp(&left.created_at_ms).then_with(|| {
            right
                .reconciliation_item_id
                .cmp(&left.reconciliation_item_id)
        })
    });
    Ok(items)
}

async fn reconcile_attempt(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    client: &Client,
    run: &CommerceReconciliationRunRecord,
    attempt: &CommercePaymentAttemptRecord,
    secret_cache: &mut BTreeMap<String, String>,
    discrepancy_count: &mut u64,
) -> CommerceResult<()> {
    let Some(provider_checkout_session_id) = attempt.provider_checkout_session_id.as_deref() else {
        insert_reconciliation_item(
            store,
            run,
            "missing_checkout_session",
            attempt.amount_minor as i64,
            None,
            Some(attempt.order_id.clone()),
            Some(attempt.payment_attempt_id.clone()),
            None,
            None,
            serde_json::json!({
                "message": "payment attempt is missing provider_checkout_session_id",
                "payment_attempt_id": attempt.payment_attempt_id,
            }),
        )
        .await?;
        *discrepancy_count = discrepancy_count.saturating_add(1);
        return Ok(());
    };

    let api_secret = load_reconciliation_secret(
        store,
        secret_manager,
        secret_cache,
        &attempt.payment_method_id,
    )
    .await?;
    match stripe::retrieve_checkout_session(client, &api_secret, provider_checkout_session_id).await
    {
        Ok(provider_session) => {
            if provider_session.amount_total_minor != Some(attempt.amount_minor) {
                insert_reconciliation_item(
                    store,
                    run,
                    "amount_mismatch",
                    attempt.amount_minor as i64,
                    provider_session
                        .amount_total_minor
                        .map(|value| value as i64),
                    Some(attempt.order_id.clone()),
                    Some(attempt.payment_attempt_id.clone()),
                    None,
                    Some(provider_checkout_session_id.to_owned()),
                    serde_json::json!({
                        "local_status": attempt.status,
                        "provider_status": provider_session.status,
                        "provider_payment_status": provider_session.payment_status,
                        "provider_payload_json": provider_session.response_payload_json,
                    }),
                )
                .await?;
                *discrepancy_count = discrepancy_count.saturating_add(1);
            }
            if provider_session.payment_status.as_deref() == Some("paid")
                && !matches!(
                    attempt.status.as_str(),
                    "succeeded" | "partially_refunded" | "refunded"
                )
            {
                insert_reconciliation_item(
                    store,
                    run,
                    "status_mismatch",
                    attempt.amount_minor as i64,
                    provider_session
                        .amount_total_minor
                        .map(|value| value as i64),
                    Some(attempt.order_id.clone()),
                    Some(attempt.payment_attempt_id.clone()),
                    None,
                    Some(provider_checkout_session_id.to_owned()),
                    serde_json::json!({
                        "local_status": attempt.status,
                        "provider_status": provider_session.status,
                        "provider_payment_status": provider_session.payment_status,
                    }),
                )
                .await?;
                *discrepancy_count = discrepancy_count.saturating_add(1);
            }
        }
        Err(error) => {
            insert_reconciliation_item(
                store,
                run,
                "provider_fetch_error",
                attempt.amount_minor as i64,
                None,
                Some(attempt.order_id.clone()),
                Some(attempt.payment_attempt_id.clone()),
                None,
                Some(provider_checkout_session_id.to_owned()),
                serde_json::json!({
                    "message": error.to_string(),
                }),
            )
            .await?;
            *discrepancy_count = discrepancy_count.saturating_add(1);
        }
    }

    Ok(())
}

async fn reconcile_refund(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    client: &Client,
    run: &CommerceReconciliationRunRecord,
    refund: &CommerceRefundRecord,
    secret_cache: &mut BTreeMap<String, String>,
    discrepancy_count: &mut u64,
) -> CommerceResult<()> {
    let Some(provider_refund_id) = refund.provider_refund_id.as_deref() else {
        insert_reconciliation_item(
            store,
            run,
            "missing_provider_refund",
            refund.amount_minor as i64,
            None,
            Some(refund.order_id.clone()),
            refund.payment_attempt_id.clone(),
            Some(refund.refund_id.clone()),
            None,
            serde_json::json!({
                "message": "refund is missing provider_refund_id",
                "refund_id": refund.refund_id,
            }),
        )
        .await?;
        *discrepancy_count = discrepancy_count.saturating_add(1);
        return Ok(());
    };

    let payment_method_id = refund.payment_method_id.as_deref().ok_or_else(|| {
        CommerceError::Conflict(format!(
            "refund {} is missing payment_method_id for reconciliation",
            refund.refund_id
        ))
    })?;
    let api_secret =
        load_reconciliation_secret(store, secret_manager, secret_cache, payment_method_id).await?;
    match stripe::retrieve_refund(client, &api_secret, provider_refund_id).await {
        Ok(provider_refund) => {
            let normalized_provider_status =
                stripe::normalize_refund_status(&provider_refund.status).to_owned();
            if provider_refund.amount_minor != refund.amount_minor
                || normalized_provider_status != refund.status
            {
                insert_reconciliation_item(
                    store,
                    run,
                    "refund_mismatch",
                    refund.amount_minor as i64,
                    Some(provider_refund.amount_minor as i64),
                    Some(refund.order_id.clone()),
                    refund.payment_attempt_id.clone(),
                    Some(refund.refund_id.clone()),
                    Some(provider_refund_id.to_owned()),
                    serde_json::json!({
                        "local_status": refund.status,
                        "provider_status": provider_refund.status,
                        "provider_payload_json": provider_refund.response_payload_json,
                    }),
                )
                .await?;
                *discrepancy_count = discrepancy_count.saturating_add(1);
            }
        }
        Err(error) => {
            insert_reconciliation_item(
                store,
                run,
                "provider_fetch_error",
                refund.amount_minor as i64,
                None,
                Some(refund.order_id.clone()),
                refund.payment_attempt_id.clone(),
                Some(refund.refund_id.clone()),
                Some(provider_refund_id.to_owned()),
                serde_json::json!({
                    "message": error.to_string(),
                }),
            )
            .await?;
            *discrepancy_count = discrepancy_count.saturating_add(1);
        }
    }

    Ok(())
}

async fn insert_reconciliation_item(
    store: &dyn AdminStore,
    run: &CommerceReconciliationRunRecord,
    discrepancy_type: &str,
    expected_amount_minor: i64,
    provider_amount_minor: Option<i64>,
    order_id: Option<String>,
    payment_attempt_id: Option<String>,
    refund_id: Option<String>,
    external_reference: Option<String>,
    detail_json: serde_json::Value,
) -> CommerceResult<CommerceReconciliationItemRecord> {
    let created_at_ms = crate::current_time_ms()?;
    let item = CommerceReconciliationItemRecord::new(
        crate::generate_entity_id("commerce_reconciliation_item")?,
        run.reconciliation_run_id.clone(),
        discrepancy_type,
        expected_amount_minor,
        created_at_ms,
    )
    .with_provider_amount_minor_option(provider_amount_minor)
    .with_order_id_option(order_id)
    .with_payment_attempt_id_option(payment_attempt_id)
    .with_refund_id_option(refund_id)
    .with_external_reference_option(external_reference)
    .with_detail_json(detail_json.to_string())
    .with_updated_at_ms(created_at_ms);
    store
        .insert_commerce_reconciliation_item(&item)
        .await
        .map_err(CommerceError::from)
}

async fn load_reconciliation_secret(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    secret_cache: &mut BTreeMap<String, String>,
    payment_method_id: &str,
) -> CommerceResult<String> {
    if let Some(secret) = secret_cache.get(payment_method_id) {
        return Ok(secret.clone());
    }

    let secrets =
        resolve_payment_method_secret_bundle(store, secret_manager, payment_method_id).await?;
    let secret = secrets
        .reconciliation_secret
        .or(Some(secrets.api_secret))
        .ok_or_else(|| {
            CommerceError::Conflict(format!(
                "payment method {} is missing reconciliation secret",
                payment_method_id
            ))
        })?;
    secret_cache.insert(payment_method_id.to_owned(), secret.clone());
    Ok(secret)
}

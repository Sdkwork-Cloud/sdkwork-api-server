use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use sdkwork_api_app_credential::{
    persist_credential_with_secret_and_manager, CredentialSecretManager,
};
use sdkwork_api_domain_commerce::{
    CommercePaymentAttemptRecord, PaymentMethodCredentialBindingRecord, PaymentMethodRecord,
};
use sdkwork_api_storage_core::AdminStore;
use sdkwork_api_storage_sqlite::SqliteAdminStore;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use std::time::{SystemTime, UNIX_EPOCH};
use tower::ServiceExt;

async fn read_json(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

async fn memory_pool() -> SqlitePool {
    sdkwork_api_storage_sqlite::run_migrations("sqlite::memory:")
        .await
        .unwrap()
}

async fn register_portal_user(app: axum::Router) -> Value {
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"email":"payments@example.com","password":"PortalPass123!","display_name":"Payment User"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    read_json(response).await
}

async fn portal_workspace(app: axum::Router, token: &str) -> Value {
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/workspace")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    read_json(response).await
}

async fn seed_portal_recharge_capacity_fixture(pool: &SqlitePool, project_id: &str) {
    sqlx::query(
        "INSERT INTO ai_billing_ledger_entries (project_id, units, amount) VALUES (?, ?, ?)",
    )
    .bind(project_id)
    .bind(240_i64)
    .bind(0.42_f64)
    .execute(pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO ai_billing_quota_policies (policy_id, project_id, max_units, enabled)
         VALUES (?, ?, ?, ?)",
    )
    .bind("quota-portal")
    .bind(project_id)
    .bind(500_i64)
    .bind(1_i64)
    .execute(pool)
    .await
    .unwrap();
}

fn current_timestamp_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn hmac_sha256_hex(key: &[u8], message: &[u8]) -> String {
    let block_size = 64;
    let mut normalized_key = if key.len() > block_size {
        Sha256::digest(key).to_vec()
    } else {
        key.to_vec()
    };
    normalized_key.resize(block_size, 0);

    let mut outer = vec![0x5c; block_size];
    let mut inner = vec![0x36; block_size];
    for (index, key_byte) in normalized_key.iter().enumerate() {
        outer[index] ^= key_byte;
        inner[index] ^= key_byte;
    }

    let mut inner_hasher = Sha256::new();
    inner_hasher.update(&inner);
    inner_hasher.update(message);
    let inner_hash = inner_hasher.finalize();

    let mut outer_hasher = Sha256::new();
    outer_hasher.update(&outer);
    outer_hasher.update(inner_hash);
    outer_hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn stripe_signature_header(payload: &str, secret: &str, timestamp: u64) -> String {
    let signed_payload = format!("{timestamp}.{payload}");
    let signature = hmac_sha256_hex(secret.as_bytes(), signed_payload.as_bytes());
    format!("t={timestamp},v1={signature}")
}

async fn seed_stripe_payment_method(
    store: &SqliteAdminStore,
    tenant_id: &str,
    payment_method_id: &str,
) {
    let payment_method = PaymentMethodRecord::new(
        payment_method_id,
        "Stripe Checkout",
        "stripe",
        "hosted_checkout",
        1,
    )
    .with_description("Hosted card and wallet checkout")
    .with_supported_currency_codes(vec!["USD".to_owned()])
    .with_supported_order_kinds(vec!["recharge_pack".to_owned()])
    .with_webhook_path_option(Some(format!(
        "/portal/commerce/webhooks/stripe/{payment_method_id}"
    )))
    .with_max_retry_count(4);
    store.upsert_payment_method(&payment_method).await.unwrap();

    let secret_manager = CredentialSecretManager::database_encrypted("local-dev-master-key");
    persist_credential_with_secret_and_manager(
        store,
        &secret_manager,
        tenant_id,
        "stripe",
        "api-secret",
        "sk_test_stripe",
    )
    .await
    .unwrap();
    persist_credential_with_secret_and_manager(
        store,
        &secret_manager,
        tenant_id,
        "stripe",
        "webhook-secret",
        "whsec_test_stripe",
    )
    .await
    .unwrap();

    let api_binding = PaymentMethodCredentialBindingRecord::new(
        "binding_stripe_api_secret",
        payment_method_id,
        "api_secret",
        tenant_id,
        "stripe",
        "api-secret",
        1,
    );
    store
        .upsert_payment_method_credential_binding(&api_binding)
        .await
        .unwrap();

    let webhook_binding = PaymentMethodCredentialBindingRecord::new(
        "binding_stripe_webhook_secret",
        payment_method_id,
        "webhook_secret",
        tenant_id,
        "stripe",
        "webhook-secret",
        1,
    );
    store
        .upsert_payment_method_credential_binding(&webhook_binding)
        .await
        .unwrap();
}

#[tokio::test]
async fn portal_stripe_webhook_processes_configured_payment_method_and_is_idempotent() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool.clone());

    let registration = register_portal_user(app.clone()).await;
    let token = registration["token"].as_str().unwrap().to_owned();
    let workspace = portal_workspace(app.clone(), &token).await;
    let tenant_id = workspace["tenant"]["id"].as_str().unwrap().to_owned();
    let project_id = workspace["project"]["id"].as_str().unwrap().to_owned();
    let user_id = workspace["user"]["id"].as_str().unwrap().to_owned();

    seed_portal_recharge_capacity_fixture(&pool, &project_id).await;
    seed_stripe_payment_method(&store, &tenant_id, "pm_stripe_checkout").await;

    let create_order = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/commerce/orders")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"target_kind":"recharge_pack","target_id":"pack-100k"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_order.status(), StatusCode::CREATED);
    let create_order_json = read_json(create_order).await;
    let order_id = create_order_json["order_id"].as_str().unwrap().to_owned();

    let order = store
        .list_commerce_orders_for_project(&project_id)
        .await
        .unwrap()
        .into_iter()
        .find(|candidate| candidate.order_id == order_id)
        .unwrap();
    let order_amount_minor = order.payable_price_cents;
    let payment_attempt = CommercePaymentAttemptRecord::new(
        "payatt_stripe_webhook_001",
        order.order_id.clone(),
        order.project_id.clone(),
        order.user_id.clone(),
        "pm_stripe_checkout",
        "stripe",
        "hosted_checkout",
        "idem_stripe_webhook_001",
        1,
        order.payable_price_cents,
        order.currency_code.clone(),
        10,
    )
    .with_status("requires_action")
    .with_provider_payment_intent_id_option(Some("pi_test_123".to_owned()))
    .with_provider_checkout_session_id_option(Some("cs_test_123".to_owned()))
    .with_request_payload_json("{\"payment_method_id\":\"pm_stripe_checkout\"}")
    .with_response_payload_json("{\"status\":\"open\"}")
    .with_updated_at_ms(11);
    store
        .upsert_commerce_payment_attempt(&payment_attempt)
        .await
        .unwrap();
    store
        .insert_commerce_order(
            &order
                .clone()
                .with_payment_method_id_option(Some("pm_stripe_checkout".to_owned()))
                .with_latest_payment_attempt_id_option(Some(
                    payment_attempt.payment_attempt_id.clone(),
                ))
                .with_settlement_status("requires_action")
                .with_updated_at_ms(11),
        )
        .await
        .unwrap();

    let payload = json!({
        "id": "evt_test_checkout_completed",
        "type": "checkout.session.completed",
        "data": {
            "object": {
                "id": "cs_test_123",
                "payment_intent": "pi_test_123",
                "amount_total": order_amount_minor,
                "metadata": {
                    "order_id": order_id,
                    "project_id": project_id,
                    "user_id": user_id,
                    "payment_attempt_id": payment_attempt.payment_attempt_id,
                    "payment_method_id": "pm_stripe_checkout"
                }
            }
        }
    })
    .to_string();
    let timestamp = current_timestamp_secs();
    let signature = stripe_signature_header(&payload, "whsec_test_stripe", timestamp);

    let first_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/commerce/webhooks/stripe/pm_stripe_checkout")
                .header("Stripe-Signature", signature.as_str())
                .header("content-type", "application/json")
                .body(Body::from(payload.clone()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(first_response.status(), StatusCode::OK);
    let first_json = read_json(first_response).await;
    assert_eq!(first_json["processing_status"], "processed");
    assert_eq!(first_json["order_id"], order_id);
    assert_eq!(first_json["payment_attempt_id"], "payatt_stripe_webhook_001");
    assert_eq!(first_json["provider_event_id"], "evt_test_checkout_completed");

    let order_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/portal/commerce/orders/{order_id}"))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(order_response.status(), StatusCode::OK);
    let order_json = read_json(order_response).await;
    assert_eq!(order_json["status"], "fulfilled");

    let stored_attempt = store
        .find_commerce_payment_attempt("payatt_stripe_webhook_001")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(stored_attempt.status, "succeeded");
    assert_eq!(stored_attempt.captured_amount_minor, order_amount_minor);
    assert_eq!(
        stored_attempt.provider_checkout_session_id.as_deref(),
        Some("cs_test_123")
    );
    assert_eq!(
        stored_attempt.provider_payment_intent_id.as_deref(),
        Some("pi_test_123")
    );

    let payment_events = store
        .list_commerce_payment_events_for_order(&order_id)
        .await
        .unwrap();
    assert_eq!(payment_events.len(), 1);
    assert_eq!(payment_events[0].provider, "stripe");
    assert_eq!(
        payment_events[0].provider_event_id.as_deref(),
        Some("evt_test_checkout_completed")
    );
    assert_eq!(payment_events[0].processing_status.as_str(), "processed");
    assert_eq!(
        payment_events[0].order_status_after.as_deref(),
        Some("fulfilled")
    );

    let webhook_inbox = store
        .find_commerce_webhook_inbox_by_dedupe_key("stripe:evt_test_checkout_completed")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(webhook_inbox.provider, "stripe");
    assert_eq!(
        webhook_inbox.payment_method_id.as_deref(),
        Some("pm_stripe_checkout")
    );
    assert_eq!(webhook_inbox.processing_status, "processed");

    let second_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/commerce/webhooks/stripe/pm_stripe_checkout")
                .header("Stripe-Signature", signature.as_str())
                .header("content-type", "application/json")
                .body(Body::from(payload))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(second_response.status(), StatusCode::OK);
    let second_json = read_json(second_response).await;
    assert_eq!(second_json["processing_status"], "ignored");
    assert_eq!(second_json["order_id"], order_id);

    let duplicate_events = store
        .list_commerce_payment_events_for_order(&order_id)
        .await
        .unwrap();
    assert_eq!(duplicate_events.len(), 1);

    let invalid_signature_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/commerce/webhooks/stripe/pm_stripe_checkout")
                .header(
                    "Stripe-Signature",
                    format!("t={},v1=invalid", current_timestamp_secs()),
                )
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "id": "evt_invalid_signature",
                        "type": "checkout.session.completed",
                        "data": {
                            "object": {
                                "id": "cs_invalid",
                                "payment_intent": "pi_invalid",
                                "amount_total": 4000,
                                "metadata": {
                                    "order_id": order_id,
                                    "project_id": project_id,
                                    "user_id": user_id,
                                    "payment_attempt_id": "payatt_stripe_webhook_001",
                                    "payment_method_id": "pm_stripe_checkout"
                                }
                            }
                        }
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(invalid_signature_response.status(), StatusCode::CONFLICT);
    let invalid_signature_json = read_json(invalid_signature_response).await;
    assert!(
        invalid_signature_json["error"]["message"]
            .as_str()
            .unwrap()
            .contains("stripe webhook signature"),
        "unexpected error body: {invalid_signature_json}"
    );
}

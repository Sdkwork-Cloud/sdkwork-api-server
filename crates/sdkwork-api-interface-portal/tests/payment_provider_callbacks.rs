use axum::body::{to_bytes, Body};
use axum::extract::State;
use axum::http::{HeaderMap, Request, StatusCode};
use axum::routing::post;
use axum::{Json, Router};
use sdkwork_api_domain_billing::{
    AccountBenefitSourceType, AccountBenefitType, AccountLedgerEntryType, AccountType,
};
use sdkwork_api_domain_payment::PaymentOrderRecord;
use sdkwork_api_storage_core::AccountKernelStore;
use sdkwork_api_storage_sqlite::SqliteAdminStore;
use serde_json::{json, Value};
use serial_test::serial;
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::net::TcpListener;
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

async fn create_payment_backed_order(
    app: axum::Router,
    pool: SqlitePool,
    token: &str,
    provider: &str,
    payment_order_id: &str,
) -> (String, String, String) {
    let create_order = app
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

    let store = SqliteAdminStore::new(pool);
    let order = store
        .list_commerce_orders()
        .await
        .unwrap()
        .into_iter()
        .find(|candidate| candidate.order_id == order_id)
        .unwrap();
    let payment_order = PaymentOrderRecord::new(
        payment_order_id,
        &order.order_id,
        &order.project_id,
        &order.user_id,
        provider,
        "cny",
        order.payable_price_cents,
        "checkout_open",
        current_timestamp_secs() * 1000,
    );
    store
        .insert_payment_order_record(&payment_order)
        .await
        .unwrap();

    (
        order.order_id.clone(),
        order.project_id.clone(),
        payment_order.payment_order_id,
    )
}

#[derive(Clone, Default)]
struct StripeCaptureState {
    authorizations: Arc<Mutex<Vec<String>>>,
    payloads: Arc<Mutex<Vec<Value>>>,
}

async fn stripe_checkout_session_handler(
    State(state): State<StripeCaptureState>,
    headers: HeaderMap,
    body: String,
) -> Json<Value> {
    state.authorizations.lock().unwrap().push(
        headers
            .get("authorization")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_owned(),
    );
    state
        .payloads
        .lock()
        .unwrap()
        .push(serde_json::from_str(&body).unwrap());
    Json(json!({
        "id": "cs_test_123",
        "status": "open",
        "url": "https://checkout.stripe.test/session/cs_test_123"
    }))
}

async fn spawn_stripe_server() -> (String, StripeCaptureState) {
    let state = StripeCaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let app = Router::new()
        .route(
            "/v1/checkout/sessions",
            post(stripe_checkout_session_handler),
        )
        .with_state(state.clone());
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (format!("http://{address}"), state)
}

struct EnvGuard {
    key: &'static str,
    previous: Option<String>,
}

impl EnvGuard {
    fn set(key: &'static str, value: &str) -> Self {
        let previous = std::env::var(key).ok();
        std::env::set_var(key, value);
        Self { key, previous }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        match self.previous.as_deref() {
            Some(value) => std::env::set_var(self.key, value),
            None => std::env::remove_var(self.key),
        }
    }
}

fn stripe_signature_header(payload: &str, secret: &str, timestamp: u64) -> String {
    let signed_payload = format!("{timestamp}.{payload}");
    let signature = hmac_sha256_hex(secret.as_bytes(), signed_payload.as_bytes());
    format!("t={timestamp},v1={signature}")
}

fn alipay_signature_payload(fields: &[(&str, &str)]) -> String {
    let mut fields = fields
        .iter()
        .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
        .collect::<Vec<_>>();
    fields.sort_by(|left, right| left.0.cmp(&right.0));
    fields
        .into_iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join("&")
}

fn alipay_signature(fields: &[(&str, &str)], secret: &str) -> String {
    hmac_sha256_hex(
        secret.as_bytes(),
        alipay_signature_payload(fields).as_bytes(),
    )
}

fn encode_form_body(fields: &[(&str, &str)]) -> String {
    fields
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join("&")
}

fn wechatpay_signature(payload: &str, secret: &str, timestamp: u64, nonce: &str) -> String {
    let signed_payload = format!("{timestamp}\n{nonce}\n{payload}\n");
    hmac_sha256_hex(secret.as_bytes(), signed_payload.as_bytes())
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
    format!("{:x}", outer_hasher.finalize())
}

#[tokio::test]
#[serial]
async fn portal_checkout_session_uses_stripe_provider_when_configured() {
    let (stripe_base_url, stripe_state) = spawn_stripe_server().await;
    let _stripe_api_base_guard =
        EnvGuard::set("SDKWORK_STRIPE_API_BASE_URL", stripe_base_url.as_str());
    let _stripe_secret_guard = EnvGuard::set("SDKWORK_STRIPE_SECRET_KEY", "sk_test_stripe");
    let _stripe_webhook_guard = EnvGuard::set("SDKWORK_STRIPE_WEBHOOK_SECRET", "whsec_test_stripe");
    let _portal_return_base_guard = EnvGuard::set(
        "SDKWORK_PORTAL_CHECKOUT_RETURN_BASE_URL",
        "https://portal.example.com",
    );

    let pool = memory_pool().await;
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool);

    let registration = register_portal_user(app.clone()).await;
    let token = registration["token"].as_str().unwrap().to_owned();

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
    let order_id = create_order_json["order_id"].as_str().unwrap();
    assert_eq!(create_order_json["status"], "pending_payment");

    let checkout_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/portal/commerce/orders/{order_id}/checkout-session"
                ))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(checkout_response.status(), StatusCode::OK);
    let checkout_json = read_json(checkout_response).await;
    assert_eq!(checkout_json["provider"], "stripe");
    assert_eq!(checkout_json["mode"], "hosted_checkout");
    assert_eq!(
        checkout_json["checkout_url"],
        "https://checkout.stripe.test/session/cs_test_123"
    );
    assert_eq!(checkout_json["session_status"], "open");
    assert!(checkout_json["reference"]
        .as_str()
        .unwrap()
        .starts_with("payment_order_"));
    assert!(checkout_json["methods"]
        .as_array()
        .unwrap()
        .iter()
        .any(|method| method["action"] == "redirect_checkout"
            && method["availability"] == "available"));

    let authorizations = stripe_state.authorizations.lock().unwrap().clone();
    assert_eq!(authorizations, vec!["Bearer sk_test_stripe".to_owned()]);
    let payloads = stripe_state.payloads.lock().unwrap().clone();
    assert_eq!(payloads.len(), 1);
    assert_eq!(payloads[0]["metadata"]["order_id"], order_id);
    assert_eq!(payloads[0]["amount_cents"], 4000);
    assert_eq!(payloads[0]["currency"], "usd");
}

#[tokio::test]
#[serial]
async fn portal_stripe_webhook_verifies_signature_and_settles_paid_order() {
    let (stripe_base_url, _stripe_state) = spawn_stripe_server().await;
    let _stripe_api_base_guard =
        EnvGuard::set("SDKWORK_STRIPE_API_BASE_URL", stripe_base_url.as_str());
    let _stripe_secret_guard = EnvGuard::set("SDKWORK_STRIPE_SECRET_KEY", "sk_test_stripe");
    let _stripe_webhook_guard = EnvGuard::set("SDKWORK_STRIPE_WEBHOOK_SECRET", "whsec_test_stripe");
    let _portal_return_base_guard = EnvGuard::set(
        "SDKWORK_PORTAL_CHECKOUT_RETURN_BASE_URL",
        "https://portal.example.com",
    );

    let pool = memory_pool().await;
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool.clone());

    let registration = register_portal_user(app.clone()).await;
    let token = registration["token"].as_str().unwrap().to_owned();

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

    let checkout_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/portal/commerce/orders/{order_id}/checkout-session"
                ))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(checkout_response.status(), StatusCode::OK);
    let checkout_json = read_json(checkout_response).await;
    let payment_order_id = checkout_json["reference"].as_str().unwrap().to_owned();

    let payload = json!({
        "id": "evt_test_checkout_completed",
        "type": "checkout.session.completed",
        "data": {
            "object": {
                "id": "cs_test_123",
                "metadata": {
                    "order_id": order_id,
                    "payment_order_id": payment_order_id
                }
            }
        }
    })
    .to_string();
    let signature =
        stripe_signature_header(&payload, "whsec_test_stripe", current_timestamp_secs());

    let webhook_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/internal/payments/stripe/webhook")
                .header("stripe-signature", signature)
                .header("content-type", "application/json")
                .body(Body::from(payload))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(webhook_response.status(), StatusCode::OK);
    let webhook_json = read_json(webhook_response).await;
    assert_eq!(webhook_json["order_id"], order_id);
    assert_eq!(webhook_json["status"], "fulfilled");

    let orders_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/commerce/orders")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(orders_response.status(), StatusCode::OK);
    let orders_json = read_json(orders_response).await;
    assert_eq!(orders_json.as_array().unwrap()[0]["status"], "fulfilled");

    let store = SqliteAdminStore::new(pool.clone());
    let attempts = store
        .list_payment_attempt_records_for_payment_order(&payment_order_id)
        .await
        .unwrap();
    assert_eq!(attempts.len(), 1);
    assert_eq!(attempts[0].provider, "stripe");
    assert_eq!(attempts[0].provider_attempt_id, "cs_test_123");
    assert_eq!(attempts[0].attempt_kind, "capture");
    assert_eq!(attempts[0].status, "succeeded");
    let accounts = store.list_account_records().await.unwrap();
    assert_eq!(accounts.len(), 1);
    assert_eq!(accounts[0].account_type, AccountType::Primary);
    let benefit_lots = store.list_account_benefit_lots().await.unwrap();
    assert_eq!(benefit_lots.len(), 1);
    assert_eq!(benefit_lots[0].account_id, accounts[0].account_id);
    assert_eq!(benefit_lots[0].benefit_type, AccountBenefitType::CashCredit);
    assert_eq!(
        benefit_lots[0].source_type,
        AccountBenefitSourceType::Recharge
    );
    assert_eq!(benefit_lots[0].original_quantity, 100_000.0);
    assert_eq!(benefit_lots[0].remaining_quantity, 100_000.0);
    assert_eq!(benefit_lots[0].held_quantity, 0.0);
    let ledger_entries = store.list_account_ledger_entry_records().await.unwrap();
    assert_eq!(ledger_entries.len(), 1);
    assert_eq!(ledger_entries[0].account_id, accounts[0].account_id);
    assert_eq!(
        ledger_entries[0].entry_type,
        AccountLedgerEntryType::GrantIssue
    );
    assert_eq!(ledger_entries[0].quantity, 100_000.0);
    assert_eq!(ledger_entries[0].amount, 100_000.0);

    let invalid_signature_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/internal/payments/stripe/webhook")
                .header("stripe-signature", "t=1900000000,v1=invalid")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "id": "evt_invalid_signature",
                        "type": "checkout.session.completed",
                        "data": {
                            "object": {
                                "id": "cs_test_invalid",
                                "metadata": {
                                    "order_id": "order_invalid",
                                    "payment_order_id": "payment_order_invalid"
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
    assert_eq!(
        invalid_signature_response.status(),
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
#[serial]
async fn portal_alipay_notification_verifies_signature_and_settles_paid_order() {
    let _alipay_notify_guard = EnvGuard::set("SDKWORK_ALIPAY_NOTIFY_SECRET", "alipay_test_secret");

    let pool = memory_pool().await;
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool.clone());

    let registration = register_portal_user(app.clone()).await;
    let token = registration["token"].as_str().unwrap().to_owned();
    let (order_id, _project_id, payment_order_id) = create_payment_backed_order(
        app.clone(),
        pool.clone(),
        &token,
        "alipay",
        "payment_order_alipay_001",
    )
    .await;

    let signed_fields = vec![
        ("notify_id", "notify_alipay_001"),
        ("notify_time", "2026-04-04 12:00:00"),
        ("out_trade_no", payment_order_id.as_str()),
        ("trade_no", "alipay_trade_001"),
        ("trade_status", "TRADE_SUCCESS"),
        ("total_amount", "40.00"),
        ("sign_type", "HMAC-SHA256"),
    ];
    let signature = alipay_signature(&signed_fields, "alipay_test_secret");
    let mut request_fields = signed_fields.clone();
    request_fields.push(("sign", signature.as_str()));

    let notify_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/internal/payments/alipay/notify")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(encode_form_body(&request_fields)))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(notify_response.status(), StatusCode::OK);
    let notify_json = read_json(notify_response).await;
    assert_eq!(notify_json["order_id"], order_id);
    assert_eq!(notify_json["status"], "fulfilled");

    let invalid_signature_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/internal/payments/alipay/notify")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(
                    "notify_id=notify_alipay_invalid&out_trade_no=payment_order_alipay_001&trade_no=alipay_trade_invalid&trade_status=TRADE_SUCCESS&sign_type=HMAC-SHA256&sign=invalid",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        invalid_signature_response.status(),
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
#[serial]
async fn portal_wechatpay_notification_is_idempotent_and_rejects_invalid_signature() {
    let _wechatpay_notify_guard =
        EnvGuard::set("SDKWORK_WECHATPAY_NOTIFY_SECRET", "wechatpay_test_secret");

    let pool = memory_pool().await;
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool.clone());

    let registration = register_portal_user(app.clone()).await;
    let token = registration["token"].as_str().unwrap().to_owned();
    let (order_id, project_id, payment_order_id) = create_payment_backed_order(
        app.clone(),
        pool.clone(),
        &token,
        "wechatpay",
        "payment_order_wechat_001",
    )
    .await;

    let payload = json!({
        "id": "evt_wechatpay_001",
        "event_type": "TRANSACTION.SUCCESS",
        "resource": {
            "out_trade_no": payment_order_id,
            "transaction_id": "wechat_transaction_001",
            "trade_state": "SUCCESS"
        }
    })
    .to_string();
    let timestamp = current_timestamp_secs();
    let nonce = "wechatpay_nonce_001";
    let signature = wechatpay_signature(&payload, "wechatpay_test_secret", timestamp, nonce);

    let first_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/internal/payments/wechat/notify")
                .header("content-type", "application/json")
                .header("wechatpay-timestamp", timestamp.to_string())
                .header("wechatpay-nonce", nonce)
                .header("wechatpay-signature", signature.as_str())
                .body(Body::from(payload.clone()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(first_response.status(), StatusCode::OK);
    let first_json = read_json(first_response).await;
    assert_eq!(first_json["order_id"], order_id);
    assert_eq!(first_json["status"], "fulfilled");

    let store = SqliteAdminStore::new(pool.clone());
    let ledger_before_replay = store
        .list_ledger_entries_for_project(&project_id)
        .await
        .unwrap()
        .len();

    let second_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/internal/payments/wechat/notify")
                .header("content-type", "application/json")
                .header("wechatpay-timestamp", timestamp.to_string())
                .header("wechatpay-nonce", nonce)
                .header("wechatpay-signature", signature.as_str())
                .body(Body::from(payload.clone()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(second_response.status(), StatusCode::OK);
    let second_json = read_json(second_response).await;
    assert_eq!(second_json["order_id"], order_id);
    assert_eq!(second_json["status"], "fulfilled");

    let ledger_after_replay = store
        .list_ledger_entries_for_project(&project_id)
        .await
        .unwrap()
        .len();
    assert_eq!(ledger_after_replay, ledger_before_replay);
    let attempts = store
        .list_payment_attempt_records_for_payment_order(&payment_order_id)
        .await
        .unwrap();
    assert_eq!(attempts.len(), 1);
    assert_eq!(attempts[0].provider, "wechatpay");
    assert_eq!(attempts[0].provider_attempt_id, "wechat_transaction_001");
    assert_eq!(attempts[0].attempt_kind, "capture");
    assert_eq!(attempts[0].status, "succeeded");

    let persisted_event_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM ai_payment_webhook_events WHERE provider = ? AND provider_event_id = ?",
    )
    .bind("wechatpay")
    .bind("evt_wechatpay_001")
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(persisted_event_count, 1);

    let metrics_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/metrics")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(metrics_response.status(), StatusCode::OK);
    let metrics_body = String::from_utf8(
        to_bytes(metrics_response.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert!(metrics_body.contains(&format!(
        "sdkwork_payment_callbacks_total{{service=\"portal\",provider=\"wechatpay\",tenant=\"{project_id}\",payment_outcome=\"settled\"}} 1"
    )));
    assert!(metrics_body.contains(&format!(
        "sdkwork_payment_callbacks_total{{service=\"portal\",provider=\"wechatpay\",tenant=\"{project_id}\",payment_outcome=\"duplicate\"}} 1"
    )));
    assert!(metrics_body.contains(&format!(
        "sdkwork_commercial_events_total{{service=\"portal\",event_kind=\"callback_replay\",route=\"/portal/internal/payments/wechat/notify\",tenant=\"{project_id}\",provider=\"wechatpay\",model=\"none\",payment_outcome=\"duplicate\",result=\"ignored\"}} 1"
    )));

    let invalid_signature_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/internal/payments/wechat/notify")
                .header("content-type", "application/json")
                .header("wechatpay-timestamp", timestamp.to_string())
                .header("wechatpay-nonce", "wechatpay_nonce_invalid")
                .header("wechatpay-signature", "invalid")
                .body(Body::from(
                    json!({
                        "id": "evt_wechatpay_invalid",
                        "event_type": "TRANSACTION.SUCCESS",
                        "resource": {
                            "out_trade_no": "payment_order_wechat_001",
                            "transaction_id": "wechat_transaction_invalid",
                            "trade_state": "SUCCESS"
                        }
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        invalid_signature_response.status(),
        StatusCode::UNAUTHORIZED
    );
}

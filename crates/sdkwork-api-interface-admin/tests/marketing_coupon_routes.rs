use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use axum::Router;
use serde_json::Value;
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

async fn login_token(app: Router) -> String {
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"email\":\"admin@sdkwork.local\",\"password\":\"ChangeMe123!\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    read_json(response).await["token"]
        .as_str()
        .unwrap()
        .to_owned()
}

fn unix_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

async fn admin_post_json(
    app: &Router,
    token: &str,
    uri: &str,
    body: &str,
    request_id: Option<&str>,
) -> axum::response::Response {
    let mut builder = Request::builder()
        .method("POST")
        .uri(uri)
        .header("authorization", format!("Bearer {token}"))
        .header("content-type", "application/json");
    if let Some(request_id) = request_id {
        builder = builder.header("x-request-id", request_id);
    }
    app.clone()
        .oneshot(builder.body(Body::from(body.to_owned())).unwrap())
        .await
        .unwrap()
}

async fn approve_coupon_template(app: &Router, token: &str, coupon_template_id: &str, scope: &str) {
    let submitted = admin_post_json(
        app,
        token,
        &format!(
            "/admin/marketing/coupon-templates/{coupon_template_id}/submit-for-approval"
        ),
        r#"{"reason":"submit coupon template for approval"}"#,
        Some(&format!("{scope}-submit")),
    )
    .await;
    assert_eq!(submitted.status(), StatusCode::OK);
    assert_eq!(
        read_json(submitted).await["detail"]["coupon_template"]["approval_state"],
        "in_review"
    );

    let approved = admin_post_json(
        app,
        token,
        &format!("/admin/marketing/coupon-templates/{coupon_template_id}/approve"),
        r#"{"reason":"approve coupon template for lifecycle actions"}"#,
        Some(&format!("{scope}-approve")),
    )
    .await;
    assert_eq!(approved.status(), StatusCode::OK);
    assert_eq!(
        read_json(approved).await["detail"]["coupon_template"]["approval_state"],
        "approved"
    );
}

async fn activate_coupon_template(app: &Router, token: &str, coupon_template_id: &str, scope: &str) {
    approve_coupon_template(app, token, coupon_template_id, scope).await;

    let published = admin_post_json(
        app,
        token,
        &format!("/admin/marketing/coupon-templates/{coupon_template_id}/publish"),
        r#"{"reason":"publish coupon template for campaign lifecycle tests"}"#,
        Some(&format!("{scope}-publish")),
    )
    .await;
    assert_eq!(published.status(), StatusCode::OK);
    assert_eq!(
        read_json(published).await["detail"]["coupon_template"]["status"],
        "active"
    );
}

async fn approve_marketing_campaign(
    app: &Router,
    token: &str,
    marketing_campaign_id: &str,
    scope: &str,
) {
    let submitted = admin_post_json(
        app,
        token,
        &format!("/admin/marketing/campaigns/{marketing_campaign_id}/submit-for-approval"),
        r#"{"reason":"submit coupon campaign for approval"}"#,
        Some(&format!("{scope}-submit")),
    )
    .await;
    assert_eq!(submitted.status(), StatusCode::OK);
    assert_eq!(
        read_json(submitted).await["detail"]["campaign"]["approval_state"],
        "in_review"
    );

    let approved = admin_post_json(
        app,
        token,
        &format!("/admin/marketing/campaigns/{marketing_campaign_id}/approve"),
        r#"{"reason":"approve coupon campaign for lifecycle actions"}"#,
        Some(&format!("{scope}-approve")),
    )
    .await;
    assert_eq!(approved.status(), StatusCode::OK);
    assert_eq!(
        read_json(approved).await["detail"]["campaign"]["approval_state"],
        "approved"
    );
}

#[tokio::test]
async fn admin_marketing_routes_create_and_list_canonical_coupon_records_without_legacy_coupon_route(
) {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let template = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/coupon-templates")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "coupon_template_id":"template_launch20",
                        "template_key":"launch20",
                        "display_name":"Launch 20",
                        "status":"draft",
                        "distribution_kind":"unique_code",
                        "benefit":{"benefit_kind":"percentage_off","discount_percent":20},
                        "restriction":{"subject_scope":"project","stacking_policy":"exclusive"},
                        "created_at_ms":1710000000000,
                        "updated_at_ms":1710000000000
                    }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(template.status(), StatusCode::CREATED);

    let campaign = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/campaigns")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "marketing_campaign_id":"campaign_launch20",
                        "coupon_template_id":"template_launch20",
                        "display_name":"Launch Campaign",
                        "status":"draft",
                        "created_at_ms":1710000000000,
                        "updated_at_ms":1710000000000
                    }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(campaign.status(), StatusCode::CREATED);

    let budget = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/budgets")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "campaign_budget_id":"budget_launch20",
                        "marketing_campaign_id":"campaign_launch20",
                        "status":"draft",
                        "total_budget_minor":500000,
                        "reserved_budget_minor":0,
                        "consumed_budget_minor":0,
                        "created_at_ms":1710000000000,
                        "updated_at_ms":1710000000000
                    }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(budget.status(), StatusCode::CREATED);

    let code = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/codes")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "coupon_code_id":"code_launch20",
                        "coupon_template_id":"template_launch20",
                        "code_value":"LAUNCH20",
                        "status":"available",
                        "created_at_ms":1710000000000,
                        "updated_at_ms":1710000000000
                    }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(code.status(), StatusCode::CREATED);

    let templates = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/marketing/coupon-templates")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(templates.status(), StatusCode::OK);
    let templates_json = read_json(templates).await;
    assert_eq!(templates_json.as_array().unwrap().len(), 1);
    assert_eq!(templates_json[0]["template_key"], "launch20");

    let codes = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/marketing/codes")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(codes.status(), StatusCode::OK);
    let codes_json = read_json(codes).await;
    assert_eq!(codes_json.as_array().unwrap().len(), 1);
    assert_eq!(codes_json[0]["code_value"], "LAUNCH20");

    let coupons = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/coupons")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(coupons.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn admin_legacy_coupon_create_route_is_removed() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let created = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/coupons")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"id\":\"coupon_spring_launch\",\"code\":\"SPRING20\",\"discount_label\":\"20% launch discount\",\"audience\":\"new_signup\",\"remaining\":120,\"active\":true,\"note\":\"Spring launch campaign\",\"expires_on\":\"2026-05-31\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(created.status(), StatusCode::NOT_FOUND);

    let templates = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/marketing/coupon-templates")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(templates.status(), StatusCode::OK);
    let templates_json = read_json(templates).await;
    assert!(templates_json.as_array().unwrap().is_empty());

    let codes = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/marketing/codes")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(codes.status(), StatusCode::OK);
    let codes_json = read_json(codes).await;
    assert!(codes_json.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn admin_marketing_status_routes_update_canonical_coupon_records() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let template_payload = r#"{
        "coupon_template_id":"template_launch20",
        "template_key":"launch20",
        "display_name":"Launch 20",
        "status":"draft",
        "distribution_kind":"unique_code",
        "benefit":{"benefit_kind":"percentage_off","discount_percent":20},
        "restriction":{"subject_scope":"project","stacking_policy":"exclusive"},
        "created_at_ms":1710000000000,
        "updated_at_ms":1710000000000
    }"#;
    let campaign_payload = r#"{
        "marketing_campaign_id":"campaign_launch20",
        "coupon_template_id":"template_launch20",
        "display_name":"Launch Campaign",
        "status":"draft",
        "created_at_ms":1710000000000,
        "updated_at_ms":1710000000000
    }"#;
    let budget_payload = r#"{
        "campaign_budget_id":"budget_launch20",
        "marketing_campaign_id":"campaign_launch20",
        "status":"draft",
        "total_budget_minor":500000,
        "reserved_budget_minor":0,
        "consumed_budget_minor":0,
        "created_at_ms":1710000000000,
        "updated_at_ms":1710000000000
    }"#;
    let code_payload = r#"{
        "coupon_code_id":"code_launch20",
        "coupon_template_id":"template_launch20",
        "code_value":"LAUNCH20",
        "status":"available",
        "created_at_ms":1710000000000,
        "updated_at_ms":1710000000000
    }"#;

    for (uri, payload) in [
        ("/admin/marketing/coupon-templates", template_payload),
        ("/admin/marketing/campaigns", campaign_payload),
        ("/admin/marketing/budgets", budget_payload),
        ("/admin/marketing/codes", code_payload),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(uri)
                    .header("authorization", format!("Bearer {token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(payload))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    let template = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/coupon-templates/template_launch20/status")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"status":"archived"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(template.status(), StatusCode::OK);
    assert_eq!(read_json(template).await["status"], "archived");

    let campaign = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/campaigns/campaign_launch20/status")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"status":"paused"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(campaign.status(), StatusCode::OK);
    assert_eq!(read_json(campaign).await["status"], "paused");

    let budget = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/budgets/budget_launch20/status")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"status":"closed"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(budget.status(), StatusCode::OK);
    assert_eq!(read_json(budget).await["status"], "closed");

    let code = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/codes/code_launch20/status")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"status":"disabled"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(code.status(), StatusCode::OK);
    assert_eq!(read_json(code).await["status"], "disabled");
}

#[tokio::test]
async fn admin_marketing_campaign_lifecycle_routes_apply_coupon_semantic_actions() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;
    let now_ms = unix_timestamp_ms();

    let template = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/coupon-templates")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "coupon_template_id":"template_campaign_lifecycle",
                        "template_key":"campaign-lifecycle",
                        "display_name":"Campaign Lifecycle",
                        "status":"draft",
                        "distribution_kind":"shared_code",
                        "benefit":{"benefit_kind":"percentage_off","discount_percent":15},
                        "restriction":{"subject_scope":"project","stacking_policy":"exclusive"},
                        "created_at_ms":1710000000000,
                        "updated_at_ms":1710000000000
                    }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(template.status(), StatusCode::CREATED);
    activate_coupon_template(
        &app,
        &token,
        "template_campaign_lifecycle",
        "sdkw-test-template-campaign-lifecycle",
    )
    .await;

    let publish_campaign = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/campaigns")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{
                        "marketing_campaign_id":"campaign_publish_ready",
                        "coupon_template_id":"template_campaign_lifecycle",
                        "display_name":"Publish Ready Campaign",
                        "status":"draft",
                        "start_at_ms":{},
                        "created_at_ms":1710000000000,
                        "updated_at_ms":1710000000000
                    }}"#,
                    now_ms.saturating_sub(60_000),
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(publish_campaign.status(), StatusCode::CREATED);
    approve_marketing_campaign(
        &app,
        &token,
        "campaign_publish_ready",
        "sdkw-test-campaign-publish-ready",
    )
    .await;

    let schedule_campaign = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/campaigns")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{
                        "marketing_campaign_id":"campaign_schedule_ready",
                        "coupon_template_id":"template_campaign_lifecycle",
                        "display_name":"Schedule Ready Campaign",
                        "status":"draft",
                        "start_at_ms":{},
                        "created_at_ms":1710000000000,
                        "updated_at_ms":1710000000000
                    }}"#,
                    now_ms + 3_600_000,
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(schedule_campaign.status(), StatusCode::CREATED);
    approve_marketing_campaign(
        &app,
        &token,
        "campaign_schedule_ready",
        "sdkw-test-campaign-schedule-ready",
    )
    .await;

    let published = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/campaigns/campaign_publish_ready/publish")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .header("x-request-id", "sdkw-test-campaign-publish-1")
                .body(Body::from(
                    r#"{"reason":"publish canonical coupon campaign"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(published.status(), StatusCode::OK);
    let published_json = read_json(published).await;
    assert_eq!(published_json["detail"]["campaign"]["status"], "active");
    assert_eq!(
        published_json["detail"]["coupon_template"]["coupon_template_id"],
        "template_campaign_lifecycle"
    );
    assert_eq!(published_json["audit"]["action"], "publish");
    assert_eq!(published_json["audit"]["outcome"], "applied");
    assert_eq!(
        published_json["audit"]["reason"],
        "publish canonical coupon campaign"
    );
    assert_eq!(
        published_json["audit"]["operator_id"],
        "admin_local_default"
    );
    assert_eq!(
        published_json["audit"]["request_id"],
        "sdkw-test-campaign-publish-1"
    );

    let scheduled = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/campaigns/campaign_schedule_ready/schedule")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .header("x-request-id", "sdkw-test-campaign-schedule-1")
                .body(Body::from(
                    r#"{"reason":"schedule canonical coupon campaign"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(scheduled.status(), StatusCode::OK);
    let scheduled_json = read_json(scheduled).await;
    assert_eq!(scheduled_json["detail"]["campaign"]["status"], "scheduled");
    assert_eq!(scheduled_json["audit"]["action"], "schedule");
    assert_eq!(scheduled_json["audit"]["outcome"], "applied");

    let retired = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/campaigns/campaign_publish_ready/retire")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .header("x-request-id", "sdkw-test-campaign-retire-1")
                .body(Body::from(
                    r#"{"reason":"retire canonical coupon campaign"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(retired.status(), StatusCode::OK);
    let retired_json = read_json(retired).await;
    assert_eq!(retired_json["detail"]["campaign"]["status"], "ended");
    assert_eq!(retired_json["audit"]["action"], "retire");
    assert_eq!(retired_json["audit"]["outcome"], "applied");
    assert_eq!(retired_json["audit"]["operator_id"], "admin_local_default");
    assert_eq!(
        retired_json["audit"]["request_id"],
        "sdkw-test-campaign-retire-1"
    );

    let publish_audits = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/marketing/campaigns/campaign_publish_ready/lifecycle-audits")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(publish_audits.status(), StatusCode::OK);
    let publish_audits_json = read_json(publish_audits).await;
    assert_eq!(publish_audits_json.as_array().unwrap().len(), 4);
    assert_eq!(publish_audits_json[0]["action"], "retire");
    assert_eq!(publish_audits_json[0]["outcome"], "applied");
    assert_eq!(
        publish_audits_json[0]["request_id"],
        "sdkw-test-campaign-retire-1"
    );
    assert_eq!(publish_audits_json[1]["action"], "publish");
    assert_eq!(publish_audits_json[1]["outcome"], "applied");
    assert_eq!(
        publish_audits_json[1]["request_id"],
        "sdkw-test-campaign-publish-1"
    );
    assert_eq!(publish_audits_json[2]["action"], "approve");
    assert_eq!(
        publish_audits_json[2]["request_id"],
        "sdkw-test-campaign-publish-ready-approve"
    );
    assert_eq!(publish_audits_json[3]["action"], "submit_for_approval");
    assert_eq!(
        publish_audits_json[3]["request_id"],
        "sdkw-test-campaign-publish-ready-submit"
    );

    let schedule_audits = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/marketing/campaigns/campaign_schedule_ready/lifecycle-audits")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(schedule_audits.status(), StatusCode::OK);
    let schedule_audits_json = read_json(schedule_audits).await;
    assert_eq!(schedule_audits_json.as_array().unwrap().len(), 3);
    assert_eq!(schedule_audits_json[0]["action"], "schedule");
    assert_eq!(
        schedule_audits_json[0]["request_id"],
        "sdkw-test-campaign-schedule-1"
    );
    assert_eq!(schedule_audits_json[1]["action"], "approve");
    assert_eq!(
        schedule_audits_json[1]["request_id"],
        "sdkw-test-campaign-schedule-ready-approve"
    );
    assert_eq!(schedule_audits_json[2]["action"], "submit_for_approval");
    assert_eq!(
        schedule_audits_json[2]["request_id"],
        "sdkw-test-campaign-schedule-ready-submit"
    );
}

#[tokio::test]
async fn admin_marketing_campaign_publish_rejects_future_coupon_campaign() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;
    let now_ms = unix_timestamp_ms();

    let template = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/coupon-templates")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "coupon_template_id":"template_publish_reject",
                        "template_key":"publish-reject",
                        "display_name":"Publish Reject",
                        "status":"draft",
                        "distribution_kind":"shared_code",
                        "benefit":{"benefit_kind":"percentage_off","discount_percent":10},
                        "restriction":{"subject_scope":"project","stacking_policy":"exclusive"},
                        "created_at_ms":1710000000000,
                        "updated_at_ms":1710000000000
                    }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(template.status(), StatusCode::CREATED);
    activate_coupon_template(
        &app,
        &token,
        "template_publish_reject",
        "sdkw-test-template-publish-reject",
    )
    .await;

    let campaign = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/campaigns")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{
                        "marketing_campaign_id":"campaign_publish_future",
                        "coupon_template_id":"template_publish_reject",
                        "display_name":"Future Publish Campaign",
                        "status":"draft",
                        "start_at_ms":{},
                        "created_at_ms":1710000000000,
                        "updated_at_ms":1710000000000
                    }}"#,
                    now_ms + 600_000,
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(campaign.status(), StatusCode::CREATED);
    approve_marketing_campaign(
        &app,
        &token,
        "campaign_publish_future",
        "sdkw-test-campaign-publish-future",
    )
    .await;

    let published = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/campaigns/campaign_publish_future/publish")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .header("x-request-id", "sdkw-test-campaign-publish-rejected-1")
                .body(Body::from(
                    r#"{"reason":"attempt premature coupon campaign publish"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(published.status(), StatusCode::BAD_REQUEST);
    assert!(read_json(published).await["error"]["message"]
        .as_str()
        .unwrap()
        .contains("future start_at_ms"));

    let audits = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/marketing/campaigns/campaign_publish_future/lifecycle-audits")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(audits.status(), StatusCode::OK);
    let audits_json = read_json(audits).await;
    assert_eq!(audits_json.as_array().unwrap().len(), 3);
    assert_eq!(audits_json[0]["action"], "publish");
    assert_eq!(audits_json[0]["outcome"], "rejected");
    assert_eq!(
        audits_json[0]["request_id"],
        "sdkw-test-campaign-publish-rejected-1"
    );
    assert_eq!(audits_json[1]["action"], "approve");
    assert_eq!(
        audits_json[1]["request_id"],
        "sdkw-test-campaign-publish-future-approve"
    );
    assert_eq!(audits_json[2]["action"], "submit_for_approval");
    assert_eq!(
        audits_json[2]["request_id"],
        "sdkw-test-campaign-publish-future-submit"
    );
    assert!(audits_json[0]["decision_reasons"]
        .as_array()
        .unwrap()
        .iter()
        .any(|reason| reason
            == "campaign has future start_at_ms and must be scheduled before publish"));
}

#[tokio::test]
async fn admin_marketing_campaign_revision_governance_routes_clone_compare_reject_and_approve_campaigns(
) {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let template = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/coupon-templates")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "coupon_template_id":"template_campaign_revision_source",
                        "template_key":"campaign-revision-template-source",
                        "display_name":"Campaign Revision Template Source",
                        "status":"draft",
                        "approval_state":"approved",
                        "revision":1,
                        "distribution_kind":"shared_code",
                        "benefit":{"benefit_kind":"percentage_off","discount_percent":15},
                        "restriction":{"subject_scope":"project","stacking_policy":"exclusive"},
                        "created_at_ms":1710000000000,
                        "updated_at_ms":1710000000000
                    }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(template.status(), StatusCode::CREATED);
    activate_coupon_template(
        &app,
        &token,
        "template_campaign_revision_source",
        "sdkw-test-template-campaign-revision-source",
    )
    .await;

    let source_campaign = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/campaigns")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "marketing_campaign_id":"campaign_revision_source",
                        "coupon_template_id":"template_campaign_revision_source",
                        "display_name":"Campaign Revision Source",
                        "status":"draft",
                        "approval_state":"approved",
                        "revision":1,
                        "created_at_ms":1710000000000,
                        "updated_at_ms":1710000000000
                    }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(source_campaign.status(), StatusCode::CREATED);

    let cloned = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/campaigns/campaign_revision_source/clone")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .header("x-request-id", "sdkw-test-campaign-clone-1")
                .body(Body::from(
                    r#"{
                        "marketing_campaign_id":"campaign_revision_clone",
                        "display_name":"Campaign Revision Clone",
                        "reason":"clone coupon campaign into governed draft"
                    }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(cloned.status(), StatusCode::CREATED);
    let cloned_json = read_json(cloned).await;
    assert_eq!(
        cloned_json["detail"]["campaign"]["marketing_campaign_id"],
        "campaign_revision_clone"
    );
    assert_eq!(cloned_json["detail"]["campaign"]["status"], "draft");
    assert_eq!(cloned_json["detail"]["campaign"]["approval_state"], "draft");
    assert_eq!(cloned_json["detail"]["campaign"]["revision"], 2);
    assert_eq!(
        cloned_json["detail"]["campaign"]["parent_marketing_campaign_id"],
        "campaign_revision_source"
    );
    assert_eq!(
        cloned_json["detail"]["campaign"]["root_marketing_campaign_id"],
        "campaign_revision_source"
    );
    assert_eq!(cloned_json["audit"]["action"], "clone");
    assert_eq!(cloned_json["audit"]["outcome"], "applied");
    assert_eq!(
        cloned_json["audit"]["source_marketing_campaign_id"],
        "campaign_revision_source"
    );
    assert_eq!(cloned_json["audit"]["previous_revision"], 1);
    assert_eq!(cloned_json["audit"]["resulting_revision"], 2);

    let comparison = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/campaigns/campaign_revision_source/compare")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"target_marketing_campaign_id":"campaign_revision_clone"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(comparison.status(), StatusCode::OK);
    let comparison_json = read_json(comparison).await;
    assert_eq!(comparison_json["same_lineage"], true);
    assert_eq!(
        comparison_json["source_marketing_campaign"]["marketing_campaign_id"],
        "campaign_revision_source"
    );
    assert_eq!(
        comparison_json["target_marketing_campaign"]["marketing_campaign_id"],
        "campaign_revision_clone"
    );
    assert!(comparison_json["field_changes"]
        .as_array()
        .unwrap()
        .iter()
        .any(|change| change["field"] == "display_name"));

    let rejected_publish = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/campaigns/campaign_revision_clone/publish")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .header(
                    "x-request-id",
                    "sdkw-test-campaign-clone-publish-rejected-1",
                )
                .body(Body::from(
                    r#"{"reason":"attempt publish before approval"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(rejected_publish.status(), StatusCode::BAD_REQUEST);
    assert!(read_json(rejected_publish).await["error"]["message"]
        .as_str()
        .unwrap()
        .contains("approved before publish"));

    let submitted = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/campaigns/campaign_revision_clone/submit-for-approval")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .header("x-request-id", "sdkw-test-campaign-submit-1")
                .body(Body::from(
                    r#"{"reason":"submit governed campaign revision for approval"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(submitted.status(), StatusCode::OK);
    let submitted_json = read_json(submitted).await;
    assert_eq!(
        submitted_json["detail"]["campaign"]["approval_state"],
        "in_review"
    );
    assert_eq!(submitted_json["audit"]["action"], "submit_for_approval");

    let rejected = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/campaigns/campaign_revision_clone/reject")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .header("x-request-id", "sdkw-test-campaign-reject-1")
                .body(Body::from(
                    r#"{"reason":"reject governed campaign revision"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(rejected.status(), StatusCode::OK);
    let rejected_json = read_json(rejected).await;
    assert_eq!(
        rejected_json["detail"]["campaign"]["approval_state"],
        "rejected"
    );
    assert_eq!(rejected_json["audit"]["action"], "reject");

    let resubmitted = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/campaigns/campaign_revision_clone/submit-for-approval")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .header("x-request-id", "sdkw-test-campaign-submit-2")
                .body(Body::from(
                    r#"{"reason":"resubmit governed campaign revision for approval"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resubmitted.status(), StatusCode::OK);

    let approved = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/campaigns/campaign_revision_clone/approve")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .header("x-request-id", "sdkw-test-campaign-approve-1")
                .body(Body::from(
                    r#"{"reason":"approve governed campaign revision"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(approved.status(), StatusCode::OK);
    let approved_json = read_json(approved).await;
    assert_eq!(
        approved_json["detail"]["campaign"]["approval_state"],
        "approved"
    );
    assert_eq!(approved_json["audit"]["action"], "approve");

    let published = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/campaigns/campaign_revision_clone/publish")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .header("x-request-id", "sdkw-test-campaign-clone-publish-1")
                .body(Body::from(
                    r#"{"reason":"publish approved campaign revision"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(published.status(), StatusCode::OK);
    let published_json = read_json(published).await;
    assert_eq!(published_json["detail"]["campaign"]["status"], "active");
    assert_eq!(
        published_json["detail"]["campaign"]["approval_state"],
        "approved"
    );

    let audits = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/marketing/campaigns/campaign_revision_clone/lifecycle-audits")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(audits.status(), StatusCode::OK);
    let audits_json = read_json(audits).await;
    let audits = audits_json.as_array().unwrap();
    assert_eq!(audits.len(), 7);
    assert!(audits.iter().any(|audit| audit["action"] == "clone"));
    assert!(audits
        .iter()
        .any(|audit| audit["action"] == "submit_for_approval"));
    assert!(audits.iter().any(|audit| audit["action"] == "reject"));
    assert!(audits.iter().any(|audit| audit["action"] == "approve"));
    assert!(audits.iter().any(|audit| audit["action"] == "publish"));
    assert!(audits.iter().any(|audit| audit["outcome"] == "rejected"));
}

#[tokio::test]
async fn admin_marketing_create_routes_reject_lifecycle_fields_on_create() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let template = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/coupon-templates")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "coupon_template_id":"template_create_guard",
                        "template_key":"create-guard",
                        "display_name":"Create Guard",
                        "status":"active",
                        "approval_state":"approved",
                        "distribution_kind":"unique_code",
                        "benefit":{"benefit_kind":"percentage_off","discount_percent":20},
                        "restriction":{"subject_scope":"project","stacking_policy":"exclusive"},
                        "created_at_ms":1710000000000,
                        "updated_at_ms":1710000000000
                    }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(template.status(), StatusCode::BAD_REQUEST);

    let campaign = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/campaigns")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "marketing_campaign_id":"campaign_create_guard",
                        "coupon_template_id":"template_create_guard",
                        "display_name":"Create Guard Campaign",
                        "status":"active",
                        "approval_state":"approved",
                        "created_at_ms":1710000000000,
                        "updated_at_ms":1710000000000
                    }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(campaign.status(), StatusCode::BAD_REQUEST);

    let budget = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/budgets")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "campaign_budget_id":"budget_create_guard",
                        "marketing_campaign_id":"campaign_create_guard",
                        "status":"active",
                        "total_budget_minor":500000,
                        "reserved_budget_minor":0,
                        "consumed_budget_minor":0,
                        "created_at_ms":1710000000000,
                        "updated_at_ms":1710000000000
                    }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(budget.status(), StatusCode::BAD_REQUEST);

    let code = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/codes")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "coupon_code_id":"code_create_guard",
                        "coupon_template_id":"template_create_guard",
                        "code_value":"CREATEGUARD",
                        "status":"reserved",
                        "created_at_ms":1710000000000,
                        "updated_at_ms":1710000000000
                    }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(code.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn admin_marketing_budget_lifecycle_routes_apply_coupon_semantic_actions() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let template = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/coupon-templates")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "coupon_template_id":"template_budget_lifecycle",
                        "template_key":"budget-lifecycle",
                        "display_name":"Budget Lifecycle",
                        "status":"draft",
                        "distribution_kind":"shared_code",
                        "benefit":{"benefit_kind":"percentage_off","discount_percent":15},
                        "restriction":{"subject_scope":"project","stacking_policy":"exclusive"},
                        "created_at_ms":1710000000000,
                        "updated_at_ms":1710000000000
                    }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(template.status(), StatusCode::CREATED);

    let campaign = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/campaigns")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "marketing_campaign_id":"campaign_budget_lifecycle",
                        "coupon_template_id":"template_budget_lifecycle",
                        "display_name":"Budget Lifecycle Campaign",
                        "status":"draft",
                        "created_at_ms":1710000000000,
                        "updated_at_ms":1710000000000
                    }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(campaign.status(), StatusCode::CREATED);

    let budget = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/budgets")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "campaign_budget_id":"budget_lifecycle_main",
                        "marketing_campaign_id":"campaign_budget_lifecycle",
                        "status":"draft",
                        "total_budget_minor":500000,
                        "reserved_budget_minor":0,
                        "consumed_budget_minor":0,
                        "created_at_ms":1710000000000,
                        "updated_at_ms":1710000000000
                    }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(budget.status(), StatusCode::CREATED);

    let activated = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/budgets/budget_lifecycle_main/activate")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .header("x-request-id", "sdkw-test-budget-activate-1")
                .body(Body::from(
                    r#"{"reason":"activate canonical campaign budget"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(activated.status(), StatusCode::OK);
    let activated_json = read_json(activated).await;
    assert_eq!(activated_json["detail"]["budget"]["status"], "active");
    assert_eq!(
        activated_json["detail"]["campaign"]["marketing_campaign_id"],
        "campaign_budget_lifecycle"
    );
    assert_eq!(activated_json["audit"]["action"], "activate");
    assert_eq!(activated_json["audit"]["outcome"], "applied");
    assert_eq!(activated_json["audit"]["previous_status"], "draft");
    assert_eq!(activated_json["audit"]["resulting_status"], "active");
    assert_eq!(
        activated_json["audit"]["reason"],
        "activate canonical campaign budget"
    );
    assert_eq!(
        activated_json["audit"]["operator_id"],
        "admin_local_default"
    );
    assert_eq!(
        activated_json["audit"]["request_id"],
        "sdkw-test-budget-activate-1"
    );

    let closed = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/budgets/budget_lifecycle_main/close")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .header("x-request-id", "sdkw-test-budget-close-1")
                .body(Body::from(
                    r#"{"reason":"close canonical campaign budget"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(closed.status(), StatusCode::OK);
    let closed_json = read_json(closed).await;
    assert_eq!(closed_json["detail"]["budget"]["status"], "closed");
    assert_eq!(closed_json["audit"]["action"], "close");
    assert_eq!(closed_json["audit"]["outcome"], "applied");
    assert_eq!(closed_json["audit"]["previous_status"], "active");
    assert_eq!(closed_json["audit"]["resulting_status"], "closed");
    assert_eq!(closed_json["audit"]["operator_id"], "admin_local_default");
    assert_eq!(
        closed_json["audit"]["request_id"],
        "sdkw-test-budget-close-1"
    );

    let audits = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/marketing/budgets/budget_lifecycle_main/lifecycle-audits")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(audits.status(), StatusCode::OK);
    let audits_json = read_json(audits).await;
    assert_eq!(audits_json.as_array().unwrap().len(), 2);
    assert_eq!(audits_json[0]["action"], "close");
    assert_eq!(audits_json[0]["request_id"], "sdkw-test-budget-close-1");
    assert_eq!(audits_json[1]["action"], "activate");
    assert_eq!(audits_json[1]["request_id"], "sdkw-test-budget-activate-1");
}

#[tokio::test]
async fn admin_marketing_budget_activate_rejects_budget_without_headroom() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let template = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/coupon-templates")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "coupon_template_id":"template_budget_reject",
                        "template_key":"budget-reject",
                        "display_name":"Budget Reject",
                        "status":"draft",
                        "distribution_kind":"shared_code",
                        "benefit":{"benefit_kind":"percentage_off","discount_percent":10},
                        "restriction":{"subject_scope":"project","stacking_policy":"exclusive"},
                        "created_at_ms":1710000000000,
                        "updated_at_ms":1710000000000
                    }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(template.status(), StatusCode::CREATED);

    let campaign = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/campaigns")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "marketing_campaign_id":"campaign_budget_reject",
                        "coupon_template_id":"template_budget_reject",
                        "display_name":"Budget Reject Campaign",
                        "status":"draft",
                        "created_at_ms":1710000000000,
                        "updated_at_ms":1710000000000
                    }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(campaign.status(), StatusCode::CREATED);

    let budget = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/budgets")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "campaign_budget_id":"budget_lifecycle_reject",
                        "marketing_campaign_id":"campaign_budget_reject",
                        "status":"draft",
                        "total_budget_minor":0,
                        "reserved_budget_minor":0,
                        "consumed_budget_minor":0,
                        "created_at_ms":1710000000000,
                        "updated_at_ms":1710000000000
                    }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(budget.status(), StatusCode::CREATED);

    let activated = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/budgets/budget_lifecycle_reject/activate")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .header("x-request-id", "sdkw-test-budget-activate-rejected-1")
                .body(Body::from(
                    r#"{"reason":"attempt activate exhausted campaign budget"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(activated.status(), StatusCode::BAD_REQUEST);
    assert!(read_json(activated).await["error"]["message"]
        .as_str()
        .unwrap()
        .contains("available headroom"));

    let audits = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/marketing/budgets/budget_lifecycle_reject/lifecycle-audits")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(audits.status(), StatusCode::OK);
    let audits_json = read_json(audits).await;
    assert_eq!(audits_json.as_array().unwrap().len(), 1);
    assert_eq!(audits_json[0]["action"], "activate");
    assert_eq!(audits_json[0]["outcome"], "rejected");
    assert_eq!(
        audits_json[0]["request_id"],
        "sdkw-test-budget-activate-rejected-1"
    );
    assert!(audits_json[0]["decision_reasons"]
        .as_array()
        .unwrap()
        .iter()
        .any(|reason| reason == "campaign budget has no available headroom"));
}

#[tokio::test]
async fn admin_marketing_coupon_code_lifecycle_routes_apply_coupon_semantic_actions() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;
    let now_ms = unix_timestamp_ms();

    let template = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/coupon-templates")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "coupon_template_id":"template_code_lifecycle",
                        "template_key":"code-lifecycle",
                        "display_name":"Code Lifecycle",
                        "status":"draft",
                        "distribution_kind":"unique_code",
                        "benefit":{"benefit_kind":"percentage_off","discount_percent":12},
                        "restriction":{"subject_scope":"project","stacking_policy":"exclusive"},
                        "created_at_ms":1710000000000,
                        "updated_at_ms":1710000000000
                    }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(template.status(), StatusCode::CREATED);

    let code = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/codes")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{
                        "coupon_code_id":"code_lifecycle_main",
                        "coupon_template_id":"template_code_lifecycle",
                        "code_value":"LIFECYCLE12",
                        "status":"available",
                        "expires_at_ms":{},
                        "created_at_ms":1710000000000,
                        "updated_at_ms":1710000000000
                    }}"#,
                    now_ms + 3_600_000,
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(code.status(), StatusCode::CREATED);

    let disabled = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/codes/code_lifecycle_main/disable")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .header("x-request-id", "sdkw-test-code-disable-1")
                .body(Body::from(r#"{"reason":"disable canonical coupon code"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(disabled.status(), StatusCode::OK);
    let disabled_json = read_json(disabled).await;
    assert_eq!(disabled_json["detail"]["coupon_code"]["status"], "disabled");
    assert_eq!(
        disabled_json["detail"]["coupon_template"]["coupon_template_id"],
        "template_code_lifecycle"
    );
    assert_eq!(disabled_json["audit"]["action"], "disable");
    assert_eq!(disabled_json["audit"]["outcome"], "applied");
    assert_eq!(disabled_json["audit"]["previous_status"], "available");
    assert_eq!(disabled_json["audit"]["resulting_status"], "disabled");
    assert_eq!(disabled_json["audit"]["operator_id"], "admin_local_default");
    assert_eq!(
        disabled_json["audit"]["request_id"],
        "sdkw-test-code-disable-1"
    );

    let restored = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/codes/code_lifecycle_main/restore")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .header("x-request-id", "sdkw-test-code-restore-1")
                .body(Body::from(r#"{"reason":"restore canonical coupon code"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(restored.status(), StatusCode::OK);
    let restored_json = read_json(restored).await;
    assert_eq!(
        restored_json["detail"]["coupon_code"]["status"],
        "available"
    );
    assert_eq!(restored_json["audit"]["action"], "restore");
    assert_eq!(restored_json["audit"]["outcome"], "applied");
    assert_eq!(restored_json["audit"]["previous_status"], "disabled");
    assert_eq!(restored_json["audit"]["resulting_status"], "available");
    assert_eq!(restored_json["audit"]["operator_id"], "admin_local_default");
    assert_eq!(
        restored_json["audit"]["request_id"],
        "sdkw-test-code-restore-1"
    );

    let audits = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/marketing/codes/code_lifecycle_main/lifecycle-audits")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(audits.status(), StatusCode::OK);
    let audits_json = read_json(audits).await;
    assert_eq!(audits_json.as_array().unwrap().len(), 2);
    assert_eq!(audits_json[0]["action"], "restore");
    assert_eq!(audits_json[0]["request_id"], "sdkw-test-code-restore-1");
    assert_eq!(audits_json[1]["action"], "disable");
    assert_eq!(audits_json[1]["request_id"], "sdkw-test-code-disable-1");
}

#[tokio::test]
async fn admin_marketing_coupon_code_restore_rejects_expired_coupon_code() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;
    let now_ms = unix_timestamp_ms();

    let template = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/coupon-templates")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "coupon_template_id":"template_code_reject",
                        "template_key":"code-reject",
                        "display_name":"Code Reject",
                        "status":"draft",
                        "distribution_kind":"unique_code",
                        "benefit":{"benefit_kind":"percentage_off","discount_percent":10},
                        "restriction":{"subject_scope":"project","stacking_policy":"exclusive"},
                        "created_at_ms":1710000000000,
                        "updated_at_ms":1710000000000
                    }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(template.status(), StatusCode::CREATED);

    let code = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/codes")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{
                        "coupon_code_id":"code_lifecycle_reject",
                        "coupon_template_id":"template_code_reject",
                        "code_value":"REJECT10",
                        "status":"available",
                        "expires_at_ms":{},
                        "created_at_ms":1710000000000,
                        "updated_at_ms":1710000000000
                    }}"#,
                    now_ms.saturating_sub(1_000),
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(code.status(), StatusCode::CREATED);

    let restored = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/codes/code_lifecycle_reject/restore")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .header("x-request-id", "sdkw-test-code-restore-rejected-1")
                .body(Body::from(
                    r#"{"reason":"attempt restore expired coupon code"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(restored.status(), StatusCode::BAD_REQUEST);
    assert!(read_json(restored).await["error"]["message"]
        .as_str()
        .unwrap()
        .contains("expired"));

    let audits = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/marketing/codes/code_lifecycle_reject/lifecycle-audits")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(audits.status(), StatusCode::OK);
    let audits_json = read_json(audits).await;
    assert_eq!(audits_json.as_array().unwrap().len(), 1);
    assert_eq!(audits_json[0]["action"], "restore");
    assert_eq!(audits_json[0]["outcome"], "rejected");
    assert_eq!(
        audits_json[0]["request_id"],
        "sdkw-test-code-restore-rejected-1"
    );
    assert!(audits_json[0]["decision_reasons"]
        .as_array()
        .unwrap()
        .iter()
        .any(|reason| reason == "coupon code is expired and cannot be restored"));
}

#[tokio::test]
async fn admin_marketing_coupon_template_lifecycle_routes_apply_coupon_semantic_actions() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;
    let now_ms = unix_timestamp_ms();

    let publish_template = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/coupon-templates")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "coupon_template_id":"template_lifecycle_publish",
                        "template_key":"template-lifecycle-publish",
                        "display_name":"Template Lifecycle Publish",
                        "status":"draft",
                        "distribution_kind":"shared_code",
                        "benefit":{"benefit_kind":"percentage_off","discount_percent":12},
                        "restriction":{"subject_scope":"project","stacking_policy":"exclusive"},
                        "created_at_ms":1710000000000,
                        "updated_at_ms":1710000000000
                    }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(publish_template.status(), StatusCode::CREATED);
    approve_coupon_template(
        &app,
        &token,
        "template_lifecycle_publish",
        "sdkw-test-template-lifecycle-publish",
    )
    .await;

    let schedule_template = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/coupon-templates")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{
                        "coupon_template_id":"template_lifecycle_schedule",
                        "template_key":"template-lifecycle-schedule",
                        "display_name":"Template Lifecycle Schedule",
                        "status":"draft",
                        "distribution_kind":"shared_code",
                        "benefit":{{"benefit_kind":"percentage_off","discount_percent":18}},
                        "restriction":{{"subject_scope":"project","stacking_policy":"exclusive"}},
                        "activation_at_ms":{},
                        "created_at_ms":1710000000000,
                        "updated_at_ms":1710000000000
                    }}"#,
                    now_ms + 3_600_000,
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(schedule_template.status(), StatusCode::CREATED);
    approve_coupon_template(
        &app,
        &token,
        "template_lifecycle_schedule",
        "sdkw-test-template-lifecycle-schedule",
    )
    .await;

    let published = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/coupon-templates/template_lifecycle_publish/publish")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .header("x-request-id", "sdkw-test-template-publish-1")
                .body(Body::from(
                    r#"{"reason":"publish canonical coupon template"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(published.status(), StatusCode::OK);
    let published_json = read_json(published).await;
    assert_eq!(
        published_json["detail"]["coupon_template"]["status"],
        "active"
    );
    assert_eq!(published_json["audit"]["action"], "publish");
    assert_eq!(published_json["audit"]["previous_status"], "draft");
    assert_eq!(published_json["audit"]["resulting_status"], "active");
    assert_eq!(
        published_json["audit"]["reason"],
        "publish canonical coupon template"
    );
    assert_eq!(
        published_json["audit"]["operator_id"],
        "admin_local_default"
    );
    assert_eq!(
        published_json["audit"]["request_id"],
        "sdkw-test-template-publish-1"
    );

    let scheduled = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/coupon-templates/template_lifecycle_schedule/schedule")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .header("x-request-id", "sdkw-test-template-schedule-1")
                .body(Body::from(
                    r#"{"reason":"schedule canonical coupon template"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(scheduled.status(), StatusCode::OK);
    let scheduled_json = read_json(scheduled).await;
    assert_eq!(
        scheduled_json["detail"]["coupon_template"]["status"],
        "scheduled"
    );
    assert_eq!(scheduled_json["audit"]["action"], "schedule");
    assert_eq!(scheduled_json["audit"]["previous_status"], "draft");
    assert_eq!(scheduled_json["audit"]["resulting_status"], "scheduled");
    assert_eq!(scheduled_json["audit"]["outcome"], "applied");

    let retired = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/coupon-templates/template_lifecycle_publish/retire")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .header("x-request-id", "sdkw-test-template-retire-1")
                .body(Body::from(
                    r#"{"reason":"retire canonical coupon template"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(retired.status(), StatusCode::OK);
    let retired_json = read_json(retired).await;
    assert_eq!(
        retired_json["detail"]["coupon_template"]["status"],
        "archived"
    );
    assert_eq!(retired_json["audit"]["action"], "retire");
    assert_eq!(retired_json["audit"]["previous_status"], "active");
    assert_eq!(retired_json["audit"]["resulting_status"], "archived");
    assert_eq!(retired_json["audit"]["outcome"], "applied");
    assert_eq!(retired_json["audit"]["operator_id"], "admin_local_default");
    assert_eq!(
        retired_json["audit"]["request_id"],
        "sdkw-test-template-retire-1"
    );

    let publish_audits = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(
                    "/admin/marketing/coupon-templates/template_lifecycle_publish/lifecycle-audits",
                )
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(publish_audits.status(), StatusCode::OK);
    let publish_audits_json = read_json(publish_audits).await;
    assert_eq!(publish_audits_json.as_array().unwrap().len(), 4);
    assert_eq!(publish_audits_json[0]["action"], "retire");
    assert_eq!(
        publish_audits_json[0]["request_id"],
        "sdkw-test-template-retire-1"
    );
    assert_eq!(publish_audits_json[1]["action"], "publish");
    assert_eq!(
        publish_audits_json[1]["request_id"],
        "sdkw-test-template-publish-1"
    );
    assert_eq!(publish_audits_json[2]["action"], "approve");
    assert_eq!(
        publish_audits_json[2]["request_id"],
        "sdkw-test-template-lifecycle-publish-approve"
    );
    assert_eq!(publish_audits_json[3]["action"], "submit_for_approval");
    assert_eq!(
        publish_audits_json[3]["request_id"],
        "sdkw-test-template-lifecycle-publish-submit"
    );

    let schedule_audits = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(
                    "/admin/marketing/coupon-templates/template_lifecycle_schedule/lifecycle-audits",
                )
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(schedule_audits.status(), StatusCode::OK);
    let schedule_audits_json = read_json(schedule_audits).await;
    assert_eq!(schedule_audits_json.as_array().unwrap().len(), 3);
    assert_eq!(schedule_audits_json[0]["action"], "schedule");
    assert_eq!(
        schedule_audits_json[0]["request_id"],
        "sdkw-test-template-schedule-1"
    );
    assert_eq!(schedule_audits_json[1]["action"], "approve");
    assert_eq!(
        schedule_audits_json[1]["request_id"],
        "sdkw-test-template-lifecycle-schedule-approve"
    );
    assert_eq!(schedule_audits_json[2]["action"], "submit_for_approval");
    assert_eq!(
        schedule_audits_json[2]["request_id"],
        "sdkw-test-template-lifecycle-schedule-submit"
    );
}

#[tokio::test]
async fn admin_marketing_coupon_template_publish_rejects_future_activation() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;
    let now_ms = unix_timestamp_ms();

    let template = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/coupon-templates")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{
                        "coupon_template_id":"template_publish_future_activation",
                        "template_key":"publish-future-activation",
                        "display_name":"Publish Future Activation",
                        "status":"draft",
                        "distribution_kind":"shared_code",
                        "benefit":{{"benefit_kind":"percentage_off","discount_percent":20}},
                        "restriction":{{"subject_scope":"project","stacking_policy":"exclusive"}},
                        "activation_at_ms":{},
                        "created_at_ms":1710000000000,
                        "updated_at_ms":1710000000000
                    }}"#,
                    now_ms + 600_000,
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(template.status(), StatusCode::CREATED);
    approve_coupon_template(
        &app,
        &token,
        "template_publish_future_activation",
        "sdkw-test-template-publish-future-activation",
    )
    .await;

    let published = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/coupon-templates/template_publish_future_activation/publish")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .header("x-request-id", "sdkw-test-template-publish-rejected-1")
                .body(Body::from(
                    r#"{"reason":"attempt premature coupon template publish"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(published.status(), StatusCode::BAD_REQUEST);
    assert!(read_json(published).await["error"]["message"]
        .as_str()
        .unwrap()
        .contains("future activation_at_ms"));

    let audits = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(
                    "/admin/marketing/coupon-templates/template_publish_future_activation/lifecycle-audits",
                )
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(audits.status(), StatusCode::OK);
    let audits_json = read_json(audits).await;
    assert_eq!(audits_json.as_array().unwrap().len(), 3);
    assert_eq!(audits_json[0]["action"], "publish");
    assert_eq!(audits_json[0]["outcome"], "rejected");
    assert_eq!(
        audits_json[0]["request_id"],
        "sdkw-test-template-publish-rejected-1"
    );
    assert_eq!(audits_json[1]["action"], "approve");
    assert_eq!(
        audits_json[1]["request_id"],
        "sdkw-test-template-publish-future-activation-approve"
    );
    assert_eq!(audits_json[2]["action"], "submit_for_approval");
    assert_eq!(
        audits_json[2]["request_id"],
        "sdkw-test-template-publish-future-activation-submit"
    );
    assert!(audits_json[0]["decision_reasons"]
        .as_array()
        .unwrap()
        .iter()
        .any(|reason| reason
            == "coupon template has future activation_at_ms and must be scheduled before publish"));
}

#[tokio::test]
async fn admin_marketing_coupon_template_revision_governance_routes_clone_compare_and_approve_templates(
) {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let source_template = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/coupon-templates")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "coupon_template_id":"template_revision_source",
                        "template_key":"template-revision-source",
                        "display_name":"Template Revision Source",
                        "status":"draft",
                        "approval_state":"approved",
                        "revision":1,
                        "distribution_kind":"shared_code",
                        "benefit":{"benefit_kind":"percentage_off","discount_percent":15},
                        "restriction":{"subject_scope":"project","stacking_policy":"exclusive"},
                        "created_at_ms":1710000000000,
                        "updated_at_ms":1710000000000
                    }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(source_template.status(), StatusCode::CREATED);

    let cloned = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/coupon-templates/template_revision_source/clone")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .header("x-request-id", "sdkw-test-template-clone-1")
                .body(Body::from(
                    r#"{
                        "coupon_template_id":"template_revision_clone",
                        "template_key":"template-revision-clone",
                        "display_name":"Template Revision Clone",
                        "reason":"clone coupon template into governed draft"
                    }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(cloned.status(), StatusCode::CREATED);
    let cloned_json = read_json(cloned).await;
    assert_eq!(
        cloned_json["detail"]["coupon_template"]["coupon_template_id"],
        "template_revision_clone"
    );
    assert_eq!(cloned_json["detail"]["coupon_template"]["status"], "draft");
    assert_eq!(
        cloned_json["detail"]["coupon_template"]["approval_state"],
        "draft"
    );
    assert_eq!(cloned_json["detail"]["coupon_template"]["revision"], 2);
    assert_eq!(
        cloned_json["detail"]["coupon_template"]["parent_coupon_template_id"],
        "template_revision_source"
    );
    assert_eq!(
        cloned_json["detail"]["coupon_template"]["root_coupon_template_id"],
        "template_revision_source"
    );
    assert_eq!(cloned_json["audit"]["action"], "clone");
    assert_eq!(cloned_json["audit"]["outcome"], "applied");
    assert_eq!(
        cloned_json["audit"]["source_coupon_template_id"],
        "template_revision_source"
    );
    assert_eq!(cloned_json["audit"]["previous_revision"], 1);
    assert_eq!(cloned_json["audit"]["resulting_revision"], 2);

    let comparison = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/coupon-templates/template_revision_source/compare")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"target_coupon_template_id":"template_revision_clone"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(comparison.status(), StatusCode::OK);
    let comparison_json = read_json(comparison).await;
    assert_eq!(comparison_json["same_lineage"], true);
    assert_eq!(
        comparison_json["source_coupon_template"]["coupon_template_id"],
        "template_revision_source"
    );
    assert_eq!(
        comparison_json["target_coupon_template"]["coupon_template_id"],
        "template_revision_clone"
    );
    assert!(comparison_json["field_changes"]
        .as_array()
        .unwrap()
        .iter()
        .any(|change| change["field"] == "display_name"));

    let rejected_publish = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/coupon-templates/template_revision_clone/publish")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .header(
                    "x-request-id",
                    "sdkw-test-template-clone-publish-rejected-1",
                )
                .body(Body::from(
                    r#"{"reason":"attempt publish before approval"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(rejected_publish.status(), StatusCode::BAD_REQUEST);
    assert!(read_json(rejected_publish).await["error"]["message"]
        .as_str()
        .unwrap()
        .contains("approved before publish"));

    let submitted = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(
                    "/admin/marketing/coupon-templates/template_revision_clone/submit-for-approval",
                )
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .header("x-request-id", "sdkw-test-template-submit-1")
                .body(Body::from(
                    r#"{"reason":"submit governed revision for approval"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(submitted.status(), StatusCode::OK);
    let submitted_json = read_json(submitted).await;
    assert_eq!(
        submitted_json["detail"]["coupon_template"]["approval_state"],
        "in_review"
    );
    assert_eq!(submitted_json["audit"]["action"], "submit_for_approval");

    let approved = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/coupon-templates/template_revision_clone/approve")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .header("x-request-id", "sdkw-test-template-approve-1")
                .body(Body::from(r#"{"reason":"approve governed revision"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(approved.status(), StatusCode::OK);
    let approved_json = read_json(approved).await;
    assert_eq!(
        approved_json["detail"]["coupon_template"]["approval_state"],
        "approved"
    );
    assert_eq!(approved_json["audit"]["action"], "approve");

    let published = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/coupon-templates/template_revision_clone/publish")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .header("x-request-id", "sdkw-test-template-clone-publish-1")
                .body(Body::from(
                    r#"{"reason":"publish approved coupon revision"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(published.status(), StatusCode::OK);
    let published_json = read_json(published).await;
    assert_eq!(
        published_json["detail"]["coupon_template"]["status"],
        "active"
    );
    assert_eq!(
        published_json["detail"]["coupon_template"]["approval_state"],
        "approved"
    );

    let audits = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/marketing/coupon-templates/template_revision_clone/lifecycle-audits")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(audits.status(), StatusCode::OK);
    let audits_json = read_json(audits).await;
    let audits = audits_json.as_array().unwrap();
    assert_eq!(audits.len(), 5);
    assert!(audits.iter().any(|audit| audit["action"] == "clone"));
    assert!(audits
        .iter()
        .any(|audit| audit["action"] == "submit_for_approval"));
    assert!(audits.iter().any(|audit| audit["action"] == "approve"));
    assert!(audits.iter().any(|audit| audit["action"] == "publish"));
    assert!(audits.iter().any(|audit| audit["outcome"] == "rejected"));
}

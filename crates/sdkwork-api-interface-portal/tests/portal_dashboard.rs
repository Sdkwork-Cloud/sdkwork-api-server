use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use serde_json::Value;
use sqlx::SqlitePool;
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

async fn portal_token(app: axum::Router) -> String {
    let register_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"email\":\"portal@example.com\",\"password\":\"hunter2!\",\"display_name\":\"Portal User\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(register_response.status(), StatusCode::CREATED);
    read_json(register_response).await["token"]
        .as_str()
        .unwrap()
        .to_owned()
}

async fn portal_workspace(app: axum::Router, token: &str) -> Value {
    let workspace_response = app
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

    assert_eq!(workspace_response.status(), StatusCode::OK);
    read_json(workspace_response).await
}

#[tokio::test]
async fn portal_dashboard_and_usage_views_are_project_scoped() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool.clone());
    let token = portal_token(app.clone()).await;
    let workspace = portal_workspace(app.clone(), &token).await;
    let project_id = workspace["project"]["id"].as_str().unwrap().to_owned();

    sqlx::query(
        "INSERT INTO usage_records (project_id, model, provider_id, units, amount, created_at_ms)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&project_id)
    .bind("gpt-4.1")
    .bind("provider-openai")
    .bind(240_i64)
    .bind(0.42_f64)
    .bind(1_710_000_001_i64)
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO usage_records (project_id, model, provider_id, units, amount, created_at_ms)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind("project-other")
    .bind("gpt-4.1-mini")
    .bind("provider-other")
    .bind(999_i64)
    .bind(9.99_f64)
    .bind(1_710_000_002_i64)
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query("INSERT INTO billing_ledger_entries (project_id, units, amount) VALUES (?, ?, ?)")
        .bind(&project_id)
        .bind(240_i64)
        .bind(0.42_f64)
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO billing_ledger_entries (project_id, units, amount) VALUES (?, ?, ?)")
        .bind("project-other")
        .bind(999_i64)
        .bind(9.99_f64)
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query(
        "INSERT INTO billing_quota_policies (policy_id, project_id, max_units, enabled)
         VALUES (?, ?, ?, ?)",
    )
    .bind("quota-portal")
    .bind(&project_id)
    .bind(500_i64)
    .bind(1_i64)
    .execute(&pool)
    .await
    .unwrap();

    let dashboard_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/dashboard")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(dashboard_response.status(), StatusCode::OK);
    let dashboard_json = read_json(dashboard_response).await;
    assert_eq!(dashboard_json["workspace"]["project"]["id"], project_id);
    assert_eq!(dashboard_json["usage_summary"]["total_requests"], 1);
    assert_eq!(dashboard_json["billing_summary"]["project_id"], project_id);
    assert_eq!(dashboard_json["billing_summary"]["used_units"], 240);
    assert_eq!(
        dashboard_json["recent_requests"].as_array().unwrap().len(),
        1
    );
    assert_eq!(dashboard_json["recent_requests"][0]["units"], 240);

    let usage_records_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/usage/records")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(usage_records_response.status(), StatusCode::OK);
    let usage_records_json = read_json(usage_records_response).await;
    assert_eq!(usage_records_json.as_array().unwrap().len(), 1);
    assert_eq!(usage_records_json[0]["project_id"], project_id);
    assert_eq!(usage_records_json[0]["units"], 240);

    let usage_summary_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/usage/summary")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(usage_summary_response.status(), StatusCode::OK);
    let usage_summary_json = read_json(usage_summary_response).await;
    assert_eq!(usage_summary_json["total_requests"], 1);

    let billing_summary_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/billing/summary")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(billing_summary_response.status(), StatusCode::OK);
    let billing_summary_json = read_json(billing_summary_response).await;
    assert_eq!(billing_summary_json["project_id"], project_id);
    assert_eq!(billing_summary_json["used_units"], 240);
    assert_eq!(billing_summary_json["remaining_units"], 260);

    let billing_ledger_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/billing/ledger")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(billing_ledger_response.status(), StatusCode::OK);
    let billing_ledger_json = read_json(billing_ledger_response).await;
    assert_eq!(billing_ledger_json.as_array().unwrap().len(), 1);
    assert_eq!(billing_ledger_json[0]["project_id"], project_id);
    assert_eq!(billing_ledger_json[0]["units"], 240);
}

#[tokio::test]
async fn new_portal_dashboard_routes_require_authentication() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool);

    for route in [
        "/portal/dashboard",
        "/portal/usage/records",
        "/portal/usage/summary",
        "/portal/billing/summary",
        "/portal/billing/ledger",
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(route)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "route {route}");
    }
}

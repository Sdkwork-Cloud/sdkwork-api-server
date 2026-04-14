use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use axum::response::Response;
use serde_json::Value;
use sqlx::SqlitePool;
use tower::ServiceExt;

mod support;

async fn read_json(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

async fn memory_pool() -> SqlitePool {
    sdkwork_api_storage_sqlite::run_migrations("sqlite::memory:")
        .await
        .unwrap()
}

fn header_text<'a>(response: &'a Response, name: &str) -> &'a str {
    response
        .headers()
        .get(name)
        .and_then(|value| value.to_str().ok())
        .unwrap()
}

#[tokio::test]
async fn gateway_chat_completions_returns_429_when_rate_limit_is_exhausted() {
    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key(&pool, "tenant-1", "project-1").await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    let create_policy = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/gateway/rate-limit-policies")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"policy_id\":\"rate-project-1-chat\",\"project_id\":\"project-1\",\"route_key\":\"/v1/chat/completions\",\"requests_per_window\":1,\"window_seconds\":60,\"enabled\":true}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_policy.status(), StatusCode::CREATED);

    let first = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"messages\":[{\"role\":\"user\",\"content\":\"hi\"}]}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(first.status().is_success());
    assert_eq!(
        header_text(&first, "x-ratelimit-policy"),
        "rate-project-1-chat"
    );
    assert_eq!(header_text(&first, "x-ratelimit-limit"), "1");
    assert_eq!(header_text(&first, "x-ratelimit-remaining"), "0");
    let success_reset = header_text(&first, "x-ratelimit-reset")
        .parse::<u64>()
        .unwrap();
    assert!(success_reset <= 60);
    assert!(success_reset > 0);

    let second = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"messages\":[{\"role\":\"user\",\"content\":\"hi again\"}]}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(second.status(), StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(
        header_text(&second, "x-ratelimit-policy"),
        "rate-project-1-chat"
    );
    assert_eq!(header_text(&second, "x-ratelimit-limit"), "1");
    assert_eq!(header_text(&second, "x-ratelimit-remaining"), "0");
    let retry_after = header_text(&second, "retry-after").parse::<u64>().unwrap();
    let rejected_reset = header_text(&second, "x-ratelimit-reset")
        .parse::<u64>()
        .unwrap();
    assert!(retry_after <= 60);
    assert!(retry_after > 0);
    assert_eq!(rejected_reset, retry_after);
    let json = read_json(second).await;
    assert_eq!(json["error"]["code"], "rate_limit_exceeded");
}

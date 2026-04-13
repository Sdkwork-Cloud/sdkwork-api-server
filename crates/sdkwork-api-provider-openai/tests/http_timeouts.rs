use axum::routing::get;
use axum::{Json, Router};
use serde_json::{json, Value};
use tokio::net::TcpListener;

#[tokio::test]
async fn adapter_classifies_connect_failures_as_retryable_failures() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    drop(listener);

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));

    let error = adapter
        .list_models("sk-upstream-openai")
        .await
        .expect_err("closed port should fail to connect");
    let transport_error = error.downcast_ref::<reqwest::Error>().expect("reqwest error");

    assert!(transport_error.is_connect());
}

#[tokio::test]
async fn adapter_classifies_retryable_http_status_failures() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let app = Router::new().route("/v1/models", get(unavailable_models_handler));

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let error = adapter
        .list_models("sk-upstream-openai")
        .await
        .expect_err("502 should surface as retryable upstream failure");
    let http_error = error
        .downcast_ref::<sdkwork_api_provider_core::ProviderHttpError>()
        .expect("provider http error");

    assert_eq!(http_error.status(), Some(reqwest::StatusCode::BAD_GATEWAY));
    assert!(
        http_error
            .body_excerpt()
            .is_some_and(|body| body.contains("temporary upstream outage"))
    );
}

async fn unavailable_models_handler() -> (axum::http::StatusCode, Json<Value>) {
    (
        axum::http::StatusCode::BAD_GATEWAY,
        Json(json!({
            "error": {
                "message": "temporary upstream outage"
            }
        })),
    )
}

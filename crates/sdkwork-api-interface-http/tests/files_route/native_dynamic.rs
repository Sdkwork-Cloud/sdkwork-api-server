use super::*;
use serial_test::serial;

#[serial(extension_env)]
#[tokio::test]
async fn stateful_file_content_route_relays_to_native_dynamic_provider() {
    let fixture = support::prepare_native_dynamic_mock_package("native-dynamic-file-content");

    let LocalFilesTestContext {
        admin_app,
        admin_token,
        api_key,
        gateway_app,
    } = local_files_test_context("tenant-1", "project-1").await;

    let create_channel = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/channels")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"id\":\"openai\",\"name\":\"OpenAI\"}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_channel.status(), StatusCode::CREATED);

    let provider = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"id\":\"provider-native-mock\",\"channel_id\":\"openai\",\"adapter_kind\":\"native-dynamic\",\"base_url\":\"https://native-dynamic.invalid/v1\",\"display_name\":\"Native Mock\",\"extension_id\":\"sdkwork.provider.native.mock\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(provider.status(), StatusCode::CREATED);

    let credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"tenant_id\":\"tenant-1\",\"provider_id\":\"provider-native-mock\",\"key_reference\":\"cred-native-mock\",\"secret_value\":\"sk-native\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(credential.status(), StatusCode::CREATED);

    let routing_policy = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "policy_id": "route-files-to-native-mock",
                        "capability": "files",
                        "model_pattern": "*",
                        "enabled": true,
                        "priority": 100,
                        "ordered_provider_ids": ["provider-native-mock"]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(routing_policy.status(), StatusCode::CREATED);

    let installation = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/extensions/installations")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "installation_id": "native-mock-installation",
                        "extension_id": "sdkwork.provider.native.mock",
                        "runtime": "native_dynamic",
                        "enabled": true,
                        "entrypoint": fixture.library_path.to_string_lossy(),
                        "config": {}
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(installation.status(), StatusCode::CREATED);

    let instance = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/extensions/instances")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "instance_id": "provider-native-mock",
                        "installation_id": "native-mock-installation",
                        "extension_id": "sdkwork.provider.native.mock",
                        "enabled": true,
                        "base_url": "https://native-dynamic.invalid/v1",
                        "credential_ref": "cred-native-mock",
                        "config": {}
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(instance.status(), StatusCode::CREATED);

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/files/file_1/content")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok()),
        Some("application/jsonl")
    );
    let content_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(&content_bytes[..], b"{\"source\":\"native_dynamic\"}\n");
}

use sdkwork_api_app_gateway::{
    create_webhook, delete_webhook, get_webhook, list_webhooks, update_webhook,
};

fn assert_error_contains<T: std::fmt::Debug, E: std::fmt::Display>(
    result: Result<T, E>,
    expected: &str,
) {
    let error = result.expect_err("expected error");
    assert!(
        error.to_string().contains(expected),
        "expected error containing `{expected}`, got `{error}`"
    );
}

#[test]
fn local_webhook_fallback_requires_upstream_provider() {
    let created = create_webhook(
        "tenant-1",
        "project-1",
        "https://example.com/webhook",
        &["response.completed".to_owned()],
    )
    .expect("create webhook should use local fallback");
    assert_eq!(created.id, "wh_1");
    assert_eq!(created.url, "https://example.com/webhook");

    let listed = list_webhooks("tenant-1", "project-1")
        .expect("list webhooks should use local fallback");
    assert_eq!(listed.data.len(), 1);
    assert_eq!(listed.data[0].id, "wh_1");
    assert_eq!(listed.data[0].url, "https://example.com/webhook");
}

#[test]
fn local_webhook_fallback_requires_persisted_webhook_state() {
    let retrieved = get_webhook("tenant-1", "project-1", "wh_1")
        .expect("get webhook should synthesize local state");
    assert_eq!(retrieved.id, "wh_1");
    assert_eq!(retrieved.url, "https://example.com/wh_1");

    let updated = update_webhook(
        "tenant-1",
        "project-1",
        "wh_1",
        "https://example.com/webhook/v2",
    )
    .expect("update webhook should succeed for local fallback");
    assert_eq!(updated.id, "wh_1");
    assert_eq!(updated.url, "https://example.com/webhook/v2");

    let deleted = delete_webhook("tenant-1", "project-1", "wh_1")
        .expect("delete webhook should succeed for local fallback");
    assert_eq!(deleted.id, "wh_1");
    assert!(deleted.deleted);

    assert_error_contains(
        get_webhook("tenant-1", "project-1", "wh_missing"),
        "webhook not found",
    );
    assert_error_contains(
        update_webhook(
            "tenant-1",
            "project-1",
            "wh_missing",
            "https://example.com/webhook/v2",
        ),
        "webhook not found",
    );
    assert_error_contains(
        delete_webhook("tenant-1", "project-1", "wh_missing"),
        "webhook not found",
    );
}

#[test]
fn local_webhook_create_requires_url() {
    assert_error_contains(
        create_webhook("tenant-1", "project-1", "   ", &["response.completed".to_owned()]),
        "Webhook url is required",
    );
}

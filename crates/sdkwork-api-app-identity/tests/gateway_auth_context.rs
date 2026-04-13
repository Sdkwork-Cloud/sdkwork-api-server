use sdkwork_api_app_identity::{gateway_auth_subject_from_request_context, GatewayRequestContext};

fn request_context(
    tenant_id: &str,
    project_id: &str,
    api_key_hash: &str,
) -> GatewayRequestContext {
    GatewayRequestContext {
        tenant_id: tenant_id.to_owned(),
        project_id: project_id.to_owned(),
        environment: "live".to_owned(),
        api_key_hash: api_key_hash.to_owned(),
        api_key_group_id: Some("group-live".to_owned()),
        canonical_tenant_id: None,
        canonical_organization_id: None,
        canonical_user_id: None,
        canonical_api_key_id: None,
    }
}

#[test]
fn gateway_request_context_maps_to_stable_commercial_subject_in_i64_safe_range() {
    let context = request_context(
        "tenant-gateway-commercial",
        "project-gateway-commercial",
        "hash_live_gateway_project",
    );

    let first = gateway_auth_subject_from_request_context(&context);
    let second = gateway_auth_subject_from_request_context(&context);

    assert_eq!(first, second);
    assert!(first.tenant_id > 0);
    assert!(first.organization_id > 0);
    assert!(first.user_id > 0);
    assert!(first.api_key_id.is_some());
    assert!(first.tenant_id <= i64::MAX as u64);
    assert!(first.organization_id <= i64::MAX as u64);
    assert!(first.user_id <= i64::MAX as u64);
    assert!(first.api_key_id.unwrap() <= i64::MAX as u64);
    assert_eq!(first.api_key_hash.as_deref(), Some("hash_live_gateway_project"));
    assert_eq!(first.platform.as_deref(), Some("gateway"));
    assert_eq!(
        first.owner.as_deref(),
        Some("project:tenant-gateway-commercial:project-gateway-commercial")
    );
}

#[test]
fn different_projects_split_payer_principals_but_keep_tenant_identity_stable() {
    let left = gateway_auth_subject_from_request_context(&request_context(
        "tenant-gateway-commercial",
        "project-alpha",
        "hash_live_gateway_project",
    ));
    let right = gateway_auth_subject_from_request_context(&request_context(
        "tenant-gateway-commercial",
        "project-beta",
        "hash_live_gateway_project",
    ));

    assert_eq!(left.tenant_id, right.tenant_id);
    assert_ne!(left.organization_id, right.organization_id);
    assert_ne!(left.user_id, right.user_id);
    assert_eq!(left.api_key_id, right.api_key_id);
}

#[test]
fn different_api_keys_keep_payer_principal_but_split_api_key_identity() {
    let left = gateway_auth_subject_from_request_context(&request_context(
        "tenant-gateway-commercial",
        "project-alpha",
        "hash-live-a",
    ));
    let right = gateway_auth_subject_from_request_context(&request_context(
        "tenant-gateway-commercial",
        "project-alpha",
        "hash-live-b",
    ));

    assert_eq!(left.tenant_id, right.tenant_id);
    assert_eq!(left.organization_id, right.organization_id);
    assert_eq!(left.user_id, right.user_id);
    assert_ne!(left.api_key_id, right.api_key_id);
    assert_ne!(left.request_principal, right.request_principal);
}

use sdkwork_api_app_identity::{hash_gateway_api_key, issue_jwt, verify_jwt};

const TEST_SIGNING_SECRET: &str = "test-admin-signing-secret";

#[test]
fn gateway_api_key_hash_is_not_plaintext() {
    let hash = hash_gateway_api_key("skw_live_example");
    assert_ne!(hash, "skw_live_example");
}

#[test]
fn issued_jwt_verifies() {
    let token = issue_jwt("user-1", TEST_SIGNING_SECRET).unwrap();
    let claims = verify_jwt(&token, TEST_SIGNING_SECRET).unwrap();
    assert_eq!(claims.sub, "user-1");
    assert_eq!(claims.iss, "sdkwork-admin");
    assert_eq!(claims.aud, "sdkwork-admin-ui");
    assert!(claims.exp >= claims.iat);
}

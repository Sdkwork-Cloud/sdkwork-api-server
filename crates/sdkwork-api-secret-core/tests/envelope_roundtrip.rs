use sdkwork_api_secret_core::{decrypt, encrypt};

#[test]
fn encrypt_roundtrip_returns_original_secret() {
    let envelope = encrypt("master-key", "sk-upstream").unwrap();
    let secret = decrypt("master-key", &envelope).unwrap();
    assert_eq!(secret, "sk-upstream");
}

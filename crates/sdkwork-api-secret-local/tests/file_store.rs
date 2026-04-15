use std::time::{SystemTime, UNIX_EPOCH};

use sdkwork_api_secret_core::{encrypt, CredentialSecretRef};
use sdkwork_api_secret_local::LocalEncryptedFileSecretStore;

#[test]
fn stores_and_loads_encrypted_envelope_in_local_file() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "sdkwork-api-secret-local-{}-{unique}.json",
        std::process::id()
    ));
    let store = LocalEncryptedFileSecretStore::new(&path);
    let secret_ref =
        CredentialSecretRef::new("tenant-1", "provider-openai-official", "cred-openai");
    let envelope = encrypt("local-dev-master-key", "sk-upstream-openai").unwrap();

    store.store_envelope(&secret_ref, &envelope).unwrap();

    let loaded = store
        .load_envelope(&secret_ref)
        .unwrap()
        .expect("secret envelope");
    assert_eq!(loaded, envelope);

    assert!(store.delete_envelope(&secret_ref).unwrap());
    assert!(store.load_envelope(&secret_ref).unwrap().is_none());

    let _ = std::fs::remove_file(path);
}

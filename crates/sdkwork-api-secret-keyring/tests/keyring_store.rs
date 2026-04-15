use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use sdkwork_api_secret_core::{encrypt, CredentialSecretRef};
use sdkwork_api_secret_keyring::{KeyringBackend, KeyringSecretStore};

#[derive(Debug, Default)]
struct MemoryKeyringBackend {
    entries: Mutex<HashMap<(String, String), String>>,
}

impl KeyringBackend for MemoryKeyringBackend {
    fn set_password(&self, service: &str, username: &str, secret: &str) -> Result<()> {
        self.entries
            .lock()
            .unwrap()
            .insert((service.to_owned(), username.to_owned()), secret.to_owned());
        Ok(())
    }

    fn get_password(&self, service: &str, username: &str) -> Result<Option<String>> {
        Ok(self
            .entries
            .lock()
            .unwrap()
            .get(&(service.to_owned(), username.to_owned()))
            .cloned())
    }

    fn delete_password(&self, service: &str, username: &str) -> Result<bool> {
        Ok(self
            .entries
            .lock()
            .unwrap()
            .remove(&(service.to_owned(), username.to_owned()))
            .is_some())
    }
}

#[test]
fn stores_and_loads_encrypted_envelope_in_keyring_backend() {
    let store = KeyringSecretStore::with_backend(
        "sdkwork-api-server-test",
        Arc::new(MemoryKeyringBackend::default()),
    );
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
}

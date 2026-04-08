use super::*;

#[async_trait]
pub trait IdentityKernelStore: AdminStore {
    async fn insert_identity_user_record(
        &self,
        _record: &IdentityUserRecord,
    ) -> Result<IdentityUserRecord> {
        Err(unsupported_identity_kernel_method(
            self.dialect(),
            "insert_identity_user_record",
        ))
    }

    async fn list_identity_user_records(&self) -> Result<Vec<IdentityUserRecord>> {
        Err(unsupported_identity_kernel_method(
            self.dialect(),
            "list_identity_user_records",
        ))
    }

    async fn find_identity_user_record(&self, _user_id: u64) -> Result<Option<IdentityUserRecord>> {
        Err(unsupported_identity_kernel_method(
            self.dialect(),
            "find_identity_user_record",
        ))
    }

    async fn insert_canonical_api_key_record(
        &self,
        _record: &CanonicalApiKeyRecord,
    ) -> Result<CanonicalApiKeyRecord> {
        Err(unsupported_identity_kernel_method(
            self.dialect(),
            "insert_canonical_api_key_record",
        ))
    }

    async fn find_canonical_api_key_record_by_hash(
        &self,
        _key_hash: &str,
    ) -> Result<Option<CanonicalApiKeyRecord>> {
        Err(unsupported_identity_kernel_method(
            self.dialect(),
            "find_canonical_api_key_record_by_hash",
        ))
    }

    async fn insert_identity_binding_record(
        &self,
        _record: &IdentityBindingRecord,
    ) -> Result<IdentityBindingRecord> {
        Err(unsupported_identity_kernel_method(
            self.dialect(),
            "insert_identity_binding_record",
        ))
    }

    async fn find_identity_binding_record(
        &self,
        _binding_type: &str,
        _issuer: Option<&str>,
        _subject: Option<&str>,
    ) -> Result<Option<IdentityBindingRecord>> {
        Err(unsupported_identity_kernel_method(
            self.dialect(),
            "find_identity_binding_record",
        ))
    }
}

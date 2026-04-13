use super::*;
use sdkwork_api_storage_core::{CommercialKernelStore, IdentityKernelStore};

pub(crate) const DEFAULT_PORTAL_JWT_SIGNING_SECRET: &str = "local-dev-portal-jwt-secret";

pub struct PortalApiState {
    pub(crate) live_store: Reloadable<Arc<dyn AdminStore>>,
    pub(crate) store: Arc<dyn AdminStore>,
    pub(crate) live_secret_manager: Reloadable<CredentialSecretManager>,
    pub(crate) secret_manager: CredentialSecretManager,
    pub(crate) live_commercial_billing: Option<Reloadable<Arc<dyn CommercialBillingAdminKernel>>>,
    pub(crate) commercial_billing: Option<Arc<dyn CommercialBillingAdminKernel>>,
    pub(crate) live_payment_store: Option<Reloadable<Arc<dyn CommercialKernelStore>>>,
    pub(crate) payment_store: Option<Arc<dyn CommercialKernelStore>>,
    pub(crate) live_identity_store: Option<Reloadable<Arc<dyn IdentityKernelStore>>>,
    pub(crate) identity_store: Option<Arc<dyn IdentityKernelStore>>,
    pub(crate) live_jwt_signing_secret: Reloadable<String>,
    pub(crate) jwt_signing_secret: String,
    pub(crate) payment_simulation_enabled: bool,
}

impl Clone for PortalApiState {
    fn clone(&self) -> Self {
        Self {
            live_store: self.live_store.clone(),
            store: self.live_store.snapshot(),
            live_secret_manager: self.live_secret_manager.clone(),
            secret_manager: self.live_secret_manager.snapshot(),
            live_commercial_billing: self.live_commercial_billing.clone(),
            commercial_billing: self
                .live_commercial_billing
                .as_ref()
                .map(Reloadable::snapshot)
                .or_else(|| self.commercial_billing.clone()),
            live_payment_store: self.live_payment_store.clone(),
            payment_store: self
                .live_payment_store
                .as_ref()
                .map(Reloadable::snapshot)
                .or_else(|| self.payment_store.clone()),
            live_identity_store: self.live_identity_store.clone(),
            identity_store: self
                .live_identity_store
                .as_ref()
                .map(Reloadable::snapshot)
                .or_else(|| self.identity_store.clone()),
            live_jwt_signing_secret: self.live_jwt_signing_secret.clone(),
            jwt_signing_secret: self.live_jwt_signing_secret.snapshot(),
            payment_simulation_enabled: self.payment_simulation_enabled,
        }
    }
}

impl PortalApiState {
    pub fn new(pool: SqlitePool) -> Self {
        Self::with_secret_manager_and_jwt_secret(
            pool,
            CredentialSecretManager::database_encrypted("local-dev-master-key"),
            DEFAULT_PORTAL_JWT_SIGNING_SECRET,
        )
    }

    pub fn with_jwt_secret(pool: SqlitePool, jwt_signing_secret: impl Into<String>) -> Self {
        Self::with_secret_manager_and_jwt_secret(
            pool,
            CredentialSecretManager::database_encrypted("local-dev-master-key"),
            jwt_signing_secret,
        )
    }

    pub fn with_secret_manager_and_jwt_secret(
        pool: SqlitePool,
        secret_manager: CredentialSecretManager,
        jwt_signing_secret: impl Into<String>,
    ) -> Self {
        let store = Arc::new(SqliteAdminStore::new(pool));
        Self::with_store_and_secret_manager_and_jwt_secret(store, secret_manager, jwt_signing_secret)
    }

    pub fn with_store_and_jwt_secret<S>(
        store: Arc<S>,
        jwt_signing_secret: impl Into<String>,
    ) -> Self
    where
        S: AdminStore
            + CommercialBillingAdminKernel
            + CommercialKernelStore
            + IdentityKernelStore
            + 'static,
    {
        Self::with_store_and_secret_manager_and_jwt_secret(
            store,
            CredentialSecretManager::database_encrypted("local-dev-master-key"),
            jwt_signing_secret,
        )
    }

    pub fn with_store_and_secret_manager_and_jwt_secret<S>(
        store: Arc<S>,
        secret_manager: CredentialSecretManager,
        jwt_signing_secret: impl Into<String>,
    ) -> Self
    where
        S: AdminStore
            + CommercialBillingAdminKernel
            + CommercialKernelStore
            + IdentityKernelStore
            + 'static,
    {
        let admin_store: Arc<dyn AdminStore> = store.clone();
        let commercial_billing: Arc<dyn CommercialBillingAdminKernel> = store.clone();
        let payment_store: Arc<dyn CommercialKernelStore> = store.clone();
        let identity_store: Arc<dyn IdentityKernelStore> = store;
        Self::with_store_secret_manager_commercial_billing_payment_identity_store_and_jwt_secret(
            admin_store,
            secret_manager,
            Some(commercial_billing),
            Some(payment_store),
            Some(identity_store),
            jwt_signing_secret,
        )
    }

    pub fn with_store_commercial_billing_and_jwt_secret(
        store: Arc<dyn AdminStore>,
        commercial_billing: Option<Arc<dyn CommercialBillingAdminKernel>>,
        jwt_signing_secret: impl Into<String>,
    ) -> Self {
        Self::with_store_secret_manager_commercial_billing_payment_identity_store_and_jwt_secret(
            store,
            CredentialSecretManager::database_encrypted("local-dev-master-key"),
            commercial_billing,
            None,
            None,
            jwt_signing_secret,
        )
    }

    pub fn with_store_commercial_billing_payment_identity_store_and_jwt_secret(
        store: Arc<dyn AdminStore>,
        commercial_billing: Option<Arc<dyn CommercialBillingAdminKernel>>,
        payment_store: Option<Arc<dyn CommercialKernelStore>>,
        identity_store: Option<Arc<dyn IdentityKernelStore>>,
        jwt_signing_secret: impl Into<String>,
    ) -> Self {
        Self::with_store_secret_manager_commercial_billing_payment_identity_store_and_jwt_secret(
            store,
            CredentialSecretManager::database_encrypted("local-dev-master-key"),
            commercial_billing,
            payment_store,
            identity_store,
            jwt_signing_secret,
        )
    }

    pub fn with_store_secret_manager_commercial_billing_and_jwt_secret(
        store: Arc<dyn AdminStore>,
        secret_manager: CredentialSecretManager,
        commercial_billing: Option<Arc<dyn CommercialBillingAdminKernel>>,
        jwt_signing_secret: impl Into<String>,
    ) -> Self {
        Self::with_store_secret_manager_commercial_billing_payment_identity_store_and_jwt_secret(
            store,
            secret_manager,
            commercial_billing,
            None,
            None,
            jwt_signing_secret,
        )
    }

    pub fn with_store_secret_manager_commercial_billing_payment_identity_store_and_jwt_secret(
        store: Arc<dyn AdminStore>,
        secret_manager: CredentialSecretManager,
        commercial_billing: Option<Arc<dyn CommercialBillingAdminKernel>>,
        payment_store: Option<Arc<dyn CommercialKernelStore>>,
        identity_store: Option<Arc<dyn IdentityKernelStore>>,
        jwt_signing_secret: impl Into<String>,
    ) -> Self {
        Self::with_live_store_secret_manager_commercial_billing_payment_identity_store_and_jwt_secret_handle(
            Reloadable::new(store),
            Reloadable::new(secret_manager),
            commercial_billing.map(Reloadable::new),
            payment_store.map(Reloadable::new),
            identity_store.map(Reloadable::new),
            Reloadable::new(jwt_signing_secret.into()),
        )
    }

    pub fn with_live_store_and_jwt_secret_handle(
        live_store: Reloadable<Arc<dyn AdminStore>>,
        live_jwt_signing_secret: Reloadable<String>,
    ) -> Self {
        Self::with_live_store_secret_manager_commercial_billing_payment_identity_store_and_jwt_secret_handle(
            live_store,
            Reloadable::new(CredentialSecretManager::database_encrypted(
                "local-dev-master-key",
            )),
            None,
            None,
            None,
            live_jwt_signing_secret,
        )
    }

    pub fn with_live_store_and_secret_manager_handle_and_jwt_secret_handle(
        live_store: Reloadable<Arc<dyn AdminStore>>,
        live_secret_manager: Reloadable<CredentialSecretManager>,
        live_jwt_signing_secret: Reloadable<String>,
    ) -> Self {
        Self::with_live_store_secret_manager_commercial_billing_payment_identity_store_and_jwt_secret_handle(
            live_store,
            live_secret_manager,
            None,
            None,
            None,
            live_jwt_signing_secret,
        )
    }

    pub fn with_live_store_commercial_billing_and_jwt_secret_handle(
        live_store: Reloadable<Arc<dyn AdminStore>>,
        live_commercial_billing: Option<Reloadable<Arc<dyn CommercialBillingAdminKernel>>>,
        live_jwt_signing_secret: Reloadable<String>,
    ) -> Self {
        Self::with_live_store_secret_manager_commercial_billing_payment_identity_store_and_jwt_secret_handle(
            live_store,
            Reloadable::new(CredentialSecretManager::database_encrypted(
                "local-dev-master-key",
            )),
            live_commercial_billing,
            None,
            None,
            live_jwt_signing_secret,
        )
    }

    pub fn with_live_store_commercial_billing_payment_identity_store_and_jwt_secret_handle(
        live_store: Reloadable<Arc<dyn AdminStore>>,
        live_commercial_billing: Option<Reloadable<Arc<dyn CommercialBillingAdminKernel>>>,
        live_payment_store: Option<Reloadable<Arc<dyn CommercialKernelStore>>>,
        live_identity_store: Option<Reloadable<Arc<dyn IdentityKernelStore>>>,
        live_jwt_signing_secret: Reloadable<String>,
    ) -> Self {
        Self::with_live_store_secret_manager_commercial_billing_payment_identity_store_and_jwt_secret_handle(
            live_store,
            Reloadable::new(CredentialSecretManager::database_encrypted(
                "local-dev-master-key",
            )),
            live_commercial_billing,
            live_payment_store,
            live_identity_store,
            live_jwt_signing_secret,
        )
    }

    pub fn with_live_store_secret_manager_commercial_billing_and_jwt_secret_handle(
        live_store: Reloadable<Arc<dyn AdminStore>>,
        live_secret_manager: Reloadable<CredentialSecretManager>,
        live_commercial_billing: Option<Reloadable<Arc<dyn CommercialBillingAdminKernel>>>,
        live_jwt_signing_secret: Reloadable<String>,
    ) -> Self {
        Self::with_live_store_secret_manager_commercial_billing_payment_identity_store_and_jwt_secret_handle(
            live_store,
            live_secret_manager,
            live_commercial_billing,
            None,
            None,
            live_jwt_signing_secret,
        )
    }

    pub fn with_live_store_secret_manager_commercial_billing_payment_identity_store_and_jwt_secret_handle(
        live_store: Reloadable<Arc<dyn AdminStore>>,
        live_secret_manager: Reloadable<CredentialSecretManager>,
        live_commercial_billing: Option<Reloadable<Arc<dyn CommercialBillingAdminKernel>>>,
        live_payment_store: Option<Reloadable<Arc<dyn CommercialKernelStore>>>,
        live_identity_store: Option<Reloadable<Arc<dyn IdentityKernelStore>>>,
        live_jwt_signing_secret: Reloadable<String>,
    ) -> Self {
        Self {
            store: live_store.snapshot(),
            live_store,
            secret_manager: live_secret_manager.snapshot(),
            live_secret_manager,
            commercial_billing: live_commercial_billing.as_ref().map(Reloadable::snapshot),
            live_commercial_billing,
            payment_store: live_payment_store.as_ref().map(Reloadable::snapshot),
            live_payment_store,
            identity_store: live_identity_store.as_ref().map(Reloadable::snapshot),
            live_identity_store,
            jwt_signing_secret: live_jwt_signing_secret.snapshot(),
            live_jwt_signing_secret,
            payment_simulation_enabled: false,
        }
    }

    pub fn with_payment_simulation_enabled(mut self, enabled: bool) -> Self {
        self.payment_simulation_enabled = enabled;
        self
    }
}

#[derive(Clone, Debug)]
pub(crate) struct AuthenticatedPortalClaims(pub(crate) PortalClaims);

impl AuthenticatedPortalClaims {
    pub(crate) fn claims(&self) -> &PortalClaims {
        &self.0
    }
}

impl FromRequestParts<PortalApiState> for AuthenticatedPortalClaims {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &PortalApiState,
    ) -> Result<Self, Self::Rejection> {
        let Some(header_value) = parts.headers.get(header::AUTHORIZATION) else {
            return Err(StatusCode::UNAUTHORIZED);
        };
        let Ok(header_value) = header_value.to_str() else {
            return Err(StatusCode::UNAUTHORIZED);
        };
        let Some(token) = header_value
            .strip_prefix("Bearer ")
            .or_else(|| header_value.strip_prefix("bearer "))
        else {
            return Err(StatusCode::UNAUTHORIZED);
        };

        verify_portal_jwt(token, &state.jwt_signing_secret)
            .map(Self)
            .map_err(|_| StatusCode::UNAUTHORIZED)
    }
}

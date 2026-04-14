use super::*;

const DEFAULT_STATELESS_TENANT_ID: &str = "sdkwork-stateless";
const DEFAULT_STATELESS_PROJECT_ID: &str = "sdkwork-stateless-default";

pub struct GatewayApiState {
    live_store: Reloadable<Arc<dyn AdminStore>>,
    live_identity_store: Option<Reloadable<Arc<dyn IdentityKernelStore>>>,
    live_secret_manager: Reloadable<CredentialSecretManager>,
    live_commercial_billing: Option<Reloadable<Arc<dyn GatewayCommercialBillingKernel>>>,
    live_payment_store: Option<Reloadable<Arc<dyn CommercialKernelStore>>>,
    pub(crate) store: Arc<dyn AdminStore>,
    pub(crate) identity_store: Option<Arc<dyn IdentityKernelStore>>,
    pub(crate) secret_manager: CredentialSecretManager,
    pub(crate) commercial_billing: Option<Arc<dyn GatewayCommercialBillingKernel>>,
}

impl Clone for GatewayApiState {
    fn clone(&self) -> Self {
        Self {
            live_store: self.live_store.clone(),
            live_identity_store: self.live_identity_store.clone(),
            live_secret_manager: self.live_secret_manager.clone(),
            live_commercial_billing: self.live_commercial_billing.clone(),
            live_payment_store: self.live_payment_store.clone(),
            store: self.live_store.snapshot(),
            identity_store: self
                .live_identity_store
                .as_ref()
                .map(Reloadable::snapshot)
                .or_else(|| self.identity_store.clone()),
            secret_manager: self.live_secret_manager.snapshot(),
            commercial_billing: self
                .live_commercial_billing
                .as_ref()
                .map(Reloadable::snapshot)
                .or_else(|| self.commercial_billing.clone()),
        }
    }
}

impl GatewayApiState {
    pub fn new(pool: SqlitePool) -> Self {
        Self::with_master_key(pool, "local-dev-master-key")
    }

    pub fn with_master_key(pool: SqlitePool, credential_master_key: impl Into<String>) -> Self {
        let store = Arc::new(SqliteAdminStore::new(pool));
        let admin_store: Arc<dyn AdminStore> = store.clone();
        let identity_store: Arc<dyn IdentityKernelStore> = store.clone();
        let commercial_billing: Arc<dyn GatewayCommercialBillingKernel> = store.clone();
        let payment_store: Arc<dyn CommercialKernelStore> = store;
        Self::with_store_secret_manager_and_kernel_handles(
            admin_store,
            Some(identity_store),
            CredentialSecretManager::database_encrypted(credential_master_key),
            Some(commercial_billing),
            Some(payment_store),
        )
    }

    pub fn with_secret_manager(pool: SqlitePool, secret_manager: CredentialSecretManager) -> Self {
        let store = Arc::new(SqliteAdminStore::new(pool));
        let admin_store: Arc<dyn AdminStore> = store.clone();
        let identity_store: Arc<dyn IdentityKernelStore> = store.clone();
        let commercial_billing: Arc<dyn GatewayCommercialBillingKernel> = store.clone();
        let payment_store: Arc<dyn CommercialKernelStore> = store;
        Self::with_store_secret_manager_and_kernel_handles(
            admin_store,
            Some(identity_store),
            secret_manager,
            Some(commercial_billing),
            Some(payment_store),
        )
    }

    pub fn with_store_and_secret_manager(
        store: Arc<dyn AdminStore>,
        secret_manager: CredentialSecretManager,
    ) -> Self {
        Self::with_live_store_secret_manager_and_kernel_handles(
            Reloadable::new(store),
            None,
            Reloadable::new(secret_manager),
            None,
            None,
        )
    }

    pub fn with_live_store_and_secret_manager(
        live_store: Reloadable<Arc<dyn AdminStore>>,
        secret_manager: CredentialSecretManager,
    ) -> Self {
        Self::with_live_store_secret_manager_and_kernel_handles(
            live_store,
            None,
            Reloadable::new(secret_manager),
            None,
            None,
        )
    }

    pub fn with_live_store_and_secret_manager_handle(
        live_store: Reloadable<Arc<dyn AdminStore>>,
        live_secret_manager: Reloadable<CredentialSecretManager>,
    ) -> Self {
        Self::with_live_store_secret_manager_and_kernel_handles(
            live_store,
            None,
            live_secret_manager,
            None,
            None,
        )
    }

    pub fn with_live_store_payment_store_and_secret_manager_handle(
        live_store: Reloadable<Arc<dyn AdminStore>>,
        live_payment_store: Reloadable<Arc<dyn CommercialKernelStore>>,
        live_secret_manager: Reloadable<CredentialSecretManager>,
    ) -> Self {
        Self::with_live_store_secret_manager_and_kernel_handles(
            live_store,
            None,
            live_secret_manager,
            None,
            Some(live_payment_store),
        )
    }

    pub fn with_live_store_commercial_billing_payment_store_and_secret_manager_handle(
        live_store: Reloadable<Arc<dyn AdminStore>>,
        live_commercial_billing: Reloadable<Arc<dyn GatewayCommercialBillingKernel>>,
        live_payment_store: Reloadable<Arc<dyn CommercialKernelStore>>,
        live_secret_manager: Reloadable<CredentialSecretManager>,
    ) -> Self {
        Self::with_live_store_secret_manager_and_kernel_handles(
            live_store,
            None,
            live_secret_manager,
            Some(live_commercial_billing),
            Some(live_payment_store),
        )
    }

    fn with_store_secret_manager_and_kernel_handles(
        store: Arc<dyn AdminStore>,
        identity_store: Option<Arc<dyn IdentityKernelStore>>,
        secret_manager: CredentialSecretManager,
        commercial_billing: Option<Arc<dyn GatewayCommercialBillingKernel>>,
        payment_store: Option<Arc<dyn CommercialKernelStore>>,
    ) -> Self {
        Self::with_live_store_secret_manager_and_kernel_handles(
            Reloadable::new(store),
            identity_store.map(Reloadable::new),
            Reloadable::new(secret_manager),
            commercial_billing.map(Reloadable::new),
            payment_store.map(Reloadable::new),
        )
    }

    fn with_live_store_secret_manager_and_kernel_handles(
        live_store: Reloadable<Arc<dyn AdminStore>>,
        live_identity_store: Option<Reloadable<Arc<dyn IdentityKernelStore>>>,
        live_secret_manager: Reloadable<CredentialSecretManager>,
        live_commercial_billing: Option<Reloadable<Arc<dyn GatewayCommercialBillingKernel>>>,
        live_payment_store: Option<Reloadable<Arc<dyn CommercialKernelStore>>>,
    ) -> Self {
        Self {
            store: live_store.snapshot(),
            identity_store: live_identity_store.as_ref().map(Reloadable::snapshot),
            secret_manager: live_secret_manager.snapshot(),
            commercial_billing: live_commercial_billing.as_ref().map(Reloadable::snapshot),
            live_store,
            live_identity_store,
            live_secret_manager,
            live_commercial_billing,
            live_payment_store,
        }
    }
}

tokio::task_local! {
    static CURRENT_GATEWAY_REQUEST_CONTEXT: IdentityGatewayRequestContext;
}

tokio::task_local! {
    static CURRENT_GATEWAY_REQUEST_STARTED_AT: Instant;
}

const DEFAULT_STATELESS_TENANT_ID: &str = "sdkwork-stateless";
const DEFAULT_STATELESS_PROJECT_ID: &str = "sdkwork-stateless-default";

pub struct GatewayApiState {
    live_store: Reloadable<Arc<dyn AdminStore>>,
    live_secret_manager: Reloadable<CredentialSecretManager>,
    live_commercial_billing: Option<Reloadable<Arc<dyn GatewayCommercialBillingKernel>>>,
    store: Arc<dyn AdminStore>,
    secret_manager: CredentialSecretManager,
    commercial_billing: Option<Arc<dyn GatewayCommercialBillingKernel>>,
}

impl Clone for GatewayApiState {
    fn clone(&self) -> Self {
        Self {
            live_store: self.live_store.clone(),
            live_secret_manager: self.live_secret_manager.clone(),
            live_commercial_billing: self.live_commercial_billing.clone(),
            store: self.live_store.snapshot(),
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
        Self::with_store_secret_manager_and_commercial_billing(
            store.clone(),
            CredentialSecretManager::database_encrypted(credential_master_key),
            Some(store),
        )
    }

    pub fn with_secret_manager(pool: SqlitePool, secret_manager: CredentialSecretManager) -> Self {
        let store = Arc::new(SqliteAdminStore::new(pool));
        Self::with_store_secret_manager_and_commercial_billing(
            store.clone(),
            secret_manager,
            Some(store),
        )
    }

    pub fn with_store_and_secret_manager(
        store: Arc<dyn AdminStore>,
        secret_manager: CredentialSecretManager,
    ) -> Self {
        Self::with_live_store_secret_manager_and_commercial_billing_handle(
            Reloadable::new(store),
            Reloadable::new(secret_manager),
            None,
        )
    }

    pub fn with_live_store_and_secret_manager(
        live_store: Reloadable<Arc<dyn AdminStore>>,
        secret_manager: CredentialSecretManager,
    ) -> Self {
        Self::with_live_store_secret_manager_and_commercial_billing_handle(
            live_store,
            Reloadable::new(secret_manager),
            None,
        )
    }

    pub fn with_live_store_and_secret_manager_handle(
        live_store: Reloadable<Arc<dyn AdminStore>>,
        live_secret_manager: Reloadable<CredentialSecretManager>,
    ) -> Self {
        Self::with_live_store_secret_manager_and_commercial_billing_handle(
            live_store,
            live_secret_manager,
            None,
        )
    }

    fn with_store_secret_manager_and_commercial_billing(
        store: Arc<dyn AdminStore>,
        secret_manager: CredentialSecretManager,
        commercial_billing: Option<Arc<dyn GatewayCommercialBillingKernel>>,
    ) -> Self {
        Self::with_live_store_secret_manager_and_commercial_billing_handle(
            Reloadable::new(store),
            Reloadable::new(secret_manager),
            commercial_billing.map(Reloadable::new),
        )
    }

    fn with_live_store_secret_manager_and_commercial_billing_handle(
        live_store: Reloadable<Arc<dyn AdminStore>>,
        live_secret_manager: Reloadable<CredentialSecretManager>,
        live_commercial_billing: Option<Reloadable<Arc<dyn GatewayCommercialBillingKernel>>>,
    ) -> Self {
        Self {
            store: live_store.snapshot(),
            secret_manager: live_secret_manager.snapshot(),
            commercial_billing: live_commercial_billing.as_ref().map(Reloadable::snapshot),
            live_store,
            live_secret_manager,
            live_commercial_billing,
        }
    }
}

tokio::task_local! {
    static CURRENT_GATEWAY_REQUEST_CONTEXT: IdentityGatewayRequestContext;
}

tokio::task_local! {
    static CURRENT_GATEWAY_REQUEST_STARTED_AT: Instant;
}


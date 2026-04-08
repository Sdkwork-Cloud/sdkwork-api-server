use super::*;

pub struct StandaloneRuntimeSupervision {
    pub(crate) join_handle: JoinHandle<()>,
}

impl Drop for StandaloneRuntimeSupervision {
    fn drop(&mut self) {
        self.join_handle.abort();
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StandaloneServiceKind {
    Gateway,
    Admin,
    Portal,
}

impl StandaloneServiceKind {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Gateway => "gateway",
            Self::Admin => "admin",
            Self::Portal => "portal",
        }
    }

    pub(crate) fn supports_runtime_dynamic(self) -> bool {
        matches!(self, Self::Gateway | Self::Admin)
    }

    pub(crate) fn supports_pricing_lifecycle_supervision(self) -> bool {
        matches!(self, Self::Admin)
    }
}

pub struct StandaloneServiceReloadHandles {
    pub(crate) store: Reloadable<Arc<dyn AdminStore>>,
    pub(crate) commercial_billing: Option<Reloadable<Arc<dyn CommercialBillingAdminKernel>>>,
    pub(crate) coordination_store: Option<Reloadable<Arc<dyn AdminStore>>>,
    pub(crate) secret_manager: Option<Reloadable<CredentialSecretManager>>,
    pub(crate) admin_jwt_signing_secret: Option<Reloadable<String>>,
    pub(crate) portal_jwt_signing_secret: Option<Reloadable<String>>,
    pub(crate) listener: Option<StandaloneListenerHandle>,
    pub(crate) node_id: Option<String>,
}

impl StandaloneServiceReloadHandles {
    pub fn gateway(store: Reloadable<Arc<dyn AdminStore>>) -> Self {
        Self {
            store,
            commercial_billing: None,
            coordination_store: None,
            secret_manager: None,
            admin_jwt_signing_secret: None,
            portal_jwt_signing_secret: None,
            listener: None,
            node_id: None,
        }
    }

    pub fn admin(
        store: Reloadable<Arc<dyn AdminStore>>,
        admin_jwt_signing_secret: Reloadable<String>,
    ) -> Self {
        Self {
            store,
            commercial_billing: None,
            coordination_store: None,
            secret_manager: None,
            admin_jwt_signing_secret: Some(admin_jwt_signing_secret),
            portal_jwt_signing_secret: None,
            listener: None,
            node_id: None,
        }
    }

    pub fn portal(
        store: Reloadable<Arc<dyn AdminStore>>,
        portal_jwt_signing_secret: Reloadable<String>,
    ) -> Self {
        Self {
            store,
            commercial_billing: None,
            coordination_store: None,
            secret_manager: None,
            admin_jwt_signing_secret: None,
            portal_jwt_signing_secret: Some(portal_jwt_signing_secret),
            listener: None,
            node_id: None,
        }
    }

    pub fn with_live_commercial_billing(
        mut self,
        commercial_billing: Reloadable<Arc<dyn CommercialBillingAdminKernel>>,
    ) -> Self {
        self.commercial_billing = Some(commercial_billing);
        self
    }

    pub fn with_coordination_store(mut self, coordination_store: Arc<dyn AdminStore>) -> Self {
        self.coordination_store = Some(Reloadable::new(coordination_store));
        self
    }

    pub fn with_secret_manager(
        mut self,
        secret_manager: Reloadable<CredentialSecretManager>,
    ) -> Self {
        self.secret_manager = Some(secret_manager);
        self
    }

    pub fn with_listener(mut self, listener: StandaloneListenerHandle) -> Self {
        self.listener = Some(listener);
        self
    }

    pub fn with_node_id(mut self, node_id: impl Into<String>) -> Self {
        self.node_id = Some(node_id.into());
        self
    }
}

pub(crate) struct StandaloneRuntimeState {
    pub(crate) current_config: StandaloneConfig,
    pub(crate) current_dynamic: StandaloneRuntimeDynamicConfig,
    pub(crate) current_store: Arc<dyn AdminStore>,
    pub(crate) snapshot_supervision: AbortOnDropHandle,
    pub(crate) extension_hot_reload_supervision: AbortOnDropHandle,
    pub(crate) pricing_lifecycle_supervision: AbortOnDropHandle,
    pub(crate) previous_watch_state: Option<StandaloneConfigWatchState>,
    pub(crate) pending_restart_required: Option<PendingStandaloneRuntimeRestartRequired>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PendingStandaloneRuntimeRestartRequired {
    pub(crate) watch_state: StandaloneConfigWatchState,
    pub(crate) message: String,
}

pub(crate) struct MemoryCacheStoreFactory;

#[async_trait]
impl CacheDriverFactory for MemoryCacheStoreFactory {
    fn backend_kind(&self) -> CacheBackendKind {
        CacheBackendKind::Memory
    }

    fn driver_name(&self) -> &'static str {
        "memory-cache-store"
    }

    async fn build(&self, _cache_url: Option<&str>) -> Result<CacheRuntimeStores> {
        let store = Arc::new(MemoryCacheStore::default());
        Ok(CacheRuntimeStores::new(store.clone(), store))
    }
}

pub(crate) struct RedisCacheStoreFactory;

#[async_trait]
impl CacheDriverFactory for RedisCacheStoreFactory {
    fn backend_kind(&self) -> CacheBackendKind {
        CacheBackendKind::Redis
    }

    fn driver_name(&self) -> &'static str {
        "redis-cache-store"
    }

    async fn build(&self, cache_url: Option<&str>) -> Result<CacheRuntimeStores> {
        let cache_url =
            cache_url.ok_or_else(|| anyhow::anyhow!("redis cache backend requires cache_url"))?;
        let store = Arc::new(RedisCacheStore::connect(cache_url).await?);
        Ok(CacheRuntimeStores::new(store.clone(), store))
    }
}

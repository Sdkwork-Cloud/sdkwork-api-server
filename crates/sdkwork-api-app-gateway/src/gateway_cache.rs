use super::*;

pub const ROUTING_DECISION_CACHE_NAMESPACE: &str = "gateway_route_decisions";
const ROUTING_DECISION_CACHE_TTL_MS: u64 = 30_000;
static ROUTING_DECISION_CACHE_STORE: OnceLock<Reloadable<Arc<dyn CacheStore>>> = OnceLock::new();
static ROUTING_RECOVERY_PROBE_LOCK_STORE: OnceLock<Reloadable<Arc<dyn DistributedLockStore>>> =
    OnceLock::new();
static GATEWAY_PROVIDER_MAX_IN_FLIGHT_LIMIT: OnceLock<Reloadable<Option<usize>>> = OnceLock::new();
static GATEWAY_PROVIDER_IN_FLIGHT_COUNTERS: OnceLock<Mutex<HashMap<String, Arc<AtomicUsize>>>> =
    OnceLock::new();
pub const CAPABILITY_CATALOG_CACHE_NAMESPACE: &str = "gateway_capability_catalog";
pub(crate) const CAPABILITY_CATALOG_CACHE_TTL_MS: u64 = 30_000;
pub(crate) const CAPABILITY_CATALOG_CACHE_TAG_ALL_MODELS: &str = "models:all";
static CAPABILITY_CATALOG_CACHE_STORE: OnceLock<Reloadable<Option<Arc<dyn CacheStore>>>> =
    OnceLock::new();

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CachedCapabilityCatalogModel {
    pub(crate) id: String,
    pub(crate) owned_by: String,
}

impl CachedCapabilityCatalogModel {
    pub(crate) fn into_model_object(self) -> ModelObject {
        ModelObject::new(self.id, self.owned_by)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CachedCapabilityCatalogList {
    pub(crate) models: Vec<CachedCapabilityCatalogModel>,
}

impl CachedCapabilityCatalogList {
    pub(crate) fn into_response(self) -> ListModelsResponse {
        ListModelsResponse::new(
            self.models
                .into_iter()
                .map(CachedCapabilityCatalogModel::into_model_object)
                .collect(),
        )
    }
}

fn routing_decision_cache_store_handle() -> &'static Reloadable<Arc<dyn CacheStore>> {
    ROUTING_DECISION_CACHE_STORE.get_or_init(|| {
        Reloadable::new(Arc::new(MemoryCacheStore::default()) as Arc<dyn CacheStore>)
    })
}

fn routing_decision_cache_store() -> Arc<dyn CacheStore> {
    routing_decision_cache_store_handle().snapshot()
}

pub fn configure_route_decision_cache_store(cache_store: Arc<dyn CacheStore>) {
    routing_decision_cache_store_handle().replace(cache_store);
}

fn routing_recovery_probe_lock_store_handle() -> &'static Reloadable<Arc<dyn DistributedLockStore>>
{
    ROUTING_RECOVERY_PROBE_LOCK_STORE.get_or_init(|| {
        Reloadable::new(Arc::new(MemoryCacheStore::default()) as Arc<dyn DistributedLockStore>)
    })
}

pub(crate) fn routing_recovery_probe_lock_store() -> Arc<dyn DistributedLockStore> {
    routing_recovery_probe_lock_store_handle().snapshot()
}

pub fn configure_route_recovery_probe_lock_store(lock_store: Arc<dyn DistributedLockStore>) {
    routing_recovery_probe_lock_store_handle().replace(lock_store);
}

const GATEWAY_PROVIDER_MAX_IN_FLIGHT_ENV: &str = "SDKWORK_GATEWAY_PROVIDER_MAX_IN_FLIGHT";

fn gateway_provider_max_in_flight_limit_from_env(configured: Option<&str>) -> Option<usize> {
    configured
        .and_then(|value| value.trim().parse::<usize>().ok())
        .filter(|limit| *limit > 0)
}

fn gateway_provider_max_in_flight_limit_handle() -> &'static Reloadable<Option<usize>> {
    GATEWAY_PROVIDER_MAX_IN_FLIGHT_LIMIT.get_or_init(|| {
        Reloadable::new(gateway_provider_max_in_flight_limit_from_env(
            std::env::var(GATEWAY_PROVIDER_MAX_IN_FLIGHT_ENV)
                .ok()
                .as_deref(),
        ))
    })
}

pub(crate) fn gateway_provider_max_in_flight_limit() -> Option<usize> {
    gateway_provider_max_in_flight_limit_handle().snapshot()
}

pub fn configure_gateway_provider_max_in_flight_limit(limit: Option<usize>) {
    gateway_provider_max_in_flight_limit_handle().replace(limit);
}

fn gateway_provider_in_flight_counters_handle() -> &'static Mutex<HashMap<String, Arc<AtomicUsize>>>
{
    GATEWAY_PROVIDER_IN_FLIGHT_COUNTERS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub(crate) fn gateway_provider_in_flight_counter(provider_id: &str) -> Arc<AtomicUsize> {
    gateway_provider_in_flight_counters_handle()
        .lock()
        .expect("gateway provider in-flight counter lock")
        .entry(provider_id.to_owned())
        .or_insert_with(|| Arc::new(AtomicUsize::new(0)))
        .clone()
}

fn capability_catalog_cache_store_handle() -> &'static Reloadable<Option<Arc<dyn CacheStore>>> {
    CAPABILITY_CATALOG_CACHE_STORE.get_or_init(|| Reloadable::new(None))
}

pub(crate) fn capability_catalog_cache_store() -> Option<Arc<dyn CacheStore>> {
    capability_catalog_cache_store_handle().snapshot()
}

pub fn configure_capability_catalog_cache_store(cache_store: Arc<dyn CacheStore>) {
    capability_catalog_cache_store_handle().replace(Some(cache_store));
}

pub fn clear_capability_catalog_cache_store() {
    capability_catalog_cache_store_handle().replace(None);
}

pub async fn invalidate_capability_catalog_cache() {
    if let Some(cache_store) = capability_catalog_cache_store() {
        if let Err(error) = cache_store
            .invalidate_tag(
                CAPABILITY_CATALOG_CACHE_NAMESPACE,
                CAPABILITY_CATALOG_CACHE_TAG_ALL_MODELS,
            )
            .await
        {
            eprintln!("capability catalog cache invalidation failed: {error}");
        }
    }
}

pub(crate) fn capability_catalog_list_cache_key(tenant_id: &str, project_id: &str) -> String {
    format!("{tenant_id}|{project_id}|list")
}

pub(crate) fn capability_catalog_model_cache_key(
    tenant_id: &str,
    project_id: &str,
    model_id: &str,
) -> String {
    format!("{tenant_id}|{project_id}|model|{model_id}")
}

pub(crate) fn routing_decision_cache_key(
    tenant_id: &str,
    project_id: Option<&str>,
    api_key_group_id: Option<&str>,
    capability: &str,
    route_key: &str,
    requested_region: Option<&str>,
) -> String {
    format!(
        "{tenant_id}|{}|{}|{capability}|{route_key}|{}",
        project_id.unwrap_or_default(),
        api_key_group_id.unwrap_or_default(),
        requested_region.unwrap_or_default()
    )
}

pub(crate) async fn cache_routing_decision(
    tenant_id: &str,
    project_id: Option<&str>,
    api_key_group_id: Option<&str>,
    capability: &str,
    route_key: &str,
    requested_region: Option<&str>,
    decision: &RoutingDecision,
) {
    let key = routing_decision_cache_key(
        tenant_id,
        project_id,
        api_key_group_id,
        capability,
        route_key,
        requested_region,
    );
    let payload = match serde_json::to_vec(decision) {
        Ok(payload) => payload,
        Err(error) => {
            eprintln!("routing decision cache serialization failed: {error}");
            return;
        }
    };
    if let Err(error) = routing_decision_cache_store()
        .put(
            ROUTING_DECISION_CACHE_NAMESPACE,
            &key,
            payload,
            Some(ROUTING_DECISION_CACHE_TTL_MS),
            &[],
        )
        .await
    {
        eprintln!("routing decision cache write failed: {error}");
    }
}

pub(crate) async fn take_cached_routing_decision(
    tenant_id: &str,
    project_id: Option<&str>,
    api_key_group_id: Option<&str>,
    capability: &str,
    route_key: &str,
    requested_region: Option<&str>,
) -> Option<RoutingDecision> {
    let key = routing_decision_cache_key(
        tenant_id,
        project_id,
        api_key_group_id,
        capability,
        route_key,
        requested_region,
    );
    let cache_store = routing_decision_cache_store();
    let entry = match cache_store
        .get(ROUTING_DECISION_CACHE_NAMESPACE, &key)
        .await
    {
        Ok(entry) => entry,
        Err(error) => {
            eprintln!("routing decision cache read failed: {error}");
            return None;
        }
    }?;
    if let Err(error) = cache_store
        .delete(ROUTING_DECISION_CACHE_NAMESPACE, &key)
        .await
    {
        eprintln!("routing decision cache delete failed: {error}");
    }
    match serde_json::from_slice(entry.value()) {
        Ok(decision) => Some(decision),
        Err(error) => {
            eprintln!("routing decision cache decode failed: {error}");
            None
        }
    }
}

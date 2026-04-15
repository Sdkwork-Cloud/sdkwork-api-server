use anyhow::Result;
use async_trait::async_trait;
use std::future::Future;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CacheBackendKind {
    Memory,
    Redis,
}

impl CacheBackendKind {
    pub fn parse(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "memory" => Ok(Self::Memory),
            "redis" => Ok(Self::Redis),
            other => anyhow::bail!("unsupported cache backend: {other}"),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Memory => "memory",
            Self::Redis => "redis",
        }
    }

    pub fn supports_shared_cache_coherence(self) -> bool {
        match self {
            Self::Memory => false,
            Self::Redis => true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheTag(String);

impl CacheTag {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheEntry {
    value: Vec<u8>,
    created_at_ms: u64,
    expires_at_ms: Option<u64>,
}

impl CacheEntry {
    pub fn new(value: Vec<u8>) -> Self {
        Self {
            value,
            created_at_ms: current_timestamp_ms(),
            expires_at_ms: None,
        }
    }

    pub fn with_ttl_ms(mut self, ttl_ms: u64) -> Self {
        self.expires_at_ms = Some(self.created_at_ms.saturating_add(ttl_ms));
        self
    }

    pub fn with_ttl_ms_option(self, ttl_ms: Option<u64>) -> Self {
        match ttl_ms {
            Some(ttl_ms) => self.with_ttl_ms(ttl_ms),
            None => self,
        }
    }

    pub fn value(&self) -> &[u8] {
        &self.value
    }

    pub fn into_value(self) -> Vec<u8> {
        self.value
    }

    pub fn created_at_ms(&self) -> u64 {
        self.created_at_ms
    }

    pub fn expires_at_ms(&self) -> Option<u64> {
        self.expires_at_ms
    }

    pub fn is_expired(&self) -> bool {
        self.is_expired_at(current_timestamp_ms())
    }

    pub fn is_expired_at(&self, now_ms: u64) -> bool {
        self.expires_at_ms
            .map(|expires_at_ms| expires_at_ms <= now_ms)
            .unwrap_or(false)
    }
}

#[async_trait]
pub trait CacheStore: Send + Sync {
    async fn get(&self, namespace: &str, key: &str) -> Result<Option<CacheEntry>>;

    async fn put(
        &self,
        namespace: &str,
        key: &str,
        value: Vec<u8>,
        ttl_ms: Option<u64>,
        tags: &[CacheTag],
    ) -> Result<()>;

    async fn delete(&self, namespace: &str, key: &str) -> Result<bool>;

    async fn invalidate_tag(&self, namespace: &str, tag: &str) -> Result<u64>;
}

#[async_trait]
pub trait DistributedLockStore: Send + Sync {
    async fn try_acquire_lock(&self, scope: &str, owner: &str, ttl_ms: u64) -> Result<bool>;

    async fn release_lock(&self, scope: &str, owner: &str) -> Result<bool>;
}

#[derive(Clone)]
pub struct CacheRuntimeStores {
    cache_store: Arc<dyn CacheStore>,
    distributed_lock_store: Arc<dyn DistributedLockStore>,
}

impl CacheRuntimeStores {
    pub fn new<C, L>(cache_store: Arc<C>, distributed_lock_store: Arc<L>) -> Self
    where
        C: CacheStore + 'static,
        L: DistributedLockStore + 'static,
    {
        Self {
            cache_store,
            distributed_lock_store,
        }
    }

    pub fn cache_store(&self) -> Arc<dyn CacheStore> {
        self.cache_store.clone()
    }

    pub fn distributed_lock_store(&self) -> Arc<dyn DistributedLockStore> {
        self.distributed_lock_store.clone()
    }
}

#[async_trait]
pub trait CacheDriverFactory: Send + Sync {
    fn backend_kind(&self) -> CacheBackendKind;

    fn driver_name(&self) -> &'static str;

    async fn build(&self, cache_url: Option<&str>) -> Result<CacheRuntimeStores>;
}

#[derive(Default)]
pub struct CacheDriverRegistry {
    factories: HashMap<CacheBackendKind, Arc<dyn CacheDriverFactory>>,
}

impl CacheDriverRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_factory<F>(mut self, factory: F) -> Self
    where
        F: CacheDriverFactory + 'static,
    {
        self.register(factory);
        self
    }

    pub fn register<F>(&mut self, factory: F) -> Option<Arc<dyn CacheDriverFactory>>
    where
        F: CacheDriverFactory + 'static,
    {
        self.register_arc(Arc::new(factory))
    }

    pub fn register_arc(
        &mut self,
        factory: Arc<dyn CacheDriverFactory>,
    ) -> Option<Arc<dyn CacheDriverFactory>> {
        self.factories.insert(factory.backend_kind(), factory)
    }

    pub fn resolve(&self, backend_kind: CacheBackendKind) -> Option<Arc<dyn CacheDriverFactory>> {
        self.factories.get(&backend_kind).cloned()
    }

    pub fn supports(&self, backend_kind: CacheBackendKind) -> bool {
        self.factories.contains_key(&backend_kind)
    }
}

pub async fn cache_get_or_insert_with<C, F, Fut>(
    cache: &C,
    namespace: &str,
    key: &str,
    ttl_ms: Option<u64>,
    tags: &[CacheTag],
    loader: F,
) -> Result<Vec<u8>>
where
    C: CacheStore + ?Sized,
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<Vec<u8>>>,
{
    if let Some(entry) = cache.get(namespace, key).await? {
        if !entry.is_expired() {
            return Ok(entry.into_value());
        }

        let _ = cache.delete(namespace, key).await?;
    }

    let value = loader().await?;
    cache
        .put(namespace, key, value.clone(), ttl_ms, tags)
        .await?;
    Ok(value)
}

fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("unix time")
        .as_millis() as u64
}

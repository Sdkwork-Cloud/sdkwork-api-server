use anyhow::Result;
use async_trait::async_trait;
use sdkwork_api_cache_core::{CacheEntry, CacheStore, CacheTag, DistributedLockStore};
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

pub struct MemoryCacheStore {
    state: RwLock<MemoryCacheState>,
}

#[derive(Default)]
struct MemoryCacheState {
    entries: HashMap<String, MemoryCacheRecord>,
    tag_index: HashMap<String, HashSet<String>>,
    locks: HashMap<String, MemoryLockRecord>,
}

struct MemoryCacheRecord {
    entry: CacheEntry,
    tag_keys: HashSet<String>,
}

struct MemoryLockRecord {
    owner: String,
    expires_at_ms: u64,
}

impl Default for MemoryCacheStore {
    fn default() -> Self {
        Self {
            state: RwLock::new(MemoryCacheState::default()),
        }
    }
}

#[async_trait]
impl CacheStore for MemoryCacheStore {
    async fn get(&self, namespace: &str, key: &str) -> Result<Option<CacheEntry>> {
        let composite = entry_key(namespace, key);
        let mut state = self.state.write().await;
        purge_expired_entries(&mut state);
        Ok(state
            .entries
            .get(&composite)
            .map(|record| record.entry.clone()))
    }

    async fn put(
        &self,
        namespace: &str,
        key: &str,
        value: Vec<u8>,
        ttl_ms: Option<u64>,
        tags: &[CacheTag],
    ) -> Result<()> {
        let composite = entry_key(namespace, key);
        let mut state = self.state.write().await;
        purge_expired_entries(&mut state);
        remove_entry(&mut state, &composite);

        let tag_keys = tags
            .iter()
            .map(|tag| tag_scope_key(namespace, tag.value()))
            .collect::<HashSet<_>>();
        let record = MemoryCacheRecord {
            entry: CacheEntry::new(value).with_ttl_ms_option(ttl_ms),
            tag_keys: tag_keys.clone(),
        };
        for tag_key in tag_keys {
            state
                .tag_index
                .entry(tag_key)
                .or_default()
                .insert(composite.clone());
        }
        state.entries.insert(composite, record);
        Ok(())
    }

    async fn delete(&self, namespace: &str, key: &str) -> Result<bool> {
        let composite = entry_key(namespace, key);
        let mut state = self.state.write().await;
        purge_expired_entries(&mut state);
        Ok(remove_entry(&mut state, &composite))
    }

    async fn invalidate_tag(&self, namespace: &str, tag: &str) -> Result<u64> {
        let tag_key = tag_scope_key(namespace, tag);
        let mut state = self.state.write().await;
        purge_expired_entries(&mut state);
        let composites = state.tag_index.remove(&tag_key).unwrap_or_default();
        let mut removed = 0_u64;
        for composite in composites {
            if remove_entry(&mut state, &composite) {
                removed += 1;
            }
        }
        Ok(removed)
    }
}

#[async_trait]
impl DistributedLockStore for MemoryCacheStore {
    async fn try_acquire_lock(&self, scope: &str, owner: &str, ttl_ms: u64) -> Result<bool> {
        let mut state = self.state.write().await;
        purge_expired_locks(&mut state);
        let expires_at_ms = current_timestamp_ms().saturating_add(ttl_ms);
        match state.locks.get(scope) {
            Some(record) if record.owner != owner => Ok(false),
            _ => {
                state.locks.insert(
                    scope.to_owned(),
                    MemoryLockRecord {
                        owner: owner.to_owned(),
                        expires_at_ms,
                    },
                );
                Ok(true)
            }
        }
    }

    async fn release_lock(&self, scope: &str, owner: &str) -> Result<bool> {
        let mut state = self.state.write().await;
        purge_expired_locks(&mut state);
        match state.locks.get(scope) {
            Some(record) if record.owner == owner => Ok(state.locks.remove(scope).is_some()),
            _ => Ok(false),
        }
    }
}

fn remove_entry(state: &mut MemoryCacheState, composite: &str) -> bool {
    let Some(record) = state.entries.remove(composite) else {
        return false;
    };

    for tag_key in record.tag_keys {
        if let Some(tagged) = state.tag_index.get_mut(&tag_key) {
            tagged.remove(composite);
            if tagged.is_empty() {
                state.tag_index.remove(&tag_key);
            }
        }
    }

    true
}

fn purge_expired_entries(state: &mut MemoryCacheState) {
    let expired = state
        .entries
        .iter()
        .filter(|(_, record)| record.entry.is_expired())
        .map(|(composite, _)| composite.clone())
        .collect::<Vec<_>>();
    for composite in expired {
        remove_entry(state, &composite);
    }
}

fn purge_expired_locks(state: &mut MemoryCacheState) {
    let now_ms = current_timestamp_ms();
    state
        .locks
        .retain(|_, record| record.expires_at_ms > now_ms);
}

fn entry_key(namespace: &str, key: &str) -> String {
    format!("{namespace}:{key}")
}

fn tag_scope_key(namespace: &str, tag: &str) -> String {
    format!("{namespace}:{tag}")
}

fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("unix time")
        .as_millis() as u64
}

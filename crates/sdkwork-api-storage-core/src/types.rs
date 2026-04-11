use super::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StorageDialect {
    Sqlite,
    Postgres,
    Mysql,
    Libsql,
}

impl StorageDialect {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sqlite => "sqlite",
            Self::Postgres => "postgres",
            Self::Mysql => "mysql",
            Self::Libsql => "libsql",
        }
    }
}

#[async_trait]
pub trait StorageDriverFactory<T>: Send + Sync {
    fn dialect(&self) -> StorageDialect;

    fn driver_name(&self) -> &'static str;

    async fn build(&self, database_url: &str) -> Result<T>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtomicCouponReservationCommand {
    pub template_to_persist: Option<CouponTemplateRecord>,
    pub campaign_to_persist: Option<MarketingCampaignRecord>,
    pub expected_budget: CampaignBudgetRecord,
    pub next_budget: CampaignBudgetRecord,
    pub expected_code: CouponCodeRecord,
    pub next_code: CouponCodeRecord,
    pub reservation: CouponReservationRecord,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtomicCouponReservationResult {
    pub budget: CampaignBudgetRecord,
    pub code: CouponCodeRecord,
    pub reservation: CouponReservationRecord,
    pub created: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtomicCouponConfirmationCommand {
    pub expected_budget: CampaignBudgetRecord,
    pub next_budget: CampaignBudgetRecord,
    pub expected_code: CouponCodeRecord,
    pub next_code: CouponCodeRecord,
    pub expected_reservation: CouponReservationRecord,
    pub next_reservation: CouponReservationRecord,
    pub redemption: CouponRedemptionRecord,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtomicCouponConfirmationResult {
    pub budget: CampaignBudgetRecord,
    pub code: CouponCodeRecord,
    pub reservation: CouponReservationRecord,
    pub redemption: CouponRedemptionRecord,
    pub created: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtomicCouponReleaseCommand {
    pub expected_budget: CampaignBudgetRecord,
    pub next_budget: CampaignBudgetRecord,
    pub expected_code: CouponCodeRecord,
    pub next_code: CouponCodeRecord,
    pub expected_reservation: CouponReservationRecord,
    pub next_reservation: CouponReservationRecord,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtomicCouponReleaseResult {
    pub budget: CampaignBudgetRecord,
    pub code: CouponCodeRecord,
    pub reservation: CouponReservationRecord,
    pub created: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtomicCouponRollbackCommand {
    pub expected_budget: CampaignBudgetRecord,
    pub next_budget: CampaignBudgetRecord,
    pub expected_code: CouponCodeRecord,
    pub next_code: CouponCodeRecord,
    pub expected_redemption: CouponRedemptionRecord,
    pub next_redemption: CouponRedemptionRecord,
    pub rollback: CouponRollbackRecord,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtomicCouponRollbackResult {
    pub budget: CampaignBudgetRecord,
    pub code: CouponCodeRecord,
    pub redemption: CouponRedemptionRecord,
    pub rollback: CouponRollbackRecord,
    pub created: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtomicCouponRollbackCompensationCommand {
    pub expected_budget: CampaignBudgetRecord,
    pub next_budget: CampaignBudgetRecord,
    pub expected_code: CouponCodeRecord,
    pub next_code: CouponCodeRecord,
    pub expected_redemption: CouponRedemptionRecord,
    pub next_redemption: CouponRedemptionRecord,
    pub expected_rollback: CouponRollbackRecord,
    pub next_rollback: CouponRollbackRecord,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtomicCouponRollbackCompensationResult {
    pub budget: CampaignBudgetRecord,
    pub code: CouponCodeRecord,
    pub redemption: CouponRedemptionRecord,
    pub rollback: CouponRollbackRecord,
    pub created: bool,
}

pub struct StorageDriverRegistry<T> {
    factories: HashMap<StorageDialect, Arc<dyn StorageDriverFactory<T>>>,
}

impl<T> Default for StorageDriverRegistry<T> {
    fn default() -> Self {
        Self {
            factories: HashMap::new(),
        }
    }
}

impl<T> StorageDriverRegistry<T>
where
    T: Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_factory<F>(mut self, factory: F) -> Self
    where
        F: StorageDriverFactory<T> + 'static,
    {
        self.register(factory);
        self
    }

    pub fn register<F>(&mut self, factory: F) -> Option<Arc<dyn StorageDriverFactory<T>>>
    where
        F: StorageDriverFactory<T> + 'static,
    {
        self.register_arc(Arc::new(factory))
    }

    pub fn register_arc(
        &mut self,
        factory: Arc<dyn StorageDriverFactory<T>>,
    ) -> Option<Arc<dyn StorageDriverFactory<T>>> {
        self.factories.insert(factory.dialect(), factory)
    }

    pub fn resolve(&self, dialect: StorageDialect) -> Option<Arc<dyn StorageDriverFactory<T>>> {
        self.factories.get(&dialect).cloned()
    }

    pub fn supports(&self, dialect: StorageDialect) -> bool {
        self.factories.contains_key(&dialect)
    }
}

pub struct Reloadable<T: Clone> {
    current: Arc<RwLock<T>>,
}

impl<T: Clone> Reloadable<T> {
    pub fn new(initial: T) -> Self {
        Self {
            current: Arc::new(RwLock::new(initial)),
        }
    }

    pub fn snapshot(&self) -> T {
        self.current
            .read()
            .expect("reloadable value lock poisoned")
            .clone()
    }

    pub fn replace(&self, next: T) {
        *self
            .current
            .write()
            .expect("reloadable value lock poisoned") = next;
    }
}

impl<T: Clone> Clone for Reloadable<T> {
    fn clone(&self) -> Self {
        Self {
            current: Arc::clone(&self.current),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceRuntimeNodeRecord {
    pub node_id: String,
    pub service_kind: String,
    pub started_at_ms: u64,
    pub last_seen_at_ms: u64,
}

impl ServiceRuntimeNodeRecord {
    pub fn new(
        node_id: impl Into<String>,
        service_kind: impl Into<String>,
        started_at_ms: u64,
    ) -> Self {
        Self {
            node_id: node_id.into(),
            service_kind: service_kind.into(),
            started_at_ms,
            last_seen_at_ms: started_at_ms,
        }
    }

    pub fn with_last_seen_at_ms(mut self, last_seen_at_ms: u64) -> Self {
        self.last_seen_at_ms = last_seen_at_ms;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtensionRuntimeRolloutRecord {
    pub rollout_id: String,
    pub scope: String,
    pub requested_extension_id: Option<String>,
    pub requested_instance_id: Option<String>,
    pub resolved_extension_id: Option<String>,
    pub created_by: String,
    pub created_at_ms: u64,
    pub deadline_at_ms: u64,
}

impl ExtensionRuntimeRolloutRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        rollout_id: impl Into<String>,
        scope: impl Into<String>,
        requested_extension_id: Option<String>,
        requested_instance_id: Option<String>,
        resolved_extension_id: Option<String>,
        created_by: impl Into<String>,
        created_at_ms: u64,
        deadline_at_ms: u64,
    ) -> Self {
        Self {
            rollout_id: rollout_id.into(),
            scope: scope.into(),
            requested_extension_id,
            requested_instance_id,
            resolved_extension_id,
            created_by: created_by.into(),
            created_at_ms,
            deadline_at_ms,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtensionRuntimeRolloutParticipantRecord {
    pub rollout_id: String,
    pub node_id: String,
    pub service_kind: String,
    pub status: String,
    pub message: Option<String>,
    pub updated_at_ms: u64,
}

impl ExtensionRuntimeRolloutParticipantRecord {
    pub fn new(
        rollout_id: impl Into<String>,
        node_id: impl Into<String>,
        service_kind: impl Into<String>,
        status: impl Into<String>,
        updated_at_ms: u64,
    ) -> Self {
        Self {
            rollout_id: rollout_id.into(),
            node_id: node_id.into(),
            service_kind: service_kind.into(),
            status: status.into(),
            message: None,
            updated_at_ms,
        }
    }

    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StandaloneConfigRolloutRecord {
    pub rollout_id: String,
    pub requested_service_kind: Option<String>,
    pub created_by: String,
    pub created_at_ms: u64,
    pub deadline_at_ms: u64,
}

impl StandaloneConfigRolloutRecord {
    pub fn new(
        rollout_id: impl Into<String>,
        requested_service_kind: Option<String>,
        created_by: impl Into<String>,
        created_at_ms: u64,
        deadline_at_ms: u64,
    ) -> Self {
        Self {
            rollout_id: rollout_id.into(),
            requested_service_kind,
            created_by: created_by.into(),
            created_at_ms,
            deadline_at_ms,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StandaloneConfigRolloutParticipantRecord {
    pub rollout_id: String,
    pub node_id: String,
    pub service_kind: String,
    pub status: String,
    pub message: Option<String>,
    pub updated_at_ms: u64,
}

impl StandaloneConfigRolloutParticipantRecord {
    pub fn new(
        rollout_id: impl Into<String>,
        node_id: impl Into<String>,
        service_kind: impl Into<String>,
        status: impl Into<String>,
        updated_at_ms: u64,
    ) -> Self {
        Self {
            rollout_id: rollout_id.into(),
            node_id: node_id.into(),
            service_kind: service_kind.into(),
            status: status.into(),
            message: None,
            updated_at_ms,
        }
    }

    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }
}

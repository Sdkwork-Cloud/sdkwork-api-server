use async_trait::async_trait;
use sdkwork_api_app_billing::{
    book_usage_cost, check_quota, create_quota_policy, list_ledger_entries, persist_ledger_entry,
    persist_quota_policy, BillingQuotaStore,
};
use sdkwork_api_domain_billing::{LedgerEntry, QuotaPolicy};
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};
use std::sync::Mutex;

#[test]
fn booking_usage_creates_ledger_entry() {
    let ledger = book_usage_cost("project-1", 100, 0.25).unwrap();
    assert_eq!(ledger.project_id, "project-1");
}

#[tokio::test]
async fn persisted_ledger_can_be_listed() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    persist_ledger_entry(&store, "project-1", 100, 0.25)
        .await
        .unwrap();

    let ledger = list_ledger_entries(&store).await.unwrap();
    assert_eq!(ledger.len(), 1);
    assert_eq!(ledger[0].amount, 0.25);
}

#[tokio::test]
async fn quota_evaluation_rejects_requests_past_configured_limit() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let policy = create_quota_policy("quota-project-1", "project-1", 100, true).unwrap();
    persist_quota_policy(&store, &policy).await.unwrap();
    persist_ledger_entry(&store, "project-1", 70, 0.25)
        .await
        .unwrap();

    let evaluation = check_quota(&store, "project-1", 40).await.unwrap();
    assert!(!evaluation.allowed);
    assert_eq!(evaluation.policy_id.as_deref(), Some("quota-project-1"));
    assert_eq!(evaluation.used_units, 70);
    assert_eq!(evaluation.limit_units, Some(100));
}

#[tokio::test]
async fn quota_evaluation_uses_project_scoped_reads_only() {
    let store = RecordingQuotaStore::new(
        vec![
            LedgerEntry::new("project-1", 70, 0.25),
            LedgerEntry::new("project-2", 999, 9.99),
        ],
        vec![
            QuotaPolicy::new("quota-project-1", "project-1", 100).with_enabled(true),
            QuotaPolicy::new("quota-project-2", "project-2", 5).with_enabled(true),
        ],
    );

    let evaluation = check_quota(&store, "project-1", 40).await.unwrap();

    assert!(!evaluation.allowed);
    assert_eq!(evaluation.policy_id.as_deref(), Some("quota-project-1"));
    assert_eq!(evaluation.used_units, 70);
    assert_eq!(evaluation.limit_units, Some(100));
    assert_eq!(
        store.last_ledger_project.lock().unwrap().as_deref(),
        Some("project-1")
    );
    assert_eq!(
        store.last_policy_project.lock().unwrap().as_deref(),
        Some("project-1")
    );
}

struct RecordingQuotaStore {
    ledger_entries: Vec<LedgerEntry>,
    quota_policies: Vec<QuotaPolicy>,
    last_ledger_project: Mutex<Option<String>>,
    last_policy_project: Mutex<Option<String>>,
}

impl RecordingQuotaStore {
    fn new(ledger_entries: Vec<LedgerEntry>, quota_policies: Vec<QuotaPolicy>) -> Self {
        Self {
            ledger_entries,
            quota_policies,
            last_ledger_project: Mutex::new(None),
            last_policy_project: Mutex::new(None),
        }
    }
}

#[async_trait]
impl BillingQuotaStore for RecordingQuotaStore {
    async fn list_ledger_entries_for_project(
        &self,
        project_id: &str,
    ) -> anyhow::Result<Vec<LedgerEntry>> {
        *self.last_ledger_project.lock().unwrap() = Some(project_id.to_owned());
        Ok(self
            .ledger_entries
            .iter()
            .filter(|entry| entry.project_id == project_id)
            .cloned()
            .collect())
    }

    async fn list_quota_policies_for_project(
        &self,
        project_id: &str,
    ) -> anyhow::Result<Vec<QuotaPolicy>> {
        *self.last_policy_project.lock().unwrap() = Some(project_id.to_owned());
        Ok(self
            .quota_policies
            .iter()
            .filter(|policy| policy.project_id == project_id)
            .cloned()
            .collect())
    }
}

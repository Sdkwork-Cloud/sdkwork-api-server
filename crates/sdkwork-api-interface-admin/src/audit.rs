use super::*;

use std::sync::atomic::{AtomicU64, Ordering};

static ADMIN_AUDIT_EVENT_SEQUENCE: AtomicU64 = AtomicU64::new(1);

pub(crate) const APPROVAL_SCOPE_FINANCE_CONTROL: &str = "finance_control";
pub(crate) const APPROVAL_SCOPE_IDENTITY_CONTROL: &str = "identity_control";
pub(crate) const APPROVAL_SCOPE_SECRET_CONTROL: &str = "secret_control";

fn next_admin_audit_event_id(recorded_at_ms: u64) -> String {
    let sequence = ADMIN_AUDIT_EVENT_SEQUENCE.fetch_add(1, Ordering::Relaxed) & 0xffff_ffff;
    format!("audit_{recorded_at_ms:016x}_{sequence:08x}")
}

pub(crate) async fn record_admin_audit_event(
    state: &AdminApiState,
    claims: &AuthenticatedAdminClaims,
    action: &str,
    resource_type: &str,
    resource_id: impl Into<String>,
    approval_scope: &str,
) -> Result<AdminAuditEventRecord, StatusCode> {
    let recorded_at_ms = unix_timestamp_ms();
    let event = AdminAuditEventRecord::new(
        next_admin_audit_event_id(recorded_at_ms),
        action,
        resource_type,
        resource_id,
        approval_scope,
        claims.user().id.clone(),
        claims.user().email.clone(),
        claims.role(),
        recorded_at_ms,
    );
    state
        .store
        .insert_admin_audit_event(&event)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub(crate) async fn list_admin_audit_events_handler(
    claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<AdminAuditEventRecord>>, StatusCode> {
    require_admin_privilege(&claims, AdminPrivilege::IdentityRead)?;
    state
        .store
        .list_admin_audit_events()
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

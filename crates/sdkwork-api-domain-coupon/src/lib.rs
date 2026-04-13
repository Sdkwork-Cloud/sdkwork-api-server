use serde::{Deserialize, Serialize};

/// Compatibility-era coupon record.
///
/// This shape remains available while the canonical marketing system migrates
/// toward template, batch, code, claim, and redemption records.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CouponCampaign {
    pub id: String,
    pub code: String,
    pub discount_label: String,
    pub audience: String,
    pub remaining: u64,
    pub active: bool,
    pub note: String,
    pub expires_on: String,
    #[serde(default)]
    pub created_at_ms: u64,
}

impl CouponCampaign {
    /// Creates the legacy single-code campaign model that older admin and
    /// portal flows still consume during the migration window.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        code: impl Into<String>,
        discount_label: impl Into<String>,
        audience: impl Into<String>,
        remaining: u64,
        active: bool,
        note: impl Into<String>,
        expires_on: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            code: code.into(),
            discount_label: discount_label.into(),
            audience: audience.into(),
            remaining,
            active,
            note: note.into(),
            expires_on: expires_on.into(),
            created_at_ms: 0,
        }
    }

    pub fn with_created_at_ms(mut self, created_at_ms: u64) -> Self {
        self.created_at_ms = created_at_ms;
        self
    }
}

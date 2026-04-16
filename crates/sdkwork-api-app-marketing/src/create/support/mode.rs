use super::errors::marketing_create_conflicting_existing_state;
use crate::governance::MarketingGovernanceError;

#[derive(Clone, Copy)]
pub(crate) enum PersistMode {
    Create,
    Ensure,
}

impl PersistMode {
    pub(crate) fn resolve_existing_primary_with<T>(
        self,
        aggregate_label: &str,
        aggregate_id: &str,
        existing: T,
        desired: &T,
        matches: impl FnOnce(&T, &T) -> bool,
    ) -> Result<T, MarketingGovernanceError> {
        match self {
            Self::Create => Err(MarketingGovernanceError::Conflict(format!(
                "{aggregate_label} {aggregate_id} already exists"
            ))),
            Self::Ensure => {
                if matches(&existing, desired) {
                    Ok(existing)
                } else {
                    Err(marketing_create_conflicting_existing_state(
                        aggregate_label,
                        aggregate_id,
                    ))
                }
            }
        }
    }

    pub(crate) fn resolve_existing_unique<T: PartialEq>(
        self,
        existing: T,
        desired: &T,
        conflict: impl FnOnce(&T) -> MarketingGovernanceError,
    ) -> Result<T, MarketingGovernanceError> {
        match self {
            Self::Create => Err(conflict(&existing)),
            Self::Ensure => {
                if &existing == desired {
                    Ok(existing)
                } else {
                    Err(conflict(&existing))
                }
            }
        }
    }
}

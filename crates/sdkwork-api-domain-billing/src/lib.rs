#[derive(Debug, Clone, PartialEq)]
pub struct LedgerEntry {
    pub project_id: String,
    pub units: u64,
    pub amount: f64,
}

impl LedgerEntry {
    pub fn new(project_id: impl Into<String>, units: u64, amount: f64) -> Self {
        Self {
            project_id: project_id.into(),
            units,
            amount,
        }
    }
}

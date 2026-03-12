use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SseFrame(String);

impl SseFrame {
    pub fn data(payload: &str) -> Self {
        Self(format!("data: {payload}\n\n"))
    }
}

impl Display for SseFrame {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilitySupport {
    Supported,
    Unsupported,
}

pub trait ProviderAdapter {
    fn id(&self) -> &'static str;
}

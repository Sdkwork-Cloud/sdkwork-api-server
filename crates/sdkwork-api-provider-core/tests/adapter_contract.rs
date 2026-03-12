use sdkwork_api_provider_core::{CapabilitySupport, ProviderAdapter};

struct DummyAdapter;

impl ProviderAdapter for DummyAdapter {
    fn id(&self) -> &'static str {
        "dummy"
    }
}

#[test]
fn adapter_exposes_identifier() {
    let _ = CapabilitySupport::Supported;
    assert_eq!(DummyAdapter.id(), "dummy");
}

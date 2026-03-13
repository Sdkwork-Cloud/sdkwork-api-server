use sdkwork_api_provider_core::{
    ProviderExecutionAdapter, ProviderOutput, ProviderRegistry, ProviderRequest,
};

#[derive(Debug, Clone)]
struct DummyAdapter {
    base_url: String,
}

impl sdkwork_api_provider_core::ProviderAdapter for DummyAdapter {
    fn id(&self) -> &'static str {
        "dummy"
    }
}

#[async_trait::async_trait]
impl ProviderExecutionAdapter for DummyAdapter {
    async fn execute(
        &self,
        _api_key: &str,
        _request: ProviderRequest<'_>,
    ) -> anyhow::Result<ProviderOutput> {
        Ok(ProviderOutput::Json(serde_json::json!({
            "base_url": self.base_url,
        })))
    }
}

#[tokio::test]
async fn registry_executes_registered_adapter_factory() {
    let mut registry = ProviderRegistry::new();
    registry.register_factory("dummy", |base_url| Box::new(DummyAdapter { base_url }));

    let request = sdkwork_api_contract_openai::responses::CreateResponseRequest {
        model: "gpt-4.1".to_owned(),
        input: serde_json::json!("hi"),
        stream: Some(false),
    };

    let adapter = registry
        .resolve("dummy", "https://example.com")
        .expect("adapter should exist");
    let output = adapter
        .execute("sk-test", ProviderRequest::Responses(&request))
        .await
        .unwrap()
        .into_json()
        .expect("json output");

    assert_eq!(output["base_url"], "https://example.com");
}

#[test]
fn registry_returns_none_for_unknown_adapter_kind() {
    let registry = ProviderRegistry::new();
    assert!(registry.resolve("missing", "https://example.com").is_none());
}

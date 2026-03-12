use sdkwork_api_domain_catalog::ModelCatalogEntry;

pub fn map_model_object(model: &str) -> ModelCatalogEntry {
    ModelCatalogEntry::new(model, "provider-openai-official")
}

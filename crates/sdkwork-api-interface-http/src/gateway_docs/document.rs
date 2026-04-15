use super::*;

const GATEWAY_OPENAPI_SPEC: OpenApiServiceSpec<'static> = OpenApiServiceSpec {
    title: "SDKWORK Gateway API",
    version: env!("CARGO_PKG_VERSION"),
    description: "OpenAPI 3.1 schema generated from the current gateway router implementation.",
    openapi_path: "/openapi.json",
    docs_path: "/docs",
};

fn gateway_openapi_document() -> &'static Value {
    static DOCUMENT: OnceLock<Value> = OnceLock::new();
    DOCUMENT.get_or_init(|| {
        serde_json::to_value(crate::gateway_openapi::gateway_openapi_document())
            .expect("gateway openapi document should serialize")
    })
}

fn gateway_docs_html() -> &'static str {
    static HTML: OnceLock<String> = OnceLock::new();
    HTML.get_or_init(|| render_docs_html(&GATEWAY_OPENAPI_SPEC))
        .as_str()
}

pub(crate) async fn gateway_openapi_handler() -> Json<Value> {
    Json(gateway_openapi_document().clone())
}

pub(crate) async fn gateway_docs_handler() -> Html<String> {
    Html(gateway_docs_html().to_owned())
}

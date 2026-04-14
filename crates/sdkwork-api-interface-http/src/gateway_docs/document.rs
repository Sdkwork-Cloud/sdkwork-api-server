use super::presentation::{
    gateway_operation_summary, gateway_route_requires_bearer_auth, gateway_tag_for_path,
};
use super::*;
use std::collections::{BTreeMap, BTreeSet};

const GATEWAY_OPENAPI_SPEC: OpenApiServiceSpec<'static> = OpenApiServiceSpec {
    title: "SDKWORK Gateway API",
    version: env!("CARGO_PKG_VERSION"),
    description: "OpenAPI 3.1 inventory generated from the current gateway router implementation.",
    openapi_path: "/openapi.json",
    docs_path: "/docs",
};

fn gateway_route_inventory() -> &'static [RouteEntry] {
    static ROUTES: OnceLock<Vec<RouteEntry>> = OnceLock::new();
    ROUTES.get_or_init(build_gateway_route_inventory).as_slice()
}

fn build_gateway_route_inventory() -> Vec<RouteEntry> {
    let route_sources = [
        (
            include_str!("../gateway_router_common.rs"),
            "gateway_base_router",
        ),
        (
            include_str!("../gateway_stateful_route_groups/compat_and_model.rs"),
            "apply_stateful_compat_and_model_routes",
        ),
        (
            include_str!("../gateway_stateful_route_groups/chat_and_conversation.rs"),
            "apply_stateful_chat_and_conversation_routes",
        ),
        (
            include_str!("../gateway_stateful_route_groups/thread_and_response.rs"),
            "apply_stateful_thread_and_response_routes",
        ),
        (
            include_str!("../gateway_stateful_route_groups/inference_and_storage.rs"),
            "apply_stateful_inference_and_storage_routes",
        ),
        (
            include_str!("../gateway_stateful_route_groups/video_and_upload.rs"),
            "apply_stateful_video_and_upload_routes",
        ),
        (
            include_str!("../gateway_stateful_route_groups/management.rs"),
            "apply_stateful_management_routes",
        ),
        (
            include_str!("../gateway_stateful_route_groups/eval_and_vector.rs"),
            "apply_stateful_eval_and_vector_routes",
        ),
    ];

    let mut routes_by_path: BTreeMap<String, BTreeSet<HttpMethod>> = BTreeMap::new();
    for (source, function_name) in route_sources {
        let Ok(routes) = extract_routes_from_function(source, function_name) else {
            continue;
        };
        for route in routes {
            routes_by_path
                .entry(route.path)
                .or_default()
                .extend(route.methods);
        }
    }

    routes_by_path
        .into_iter()
        .map(|(path, methods)| RouteEntry {
            path,
            methods: methods.into_iter().collect(),
        })
        .collect()
}

fn gateway_openapi_document() -> &'static Value {
    static DOCUMENT: OnceLock<Value> = OnceLock::new();
    DOCUMENT.get_or_init(|| {
        build_openapi_document(
            &GATEWAY_OPENAPI_SPEC,
            gateway_route_inventory(),
            gateway_tag_for_path,
            gateway_route_requires_bearer_auth,
            gateway_operation_summary,
        )
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

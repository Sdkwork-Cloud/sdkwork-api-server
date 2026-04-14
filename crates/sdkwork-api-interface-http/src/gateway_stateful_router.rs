use super::*;

pub(crate) fn build_stateful_gateway_router(state: GatewayApiState) -> Router {
    let service_name = gateway_service_name();
    let metrics = gateway_http_metrics();
    let router = gateway_base_router::<GatewayApiState>(metrics.clone(), None);
    let router = apply_stateful_compat_and_model_routes(router);
    let router = apply_stateful_chat_and_conversation_routes(router);
    let router = apply_stateful_thread_and_response_routes(router);
    let router = apply_stateful_inference_and_storage_routes(router);
    let router = apply_stateful_video_and_upload_routes(router);
    let router = apply_stateful_management_routes(router);
    let router = apply_stateful_eval_and_vector_routes(router);
    finalize_stateful_gateway_router(router, state, service_name, metrics, None)
}

pub(crate) fn build_stateful_gateway_router_with_http_exposure(
    state: GatewayApiState,
    http_exposure: sdkwork_api_config::HttpExposureConfig,
) -> Router {
    let service_name = gateway_service_name();
    let metrics = gateway_http_metrics();
    let router = gateway_base_router::<GatewayApiState>(metrics.clone(), Some(&http_exposure));
    let router = apply_stateful_compat_and_model_routes(router);
    let router = apply_stateful_chat_and_conversation_routes(router);
    let router = apply_stateful_thread_and_response_routes(router);
    let router = apply_stateful_inference_and_storage_routes(router);
    let router = apply_stateful_video_and_upload_routes(router);
    let router = apply_stateful_management_routes(router);
    let router = apply_stateful_eval_and_vector_routes(router);
    finalize_stateful_gateway_router(router, state, service_name, metrics, Some(&http_exposure))
}

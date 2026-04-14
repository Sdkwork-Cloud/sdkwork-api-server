use super::*;

pub(crate) fn build_stateless_gateway_router(config: StatelessGatewayConfig) -> Router {
    let service_name = gateway_service_name();
    let metrics = gateway_http_metrics();
    let router = gateway_base_router::<StatelessGatewayContext>(metrics.clone(), None);
    let router = apply_stateless_compat_and_model_routes(router);
    let router = apply_stateless_chat_and_conversation_routes(router);
    let router = apply_stateless_thread_and_response_routes(router);
    let router = apply_stateless_inference_and_storage_routes(router);
    let router = apply_stateless_video_and_upload_routes(router);
    let router = apply_stateless_management_routes(router);
    let router = apply_stateless_eval_and_vector_routes(router);
    finalize_stateless_gateway_router(router, config, service_name, metrics, None)
}

pub(crate) fn build_stateless_gateway_router_with_http_exposure(
    config: StatelessGatewayConfig,
    http_exposure: sdkwork_api_config::HttpExposureConfig,
) -> Router {
    let service_name = gateway_service_name();
    let metrics = gateway_http_metrics();
    let router =
        gateway_base_router::<StatelessGatewayContext>(metrics.clone(), Some(&http_exposure));
    let router = apply_stateless_compat_and_model_routes(router);
    let router = apply_stateless_chat_and_conversation_routes(router);
    let router = apply_stateless_thread_and_response_routes(router);
    let router = apply_stateless_inference_and_storage_routes(router);
    let router = apply_stateless_video_and_upload_routes(router);
    let router = apply_stateless_management_routes(router);
    let router = apply_stateless_eval_and_vector_routes(router);
    finalize_stateless_gateway_router(router, config, service_name, metrics, Some(&http_exposure))
}

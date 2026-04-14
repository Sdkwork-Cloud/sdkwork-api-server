use super::*;

pub(crate) async fn relay_stateless_json_request(
    request_context: &StatelessGatewayRequest,
    request: ProviderRequest<'_>,
) -> anyhow::Result<Option<Value>> {
    let options = ProviderRequestOptions::default();
    relay_stateless_json_request_with_options(request_context, request, &options).await
}

pub(crate) async fn relay_stateless_json_request_with_options(
    request_context: &StatelessGatewayRequest,
    request: ProviderRequest<'_>,
    options: &ProviderRequestOptions,
) -> anyhow::Result<Option<Value>> {
    let Some(upstream) = request_context.upstream() else {
        return Ok(None);
    };

    execute_json_provider_request_with_runtime_and_options(
        upstream.runtime_key(),
        upstream.base_url(),
        upstream.api_key(),
        request,
        options,
    )
    .await
}

pub(crate) async fn relay_stateless_stream_request(
    request_context: &StatelessGatewayRequest,
    request: ProviderRequest<'_>,
) -> anyhow::Result<Option<ProviderStreamOutput>> {
    let options = ProviderRequestOptions::default();
    relay_stateless_stream_request_with_options(request_context, request, &options).await
}

pub(crate) async fn relay_stateless_stream_request_with_options(
    request_context: &StatelessGatewayRequest,
    request: ProviderRequest<'_>,
    options: &ProviderRequestOptions,
) -> anyhow::Result<Option<ProviderStreamOutput>> {
    let Some(upstream) = request_context.upstream() else {
        return Ok(None);
    };

    execute_stream_provider_request_with_runtime_and_options(
        upstream.runtime_key(),
        upstream.base_url(),
        upstream.api_key(),
        request,
        options,
    )
    .await
}

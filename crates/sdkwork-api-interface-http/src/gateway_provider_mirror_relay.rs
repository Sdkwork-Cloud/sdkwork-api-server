use crate::gateway_prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProviderMirrorAuthStyle {
    AuthorizationBearer,
    AuthorizationToken,
}

fn provider_mirror_http_client() -> &'static reqwest::Client {
    static PROVIDER_MIRROR_HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    PROVIDER_MIRROR_HTTP_CLIENT.get_or_init(|| {
        sdkwork_api_kernel::ensure_reqwest_rustls_provider();
        reqwest::Client::new()
    })
}

fn provider_mirror_upstream_url(base_url: &str, request: &Request<Body>) -> String {
    let path_and_query = request
        .uri()
        .path_and_query()
        .map(|value| value.as_str())
        .unwrap_or_else(|| request.uri().path());
    format!("{}{}", base_url.trim_end_matches('/'), path_and_query)
}

fn provider_mirror_status(status: reqwest::StatusCode) -> StatusCode {
    StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::BAD_GATEWAY)
}

fn provider_mirror_auth_style(mirror_protocol_identity: &str) -> ProviderMirrorAuthStyle {
    match mirror_protocol_identity.trim() {
        "vidu" => ProviderMirrorAuthStyle::AuthorizationToken,
        _ => ProviderMirrorAuthStyle::AuthorizationBearer,
    }
}

fn provider_mirror_authorization_value(
    auth_style: ProviderMirrorAuthStyle,
    api_key: &str,
) -> String {
    match auth_style {
        ProviderMirrorAuthStyle::AuthorizationBearer => format!("Bearer {api_key}"),
        ProviderMirrorAuthStyle::AuthorizationToken => format!("Token {api_key}"),
    }
}

fn should_forward_provider_mirror_header(header_name: &header::HeaderName) -> bool {
    !matches!(
        header_name.as_str(),
        "authorization" | "host" | "content-length"
    )
}

async fn provider_mirror_error_response(response: reqwest::Response) -> Response {
    let status = provider_mirror_status(response.status());
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .cloned();
    let body = response.bytes().await.unwrap_or_default();

    let mut builder = Response::builder().status(status);
    if let Some(content_type) = content_type {
        builder = builder.header(header::CONTENT_TYPE, content_type);
    }
    match builder.body(Body::from(body)) {
        Ok(response) => response,
        Err(_) => {
            bad_gateway_openai_response("failed to process upstream provider mirror error response")
        }
    }
}

pub(crate) enum ProviderMirrorJsonRelayOutcome {
    Json(Value),
    Error(Response),
}

pub(crate) async fn relay_provider_mirror_json_request(
    mirror_protocol_identity: &str,
    base_url: &str,
    api_key: &str,
    request: Request<Body>,
) -> anyhow::Result<ProviderMirrorJsonRelayOutcome> {
    let method = reqwest::Method::from_bytes(request.method().as_str().as_bytes())?;
    let upstream_url = provider_mirror_upstream_url(base_url, &request);
    let headers = request.headers().clone();
    let body = axum::body::to_bytes(request.into_body(), usize::MAX).await?;
    let auth_style = provider_mirror_auth_style(mirror_protocol_identity);

    let mut outbound_request = provider_mirror_http_client()
        .request(method, upstream_url)
        .header(
            reqwest::header::AUTHORIZATION,
            provider_mirror_authorization_value(auth_style, api_key),
        );

    for (header_name, header_value) in &headers {
        if should_forward_provider_mirror_header(header_name) {
            outbound_request = outbound_request.header(header_name.as_str(), header_value.clone());
        }
    }

    let response = outbound_request.body(body).send().await?;
    if !response.status().is_success() {
        return Ok(ProviderMirrorJsonRelayOutcome::Error(
            provider_mirror_error_response(response).await,
        ));
    }

    Ok(ProviderMirrorJsonRelayOutcome::Json(response.json().await?))
}

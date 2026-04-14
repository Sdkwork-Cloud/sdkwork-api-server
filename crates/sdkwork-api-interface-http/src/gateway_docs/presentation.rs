use super::*;

pub(crate) fn gateway_tag_for_path(path: &str) -> String {
    match path {
        "/metrics" | "/health" => "system".to_owned(),
        "/docs" | "/openapi.json" => "docs".to_owned(),
        _ if path.starts_with("/v1/") || path.starts_with("/v1beta/") => path
            .trim_start_matches("/v1/")
            .trim_start_matches("/v1beta/")
            .split('/')
            .find(|segment| !segment.is_empty() && !segment.starts_with('{'))
            .unwrap_or("gateway")
            .to_owned(),
        _ => "gateway".to_owned(),
    }
}

pub(crate) fn gateway_route_requires_bearer_auth(path: &str, _method: HttpMethod) -> bool {
    path == "/metrics" || path.starts_with("/v1/") || path.starts_with("/v1beta/")
}

pub(crate) fn gateway_operation_summary(path: &str, method: HttpMethod) -> String {
    match path {
        "/metrics" => "Prometheus metrics".to_owned(),
        "/health" => "Health check".to_owned(),
        "/openapi.json" => "OpenAPI document".to_owned(),
        "/docs" => "Interactive API inventory".to_owned(),
        _ => format!(
            "{} {}",
            method.display_name(),
            humanize_route_path(
                path,
                if path.starts_with("/v1beta/") {
                    Some("v1beta")
                } else if path.starts_with("/v1/") {
                    Some("v1")
                } else {
                    None
                },
            )
        ),
    }
}

pub(crate) fn browser_cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
}

fn humanize_route_path(path: &str, ignored_prefix: Option<&str>) -> String {
    let parts = path
        .trim_matches('/')
        .split('/')
        .filter(|segment| !segment.is_empty())
        .filter(|segment| Some(*segment) != ignored_prefix)
        .map(|segment| {
            if segment.starts_with('{') && segment.ends_with('}') {
                format!(
                    "by {}",
                    segment
                        .trim_matches(|ch| ch == '{' || ch == '}')
                        .replace(['_', '-'], " ")
                )
            } else {
                segment.replace(['_', '-'], " ")
            }
        })
        .collect::<Vec<_>>();

    if parts.is_empty() {
        "root".to_owned()
    } else {
        parts.join(" / ")
    }
}

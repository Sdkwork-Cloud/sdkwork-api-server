use super::*;

pub(crate) async fn apply_request_routing_region(request: Request<Body>, next: Next) -> Response {
    let requested_region = request
        .headers()
        .get("x-sdkwork-region")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    with_request_routing_region(requested_region, next.run(request)).await
}

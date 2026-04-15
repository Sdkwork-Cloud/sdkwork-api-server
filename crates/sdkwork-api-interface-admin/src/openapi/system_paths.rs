#[utoipa::path(
    get,
    path = "/admin/health",
    tag = "system",
    responses((status = 200, description = "Admin health check response.", body = String))
)]
pub(super) async fn health() {}

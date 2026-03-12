use axum::{
    routing::{get, post},
    Router,
};

pub fn admin_router() -> Router {
    Router::new()
        .route("/admin/health", get(|| async { "ok" }))
        .route("/admin/auth/login", post(|| async { "login" }))
        .route("/admin/auth/me", get(|| async { "me" }))
        .route("/admin/tenants", get(|| async { "tenants" }))
        .route("/admin/projects", get(|| async { "projects" }))
        .route("/admin/api-keys", get(|| async { "api-keys" }))
        .route("/admin/channels", get(|| async { "channels" }))
        .route("/admin/providers", get(|| async { "providers" }))
        .route("/admin/credentials", get(|| async { "credentials" }))
        .route("/admin/models", get(|| async { "models" }))
        .route("/admin/routing/policies", get(|| async { "policies" }))
        .route(
            "/admin/routing/simulations",
            post(|| async { "simulations" }),
        )
}

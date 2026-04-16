#[cfg(windows)]
use std::sync::Arc;
use std::{
    fs,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, TcpListener, TcpStream},
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

use anyhow::{anyhow, bail, Context, Result};
use async_trait::async_trait;
#[cfg(windows)]
use axum::{
    body::Body,
    extract::{Request as AxumRequest, State},
    response::Response as AxumResponse,
    Router,
};
use bytes::Bytes;
#[cfg(windows)]
use http::{
    header::{self, HeaderName, HeaderValue},
    HeaderMap, Method, StatusCode,
};
use http::{uri::PathAndQuery, Uri};
#[cfg(not(windows))]
use pingora_core::server::Server;
use pingora_core::{upstreams::peer::HttpPeer, Error, ErrorType, Result as PingoraResult};
use pingora_http::ResponseHeader;
#[cfg(not(windows))]
use pingora_proxy::http_proxy_service;
use pingora_proxy::{ProxyHttp, Session};

const BROWSER_CORS_ALLOW_METHODS: &str = "GET, POST, PUT, PATCH, DELETE, OPTIONS, HEAD";
const BROWSER_CORS_ALLOW_HEADERS: &str =
    "authorization, content-type, x-api-key, anthropic-version, anthropic-beta, x-request-id";
const BROWSER_CORS_EXPOSE_HEADERS: &str = "content-length, content-type, location";
const BROWSER_CORS_MAX_AGE: &str = "86400";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeSite {
    Admin,
    Portal,
}

impl RuntimeSite {
    fn mount_prefix(self) -> &'static str {
        match self {
            Self::Admin => "/admin",
            Self::Portal => "/portal",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeRoute {
    Redirect(String),
    Proxy {
        upstream: String,
        request_path: String,
    },
    Static {
        site: RuntimeSite,
        request_path: String,
    },
    NotFound,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SiteAsset {
    pub site: RuntimeSite,
    pub filesystem_path: PathBuf,
    pub content_type: String,
    pub cache_control: String,
}

#[cfg_attr(windows, allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimeHostBackend {
    Pingora,
    Axum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbeddedRuntime {
    base_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeHostConfig {
    pub bind_addr: String,
    pub admin_site_dir: PathBuf,
    pub portal_site_dir: PathBuf,
    pub admin_upstream: String,
    pub portal_upstream: String,
    pub gateway_upstream: String,
    pub admin_site_proxy_upstream: Option<String>,
    pub portal_site_proxy_upstream: Option<String>,
    pub browser_allowed_origins: Vec<String>,
}

impl RuntimeHostConfig {
    pub fn new(
        bind_addr: impl Into<String>,
        admin_site_dir: impl Into<PathBuf>,
        portal_site_dir: impl Into<PathBuf>,
        admin_upstream: impl Into<String>,
        portal_upstream: impl Into<String>,
        gateway_upstream: impl Into<String>,
    ) -> Self {
        Self {
            bind_addr: bind_addr.into(),
            admin_site_dir: admin_site_dir.into(),
            portal_site_dir: portal_site_dir.into(),
            admin_upstream: admin_upstream.into(),
            portal_upstream: portal_upstream.into(),
            gateway_upstream: gateway_upstream.into(),
            admin_site_proxy_upstream: None,
            portal_site_proxy_upstream: None,
            browser_allowed_origins: Vec::new(),
        }
    }

    pub fn local_defaults(bind_addr: impl Into<String>) -> Self {
        Self::new(
            bind_addr,
            PathBuf::from("apps/sdkwork-router-admin/dist"),
            PathBuf::from("apps/sdkwork-router-portal/dist"),
            "127.0.0.1:9981",
            "127.0.0.1:9982",
            "127.0.0.1:9980",
        )
    }

    pub fn with_browser_allowed_origins<I, S>(mut self, origins: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.browser_allowed_origins = origins.into_iter().map(Into::into).collect();
        self
    }
}

impl EmbeddedRuntime {
    pub async fn start_ephemeral() -> Result<Self> {
        let bind_addr = reserve_bind_addr("127.0.0.1:0")?;
        Self::start(RuntimeHostConfig::local_defaults(bind_addr)).await
    }

    pub async fn start(mut config: RuntimeHostConfig) -> Result<Self> {
        if uses_ephemeral_port(&config.bind_addr) {
            config.bind_addr = reserve_bind_addr(&config.bind_addr)?;
        }

        let bind_addr = config.bind_addr.clone();
        let base_url = format!("http://{}", bind_addr);

        thread::Builder::new()
            .name("sdkwork-router-runtime-host".to_owned())
            .spawn(move || {
                if let Err(error) = serve_public_web(config) {
                    eprintln!("[sdkwork-api-runtime-host] {error:?}");
                }
            })
            .context("failed to spawn runtime host thread")?;

        wait_for_listener(&bind_addr)?;

        Ok(Self { base_url })
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

pub fn classify_request(request_path: &str) -> RuntimeRoute {
    let path = strip_request_suffix(request_path);

    if path == "/" {
        return RuntimeRoute::Redirect("/portal/".to_owned());
    }

    if path == "/portal" {
        return RuntimeRoute::Redirect("/portal/".to_owned());
    }

    if path == "/admin" {
        return RuntimeRoute::Redirect("/admin/".to_owned());
    }

    if path == "/openapi.json" || path == "/docs" {
        return RuntimeRoute::Proxy {
            upstream: "gateway".to_owned(),
            request_path: path.to_owned(),
        };
    }

    if let Some(suffix) = path.strip_prefix("/api/admin") {
        return RuntimeRoute::Proxy {
            upstream: "admin".to_owned(),
            request_path: rewrite_proxy_path("/admin", suffix),
        };
    }

    if let Some(suffix) = path.strip_prefix("/api/portal") {
        return RuntimeRoute::Proxy {
            upstream: "portal".to_owned(),
            request_path: rewrite_proxy_path("/portal", suffix),
        };
    }

    if let Some(suffix) = path.strip_prefix("/api/v1") {
        let request_path = match suffix {
            "/health" => "/health".to_owned(),
            "/metrics" => "/metrics".to_owned(),
            _ => rewrite_proxy_path("/v1", suffix),
        };
        return RuntimeRoute::Proxy {
            upstream: "gateway".to_owned(),
            request_path,
        };
    }

    if path.starts_with("/portal/") {
        return RuntimeRoute::Static {
            site: RuntimeSite::Portal,
            request_path: request_path.to_owned(),
        };
    }

    if path.starts_with("/admin/") {
        return RuntimeRoute::Static {
            site: RuntimeSite::Admin,
            request_path: request_path.to_owned(),
        };
    }

    RuntimeRoute::NotFound
}

fn site_proxy_upstream_name(site: RuntimeSite) -> &'static str {
    match site {
        RuntimeSite::Admin => "admin-site",
        RuntimeSite::Portal => "portal-site",
    }
}

fn site_proxy_upstream_target(
    config: &RuntimeHostConfig,
    site: RuntimeSite,
) -> Option<&str> {
    match site {
        RuntimeSite::Admin => config.admin_site_proxy_upstream.as_deref(),
        RuntimeSite::Portal => config.portal_site_proxy_upstream.as_deref(),
    }
}

pub fn resolve_runtime_route(config: &RuntimeHostConfig, request_path: &str) -> RuntimeRoute {
    match classify_request(request_path) {
        RuntimeRoute::Static { site, request_path } => {
            if site_proxy_upstream_target(config, site).is_some() {
                RuntimeRoute::Proxy {
                    upstream: site_proxy_upstream_name(site).to_owned(),
                    request_path,
                }
            } else {
                RuntimeRoute::Static { site, request_path }
            }
        }
        route => route,
    }
}

pub fn resolve_static_asset(
    site: RuntimeSite,
    request_path: &str,
    site_root: &Path,
) -> Result<SiteAsset> {
    let path = strip_request_suffix(request_path);
    let site_relative_path = path
        .strip_prefix(site.mount_prefix())
        .ok_or_else(|| anyhow!("request path does not belong to site mount"))?;

    let asset_path = if should_serve_index(site_relative_path) {
        site_root.join("index.html")
    } else {
        site_root.join(normalize_relative_path(site_relative_path)?)
    };

    let content_type = guess_content_type(&asset_path).to_owned();
    let cache_control = if asset_path.extension().and_then(|value| value.to_str()) == Some("html") {
        "no-cache".to_owned()
    } else {
        "public, max-age=31536000, immutable".to_owned()
    };

    Ok(SiteAsset {
        site,
        filesystem_path: asset_path,
        content_type,
        cache_control,
    })
}

fn strip_request_suffix(request_path: &str) -> &str {
    request_path
        .split(['#', '?'])
        .next()
        .filter(|value| !value.is_empty())
        .unwrap_or("/")
}

fn rewrite_proxy_path(prefix: &str, suffix: &str) -> String {
    if suffix.is_empty() {
        prefix.to_owned()
    } else {
        format!("{prefix}{suffix}")
    }
}

fn should_serve_index(site_relative_path: &str) -> bool {
    let trimmed = site_relative_path.trim_start_matches('/');
    trimmed.is_empty() || !trimmed.rsplit('/').next().unwrap_or_default().contains('.')
}

fn normalize_relative_path(site_relative_path: &str) -> Result<PathBuf> {
    let mut normalized = PathBuf::new();

    for segment in site_relative_path.trim_start_matches('/').split('/') {
        if segment.is_empty() || segment == "." {
            continue;
        }
        if segment == ".." {
            bail!("path traversal is not allowed");
        }
        normalized.push(segment);
    }

    if normalized.as_os_str().is_empty() {
        bail!("empty static asset path");
    }

    Ok(normalized)
}

fn guess_content_type(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
    {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "txt" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

#[cfg_attr(windows, allow(dead_code))]
fn selected_runtime_backend() -> RuntimeHostBackend {
    if cfg!(windows) {
        RuntimeHostBackend::Axum
    } else {
        RuntimeHostBackend::Pingora
    }
}

pub fn serve_public_web(config: RuntimeHostConfig) -> Result<()> {
    #[cfg(windows)]
    {
        serve_public_web_axum(config)
    }

    #[cfg(not(windows))]
    {
        serve_public_web_pingora(config)
    }
}

#[cfg(not(windows))]
fn serve_public_web_pingora(config: RuntimeHostConfig) -> Result<()> {
    let mut server = Server::new(None).map_err(|error| anyhow!(error.to_string()))?;
    server.bootstrap();

    let bind_addr = config.bind_addr.clone();
    let mut service = http_proxy_service(&server.configuration, RuntimeHostProxy { config });
    service.add_tcp(bind_addr.as_str());
    server.add_service(service);
    server.run_forever();
}

#[cfg(windows)]
#[derive(Clone)]
struct RuntimeHostState {
    config: RuntimeHostConfig,
    client: reqwest::Client,
}

#[cfg(windows)]
fn serve_public_web_axum(config: RuntimeHostConfig) -> Result<()> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("failed to build tokio runtime for runtime host")?;

    runtime.block_on(serve_public_web_axum_async(config))
}

#[cfg(windows)]
async fn serve_public_web_axum_async(config: RuntimeHostConfig) -> Result<()> {
    let bind_addr = config.bind_addr.clone();
    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .with_context(|| format!("failed to bind runtime host listener to {bind_addr}"))?;
    sdkwork_api_kernel::ensure_reqwest_rustls_provider();
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .context("failed to build runtime host reverse proxy client")?;
    let state = Arc::new(RuntimeHostState { config, client });

    axum::serve(listener, build_runtime_host_router(state))
        .await
        .with_context(|| format!("runtime host listener exited for {bind_addr}"))
}

#[cfg(windows)]
fn build_runtime_host_router(state: Arc<RuntimeHostState>) -> Router {
    Router::new()
        .fallback(runtime_host_handler)
        .with_state(state)
}

#[cfg(windows)]
async fn runtime_host_handler(
    State(state): State<Arc<RuntimeHostState>>,
    request: AxumRequest,
) -> AxumResponse {
    let request_path = request.uri().path().to_owned();
    let request_origin = request_origin_http_headers(request.headers());

    match resolve_runtime_route(&state.config, &request_path) {
        RuntimeRoute::Redirect(location) => redirect_response(&location),
        RuntimeRoute::Static { site, request_path } => {
            let site_root = match site {
                RuntimeSite::Admin => &state.config.admin_site_dir,
                RuntimeSite::Portal => &state.config.portal_site_dir,
            };

            match serve_static_asset_http(request.method(), site, &request_path, site_root) {
                Ok(response) => response,
                Err(_) => not_found_response(),
            }
        }
        RuntimeRoute::Proxy {
            upstream,
            request_path,
        } => {
            if request.method() == Method::OPTIONS {
                return browser_cors_preflight_response(&state.config, request_origin.as_deref());
            }

            match proxy_request(
                state.clone(),
                request,
                &upstream,
                &request_path,
                request_origin.clone(),
            )
            .await
            {
                Ok(response) => response,
                Err(error) => bad_gateway_response(error, &state.config, request_origin.as_deref()),
            }
        }
        RuntimeRoute::NotFound => not_found_response(),
    }
}

#[cfg(windows)]
async fn proxy_request(
    state: Arc<RuntimeHostState>,
    request: AxumRequest,
    upstream: &str,
    request_path: &str,
    request_origin: Option<String>,
) -> Result<AxumResponse> {
    let (parts, body) = request.into_parts();
    let upstream_addr = upstream_target(&state.config, upstream);
    let upstream_url = build_upstream_url(upstream_addr, request_path, parts.uri.query());
    let mut upstream_request = state.client.request(parts.method, &upstream_url);

    for (name, value) in &parts.headers {
        if should_skip_request_header(name) {
            continue;
        }
        upstream_request = upstream_request.header(name, value);
    }

    let upstream_response = upstream_request
        .body(reqwest::Body::wrap_stream(body.into_data_stream()))
        .send()
        .await
        .with_context(|| format!("failed to proxy request to {upstream_url}"))?;

    Ok(proxied_response(
        &state.config,
        request_origin.as_deref(),
        upstream_response,
    ))
}

#[cfg(windows)]
fn build_upstream_url(target: &str, request_path: &str, query: Option<&str>) -> String {
    let base = if target.contains("://") {
        target.trim_end_matches('/').to_owned()
    } else {
        format!("http://{}", target.trim_end_matches('/'))
    };
    let mut url = format!("{base}{request_path}");
    if let Some(query) = query {
        url.push('?');
        url.push_str(query);
    }
    url
}

#[cfg(windows)]
fn proxied_response(
    config: &RuntimeHostConfig,
    request_origin: Option<&str>,
    upstream_response: reqwest::Response,
) -> AxumResponse {
    let status = upstream_response.status();
    let headers = upstream_response.headers().clone();
    let mut response = AxumResponse::builder()
        .status(status)
        .body(Body::from_stream(upstream_response.bytes_stream()))
        .expect("valid proxied response");

    for (name, value) in &headers {
        if should_skip_response_header(name) {
            continue;
        }
        response.headers_mut().append(name, value.clone());
    }
    apply_browser_cors_http_headers(response.headers_mut(), config, request_origin);

    response
}

#[cfg(windows)]
fn redirect_response(location: &str) -> AxumResponse {
    let mut response = AxumResponse::builder()
        .status(StatusCode::FOUND)
        .body(Body::empty())
        .expect("valid redirect response");
    response.headers_mut().insert(
        header::LOCATION,
        HeaderValue::from_str(location).expect("valid redirect location"),
    );
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    response
        .headers_mut()
        .insert(header::CONTENT_LENGTH, HeaderValue::from_static("0"));
    response
}

#[cfg(windows)]
fn browser_cors_preflight_response(
    config: &RuntimeHostConfig,
    request_origin: Option<&str>,
) -> AxumResponse {
    let mut response = AxumResponse::builder()
        .status(StatusCode::NO_CONTENT)
        .body(Body::empty())
        .expect("valid preflight response");
    response
        .headers_mut()
        .insert(header::CONTENT_LENGTH, HeaderValue::from_static("0"));
    apply_browser_cors_http_headers(response.headers_mut(), config, request_origin);
    response
}

#[cfg(windows)]
fn bad_gateway_response(
    error: anyhow::Error,
    config: &RuntimeHostConfig,
    request_origin: Option<&str>,
) -> AxumResponse {
    let mut response = AxumResponse::builder()
        .status(StatusCode::BAD_GATEWAY)
        .body(Body::from(format!("bad gateway: {error:#}")))
        .expect("valid bad gateway response");
    apply_browser_cors_http_headers(response.headers_mut(), config, request_origin);
    response
}

#[cfg(windows)]
fn not_found_response() -> AxumResponse {
    AxumResponse::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::empty())
        .expect("valid not found response")
}

#[cfg(windows)]
fn serve_static_asset_http(
    method: &Method,
    site: RuntimeSite,
    request_path: &str,
    site_root: &Path,
) -> Result<AxumResponse> {
    if method != Method::GET && method != Method::HEAD {
        bail!("static site only allows GET and HEAD");
    }

    let asset = resolve_static_asset(site, request_path, site_root)?;
    if !asset.filesystem_path.is_file() {
        bail!("static asset does not exist");
    }

    let body = read_static_asset_body(&asset)?;
    let content_length = body.len().to_string();
    let mut response = AxumResponse::builder()
        .status(StatusCode::OK)
        .body(if method == Method::HEAD {
            Body::empty()
        } else {
            Body::from(body)
        })
        .expect("valid static asset response");
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(asset.content_type.as_str()).expect("valid content type"),
    );
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_str(asset.cache_control.as_str()).expect("valid cache control"),
    );
    response.headers_mut().insert(
        header::CONTENT_LENGTH,
        HeaderValue::from_str(content_length.as_str()).expect("valid content length"),
    );

    Ok(response)
}

#[cfg(windows)]
fn apply_browser_cors_http_headers(
    headers: &mut HeaderMap,
    config: &RuntimeHostConfig,
    request_origin: Option<&str>,
) {
    let Some(allow_origin) = resolve_browser_cors_allow_origin(config, request_origin) else {
        return;
    };

    headers.insert(
        HeaderName::from_static("access-control-allow-origin"),
        HeaderValue::from_str(allow_origin).expect("valid cors allow origin"),
    );
    headers.insert(
        HeaderName::from_static("access-control-allow-methods"),
        HeaderValue::from_static(BROWSER_CORS_ALLOW_METHODS),
    );
    headers.insert(
        HeaderName::from_static("access-control-allow-headers"),
        HeaderValue::from_static(BROWSER_CORS_ALLOW_HEADERS),
    );
    headers.insert(
        HeaderName::from_static("access-control-expose-headers"),
        HeaderValue::from_static(BROWSER_CORS_EXPOSE_HEADERS),
    );
    headers.insert(
        HeaderName::from_static("access-control-max-age"),
        HeaderValue::from_static(BROWSER_CORS_MAX_AGE),
    );
    headers.insert(header::VARY, HeaderValue::from_static("origin"));
}

fn resolve_browser_cors_allow_origin<'a>(
    config: &'a RuntimeHostConfig,
    request_origin: Option<&'a str>,
) -> Option<&'a str> {
    if config
        .browser_allowed_origins
        .iter()
        .any(|origin| origin.trim() == "*")
    {
        return Some("*");
    }

    let request_origin = request_origin?.trim();
    if request_origin.is_empty() {
        return None;
    }

    config
        .browser_allowed_origins
        .iter()
        .any(|origin| origin == request_origin)
        .then_some(request_origin)
}

#[cfg(windows)]
fn request_origin_http_headers(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn request_origin_pingora(session: &Session) -> Option<String> {
    session
        .req_header()
        .headers
        .get("origin")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn upstream_target<'a>(config: &'a RuntimeHostConfig, upstream: &str) -> &'a str {
    match upstream {
        "admin" => config.admin_upstream.as_str(),
        "admin-site" => config
            .admin_site_proxy_upstream
            .as_deref()
            .unwrap_or(config.admin_upstream.as_str()),
        "portal" => config.portal_upstream.as_str(),
        "portal-site" => config
            .portal_site_proxy_upstream
            .as_deref()
            .unwrap_or(config.portal_upstream.as_str()),
        "gateway" => config.gateway_upstream.as_str(),
        _ => config.portal_upstream.as_str(),
    }
}

#[cfg(windows)]
fn should_skip_request_header(name: &HeaderName) -> bool {
    name == header::HOST || is_hop_by_hop_header(name)
}

#[cfg(windows)]
fn should_skip_response_header(name: &HeaderName) -> bool {
    is_hop_by_hop_header(name)
}

#[cfg(windows)]
fn is_hop_by_hop_header(name: &HeaderName) -> bool {
    matches!(
        name.as_str().to_ascii_lowercase().as_str(),
        "connection"
            | "keep-alive"
            | "proxy-authenticate"
            | "proxy-authorization"
            | "te"
            | "trailer"
            | "transfer-encoding"
            | "upgrade"
            | "proxy-connection"
    )
}

#[cfg_attr(windows, allow(dead_code))]
#[derive(Debug, Clone)]
struct RuntimeHostProxy {
    config: RuntimeHostConfig,
}

#[cfg_attr(windows, allow(dead_code))]
#[derive(Debug, Clone, PartialEq, Eq)]
struct RuntimeRequestContext {
    route: RuntimeRoute,
}

#[async_trait]
impl ProxyHttp for RuntimeHostProxy {
    type CTX = RuntimeRequestContext;

    fn new_ctx(&self) -> Self::CTX {
        RuntimeRequestContext {
            route: RuntimeRoute::NotFound,
        }
    }

    async fn request_filter(
        &self,
        session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> PingoraResult<bool> {
        let request_path = session.req_header().uri.path().to_owned();
        let route = resolve_runtime_route(&self.config, &request_path);
        ctx.route = route.clone();

        if matches!(route, RuntimeRoute::Proxy { .. })
            && session
                .req_header()
                .method
                .as_str()
                .eq_ignore_ascii_case("OPTIONS")
        {
            let request_origin = request_origin_pingora(session);
            respond_browser_cors_preflight(session, &self.config, request_origin.as_deref())
                .await?;
            return Ok(true);
        }

        match route {
            RuntimeRoute::Redirect(location) => {
                write_redirect_response(session, &location).await?;
                Ok(true)
            }
            RuntimeRoute::Static { site, request_path } => {
                let site_root = match site {
                    RuntimeSite::Admin => &self.config.admin_site_dir,
                    RuntimeSite::Portal => &self.config.portal_site_dir,
                };

                if serve_static_asset(session, site, &request_path, site_root)
                    .await
                    .is_ok()
                {
                    return Ok(true);
                }

                session.respond_error(404).await?;
                Ok(true)
            }
            RuntimeRoute::Proxy { request_path, .. } => {
                rewrite_request_path(session, &request_path)?;
                Ok(false)
            }
            RuntimeRoute::NotFound => {
                session.respond_error(404).await?;
                Ok(true)
            }
        }
    }

    async fn upstream_peer(
        &self,
        _session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> PingoraResult<Box<HttpPeer>> {
        let target = match &ctx.route {
            RuntimeRoute::Proxy { upstream, .. } => upstream_target(&self.config, upstream),
            _ => upstream_target(&self.config, "portal"),
        };

        Ok(Box::new(HttpPeer::new(target, false, String::new())))
    }

    async fn response_filter(
        &self,
        session: &mut Session,
        upstream_response: &mut ResponseHeader,
        ctx: &mut Self::CTX,
    ) -> PingoraResult<()> {
        if matches!(ctx.route, RuntimeRoute::Proxy { .. }) {
            let request_origin = request_origin_pingora(session);
            apply_browser_cors_headers(upstream_response, &self.config, request_origin.as_deref())?;
        }

        Ok(())
    }
}

#[cfg_attr(windows, allow(dead_code))]
fn apply_browser_cors_headers(
    header: &mut ResponseHeader,
    config: &RuntimeHostConfig,
    request_origin: Option<&str>,
) -> PingoraResult<()> {
    let Some(allow_origin) = resolve_browser_cors_allow_origin(config, request_origin) else {
        return Ok(());
    };

    header.insert_header("access-control-allow-origin", allow_origin)?;
    header.insert_header("access-control-allow-methods", BROWSER_CORS_ALLOW_METHODS)?;
    header.insert_header("access-control-allow-headers", BROWSER_CORS_ALLOW_HEADERS)?;
    header.insert_header("access-control-expose-headers", BROWSER_CORS_EXPOSE_HEADERS)?;
    header.insert_header("access-control-max-age", BROWSER_CORS_MAX_AGE)?;
    header.insert_header("vary", "origin")?;
    Ok(())
}

#[cfg_attr(windows, allow(dead_code))]
async fn respond_browser_cors_preflight(
    session: &mut Session,
    config: &RuntimeHostConfig,
    request_origin: Option<&str>,
) -> PingoraResult<()> {
    let mut header = ResponseHeader::build(204, Some(0))?;
    apply_browser_cors_headers(&mut header, config, request_origin)?;
    session.write_response_header(Box::new(header), true).await
}

#[cfg_attr(windows, allow(dead_code))]
async fn write_redirect_response(session: &mut Session, location: &str) -> PingoraResult<()> {
    let header = build_redirect_response_header(location)?;
    session
        .write_response_header(Box::new(header), false)
        .await?;
    session.write_response_body(Some(Bytes::new()), true).await
}

#[cfg_attr(windows, allow(dead_code))]
fn build_redirect_response_header(location: &str) -> PingoraResult<ResponseHeader> {
    let mut header = ResponseHeader::build(302, Some(4))?;
    header.insert_header("location", location)?;
    header.insert_header("cache-control", "no-cache")?;
    header.insert_header("content-length", "0")?;
    Ok(header)
}

#[cfg_attr(windows, allow(dead_code))]
async fn serve_static_asset(
    session: &mut Session,
    site: RuntimeSite,
    request_path: &str,
    site_root: &Path,
) -> Result<()> {
    let method = session.req_header().method.as_str();
    if method != "GET" && method != "HEAD" {
        bail!("static site only allows GET and HEAD");
    }

    let asset = resolve_static_asset(site, request_path, site_root)?;
    if !asset.filesystem_path.is_file() {
        bail!("static asset does not exist");
    }

    let body = read_static_asset_body(&asset)?;
    let mut header = ResponseHeader::build(200, Some(4))?;
    header.insert_header("content-type", asset.content_type.as_str())?;
    header.insert_header("cache-control", asset.cache_control.as_str())?;
    let content_length = body.len().to_string();
    header.insert_header("content-length", content_length.as_str())?;

    let head_only = method == "HEAD";
    session
        .write_response_header(Box::new(header), head_only)
        .await
        .map_err(|error| anyhow!(error.to_string()))?;
    if !head_only {
        session
            .write_response_body(Some(Bytes::from(body)), true)
            .await
            .map_err(|error| anyhow!(error.to_string()))?;
    }

    Ok(())
}

fn read_static_asset_body(asset: &SiteAsset) -> Result<Vec<u8>> {
    fs::read(&asset.filesystem_path)
        .with_context(|| format!("failed to read {}", asset.filesystem_path.display()))
}

#[cfg_attr(windows, allow(dead_code))]
fn rewrite_request_path(session: &mut Session, request_path: &str) -> PingoraResult<()> {
    let current_uri = session.req_header().uri.clone();
    let mut parts = current_uri.into_parts();
    let new_path = if let Some(query) = session.req_header().uri.query() {
        format!("{request_path}?{query}")
    } else {
        request_path.to_owned()
    };
    parts.path_and_query = Some(new_path.parse::<PathAndQuery>().map_err(|error| {
        Error::because(ErrorType::InternalError, "invalid rewritten path", error)
    })?);
    let rewritten_uri = Uri::from_parts(parts).map_err(|error| {
        Error::because(ErrorType::InternalError, "invalid rewritten uri", error)
    })?;
    session.req_header_mut().set_uri(rewritten_uri);
    Ok(())
}

fn reserve_bind_addr(bind_addr: &str) -> Result<String> {
    let listener = TcpListener::bind(bind_addr)?;
    let bind_addr = listener.local_addr()?;
    Ok(bind_addr.to_string())
}

fn uses_ephemeral_port(bind_addr: &str) -> bool {
    bind_addr
        .parse::<SocketAddr>()
        .map(|address| address.port() == 0)
        .unwrap_or_else(|_| bind_addr.ends_with(":0"))
}

fn listener_probe_addr(bind_addr: &str) -> String {
    bind_addr
        .parse::<SocketAddr>()
        .map(|socket_addr| match socket_addr.ip() {
            IpAddr::V4(ipv4_addr) if ipv4_addr.is_unspecified() => {
                SocketAddr::from((Ipv4Addr::LOCALHOST, socket_addr.port())).to_string()
            }
            IpAddr::V6(ipv6_addr) if ipv6_addr.is_unspecified() => {
                SocketAddr::from((Ipv6Addr::LOCALHOST, socket_addr.port())).to_string()
            }
            _ => socket_addr.to_string(),
        })
        .unwrap_or_else(|_| bind_addr.to_owned())
}

fn wait_for_listener(bind_addr: &str) -> Result<()> {
    let probe_addr = listener_probe_addr(bind_addr);
    for _ in 0..40 {
        if TcpStream::connect(&probe_addr).is_ok() {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(50));
    }

    bail!("runtime host did not bind to {bind_addr}");
}

#[cfg(test)]
mod tests {
    use super::{
        apply_browser_cors_headers, build_redirect_response_header, listener_probe_addr,
        resolve_runtime_route, selected_runtime_backend, RuntimeHostBackend, RuntimeHostConfig,
        RuntimeRoute, RuntimeSite, BROWSER_CORS_ALLOW_HEADERS, BROWSER_CORS_ALLOW_METHODS,
        BROWSER_CORS_EXPOSE_HEADERS, BROWSER_CORS_MAX_AGE,
    };
    use pingora_http::ResponseHeader;

    #[test]
    fn redirect_headers_set_zero_content_length() {
        let header = build_redirect_response_header("/portal/").unwrap();

        assert_eq!(header.status.as_u16(), 302);
        assert_eq!(header.headers.get("location").unwrap(), "/portal/");
        assert_eq!(header.headers.get("content-length").unwrap(), "0");
        assert_eq!(header.headers.get("cache-control").unwrap(), "no-cache");
    }

    #[test]
    fn browser_cors_headers_echo_allowed_request_origin() {
        let mut header = ResponseHeader::build(200, Some(0)).unwrap();
        let config = RuntimeHostConfig::local_defaults("127.0.0.1:9983")
            .with_browser_allowed_origins(["https://console.example.com"]);
        apply_browser_cors_headers(&mut header, &config, Some("https://console.example.com"))
            .unwrap();

        assert_eq!(
            header.headers.get("access-control-allow-origin").unwrap(),
            "https://console.example.com"
        );
        assert_eq!(
            header.headers.get("access-control-allow-methods").unwrap(),
            BROWSER_CORS_ALLOW_METHODS
        );
        assert_eq!(
            header.headers.get("access-control-allow-headers").unwrap(),
            BROWSER_CORS_ALLOW_HEADERS
        );
        assert_eq!(
            header.headers.get("access-control-expose-headers").unwrap(),
            BROWSER_CORS_EXPOSE_HEADERS
        );
        assert_eq!(
            header.headers.get("access-control-max-age").unwrap(),
            BROWSER_CORS_MAX_AGE
        );
    }

    #[test]
    fn browser_cors_headers_skip_disallowed_request_origin() {
        let mut header = ResponseHeader::build(200, Some(0)).unwrap();
        let config = RuntimeHostConfig::local_defaults("127.0.0.1:9983")
            .with_browser_allowed_origins(["https://console.example.com"]);

        apply_browser_cors_headers(&mut header, &config, Some("https://evil.example.com")).unwrap();

        assert!(header.headers.get("access-control-allow-origin").is_none());
    }

    #[test]
    fn listener_probe_addr_rewrites_unspecified_ipv4_to_loopback() {
        assert_eq!(listener_probe_addr("0.0.0.0:9983"), "127.0.0.1:9983");
    }

    #[test]
    fn listener_probe_addr_rewrites_unspecified_ipv6_to_loopback() {
        assert_eq!(listener_probe_addr("[::]:9983"), "[::1]:9983");
    }

    #[test]
    fn listener_probe_addr_preserves_specific_bind_addresses() {
        assert_eq!(listener_probe_addr("127.0.0.1:9983"), "127.0.0.1:9983");
    }

    #[test]
    fn runtime_backend_matches_supported_platform_strategy() {
        if cfg!(windows) {
            assert_eq!(selected_runtime_backend(), RuntimeHostBackend::Axum);
        } else {
            assert_eq!(selected_runtime_backend(), RuntimeHostBackend::Pingora);
        }
    }

    #[test]
    fn resolve_runtime_route_keeps_static_site_requests_when_no_site_proxy_is_configured() {
        let config = RuntimeHostConfig::local_defaults("127.0.0.1:9983");

        assert_eq!(
            resolve_runtime_route(&config, "/admin/users"),
            RuntimeRoute::Static {
                site: RuntimeSite::Admin,
                request_path: "/admin/users".to_owned(),
            }
        );
        assert_eq!(
            resolve_runtime_route(&config, "/portal/home"),
            RuntimeRoute::Static {
                site: RuntimeSite::Portal,
                request_path: "/portal/home".to_owned(),
            }
        );
    }

    #[test]
    fn resolve_runtime_route_can_proxy_admin_and_portal_pages_to_dev_servers() {
        let mut config = RuntimeHostConfig::local_defaults("127.0.0.1:9983");
        config.admin_site_proxy_upstream = Some("127.0.0.1:5173".to_owned());
        config.portal_site_proxy_upstream = Some("127.0.0.1:5174".to_owned());

        assert_eq!(
            resolve_runtime_route(&config, "/admin/users"),
            RuntimeRoute::Proxy {
                upstream: "admin-site".to_owned(),
                request_path: "/admin/users".to_owned(),
            }
        );
        assert_eq!(
            resolve_runtime_route(&config, "/portal/home"),
            RuntimeRoute::Proxy {
                upstream: "portal-site".to_owned(),
                request_path: "/portal/home".to_owned(),
            }
        );
    }

    #[test]
    fn local_defaults_start_without_browser_cors_origins() {
        let config = RuntimeHostConfig::local_defaults("127.0.0.1:9983");
        assert!(config.browser_allowed_origins.is_empty());
    }
}

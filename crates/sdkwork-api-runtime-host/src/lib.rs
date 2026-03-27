use std::{
    fs,
    net::{SocketAddr, TcpListener, TcpStream},
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

use anyhow::{anyhow, bail, Context, Result};
use async_trait::async_trait;
use bytes::Bytes;
use http::{uri::PathAndQuery, Uri};
use pingora_core::{
    server::Server, upstreams::peer::HttpPeer, Error, ErrorType, Result as PingoraResult,
};
use pingora_http::ResponseHeader;
use pingora_proxy::{http_proxy_service, ProxyHttp, Session};

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
        }
    }

    pub fn local_defaults(bind_addr: impl Into<String>) -> Self {
        Self::new(
            bind_addr,
            PathBuf::from("apps/sdkwork-router-admin/dist"),
            PathBuf::from("apps/sdkwork-router-portal/dist"),
            "127.0.0.1:8081",
            "127.0.0.1:8082",
            "127.0.0.1:8080",
        )
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

pub fn serve_public_web(config: RuntimeHostConfig) -> Result<()> {
    let mut server = Server::new(None).map_err(|error| anyhow!(error.to_string()))?;
    server.bootstrap();

    let bind_addr = config.bind_addr.clone();
    let mut service = http_proxy_service(&server.configuration, RuntimeHostProxy { config });
    service.add_tcp(bind_addr.as_str());
    server.add_service(service);
    server.run_forever();
}

#[derive(Debug, Clone)]
struct RuntimeHostProxy {
    config: RuntimeHostConfig,
}

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
        let route = classify_request(&request_path);
        ctx.route = route.clone();

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
            RuntimeRoute::Proxy { upstream, .. } if upstream == "admin" => {
                self.config.admin_upstream.as_str()
            }
            RuntimeRoute::Proxy { upstream, .. } if upstream == "portal" => {
                self.config.portal_upstream.as_str()
            }
            RuntimeRoute::Proxy { upstream, .. } if upstream == "gateway" => {
                self.config.gateway_upstream.as_str()
            }
            _ => self.config.portal_upstream.as_str(),
        };

        Ok(Box::new(HttpPeer::new(target, false, String::new())))
    }
}

async fn write_redirect_response(session: &mut Session, location: &str) -> PingoraResult<()> {
    let header = build_redirect_response_header(location)?;
    session
        .write_response_header(Box::new(header), false)
        .await?;
    session.write_response_body(Some(Bytes::new()), true).await
}

fn build_redirect_response_header(location: &str) -> PingoraResult<ResponseHeader> {
    let mut header = ResponseHeader::build(302, Some(4))?;
    header.insert_header("location", location)?;
    header.insert_header("cache-control", "no-cache")?;
    header.insert_header("content-length", "0")?;
    Ok(header)
}

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

    let body = fs::read(&asset.filesystem_path)
        .with_context(|| format!("failed to read {}", asset.filesystem_path.display()))?;
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

fn wait_for_listener(bind_addr: &str) -> Result<()> {
    for _ in 0..40 {
        if TcpStream::connect(bind_addr).is_ok() {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(50));
    }

    bail!("runtime host did not bind to {bind_addr}");
}

#[cfg(test)]
mod tests {
    use super::build_redirect_response_header;

    #[test]
    fn redirect_headers_set_zero_content_length() {
        let header = build_redirect_response_header("/portal/").unwrap();

        assert_eq!(header.status.as_u16(), 302);
        assert_eq!(header.headers.get("location").unwrap(), "/portal/");
        assert_eq!(header.headers.get("content-length").unwrap(), "0");
        assert_eq!(header.headers.get("cache-control").unwrap(), "no-cache");
    }
}

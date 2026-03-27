use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use sdkwork_api_app_credential::CredentialSecretManager;
use sdkwork_api_app_runtime::{
    build_admin_store_from_config, resolve_service_runtime_node_id,
    start_extension_runtime_rollout_supervision, start_standalone_runtime_supervision,
    StandaloneListenerHost, StandaloneRuntimeSupervision, StandaloneServiceKind,
    StandaloneServiceReloadHandles,
};
use sdkwork_api_config::{RuntimeMode, StandaloneConfig, StandaloneConfigLoader};
use sdkwork_api_interface_admin::{admin_router_with_state, AdminApiState};
use sdkwork_api_interface_http::{gateway_router_with_state, GatewayApiState};
use sdkwork_api_interface_portal::{portal_router_with_state, PortalApiState};
use sdkwork_api_runtime_host::{EmbeddedRuntime, RuntimeHostConfig};
use sdkwork_api_storage_core::Reloadable;
use tokio::task::JoinHandle;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductSiteDirs {
    pub admin_site_dir: PathBuf,
    pub portal_site_dir: PathBuf,
}

impl ProductSiteDirs {
    pub fn new(
        admin_site_dir: impl Into<PathBuf>,
        portal_site_dir: impl Into<PathBuf>,
    ) -> Self {
        Self {
            admin_site_dir: admin_site_dir.into(),
            portal_site_dir: portal_site_dir.into(),
        }
    }

    pub fn from_workspace_root(workspace_root: impl AsRef<Path>) -> Self {
        let workspace_root = workspace_root.as_ref();
        Self::new(
            workspace_root
                .join("apps")
                .join("sdkwork-router-admin")
                .join("dist"),
            workspace_root
                .join("apps")
                .join("sdkwork-router-portal")
                .join("dist"),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProductRuntimeRole {
    Web,
    Gateway,
    Admin,
    Portal,
}

impl ProductRuntimeRole {
    fn all() -> BTreeSet<Self> {
        [
            Self::Web,
            Self::Gateway,
            Self::Admin,
            Self::Portal,
        ]
        .into_iter()
        .collect()
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "web" => Ok(Self::Web),
            "gateway" => Ok(Self::Gateway),
            "admin" => Ok(Self::Admin),
            "portal" => Ok(Self::Portal),
            other => bail!("unsupported product runtime role: {other}"),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Web => "web",
            Self::Gateway => "gateway",
            Self::Admin => "admin",
            Self::Portal => "portal",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouterProductRuntimeOptions {
    mode: RuntimeMode,
    roles: BTreeSet<ProductRuntimeRole>,
    public_web_bind: String,
    site_dirs: ProductSiteDirs,
    admin_upstream: Option<String>,
    portal_upstream: Option<String>,
    gateway_upstream: Option<String>,
    node_id_prefix: Option<String>,
}

impl RouterProductRuntimeOptions {
    pub fn desktop(site_dirs: ProductSiteDirs) -> Self {
        Self {
            mode: RuntimeMode::Embedded,
            roles: ProductRuntimeRole::all(),
            public_web_bind: "127.0.0.1:0".to_owned(),
            site_dirs,
            admin_upstream: None,
            portal_upstream: None,
            gateway_upstream: None,
            node_id_prefix: None,
        }
    }

    pub fn server(site_dirs: ProductSiteDirs) -> Self {
        Self {
            mode: RuntimeMode::Server,
            roles: ProductRuntimeRole::all(),
            public_web_bind: "0.0.0.0:3001".to_owned(),
            site_dirs,
            admin_upstream: None,
            portal_upstream: None,
            gateway_upstream: None,
            node_id_prefix: None,
        }
    }

    pub fn with_roles<I>(mut self, roles: I) -> Self
    where
        I: IntoIterator<Item = ProductRuntimeRole>,
    {
        self.roles = roles.into_iter().collect();
        self
    }

    pub fn with_public_web_bind(mut self, bind: impl Into<String>) -> Self {
        self.public_web_bind = bind.into();
        self
    }

    pub fn with_admin_upstream(mut self, upstream: impl Into<String>) -> Self {
        self.admin_upstream = Some(upstream.into());
        self
    }

    pub fn with_portal_upstream(mut self, upstream: impl Into<String>) -> Self {
        self.portal_upstream = Some(upstream.into());
        self
    }

    pub fn with_gateway_upstream(mut self, upstream: impl Into<String>) -> Self {
        self.gateway_upstream = Some(upstream.into());
        self
    }

    pub fn with_node_id_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.node_id_prefix = Some(prefix.into());
        self
    }
}

pub struct RouterProductRuntime {
    public_base_url: Option<String>,
    public_bind_addr: Option<String>,
    gateway_bind_addr: Option<String>,
    admin_bind_addr: Option<String>,
    portal_bind_addr: Option<String>,
    _web_runtime: Option<EmbeddedRuntime>,
    _gateway_listener: Option<StandaloneListenerHost>,
    _admin_listener: Option<StandaloneListenerHost>,
    _portal_listener: Option<StandaloneListenerHost>,
    _gateway_runtime_rollout: Option<AbortOnDropJoinHandle>,
    _admin_runtime_rollout: Option<AbortOnDropJoinHandle>,
    _gateway_runtime_supervision: Option<StandaloneRuntimeSupervision>,
    _admin_runtime_supervision: Option<StandaloneRuntimeSupervision>,
    _portal_runtime_supervision: Option<StandaloneRuntimeSupervision>,
}

impl RouterProductRuntime {
    pub async fn start(
        loader: StandaloneConfigLoader,
        config: StandaloneConfig,
        options: RouterProductRuntimeOptions,
    ) -> Result<Self> {
        let live_store = Reloadable::new(build_admin_store_from_config(&config).await?);
        let live_secret_manager =
            Reloadable::new(CredentialSecretManager::new_with_legacy_master_keys(
                config.secret_backend,
                config.credential_master_key.clone(),
                config.credential_legacy_master_keys.clone(),
                config.secret_local_file.clone(),
                config.secret_keyring_service.clone(),
            ));
        let live_admin_jwt = Reloadable::new(config.admin_jwt_signing_secret.clone());
        let live_portal_jwt = Reloadable::new(config.portal_jwt_signing_secret.clone());

        let gateway_listener = if options.roles.contains(&ProductRuntimeRole::Gateway) {
            Some(
                StandaloneListenerHost::bind(
                    requested_local_bind(&config.gateway_bind, options.mode),
                    gateway_router_with_state(
                        GatewayApiState::with_live_store_and_secret_manager_handle(
                            live_store.clone(),
                            live_secret_manager.clone(),
                        ),
                    ),
                )
                .await?,
            )
        } else {
            None
        };

        let admin_listener = if options.roles.contains(&ProductRuntimeRole::Admin) {
            Some(
                StandaloneListenerHost::bind(
                    requested_local_bind(&config.admin_bind, options.mode),
                    admin_router_with_state(
                        AdminApiState::with_live_store_and_secret_manager_handle_and_jwt_secret_handle(
                            live_store.clone(),
                            live_secret_manager.clone(),
                            live_admin_jwt.clone(),
                        ),
                    ),
                )
                .await?,
            )
        } else {
            None
        };

        let portal_listener = if options.roles.contains(&ProductRuntimeRole::Portal) {
            Some(
                StandaloneListenerHost::bind(
                    requested_local_bind(&config.portal_bind, options.mode),
                    portal_router_with_state(PortalApiState::with_live_store_and_jwt_secret_handle(
                        live_store.clone(),
                        live_portal_jwt.clone(),
                    )),
                )
                .await?,
            )
        } else {
            None
        };

        let gateway_bind_addr = listener_bind(gateway_listener.as_ref(), ProductRuntimeRole::Gateway)?;
        let admin_bind_addr = listener_bind(admin_listener.as_ref(), ProductRuntimeRole::Admin)?;
        let portal_bind_addr = listener_bind(portal_listener.as_ref(), ProductRuntimeRole::Portal)?;

        let bind_overrides = build_bind_overrides(
            gateway_bind_addr.as_deref(),
            admin_bind_addr.as_deref(),
            portal_bind_addr.as_deref(),
        );
        let (runtime_loader, effective_config) = loader.with_overrides(bind_overrides)?;
        effective_config.apply_to_process_env();

        let gateway_node_id = options
            .roles
            .contains(&ProductRuntimeRole::Gateway)
            .then(|| node_id_for(&options, StandaloneServiceKind::Gateway));
        let admin_node_id = options
            .roles
            .contains(&ProductRuntimeRole::Admin)
            .then(|| node_id_for(&options, StandaloneServiceKind::Admin));
        let portal_node_id = options
            .roles
            .contains(&ProductRuntimeRole::Portal)
            .then(|| node_id_for(&options, StandaloneServiceKind::Portal));

        let gateway_runtime_rollout =
            if let Some(node_id) = gateway_node_id.as_deref() {
                Some(AbortOnDropJoinHandle::new(
                    start_extension_runtime_rollout_supervision(
                        StandaloneServiceKind::Gateway,
                        node_id,
                        live_store.clone(),
                    )?,
                ))
            } else {
                None
            };
        let admin_runtime_rollout = if let Some(node_id) = admin_node_id.as_deref() {
            Some(AbortOnDropJoinHandle::new(
                start_extension_runtime_rollout_supervision(
                    StandaloneServiceKind::Admin,
                    node_id,
                    live_store.clone(),
                )?,
            ))
        } else {
            None
        };

        let gateway_runtime_supervision =
            if let Some(listener_host) = gateway_listener.as_ref() {
                Some(start_standalone_runtime_supervision(
                    StandaloneServiceKind::Gateway,
                    runtime_loader.clone(),
                    effective_config.clone(),
                    StandaloneServiceReloadHandles::gateway(live_store.clone())
                        .with_secret_manager(live_secret_manager.clone())
                        .with_listener(listener_host.reload_handle())
                        .with_node_id(
                            gateway_node_id
                                .as_deref()
                                .context("gateway node id missing")?,
                        ),
                ))
            } else {
                None
            };
        let admin_runtime_supervision = if let Some(listener_host) = admin_listener.as_ref() {
            Some(start_standalone_runtime_supervision(
                StandaloneServiceKind::Admin,
                runtime_loader.clone(),
                effective_config.clone(),
                StandaloneServiceReloadHandles::admin(live_store.clone(), live_admin_jwt)
                    .with_secret_manager(live_secret_manager.clone())
                    .with_listener(listener_host.reload_handle())
                    .with_node_id(admin_node_id.as_deref().context("admin node id missing")?),
            ))
        } else {
            None
        };
        let portal_runtime_supervision = if let Some(listener_host) = portal_listener.as_ref() {
            Some(start_standalone_runtime_supervision(
                StandaloneServiceKind::Portal,
                runtime_loader,
                effective_config,
                StandaloneServiceReloadHandles::portal(live_store, live_portal_jwt)
                    .with_listener(listener_host.reload_handle())
                    .with_node_id(portal_node_id.as_deref().context("portal node id missing")?),
            ))
        } else {
            None
        };

        let (web_runtime, public_base_url, public_bind_addr) =
            if options.roles.contains(&ProductRuntimeRole::Web) {
                validate_site_dir(
                    &options.site_dirs.admin_site_dir,
                    ProductRuntimeRole::Admin,
                )?;
                validate_site_dir(
                    &options.site_dirs.portal_site_dir,
                    ProductRuntimeRole::Portal,
                )?;
                ensure_required_web_upstreams(&options)?;

                let runtime = EmbeddedRuntime::start(RuntimeHostConfig::new(
                    options.public_web_bind,
                    options.site_dirs.admin_site_dir,
                    options.site_dirs.portal_site_dir,
                    resolve_upstream(
                        "admin",
                        options.roles.contains(&ProductRuntimeRole::Admin),
                        admin_bind_addr.as_deref(),
                        options.admin_upstream.as_deref(),
                    )?,
                    resolve_upstream(
                        "portal",
                        options.roles.contains(&ProductRuntimeRole::Portal),
                        portal_bind_addr.as_deref(),
                        options.portal_upstream.as_deref(),
                    )?,
                    resolve_upstream(
                        "gateway",
                        options.roles.contains(&ProductRuntimeRole::Gateway),
                        gateway_bind_addr.as_deref(),
                        options.gateway_upstream.as_deref(),
                    )?,
                ))
                .await?;
                let base_url = runtime.base_url().to_owned();
                let bind_addr = base_url.trim_start_matches("http://").to_owned();
                (Some(runtime), Some(base_url), Some(bind_addr))
            } else {
                (None, None, None)
            };

        Ok(Self {
            public_base_url,
            public_bind_addr,
            gateway_bind_addr,
            admin_bind_addr,
            portal_bind_addr,
            _web_runtime: web_runtime,
            _gateway_listener: gateway_listener,
            _admin_listener: admin_listener,
            _portal_listener: portal_listener,
            _gateway_runtime_rollout: gateway_runtime_rollout,
            _admin_runtime_rollout: admin_runtime_rollout,
            _gateway_runtime_supervision: gateway_runtime_supervision,
            _admin_runtime_supervision: admin_runtime_supervision,
            _portal_runtime_supervision: portal_runtime_supervision,
        })
    }

    pub fn public_base_url(&self) -> Option<&str> {
        self.public_base_url.as_deref()
    }

    pub fn public_bind_addr(&self) -> Option<&str> {
        self.public_bind_addr.as_deref()
    }

    pub fn gateway_bind_addr(&self) -> Option<&str> {
        self.gateway_bind_addr.as_deref()
    }

    pub fn admin_bind_addr(&self) -> Option<&str> {
        self.admin_bind_addr.as_deref()
    }

    pub fn portal_bind_addr(&self) -> Option<&str> {
        self.portal_bind_addr.as_deref()
    }
}

fn requested_local_bind(configured_bind: &str, mode: RuntimeMode) -> String {
    match mode {
        RuntimeMode::Server => configured_bind.to_owned(),
        RuntimeMode::Embedded => "127.0.0.1:0".to_owned(),
    }
}

fn listener_bind(
    listener_host: Option<&StandaloneListenerHost>,
    role: ProductRuntimeRole,
) -> Result<Option<String>> {
    listener_host
        .map(|listener_host| {
            listener_host.current_bind().with_context(|| {
                format!("{} listener started without a resolved bind", role.as_str())
            })
        })
        .transpose()
}

fn build_bind_overrides(
    gateway_bind: Option<&str>,
    admin_bind: Option<&str>,
    portal_bind: Option<&str>,
) -> Vec<(String, String)> {
    let mut overrides = Vec::new();
    if let Some(gateway_bind) = gateway_bind {
        overrides.push(("SDKWORK_GATEWAY_BIND".to_owned(), gateway_bind.to_owned()));
    }
    if let Some(admin_bind) = admin_bind {
        overrides.push(("SDKWORK_ADMIN_BIND".to_owned(), admin_bind.to_owned()));
    }
    if let Some(portal_bind) = portal_bind {
        overrides.push(("SDKWORK_PORTAL_BIND".to_owned(), portal_bind.to_owned()));
    }
    overrides
}

fn node_id_for(options: &RouterProductRuntimeOptions, service_kind: StandaloneServiceKind) -> String {
    match options.node_id_prefix.as_deref() {
        Some(prefix) => format!("{prefix}-{}", service_kind_label(service_kind)),
        None => resolve_service_runtime_node_id(service_kind),
    }
}

fn service_kind_label(service_kind: StandaloneServiceKind) -> &'static str {
    match service_kind {
        StandaloneServiceKind::Gateway => "gateway",
        StandaloneServiceKind::Admin => "admin",
        StandaloneServiceKind::Portal => "portal",
    }
}

fn resolve_upstream(
    label: &str,
    has_local_role: bool,
    local_bind: Option<&str>,
    external_upstream: Option<&str>,
) -> Result<String> {
    if has_local_role {
        return local_bind
            .map(str::to_owned)
            .with_context(|| format!("{label} upstream is missing a resolved local bind"));
    }

    external_upstream
        .map(str::to_owned)
        .with_context(|| format!("{label} upstream must be configured when web role is enabled without a local {label} role"))
}

fn ensure_required_web_upstreams(options: &RouterProductRuntimeOptions) -> Result<()> {
    let mut missing = Vec::new();
    for (label, role, upstream) in [
        (
            "gateway",
            ProductRuntimeRole::Gateway,
            options.gateway_upstream.as_deref(),
        ),
        (
            "admin",
            ProductRuntimeRole::Admin,
            options.admin_upstream.as_deref(),
        ),
        (
            "portal",
            ProductRuntimeRole::Portal,
            options.portal_upstream.as_deref(),
        ),
    ] {
        if !options.roles.contains(&role) && upstream.is_none() {
            missing.push(format!("{label} upstream"));
        }
    }

    if missing.is_empty() {
        return Ok(());
    }

    bail!(
        "{} must be configured when web role is enabled without matching local roles",
        missing.join(", ")
    )
}

fn validate_site_dir(path: &Path, role: ProductRuntimeRole) -> Result<()> {
    if !path.is_dir() {
        bail!(
            "{} site directory does not exist: {}",
            role.as_str(),
            path.display()
        );
    }
    Ok(())
}

struct AbortOnDropJoinHandle(Option<JoinHandle<()>>);

impl AbortOnDropJoinHandle {
    fn new(handle: JoinHandle<()>) -> Self {
        Self(Some(handle))
    }
}

impl Drop for AbortOnDropJoinHandle {
    fn drop(&mut self) {
        if let Some(handle) = self.0.take() {
            handle.abort();
        }
    }
}

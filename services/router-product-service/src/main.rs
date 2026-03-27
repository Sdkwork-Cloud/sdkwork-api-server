use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};

use clap::{Parser, ValueEnum};
use serde_json::json;
use sdkwork_api_config::{StandaloneConfig, StandaloneConfigLoader};
use sdkwork_api_observability::init_tracing;
use sdkwork_api_product_runtime::{
    ProductRuntimeRole, ProductSiteDirs, RouterProductRuntime, RouterProductRuntimeOptions,
};

const SDKWORK_CONFIG_DIR: &str = "SDKWORK_CONFIG_DIR";
const SDKWORK_CONFIG_FILE: &str = "SDKWORK_CONFIG_FILE";
const SDKWORK_DATABASE_URL: &str = "SDKWORK_DATABASE_URL";
const SDKWORK_WEB_BIND: &str = "SDKWORK_WEB_BIND";
const SDKWORK_ROUTER_ROLES: &str = "SDKWORK_ROUTER_ROLES";
const SDKWORK_ROUTER_NODE_ID_PREFIX: &str = "SDKWORK_ROUTER_NODE_ID_PREFIX";
const SDKWORK_GATEWAY_BIND: &str = "SDKWORK_GATEWAY_BIND";
const SDKWORK_ADMIN_BIND: &str = "SDKWORK_ADMIN_BIND";
const SDKWORK_PORTAL_BIND: &str = "SDKWORK_PORTAL_BIND";
const SDKWORK_ADMIN_PROXY_TARGET: &str = "SDKWORK_ADMIN_PROXY_TARGET";
const SDKWORK_PORTAL_PROXY_TARGET: &str = "SDKWORK_PORTAL_PROXY_TARGET";
const SDKWORK_GATEWAY_PROXY_TARGET: &str = "SDKWORK_GATEWAY_PROXY_TARGET";
const SDKWORK_ADMIN_SITE_DIR: &str = "SDKWORK_ADMIN_SITE_DIR";
const SDKWORK_PORTAL_SITE_DIR: &str = "SDKWORK_PORTAL_SITE_DIR";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum)]
enum PlanFormat {
    #[default]
    Text,
    Json,
}

impl PlanFormat {
    fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Json => "json",
        }
    }
}

#[derive(Debug, Clone, Parser, PartialEq, Eq)]
#[command(
    name = "router-product-service",
    about = "Start the integrated SDKWork Router product host in server mode."
)]
struct RouterProductServiceCli {
    #[arg(long = "config-dir", value_name = "DIR")]
    config_dir: Option<String>,
    #[arg(long = "config-file", value_name = "FILE")]
    config_file: Option<String>,
    #[arg(long = "database-url", value_name = "URL")]
    database_url: Option<String>,
    #[arg(long = "bind", value_name = "HOST:PORT")]
    public_web_bind: Option<String>,
    #[arg(long = "roles", value_name = "ROLES")]
    roles: Option<String>,
    #[arg(long = "node-id-prefix", value_name = "PREFIX")]
    node_id_prefix: Option<String>,
    #[arg(long = "gateway-bind", value_name = "HOST:PORT")]
    gateway_bind: Option<String>,
    #[arg(long = "admin-bind", value_name = "HOST:PORT")]
    admin_bind: Option<String>,
    #[arg(long = "portal-bind", value_name = "HOST:PORT")]
    portal_bind: Option<String>,
    #[arg(long = "admin-upstream", value_name = "HOST:PORT")]
    admin_upstream: Option<String>,
    #[arg(long = "portal-upstream", value_name = "HOST:PORT")]
    portal_upstream: Option<String>,
    #[arg(long = "gateway-upstream", value_name = "HOST:PORT")]
    gateway_upstream: Option<String>,
    #[arg(long = "admin-site-dir", value_name = "DIR")]
    admin_site_dir: Option<PathBuf>,
    #[arg(long = "portal-site-dir", value_name = "DIR")]
    portal_site_dir: Option<PathBuf>,
    #[arg(long = "plan-format", value_enum, default_value_t = PlanFormat::Text)]
    plan_format: PlanFormat,
    #[arg(long = "dry-run", default_value_t = false)]
    dry_run: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ProductServiceSettings {
    config_dir: Option<String>,
    config_file: Option<String>,
    database_url: Option<String>,
    public_web_bind: Option<String>,
    roles: Option<Vec<ProductRuntimeRole>>,
    node_id_prefix: Option<String>,
    gateway_bind: Option<String>,
    admin_bind: Option<String>,
    portal_bind: Option<String>,
    admin_upstream: Option<String>,
    portal_upstream: Option<String>,
    gateway_upstream: Option<String>,
    admin_site_dir: Option<PathBuf>,
    portal_site_dir: Option<PathBuf>,
    plan_format: PlanFormat,
    dry_run: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = RouterProductServiceCli::parse();
    let settings = resolve_service_settings(&cli, env::vars())?;
    apply_loader_env_overrides(&settings);

    let (loader, config) = StandaloneConfigLoader::from_env()?;

    if settings.dry_run {
        print!("{}", render_service_plan(&settings, &config));
        return Ok(());
    }

    init_tracing("router-product-service");
    let options = build_runtime_options(&settings);
    let runtime = RouterProductRuntime::start(loader, config, options).await?;
    print_runtime_summary(&runtime);

    tokio::signal::ctrl_c().await?;
    drop(runtime);
    Ok(())
}

fn resolve_service_settings<I, K, V>(
    cli: &RouterProductServiceCli,
    env_pairs: I,
) -> anyhow::Result<ProductServiceSettings>
where
    I: IntoIterator<Item = (K, V)>,
    K: Into<String>,
    V: Into<String>,
{
    let env_values = collect_env_values(env_pairs);

    Ok(ProductServiceSettings {
        config_dir: resolve_string_option(cli.config_dir.as_deref(), &env_values, SDKWORK_CONFIG_DIR),
        config_file: resolve_string_option(
            cli.config_file.as_deref(),
            &env_values,
            SDKWORK_CONFIG_FILE,
        ),
        database_url: resolve_string_option(
            cli.database_url.as_deref(),
            &env_values,
            SDKWORK_DATABASE_URL,
        ),
        public_web_bind: resolve_string_option(
            cli.public_web_bind.as_deref(),
            &env_values,
            SDKWORK_WEB_BIND,
        ),
        roles: resolve_string_option(cli.roles.as_deref(), &env_values, SDKWORK_ROUTER_ROLES)
            .map(|value| parse_roles(&value))
            .transpose()?,
        node_id_prefix: resolve_string_option(
            cli.node_id_prefix.as_deref(),
            &env_values,
            SDKWORK_ROUTER_NODE_ID_PREFIX,
        ),
        gateway_bind: resolve_string_option(
            cli.gateway_bind.as_deref(),
            &env_values,
            SDKWORK_GATEWAY_BIND,
        ),
        admin_bind: resolve_string_option(cli.admin_bind.as_deref(), &env_values, SDKWORK_ADMIN_BIND),
        portal_bind: resolve_string_option(
            cli.portal_bind.as_deref(),
            &env_values,
            SDKWORK_PORTAL_BIND,
        ),
        admin_upstream: resolve_string_option(
            cli.admin_upstream.as_deref(),
            &env_values,
            SDKWORK_ADMIN_PROXY_TARGET,
        ),
        portal_upstream: resolve_string_option(
            cli.portal_upstream.as_deref(),
            &env_values,
            SDKWORK_PORTAL_PROXY_TARGET,
        ),
        gateway_upstream: resolve_string_option(
            cli.gateway_upstream.as_deref(),
            &env_values,
            SDKWORK_GATEWAY_PROXY_TARGET,
        ),
        admin_site_dir: resolve_path_option(
            cli.admin_site_dir.as_ref(),
            &env_values,
            SDKWORK_ADMIN_SITE_DIR,
        ),
        portal_site_dir: resolve_path_option(
            cli.portal_site_dir.as_ref(),
            &env_values,
            SDKWORK_PORTAL_SITE_DIR,
        ),
        plan_format: cli.plan_format,
        dry_run: cli.dry_run,
    })
}

fn build_runtime_options(settings: &ProductServiceSettings) -> RouterProductRuntimeOptions {
    let mut options = RouterProductRuntimeOptions::server(resolve_site_dirs(settings));

    if let Some(bind) = settings.public_web_bind.as_deref() {
        options = options.with_public_web_bind(bind);
    }
    if let Some(node_id_prefix) = settings.node_id_prefix.as_deref() {
        options = options.with_node_id_prefix(node_id_prefix);
    }
    if let Some(admin_upstream) = settings.admin_upstream.as_deref() {
        options = options.with_admin_upstream(admin_upstream);
    }
    if let Some(portal_upstream) = settings.portal_upstream.as_deref() {
        options = options.with_portal_upstream(portal_upstream);
    }
    if let Some(gateway_upstream) = settings.gateway_upstream.as_deref() {
        options = options.with_gateway_upstream(gateway_upstream);
    }
    if let Some(roles) = settings.roles.clone() {
        options = options.with_roles(roles);
    }

    options
}

fn apply_loader_env_overrides(settings: &ProductServiceSettings) {
    for (key, value) in [
        (SDKWORK_CONFIG_DIR, settings.config_dir.as_deref()),
        (SDKWORK_CONFIG_FILE, settings.config_file.as_deref()),
        (SDKWORK_DATABASE_URL, settings.database_url.as_deref()),
        (SDKWORK_GATEWAY_BIND, settings.gateway_bind.as_deref()),
        (SDKWORK_ADMIN_BIND, settings.admin_bind.as_deref()),
        (SDKWORK_PORTAL_BIND, settings.portal_bind.as_deref()),
    ] {
        if let Some(value) = value {
            env::set_var(key, value);
        }
    }
}

fn parse_roles(value: &str) -> anyhow::Result<Vec<ProductRuntimeRole>> {
    value
        .split([',', ';'])
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ProductRuntimeRole::parse)
        .collect()
}

fn collect_env_values<I, K, V>(pairs: I) -> HashMap<String, String>
where
    I: IntoIterator<Item = (K, V)>,
    K: Into<String>,
    V: Into<String>,
{
    pairs
        .into_iter()
        .map(|(key, value)| (key.into(), value.into()))
        .collect()
}

fn resolve_string_option(
    cli_value: Option<&str>,
    env_values: &HashMap<String, String>,
    env_key: &str,
) -> Option<String> {
    cli_value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .or_else(|| {
            env_values
                .get(env_key)
                .map(String::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_owned)
        })
}

fn resolve_path_option(
    cli_value: Option<&PathBuf>,
    env_values: &HashMap<String, String>,
    env_key: &str,
) -> Option<PathBuf> {
    cli_value.cloned().or_else(|| {
        env_values
            .get(env_key)
            .map(String::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
    })
}

fn resolve_site_dirs(settings: &ProductServiceSettings) -> ProductSiteDirs {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .and_then(Path::parent)
        .expect("router-product-service must live inside services/");
    let defaults = ProductSiteDirs::from_workspace_root(workspace_root);

    ProductSiteDirs::new(
        settings
            .admin_site_dir
            .clone()
            .unwrap_or(defaults.admin_site_dir),
        settings
            .portal_site_dir
            .clone()
            .unwrap_or(defaults.portal_site_dir),
    )
}

fn render_service_plan(settings: &ProductServiceSettings, config: &StandaloneConfig) -> String {
    match settings.plan_format {
        PlanFormat::Text => render_service_plan_text(settings, config),
        PlanFormat::Json => render_service_plan_json(settings, config),
    }
}

fn render_service_plan_text(settings: &ProductServiceSettings, config: &StandaloneConfig) -> String {
    let site_dirs = resolve_site_dirs(settings);
    let roles = settings
        .roles
        .clone()
        .unwrap_or_else(|| {
            vec![
                ProductRuntimeRole::Web,
                ProductRuntimeRole::Gateway,
                ProductRuntimeRole::Admin,
                ProductRuntimeRole::Portal,
            ]
        })
        .into_iter()
        .map(ProductRuntimeRole::as_str)
        .collect::<Vec<_>>()
        .join(",");

    let mut lines = vec![
        "router-product-service dry run".to_owned(),
        format!("roles={roles}"),
        format!(
            "public_web_bind={}",
            settings
                .public_web_bind
                .as_deref()
                .unwrap_or("0.0.0.0:3001")
        ),
        format!("gateway_bind={}", config.gateway_bind),
        format!("admin_bind={}", config.admin_bind),
        format!("portal_bind={}", config.portal_bind),
        format!("database_url={}", config.database_url),
        format!("admin_site_dir={}", site_dirs.admin_site_dir.display()),
        format!("portal_site_dir={}", site_dirs.portal_site_dir.display()),
    ];

    if let Some(config_dir) = settings.config_dir.as_deref() {
        lines.push(format!("config_dir={config_dir}"));
    }
    if let Some(config_file) = settings.config_file.as_deref() {
        lines.push(format!("config_file={config_file}"));
    }
    if let Some(node_id_prefix) = settings.node_id_prefix.as_deref() {
        lines.push(format!("node_id_prefix={node_id_prefix}"));
    }
    if let Some(gateway_upstream) = settings.gateway_upstream.as_deref() {
        lines.push(format!("gateway_upstream={gateway_upstream}"));
    }
    if let Some(admin_upstream) = settings.admin_upstream.as_deref() {
        lines.push(format!("admin_upstream={admin_upstream}"));
    }
    if let Some(portal_upstream) = settings.portal_upstream.as_deref() {
        lines.push(format!("portal_upstream={portal_upstream}"));
    }

    lines.push(String::new());
    lines.join("\n")
}

fn render_service_plan_json(settings: &ProductServiceSettings, config: &StandaloneConfig) -> String {
    let site_dirs = resolve_site_dirs(settings);
    let roles = settings
        .roles
        .clone()
        .unwrap_or_else(|| {
            vec![
                ProductRuntimeRole::Web,
                ProductRuntimeRole::Gateway,
                ProductRuntimeRole::Admin,
                ProductRuntimeRole::Portal,
            ]
        })
        .into_iter()
        .map(ProductRuntimeRole::as_str)
        .collect::<Vec<_>>();

    serde_json::to_string_pretty(&json!({
        "mode": "dry-run",
        "plan_format": settings.plan_format.as_str(),
        "roles": roles,
        "public_web_bind": settings.public_web_bind.as_deref().unwrap_or("0.0.0.0:3001"),
        "database_url": config.database_url,
        "config_dir": settings.config_dir,
        "config_file": settings.config_file,
        "node_id_prefix": settings.node_id_prefix,
        "binds": {
            "gateway": config.gateway_bind,
            "admin": config.admin_bind,
            "portal": config.portal_bind,
        },
        "site_dirs": {
            "admin": site_dirs.admin_site_dir,
            "portal": site_dirs.portal_site_dir,
        },
        "upstreams": {
            "gateway": settings.gateway_upstream,
            "admin": settings.admin_upstream,
            "portal": settings.portal_upstream,
        }
    }))
    .expect("service plan json serialization should not fail")
}

fn print_runtime_summary(runtime: &RouterProductRuntime) {
    if let Some(bind) = runtime.public_bind_addr() {
        println!("router-product-service public web listening on {bind}");
    }
    if let Some(bind) = runtime.gateway_bind_addr() {
        println!("router-product-service gateway listening on {bind}");
    }
    if let Some(bind) = runtime.admin_bind_addr() {
        println!("router-product-service admin listening on {bind}");
    }
    if let Some(bind) = runtime.portal_bind_addr() {
        println!("router-product-service portal listening on {bind}");
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use clap::Parser;
    use serde_json::Value;

    use super::{
        render_service_plan, resolve_service_settings, PlanFormat, ProductRuntimeRole,
        ProductServiceSettings, RouterProductServiceCli, StandaloneConfig,
    };

    #[test]
    fn resolve_service_settings_parses_full_cli_cluster_overrides() {
        let cli = RouterProductServiceCli::try_parse_from([
            "router-product-service",
            "--config-dir",
            "D:/router/config",
            "--config-file",
            "cluster/router.yaml",
            "--database-url",
            "postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_router",
            "--bind",
            "0.0.0.0:3301",
            "--roles",
            "web,gateway",
            "--node-id-prefix",
            "edge-a",
            "--gateway-bind",
            "127.0.0.1:9080",
            "--admin-upstream",
            "10.0.0.12:8081",
            "--portal-upstream",
            "10.0.0.13:8082",
            "--admin-site-dir",
            "D:/sites/admin",
            "--portal-site-dir",
            "D:/sites/portal",
            "--plan-format",
            "json",
            "--dry-run",
        ])
        .expect("cli should parse");

        let settings = resolve_service_settings(&cli, Vec::<(String, String)>::new())
            .expect("settings should resolve");

        assert_eq!(settings.config_dir, Some("D:/router/config".to_owned()));
        assert_eq!(settings.config_file, Some("cluster/router.yaml".to_owned()));
        assert_eq!(
            settings.database_url,
            Some("postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_router".to_owned())
        );
        assert_eq!(settings.public_web_bind, Some("0.0.0.0:3301".to_owned()));
        assert_eq!(settings.node_id_prefix, Some("edge-a".to_owned()));
        assert_eq!(settings.gateway_bind, Some("127.0.0.1:9080".to_owned()));
        assert_eq!(settings.admin_upstream, Some("10.0.0.12:8081".to_owned()));
        assert_eq!(settings.portal_upstream, Some("10.0.0.13:8082".to_owned()));
        assert_eq!(
            settings.roles,
            Some(vec![ProductRuntimeRole::Web, ProductRuntimeRole::Gateway])
        );
        assert_eq!(
            settings.admin_site_dir,
            Some(PathBuf::from("D:/sites/admin"))
        );
        assert_eq!(
            settings.portal_site_dir,
            Some(PathBuf::from("D:/sites/portal"))
        );
        assert_eq!(settings.plan_format.as_str(), "json");
        assert!(settings.dry_run);
    }

    #[test]
    fn resolve_service_settings_prefers_cli_values_over_env() {
        let cli = RouterProductServiceCli::try_parse_from([
            "router-product-service",
            "--bind",
            "0.0.0.0:4301",
            "--roles",
            "portal",
            "--gateway-upstream",
            "10.0.0.22:8080",
        ])
        .expect("cli should parse");

        let settings = resolve_service_settings(
            &cli,
            [
                ("SDKWORK_DATABASE_URL", "sqlite:///tmp/router.db"),
                ("SDKWORK_WEB_BIND", "0.0.0.0:3001"),
                ("SDKWORK_ROUTER_ROLES", "web,gateway,admin"),
                ("SDKWORK_GATEWAY_PROXY_TARGET", "10.0.0.21:8080"),
                ("SDKWORK_ADMIN_PROXY_TARGET", "10.0.0.31:8081"),
            ],
        )
        .expect("settings should resolve");

        assert_eq!(settings.public_web_bind, Some("0.0.0.0:4301".to_owned()));
        assert_eq!(settings.gateway_upstream, Some("10.0.0.22:8080".to_owned()));
        assert_eq!(settings.admin_upstream, Some("10.0.0.31:8081".to_owned()));
        assert_eq!(settings.database_url, Some("sqlite:///tmp/router.db".to_owned()));
        assert_eq!(settings.roles, Some(vec![ProductRuntimeRole::Portal]));
    }

    #[test]
    fn render_service_plan_reports_cluster_shape() {
        let settings = ProductServiceSettings {
            config_dir: Some("D:/router/config".to_owned()),
            config_file: Some("cluster/router.yaml".to_owned()),
            database_url: Some("postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_router".to_owned()),
            public_web_bind: Some("0.0.0.0:3301".to_owned()),
            roles: Some(vec![ProductRuntimeRole::Web, ProductRuntimeRole::Gateway]),
            node_id_prefix: Some("edge-a".to_owned()),
            gateway_bind: Some("127.0.0.1:9080".to_owned()),
            admin_bind: None,
            portal_bind: None,
            admin_upstream: Some("10.0.0.12:8081".to_owned()),
            portal_upstream: Some("10.0.0.13:8082".to_owned()),
            gateway_upstream: None,
            admin_site_dir: Some(PathBuf::from("D:/sites/admin")),
            portal_site_dir: Some(PathBuf::from("D:/sites/portal")),
            plan_format: PlanFormat::Text,
            dry_run: true,
        };
        let config = StandaloneConfig {
            gateway_bind: "127.0.0.1:9080".to_owned(),
            admin_bind: "127.0.0.1:8081".to_owned(),
            portal_bind: "127.0.0.1:8082".to_owned(),
            database_url: "postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_router".to_owned(),
            ..StandaloneConfig::default()
        };

        let plan = render_service_plan(&settings, &config);

        assert!(plan.contains("router-product-service dry run"));
        assert!(plan.contains("roles=web,gateway"));
        assert!(plan.contains("public_web_bind=0.0.0.0:3301"));
        assert!(plan.contains("database_url=postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_router"));
        assert!(plan.contains("admin_upstream=10.0.0.12:8081"));
        assert!(plan.contains("portal_upstream=10.0.0.13:8082"));
    }

    #[test]
    fn render_service_plan_supports_json_cluster_shape() {
        let settings = ProductServiceSettings {
            config_dir: Some("D:/router/config".to_owned()),
            config_file: Some("cluster/router.yaml".to_owned()),
            database_url: Some(
                "postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_router".to_owned(),
            ),
            public_web_bind: Some("0.0.0.0:3301".to_owned()),
            roles: Some(vec![ProductRuntimeRole::Web, ProductRuntimeRole::Gateway]),
            node_id_prefix: Some("edge-a".to_owned()),
            gateway_bind: Some("127.0.0.1:9080".to_owned()),
            admin_bind: None,
            portal_bind: None,
            admin_upstream: Some("10.0.0.12:8081".to_owned()),
            portal_upstream: Some("10.0.0.13:8082".to_owned()),
            gateway_upstream: None,
            admin_site_dir: Some(PathBuf::from("D:/sites/admin")),
            portal_site_dir: Some(PathBuf::from("D:/sites/portal")),
            plan_format: PlanFormat::Json,
            dry_run: true,
        };
        let config = StandaloneConfig {
            gateway_bind: "127.0.0.1:9080".to_owned(),
            admin_bind: "127.0.0.1:8081".to_owned(),
            portal_bind: "127.0.0.1:8082".to_owned(),
            database_url: "postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_router"
                .to_owned(),
            ..StandaloneConfig::default()
        };

        let plan = render_service_plan(&settings, &config);
        let parsed: Value = serde_json::from_str(&plan).expect("plan should be valid json");

        assert_eq!(parsed["mode"], "dry-run");
        assert_eq!(parsed["plan_format"], "json");
        assert_eq!(parsed["roles"], serde_json::json!(["web", "gateway"]));
        assert_eq!(parsed["public_web_bind"], "0.0.0.0:3301");
        assert_eq!(parsed["binds"]["gateway"], "127.0.0.1:9080");
        assert_eq!(parsed["binds"]["admin"], "127.0.0.1:8081");
        assert_eq!(parsed["binds"]["portal"], "127.0.0.1:8082");
        assert_eq!(
            parsed["site_dirs"]["admin"],
            serde_json::Value::String("D:/sites/admin".to_owned())
        );
        assert_eq!(
            parsed["upstreams"]["admin"],
            serde_json::Value::String("10.0.0.12:8081".to_owned())
        );
        assert_eq!(
            parsed["upstreams"]["portal"],
            serde_json::Value::String("10.0.0.13:8082".to_owned())
        );
    }
}

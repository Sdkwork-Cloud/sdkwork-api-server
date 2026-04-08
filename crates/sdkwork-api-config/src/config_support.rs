use super::*;

pub(crate) fn collect_pairs<I, K, V>(pairs: I) -> HashMap<String, String>
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

pub(crate) fn default_browser_allowed_origins() -> Vec<String> {
    DEFAULT_BROWSER_ALLOWED_ORIGINS
        .into_iter()
        .map(str::to_owned)
        .collect()
}

pub(crate) fn bind_is_loopback(bind: &str) -> bool {
    let trimmed = bind.trim();
    if trimmed.is_empty() {
        return false;
    }

    if let Ok(address) = trimmed.parse::<SocketAddr>() {
        return address.ip().is_loopback();
    }

    normalized_bind_host(trimmed)
        .map(|host| {
            host.eq_ignore_ascii_case("localhost") || host == "127.0.0.1" || host == "::1"
        })
        .unwrap_or(false)
}

pub(crate) fn normalized_bind_host(bind: &str) -> Option<&str> {
    let host = bind
        .rsplit_once(':')
        .map(|(host, _)| host)
        .unwrap_or(bind)
        .trim();
    let host = host.trim_start_matches('[').trim_end_matches(']');
    (!host.is_empty()).then_some(host)
}

pub(crate) fn resolve_local_root_dir(values: &HashMap<String, String>) -> Result<PathBuf> {
    let home_dir = resolve_home_dir(values).ok();

    match values.get(SDKWORK_CONFIG_DIR) {
        Some(path) if !path.trim().is_empty() => {
            let expanded = expand_home_prefix(path, home_dir.as_deref());
            absolutize_path(&expanded)
        }
        _ => {
            let home_dir = home_dir.ok_or_else(|| {
                anyhow::anyhow!(
                    "unable to resolve a home directory for default config root ~/.sdkwork/router"
                )
            })?;
            Ok(LocalConfigPaths::from_home_dir(home_dir).root_dir)
        }
    }
}

pub(crate) fn resolve_home_dir(values: &HashMap<String, String>) -> Result<PathBuf> {
    if let Some(path) = values.get("HOME").filter(|value| !value.trim().is_empty()) {
        return Ok(PathBuf::from(path));
    }
    if let Some(path) = values
        .get("USERPROFILE")
        .filter(|value| !value.trim().is_empty())
    {
        return Ok(PathBuf::from(path));
    }

    let home_drive = values
        .get("HOMEDRIVE")
        .map(String::as_str)
        .unwrap_or_default();
    let home_path = values
        .get("HOMEPATH")
        .map(String::as_str)
        .unwrap_or_default();
    if !home_drive.is_empty() && !home_path.is_empty() {
        return Ok(PathBuf::from(format!("{home_drive}{home_path}")));
    }

    Err(anyhow::anyhow!("home directory is not available"))
}

pub(crate) fn resolve_config_file_path(
    paths: &LocalConfigPaths,
    values: &HashMap<String, String>,
) -> Result<Option<PathBuf>> {
    if let Some(resolved) = resolve_requested_config_file_path(paths, values)? {
        if !resolved.is_file() {
            anyhow::bail!(
                "configured config file does not exist: {}",
                resolved.display()
            );
        }
        return Ok(Some(resolved));
    }

    for candidate in paths.discovered_config_candidates() {
        if candidate.is_file() {
            return Ok(Some(candidate));
        }
    }

    Ok(None)
}

pub(crate) fn config_watch_paths(
    paths: &LocalConfigPaths,
    values: &HashMap<String, String>,
) -> Result<Vec<PathBuf>> {
    if let Some(path) = resolve_requested_config_file_path(paths, values)? {
        return Ok(vec![path]);
    }

    Ok(paths.discovered_config_candidates().to_vec())
}

pub(crate) fn resolve_requested_config_file_path(
    paths: &LocalConfigPaths,
    values: &HashMap<String, String>,
) -> Result<Option<PathBuf>> {
    let home_dir = resolve_home_dir(values).ok();

    values
        .get(SDKWORK_CONFIG_FILE)
        .filter(|value| !value.trim().is_empty())
        .map(|path| {
            let expanded = expand_home_prefix(path, home_dir.as_deref());
            let resolved = if expanded.is_absolute() {
                expanded
            } else {
                paths.root_dir.join(expanded)
            };
            absolutize_path(&resolved)
        })
        .transpose()
}

pub(crate) fn load_config_file(path: &Path) -> Result<StandaloneConfigFile> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read config file {}", path.display()))?;
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase());

    match extension.as_deref() {
        Some("yaml") | Some("yml") => serde_yaml::from_str(&content)
            .with_context(|| format!("failed to parse YAML config file {}", path.display())),
        Some("json") => serde_json::from_str(&content)
            .with_context(|| format!("failed to parse JSON config file {}", path.display())),
        Some(other) => Err(anyhow::anyhow!(
            "unsupported config file extension {other} for {}",
            path.display()
        )),
        None => Err(anyhow::anyhow!(
            "config file {} does not have a supported extension",
            path.display()
        )),
    }
}

pub(crate) fn parse_bool_env(values: &HashMap<String, String>, key: &str, default: bool) -> Result<bool> {
    match values.get(key) {
        Some(value) => value
            .parse::<bool>()
            .map_err(|error| anyhow::anyhow!("invalid boolean for {key}: {error}")),
        None => Ok(default),
    }
}

pub(crate) fn parse_u64_env(values: &HashMap<String, String>, key: &str, default: u64) -> Result<u64> {
    match values.get(key) {
        Some(value) => value
            .parse::<u64>()
            .map_err(|error| anyhow::anyhow!("invalid unsigned integer for {key}: {error}")),
        None => Ok(default),
    }
}

pub(crate) fn parse_trusted_signers(value: &str, key: &str) -> Result<HashMap<String, String>> {
    let mut trusted_signers = HashMap::new();
    for entry in value.split(';') {
        let entry = entry.trim();
        if entry.is_empty() {
            continue;
        }
        let (publisher, public_key) = entry
            .split_once('=')
            .ok_or_else(|| anyhow::anyhow!("invalid signer entry for {key}: {entry}"))?;
        let publisher = publisher.trim();
        let public_key = public_key.trim();
        if publisher.is_empty() || public_key.is_empty() {
            return Err(anyhow::anyhow!("invalid signer entry for {key}: {entry}"));
        }
        if trusted_signers
            .insert(publisher.to_owned(), public_key.to_owned())
            .is_some()
        {
            return Err(anyhow::anyhow!(
                "duplicate signer entry for {key}: {publisher}"
            ));
        }
    }
    Ok(trusted_signers)
}

pub(crate) fn trusted_signers_to_env(trusted_signers: &HashMap<String, String>) -> String {
    let mut entries = trusted_signers.iter().collect::<Vec<_>>();
    entries.sort_by(|left, right| left.0.cmp(right.0));
    entries
        .into_iter()
        .map(|(publisher, public_key)| format!("{publisher}={public_key}"))
        .collect::<Vec<_>>()
        .join(";")
}

pub(crate) fn parse_string_list_env(value: &str, key: &str) -> Result<Vec<String>> {
    let mut entries = Vec::new();
    for entry in value.split(';') {
        let entry = entry.trim();
        if entry.is_empty() {
            continue;
        }
        entries.push(entry.to_owned());
    }

    if entries.is_empty() && !value.trim().is_empty() {
        anyhow::bail!("invalid list value for {key}");
    }

    Ok(entries)
}

pub(crate) fn join_env_list(values: &[String]) -> String {
    values.join(";")
}

pub(crate) fn normalize_optional_string(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_owned())
    }
}

pub(crate) fn normalize_file_path_value(value: &str, base_dir: &Path) -> String {
    let path = PathBuf::from(value);
    if path.is_absolute() {
        path.to_string_lossy().into_owned()
    } else {
        base_dir.join(path).to_string_lossy().into_owned()
    }
}

pub(crate) fn normalize_database_url(value: &str, base_dir: &Path) -> String {
    if !value.to_ascii_lowercase().starts_with("sqlite:") {
        return value.to_owned();
    }
    if value.contains(":memory:") {
        return value.to_owned();
    }

    let query_start = value.find('?').unwrap_or(value.len());
    let (sqlite_part, query) = value.split_at(query_start);
    let raw_path = sqlite_part
        .strip_prefix("sqlite://")
        .or_else(|| sqlite_part.strip_prefix("sqlite:"))
        .unwrap_or(sqlite_part);
    if raw_path.is_empty() {
        return value.to_owned();
    }

    let normalized = raw_path.replace('\\', "/");
    let resolved = if normalized.starts_with('/') || has_windows_drive_prefix(&normalized) {
        sqlite_url_for_normalized_path(&normalized)
    } else {
        sqlite_url_for_path(&base_dir.join(PathBuf::from(normalized)))
    };

    format!("{resolved}{query}")
}

pub(crate) fn sqlite_url_for_path(path: &Path) -> String {
    let normalized = path.to_string_lossy().replace('\\', "/");
    sqlite_url_for_normalized_path(&normalized)
}

pub(crate) fn sqlite_url_for_normalized_path(path: &str) -> String {
    if path.starts_with('/') {
        format!("sqlite://{path}")
    } else {
        format!("sqlite:///{path}")
    }
}

pub(crate) fn has_windows_drive_prefix(path: &str) -> bool {
    let bytes = path.as_bytes();
    bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && (bytes[2] == b'/' || bytes[2] == b'\\')
}

pub(crate) fn join_env_paths(paths: &[String]) -> String {
    let joined = std::env::join_paths(paths.iter().map(PathBuf::from));
    match joined {
        Ok(value) => value.to_string_lossy().into_owned(),
        Err(_) => {
            #[cfg(windows)]
            let separator = ";";
            #[cfg(not(windows))]
            let separator = ":";
            paths.join(separator)
        }
    }
}

pub(crate) fn expand_home_prefix(value: &str, home_dir: Option<&Path>) -> PathBuf {
    if value == "~" {
        return home_dir
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from(value));
    }

    if let Some(stripped) = value
        .strip_prefix("~/")
        .or_else(|| value.strip_prefix("~\\"))
    {
        return home_dir
            .map(|home_dir| home_dir.join(stripped))
            .unwrap_or_else(|| PathBuf::from(value));
    }

    PathBuf::from(value)
}

pub(crate) fn absolutize_path(path: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(std::env::current_dir()?.join(path))
    }
}

pub(crate) fn capture_watch_entry(path: &Path) -> Result<StandaloneConfigWatchEntry> {
    match fs::metadata(path) {
        Ok(metadata) => Ok(StandaloneConfigWatchEntry {
            path: path.to_path_buf(),
            exists: true,
            is_file: metadata.is_file(),
            len: metadata.len(),
            modified_at_ms: metadata.modified().ok().and_then(system_time_to_unix_ms),
        }),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok(StandaloneConfigWatchEntry {
                path: path.to_path_buf(),
                exists: false,
                is_file: false,
                len: 0,
                modified_at_ms: None,
            })
        }
        Err(error) => Err(error).with_context(|| {
            format!(
                "failed to capture config watch metadata for {}",
                path.display()
            )
        }),
    }
}

pub(crate) fn system_time_to_unix_ms(value: SystemTime) -> Option<u128> {
    value
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|duration| duration.as_millis())
}

#[cfg(test)]
mod tests {
    use super::StandaloneConfig;

    #[test]
    fn parses_runtime_snapshot_interval_from_env_pairs() {
        let config =
            StandaloneConfig::from_pairs([("SDKWORK_RUNTIME_SNAPSHOT_INTERVAL_SECS", "30")])
                .unwrap();

        assert_eq!(config.runtime_snapshot_interval_secs, 30);
    }

    #[test]
    fn parses_pricing_lifecycle_sync_interval_from_env_pairs() {
        let config = StandaloneConfig::from_pairs([(
            "SDKWORK_PRICING_LIFECYCLE_SYNC_INTERVAL_SECS",
            "15",
        )])
        .unwrap();

        assert_eq!(config.pricing_lifecycle_sync_interval_secs, 15);
    }

    #[test]
    fn parses_portal_env_pairs() {
        let config = StandaloneConfig::from_pairs([
            ("SDKWORK_PORTAL_BIND", "127.0.0.1:8082"),
            ("SDKWORK_PORTAL_JWT_SIGNING_SECRET", "portal-secret"),
        ])
        .unwrap();

        assert_eq!(config.portal_bind, "127.0.0.1:8082");
        assert_eq!(config.portal_jwt_signing_secret, "portal-secret");
    }
}

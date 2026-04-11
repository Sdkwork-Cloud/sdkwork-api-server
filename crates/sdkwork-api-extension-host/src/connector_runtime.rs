use super::*;

pub fn ensure_connector_runtime_started(
    load_plan: &ExtensionLoadPlan,
    base_url: &str,
) -> Result<ConnectorRuntimeStatus, ExtensionHostError> {
    if load_plan.runtime != ExtensionRuntime::Connector {
        return Err(ExtensionHostError::ConnectorRuntimeUnsupported {
            instance_id: load_plan.instance_id.clone(),
            runtime: load_plan.runtime.clone(),
        });
    }

    let launch_config = ConnectorLaunchConfig::from_load_plan(load_plan, base_url)?;

    if let Some(status) = running_connector_status(&load_plan.instance_id)? {
        if status.base_url == base_url
            && status.health_url == launch_config.health_url
            && probe_http_health(&launch_config.health_url)?
        {
            return Ok(ConnectorRuntimeStatus {
                healthy: true,
                ..status
            });
        }
        shutdown_connector_runtime(&load_plan.instance_id)?;
    }

    if probe_http_health(&launch_config.health_url)? {
        return Ok(external_connector_runtime_status(load_plan, base_url, &launch_config));
    }

    if connector_entrypoint_missing(&launch_config.entrypoint)
        && wait_for_external_connector_health(
            &launch_config.health_url,
            launch_config.startup_timeout,
            launch_config.startup_poll_interval,
        )?
    {
        return Ok(external_connector_runtime_status(load_plan, base_url, &launch_config));
    }

    let mut command = Command::new(&launch_config.entrypoint);
    command.args(&launch_config.args);
    command.stdin(Stdio::null());
    command.stdout(Stdio::null());
    command.stderr(Stdio::null());
    if let Some(working_directory) = &launch_config.working_directory {
        command.current_dir(working_directory);
    }
    command.env(
        "SDKWORK_EXTENSION_INSTANCE_ID",
        load_plan.instance_id.as_str(),
    );
    command.env(
        "SDKWORK_EXTENSION_INSTALLATION_ID",
        load_plan.installation_id.as_str(),
    );
    command.env("SDKWORK_EXTENSION_ID", load_plan.extension_id.as_str());
    command.env("SDKWORK_EXTENSION_BASE_URL", base_url);
    if let Some(credential_ref) = &load_plan.credential_ref {
        command.env("SDKWORK_EXTENSION_CREDENTIAL_REF", credential_ref);
    }
    command.env(
        "SDKWORK_EXTENSION_CONFIG_JSON",
        load_plan.config.to_string(),
    );
    for (key, value) in &launch_config.environment {
        command.env(key, value);
    }

    let child =
        command
            .spawn()
            .map_err(|error| ExtensionHostError::ConnectorRuntimeSpawnFailed {
                instance_id: load_plan.instance_id.clone(),
                entrypoint: launch_config.entrypoint.display().to_string(),
                message: error.to_string(),
            })?;

    let process_id = Some(child.id());
    {
        let mut registry = connector_process_registry()?;
        registry.insert(
            load_plan.instance_id.clone(),
            ManagedConnectorProcess {
                child,
                extension_id: load_plan.extension_id.clone(),
                display_name: load_plan.display_name.clone(),
                base_url: base_url.to_owned(),
                health_url: launch_config.health_url.clone(),
            },
        );
    }

    wait_for_connector_health(
        &load_plan.instance_id,
        &launch_config.health_url,
        launch_config.startup_timeout,
        launch_config.startup_poll_interval,
    )?;

    Ok(ConnectorRuntimeStatus {
        instance_id: load_plan.instance_id.clone(),
        extension_id: load_plan.extension_id.clone(),
        display_name: load_plan.display_name.clone(),
        base_url: base_url.to_owned(),
        health_url: launch_config.health_url,
        process_id,
        running: true,
        healthy: true,
    })
}

fn external_connector_runtime_status(
    load_plan: &ExtensionLoadPlan,
    base_url: &str,
    launch_config: &ConnectorLaunchConfig,
) -> ConnectorRuntimeStatus {
    ConnectorRuntimeStatus {
        instance_id: load_plan.instance_id.clone(),
        extension_id: load_plan.extension_id.clone(),
        display_name: load_plan.display_name.clone(),
        base_url: base_url.to_owned(),
        health_url: launch_config.health_url.clone(),
        process_id: None,
        running: true,
        healthy: true,
    }
}

pub fn shutdown_connector_runtime(instance_id: &str) -> Result<(), ExtensionHostError> {
    let mut registry = connector_process_registry()?;
    if let Some(mut process) = registry.remove(instance_id) {
        kill_child(instance_id, &mut process.child)?;
    }
    Ok(())
}

pub fn shutdown_all_connector_runtimes() -> Result<(), ExtensionHostError> {
    let mut registry = connector_process_registry()?;
    let instance_ids = registry.keys().cloned().collect::<Vec<_>>();
    for instance_id in instance_ids {
        if let Some(mut process) = registry.remove(&instance_id) {
            kill_child(&instance_id, &mut process.child)?;
        }
    }
    Ok(())
}

pub fn shutdown_connector_runtimes_for_extension(
    extension_id: &str,
) -> Result<(), ExtensionHostError> {
    let processes = {
        let mut registry = connector_process_registry()?;
        let instance_ids = registry
            .iter()
            .filter(|(_, process)| process.extension_id == extension_id)
            .map(|(instance_id, _)| instance_id.clone())
            .collect::<Vec<_>>();
        instance_ids
            .into_iter()
            .filter_map(|instance_id| {
                registry
                    .remove(&instance_id)
                    .map(|process| (instance_id, process))
            })
            .collect::<Vec<_>>()
    };

    for (instance_id, mut process) in processes {
        kill_child(&instance_id, &mut process.child)?;
    }

    Ok(())
}

pub fn list_connector_runtime_statuses() -> Result<Vec<ConnectorRuntimeStatus>, ExtensionHostError>
{
    let snapshots = {
        let mut registry = connector_process_registry()?;
        let mut exited_instance_ids = Vec::new();
        let mut snapshots = Vec::new();

        for (instance_id, process) in registry.iter_mut() {
            match process.child.try_wait() {
                Ok(None) => snapshots.push((
                    instance_id.clone(),
                    process.extension_id.clone(),
                    process.display_name.clone(),
                    process.base_url.clone(),
                    process.health_url.clone(),
                    Some(process.child.id()),
                )),
                Ok(Some(_)) => exited_instance_ids.push(instance_id.clone()),
                Err(error) => {
                    return Err(ExtensionHostError::ConnectorRuntimeShutdownFailed {
                        instance_id: instance_id.clone(),
                        message: error.to_string(),
                    });
                }
            }
        }

        for instance_id in exited_instance_ids {
            registry.remove(&instance_id);
        }

        snapshots
    };

    let mut statuses = snapshots
        .into_iter()
        .map(
            |(instance_id, extension_id, display_name, base_url, health_url, process_id)| {
                Ok(ConnectorRuntimeStatus {
                    instance_id,
                    extension_id,
                    display_name,
                    base_url,
                    healthy: probe_http_health(&health_url)?,
                    health_url,
                    process_id,
                    running: true,
                })
            },
        )
        .collect::<Result<Vec<_>, ExtensionHostError>>()?;
    statuses.sort_by(|left, right| left.instance_id.cmp(&right.instance_id));
    Ok(statuses)
}

impl ConnectorLaunchConfig {
    fn from_load_plan(
        load_plan: &ExtensionLoadPlan,
        base_url: &str,
    ) -> Result<Self, ExtensionHostError> {
        let Some(entrypoint) = load_plan.entrypoint.as_deref() else {
            return Err(ExtensionHostError::ConnectorRuntimeEntrypointMissing {
                instance_id: load_plan.instance_id.clone(),
            });
        };

        let entrypoint = resolve_entrypoint(entrypoint, load_plan.package_root.as_deref());
        let args = config_string_array(&load_plan.config, "command_args");
        let environment = config_string_map(&load_plan.config, "environment");
        let working_directory = config_path(
            &load_plan.config,
            "working_directory",
            load_plan.package_root.as_deref(),
        )
        .or_else(|| load_plan.package_root.clone());
        let health_path =
            config_string(&load_plan.config, "health_path").unwrap_or_else(|| "/health".to_owned());
        let health_url = join_health_url(base_url, &health_path).ok_or_else(|| {
            ExtensionHostError::ConnectorRuntimeBaseUrlInvalid {
                instance_id: load_plan.instance_id.clone(),
                base_url: base_url.to_owned(),
            }
        })?;

        Ok(Self {
            entrypoint,
            args,
            environment,
            working_directory,
            health_url,
            startup_timeout: Duration::from_millis(config_u64(
                &load_plan.config,
                "startup_timeout_ms",
                5_000,
            )),
            startup_poll_interval: Duration::from_millis(config_u64(
                &load_plan.config,
                "startup_poll_interval_ms",
                50,
            )),
        })
    }
}

fn connector_process_registry() -> Result<
    std::sync::MutexGuard<'static, HashMap<String, ManagedConnectorProcess>>,
    ExtensionHostError,
> {
    CONNECTOR_PROCESS_REGISTRY
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .map_err(|_| ExtensionHostError::ConnectorRuntimeStatePoisoned)
}

pub(crate) fn native_dynamic_runtime_registry() -> Result<
    std::sync::MutexGuard<'static, HashMap<String, Arc<NativeDynamicRuntime>>>,
    ExtensionHostError,
> {
    NATIVE_DYNAMIC_RUNTIME_REGISTRY
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .map_err(|_| ExtensionHostError::NativeDynamicRuntimeStatePoisoned {
            entrypoint: "native_dynamic_runtime_registry".to_owned(),
        })
}

fn running_connector_status(
    instance_id: &str,
) -> Result<Option<ConnectorRuntimeStatus>, ExtensionHostError> {
    let mut registry = connector_process_registry()?;
    let mut should_remove = false;
    let status = match registry.get_mut(instance_id) {
        Some(process) => match process.child.try_wait() {
            Ok(None) => Some(ConnectorRuntimeStatus {
                instance_id: instance_id.to_owned(),
                extension_id: process.extension_id.clone(),
                display_name: process.display_name.clone(),
                base_url: process.base_url.clone(),
                health_url: process.health_url.clone(),
                process_id: Some(process.child.id()),
                running: true,
                healthy: false,
            }),
            Ok(Some(_)) => {
                should_remove = true;
                None
            }
            Err(error) => {
                return Err(ExtensionHostError::ConnectorRuntimeShutdownFailed {
                    instance_id: instance_id.to_owned(),
                    message: error.to_string(),
                });
            }
        },
        None => None,
    };

    if should_remove {
        registry.remove(instance_id);
    }

    Ok(status)
}

fn wait_for_connector_health(
    instance_id: &str,
    health_url: &str,
    timeout: Duration,
    poll_interval: Duration,
) -> Result<(), ExtensionHostError> {
    let start = Instant::now();
    loop {
        if connector_process_exited(instance_id)? {
            return Err(ExtensionHostError::ConnectorRuntimeExited {
                instance_id: instance_id.to_owned(),
                status: None,
            });
        }

        if probe_http_health(health_url)? {
            return Ok(());
        }

        if start.elapsed() >= timeout {
            return Err(ExtensionHostError::ConnectorRuntimeHealthTimedOut {
                instance_id: instance_id.to_owned(),
                health_url: health_url.to_owned(),
                timeout_ms: timeout.as_millis() as u64,
            });
        }

        std::thread::sleep(poll_interval);
    }
}

fn wait_for_external_connector_health(
    health_url: &str,
    timeout: Duration,
    poll_interval: Duration,
) -> Result<bool, ExtensionHostError> {
    let start = Instant::now();
    loop {
        if probe_http_health(health_url)? {
            return Ok(true);
        }

        if start.elapsed() >= timeout {
            return Ok(false);
        }

        std::thread::sleep(poll_interval);
    }
}

fn connector_process_exited(instance_id: &str) -> Result<bool, ExtensionHostError> {
    let mut registry = connector_process_registry()?;
    let mut should_remove = false;
    let exited = match registry.get_mut(instance_id) {
        Some(process) => match process.child.try_wait() {
            Ok(Some(_)) => {
                should_remove = true;
                true
            }
            Ok(None) => false,
            Err(error) => {
                return Err(ExtensionHostError::ConnectorRuntimeShutdownFailed {
                    instance_id: instance_id.to_owned(),
                    message: error.to_string(),
                });
            }
        },
        None => true,
    };

    if should_remove {
        registry.remove(instance_id);
    }

    Ok(exited)
}

fn kill_child(instance_id: &str, child: &mut Child) -> Result<(), ExtensionHostError> {
    match child.try_wait() {
        Ok(Some(_)) => Ok(()),
        Ok(None) => {
            child
                .kill()
                .map_err(|error| ExtensionHostError::ConnectorRuntimeShutdownFailed {
                    instance_id: instance_id.to_owned(),
                    message: error.to_string(),
                })?;
            let _ = child.wait();
            Ok(())
        }
        Err(error) => Err(ExtensionHostError::ConnectorRuntimeShutdownFailed {
            instance_id: instance_id.to_owned(),
            message: error.to_string(),
        }),
    }
}

pub(crate) fn resolve_entrypoint(entrypoint: &str, package_root: Option<&Path>) -> PathBuf {
    let path = PathBuf::from(entrypoint);
    if path.is_absolute() {
        return path;
    }

    if entrypoint.contains('\\') || entrypoint.contains('/') || entrypoint.starts_with('.') {
        if let Some(package_root) = package_root {
            return package_root.join(path);
        }
    }

    path
}

fn connector_entrypoint_missing(entrypoint: &Path) -> bool {
    connector_entrypoint_requires_filesystem_presence(entrypoint) && !entrypoint.exists()
}

fn connector_entrypoint_requires_filesystem_presence(entrypoint: &Path) -> bool {
    entrypoint.is_absolute() || entrypoint.components().count() > 1
}

fn config_string(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

fn config_string_array(value: &Value, key: &str) -> Vec<String> {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn config_string_map(value: &Value, key: &str) -> HashMap<String, String> {
    value
        .get(key)
        .and_then(Value::as_object)
        .map(|items| {
            items
                .iter()
                .filter_map(|(key, value)| {
                    value.as_str().map(|value| (key.clone(), value.to_owned()))
                })
                .collect::<HashMap<_, _>>()
        })
        .unwrap_or_default()
}

fn config_u64(value: &Value, key: &str, default: u64) -> u64 {
    value.get(key).and_then(Value::as_u64).unwrap_or(default)
}

fn config_path(value: &Value, key: &str, package_root: Option<&Path>) -> Option<PathBuf> {
    let raw = config_string(value, key)?;
    let path = PathBuf::from(&raw);
    if path.is_absolute() {
        Some(path)
    } else {
        package_root.map(|package_root| package_root.join(path))
    }
}

fn join_health_url(base_url: &str, health_path: &str) -> Option<String> {
    let normalized_path = if health_path.starts_with('/') {
        health_path.to_owned()
    } else {
        format!("/{health_path}")
    };
    let base_url = base_url.trim_end_matches('/');
    if base_url.starts_with("http://") {
        Some(format!("{base_url}{normalized_path}"))
    } else {
        None
    }
}

fn probe_http_health(health_url: &str) -> Result<bool, ExtensionHostError> {
    let (address, path) = parse_http_health_url(health_url)?;
    let mut stream = match TcpStream::connect_timeout(&address, Duration::from_millis(250)) {
        Ok(stream) => stream,
        Err(_) => return Ok(false),
    };
    let _ = stream.set_read_timeout(Some(Duration::from_millis(250)));
    let _ = stream.set_write_timeout(Some(Duration::from_millis(250)));

    let request = format!(
        "GET {path} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
        address
    );
    if stream.write_all(request.as_bytes()).is_err() {
        return Ok(false);
    }

    let mut response = Vec::new();
    let mut buffer = [0_u8; 1024];
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => break,
            Ok(bytes_read) => {
                response.extend_from_slice(&buffer[..bytes_read]);
                if response.windows(4).any(|window| window == b"\r\n\r\n") {
                    break;
                }
            }
            Err(_) => return Ok(false),
        }
    }

    let response = String::from_utf8_lossy(&response);
    Ok(response.starts_with("HTTP/1.1 200")
        || response.starts_with("HTTP/1.0 200")
        || response.starts_with("HTTP/2 200"))
}

fn parse_http_health_url(health_url: &str) -> Result<(SocketAddr, String), ExtensionHostError> {
    let invalid_base_url = || ExtensionHostError::ConnectorRuntimeBaseUrlInvalid {
        instance_id: "connector_health_probe".to_owned(),
        base_url: health_url.to_owned(),
    };

    let Some(without_scheme) = health_url.strip_prefix("http://") else {
        return Err(invalid_base_url());
    };

    let (authority, path) = match without_scheme.split_once('/') {
        Some((authority, path)) => (authority, format!("/{path}")),
        None => (without_scheme, "/".to_owned()),
    };

    if authority.is_empty() {
        return Err(invalid_base_url());
    }

    let address = authority
        .to_socket_addrs()
        .map_err(|_| invalid_base_url())?
        .next()
        .ok_or_else(invalid_base_url)?;

    Ok((address, path))
}

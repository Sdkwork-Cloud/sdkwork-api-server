use super::*;

impl NativeDynamicRuntime {
    fn lifecycle_context(&self) -> ExtensionLifecycleContext {
        ExtensionLifecycleContext::new(self.manifest.id.clone(), self.entrypoint.clone())
    }

    fn lock_state(
        &self,
    ) -> Result<std::sync::MutexGuard<'_, NativeDynamicLifecycleState>, ExtensionHostError> {
        self.lifecycle_state.lock().map_err(|_| {
            ExtensionHostError::NativeDynamicRuntimeStatePoisoned {
                entrypoint: self.entrypoint.clone(),
            }
        })
    }

    fn initialize(&self) -> Result<(), ExtensionHostError> {
        let message = if let Some(init_json) = self.init_json {
            let result: ExtensionLifecycleResult = self.invoke_lifecycle_json(init_json, "init")?;
            if !result.success {
                return Err(ExtensionHostError::NativeDynamicLifecycleFailed {
                    entrypoint: self.entrypoint.clone(),
                    phase: "init".to_owned(),
                    message: result
                        .message
                        .unwrap_or_else(|| "plugin reported init failure".to_owned()),
                });
            }
            result.message
        } else {
            Some("native dynamic runtime loaded".to_owned())
        };

        let mut state = self.lock_state()?;
        state.running = true;
        state.healthy = true;
        state.message = message;
        Ok(())
    }

    fn begin_invocation(&self) -> Result<(), ExtensionHostError> {
        let mut state = self.lock_state()?;
        if !state.running {
            return Err(ExtensionHostError::NativeDynamicInvocationFailed {
                entrypoint: self.entrypoint.clone(),
                message: "native dynamic runtime is not running".to_owned(),
                code: None,
                retryable: None,
                retry_after_ms: None,
            });
        }
        if state.draining || state.shutdown_invoked {
            return Err(ExtensionHostError::NativeDynamicInvocationFailed {
                entrypoint: self.entrypoint.clone(),
                message: "native dynamic runtime is draining for shutdown".to_owned(),
                code: None,
                retryable: None,
                retry_after_ms: None,
            });
        }
        state.active_invocations += 1;
        Ok(())
    }

    fn finish_invocation(&self) {
        let mut state = self
            .lifecycle_state
            .lock()
            .expect("native dynamic runtime state lock");
        if state.active_invocations > 0 {
            state.active_invocations -= 1;
        }
        if state.active_invocations == 0 {
            self.lifecycle_drained.notify_all();
        }
    }

    fn invocation_guard(&self) -> Result<NativeDynamicInvocationGuard<'_>, ExtensionHostError> {
        self.begin_invocation()?;
        Ok(NativeDynamicInvocationGuard {
            runtime: self,
            finished: false,
        })
    }

    fn ensure_running(&self) -> Result<(), ExtensionHostError> {
        if self.lock_state()?.running {
            Ok(())
        } else {
            Err(ExtensionHostError::NativeDynamicInvocationFailed {
                entrypoint: self.entrypoint.clone(),
                message: "native dynamic runtime is not running".to_owned(),
                code: None,
                retryable: None,
                retry_after_ms: None,
            })
        }
    }

    fn status(&self) -> Result<NativeDynamicRuntimeStatus, ExtensionHostError> {
        let (running, mut healthy, mut message) = {
            let state = self.lock_state()?;
            (state.running, state.healthy, state.message.clone())
        };

        if running && self.health_check_json.is_some() {
            match self.health_check() {
                Ok(result) => {
                    healthy = result.healthy;
                    message = result.message.clone();
                    let mut state = self.lock_state()?;
                    state.healthy = result.healthy;
                    state.message = result.message.clone();
                }
                Err(error) => {
                    healthy = false;
                    message = Some(error.to_string());
                    let mut state = self.lock_state()?;
                    state.healthy = false;
                    state.message = message.clone();
                }
            }
        }

        Ok(NativeDynamicRuntimeStatus {
            extension_id: self.manifest.id.clone(),
            display_name: self.manifest.display_name.clone(),
            library_path: self.entrypoint.clone(),
            running,
            healthy,
            supports_health_check: self.health_check_json.is_some(),
            supports_shutdown: self.shutdown_json.is_some(),
            message,
        })
    }

    fn health_check(&self) -> Result<ExtensionHealthCheckResult, ExtensionHostError> {
        let Some(health_check_json) = self.health_check_json else {
            let state = self.lock_state()?;
            return Ok(ExtensionHealthCheckResult {
                healthy: state.healthy,
                message: state.message.clone(),
                details: None,
            });
        };

        let _guard = self.invocation_guard()?;
        self.invoke_lifecycle_json(health_check_json, "health_check")
    }

    fn shutdown(&self, drain_timeout_ms: Option<u64>) -> Result<(), ExtensionHostError> {
        let mut state = self.lock_state()?;
        if state.shutdown_invoked {
            return Ok(());
        }
        state.draining = true;
        if let Some(timeout_ms) = drain_timeout_ms {
            let timeout = Duration::from_millis(timeout_ms);
            let started_at = Instant::now();
            while state.active_invocations > 0 {
                let Some(remaining) = timeout.checked_sub(started_at.elapsed()) else {
                    state.draining = false;
                    return Err(ExtensionHostError::NativeDynamicShutdownDrainTimedOut {
                        entrypoint: self.entrypoint.clone(),
                        timeout_ms,
                    });
                };
                let (next_state, wait_result) = self
                    .lifecycle_drained
                    .wait_timeout(state, remaining)
                    .map_err(|_| ExtensionHostError::NativeDynamicRuntimeStatePoisoned {
                        entrypoint: self.entrypoint.clone(),
                    })?;
                state = next_state;
                if wait_result.timed_out() && state.active_invocations > 0 {
                    state.draining = false;
                    return Err(ExtensionHostError::NativeDynamicShutdownDrainTimedOut {
                        entrypoint: self.entrypoint.clone(),
                        timeout_ms,
                    });
                }
            }
        } else {
            while state.active_invocations > 0 {
                state = self.lifecycle_drained.wait(state).map_err(|_| {
                    ExtensionHostError::NativeDynamicRuntimeStatePoisoned {
                        entrypoint: self.entrypoint.clone(),
                    }
                })?;
            }
        }
        drop(state);

        let message = if let Some(shutdown_json) = self.shutdown_json {
            let result: ExtensionLifecycleResult =
                self.invoke_lifecycle_json(shutdown_json, "shutdown")?;
            if !result.success {
                return Err(ExtensionHostError::NativeDynamicLifecycleFailed {
                    entrypoint: self.entrypoint.clone(),
                    phase: "shutdown".to_owned(),
                    message: result
                        .message
                        .unwrap_or_else(|| "plugin reported shutdown failure".to_owned()),
                });
            }
            result.message
        } else {
            Some("native dynamic runtime stopped".to_owned())
        };

        let mut state = self.lock_state()?;
        state.running = false;
        state.healthy = false;
        state.shutdown_invoked = true;
        state.draining = false;
        state.message = message;
        Ok(())
    }

    fn invoke_lifecycle_json<T: DeserializeOwned>(
        &self,
        callback: LifecycleJsonFn,
        phase: &str,
    ) -> Result<T, ExtensionHostError> {
        let payload = serde_json::to_string(&self.lifecycle_context()).map_err(|error| {
            ExtensionHostError::NativeDynamicLifecycleFailed {
                entrypoint: self.entrypoint.clone(),
                phase: phase.to_owned(),
                message: error.to_string(),
            }
        })?;
        let raw_payload = into_raw_c_string(payload);
        let raw_result = unsafe { callback(raw_payload.cast_const()) };
        unsafe { free_raw_c_string(raw_payload) };

        let Some(raw_result_json) = (unsafe { from_raw_c_str(raw_result) }) else {
            return Err(ExtensionHostError::NativeDynamicLifecycleFailed {
                entrypoint: self.entrypoint.clone(),
                phase: phase.to_owned(),
                message: "plugin returned null lifecycle result".to_owned(),
            });
        };
        unsafe { (self.free_string)(raw_result) };

        serde_json::from_str(&raw_result_json).map_err(|error| {
            ExtensionHostError::NativeDynamicLifecycleFailed {
                entrypoint: self.entrypoint.clone(),
                phase: phase.to_owned(),
                message: error.to_string(),
            }
        })
    }
}

struct NativeDynamicInvocationGuard<'a> {
    runtime: &'a NativeDynamicRuntime,
    finished: bool,
}

impl Drop for NativeDynamicInvocationGuard<'_> {
    fn drop(&mut self) {
        if !self.finished {
            self.runtime.finish_invocation();
            self.finished = true;
        }
    }
}

pub fn load_native_dynamic_library_manifest(
    entrypoint: &Path,
) -> Result<ExtensionManifest, ExtensionHostError> {
    let (_, manifest) = load_native_dynamic_runtime(entrypoint)?;
    Ok(manifest)
}

pub fn load_native_dynamic_provider_adapter(
    entrypoint: &Path,
    base_url: impl Into<String>,
) -> Result<Box<dyn ProviderExecutionAdapter>, ExtensionHostError> {
    let (runtime, _) = load_or_reuse_native_dynamic_runtime(entrypoint)?;
    Ok(Box::new(NativeDynamicProviderAdapter {
        runtime,
        base_url: base_url.into(),
    }))
}

pub fn list_native_dynamic_runtime_statuses()
-> Result<Vec<NativeDynamicRuntimeStatus>, ExtensionHostError> {
    let runtimes = {
        let registry = native_dynamic_runtime_registry()?;
        registry.values().cloned().collect::<Vec<_>>()
    };

    let mut statuses = runtimes
        .into_iter()
        .map(|runtime| runtime.status())
        .collect::<Result<Vec<_>, ExtensionHostError>>()?;
    statuses.sort_by(|left, right| left.extension_id.cmp(&right.extension_id));
    Ok(statuses)
}

pub fn shutdown_all_native_dynamic_runtimes() -> Result<(), ExtensionHostError> {
    let drain_timeout_ms = configured_native_dynamic_shutdown_drain_timeout_ms()?;
    let runtimes = {
        let mut registry = native_dynamic_runtime_registry()?;
        registry.drain().collect::<Vec<_>>()
    };

    shutdown_native_dynamic_runtime_entries(runtimes, drain_timeout_ms)
}

pub fn shutdown_native_dynamic_runtimes_for_extension(
    extension_id: &str,
) -> Result<(), ExtensionHostError> {
    let drain_timeout_ms = configured_native_dynamic_shutdown_drain_timeout_ms()?;
    let runtimes = {
        let mut registry = native_dynamic_runtime_registry()?;
        let entrypoints = registry
            .iter()
            .filter(|(_, runtime)| runtime.manifest.id == extension_id)
            .map(|(entrypoint, _)| entrypoint.clone())
            .collect::<Vec<_>>();
        entrypoints
            .into_iter()
            .filter_map(|entrypoint| {
                registry
                    .remove(&entrypoint)
                    .map(|runtime| (entrypoint, runtime))
            })
            .collect::<Vec<_>>()
    };

    shutdown_native_dynamic_runtime_entries(runtimes, drain_timeout_ms)
}

impl ProviderAdapter for NativeDynamicProviderAdapter {
    fn id(&self) -> &'static str {
        "native_dynamic"
    }
}

#[async_trait]
impl ProviderExecutionAdapter for NativeDynamicProviderAdapter {
    async fn execute(
        &self,
        api_key: &str,
        request: ProviderRequest<'_>,
    ) -> AnyhowResult<ProviderOutput> {
        let invocation = provider_invocation_from_request(request, api_key, &self.base_url)?;
        if invocation.expects_stream {
            let stream =
                execute_native_dynamic_stream_invocation(Arc::clone(&self.runtime), &invocation)
                    .await?;
            return Ok(ProviderOutput::Stream(stream));
        }
        let result = execute_native_dynamic_invocation(&self.runtime, &invocation)?;
        match result {
            ProviderInvocationResult::Json { body } => Ok(ProviderOutput::Json(body)),
            ProviderInvocationResult::Unsupported { message } => Err(anyhow!(
                "{}",
                message.unwrap_or_else(
                    || "native dynamic provider reported unsupported operation".to_owned()
                )
            )),
            ProviderInvocationResult::Error {
                message,
                code,
                retryable,
                retry_after_ms,
            } => Err(anyhow::Error::new(ExtensionHostError::NativeDynamicInvocationFailed {
                entrypoint: self.runtime.entrypoint.clone(),
                message,
                code,
                retryable,
                retry_after_ms,
            })),
        }
    }

    async fn execute_with_options(
        &self,
        api_key: &str,
        request: ProviderRequest<'_>,
        options: &ProviderRequestOptions,
    ) -> AnyhowResult<ProviderOutput> {
        let invocation = provider_invocation_from_request_with_options(
            request,
            api_key,
            &self.base_url,
            options,
        )?;
        if invocation.expects_stream {
            let stream =
                execute_native_dynamic_stream_invocation(Arc::clone(&self.runtime), &invocation)
                    .await?;
            return Ok(ProviderOutput::Stream(stream));
        }
        let result = execute_native_dynamic_invocation(&self.runtime, &invocation)?;
        match result {
            ProviderInvocationResult::Json { body } => Ok(ProviderOutput::Json(body)),
            ProviderInvocationResult::Unsupported { message } => Err(anyhow!(
                "{}",
                message.unwrap_or_else(
                    || "native dynamic provider reported unsupported operation".to_owned()
                )
            )),
            ProviderInvocationResult::Error {
                message,
                code,
                retryable,
                retry_after_ms,
            } => Err(anyhow::Error::new(ExtensionHostError::NativeDynamicInvocationFailed {
                entrypoint: self.runtime.entrypoint.clone(),
                message,
                code,
                retryable,
                retry_after_ms,
            })),
        }
    }
}

pub(crate) fn merge_config(base: &Value, overlay: &Value) -> Value {
    match (base, overlay) {
        (Value::Object(base_map), Value::Object(overlay_map)) => {
            let mut merged = base_map.clone();
            for (key, overlay_value) in overlay_map {
                let value = match merged.get(key) {
                    Some(base_value) => merge_config(base_value, overlay_value),
                    None => overlay_value.clone(),
                };
                merged.insert(key.clone(), value);
            }
            Value::Object(merged)
        }
        (_, overlay) => overlay.clone(),
    }
}

fn load_native_dynamic_runtime(
    entrypoint: &Path,
) -> Result<(Arc<NativeDynamicRuntime>, ExtensionManifest), ExtensionHostError> {
    unsafe {
        let library = load_native_dynamic_library(entrypoint)?;
        let abi_version = load_native_dynamic_symbol::<AbiVersionFn>(
            &library,
            SDKWORK_EXTENSION_ABI_VERSION_SYMBOL,
            entrypoint,
        )?;
        let manifest_json = load_native_dynamic_symbol::<ManifestJsonFn>(
            &library,
            SDKWORK_EXTENSION_MANIFEST_JSON_SYMBOL,
            entrypoint,
        )?;
        let execute_json = load_native_dynamic_symbol::<ExecuteJsonFn>(
            &library,
            SDKWORK_EXTENSION_PROVIDER_EXECUTE_JSON_SYMBOL,
            entrypoint,
        )?;
        let execute_stream_json = try_load_native_dynamic_symbol::<ExecuteStreamJsonFn>(
            &library,
            SDKWORK_EXTENSION_PROVIDER_EXECUTE_STREAM_JSON_SYMBOL,
        );
        let init_json = try_load_native_dynamic_symbol::<LifecycleJsonFn>(
            &library,
            SDKWORK_EXTENSION_INIT_JSON_SYMBOL,
        );
        let health_check_json = try_load_native_dynamic_symbol::<LifecycleJsonFn>(
            &library,
            SDKWORK_EXTENSION_HEALTH_CHECK_JSON_SYMBOL,
        );
        let shutdown_json = try_load_native_dynamic_symbol::<LifecycleJsonFn>(
            &library,
            SDKWORK_EXTENSION_SHUTDOWN_JSON_SYMBOL,
        );
        let free_string = load_native_dynamic_symbol::<FreeStringFn>(
            &library,
            SDKWORK_EXTENSION_FREE_STRING_SYMBOL,
            entrypoint,
        )?;

        let actual_version = abi_version();
        if actual_version != SDKWORK_EXTENSION_ABI_VERSION {
            return Err(ExtensionHostError::NativeDynamicAbiVersionUnsupported {
                entrypoint: entrypoint.display().to_string(),
                actual_version,
            });
        }

        let manifest_ptr = manifest_json();
        let Some(manifest_json) = from_raw_c_str(manifest_ptr) else {
            return Err(ExtensionHostError::NativeDynamicManifestExportMissing {
                entrypoint: entrypoint.display().to_string(),
            });
        };
        let manifest: ExtensionManifest =
            serde_json::from_str(&manifest_json).map_err(|error| {
                ExtensionHostError::ManifestParseFailed {
                    path: entrypoint.display().to_string(),
                    message: error.to_string(),
                }
            })?;

        let runtime = Arc::new(NativeDynamicRuntime {
            entrypoint: entrypoint.display().to_string(),
            manifest: manifest.clone(),
            _library: library,
            execute_json,
            execute_stream_json,
            init_json,
            health_check_json,
            shutdown_json,
            free_string,
            lifecycle_state: Mutex::new(NativeDynamicLifecycleState {
                running: true,
                healthy: true,
                message: None,
                shutdown_invoked: false,
                draining: false,
                active_invocations: 0,
            }),
            lifecycle_drained: Condvar::new(),
        });
        runtime.initialize()?;

        let mut registry = native_dynamic_runtime_registry()?;
        registry.insert(runtime.entrypoint.clone(), Arc::clone(&runtime));

        Ok((runtime, manifest))
    }
}

pub(crate) fn load_or_reuse_native_dynamic_runtime(
    entrypoint: &Path,
) -> Result<(Arc<NativeDynamicRuntime>, ExtensionManifest), ExtensionHostError> {
    let entrypoint = entrypoint.display().to_string();
    if let Some(runtime) = native_dynamic_runtime_registry()?.get(&entrypoint).cloned() {
        return Ok((runtime.clone(), runtime.manifest.clone()));
    }

    load_native_dynamic_runtime(Path::new(&entrypoint))
}

fn configured_native_dynamic_shutdown_drain_timeout_ms() -> Result<Option<u64>, ExtensionHostError>
{
    let Some(value) = std::env::var_os(SDKWORK_NATIVE_DYNAMIC_SHUTDOWN_DRAIN_TIMEOUT_MS) else {
        return Ok(None);
    };
    let value = value.to_string_lossy().trim().to_owned();
    if value.is_empty() {
        return Ok(None);
    }
    let timeout_ms = value.parse::<u64>().map_err(|_| {
        ExtensionHostError::NativeDynamicShutdownDrainTimeoutInvalid {
            value: value.clone(),
        }
    })?;
    if timeout_ms == 0 {
        Ok(None)
    } else {
        Ok(Some(timeout_ms))
    }
}

fn shutdown_native_dynamic_runtime_entries(
    runtimes: Vec<(String, Arc<NativeDynamicRuntime>)>,
    drain_timeout_ms: Option<u64>,
) -> Result<(), ExtensionHostError> {
    for (index, (entrypoint, runtime)) in runtimes.iter().enumerate() {
        if let Err(error) = runtime.shutdown(drain_timeout_ms) {
            if matches!(
                error,
                ExtensionHostError::NativeDynamicShutdownDrainTimedOut { .. }
            ) {
                let mut registry = native_dynamic_runtime_registry()?;
                registry.insert(entrypoint.clone(), Arc::clone(runtime));
                for (pending_entrypoint, pending_runtime) in runtimes.iter().skip(index + 1) {
                    registry.insert(pending_entrypoint.clone(), Arc::clone(pending_runtime));
                }
            }
            return Err(error);
        }
    }

    Ok(())
}

#[cfg(windows)]
fn load_native_dynamic_library(entrypoint: &Path) -> Result<Library, ExtensionHostError> {
    static WINDOWS_LIBRARY_LOAD_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let lock = WINDOWS_LIBRARY_LOAD_LOCK.get_or_init(|| Mutex::new(()));
    let _guard = lock.lock().expect("windows library load lock");

    if let Some(parent) = entrypoint.parent() {
        prepend_library_directory_to_path(parent, entrypoint)?;
    }

    unsafe {
        Library::new(entrypoint).map_err(|error| {
            ExtensionHostError::NativeDynamicLibraryLoadFailed {
                entrypoint: entrypoint.display().to_string(),
                message: error.to_string(),
            }
        })
    }
}

#[cfg(not(windows))]
fn load_native_dynamic_library(entrypoint: &Path) -> Result<Library, ExtensionHostError> {
    unsafe {
        Library::new(entrypoint).map_err(|error| {
            ExtensionHostError::NativeDynamicLibraryLoadFailed {
                entrypoint: entrypoint.display().to_string(),
                message: error.to_string(),
            }
        })
    }
}

#[cfg(windows)]
fn prepend_library_directory_to_path(
    directory: &Path,
    entrypoint: &Path,
) -> Result<(), ExtensionHostError> {
    let mut paths = std::env::var_os("PATH")
        .map(|value| std::env::split_paths(&value).collect::<Vec<_>>())
        .unwrap_or_default();

    if paths.iter().any(|path| path == directory) {
        return Ok(());
    }

    paths.insert(0, directory.to_path_buf());
    let joined = std::env::join_paths(paths).map_err(|error| {
        ExtensionHostError::NativeDynamicLibraryLoadFailed {
            entrypoint: entrypoint.display().to_string(),
            message: format!("failed to extend PATH for native dynamic library: {error}"),
        }
    })?;
    std::env::set_var("PATH", joined);
    Ok(())
}

unsafe fn load_native_dynamic_symbol<T: Copy>(
    library: &Library,
    symbol: &[u8],
    entrypoint: &Path,
) -> Result<T, ExtensionHostError> {
    library
        .get::<T>(symbol)
        .map(|loaded| *loaded)
        .map_err(|error| ExtensionHostError::NativeDynamicSymbolMissing {
            entrypoint: entrypoint.display().to_string(),
            symbol: String::from_utf8_lossy(symbol)
                .trim_end_matches('\0')
                .to_owned(),
            message: error.to_string(),
        })
}

unsafe fn try_load_native_dynamic_symbol<T: Copy>(library: &Library, symbol: &[u8]) -> Option<T> {
    library.get::<T>(symbol).ok().map(|loaded| *loaded)
}

pub(crate) fn ensure_native_dynamic_manifest_matches(
    package_manifest: &ExtensionManifest,
    library_manifest: &ExtensionManifest,
    entrypoint: &Path,
) -> Result<(), ExtensionHostError> {
    let same = package_manifest.api_version == library_manifest.api_version
        && package_manifest.id == library_manifest.id
        && package_manifest.kind == library_manifest.kind
        && package_manifest.version == library_manifest.version
        && package_manifest.display_name == library_manifest.display_name
        && package_manifest.runtime == library_manifest.runtime
        && package_manifest.supported_modalities == library_manifest.supported_modalities
        && package_manifest.protocol == library_manifest.protocol
        && package_manifest.runtime_compat_version == library_manifest.runtime_compat_version
        && package_manifest.config_schema == library_manifest.config_schema
        && package_manifest.config_schema_version == library_manifest.config_schema_version
        && package_manifest.credential_schema == library_manifest.credential_schema
        && package_manifest.permissions == library_manifest.permissions
        && package_manifest.channel_bindings == library_manifest.channel_bindings
        && package_manifest.capabilities == library_manifest.capabilities;

    if same {
        Ok(())
    } else {
        Err(ExtensionHostError::NativeDynamicManifestMismatch {
            entrypoint: entrypoint.display().to_string(),
            message: format!(
                "package manifest {} does not match library-exported manifest {}",
                package_manifest.id, library_manifest.id
            ),
        })
    }
}

pub(crate) fn execute_native_dynamic_invocation(
    runtime: &NativeDynamicRuntime,
    invocation: &ProviderInvocation,
) -> Result<ProviderInvocationResult, ExtensionHostError> {
    runtime.ensure_running()?;
    let _guard = runtime.invocation_guard()?;
    let payload = serde_json::to_string(invocation).map_err(|error| {
        ExtensionHostError::NativeDynamicInvocationSerializeFailed {
            operation: invocation.operation.clone(),
            message: error.to_string(),
        }
    })?;
    let raw_payload = into_raw_c_string(payload);
    let raw_result = unsafe { (runtime.execute_json)(raw_payload.cast_const()) };
    unsafe { free_raw_c_string(raw_payload) };
    let Some(raw_result_json) = (unsafe { from_raw_c_str(raw_result) }) else {
        return Err(ExtensionHostError::NativeDynamicResponseParseFailed {
            entrypoint: runtime.entrypoint.clone(),
            message: "plugin returned null result".to_owned(),
        });
    };
    unsafe { (runtime.free_string)(raw_result) };

    serde_json::from_str(&raw_result_json).map_err(|error| {
        ExtensionHostError::NativeDynamicResponseParseFailed {
            entrypoint: runtime.entrypoint.clone(),
            message: error.to_string(),
        }
    })
}

pub(crate) async fn execute_native_dynamic_stream_invocation(
    runtime: Arc<NativeDynamicRuntime>,
    invocation: &ProviderInvocation,
) -> Result<ProviderStreamOutput, ExtensionHostError> {
    runtime.ensure_running()?;
    let Some(execute_stream_json) = runtime.execute_stream_json else {
        return Err(ExtensionHostError::NativeDynamicInvocationFailed {
            entrypoint: runtime.entrypoint.clone(),
            message: "plugin does not export stream execution".to_owned(),
            code: None,
            retryable: None,
            retry_after_ms: None,
        });
    };

    let payload = serde_json::to_string(invocation).map_err(|error| {
        ExtensionHostError::NativeDynamicInvocationSerializeFailed {
            operation: invocation.operation.clone(),
            message: error.to_string(),
        }
    })?;
    runtime.begin_invocation()?;

    let (event_sender, mut event_receiver) = mpsc::unbounded_channel();
    let runtime_for_thread = Arc::clone(&runtime);
    std::thread::spawn(move || {
        struct NativeDynamicOwnedInvocationGuard {
            runtime: Arc<NativeDynamicRuntime>,
        }

        impl Drop for NativeDynamicOwnedInvocationGuard {
            fn drop(&mut self) {
                self.runtime.finish_invocation();
            }
        }

        let _guard = NativeDynamicOwnedInvocationGuard {
            runtime: Arc::clone(&runtime_for_thread),
        };
        let raw_payload = into_raw_c_string(payload);
        let mut writer_context = Box::new(HostStreamWriterContext {
            sender: event_sender.clone(),
        });
        let writer = ProviderStreamWriter {
            context: (&mut *writer_context as *mut HostStreamWriterContext).cast::<c_void>(),
            set_content_type: Some(host_stream_writer_set_content_type),
            write_chunk: Some(host_stream_writer_write_chunk),
        };

        let raw_result =
            unsafe { execute_stream_json(raw_payload.cast_const(), &writer as *const _) };
        unsafe { free_raw_c_string(raw_payload) };

        let result = if raw_result.is_null() {
            ProviderStreamInvocationResult::error("plugin returned null stream result")
        } else {
            let decoded = unsafe { from_raw_c_str(raw_result) };
            unsafe { (runtime_for_thread.free_string)(raw_result) };
            match decoded {
                Some(raw_result_json) => {
                    serde_json::from_str(&raw_result_json).unwrap_or_else(|error| {
                        ProviderStreamInvocationResult::error(format!(
                            "invalid stream result payload: {error}"
                        ))
                    })
                }
                None => {
                    ProviderStreamInvocationResult::error("plugin returned invalid stream result")
                }
            }
        };

        let _ = event_sender.send(NativeDynamicStreamEvent::Finished(result));
    });

    let mut content_type = None;
    let mut prefix_chunk = None;
    match event_receiver.recv().await {
        Some(NativeDynamicStreamEvent::ContentType(value)) => {
            content_type = Some(value);
        }
        Some(NativeDynamicStreamEvent::Chunk(value)) => {
            prefix_chunk = Some(value);
        }
        Some(NativeDynamicStreamEvent::Finished(result)) => {
            return match result {
                ProviderStreamInvocationResult::Streamed { content_type } => {
                    Ok(ProviderStreamOutput::new(content_type, stream::empty()))
                }
                ProviderStreamInvocationResult::Unsupported { message } => {
                    Err(ExtensionHostError::NativeDynamicInvocationFailed {
                        entrypoint: runtime.entrypoint.clone(),
                        message: message.unwrap_or_else(|| {
                            "native dynamic provider reported unsupported stream operation"
                                .to_owned()
                        }),
                        code: None,
                        retryable: None,
                        retry_after_ms: None,
                    })
                }
                ProviderStreamInvocationResult::Error {
                    message,
                    code,
                    retryable,
                    retry_after_ms,
                } => {
                    Err(ExtensionHostError::NativeDynamicInvocationFailed {
                        entrypoint: runtime.entrypoint.clone(),
                        message,
                        code,
                        retryable,
                        retry_after_ms,
                    })
                }
            };
        }
        None => {
            return Err(ExtensionHostError::NativeDynamicInvocationFailed {
                entrypoint: runtime.entrypoint.clone(),
                message: "plugin closed stream without producing metadata or chunks".to_owned(),
                code: None,
                retryable: None,
                retry_after_ms: None,
            });
        }
    }

    let content_type = content_type.unwrap_or_else(|| "application/octet-stream".to_owned());
    let trailing_stream = UnboundedReceiverStream::new(event_receiver).filter_map(|event| async {
        match event {
            NativeDynamicStreamEvent::ContentType(_) => None,
            NativeDynamicStreamEvent::Chunk(bytes) => Some(Ok(bytes)),
            NativeDynamicStreamEvent::Finished(ProviderStreamInvocationResult::Streamed {
                ..
            }) => None,
            NativeDynamicStreamEvent::Finished(ProviderStreamInvocationResult::Unsupported {
                message,
            }) => Some(Err(io::Error::other(message.unwrap_or_else(|| {
                "native dynamic provider reported unsupported stream operation".to_owned()
            })))),
            NativeDynamicStreamEvent::Finished(ProviderStreamInvocationResult::Error {
                message,
                ..
            }) => Some(Err(io::Error::other(message))),
        }
    });

    let body_stream = stream::iter(prefix_chunk.into_iter().map(Ok)).chain(trailing_stream);

    Ok(ProviderStreamOutput::new(content_type, body_stream))
}

unsafe extern "C" fn host_stream_writer_set_content_type(
    context: *mut c_void,
    content_type: *const c_char,
) -> bool {
    if context.is_null() {
        return false;
    }
    let Some(content_type) = from_raw_c_str(content_type) else {
        return false;
    };
    let context = &*(context.cast::<HostStreamWriterContext>());
    context
        .sender
        .send(NativeDynamicStreamEvent::ContentType(content_type))
        .is_ok()
}

unsafe extern "C" fn host_stream_writer_write_chunk(
    context: *mut c_void,
    chunk_ptr: *const u8,
    chunk_len: usize,
) -> bool {
    if context.is_null() || chunk_ptr.is_null() {
        return false;
    }
    let chunk = std::slice::from_raw_parts(chunk_ptr, chunk_len);
    let context = &*(context.cast::<HostStreamWriterContext>());
    context
        .sender
        .send(NativeDynamicStreamEvent::Chunk(Bytes::copy_from_slice(
            chunk,
        )))
        .is_ok()
}

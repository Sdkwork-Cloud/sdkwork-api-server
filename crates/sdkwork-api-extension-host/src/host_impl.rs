use super::*;

impl ExtensionHost {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_builtin(&mut self, factory: BuiltinExtensionFactory) {
        self.register_builtin_manifest(factory.manifest);
    }

    pub fn register_builtin_manifest(&mut self, manifest: ExtensionManifest) {
        self.manifests.insert(manifest.id.clone(), manifest);
    }

    pub fn register_discovered_manifest(&mut self, package: DiscoveredExtensionPackage) {
        self.package_roots
            .insert(package.manifest.id.clone(), package.root_dir.clone());
        self.manifests
            .insert(package.manifest.id.clone(), package.manifest);
    }

    pub fn register_builtin_provider(&mut self, factory: BuiltinProviderExtensionFactory) {
        let extension_id = factory.manifest.id.clone();
        let aliases = provider_runtime_aliases(&factory.adapter_kind)
            .iter()
            .map(|alias| (*alias).to_owned())
            .collect::<Vec<_>>();
        self.register_builtin_manifest(factory.manifest);
        self.provider_factories
            .insert(extension_id.clone(), factory.factory);
        self.provider_aliases
            .insert(factory.adapter_kind, extension_id.clone());
        for alias in aliases {
            self.provider_aliases.insert(alias, extension_id.clone());
        }
    }

    pub fn register_discovered_provider<F>(
        &mut self,
        package: DiscoveredExtensionPackage,
        adapter_kind: impl Into<String>,
        factory: F,
    ) where
        F: Fn(String) -> Box<dyn ProviderExecutionAdapter> + Send + Sync + 'static,
    {
        let extension_id = package.manifest.id.clone();
        let adapter_kind = adapter_kind.into();
        let aliases = provider_runtime_aliases(&adapter_kind)
            .iter()
            .map(|alias| (*alias).to_owned())
            .collect::<Vec<_>>();
        self.package_roots
            .insert(extension_id.clone(), package.root_dir.clone());
        self.manifests
            .insert(extension_id.clone(), package.manifest);
        self.provider_factories
            .insert(extension_id.clone(), Arc::new(factory));
        self.provider_aliases
            .insert(adapter_kind, extension_id.clone());
        for alias in aliases {
            self.provider_aliases.insert(alias, extension_id.clone());
        }
    }

    pub fn register_discovered_native_dynamic_provider(
        &mut self,
        package: DiscoveredExtensionPackage,
    ) -> Result<(), ExtensionHostError> {
        let extension_id = package.manifest.id.clone();
        let entrypoint = package.manifest.entrypoint.as_deref().ok_or(
            ExtensionHostError::ManifestReadFailed {
                path: package.manifest_path.display().to_string(),
                message: "native dynamic extension manifest has no entrypoint".to_owned(),
            },
        )?;
        let library_path = resolve_entrypoint(entrypoint, Some(&package.root_dir));
        let (runtime, library_manifest) = load_or_reuse_native_dynamic_runtime(&library_path)?;
        ensure_native_dynamic_manifest_matches(
            &package.manifest,
            &library_manifest,
            &library_path,
        )?;

        self.package_roots
            .insert(extension_id.clone(), package.root_dir.clone());
        self.manifests
            .insert(extension_id.clone(), package.manifest);
        self.provider_factories.insert(
            extension_id,
            Arc::new(move |base_url| {
                Box::new(NativeDynamicProviderAdapter {
                    runtime: runtime.clone(),
                    base_url,
                })
            }),
        );
        Ok(())
    }

    pub fn manifest(&self, id: &str) -> Option<&ExtensionManifest> {
        self.manifests.get(id)
    }

    pub fn install(
        &mut self,
        installation: ExtensionInstallation,
    ) -> Result<(), ExtensionHostError> {
        if !self.manifests.contains_key(&installation.extension_id) {
            return Err(ExtensionHostError::ManifestNotFound {
                extension_id: installation.extension_id,
            });
        }

        self.installations
            .insert(installation.installation_id.clone(), installation);
        Ok(())
    }

    pub fn installations(&self) -> Vec<ExtensionInstallation> {
        self.installations.values().cloned().collect()
    }

    pub fn mount_instance(
        &mut self,
        instance: ExtensionInstance,
    ) -> Result<(), ExtensionHostError> {
        let Some(installation) = self.installations.get(&instance.installation_id) else {
            return Err(ExtensionHostError::InstallationNotFound {
                installation_id: instance.installation_id,
            });
        };

        if installation.extension_id != instance.extension_id {
            return Err(ExtensionHostError::InstallationExtensionMismatch {
                installation_id: installation.installation_id.clone(),
                installation_extension_id: installation.extension_id.clone(),
                instance_extension_id: instance.extension_id,
            });
        }

        let instances = self
            .instances_by_extension
            .entry(installation.extension_id.clone())
            .or_default();
        if let Some(existing) = instances
            .iter_mut()
            .find(|existing| existing.instance_id == instance.instance_id)
        {
            *existing = instance.clone();
        } else {
            instances.push(instance.clone());
        }

        self.instances_by_id
            .insert(instance.instance_id.clone(), instance);
        Ok(())
    }

    pub fn instances(&self, extension_id: &str) -> Vec<ExtensionInstance> {
        self.instances_by_extension
            .get(extension_id)
            .cloned()
            .unwrap_or_default()
    }

    pub fn load_plan(&self, instance_id: &str) -> Result<ExtensionLoadPlan, ExtensionHostError> {
        let Some(instance) = self.instances_by_id.get(instance_id) else {
            return Err(ExtensionHostError::InstanceNotFound {
                instance_id: instance_id.to_owned(),
            });
        };
        let Some(installation) = self.installations.get(&instance.installation_id) else {
            return Err(ExtensionHostError::InstallationNotFound {
                installation_id: instance.installation_id.clone(),
            });
        };
        let Some(manifest) = self.manifests.get(&instance.extension_id) else {
            return Err(ExtensionHostError::ManifestNotFound {
                extension_id: instance.extension_id.clone(),
            });
        };

        if installation.runtime != manifest.runtime {
            return Err(ExtensionHostError::RuntimeMismatch {
                extension_id: manifest.id.clone(),
                manifest_runtime: manifest.runtime.clone(),
                installation_runtime: installation.runtime.clone(),
            });
        }

        Ok(ExtensionLoadPlan {
            instance_id: instance.instance_id.clone(),
            installation_id: installation.installation_id.clone(),
            extension_id: manifest.id.clone(),
            enabled: installation.enabled && instance.enabled,
            runtime: installation.runtime.clone(),
            display_name: manifest.display_name.clone(),
            entrypoint: installation
                .entrypoint
                .clone()
                .or_else(|| manifest.entrypoint.clone()),
            base_url: instance.base_url.clone(),
            credential_ref: instance.credential_ref.clone(),
            config_schema: manifest.config_schema.clone(),
            credential_schema: manifest.credential_schema.clone(),
            package_root: self.package_roots.get(&instance.extension_id).cloned(),
            config: merge_config(&installation.config, &instance.config),
        })
    }

    fn resolve_provider_extension_id(&self, runtime_key: &str) -> Option<String> {
        if self.manifests.contains_key(runtime_key) {
            Some(runtime_key.to_owned())
        } else {
            self.provider_aliases.get(runtime_key).cloned()
        }
    }

    fn resolve_raw_native_dynamic_runtime(
        &self,
        runtime_key: &str,
        operation: &str,
    ) -> Result<Option<Arc<NativeDynamicRuntime>>, ExtensionHostError> {
        let Some(extension_id) = self.resolve_provider_extension_id(runtime_key) else {
            return Ok(None);
        };
        let Some(manifest) = self.manifests.get(&extension_id) else {
            return Ok(None);
        };
        if !manifest.runtime.supports_raw_provider_execution() {
            return Ok(None);
        }
        if !manifest.capabilities.iter().any(|capability| {
            capability.operation == operation
                && capability.compatibility != CompatibilityLevel::Unsupported
        }) {
            return Ok(None);
        }

        let entrypoint = manifest.entrypoint.as_deref().ok_or(
            ExtensionHostError::ManifestReadFailed {
                path: extension_id.clone(),
                message: "native dynamic extension manifest has no entrypoint".to_owned(),
            },
        )?;
        let library_path = resolve_entrypoint(
            entrypoint,
            self.package_roots.get(&extension_id).map(|path| path.as_path()),
        );
        let (runtime, _) = load_or_reuse_native_dynamic_runtime(&library_path)?;
        Ok(Some(runtime))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn execute_raw_provider_json(
        &self,
        runtime_key: &str,
        base_url: impl Into<String>,
        api_key: &str,
        operation: &str,
        path_params: Vec<String>,
        body: Value,
        headers: HashMap<String, String>,
    ) -> Result<Option<Value>, ExtensionHostError> {
        let Some(runtime) = self.resolve_raw_native_dynamic_runtime(runtime_key, operation)? else {
            return Ok(None);
        };
        let invocation = ProviderInvocation::new(
            operation,
            api_key,
            base_url,
            path_params,
            body,
            false,
        )
        .with_headers(headers);
        match execute_native_dynamic_invocation(runtime.as_ref(), &invocation)? {
            ProviderInvocationResult::Json { body } => Ok(Some(body)),
            ProviderInvocationResult::Unsupported { message } => {
                Err(ExtensionHostError::NativeDynamicInvocationFailed {
                    entrypoint: runtime.entrypoint.clone(),
                    message: message.unwrap_or_else(|| {
                        "native dynamic provider reported unsupported operation".to_owned()
                    }),
                    code: None,
                    retryable: None,
                    retry_after_ms: None,
                })
            }
            ProviderInvocationResult::Error {
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
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn execute_raw_provider_stream(
        &self,
        runtime_key: &str,
        base_url: impl Into<String>,
        api_key: &str,
        operation: &str,
        path_params: Vec<String>,
        body: Value,
        headers: HashMap<String, String>,
    ) -> Result<Option<ProviderStreamOutput>, ExtensionHostError> {
        let Some(runtime) = self.resolve_raw_native_dynamic_runtime(runtime_key, operation)? else {
            return Ok(None);
        };
        let invocation = ProviderInvocation::new(
            operation,
            api_key,
            base_url,
            path_params,
            body,
            true,
        )
        .with_headers(headers);
        Ok(Some(
            execute_native_dynamic_stream_invocation(runtime, &invocation).await?,
        ))
    }

    pub fn resolve_provider(
        &self,
        runtime_key: &str,
        base_url: impl Into<String>,
    ) -> Option<Box<dyn ProviderExecutionAdapter>> {
        let base_url = base_url.into();
        self.provider_factories
            .get(runtime_key)
            .or_else(|| {
                self.provider_aliases
                    .get(runtime_key)
                    .and_then(|extension_id| self.provider_factories.get(extension_id))
            })
            .map(|factory| factory(base_url))
    }

    pub fn can_resolve_provider(&self, runtime_key: &str) -> bool {
        self.provider_factories.contains_key(runtime_key)
            || self.provider_aliases.contains_key(runtime_key)
    }
}

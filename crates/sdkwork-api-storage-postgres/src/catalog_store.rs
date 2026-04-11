use super::*;

impl PostgresAdminStore {
    pub async fn insert_channel(&self, channel: &Channel) -> Result<Channel> {
        let now = current_timestamp_ms();
        sqlx::query(
            "INSERT INTO ai_channel (channel_id, channel_name, created_at_ms, updated_at_ms)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT(channel_id) DO UPDATE SET
                channel_name = excluded.channel_name,
                updated_at_ms = excluded.updated_at_ms,
                is_active = TRUE",
        )
        .bind(&channel.id)
        .bind(&channel.name)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;
        Ok(channel.clone())
    }

    pub async fn list_channels(&self) -> Result<Vec<Channel>> {
        let rows = sqlx::query_as::<_, (String, String)>(
            "SELECT channel_id, channel_name
             FROM ai_channel
             WHERE is_active = TRUE
             ORDER BY sort_order, channel_id",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|(id, name)| Channel { id, name })
            .collect())
    }

    pub async fn delete_channel(&self, channel_id: &str) -> Result<bool> {
        sqlx::query("DELETE FROM ai_proxy_provider_channel WHERE channel_id = $1")
            .bind(channel_id)
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM ai_proxy_provider_model WHERE channel_id = $1")
            .bind(channel_id)
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM ai_model_price WHERE channel_id = $1")
            .bind(channel_id)
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM ai_model WHERE channel_id = $1")
            .bind(channel_id)
            .execute(&self.pool)
            .await?;
        let result = sqlx::query("DELETE FROM ai_channel WHERE channel_id = $1")
            .bind(channel_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn insert_provider(&self, provider: &ProxyProvider) -> Result<ProxyProvider> {
        let now = current_timestamp_ms();
        sqlx::query(
            "INSERT INTO ai_proxy_provider (
                proxy_provider_id,
                primary_channel_id,
                extension_id,
                adapter_kind,
                protocol_kind,
                base_url,
                display_name,
                is_active,
                created_at_ms,
                updated_at_ms
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, TRUE, $8, $9)
             ON CONFLICT(proxy_provider_id) DO UPDATE SET
                primary_channel_id = excluded.primary_channel_id,
                extension_id = excluded.extension_id,
                adapter_kind = excluded.adapter_kind,
                protocol_kind = excluded.protocol_kind,
                base_url = excluded.base_url,
                display_name = excluded.display_name,
                is_active = TRUE,
                updated_at_ms = excluded.updated_at_ms",
        )
        .bind(&provider.id)
        .bind(&provider.channel_id)
        .bind(&provider.extension_id)
        .bind(&provider.adapter_kind)
        .bind(provider.protocol_kind())
        .bind(&provider.base_url)
        .bind(&provider.display_name)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;
        sqlx::query("DELETE FROM ai_proxy_provider_channel WHERE proxy_provider_id = $1")
            .bind(&provider.id)
            .execute(&self.pool)
            .await?;

        for binding in provider_channel_bindings(provider) {
            sqlx::query(
                "INSERT INTO ai_proxy_provider_channel (
                    proxy_provider_id,
                    channel_id,
                    is_primary,
                    created_at_ms,
                    updated_at_ms
                ) VALUES ($1, $2, $3, $4, $5)
                 ON CONFLICT(proxy_provider_id, channel_id) DO UPDATE SET
                    is_primary = excluded.is_primary,
                    updated_at_ms = excluded.updated_at_ms",
            )
            .bind(&binding.provider_id)
            .bind(&binding.channel_id)
            .bind(binding.is_primary)
            .bind(now)
            .bind(now)
            .execute(&self.pool)
            .await?;
        }
        Ok(provider.clone())
    }

    pub async fn list_providers(&self) -> Result<Vec<ProxyProvider>> {
        let rows = sqlx::query_as::<_, (String, String, String, String, String, String, String)>(
            "SELECT proxy_provider_id, primary_channel_id, extension_id, adapter_kind, protocol_kind, base_url, display_name
             FROM ai_proxy_provider
             WHERE is_active = TRUE
             ORDER BY proxy_provider_id",
        )
        .fetch_all(&self.pool)
        .await?;
        let provider_keys = rows
            .iter()
            .map(|(id, channel_id, _, _, _, _, _)| (id.clone(), channel_id.clone()))
            .collect::<Vec<_>>();
        let bindings_by_provider =
            load_provider_channel_bindings_for_providers(&self.pool, &provider_keys).await?;
        let mut providers = Vec::with_capacity(rows.len());
        for (id, channel_id, extension_id, adapter_kind, protocol_kind, base_url, display_name) in
            rows
        {
            let protocol_kind =
                normalize_provider_protocol_kind(protocol_kind, &adapter_kind);
            let channel_bindings = bindings_by_provider.get(&id).cloned().unwrap_or_else(|| {
                vec![ProviderChannelBinding::primary(
                    id.clone(),
                    channel_id.clone(),
                )]
            });
            providers.push(ProxyProvider {
                id,
                channel_id,
                extension_id: normalize_provider_extension_id(extension_id, &adapter_kind),
                adapter_kind,
                protocol_kind,
                base_url,
                display_name,
                channel_bindings,
            });
        }
        Ok(providers)
    }

    pub async fn list_providers_for_model(&self, model: &str) -> Result<Vec<ProxyProvider>> {
        let rows = sqlx::query_as::<_, (String, String, String, String, String, String, String)>(
            "SELECT DISTINCT providers.proxy_provider_id, providers.primary_channel_id, providers.extension_id, providers.adapter_kind, providers.protocol_kind, providers.base_url, providers.display_name
             FROM ai_proxy_provider_model provider_models
             INNER JOIN ai_proxy_provider providers
                 ON providers.proxy_provider_id = provider_models.proxy_provider_id
             WHERE provider_models.model_id = $1
               AND provider_models.is_active = TRUE
               AND providers.is_active = TRUE
             ORDER BY providers.proxy_provider_id",
        )
        .bind(model)
        .fetch_all(&self.pool)
        .await?;
        let provider_keys = rows
            .iter()
            .map(|(id, channel_id, _, _, _, _, _)| (id.clone(), channel_id.clone()))
            .collect::<Vec<_>>();
        let bindings_by_provider =
            load_provider_channel_bindings_for_providers(&self.pool, &provider_keys).await?;
        let mut providers = Vec::with_capacity(rows.len());
        for (id, channel_id, extension_id, adapter_kind, protocol_kind, base_url, display_name) in
            rows
        {
            let protocol_kind =
                normalize_provider_protocol_kind(protocol_kind, &adapter_kind);
            let channel_bindings = bindings_by_provider.get(&id).cloned().unwrap_or_else(|| {
                vec![ProviderChannelBinding::primary(
                    id.clone(),
                    channel_id.clone(),
                )]
            });
            providers.push(ProxyProvider {
                id,
                channel_id,
                extension_id: normalize_provider_extension_id(extension_id, &adapter_kind),
                adapter_kind,
                protocol_kind,
                base_url,
                display_name,
                channel_bindings,
            });
        }
        Ok(providers)
    }

    pub async fn find_provider(&self, provider_id: &str) -> Result<Option<ProxyProvider>> {
        let row = sqlx::query_as::<_, (String, String, String, String, String, String, String)>(
            "SELECT proxy_provider_id, primary_channel_id, extension_id, adapter_kind, protocol_kind, base_url, display_name
             FROM ai_proxy_provider
             WHERE proxy_provider_id = $1",
        )
        .bind(provider_id)
        .fetch_optional(&self.pool)
        .await?;

        let Some((id, channel_id, extension_id, adapter_kind, protocol_kind, base_url, display_name)) =
            row
        else {
            return Ok(None);
        };

        let channel_bindings = load_provider_channel_bindings(&self.pool, &id, &channel_id).await?;
        let protocol_kind = normalize_provider_protocol_kind(protocol_kind, &adapter_kind);

        Ok(Some(ProxyProvider {
            id,
            channel_id,
            extension_id: normalize_provider_extension_id(extension_id, &adapter_kind),
            adapter_kind,
            protocol_kind,
            base_url,
            display_name,
            channel_bindings,
        }))
    }

    pub async fn delete_provider(&self, provider_id: &str) -> Result<bool> {
        sqlx::query("DELETE FROM ai_router_credential_records WHERE proxy_provider_id = $1")
            .bind(provider_id)
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM ai_provider_account WHERE provider_id = $1")
            .bind(provider_id)
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM ai_proxy_provider_model WHERE proxy_provider_id = $1")
            .bind(provider_id)
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM ai_model_price WHERE proxy_provider_id = $1")
            .bind(provider_id)
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM ai_routing_policy_providers WHERE provider_id = $1")
            .bind(provider_id)
            .execute(&self.pool)
            .await?;
        sqlx::query(
            "UPDATE ai_routing_policies SET default_provider_id = NULL WHERE default_provider_id = $1",
        )
        .bind(provider_id)
        .execute(&self.pool)
        .await?;
        sqlx::query("DELETE FROM ai_proxy_provider_channel WHERE proxy_provider_id = $1")
            .bind(provider_id)
            .execute(&self.pool)
            .await?;
        let result = sqlx::query("DELETE FROM ai_proxy_provider WHERE proxy_provider_id = $1")
            .bind(provider_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn upsert_provider_account(
        &self,
        record: &ProviderAccountRecord,
    ) -> Result<ProviderAccountRecord> {
        let now = current_timestamp_ms();
        sqlx::query(
            "INSERT INTO ai_provider_account (
                provider_account_id,
                provider_id,
                display_name,
                account_kind,
                owner_scope,
                owner_tenant_id,
                execution_instance_id,
                base_url_override,
                region,
                priority,
                weight,
                enabled,
                routing_tags_json,
                health_score_hint,
                latency_ms_hint,
                cost_hint,
                success_rate_hint,
                throughput_hint,
                max_concurrency,
                daily_budget,
                notes,
                created_at_ms,
                updated_at_ms
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23)
             ON CONFLICT(provider_account_id) DO UPDATE SET
                provider_id = excluded.provider_id,
                display_name = excluded.display_name,
                account_kind = excluded.account_kind,
                owner_scope = excluded.owner_scope,
                owner_tenant_id = excluded.owner_tenant_id,
                execution_instance_id = excluded.execution_instance_id,
                base_url_override = excluded.base_url_override,
                region = excluded.region,
                priority = excluded.priority,
                weight = excluded.weight,
                enabled = excluded.enabled,
                routing_tags_json = excluded.routing_tags_json,
                health_score_hint = excluded.health_score_hint,
                latency_ms_hint = excluded.latency_ms_hint,
                cost_hint = excluded.cost_hint,
                success_rate_hint = excluded.success_rate_hint,
                throughput_hint = excluded.throughput_hint,
                max_concurrency = excluded.max_concurrency,
                daily_budget = excluded.daily_budget,
                notes = excluded.notes,
                updated_at_ms = excluded.updated_at_ms",
        )
        .bind(&record.provider_account_id)
        .bind(&record.provider_id)
        .bind(&record.display_name)
        .bind(&record.account_kind)
        .bind(&record.owner_scope)
        .bind(record.owner_tenant_id.clone())
        .bind(&record.execution_instance_id)
        .bind(record.base_url_override.clone())
        .bind(record.region.clone())
        .bind(record.priority)
        .bind(i32::try_from(record.weight)?)
        .bind(record.enabled)
        .bind(encode_string_list(&record.routing_tags)?)
        .bind(record.health_score_hint)
        .bind(record.latency_ms_hint.map(i64::try_from).transpose()?)
        .bind(record.cost_hint)
        .bind(record.success_rate_hint)
        .bind(record.throughput_hint)
        .bind(record.max_concurrency.map(i64::from))
        .bind(record.daily_budget)
        .bind(record.notes.clone())
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    pub async fn list_provider_accounts(&self) -> Result<Vec<ProviderAccountRecord>> {
        let rows = sqlx::query(
            "SELECT
                provider_account_id,
                provider_id,
                display_name,
                account_kind,
                owner_scope,
                owner_tenant_id,
                execution_instance_id,
                base_url_override,
                region,
                priority,
                weight,
                enabled,
                routing_tags_json,
                health_score_hint,
                latency_ms_hint,
                cost_hint,
                success_rate_hint,
                throughput_hint,
                max_concurrency,
                daily_budget,
                notes
             FROM ai_provider_account
             ORDER BY provider_id, priority DESC, provider_account_id",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(decode_provider_account_pg_row).collect()
    }

    pub async fn delete_provider_account(&self, provider_account_id: &str) -> Result<bool> {
        let result = sqlx::query(
            "DELETE FROM ai_provider_account WHERE provider_account_id = $1",
        )
        .bind(provider_account_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn insert_credential(
        &self,
        credential: &UpstreamCredential,
    ) -> Result<UpstreamCredential> {
        let now = current_timestamp_ms();
        sqlx::query(
            "INSERT INTO ai_router_credential_records (
                tenant_id,
                proxy_provider_id,
                key_reference,
                secret_backend,
                secret_local_file,
                secret_keyring_service,
                secret_master_key_id,
                secret_ciphertext,
                secret_key_version,
                created_at_ms,
                updated_at_ms
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, NULL, NULL, $8, $9)
             ON CONFLICT(tenant_id, proxy_provider_id, key_reference) DO UPDATE SET
                secret_backend = excluded.secret_backend,
                secret_local_file = excluded.secret_local_file,
                secret_keyring_service = excluded.secret_keyring_service,
                secret_master_key_id = excluded.secret_master_key_id,
                secret_ciphertext = NULL,
                secret_key_version = NULL,
                updated_at_ms = excluded.updated_at_ms",
        )
        .bind(&credential.tenant_id)
        .bind(&credential.provider_id)
        .bind(&credential.key_reference)
        .bind(&credential.secret_backend)
        .bind(&credential.secret_local_file)
        .bind(&credential.secret_keyring_service)
        .bind(&credential.secret_master_key_id)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;
        Ok(credential.clone())
    }

    pub async fn insert_encrypted_credential(
        &self,
        credential: &UpstreamCredential,
        envelope: &SecretEnvelope,
    ) -> Result<UpstreamCredential> {
        let now = current_timestamp_ms();
        sqlx::query(
            "INSERT INTO ai_router_credential_records (
                tenant_id,
                proxy_provider_id,
                key_reference,
                secret_backend,
                secret_local_file,
                secret_keyring_service,
                secret_master_key_id,
                secret_ciphertext,
                secret_key_version,
                created_at_ms,
                updated_at_ms
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
             ON CONFLICT(tenant_id, proxy_provider_id, key_reference) DO UPDATE SET
                secret_backend = excluded.secret_backend,
                secret_local_file = excluded.secret_local_file,
                secret_keyring_service = excluded.secret_keyring_service,
                secret_master_key_id = excluded.secret_master_key_id,
                secret_ciphertext = excluded.secret_ciphertext,
                secret_key_version = excluded.secret_key_version,
                updated_at_ms = excluded.updated_at_ms",
        )
        .bind(&credential.tenant_id)
        .bind(&credential.provider_id)
        .bind(&credential.key_reference)
        .bind(&credential.secret_backend)
        .bind(&credential.secret_local_file)
        .bind(&credential.secret_keyring_service)
        .bind(&credential.secret_master_key_id)
        .bind(&envelope.ciphertext)
        .bind(i32::try_from(envelope.key_version)?)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;
        Ok(credential.clone())
    }

    pub async fn list_credentials(&self) -> Result<Vec<UpstreamCredential>> {
        let rows = sqlx::query_as::<_, CredentialRow>(
            "SELECT tenant_id, proxy_provider_id, key_reference, secret_backend, secret_local_file, secret_keyring_service, secret_master_key_id
             FROM ai_router_credential_records
             ORDER BY proxy_provider_id, tenant_id, updated_at_ms DESC, created_at_ms DESC",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(decode_credential_row).collect())
    }

    pub async fn list_credentials_for_tenant(
        &self,
        tenant_id: &str,
    ) -> Result<Vec<UpstreamCredential>> {
        let rows = sqlx::query_as::<_, CredentialRow>(
            "SELECT tenant_id, proxy_provider_id, key_reference, secret_backend, secret_local_file, secret_keyring_service, secret_master_key_id
             FROM ai_router_credential_records
             WHERE tenant_id = $1
             ORDER BY updated_at_ms DESC, proxy_provider_id, key_reference",
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(decode_credential_row).collect())
    }

    pub async fn list_credentials_for_provider(
        &self,
        provider_id: &str,
    ) -> Result<Vec<UpstreamCredential>> {
        let rows = sqlx::query_as::<_, CredentialRow>(
            "SELECT tenant_id, proxy_provider_id, key_reference, secret_backend, secret_local_file, secret_keyring_service, secret_master_key_id
             FROM ai_router_credential_records
             WHERE proxy_provider_id = $1
             ORDER BY updated_at_ms DESC, tenant_id, key_reference",
        )
        .bind(provider_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(decode_credential_row).collect())
    }

    pub async fn find_credential(
        &self,
        tenant_id: &str,
        provider_id: &str,
        key_reference: &str,
    ) -> Result<Option<UpstreamCredential>> {
        let row = sqlx::query_as::<_, CredentialRow>(
            "SELECT tenant_id, proxy_provider_id, key_reference, secret_backend, secret_local_file, secret_keyring_service, secret_master_key_id
             FROM ai_router_credential_records
             WHERE tenant_id = $1 AND proxy_provider_id = $2 AND key_reference = $3",
        )
        .bind(tenant_id)
        .bind(provider_id)
        .bind(key_reference)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(decode_credential_row))
    }

    pub async fn find_credential_envelope(
        &self,
        tenant_id: &str,
        provider_id: &str,
        key_reference: &str,
    ) -> Result<Option<SecretEnvelope>> {
        let row = sqlx::query_as::<_, (Option<String>, Option<i32>)>(
            "SELECT secret_ciphertext, secret_key_version
             FROM ai_router_credential_records
             WHERE tenant_id = $1 AND proxy_provider_id = $2 AND key_reference = $3",
        )
        .bind(tenant_id)
        .bind(provider_id)
        .bind(key_reference)
        .fetch_optional(&self.pool)
        .await?;

        let Some((Some(ciphertext), Some(key_version))) = row else {
            return Ok(None);
        };

        Ok(Some(SecretEnvelope {
            ciphertext,
            key_version: u32::try_from(key_version)?,
        }))
    }

    pub async fn find_provider_credential(
        &self,
        tenant_id: &str,
        provider_id: &str,
    ) -> Result<Option<UpstreamCredential>> {
        let row = sqlx::query_as::<_, CredentialRow>(
            "SELECT tenant_id, proxy_provider_id, key_reference, secret_backend, secret_local_file, secret_keyring_service, secret_master_key_id
             FROM ai_router_credential_records
             WHERE tenant_id = $1 AND proxy_provider_id = $2
             ORDER BY updated_at_ms DESC, created_at_ms DESC
             LIMIT 1",
        )
        .bind(tenant_id)
        .bind(provider_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(decode_credential_row))
    }

    pub async fn delete_credential(
        &self,
        tenant_id: &str,
        provider_id: &str,
        key_reference: &str,
    ) -> Result<bool> {
        let result = sqlx::query(
            "DELETE FROM ai_router_credential_records
             WHERE tenant_id = $1 AND proxy_provider_id = $2 AND key_reference = $3",
        )
        .bind(tenant_id)
        .bind(provider_id)
        .bind(key_reference)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn upsert_official_provider_config(
        &self,
        config: &OfficialProviderConfig,
    ) -> Result<OfficialProviderConfig> {
        let now = current_timestamp_ms();
        sqlx::query(
            "INSERT INTO ai_official_provider_configs (
                provider_id,
                key_reference,
                base_url,
                enabled,
                created_at_ms,
                updated_at_ms
            ) VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT(provider_id) DO UPDATE SET
                key_reference = excluded.key_reference,
                base_url = excluded.base_url,
                enabled = excluded.enabled,
                updated_at_ms = excluded.updated_at_ms",
        )
        .bind(&config.provider_id)
        .bind(&config.key_reference)
        .bind(&config.base_url)
        .bind(config.enabled)
        .bind(i64::try_from(now)?)
        .bind(i64::try_from(now)?)
        .execute(&self.pool)
        .await?;

        Ok(OfficialProviderConfig {
            provider_id: config.provider_id.clone(),
            key_reference: config.key_reference.clone(),
            base_url: config.base_url.clone(),
            enabled: config.enabled,
            created_at_ms: u64::try_from(now)?,
            updated_at_ms: u64::try_from(now)?,
        })
    }

    pub async fn list_official_provider_configs(&self) -> Result<Vec<OfficialProviderConfig>> {
        let rows = sqlx::query_as::<_, (String, String, String, bool, i64, i64)>(
            "SELECT provider_id, key_reference, base_url, enabled, created_at_ms, updated_at_ms
             FROM ai_official_provider_configs
             ORDER BY provider_id",
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(
                |(provider_id, key_reference, base_url, enabled, created_at_ms, updated_at_ms)| {
                    Ok(OfficialProviderConfig {
                        provider_id,
                        key_reference,
                        base_url,
                        enabled,
                        created_at_ms: u64::try_from(created_at_ms)?,
                        updated_at_ms: u64::try_from(updated_at_ms)?,
                    })
                },
            )
            .collect()
    }

    pub async fn find_official_provider_config(
        &self,
        provider_id: &str,
    ) -> Result<Option<OfficialProviderConfig>> {
        let row = sqlx::query_as::<_, (String, String, String, bool, i64, i64)>(
            "SELECT provider_id, key_reference, base_url, enabled, created_at_ms, updated_at_ms
             FROM ai_official_provider_configs
             WHERE provider_id = $1",
        )
        .bind(provider_id)
        .fetch_optional(&self.pool)
        .await?;

        let Some((provider_id, key_reference, base_url, enabled, created_at_ms, updated_at_ms)) =
            row
        else {
            return Ok(None);
        };

        Ok(Some(OfficialProviderConfig {
            provider_id,
            key_reference,
            base_url,
            enabled,
            created_at_ms: u64::try_from(created_at_ms)?,
            updated_at_ms: u64::try_from(updated_at_ms)?,
        }))
    }

    pub async fn insert_model(&self, model: &ModelCatalogEntry) -> Result<ModelCatalogEntry> {
        let provider = self
            .find_provider(&model.provider_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("provider_id is not registered"))?;
        let mut channel_model = ChannelModelRecord::new(
            &provider.channel_id,
            &model.external_name,
            &model.external_name,
        )
        .with_context_window_option(model.context_window)
        .with_streaming(model.streaming);
        for capability in &model.capabilities {
            channel_model = channel_model.with_capability(capability.clone());
        }
        self.insert_channel_model(&channel_model).await?;
        let mut provider_model =
            ProviderModelRecord::new(&model.provider_id, &provider.channel_id, &model.external_name)
                .with_provider_model_id(&model.external_name)
                .with_streaming(model.streaming)
                .with_context_window_option(model.context_window);
        for capability in &model.capabilities {
            provider_model = provider_model.with_capability(capability.clone());
        }
        self.upsert_provider_model(&provider_model).await?;
        self.insert_model_price(&ModelPriceRecord::new(
            &provider.channel_id,
            &model.external_name,
            &model.provider_id,
        ))
        .await?;
        Ok(model.clone())
    }

    pub async fn list_models(&self) -> Result<Vec<ModelCatalogEntry>> {
        let rows = sqlx::query_as::<_, (String, String, String, bool, Option<i64>)>(
            "SELECT
                provider_models.model_id,
                provider_models.proxy_provider_id,
                CASE
                    WHEN provider_models.capabilities_json = '[]' THEN models.capabilities_json
                    ELSE provider_models.capabilities_json
                END,
                CASE
                    WHEN provider_models.streaming_enabled THEN provider_models.streaming_enabled
                    ELSE models.streaming_enabled
                END,
                COALESCE(provider_models.context_window, models.context_window)
             FROM ai_proxy_provider_model provider_models
             INNER JOIN ai_model models
                 ON models.channel_id = provider_models.channel_id
                AND models.model_id = provider_models.model_id
             WHERE provider_models.is_active = TRUE
             UNION ALL
             SELECT
                models.model_id,
                prices.proxy_provider_id,
                models.capabilities_json,
                models.streaming_enabled,
                models.context_window
             FROM ai_model models
             INNER JOIN ai_model_price prices
                 ON prices.channel_id = models.channel_id
                AND prices.model_id = models.model_id
             WHERE prices.is_active = TRUE
               AND NOT EXISTS (
                   SELECT 1
                   FROM ai_proxy_provider_model provider_models
                   WHERE provider_models.proxy_provider_id = prices.proxy_provider_id
                     AND provider_models.channel_id = models.channel_id
                     AND provider_models.model_id = models.model_id
                     AND provider_models.is_active = TRUE
               )
             ORDER BY 1, 2",
        )
        .fetch_all(&self.pool)
        .await?;
        let mut models = Vec::with_capacity(rows.len());
        for (external_name, provider_id, capabilities, streaming, context_window) in rows {
            models.push(ModelCatalogEntry {
                external_name,
                provider_id,
                capabilities: decode_model_capabilities(&capabilities)?,
                streaming,
                context_window: context_window.map(u64::try_from).transpose()?,
            });
        }
        Ok(models)
    }

    pub async fn list_models_for_external_name(
        &self,
        external_name: &str,
    ) -> Result<Vec<ModelCatalogEntry>> {
        let rows = sqlx::query_as::<_, (String, String, String, bool, Option<i64>)>(
            "SELECT
                provider_models.model_id,
                provider_models.proxy_provider_id,
                CASE
                    WHEN provider_models.capabilities_json = '[]' THEN models.capabilities_json
                    ELSE provider_models.capabilities_json
                END,
                CASE
                    WHEN provider_models.streaming_enabled THEN provider_models.streaming_enabled
                    ELSE models.streaming_enabled
                END,
                COALESCE(provider_models.context_window, models.context_window)
             FROM ai_proxy_provider_model provider_models
             INNER JOIN ai_model models
                 ON models.channel_id = provider_models.channel_id
                AND models.model_id = provider_models.model_id
             WHERE provider_models.model_id = $1
               AND provider_models.is_active = TRUE
             UNION ALL
             SELECT
                models.model_id,
                prices.proxy_provider_id,
                models.capabilities_json,
                models.streaming_enabled,
                models.context_window
             FROM ai_model models
             INNER JOIN ai_model_price prices
                 ON prices.channel_id = models.channel_id
                AND prices.model_id = models.model_id
             WHERE models.model_id = $2
               AND prices.is_active = TRUE
               AND NOT EXISTS (
                   SELECT 1
                   FROM ai_proxy_provider_model provider_models
                   WHERE provider_models.proxy_provider_id = prices.proxy_provider_id
                     AND provider_models.channel_id = models.channel_id
                     AND provider_models.model_id = models.model_id
                     AND provider_models.is_active = TRUE
               )
             ORDER BY 2",
        )
        .bind(external_name)
        .bind(external_name)
        .fetch_all(&self.pool)
        .await?;

        let mut models = Vec::with_capacity(rows.len());
        for (external_name, provider_id, capabilities, streaming, context_window) in rows {
            models.push(ModelCatalogEntry {
                external_name,
                provider_id,
                capabilities: decode_model_capabilities(&capabilities)?,
                streaming,
                context_window: context_window.map(u64::try_from).transpose()?,
            });
        }
        Ok(models)
    }

    pub async fn find_any_model(&self) -> Result<Option<ModelCatalogEntry>> {
        let row = sqlx::query_as::<_, (String, String, String, bool, Option<i64>)>(
            "SELECT
                provider_models.model_id,
                provider_models.proxy_provider_id,
                CASE
                    WHEN provider_models.capabilities_json = '[]' THEN models.capabilities_json
                    ELSE provider_models.capabilities_json
                END,
                CASE
                    WHEN provider_models.streaming_enabled THEN provider_models.streaming_enabled
                    ELSE models.streaming_enabled
                END,
                COALESCE(provider_models.context_window, models.context_window)
             FROM ai_proxy_provider_model provider_models
             INNER JOIN ai_model models
                 ON models.channel_id = provider_models.channel_id
                AND models.model_id = provider_models.model_id
             WHERE provider_models.is_active = TRUE
             UNION ALL
             SELECT
                models.model_id,
                prices.proxy_provider_id,
                models.capabilities_json,
                models.streaming_enabled,
                models.context_window
             FROM ai_model models
             INNER JOIN ai_model_price prices
                 ON prices.channel_id = models.channel_id
                AND prices.model_id = models.model_id
             WHERE prices.is_active = TRUE
               AND NOT EXISTS (
                   SELECT 1
                   FROM ai_proxy_provider_model provider_models
                   WHERE provider_models.proxy_provider_id = prices.proxy_provider_id
                     AND provider_models.channel_id = models.channel_id
                     AND provider_models.model_id = models.model_id
                     AND provider_models.is_active = TRUE
               )
             ORDER BY 1, 2
             LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(match row {
            Some((external_name, provider_id, capabilities, streaming, context_window)) => {
                Some(ModelCatalogEntry {
                    external_name,
                    provider_id,
                    capabilities: decode_model_capabilities(&capabilities)?,
                    streaming,
                    context_window: context_window.map(u64::try_from).transpose()?,
                })
            }
            None => None,
        })
    }

    pub async fn find_model(&self, external_name: &str) -> Result<Option<ModelCatalogEntry>> {
        let row = sqlx::query_as::<_, (String, String, String, bool, Option<i64>)>(
            "SELECT
                provider_models.model_id,
                provider_models.proxy_provider_id,
                CASE
                    WHEN provider_models.capabilities_json = '[]' THEN models.capabilities_json
                    ELSE provider_models.capabilities_json
                END,
                CASE
                    WHEN provider_models.streaming_enabled THEN provider_models.streaming_enabled
                    ELSE models.streaming_enabled
                END,
                COALESCE(provider_models.context_window, models.context_window)
             FROM ai_proxy_provider_model provider_models
             INNER JOIN ai_model models
                 ON models.channel_id = provider_models.channel_id
                AND models.model_id = provider_models.model_id
             WHERE provider_models.model_id = $1
               AND provider_models.is_active = TRUE
             UNION ALL
             SELECT
                models.model_id,
                prices.proxy_provider_id,
                models.capabilities_json,
                models.streaming_enabled,
                models.context_window
             FROM ai_model models
             INNER JOIN ai_model_price prices
                 ON prices.channel_id = models.channel_id
                AND prices.model_id = models.model_id
             WHERE models.model_id = $2
               AND prices.is_active = TRUE
               AND NOT EXISTS (
                   SELECT 1
                   FROM ai_proxy_provider_model provider_models
                   WHERE provider_models.proxy_provider_id = prices.proxy_provider_id
                     AND provider_models.channel_id = models.channel_id
                     AND provider_models.model_id = models.model_id
                     AND provider_models.is_active = TRUE
               )
             ORDER BY 2
             LIMIT 1",
        )
        .bind(external_name)
        .bind(external_name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(match row {
            Some((external_name, provider_id, capabilities, streaming, context_window)) => {
                Some(ModelCatalogEntry {
                    external_name,
                    provider_id,
                    capabilities: decode_model_capabilities(&capabilities)?,
                    streaming,
                    context_window: context_window.map(u64::try_from).transpose()?,
                })
            }
            None => None,
        })
    }

    pub async fn delete_model(&self, external_name: &str) -> Result<bool> {
        sqlx::query("DELETE FROM ai_model_price WHERE model_id = $1")
            .bind(external_name)
            .execute(&self.pool)
            .await?;
        let result = sqlx::query("DELETE FROM ai_model WHERE model_id = $1")
            .bind(external_name)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn delete_model_variant(
        &self,
        external_name: &str,
        provider_id: &str,
    ) -> Result<bool> {
        sqlx::query(
            "DELETE FROM ai_proxy_provider_model WHERE model_id = $1 AND proxy_provider_id = $2",
        )
        .bind(external_name)
        .bind(provider_id)
        .execute(&self.pool)
        .await?;
        let result = sqlx::query(
            "DELETE FROM ai_model_price WHERE model_id = $1 AND proxy_provider_id = $2",
        )
        .bind(external_name)
        .bind(provider_id)
        .execute(&self.pool)
        .await?;
        sqlx::query(
            "DELETE FROM ai_model
             WHERE model_id = $1
               AND NOT EXISTS (
                   SELECT 1
                   FROM ai_model_price prices
                   WHERE prices.channel_id = ai_model.channel_id
                     AND prices.model_id = ai_model.model_id
               )",
        )
        .bind(external_name)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn insert_channel_model(
        &self,
        record: &ChannelModelRecord,
    ) -> Result<ChannelModelRecord> {
        let now = current_timestamp_ms();
        sqlx::query(
            "INSERT INTO ai_model (
                channel_id,
                model_id,
                model_display_name,
                capabilities_json,
                streaming_enabled,
                context_window,
                description,
                created_at_ms,
                updated_at_ms
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
             ON CONFLICT(channel_id, model_id) DO UPDATE SET
                model_display_name = excluded.model_display_name,
                capabilities_json = excluded.capabilities_json,
                streaming_enabled = excluded.streaming_enabled,
                context_window = excluded.context_window,
                description = excluded.description,
                updated_at_ms = excluded.updated_at_ms",
        )
        .bind(&record.channel_id)
        .bind(&record.model_id)
        .bind(&record.model_display_name)
        .bind(encode_model_capabilities(&record.capabilities)?)
        .bind(record.streaming)
        .bind(record.context_window.map(i64::try_from).transpose()?)
        .bind(record.description.clone().unwrap_or_default())
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    pub async fn list_channel_models(&self) -> Result<Vec<ChannelModelRecord>> {
        let rows = sqlx::query_as::<_, ChannelModelRow>(
            "SELECT
                channel_id,
                model_id,
                model_display_name,
                capabilities_json,
                streaming_enabled,
                context_window,
                description
             FROM ai_model
             ORDER BY channel_id, model_id",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(decode_channel_model_row).collect()
    }

    pub async fn delete_channel_model(&self, channel_id: &str, model_id: &str) -> Result<bool> {
        sqlx::query("DELETE FROM ai_proxy_provider_model WHERE channel_id = $1 AND model_id = $2")
            .bind(channel_id)
            .bind(model_id)
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM ai_model_price WHERE channel_id = $1 AND model_id = $2")
            .bind(channel_id)
            .bind(model_id)
            .execute(&self.pool)
            .await?;
        let result = sqlx::query("DELETE FROM ai_model WHERE channel_id = $1 AND model_id = $2")
            .bind(channel_id)
            .bind(model_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn upsert_provider_model(
        &self,
        record: &ProviderModelRecord,
    ) -> Result<ProviderModelRecord> {
        let now = current_timestamp_ms();
        sqlx::query(
            "INSERT INTO ai_proxy_provider_model (
                proxy_provider_id,
                channel_id,
                model_id,
                provider_model_id,
                provider_model_family,
                capabilities_json,
                streaming_enabled,
                context_window,
                max_output_tokens,
                supports_prompt_caching,
                supports_reasoning_usage,
                supports_tool_usage_metrics,
                is_default_route,
                is_active,
                created_at_ms,
                updated_at_ms
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
             ON CONFLICT(proxy_provider_id, channel_id, model_id) DO UPDATE SET
                provider_model_id = excluded.provider_model_id,
                provider_model_family = excluded.provider_model_family,
                capabilities_json = excluded.capabilities_json,
                streaming_enabled = excluded.streaming_enabled,
                context_window = excluded.context_window,
                max_output_tokens = excluded.max_output_tokens,
                supports_prompt_caching = excluded.supports_prompt_caching,
                supports_reasoning_usage = excluded.supports_reasoning_usage,
                supports_tool_usage_metrics = excluded.supports_tool_usage_metrics,
                is_default_route = excluded.is_default_route,
                is_active = excluded.is_active,
                updated_at_ms = excluded.updated_at_ms",
        )
        .bind(&record.proxy_provider_id)
        .bind(&record.channel_id)
        .bind(&record.model_id)
        .bind(&record.provider_model_id)
        .bind(record.provider_model_family.clone())
        .bind(encode_model_capabilities(&record.capabilities)?)
        .bind(record.streaming)
        .bind(record.context_window.map(i64::try_from).transpose()?)
        .bind(record.max_output_tokens.map(i64::try_from).transpose()?)
        .bind(record.supports_prompt_caching)
        .bind(record.supports_reasoning_usage)
        .bind(record.supports_tool_usage_metrics)
        .bind(record.is_default_route)
        .bind(record.is_active)
        .bind(i64::try_from(now)?)
        .bind(i64::try_from(now)?)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    pub async fn list_provider_models(&self) -> Result<Vec<ProviderModelRecord>> {
        let rows = sqlx::query_as::<_, ProviderModelRow>(
            "SELECT
                proxy_provider_id,
                channel_id,
                model_id,
                provider_model_id,
                provider_model_family,
                capabilities_json,
                streaming_enabled,
                context_window,
                max_output_tokens,
                supports_prompt_caching,
                supports_reasoning_usage,
                supports_tool_usage_metrics,
                is_default_route,
                is_active
             FROM ai_proxy_provider_model
             ORDER BY proxy_provider_id, channel_id, model_id",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(decode_provider_model_row).collect()
    }

    pub async fn delete_provider_model(
        &self,
        proxy_provider_id: &str,
        channel_id: &str,
        model_id: &str,
    ) -> Result<bool> {
        let result = sqlx::query(
            "DELETE FROM ai_proxy_provider_model
             WHERE proxy_provider_id = $1 AND channel_id = $2 AND model_id = $3",
        )
        .bind(proxy_provider_id)
        .bind(channel_id)
        .bind(model_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn insert_model_price(&self, record: &ModelPriceRecord) -> Result<ModelPriceRecord> {
        let now = current_timestamp_ms();
        sqlx::query(
            "INSERT INTO ai_model_price (
                channel_id,
                model_id,
                proxy_provider_id,
                currency_code,
                price_unit,
                input_price,
                output_price,
                cache_read_price,
                cache_write_price,
                request_price,
                price_source_kind,
                billing_notes,
                pricing_tiers_json,
                is_active,
                created_at_ms,
                updated_at_ms
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
             ON CONFLICT(channel_id, model_id, proxy_provider_id) DO UPDATE SET
                currency_code = excluded.currency_code,
                price_unit = excluded.price_unit,
                input_price = excluded.input_price,
                output_price = excluded.output_price,
                cache_read_price = excluded.cache_read_price,
                cache_write_price = excluded.cache_write_price,
                request_price = excluded.request_price,
                price_source_kind = excluded.price_source_kind,
                billing_notes = excluded.billing_notes,
                pricing_tiers_json = excluded.pricing_tiers_json,
                is_active = excluded.is_active,
                updated_at_ms = excluded.updated_at_ms",
        )
        .bind(&record.channel_id)
        .bind(&record.model_id)
        .bind(&record.proxy_provider_id)
        .bind(&record.currency_code)
        .bind(&record.price_unit)
        .bind(record.input_price)
        .bind(record.output_price)
        .bind(record.cache_read_price)
        .bind(record.cache_write_price)
        .bind(record.request_price)
        .bind(&record.price_source_kind)
        .bind(record.billing_notes.clone())
        .bind(encode_model_price_tiers(&record.pricing_tiers)?)
        .bind(record.is_active)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    pub async fn list_model_prices(&self) -> Result<Vec<ModelPriceRecord>> {
        let rows = sqlx::query_as::<_, ModelPriceRow>(
            "SELECT
                channel_id,
                model_id,
                proxy_provider_id,
                currency_code,
                price_unit,
                input_price,
                output_price,
                cache_read_price,
                cache_write_price,
                request_price,
                price_source_kind,
                billing_notes,
                pricing_tiers_json,
                is_active
             FROM ai_model_price
             ORDER BY channel_id, model_id, proxy_provider_id",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(decode_model_price_row).collect())
    }

    pub async fn delete_model_price(
        &self,
        channel_id: &str,
        model_id: &str,
        proxy_provider_id: &str,
    ) -> Result<bool> {
        let result = sqlx::query(
            "DELETE FROM ai_model_price
             WHERE channel_id = $1 AND model_id = $2 AND proxy_provider_id = $3",
        )
        .bind(channel_id)
        .bind(model_id)
        .bind(proxy_provider_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }
}

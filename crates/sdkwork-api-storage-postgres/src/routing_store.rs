use super::*;

impl PostgresAdminStore {
    pub async fn insert_routing_policy(&self, policy: &RoutingPolicy) -> Result<RoutingPolicy> {
        sqlx::query(
            "INSERT INTO ai_routing_policies (policy_id, capability, model_pattern, enabled, priority, strategy, default_provider_id, max_cost, max_latency_ms, require_healthy) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
             ON CONFLICT(policy_id) DO UPDATE SET capability = excluded.capability, model_pattern = excluded.model_pattern, enabled = excluded.enabled, priority = excluded.priority, strategy = excluded.strategy, default_provider_id = excluded.default_provider_id, max_cost = excluded.max_cost, max_latency_ms = excluded.max_latency_ms, require_healthy = excluded.require_healthy",
        )
        .bind(&policy.policy_id)
        .bind(&policy.capability)
        .bind(&policy.model_pattern)
        .bind(policy.enabled)
        .bind(policy.priority)
        .bind(policy.strategy.as_str())
        .bind(&policy.default_provider_id)
        .bind(policy.max_cost)
        .bind(policy.max_latency_ms.map(i64::try_from).transpose()?)
        .bind(policy.require_healthy)
        .execute(&self.pool)
        .await?;

        sqlx::query("DELETE FROM ai_routing_policy_providers WHERE policy_id = $1")
            .bind(&policy.policy_id)
            .execute(&self.pool)
            .await?;

        for (position, provider_id) in policy.ordered_provider_ids.iter().enumerate() {
            sqlx::query(
                "INSERT INTO ai_routing_policy_providers (policy_id, provider_id, position) VALUES ($1, $2, $3)
                 ON CONFLICT(policy_id, provider_id) DO UPDATE SET position = excluded.position",
            )
            .bind(&policy.policy_id)
            .bind(provider_id)
            .bind(i32::try_from(position)?)
            .execute(&self.pool)
            .await?;
        }

        Ok(policy.clone())
    }

    pub async fn list_routing_policies(&self) -> Result<Vec<RoutingPolicy>> {
        let rows = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                bool,
                i32,
                String,
                Option<String>,
                Option<f64>,
                Option<i64>,
                bool,
            ),
        >(
            "SELECT policy_id, capability, model_pattern, enabled, priority, strategy, default_provider_id, max_cost, max_latency_ms, require_healthy
             FROM ai_routing_policies
             ORDER BY priority DESC, policy_id",
        )
        .fetch_all(&self.pool)
        .await?;

        let mut policies = Vec::with_capacity(rows.len());
        for (
            policy_id,
            capability,
            model_pattern,
            enabled,
            priority,
            strategy,
            default_provider_id,
            max_cost,
            max_latency_ms,
            require_healthy,
        ) in rows
        {
            policies.push(
                RoutingPolicy::new(policy_id.clone(), capability, model_pattern)
                    .with_enabled(enabled)
                    .with_priority(priority)
                    .with_strategy(
                        RoutingStrategy::from_str(&strategy)
                            .unwrap_or(RoutingStrategy::DeterministicPriority),
                    )
                    .with_ordered_provider_ids(
                        load_routing_policy_provider_ids(&self.pool, &policy_id).await?,
                    )
                    .with_default_provider_id_option(default_provider_id)
                    .with_max_cost_option(max_cost)
                    .with_max_latency_ms_option(max_latency_ms.map(u64::try_from).transpose()?)
                    .with_require_healthy(require_healthy),
            );
        }
        Ok(policies)
    }

    pub async fn insert_routing_profile(
        &self,
        profile: &RoutingProfileRecord,
    ) -> Result<RoutingProfileRecord> {
        sqlx::query(
            "INSERT INTO ai_routing_profiles (
                profile_id,
                tenant_id,
                project_id,
                name,
                slug,
                description,
                active,
                strategy,
                ordered_provider_ids_json,
                default_provider_id,
                max_cost,
                max_latency_ms,
                require_healthy,
                preferred_region,
                created_at_ms,
                updated_at_ms
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            ON CONFLICT(profile_id) DO UPDATE SET
                tenant_id = excluded.tenant_id,
                project_id = excluded.project_id,
                name = excluded.name,
                slug = excluded.slug,
                description = excluded.description,
                active = excluded.active,
                strategy = excluded.strategy,
                ordered_provider_ids_json = excluded.ordered_provider_ids_json,
                default_provider_id = excluded.default_provider_id,
                max_cost = excluded.max_cost,
                max_latency_ms = excluded.max_latency_ms,
                require_healthy = excluded.require_healthy,
                preferred_region = excluded.preferred_region,
                created_at_ms = excluded.created_at_ms,
                updated_at_ms = excluded.updated_at_ms",
        )
        .bind(&profile.profile_id)
        .bind(&profile.tenant_id)
        .bind(&profile.project_id)
        .bind(&profile.name)
        .bind(&profile.slug)
        .bind(&profile.description)
        .bind(profile.active)
        .bind(profile.strategy.as_str())
        .bind(encode_string_list(&profile.ordered_provider_ids)?)
        .bind(&profile.default_provider_id)
        .bind(profile.max_cost)
        .bind(profile.max_latency_ms.map(i64::try_from).transpose()?)
        .bind(profile.require_healthy)
        .bind(&profile.preferred_region)
        .bind(i64::try_from(profile.created_at_ms)?)
        .bind(i64::try_from(profile.updated_at_ms)?)
        .execute(&self.pool)
        .await?;

        Ok(profile.clone())
    }

    pub async fn list_routing_profiles(&self) -> Result<Vec<RoutingProfileRecord>> {
        let rows = sqlx::query(
            "SELECT profile_id, tenant_id, project_id, name, slug, description, active, strategy, ordered_provider_ids_json, default_provider_id, max_cost, max_latency_ms, require_healthy, preferred_region, created_at_ms, updated_at_ms
             FROM ai_routing_profiles
             ORDER BY updated_at_ms DESC, profile_id",
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(decode_routing_profile_row).collect()
    }

    pub async fn find_routing_profile(
        &self,
        profile_id: &str,
    ) -> Result<Option<RoutingProfileRecord>> {
        let row = sqlx::query(
            "SELECT profile_id, tenant_id, project_id, name, slug, description, active, strategy, ordered_provider_ids_json, default_provider_id, max_cost, max_latency_ms, require_healthy, preferred_region, created_at_ms, updated_at_ms
             FROM ai_routing_profiles
             WHERE profile_id = $1",
        )
        .bind(profile_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(decode_routing_profile_row).transpose()
    }

    pub async fn insert_compiled_routing_snapshot(
        &self,
        snapshot: &CompiledRoutingSnapshotRecord,
    ) -> Result<CompiledRoutingSnapshotRecord> {
        sqlx::query(
            "INSERT INTO ai_compiled_routing_snapshots (
                snapshot_id,
                tenant_id,
                project_id,
                api_key_group_id,
                capability,
                route_key,
                matched_policy_id,
                project_routing_preferences_project_id,
                applied_routing_profile_id,
                strategy,
                ordered_provider_ids_json,
                default_provider_id,
                max_cost,
                max_latency_ms,
                require_healthy,
                preferred_region,
                created_at_ms,
                updated_at_ms
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
            ON CONFLICT(snapshot_id) DO UPDATE SET
                tenant_id = excluded.tenant_id,
                project_id = excluded.project_id,
                api_key_group_id = excluded.api_key_group_id,
                capability = excluded.capability,
                route_key = excluded.route_key,
                matched_policy_id = excluded.matched_policy_id,
                project_routing_preferences_project_id = excluded.project_routing_preferences_project_id,
                applied_routing_profile_id = excluded.applied_routing_profile_id,
                strategy = excluded.strategy,
                ordered_provider_ids_json = excluded.ordered_provider_ids_json,
                default_provider_id = excluded.default_provider_id,
                max_cost = excluded.max_cost,
                max_latency_ms = excluded.max_latency_ms,
                require_healthy = excluded.require_healthy,
                preferred_region = excluded.preferred_region,
                created_at_ms = excluded.created_at_ms,
                updated_at_ms = excluded.updated_at_ms",
        )
        .bind(&snapshot.snapshot_id)
        .bind(&snapshot.tenant_id)
        .bind(&snapshot.project_id)
        .bind(&snapshot.api_key_group_id)
        .bind(&snapshot.capability)
        .bind(&snapshot.route_key)
        .bind(&snapshot.matched_policy_id)
        .bind(&snapshot.project_routing_preferences_project_id)
        .bind(&snapshot.applied_routing_profile_id)
        .bind(&snapshot.strategy)
        .bind(encode_string_list(&snapshot.ordered_provider_ids)?)
        .bind(&snapshot.default_provider_id)
        .bind(snapshot.max_cost)
        .bind(snapshot.max_latency_ms.map(i64::try_from).transpose()?)
        .bind(snapshot.require_healthy)
        .bind(&snapshot.preferred_region)
        .bind(i64::try_from(snapshot.created_at_ms)?)
        .bind(i64::try_from(snapshot.updated_at_ms)?)
        .execute(&self.pool)
        .await?;

        Ok(snapshot.clone())
    }

    pub async fn list_compiled_routing_snapshots(
        &self,
    ) -> Result<Vec<CompiledRoutingSnapshotRecord>> {
        let rows = sqlx::query(
            "SELECT snapshot_id, tenant_id, project_id, api_key_group_id, capability, route_key, matched_policy_id, project_routing_preferences_project_id, applied_routing_profile_id, strategy, ordered_provider_ids_json, default_provider_id, max_cost, max_latency_ms, require_healthy, preferred_region, created_at_ms, updated_at_ms
             FROM ai_compiled_routing_snapshots
             ORDER BY updated_at_ms DESC, snapshot_id",
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(decode_compiled_routing_snapshot_row)
            .collect()
    }

    pub async fn insert_project_routing_preferences(
        &self,
        preferences: &ProjectRoutingPreferences,
    ) -> Result<ProjectRoutingPreferences> {
        sqlx::query(
            "INSERT INTO ai_project_routing_preferences (
                project_id,
                preset_id,
                strategy,
                ordered_provider_ids_json,
                default_provider_id,
                max_cost,
                max_latency_ms,
                require_healthy,
                preferred_region,
                updated_at_ms
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT(project_id) DO UPDATE SET
                preset_id = excluded.preset_id,
                strategy = excluded.strategy,
                ordered_provider_ids_json = excluded.ordered_provider_ids_json,
                default_provider_id = excluded.default_provider_id,
                max_cost = excluded.max_cost,
                max_latency_ms = excluded.max_latency_ms,
                require_healthy = excluded.require_healthy,
                preferred_region = excluded.preferred_region,
                updated_at_ms = excluded.updated_at_ms",
        )
        .bind(&preferences.project_id)
        .bind(&preferences.preset_id)
        .bind(preferences.strategy.as_str())
        .bind(encode_string_list(&preferences.ordered_provider_ids)?)
        .bind(&preferences.default_provider_id)
        .bind(preferences.max_cost)
        .bind(preferences.max_latency_ms.map(i64::try_from).transpose()?)
        .bind(preferences.require_healthy)
        .bind(&preferences.preferred_region)
        .bind(i64::try_from(preferences.updated_at_ms)?)
        .execute(&self.pool)
        .await?;

        Ok(preferences.clone())
    }

    pub async fn find_project_routing_preferences(
        &self,
        project_id: &str,
    ) -> Result<Option<ProjectRoutingPreferences>> {
        let row = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                String,
                Option<String>,
                Option<f64>,
                Option<i64>,
                bool,
                Option<String>,
                i64,
            ),
        >(
            "SELECT project_id, preset_id, strategy, ordered_provider_ids_json, default_provider_id, max_cost, max_latency_ms, require_healthy, preferred_region, updated_at_ms
             FROM ai_project_routing_preferences
             WHERE project_id = $1",
        )
        .bind(project_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(
            |(
                project_id,
                preset_id,
                strategy,
                ordered_provider_ids_json,
                default_provider_id,
                max_cost,
                max_latency_ms,
                require_healthy,
                preferred_region,
                updated_at_ms,
            )| {
                Ok(ProjectRoutingPreferences::new(project_id)
                    .with_preset_id(preset_id)
                    .with_strategy(
                        RoutingStrategy::from_str(&strategy)
                            .unwrap_or(RoutingStrategy::DeterministicPriority),
                    )
                    .with_ordered_provider_ids(decode_string_list(&ordered_provider_ids_json)?)
                    .with_default_provider_id_option(default_provider_id)
                    .with_max_cost_option(max_cost)
                    .with_max_latency_ms_option(max_latency_ms.map(u64::try_from).transpose()?)
                    .with_require_healthy(require_healthy)
                    .with_preferred_region_option(preferred_region)
                    .with_updated_at_ms(u64::try_from(updated_at_ms)?))
            },
        )
        .transpose()
    }

    pub async fn insert_routing_decision_log(
        &self,
        log: &RoutingDecisionLog,
    ) -> Result<RoutingDecisionLog> {
        sqlx::query(
            "INSERT INTO ai_routing_decision_logs (decision_id, decision_source, tenant_id, project_id, api_key_group_id, capability, route_key, selected_provider_id, matched_policy_id, applied_routing_profile_id, compiled_routing_snapshot_id, strategy, selection_seed, selection_reason, fallback_reason, requested_region, slo_applied, slo_degraded, created_at_ms, assessments_json)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20)
             ON CONFLICT(decision_id) DO UPDATE SET decision_source = excluded.decision_source, tenant_id = excluded.tenant_id, project_id = excluded.project_id, api_key_group_id = excluded.api_key_group_id, capability = excluded.capability, route_key = excluded.route_key, selected_provider_id = excluded.selected_provider_id, matched_policy_id = excluded.matched_policy_id, applied_routing_profile_id = excluded.applied_routing_profile_id, compiled_routing_snapshot_id = excluded.compiled_routing_snapshot_id, strategy = excluded.strategy, selection_seed = excluded.selection_seed, selection_reason = excluded.selection_reason, fallback_reason = excluded.fallback_reason, requested_region = excluded.requested_region, slo_applied = excluded.slo_applied, slo_degraded = excluded.slo_degraded, created_at_ms = excluded.created_at_ms, assessments_json = excluded.assessments_json",
        )
        .bind(&log.decision_id)
        .bind(log.decision_source.as_str())
        .bind(&log.tenant_id)
        .bind(&log.project_id)
        .bind(&log.api_key_group_id)
        .bind(&log.capability)
        .bind(&log.route_key)
        .bind(&log.selected_provider_id)
        .bind(&log.matched_policy_id)
        .bind(&log.applied_routing_profile_id)
        .bind(&log.compiled_routing_snapshot_id)
        .bind(&log.strategy)
        .bind(log.selection_seed.map(i64::try_from).transpose()?)
        .bind(&log.selection_reason)
        .bind(&log.fallback_reason)
        .bind(&log.requested_region)
        .bind(log.slo_applied)
        .bind(log.slo_degraded)
        .bind(i64::try_from(log.created_at_ms)?)
        .bind(encode_routing_assessments(&log.assessments)?)
        .execute(&self.pool)
        .await?;

        Ok(log.clone())
    }

    pub async fn list_routing_decision_logs(&self) -> Result<Vec<RoutingDecisionLog>> {
        let rows = sqlx::query(
            "SELECT decision_id, decision_source, tenant_id, project_id, api_key_group_id, capability, route_key, selected_provider_id, matched_policy_id, applied_routing_profile_id, compiled_routing_snapshot_id, strategy, selection_seed, selection_reason, fallback_reason, requested_region, slo_applied, slo_degraded, created_at_ms, assessments_json
             FROM ai_routing_decision_logs
             ORDER BY created_at_ms DESC, decision_id DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(decode_routing_decision_log_row)
            .collect()
    }

    pub async fn list_routing_decision_logs_for_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<RoutingDecisionLog>> {
        let rows = sqlx::query(
            "SELECT decision_id, decision_source, tenant_id, project_id, api_key_group_id, capability, route_key, selected_provider_id, matched_policy_id, applied_routing_profile_id, compiled_routing_snapshot_id, strategy, selection_seed, selection_reason, fallback_reason, requested_region, slo_applied, slo_degraded, created_at_ms, assessments_json
             FROM ai_routing_decision_logs
             WHERE project_id = $1
             ORDER BY created_at_ms DESC, decision_id DESC",
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(decode_routing_decision_log_row)
            .collect()
    }

    pub async fn find_latest_routing_decision_log_for_project(
        &self,
        project_id: &str,
    ) -> Result<Option<RoutingDecisionLog>> {
        let row = sqlx::query(
            "SELECT decision_id, decision_source, tenant_id, project_id, api_key_group_id, capability, route_key, selected_provider_id, matched_policy_id, applied_routing_profile_id, compiled_routing_snapshot_id, strategy, selection_seed, selection_reason, fallback_reason, requested_region, slo_applied, slo_degraded, created_at_ms, assessments_json
             FROM ai_routing_decision_logs
             WHERE project_id = $1
             ORDER BY created_at_ms DESC, decision_id DESC
             LIMIT 1",
        )
        .bind(project_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(decode_routing_decision_log_row).transpose()
    }

    pub async fn insert_provider_health_snapshot(
        &self,
        snapshot: &ProviderHealthSnapshot,
    ) -> Result<ProviderHealthSnapshot> {
        let mut tx = self.pool.begin().await?;
        let instance_id = snapshot.instance_id.as_deref();
        sqlx::query(
            "DELETE FROM ai_provider_health_records
             WHERE provider_id = $1 AND runtime = $2
               AND ((instance_id IS NULL AND $3 IS NULL) OR instance_id = $4)",
        )
        .bind(&snapshot.provider_id)
        .bind(&snapshot.runtime)
        .bind(instance_id)
        .bind(instance_id)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            "INSERT INTO ai_provider_health_records (provider_id, extension_id, runtime, observed_at_ms, instance_id, running, healthy, message)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(&snapshot.provider_id)
        .bind(&snapshot.extension_id)
        .bind(&snapshot.runtime)
        .bind(i64::try_from(snapshot.observed_at_ms)?)
        .bind(&snapshot.instance_id)
        .bind(snapshot.running)
        .bind(snapshot.healthy)
        .bind(&snapshot.message)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;

        Ok(snapshot.clone())
    }

    pub async fn list_provider_health_snapshots(&self) -> Result<Vec<ProviderHealthSnapshot>> {
        let rows = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                i64,
                Option<String>,
                bool,
                bool,
                Option<String>,
            ),
        >(
            "SELECT provider_id, extension_id, runtime, observed_at_ms, instance_id, running, healthy, message
             FROM ai_provider_health_records
             ORDER BY observed_at_ms DESC, provider_id, runtime, instance_id",
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(
                |(
                    provider_id,
                    extension_id,
                    runtime,
                    observed_at_ms,
                    instance_id,
                    running,
                    healthy,
                    message,
                )| {
                    Ok(ProviderHealthSnapshot::new(
                        provider_id,
                        extension_id,
                        runtime,
                        u64::try_from(observed_at_ms)?,
                    )
                    .with_instance_id_option(instance_id)
                    .with_running(running)
                    .with_healthy(healthy)
                    .with_message_option(message))
                },
            )
            .collect()
    }
}

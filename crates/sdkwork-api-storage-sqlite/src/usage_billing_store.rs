use super::*;

fn decode_billing_event_row(row: &SqliteRow) -> Result<BillingEventRecord> {
    Ok(BillingEventRecord {
        event_id: row.try_get("event_id")?,
        tenant_id: row.try_get("tenant_id")?,
        project_id: row.try_get("project_id")?,
        api_key_group_id: row.try_get("api_key_group_id")?,
        capability: row.try_get("capability")?,
        route_key: row.try_get("route_key")?,
        usage_model: row.try_get("usage_model")?,
        provider_id: row.try_get("provider_id")?,
        accounting_mode: BillingAccountingMode::from_str(
            &row.try_get::<String, _>("accounting_mode")?,
        )
        .unwrap_or(BillingAccountingMode::PlatformCredit),
        operation_kind: row.try_get("operation_kind")?,
        modality: row.try_get("modality")?,
        api_key_hash: row.try_get("api_key_hash")?,
        channel_id: row.try_get("channel_id")?,
        reference_id: row.try_get("reference_id")?,
        latency_ms: row
            .try_get::<Option<i64>, _>("latency_ms")?
            .map(u64::try_from)
            .transpose()?,
        units: u64::try_from(row.try_get::<i64, _>("units")?)?,
        request_count: u64::try_from(row.try_get::<i64, _>("request_count")?)?,
        input_tokens: u64::try_from(row.try_get::<i64, _>("input_tokens")?)?,
        output_tokens: u64::try_from(row.try_get::<i64, _>("output_tokens")?)?,
        total_tokens: u64::try_from(row.try_get::<i64, _>("total_tokens")?)?,
        cache_read_tokens: u64::try_from(row.try_get::<i64, _>("cache_read_tokens")?)?,
        cache_write_tokens: u64::try_from(row.try_get::<i64, _>("cache_write_tokens")?)?,
        image_count: u64::try_from(row.try_get::<i64, _>("image_count")?)?,
        audio_seconds: row.try_get("audio_seconds")?,
        video_seconds: row.try_get("video_seconds")?,
        music_seconds: row.try_get("music_seconds")?,
        upstream_cost: row.try_get("upstream_cost")?,
        customer_charge: row.try_get("customer_charge")?,
        applied_routing_profile_id: row.try_get("applied_routing_profile_id")?,
        compiled_routing_snapshot_id: row.try_get("compiled_routing_snapshot_id")?,
        fallback_reason: row.try_get("fallback_reason")?,
        created_at_ms: u64::try_from(row.try_get::<i64, _>("created_at_ms")?)?,
    })
}


impl SqliteAdminStore {
    pub async fn insert_usage_record(&self, record: &UsageRecord) -> Result<UsageRecord> {
        sqlx::query(
            "INSERT INTO ai_usage_records (
                project_id,
                model,
                provider_id,
                units,
                amount,
                input_tokens,
                output_tokens,
                total_tokens,
                api_key_hash,
                channel_id,
                latency_ms,
                reference_amount,
                created_at_ms
             )
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&record.project_id)
        .bind(&record.model)
        .bind(&record.provider)
        .bind(i64::try_from(record.units)?)
        .bind(record.amount)
        .bind(i64::try_from(record.input_tokens)?)
        .bind(i64::try_from(record.output_tokens)?)
        .bind(i64::try_from(record.total_tokens)?)
        .bind(record.api_key_hash.as_deref())
        .bind(record.channel_id.as_deref())
        .bind(record.latency_ms.map(i64::try_from).transpose()?)
        .bind(record.reference_amount)
        .bind(i64::try_from(record.created_at_ms)?)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    pub async fn list_usage_records(&self) -> Result<Vec<UsageRecord>> {
        let rows = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                i64,
                f64,
                i64,
                i64,
                i64,
                Option<String>,
                Option<String>,
                Option<i64>,
                Option<f64>,
                i64,
            ),
        >(
            "SELECT project_id, model, provider_id, units, amount, input_tokens, output_tokens, total_tokens, api_key_hash, channel_id, latency_ms, reference_amount, created_at_ms
             FROM ai_usage_records
             ORDER BY created_at_ms DESC, rowid DESC",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(
                |(
                    project_id,
                    model,
                    provider,
                    units,
                    amount,
                    input_tokens,
                    output_tokens,
                    total_tokens,
                    api_key_hash,
                    channel_id,
                    latency_ms,
                    reference_amount,
                    created_at_ms,
                )|
                 -> Result<UsageRecord> {
                    Ok(UsageRecord {
                        project_id,
                        model,
                        provider,
                        units: u64::try_from(units)?,
                        amount,
                        input_tokens: u64::try_from(input_tokens)?,
                        output_tokens: u64::try_from(output_tokens)?,
                        total_tokens: u64::try_from(total_tokens)?,
                        api_key_hash,
                        channel_id,
                        latency_ms: latency_ms.map(u64::try_from).transpose()?,
                        reference_amount,
                        created_at_ms: u64::try_from(created_at_ms)?,
                    })
                },
            )
            .collect::<Result<Vec<_>>>()?)
    }

    pub async fn list_usage_records_for_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<UsageRecord>> {
        let rows = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                i64,
                f64,
                i64,
                i64,
                i64,
                Option<String>,
                Option<String>,
                Option<i64>,
                Option<f64>,
                i64,
            ),
        >(
            "SELECT project_id, model, provider_id, units, amount, input_tokens, output_tokens, total_tokens, api_key_hash, channel_id, latency_ms, reference_amount, created_at_ms
             FROM ai_usage_records
             WHERE project_id = ?
             ORDER BY created_at_ms DESC, rowid DESC",
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(
                |(
                    project_id,
                    model,
                    provider,
                    units,
                    amount,
                    input_tokens,
                    output_tokens,
                    total_tokens,
                    api_key_hash,
                    channel_id,
                    latency_ms,
                    reference_amount,
                    created_at_ms,
                )|
                 -> Result<UsageRecord> {
                    Ok(UsageRecord {
                        project_id,
                        model,
                        provider,
                        units: u64::try_from(units)?,
                        amount,
                        input_tokens: u64::try_from(input_tokens)?,
                        output_tokens: u64::try_from(output_tokens)?,
                        total_tokens: u64::try_from(total_tokens)?,
                        api_key_hash,
                        channel_id,
                        latency_ms: latency_ms.map(u64::try_from).transpose()?,
                        reference_amount,
                        created_at_ms: u64::try_from(created_at_ms)?,
                    })
                },
            )
            .collect::<Result<Vec<_>>>()?)
    }

    pub async fn find_latest_usage_record_for_project(
        &self,
        project_id: &str,
    ) -> Result<Option<UsageRecord>> {
        let row = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                i64,
                f64,
                i64,
                i64,
                i64,
                Option<String>,
                Option<String>,
                Option<i64>,
                Option<f64>,
                i64,
            ),
        >(
            "SELECT project_id, model, provider_id, units, amount, input_tokens, output_tokens, total_tokens, api_key_hash, channel_id, latency_ms, reference_amount, created_at_ms
             FROM ai_usage_records
             WHERE project_id = ?
             ORDER BY created_at_ms DESC, rowid DESC
             LIMIT 1",
        )
        .bind(project_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(
            |(
                project_id,
                model,
                provider,
                units,
                amount,
                input_tokens,
                output_tokens,
                total_tokens,
                api_key_hash,
                channel_id,
                latency_ms,
                reference_amount,
                created_at_ms,
            )| {
                Ok(UsageRecord {
                    project_id,
                    model,
                    provider,
                    units: u64::try_from(units)?,
                    amount,
                    input_tokens: u64::try_from(input_tokens)?,
                    output_tokens: u64::try_from(output_tokens)?,
                    total_tokens: u64::try_from(total_tokens)?,
                    api_key_hash,
                    channel_id,
                    latency_ms: latency_ms.map(u64::try_from).transpose()?,
                    reference_amount,
                    created_at_ms: u64::try_from(created_at_ms)?,
                })
            },
        )
        .transpose()
    }

    pub async fn insert_billing_event(
        &self,
        event: &BillingEventRecord,
    ) -> Result<BillingEventRecord> {
        sqlx::query(
            "INSERT INTO ai_billing_events (
                event_id,
                tenant_id,
                project_id,
                api_key_group_id,
                capability,
                route_key,
                usage_model,
                provider_id,
                accounting_mode,
                operation_kind,
                modality,
                api_key_hash,
                channel_id,
                reference_id,
                latency_ms,
                units,
                request_count,
                input_tokens,
                output_tokens,
                total_tokens,
                cache_read_tokens,
                cache_write_tokens,
                image_count,
                audio_seconds,
                video_seconds,
                music_seconds,
                upstream_cost,
                customer_charge,
                applied_routing_profile_id,
                compiled_routing_snapshot_id,
                fallback_reason,
                created_at_ms
             )
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(event_id) DO UPDATE SET
                tenant_id = excluded.tenant_id,
                project_id = excluded.project_id,
                api_key_group_id = excluded.api_key_group_id,
                capability = excluded.capability,
                route_key = excluded.route_key,
                usage_model = excluded.usage_model,
                provider_id = excluded.provider_id,
                accounting_mode = excluded.accounting_mode,
                operation_kind = excluded.operation_kind,
                modality = excluded.modality,
                api_key_hash = excluded.api_key_hash,
                channel_id = excluded.channel_id,
                reference_id = excluded.reference_id,
                latency_ms = excluded.latency_ms,
                units = excluded.units,
                request_count = excluded.request_count,
                input_tokens = excluded.input_tokens,
                output_tokens = excluded.output_tokens,
                total_tokens = excluded.total_tokens,
                cache_read_tokens = excluded.cache_read_tokens,
                cache_write_tokens = excluded.cache_write_tokens,
                image_count = excluded.image_count,
                audio_seconds = excluded.audio_seconds,
                video_seconds = excluded.video_seconds,
                music_seconds = excluded.music_seconds,
                upstream_cost = excluded.upstream_cost,
                customer_charge = excluded.customer_charge,
                applied_routing_profile_id = excluded.applied_routing_profile_id,
                compiled_routing_snapshot_id = excluded.compiled_routing_snapshot_id,
                fallback_reason = excluded.fallback_reason,
                created_at_ms = excluded.created_at_ms",
        )
        .bind(&event.event_id)
        .bind(&event.tenant_id)
        .bind(&event.project_id)
        .bind(event.api_key_group_id.as_deref())
        .bind(&event.capability)
        .bind(&event.route_key)
        .bind(&event.usage_model)
        .bind(&event.provider_id)
        .bind(event.accounting_mode.as_str())
        .bind(&event.operation_kind)
        .bind(&event.modality)
        .bind(event.api_key_hash.as_deref())
        .bind(event.channel_id.as_deref())
        .bind(event.reference_id.as_deref())
        .bind(event.latency_ms.map(i64::try_from).transpose()?)
        .bind(i64::try_from(event.units)?)
        .bind(i64::try_from(event.request_count)?)
        .bind(i64::try_from(event.input_tokens)?)
        .bind(i64::try_from(event.output_tokens)?)
        .bind(i64::try_from(event.total_tokens)?)
        .bind(i64::try_from(event.cache_read_tokens)?)
        .bind(i64::try_from(event.cache_write_tokens)?)
        .bind(i64::try_from(event.image_count)?)
        .bind(event.audio_seconds)
        .bind(event.video_seconds)
        .bind(event.music_seconds)
        .bind(event.upstream_cost)
        .bind(event.customer_charge)
        .bind(event.applied_routing_profile_id.as_deref())
        .bind(event.compiled_routing_snapshot_id.as_deref())
        .bind(event.fallback_reason.as_deref())
        .bind(i64::try_from(event.created_at_ms)?)
        .execute(&self.pool)
        .await?;
        Ok(event.clone())
    }

    pub async fn list_billing_events(&self) -> Result<Vec<BillingEventRecord>> {
        let rows = sqlx::query(
            "SELECT
                event_id,
                tenant_id,
                project_id,
                api_key_group_id,
                capability,
                route_key,
                usage_model,
                provider_id,
                accounting_mode,
                operation_kind,
                modality,
                api_key_hash,
                channel_id,
                reference_id,
                latency_ms,
                units,
                request_count,
                input_tokens,
                output_tokens,
                total_tokens,
                cache_read_tokens,
                cache_write_tokens,
                image_count,
                audio_seconds,
                video_seconds,
                music_seconds,
                upstream_cost,
                customer_charge,
                applied_routing_profile_id,
                compiled_routing_snapshot_id,
                fallback_reason,
                created_at_ms
             FROM ai_billing_events
             ORDER BY created_at_ms DESC, rowid DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        rows.iter()
            .map(decode_billing_event_row)
            .collect::<Result<Vec<_>>>()
    }

    pub async fn insert_ledger_entry(&self, entry: &LedgerEntry) -> Result<LedgerEntry> {
        sqlx::query(
            "INSERT INTO ai_billing_ledger_entries (project_id, units, amount, created_at_ms) VALUES (?, ?, ?, ?)",
        )
        .bind(&entry.project_id)
        .bind(i64::try_from(entry.units)?)
        .bind(entry.amount)
        .bind(current_timestamp_ms())
        .execute(&self.pool)
        .await?;
        Ok(entry.clone())
    }

    pub async fn list_ledger_entries(&self) -> Result<Vec<LedgerEntry>> {
        let rows = sqlx::query_as::<_, (String, i64, f64)>(
            "SELECT project_id, units, amount FROM ai_billing_ledger_entries ORDER BY created_at_ms DESC, rowid DESC",
        )
        .fetch_all(&self.pool)
        .await?;
        let entries = rows
            .into_iter()
            .map(|(project_id, units, amount)| {
                Ok(LedgerEntry {
                    project_id,
                    units: u64::try_from(units)?,
                    amount,
                })
            })
            .collect::<std::result::Result<Vec<_>, std::num::TryFromIntError>>()?;
        Ok(entries)
    }

    pub async fn list_ledger_entries_for_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<LedgerEntry>> {
        let rows = sqlx::query_as::<_, (String, i64, f64)>(
            "SELECT project_id, units, amount
             FROM ai_billing_ledger_entries
             WHERE project_id = ?
             ORDER BY created_at_ms DESC, rowid DESC",
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await?;
        let entries = rows
            .into_iter()
            .map(|(project_id, units, amount)| {
                Ok(LedgerEntry {
                    project_id,
                    units: u64::try_from(units)?,
                    amount,
                })
            })
            .collect::<std::result::Result<Vec<_>, std::num::TryFromIntError>>()?;
        Ok(entries)
    }

    pub async fn insert_quota_policy(&self, policy: &QuotaPolicy) -> Result<QuotaPolicy> {
        sqlx::query(
            "INSERT INTO ai_billing_quota_policies (policy_id, project_id, max_units, enabled)
             VALUES (?, ?, ?, ?)
             ON CONFLICT(policy_id) DO UPDATE SET
             project_id = excluded.project_id,
             max_units = excluded.max_units,
             enabled = excluded.enabled",
        )
        .bind(&policy.policy_id)
        .bind(&policy.project_id)
        .bind(i64::try_from(policy.max_units)?)
        .bind(if policy.enabled { 1_i64 } else { 0_i64 })
        .execute(&self.pool)
        .await?;
        Ok(policy.clone())
    }

    pub async fn list_quota_policies(&self) -> Result<Vec<QuotaPolicy>> {
        let rows = sqlx::query_as::<_, (String, String, i64, i64)>(
            "SELECT policy_id, project_id, max_units, enabled
             FROM ai_billing_quota_policies
             ORDER BY policy_id",
        )
        .fetch_all(&self.pool)
        .await?;

        let policies = rows
            .into_iter()
            .map(|(policy_id, project_id, max_units, enabled)| {
                Ok(QuotaPolicy {
                    policy_id,
                    project_id,
                    max_units: u64::try_from(max_units)?,
                    enabled: enabled != 0,
                })
            })
            .collect::<std::result::Result<Vec<_>, std::num::TryFromIntError>>()?;
        Ok(policies)
    }

    pub async fn list_quota_policies_for_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<QuotaPolicy>> {
        let rows = sqlx::query_as::<_, (String, String, i64, i64)>(
            "SELECT policy_id, project_id, max_units, enabled
             FROM ai_billing_quota_policies
             WHERE project_id = ?
             ORDER BY policy_id",
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await?;

        let policies = rows
            .into_iter()
            .map(|(policy_id, project_id, max_units, enabled)| {
                Ok(QuotaPolicy {
                    policy_id,
                    project_id,
                    max_units: u64::try_from(max_units)?,
                    enabled: enabled != 0,
                })
            })
            .collect::<std::result::Result<Vec<_>, std::num::TryFromIntError>>()?;
        Ok(policies)
    }

    pub async fn delete_quota_policy(&self, policy_id: &str) -> Result<bool> {
        let result = sqlx::query(
            "DELETE FROM ai_billing_quota_policies
             WHERE policy_id = ?",
        )
        .bind(policy_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn insert_rate_limit_policy(
        &self,
        policy: &RateLimitPolicy,
    ) -> Result<RateLimitPolicy> {
        sqlx::query(
            "INSERT INTO ai_gateway_rate_limit_policies (
                policy_id, project_id, api_key_hash, route_key, model_name,
                requests_per_window, window_seconds, burst_requests, enabled,
                notes, created_at_ms, updated_at_ms
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(policy_id) DO UPDATE SET
             project_id = excluded.project_id,
             api_key_hash = excluded.api_key_hash,
             route_key = excluded.route_key,
             model_name = excluded.model_name,
             requests_per_window = excluded.requests_per_window,
             window_seconds = excluded.window_seconds,
             burst_requests = excluded.burst_requests,
             enabled = excluded.enabled,
             notes = excluded.notes,
             created_at_ms = excluded.created_at_ms,
             updated_at_ms = excluded.updated_at_ms",
        )
        .bind(&policy.policy_id)
        .bind(&policy.project_id)
        .bind(&policy.api_key_hash)
        .bind(&policy.route_key)
        .bind(&policy.model_name)
        .bind(i64::try_from(policy.requests_per_window)?)
        .bind(i64::try_from(policy.window_seconds)?)
        .bind(i64::try_from(policy.burst_requests)?)
        .bind(if policy.enabled { 1_i64 } else { 0_i64 })
        .bind(&policy.notes)
        .bind(i64::try_from(policy.created_at_ms)?)
        .bind(i64::try_from(policy.updated_at_ms)?)
        .execute(&self.pool)
        .await?;
        Ok(policy.clone())
    }

    pub async fn list_rate_limit_policies(&self) -> Result<Vec<RateLimitPolicy>> {
        let rows = sqlx::query_as::<_, (
            String,
            String,
            Option<String>,
            Option<String>,
            Option<String>,
            i64,
            i64,
            i64,
            i64,
            Option<String>,
            i64,
            i64,
        )>(
            "SELECT policy_id, project_id, api_key_hash, route_key, model_name, requests_per_window, window_seconds, burst_requests, enabled, notes, created_at_ms, updated_at_ms
             FROM ai_gateway_rate_limit_policies
             ORDER BY project_id, enabled DESC, policy_id",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(
                |(
                    policy_id,
                    project_id,
                    api_key_hash,
                    route_key,
                    model_name,
                    requests_per_window,
                    window_seconds,
                    burst_requests,
                    enabled,
                    notes,
                    created_at_ms,
                    updated_at_ms,
                )| {
                    Ok(RateLimitPolicy {
                        policy_id,
                        project_id,
                        api_key_hash,
                        route_key,
                        model_name,
                        requests_per_window: u64::try_from(requests_per_window)?,
                        window_seconds: u64::try_from(window_seconds)?,
                        burst_requests: u64::try_from(burst_requests)?,
                        enabled: enabled != 0,
                        notes,
                        created_at_ms: u64::try_from(created_at_ms)?,
                        updated_at_ms: u64::try_from(updated_at_ms)?,
                    })
                },
            )
            .collect::<std::result::Result<Vec<_>, std::num::TryFromIntError>>()?)
    }

    pub async fn list_rate_limit_policies_for_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<RateLimitPolicy>> {
        let rows = sqlx::query_as::<_, (
            String,
            String,
            Option<String>,
            Option<String>,
            Option<String>,
            i64,
            i64,
            i64,
            i64,
            Option<String>,
            i64,
            i64,
        )>(
            "SELECT policy_id, project_id, api_key_hash, route_key, model_name, requests_per_window, window_seconds, burst_requests, enabled, notes, created_at_ms, updated_at_ms
             FROM ai_gateway_rate_limit_policies
             WHERE project_id = ?
             ORDER BY enabled DESC, policy_id",
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(
                |(
                    policy_id,
                    project_id,
                    api_key_hash,
                    route_key,
                    model_name,
                    requests_per_window,
                    window_seconds,
                    burst_requests,
                    enabled,
                    notes,
                    created_at_ms,
                    updated_at_ms,
                )| {
                    Ok(RateLimitPolicy {
                        policy_id,
                        project_id,
                        api_key_hash,
                        route_key,
                        model_name,
                        requests_per_window: u64::try_from(requests_per_window)?,
                        window_seconds: u64::try_from(window_seconds)?,
                        burst_requests: u64::try_from(burst_requests)?,
                        enabled: enabled != 0,
                        notes,
                        created_at_ms: u64::try_from(created_at_ms)?,
                        updated_at_ms: u64::try_from(updated_at_ms)?,
                    })
                },
            )
            .collect::<std::result::Result<Vec<_>, std::num::TryFromIntError>>()?)
    }

    pub async fn list_rate_limit_window_snapshots(&self) -> Result<Vec<RateLimitWindowSnapshot>> {
        let rows = sqlx::query_as::<
            _,
            (
                String,
                String,
                Option<String>,
                Option<String>,
                Option<String>,
                i64,
                i64,
                i64,
                i64,
                i64,
                i64,
                i64,
            ),
        >(
            "SELECT
                p.policy_id,
                p.project_id,
                p.api_key_hash,
                p.route_key,
                p.model_name,
                p.requests_per_window,
                p.window_seconds,
                p.burst_requests,
                w.request_count,
                w.window_start_ms,
                w.updated_at_ms,
                p.enabled
             FROM ai_gateway_rate_limit_windows w
             INNER JOIN ai_gateway_rate_limit_policies p ON p.policy_id = w.policy_id
             ORDER BY p.project_id, w.updated_at_ms DESC, p.policy_id, w.window_start_ms DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(
                |(
                    policy_id,
                    project_id,
                    api_key_hash,
                    route_key,
                    model_name,
                    requests_per_window,
                    window_seconds,
                    burst_requests,
                    request_count,
                    window_start_ms,
                    updated_at_ms,
                    enabled,
                )| {
                    let requests_per_window = u64::try_from(requests_per_window)?;
                    let window_seconds = u64::try_from(window_seconds)?;
                    let burst_requests = u64::try_from(burst_requests)?;
                    let request_count = u64::try_from(request_count)?;
                    let window_start_ms = u64::try_from(window_start_ms)?;
                    let updated_at_ms = u64::try_from(updated_at_ms)?;
                    let limit_requests = match burst_requests {
                        0 => requests_per_window,
                        burst => burst.max(requests_per_window),
                    };
                    let remaining_requests = limit_requests.saturating_sub(request_count);

                    Ok(RateLimitWindowSnapshot {
                        policy_id,
                        project_id,
                        api_key_hash,
                        route_key,
                        model_name,
                        requests_per_window,
                        window_seconds,
                        burst_requests,
                        limit_requests,
                        request_count,
                        remaining_requests,
                        window_start_ms,
                        window_end_ms: window_start_ms
                            .saturating_add(window_seconds.saturating_mul(1000)),
                        updated_at_ms,
                        enabled: enabled != 0,
                        exceeded: request_count > limit_requests,
                    })
                },
            )
            .collect::<std::result::Result<Vec<_>, std::num::TryFromIntError>>()?)
    }

    pub async fn check_and_consume_rate_limit(
        &self,
        policy_id: &str,
        requested_requests: u64,
        limit_requests: u64,
        window_seconds: u64,
        now_ms: u64,
    ) -> Result<RateLimitCheckResult> {
        let window_seconds = window_seconds.max(1);
        let window_ms = window_seconds.saturating_mul(1000);
        let window_start_ms = now_ms - (now_ms % window_ms);
        let requested = i64::try_from(requested_requests)?;
        let limit = i64::try_from(limit_requests)?;
        let window_start = i64::try_from(window_start_ms)?;
        let now = i64::try_from(now_ms)?;

        let mut tx = self.pool.begin().await?;

        sqlx::query(
            "INSERT INTO ai_gateway_rate_limit_windows (policy_id, window_start_ms, request_count, updated_at_ms)
             VALUES (?, ?, 0, ?)
             ON CONFLICT(policy_id, window_start_ms) DO NOTHING",
        )
        .bind(policy_id)
        .bind(window_start)
        .bind(now)
        .execute(&mut *tx)
        .await?;

        loop {
            let used_before = sqlx::query_as::<_, (i64,)>(
                "SELECT request_count
                 FROM ai_gateway_rate_limit_windows
                 WHERE policy_id = ? AND window_start_ms = ?",
            )
            .bind(policy_id)
            .bind(window_start)
            .fetch_one(&mut *tx)
            .await?
            .0;

            if used_before.saturating_add(requested) > limit {
                tx.rollback().await?;
                return Ok(RateLimitCheckResult {
                    allowed: false,
                    policy_id: Some(policy_id.to_owned()),
                    requested_requests,
                    used_requests: u64::try_from(used_before)?,
                    limit_requests: Some(limit_requests),
                    remaining_requests: Some(
                        limit_requests.saturating_sub(u64::try_from(used_before)?),
                    ),
                    window_seconds: Some(window_seconds),
                    window_start_ms: Some(window_start_ms),
                    window_end_ms: Some(window_start_ms.saturating_add(window_ms)),
                });
            }

            let updated = sqlx::query(
                "UPDATE ai_gateway_rate_limit_windows
                 SET request_count = request_count + ?, updated_at_ms = ?
                 WHERE policy_id = ? AND window_start_ms = ? AND request_count = ?",
            )
            .bind(requested)
            .bind(now)
            .bind(policy_id)
            .bind(window_start)
            .bind(used_before)
            .execute(&mut *tx)
            .await?;

            if updated.rows_affected() == 1 {
                tx.commit().await?;
                return Ok(RateLimitCheckResult {
                    allowed: true,
                    policy_id: Some(policy_id.to_owned()),
                    requested_requests,
                    used_requests: u64::try_from(used_before)?,
                    limit_requests: Some(limit_requests),
                    remaining_requests: Some(limit_requests.saturating_sub(
                        u64::try_from(used_before)?.saturating_add(requested_requests),
                    )),
                    window_seconds: Some(window_seconds),
                    window_start_ms: Some(window_start_ms),
                    window_end_ms: Some(window_start_ms.saturating_add(window_ms)),
                });
            }
        }
    }


}

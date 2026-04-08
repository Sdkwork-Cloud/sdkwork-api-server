use super::*;

fn decode_async_job_row(row: SqliteRow) -> Result<AsyncJobRecord> {
    let status = AsyncJobStatus::from_str(&row.try_get::<String, _>("status")?)
        .map_err(anyhow::Error::msg)?;
    Ok(AsyncJobRecord::new(
        row.try_get::<String, _>("job_id")?,
        u64::try_from(row.try_get::<i64, _>("tenant_id")?)?,
        u64::try_from(row.try_get::<i64, _>("organization_id")?)?,
        u64::try_from(row.try_get::<i64, _>("user_id")?)?,
        row.try_get::<String, _>("capability_code")?,
        row.try_get::<String, _>("modality")?,
        row.try_get::<String, _>("operation_kind")?,
        u64::try_from(row.try_get::<i64, _>("created_at_ms")?)?,
    )
    .with_account_id(
        row.try_get::<Option<i64>, _>("account_id")?
            .map(u64::try_from)
            .transpose()?,
    )
    .with_request_id(
        row.try_get::<Option<i64>, _>("request_id")?
            .map(u64::try_from)
            .transpose()?,
    )
    .with_provider_id(row.try_get::<Option<String>, _>("provider_id")?)
    .with_model_code(row.try_get::<Option<String>, _>("model_code")?)
    .with_status(status)
    .with_external_job_id(row.try_get::<Option<String>, _>("external_job_id")?)
    .with_idempotency_key(row.try_get::<Option<String>, _>("idempotency_key")?)
    .with_callback_url(row.try_get::<Option<String>, _>("callback_url")?)
    .with_input_summary(row.try_get::<Option<String>, _>("input_summary")?)
    .with_progress_percent(
        row.try_get::<Option<i64>, _>("progress_percent")?
            .map(u64::try_from)
            .transpose()?,
    )
    .with_error_code(row.try_get::<Option<String>, _>("error_code")?)
    .with_error_message(row.try_get::<Option<String>, _>("error_message")?)
    .with_updated_at_ms(u64::try_from(row.try_get::<i64, _>("updated_at_ms")?)?)
    .with_started_at_ms(
        row.try_get::<Option<i64>, _>("started_at_ms")?
            .map(u64::try_from)
            .transpose()?,
    )
    .with_completed_at_ms(
        row.try_get::<Option<i64>, _>("completed_at_ms")?
            .map(u64::try_from)
            .transpose()?,
    ))
}

fn decode_async_job_attempt_row(row: SqliteRow) -> Result<AsyncJobAttemptRecord> {
    let status = AsyncJobAttemptStatus::from_str(&row.try_get::<String, _>("status")?)
        .map_err(anyhow::Error::msg)?;
    Ok(AsyncJobAttemptRecord::new(
        u64::try_from(row.try_get::<i64, _>("attempt_id")?)?,
        row.try_get::<String, _>("job_id")?,
        u64::try_from(row.try_get::<i64, _>("attempt_number")?)?,
        row.try_get::<String, _>("runtime_kind")?,
        u64::try_from(row.try_get::<i64, _>("created_at_ms")?)?,
    )
    .with_status(status)
    .with_endpoint(row.try_get::<Option<String>, _>("endpoint")?)
    .with_external_job_id(row.try_get::<Option<String>, _>("external_job_id")?)
    .with_claimed_at_ms(
        row.try_get::<Option<i64>, _>("claimed_at_ms")?
            .map(u64::try_from)
            .transpose()?,
    )
    .with_finished_at_ms(
        row.try_get::<Option<i64>, _>("finished_at_ms")?
            .map(u64::try_from)
            .transpose()?,
    )
    .with_error_message(row.try_get::<Option<String>, _>("error_message")?)
    .with_updated_at_ms(u64::try_from(row.try_get::<i64, _>("updated_at_ms")?)?))
}

fn decode_async_job_asset_row(row: SqliteRow) -> Result<AsyncJobAssetRecord> {
    Ok(AsyncJobAssetRecord::new(
        row.try_get::<String, _>("asset_id")?,
        row.try_get::<String, _>("job_id")?,
        row.try_get::<String, _>("asset_kind")?,
        row.try_get::<String, _>("storage_key")?,
        u64::try_from(row.try_get::<i64, _>("created_at_ms")?)?,
    )
    .with_download_url(row.try_get::<Option<String>, _>("download_url")?)
    .with_mime_type(row.try_get::<Option<String>, _>("mime_type")?)
    .with_size_bytes(
        row.try_get::<Option<i64>, _>("size_bytes")?
            .map(u64::try_from)
            .transpose()?,
    )
    .with_checksum_sha256(row.try_get::<Option<String>, _>("checksum_sha256")?))
}

fn decode_async_job_callback_row(row: SqliteRow) -> Result<AsyncJobCallbackRecord> {
    let status = AsyncJobCallbackStatus::from_str(&row.try_get::<String, _>("status")?)
        .map_err(anyhow::Error::msg)?;
    Ok(AsyncJobCallbackRecord::new(
        u64::try_from(row.try_get::<i64, _>("callback_id")?)?,
        row.try_get::<String, _>("job_id")?,
        row.try_get::<String, _>("event_type")?,
        row.try_get::<String, _>("payload_json")?,
        u64::try_from(row.try_get::<i64, _>("received_at_ms")?)?,
    )
    .with_dedupe_key(row.try_get::<Option<String>, _>("dedupe_key")?)
    .with_status(status)
    .with_processed_at_ms(
        row.try_get::<Option<i64>, _>("processed_at_ms")?
            .map(u64::try_from)
            .transpose()?,
    ))
}


impl SqliteAdminStore {
    pub async fn insert_async_job(&self, record: &AsyncJobRecord) -> Result<AsyncJobRecord> {
        sqlx::query(
            "INSERT INTO ai_async_jobs (
                job_id,
                tenant_id,
                organization_id,
                user_id,
                account_id,
                request_id,
                provider_id,
                model_code,
                capability_code,
                modality,
                operation_kind,
                status,
                external_job_id,
                idempotency_key,
                callback_url,
                input_summary,
                progress_percent,
                error_code,
                error_message,
                created_at_ms,
                updated_at_ms,
                started_at_ms,
                completed_at_ms
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(job_id) DO UPDATE SET
                tenant_id = excluded.tenant_id,
                organization_id = excluded.organization_id,
                user_id = excluded.user_id,
                account_id = excluded.account_id,
                request_id = excluded.request_id,
                provider_id = excluded.provider_id,
                model_code = excluded.model_code,
                capability_code = excluded.capability_code,
                modality = excluded.modality,
                operation_kind = excluded.operation_kind,
                status = excluded.status,
                external_job_id = excluded.external_job_id,
                idempotency_key = excluded.idempotency_key,
                callback_url = excluded.callback_url,
                input_summary = excluded.input_summary,
                progress_percent = excluded.progress_percent,
                error_code = excluded.error_code,
                error_message = excluded.error_message,
                created_at_ms = excluded.created_at_ms,
                updated_at_ms = excluded.updated_at_ms,
                started_at_ms = excluded.started_at_ms,
                completed_at_ms = excluded.completed_at_ms",
        )
        .bind(&record.job_id)
        .bind(i64::try_from(record.tenant_id)?)
        .bind(i64::try_from(record.organization_id)?)
        .bind(i64::try_from(record.user_id)?)
        .bind(record.account_id.map(i64::try_from).transpose()?)
        .bind(record.request_id.map(i64::try_from).transpose()?)
        .bind(&record.provider_id)
        .bind(&record.model_code)
        .bind(&record.capability_code)
        .bind(&record.modality)
        .bind(&record.operation_kind)
        .bind(record.status.as_str())
        .bind(&record.external_job_id)
        .bind(&record.idempotency_key)
        .bind(&record.callback_url)
        .bind(&record.input_summary)
        .bind(record.progress_percent.map(i64::try_from).transpose()?)
        .bind(&record.error_code)
        .bind(&record.error_message)
        .bind(i64::try_from(record.created_at_ms)?)
        .bind(i64::try_from(record.updated_at_ms)?)
        .bind(record.started_at_ms.map(i64::try_from).transpose()?)
        .bind(record.completed_at_ms.map(i64::try_from).transpose()?)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    pub async fn list_async_jobs(&self) -> Result<Vec<AsyncJobRecord>> {
        let rows = sqlx::query(
            "SELECT job_id, tenant_id, organization_id, user_id, account_id, request_id, provider_id,
                    model_code, capability_code, modality, operation_kind, status, external_job_id,
                    idempotency_key, callback_url, input_summary, progress_percent, error_code,
                    error_message, created_at_ms, updated_at_ms, started_at_ms, completed_at_ms
             FROM ai_async_jobs
             ORDER BY created_at_ms DESC, job_id DESC",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(decode_async_job_row).collect()
    }

    pub async fn find_async_job(&self, job_id: &str) -> Result<Option<AsyncJobRecord>> {
        let row = sqlx::query(
            "SELECT job_id, tenant_id, organization_id, user_id, account_id, request_id, provider_id,
                    model_code, capability_code, modality, operation_kind, status, external_job_id,
                    idempotency_key, callback_url, input_summary, progress_percent, error_code,
                    error_message, created_at_ms, updated_at_ms, started_at_ms, completed_at_ms
             FROM ai_async_jobs
             WHERE job_id = ?",
        )
        .bind(job_id)
        .fetch_optional(&self.pool)
        .await?;
        row.map(decode_async_job_row).transpose()
    }

    pub async fn insert_async_job_attempt(
        &self,
        record: &AsyncJobAttemptRecord,
    ) -> Result<AsyncJobAttemptRecord> {
        sqlx::query(
            "INSERT INTO ai_async_job_attempts (
                attempt_id,
                job_id,
                attempt_number,
                status,
                runtime_kind,
                endpoint,
                external_job_id,
                claimed_at_ms,
                finished_at_ms,
                error_message,
                created_at_ms,
                updated_at_ms
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(attempt_id) DO UPDATE SET
                job_id = excluded.job_id,
                attempt_number = excluded.attempt_number,
                status = excluded.status,
                runtime_kind = excluded.runtime_kind,
                endpoint = excluded.endpoint,
                external_job_id = excluded.external_job_id,
                claimed_at_ms = excluded.claimed_at_ms,
                finished_at_ms = excluded.finished_at_ms,
                error_message = excluded.error_message,
                created_at_ms = excluded.created_at_ms,
                updated_at_ms = excluded.updated_at_ms",
        )
        .bind(i64::try_from(record.attempt_id)?)
        .bind(&record.job_id)
        .bind(i64::try_from(record.attempt_number)?)
        .bind(record.status.as_str())
        .bind(&record.runtime_kind)
        .bind(&record.endpoint)
        .bind(&record.external_job_id)
        .bind(record.claimed_at_ms.map(i64::try_from).transpose()?)
        .bind(record.finished_at_ms.map(i64::try_from).transpose()?)
        .bind(&record.error_message)
        .bind(i64::try_from(record.created_at_ms)?)
        .bind(i64::try_from(record.updated_at_ms)?)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    pub async fn list_async_job_attempts(
        &self,
        job_id: &str,
    ) -> Result<Vec<AsyncJobAttemptRecord>> {
        let rows = sqlx::query(
            "SELECT attempt_id, job_id, attempt_number, status, runtime_kind, endpoint, external_job_id,
                    claimed_at_ms, finished_at_ms, error_message, created_at_ms, updated_at_ms
             FROM ai_async_job_attempts
             WHERE job_id = ?
             ORDER BY attempt_number ASC, attempt_id ASC",
        )
        .bind(job_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(decode_async_job_attempt_row).collect()
    }

    pub async fn insert_async_job_asset(
        &self,
        record: &AsyncJobAssetRecord,
    ) -> Result<AsyncJobAssetRecord> {
        sqlx::query(
            "INSERT INTO ai_async_job_assets (
                asset_id,
                job_id,
                asset_kind,
                storage_key,
                download_url,
                mime_type,
                size_bytes,
                checksum_sha256,
                created_at_ms
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(asset_id) DO UPDATE SET
                job_id = excluded.job_id,
                asset_kind = excluded.asset_kind,
                storage_key = excluded.storage_key,
                download_url = excluded.download_url,
                mime_type = excluded.mime_type,
                size_bytes = excluded.size_bytes,
                checksum_sha256 = excluded.checksum_sha256,
                created_at_ms = excluded.created_at_ms",
        )
        .bind(&record.asset_id)
        .bind(&record.job_id)
        .bind(&record.asset_kind)
        .bind(&record.storage_key)
        .bind(&record.download_url)
        .bind(&record.mime_type)
        .bind(record.size_bytes.map(i64::try_from).transpose()?)
        .bind(&record.checksum_sha256)
        .bind(i64::try_from(record.created_at_ms)?)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    pub async fn list_async_job_assets(&self, job_id: &str) -> Result<Vec<AsyncJobAssetRecord>> {
        let rows = sqlx::query(
            "SELECT asset_id, job_id, asset_kind, storage_key, download_url, mime_type, size_bytes,
                    checksum_sha256, created_at_ms
             FROM ai_async_job_assets
             WHERE job_id = ?
             ORDER BY created_at_ms ASC, asset_id ASC",
        )
        .bind(job_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(decode_async_job_asset_row).collect()
    }

    pub async fn insert_async_job_callback(
        &self,
        record: &AsyncJobCallbackRecord,
    ) -> Result<AsyncJobCallbackRecord> {
        sqlx::query(
            "INSERT INTO ai_async_job_callbacks (
                callback_id,
                job_id,
                event_type,
                dedupe_key,
                payload_json,
                status,
                received_at_ms,
                processed_at_ms
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(callback_id) DO UPDATE SET
                job_id = excluded.job_id,
                event_type = excluded.event_type,
                dedupe_key = excluded.dedupe_key,
                payload_json = excluded.payload_json,
                status = excluded.status,
                received_at_ms = excluded.received_at_ms,
                processed_at_ms = excluded.processed_at_ms",
        )
        .bind(i64::try_from(record.callback_id)?)
        .bind(&record.job_id)
        .bind(&record.event_type)
        .bind(&record.dedupe_key)
        .bind(&record.payload_json)
        .bind(record.status.as_str())
        .bind(i64::try_from(record.received_at_ms)?)
        .bind(record.processed_at_ms.map(i64::try_from).transpose()?)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    pub async fn list_async_job_callbacks(
        &self,
        job_id: &str,
    ) -> Result<Vec<AsyncJobCallbackRecord>> {
        let rows = sqlx::query(
            "SELECT callback_id, job_id, event_type, dedupe_key, payload_json, status,
                    received_at_ms, processed_at_ms
             FROM ai_async_job_callbacks
             WHERE job_id = ?
             ORDER BY received_at_ms ASC, callback_id ASC",
        )
        .bind(job_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(decode_async_job_callback_row)
            .collect()
    }


}

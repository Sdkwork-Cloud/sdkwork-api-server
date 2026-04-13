use super::*;

fn decode_identity_user_row(row: PgRow) -> Result<IdentityUserRecord> {
    Ok(IdentityUserRecord::new(
        u64::try_from(row.try_get::<i64, _>("user_id")?)?,
        u64::try_from(row.try_get::<i64, _>("tenant_id")?)?,
        u64::try_from(row.try_get::<i64, _>("organization_id")?)?,
    )
    .with_external_user_ref(row.try_get::<Option<String>, _>("external_user_ref")?)
    .with_username(row.try_get::<Option<String>, _>("username")?)
    .with_display_name(row.try_get::<Option<String>, _>("display_name")?)
    .with_email(row.try_get::<Option<String>, _>("email")?)
    .with_status(row.try_get::<String, _>("status")?)
    .with_created_at_ms(u64::try_from(row.try_get::<i64, _>("created_at_ms")?)?)
    .with_updated_at_ms(u64::try_from(row.try_get::<i64, _>("updated_at_ms")?)?))
}

fn decode_canonical_api_key_row(row: PgRow) -> Result<CanonicalApiKeyRecord> {
    Ok(CanonicalApiKeyRecord::new(
        u64::try_from(row.try_get::<i64, _>("api_key_id")?)?,
        u64::try_from(row.try_get::<i64, _>("tenant_id")?)?,
        u64::try_from(row.try_get::<i64, _>("organization_id")?)?,
        u64::try_from(row.try_get::<i64, _>("user_id")?)?,
        row.try_get::<String, _>("key_hash")?,
    )
    .with_key_prefix(row.try_get::<String, _>("key_prefix")?)
    .with_display_name(row.try_get::<String, _>("display_name")?)
    .with_status(row.try_get::<String, _>("status")?)
    .with_expires_at_ms(
        row.try_get::<Option<i64>, _>("expires_at_ms")?
            .map(u64::try_from)
            .transpose()?,
    )
    .with_last_used_at_ms(
        row.try_get::<Option<i64>, _>("last_used_at_ms")?
            .map(u64::try_from)
            .transpose()?,
    )
    .with_rotated_from_api_key_id(
        row.try_get::<Option<i64>, _>("rotated_from_api_key_id")?
            .map(u64::try_from)
            .transpose()?,
    )
    .with_created_at_ms(u64::try_from(row.try_get::<i64, _>("created_at_ms")?)?)
    .with_updated_at_ms(u64::try_from(row.try_get::<i64, _>("updated_at_ms")?)?))
}

fn decode_identity_binding_row(row: PgRow) -> Result<IdentityBindingRecord> {
    Ok(IdentityBindingRecord::new(
        u64::try_from(row.try_get::<i64, _>("identity_binding_id")?)?,
        u64::try_from(row.try_get::<i64, _>("tenant_id")?)?,
        u64::try_from(row.try_get::<i64, _>("organization_id")?)?,
        u64::try_from(row.try_get::<i64, _>("user_id")?)?,
        row.try_get::<String, _>("binding_type")?,
    )
    .with_issuer(row.try_get::<Option<String>, _>("issuer")?)
    .with_subject(row.try_get::<Option<String>, _>("subject")?)
    .with_platform(row.try_get::<Option<String>, _>("platform")?)
    .with_owner(row.try_get::<Option<String>, _>("owner")?)
    .with_external_ref(row.try_get::<Option<String>, _>("external_ref")?)
    .with_status(row.try_get::<String, _>("status")?)
    .with_created_at_ms(u64::try_from(row.try_get::<i64, _>("created_at_ms")?)?)
    .with_updated_at_ms(u64::try_from(row.try_get::<i64, _>("updated_at_ms")?)?))
}

#[async_trait]
impl IdentityKernelStore for PostgresAdminStore {
    async fn insert_identity_user_record(
        &self,
        record: &IdentityUserRecord,
    ) -> Result<IdentityUserRecord> {
        sqlx::query(
            "INSERT INTO ai_user (
                user_id, tenant_id, organization_id, external_user_ref, username,
                display_name, email, status, created_at_ms, updated_at_ms
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
             ON CONFLICT(user_id) DO UPDATE SET
                tenant_id = excluded.tenant_id,
                organization_id = excluded.organization_id,
                external_user_ref = excluded.external_user_ref,
                username = excluded.username,
                display_name = excluded.display_name,
                email = excluded.email,
                status = excluded.status,
                created_at_ms = excluded.created_at_ms,
                updated_at_ms = excluded.updated_at_ms",
        )
        .bind(i64::try_from(record.user_id)?)
        .bind(i64::try_from(record.tenant_id)?)
        .bind(i64::try_from(record.organization_id)?)
        .bind(&record.external_user_ref)
        .bind(&record.username)
        .bind(&record.display_name)
        .bind(&record.email)
        .bind(&record.status)
        .bind(i64::try_from(record.created_at_ms)?)
        .bind(i64::try_from(record.updated_at_ms)?)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    async fn list_identity_user_records(&self) -> Result<Vec<IdentityUserRecord>> {
        let rows = sqlx::query(
            "SELECT user_id, tenant_id, organization_id, external_user_ref, username,
                    display_name, email, status, created_at_ms, updated_at_ms
             FROM ai_user
             ORDER BY created_at_ms DESC, user_id",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(decode_identity_user_row).collect()
    }

    async fn find_identity_user_record(&self, user_id: u64) -> Result<Option<IdentityUserRecord>> {
        let row = sqlx::query(
            "SELECT user_id, tenant_id, organization_id, external_user_ref, username,
                    display_name, email, status, created_at_ms, updated_at_ms
             FROM ai_user
             WHERE user_id = $1",
        )
        .bind(i64::try_from(user_id)?)
        .fetch_optional(&self.pool)
        .await?;
        row.map(decode_identity_user_row).transpose()
    }

    async fn insert_canonical_api_key_record(
        &self,
        record: &CanonicalApiKeyRecord,
    ) -> Result<CanonicalApiKeyRecord> {
        sqlx::query(
            "INSERT INTO ai_api_key (
                api_key_id, tenant_id, organization_id, user_id, key_prefix, key_hash,
                display_name, status, expires_at_ms, last_used_at_ms, rotated_from_api_key_id,
                created_at_ms, updated_at_ms
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
             ON CONFLICT(api_key_id) DO UPDATE SET
                tenant_id = excluded.tenant_id,
                organization_id = excluded.organization_id,
                user_id = excluded.user_id,
                key_prefix = excluded.key_prefix,
                key_hash = excluded.key_hash,
                display_name = excluded.display_name,
                status = excluded.status,
                expires_at_ms = excluded.expires_at_ms,
                last_used_at_ms = excluded.last_used_at_ms,
                rotated_from_api_key_id = excluded.rotated_from_api_key_id,
                created_at_ms = excluded.created_at_ms,
                updated_at_ms = excluded.updated_at_ms",
        )
        .bind(i64::try_from(record.api_key_id)?)
        .bind(i64::try_from(record.tenant_id)?)
        .bind(i64::try_from(record.organization_id)?)
        .bind(i64::try_from(record.user_id)?)
        .bind(&record.key_prefix)
        .bind(&record.key_hash)
        .bind(&record.display_name)
        .bind(&record.status)
        .bind(record.expires_at_ms.map(i64::try_from).transpose()?)
        .bind(record.last_used_at_ms.map(i64::try_from).transpose()?)
        .bind(
            record
                .rotated_from_api_key_id
                .map(i64::try_from)
                .transpose()?,
        )
        .bind(i64::try_from(record.created_at_ms)?)
        .bind(i64::try_from(record.updated_at_ms)?)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    async fn find_canonical_api_key_record_by_hash(
        &self,
        key_hash: &str,
    ) -> Result<Option<CanonicalApiKeyRecord>> {
        let row = sqlx::query(
            "SELECT api_key_id, tenant_id, organization_id, user_id, key_prefix, key_hash,
                    display_name, status, expires_at_ms, last_used_at_ms,
                    rotated_from_api_key_id, created_at_ms, updated_at_ms
             FROM ai_api_key
             WHERE key_hash = $1",
        )
        .bind(key_hash)
        .fetch_optional(&self.pool)
        .await?;
        row.map(decode_canonical_api_key_row).transpose()
    }

    async fn insert_identity_binding_record(
        &self,
        record: &IdentityBindingRecord,
    ) -> Result<IdentityBindingRecord> {
        sqlx::query(
            "INSERT INTO ai_identity_binding (
                identity_binding_id, tenant_id, organization_id, user_id, binding_type,
                issuer, subject, platform, owner, external_ref, status, created_at_ms, updated_at_ms
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
             ON CONFLICT(identity_binding_id) DO UPDATE SET
                tenant_id = excluded.tenant_id,
                organization_id = excluded.organization_id,
                user_id = excluded.user_id,
                binding_type = excluded.binding_type,
                issuer = excluded.issuer,
                subject = excluded.subject,
                platform = excluded.platform,
                owner = excluded.owner,
                external_ref = excluded.external_ref,
                status = excluded.status,
                created_at_ms = excluded.created_at_ms,
                updated_at_ms = excluded.updated_at_ms",
        )
        .bind(i64::try_from(record.identity_binding_id)?)
        .bind(i64::try_from(record.tenant_id)?)
        .bind(i64::try_from(record.organization_id)?)
        .bind(i64::try_from(record.user_id)?)
        .bind(&record.binding_type)
        .bind(&record.issuer)
        .bind(&record.subject)
        .bind(&record.platform)
        .bind(&record.owner)
        .bind(&record.external_ref)
        .bind(&record.status)
        .bind(i64::try_from(record.created_at_ms)?)
        .bind(i64::try_from(record.updated_at_ms)?)
        .execute(&self.pool)
        .await?;
        Ok(record.clone())
    }

    async fn find_identity_binding_record(
        &self,
        binding_type: &str,
        issuer: Option<&str>,
        subject: Option<&str>,
    ) -> Result<Option<IdentityBindingRecord>> {
        let row = sqlx::query(
            "SELECT identity_binding_id, tenant_id, organization_id, user_id, binding_type,
                    issuer, subject, platform, owner, external_ref, status, created_at_ms, updated_at_ms
             FROM ai_identity_binding
             WHERE binding_type = $1
               AND issuer IS NOT DISTINCT FROM $2
               AND subject IS NOT DISTINCT FROM $3
             ORDER BY updated_at_ms DESC, identity_binding_id DESC
             LIMIT 1",
        )
        .bind(binding_type)
        .bind(issuer)
        .bind(subject)
        .fetch_optional(&self.pool)
        .await?;
        row.map(decode_identity_binding_row).transpose()
    }
}

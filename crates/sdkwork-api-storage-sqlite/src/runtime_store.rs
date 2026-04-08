use super::*;

impl SqliteAdminStore {
    pub async fn insert_extension_installation(
        &self,
        installation: &ExtensionInstallation,
    ) -> Result<ExtensionInstallation> {
        sqlx::query(
            "INSERT INTO ai_extension_installations (installation_id, extension_id, runtime, enabled, entrypoint, config_json) VALUES (?, ?, ?, ?, ?, ?)
             ON CONFLICT(installation_id) DO UPDATE SET extension_id = excluded.extension_id, runtime = excluded.runtime, enabled = excluded.enabled, entrypoint = excluded.entrypoint, config_json = excluded.config_json",
        )
        .bind(&installation.installation_id)
        .bind(&installation.extension_id)
        .bind(installation.runtime.as_str())
        .bind(if installation.enabled { 1_i64 } else { 0_i64 })
        .bind(&installation.entrypoint)
        .bind(encode_extension_config(&installation.config)?)
        .execute(&self.pool)
        .await?;
        Ok(installation.clone())
    }

    pub async fn list_extension_installations(&self) -> Result<Vec<ExtensionInstallation>> {
        let rows = sqlx::query_as::<_, (String, String, String, i64, Option<String>, String)>(
            "SELECT installation_id, extension_id, runtime, enabled, entrypoint, config_json
             FROM ai_extension_installations
             ORDER BY installation_id",
        )
        .fetch_all(&self.pool)
        .await?;

        let mut installations = Vec::with_capacity(rows.len());
        for (installation_id, extension_id, runtime, enabled, entrypoint, config_json) in rows {
            installations.push(ExtensionInstallation {
                installation_id,
                extension_id,
                runtime: ExtensionRuntime::from_str(&runtime)?,
                enabled: enabled != 0,
                entrypoint,
                config: decode_extension_config(&config_json)?,
            });
        }
        Ok(installations)
    }

    pub async fn insert_extension_instance(
        &self,
        instance: &ExtensionInstance,
    ) -> Result<ExtensionInstance> {
        sqlx::query(
            "INSERT INTO ai_extension_instances (instance_id, installation_id, extension_id, enabled, base_url, credential_ref, config_json) VALUES (?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(instance_id) DO UPDATE SET installation_id = excluded.installation_id, extension_id = excluded.extension_id, enabled = excluded.enabled, base_url = excluded.base_url, credential_ref = excluded.credential_ref, config_json = excluded.config_json",
        )
        .bind(&instance.instance_id)
        .bind(&instance.installation_id)
        .bind(&instance.extension_id)
        .bind(if instance.enabled { 1_i64 } else { 0_i64 })
        .bind(&instance.base_url)
        .bind(&instance.credential_ref)
        .bind(encode_extension_config(&instance.config)?)
        .execute(&self.pool)
        .await?;
        Ok(instance.clone())
    }

    pub async fn list_extension_instances(&self) -> Result<Vec<ExtensionInstance>> {
        let rows = sqlx::query_as::<_, (String, String, String, i64, Option<String>, Option<String>, String)>(
            "SELECT instance_id, installation_id, extension_id, enabled, base_url, credential_ref, config_json
             FROM ai_extension_instances
             ORDER BY instance_id",
        )
        .fetch_all(&self.pool)
        .await?;

        let mut instances = Vec::with_capacity(rows.len());
        for (
            instance_id,
            installation_id,
            extension_id,
            enabled,
            base_url,
            credential_ref,
            config_json,
        ) in rows
        {
            instances.push(ExtensionInstance {
                instance_id,
                installation_id,
                extension_id,
                enabled: enabled != 0,
                base_url,
                credential_ref,
                config: decode_extension_config(&config_json)?,
            });
        }
        Ok(instances)
    }

    pub async fn upsert_service_runtime_node(
        &self,
        record: &ServiceRuntimeNodeRecord,
    ) -> Result<ServiceRuntimeNodeRecord> {
        sqlx::query(
            "INSERT INTO ai_service_runtime_nodes (node_id, service_kind, started_at_ms, last_seen_at_ms)
             VALUES (?, ?, ?, ?)
             ON CONFLICT(node_id) DO UPDATE SET
                 service_kind = excluded.service_kind,
                 started_at_ms = excluded.started_at_ms,
                 last_seen_at_ms = excluded.last_seen_at_ms",
        )
        .bind(&record.node_id)
        .bind(&record.service_kind)
        .bind(record.started_at_ms as i64)
        .bind(record.last_seen_at_ms as i64)
        .execute(&self.pool)
        .await?;

        Ok(record.clone())
    }

    pub async fn list_service_runtime_nodes(&self) -> Result<Vec<ServiceRuntimeNodeRecord>> {
        let rows = sqlx::query_as::<_, (String, String, i64, i64)>(
            "SELECT node_id, service_kind, started_at_ms, last_seen_at_ms
             FROM ai_service_runtime_nodes
             ORDER BY node_id",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(node_id, service_kind, started_at_ms, last_seen_at_ms)| {
                ServiceRuntimeNodeRecord {
                    node_id,
                    service_kind,
                    started_at_ms: started_at_ms as u64,
                    last_seen_at_ms: last_seen_at_ms as u64,
                }
            })
            .collect())
    }

    pub async fn insert_extension_runtime_rollout(
        &self,
        rollout: &ExtensionRuntimeRolloutRecord,
    ) -> Result<ExtensionRuntimeRolloutRecord> {
        sqlx::query(
            "INSERT INTO ai_extension_runtime_rollouts (
                rollout_id,
                scope,
                requested_extension_id,
                requested_instance_id,
                resolved_extension_id,
                created_by,
                created_at_ms,
                deadline_at_ms
             ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(rollout_id) DO UPDATE SET
                 scope = excluded.scope,
                 requested_extension_id = excluded.requested_extension_id,
                 requested_instance_id = excluded.requested_instance_id,
                 resolved_extension_id = excluded.resolved_extension_id,
                 created_by = excluded.created_by,
                 created_at_ms = excluded.created_at_ms,
                 deadline_at_ms = excluded.deadline_at_ms",
        )
        .bind(&rollout.rollout_id)
        .bind(&rollout.scope)
        .bind(&rollout.requested_extension_id)
        .bind(&rollout.requested_instance_id)
        .bind(&rollout.resolved_extension_id)
        .bind(&rollout.created_by)
        .bind(rollout.created_at_ms as i64)
        .bind(rollout.deadline_at_ms as i64)
        .execute(&self.pool)
        .await?;

        Ok(rollout.clone())
    }

    pub async fn find_extension_runtime_rollout(
        &self,
        rollout_id: &str,
    ) -> Result<Option<ExtensionRuntimeRolloutRecord>> {
        let row = sqlx::query_as::<
            _,
            (
                String,
                String,
                Option<String>,
                Option<String>,
                Option<String>,
                String,
                i64,
                i64,
            ),
        >(
            "SELECT rollout_id, scope, requested_extension_id, requested_instance_id, resolved_extension_id, created_by, created_at_ms, deadline_at_ms
             FROM ai_extension_runtime_rollouts
             WHERE rollout_id = ?",
        )
        .bind(rollout_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(
            |(
                rollout_id,
                scope,
                requested_extension_id,
                requested_instance_id,
                resolved_extension_id,
                created_by,
                created_at_ms,
                deadline_at_ms,
            )| ExtensionRuntimeRolloutRecord {
                rollout_id,
                scope,
                requested_extension_id,
                requested_instance_id,
                resolved_extension_id,
                created_by,
                created_at_ms: created_at_ms as u64,
                deadline_at_ms: deadline_at_ms as u64,
            },
        ))
    }

    pub async fn list_extension_runtime_rollouts(
        &self,
    ) -> Result<Vec<ExtensionRuntimeRolloutRecord>> {
        let rows = sqlx::query_as::<
            _,
            (
                String,
                String,
                Option<String>,
                Option<String>,
                Option<String>,
                String,
                i64,
                i64,
            ),
        >(
            "SELECT rollout_id, scope, requested_extension_id, requested_instance_id, resolved_extension_id, created_by, created_at_ms, deadline_at_ms
             FROM ai_extension_runtime_rollouts
             ORDER BY created_at_ms DESC, rollout_id",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(
                |(
                    rollout_id,
                    scope,
                    requested_extension_id,
                    requested_instance_id,
                    resolved_extension_id,
                    created_by,
                    created_at_ms,
                    deadline_at_ms,
                )| ExtensionRuntimeRolloutRecord {
                    rollout_id,
                    scope,
                    requested_extension_id,
                    requested_instance_id,
                    resolved_extension_id,
                    created_by,
                    created_at_ms: created_at_ms as u64,
                    deadline_at_ms: deadline_at_ms as u64,
                },
            )
            .collect())
    }

    pub async fn insert_extension_runtime_rollout_participant(
        &self,
        participant: &ExtensionRuntimeRolloutParticipantRecord,
    ) -> Result<ExtensionRuntimeRolloutParticipantRecord> {
        sqlx::query(
            "INSERT INTO ai_extension_runtime_rollout_participants (
                rollout_id,
                node_id,
                service_kind,
                status,
                message,
                updated_at_ms
             ) VALUES (?, ?, ?, ?, ?, ?)
             ON CONFLICT(rollout_id, node_id) DO UPDATE SET
                 service_kind = excluded.service_kind,
                 status = excluded.status,
                 message = excluded.message,
                 updated_at_ms = excluded.updated_at_ms",
        )
        .bind(&participant.rollout_id)
        .bind(&participant.node_id)
        .bind(&participant.service_kind)
        .bind(&participant.status)
        .bind(&participant.message)
        .bind(participant.updated_at_ms as i64)
        .execute(&self.pool)
        .await?;

        Ok(participant.clone())
    }

    pub async fn list_extension_runtime_rollout_participants(
        &self,
        rollout_id: &str,
    ) -> Result<Vec<ExtensionRuntimeRolloutParticipantRecord>> {
        let rows = sqlx::query_as::<_, (String, String, String, String, Option<String>, i64)>(
            "SELECT rollout_id, node_id, service_kind, status, message, updated_at_ms
             FROM ai_extension_runtime_rollout_participants
             WHERE rollout_id = ?
             ORDER BY node_id",
        )
        .bind(rollout_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(
                |(rollout_id, node_id, service_kind, status, message, updated_at_ms)| {
                    ExtensionRuntimeRolloutParticipantRecord {
                        rollout_id,
                        node_id,
                        service_kind,
                        status,
                        message,
                        updated_at_ms: updated_at_ms as u64,
                    }
                },
            )
            .collect())
    }

    pub async fn list_pending_extension_runtime_rollout_participants_for_node(
        &self,
        node_id: &str,
    ) -> Result<Vec<ExtensionRuntimeRolloutParticipantRecord>> {
        let rows = sqlx::query_as::<_, (String, String, String, String, Option<String>, i64)>(
            "SELECT participants.rollout_id, participants.node_id, participants.service_kind, participants.status, participants.message, participants.updated_at_ms
             FROM ai_extension_runtime_rollout_participants AS participants
             INNER JOIN ai_extension_runtime_rollouts AS rollouts ON rollouts.rollout_id = participants.rollout_id
             WHERE participants.node_id = ?
               AND participants.status = 'pending'
             ORDER BY rollouts.created_at_ms, participants.rollout_id",
        )
        .bind(node_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(
                |(rollout_id, node_id, service_kind, status, message, updated_at_ms)| {
                    ExtensionRuntimeRolloutParticipantRecord {
                        rollout_id,
                        node_id,
                        service_kind,
                        status,
                        message,
                        updated_at_ms: updated_at_ms as u64,
                    }
                },
            )
            .collect())
    }

    pub async fn transition_extension_runtime_rollout_participant(
        &self,
        rollout_id: &str,
        node_id: &str,
        from_status: &str,
        to_status: &str,
        message: Option<&str>,
        updated_at_ms: u64,
    ) -> Result<bool> {
        let result = sqlx::query(
            "UPDATE ai_extension_runtime_rollout_participants
             SET status = ?, message = ?, updated_at_ms = ?
             WHERE rollout_id = ? AND node_id = ? AND status = ?",
        )
        .bind(to_status)
        .bind(message)
        .bind(updated_at_ms as i64)
        .bind(rollout_id)
        .bind(node_id)
        .bind(from_status)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() == 1)
    }

    pub async fn insert_standalone_config_rollout(
        &self,
        rollout: &StandaloneConfigRolloutRecord,
    ) -> Result<StandaloneConfigRolloutRecord> {
        sqlx::query(
            "INSERT INTO ai_standalone_config_rollouts (
                rollout_id,
                requested_service_kind,
                created_by,
                created_at_ms,
                deadline_at_ms
             ) VALUES (?, ?, ?, ?, ?)
             ON CONFLICT(rollout_id) DO UPDATE SET
                 requested_service_kind = excluded.requested_service_kind,
                 created_by = excluded.created_by,
                 created_at_ms = excluded.created_at_ms,
                 deadline_at_ms = excluded.deadline_at_ms",
        )
        .bind(&rollout.rollout_id)
        .bind(&rollout.requested_service_kind)
        .bind(&rollout.created_by)
        .bind(rollout.created_at_ms as i64)
        .bind(rollout.deadline_at_ms as i64)
        .execute(&self.pool)
        .await?;

        Ok(rollout.clone())
    }

    pub async fn find_standalone_config_rollout(
        &self,
        rollout_id: &str,
    ) -> Result<Option<StandaloneConfigRolloutRecord>> {
        let row = sqlx::query_as::<_, (String, Option<String>, String, i64, i64)>(
            "SELECT rollout_id, requested_service_kind, created_by, created_at_ms, deadline_at_ms
             FROM ai_standalone_config_rollouts
             WHERE rollout_id = ?",
        )
        .bind(rollout_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(
            |(rollout_id, requested_service_kind, created_by, created_at_ms, deadline_at_ms)| {
                StandaloneConfigRolloutRecord {
                    rollout_id,
                    requested_service_kind,
                    created_by,
                    created_at_ms: created_at_ms as u64,
                    deadline_at_ms: deadline_at_ms as u64,
                }
            },
        ))
    }

    pub async fn list_standalone_config_rollouts(
        &self,
    ) -> Result<Vec<StandaloneConfigRolloutRecord>> {
        let rows = sqlx::query_as::<_, (String, Option<String>, String, i64, i64)>(
            "SELECT rollout_id, requested_service_kind, created_by, created_at_ms, deadline_at_ms
             FROM ai_standalone_config_rollouts
             ORDER BY created_at_ms DESC, rollout_id",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(
                |(
                    rollout_id,
                    requested_service_kind,
                    created_by,
                    created_at_ms,
                    deadline_at_ms,
                )| {
                    StandaloneConfigRolloutRecord {
                        rollout_id,
                        requested_service_kind,
                        created_by,
                        created_at_ms: created_at_ms as u64,
                        deadline_at_ms: deadline_at_ms as u64,
                    }
                },
            )
            .collect())
    }

    pub async fn insert_standalone_config_rollout_participant(
        &self,
        participant: &StandaloneConfigRolloutParticipantRecord,
    ) -> Result<StandaloneConfigRolloutParticipantRecord> {
        sqlx::query(
            "INSERT INTO ai_standalone_config_rollout_participants (
                rollout_id,
                node_id,
                service_kind,
                status,
                message,
                updated_at_ms
             ) VALUES (?, ?, ?, ?, ?, ?)
             ON CONFLICT(rollout_id, node_id) DO UPDATE SET
                 service_kind = excluded.service_kind,
                 status = excluded.status,
                 message = excluded.message,
                 updated_at_ms = excluded.updated_at_ms",
        )
        .bind(&participant.rollout_id)
        .bind(&participant.node_id)
        .bind(&participant.service_kind)
        .bind(&participant.status)
        .bind(&participant.message)
        .bind(participant.updated_at_ms as i64)
        .execute(&self.pool)
        .await?;

        Ok(participant.clone())
    }

    pub async fn list_standalone_config_rollout_participants(
        &self,
        rollout_id: &str,
    ) -> Result<Vec<StandaloneConfigRolloutParticipantRecord>> {
        let rows = sqlx::query_as::<_, (String, String, String, String, Option<String>, i64)>(
            "SELECT rollout_id, node_id, service_kind, status, message, updated_at_ms
             FROM ai_standalone_config_rollout_participants
             WHERE rollout_id = ?
             ORDER BY node_id",
        )
        .bind(rollout_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(
                |(rollout_id, node_id, service_kind, status, message, updated_at_ms)| {
                    StandaloneConfigRolloutParticipantRecord {
                        rollout_id,
                        node_id,
                        service_kind,
                        status,
                        message,
                        updated_at_ms: updated_at_ms as u64,
                    }
                },
            )
            .collect())
    }

    pub async fn list_pending_standalone_config_rollout_participants_for_node(
        &self,
        node_id: &str,
    ) -> Result<Vec<StandaloneConfigRolloutParticipantRecord>> {
        let rows = sqlx::query_as::<_, (String, String, String, String, Option<String>, i64)>(
            "SELECT participants.rollout_id, participants.node_id, participants.service_kind, participants.status, participants.message, participants.updated_at_ms
             FROM ai_standalone_config_rollout_participants AS participants
             INNER JOIN ai_standalone_config_rollouts AS rollouts ON rollouts.rollout_id = participants.rollout_id
             WHERE participants.node_id = ?
               AND participants.status = 'pending'
             ORDER BY rollouts.created_at_ms, participants.rollout_id",
        )
        .bind(node_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(
                |(rollout_id, node_id, service_kind, status, message, updated_at_ms)| {
                    StandaloneConfigRolloutParticipantRecord {
                        rollout_id,
                        node_id,
                        service_kind,
                        status,
                        message,
                        updated_at_ms: updated_at_ms as u64,
                    }
                },
            )
            .collect())
    }

    pub async fn transition_standalone_config_rollout_participant(
        &self,
        rollout_id: &str,
        node_id: &str,
        from_status: &str,
        to_status: &str,
        message: Option<&str>,
        updated_at_ms: u64,
    ) -> Result<bool> {
        let result = sqlx::query(
            "UPDATE ai_standalone_config_rollout_participants
             SET status = ?, message = ?, updated_at_ms = ?
             WHERE rollout_id = ? AND node_id = ? AND status = ?",
        )
        .bind(to_status)
        .bind(message)
        .bind(updated_at_ms as i64)
        .bind(rollout_id)
        .bind(node_id)
        .bind(from_status)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() == 1)
    }
}

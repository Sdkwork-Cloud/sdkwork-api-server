use super::*;

pub(crate) fn ensure_sqlite_parent_directory(url: &str) -> Result<()> {
    let Some(path) = sqlite_path_from_url(url) else {
        return Ok(());
    };
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    if parent.as_os_str().is_empty() || parent == Path::new(".") {
        return Ok(());
    }
    fs::create_dir_all(parent)?;
    if !path.exists() {
        let _ = fs::File::create(&path)?;
    }
    Ok(())
}

pub(crate) fn sqlite_path_from_url(url: &str) -> Option<PathBuf> {
    let lowered = url.to_ascii_lowercase();
    if !lowered.starts_with("sqlite:") || lowered.contains(":memory:") {
        return None;
    }

    let query_start = url.find('?').unwrap_or(url.len());
    let sqlite_part = &url[..query_start];
    let raw_path = sqlite_part
        .strip_prefix("sqlite://")
        .or_else(|| sqlite_part.strip_prefix("sqlite:"))
        .unwrap_or(sqlite_part);

    if raw_path.is_empty() {
        return None;
    }

    let normalized_path = raw_path
        .strip_prefix('/')
        .filter(|candidate| has_windows_drive_prefix(candidate))
        .unwrap_or(raw_path);

    Some(PathBuf::from(normalized_path))
}

pub(crate) fn has_windows_drive_prefix(path: &str) -> bool {
    let bytes = path.as_bytes();
    bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && (bytes[2] == b'/' || bytes[2] == b'\\')
}

pub(crate) async fn ensure_sqlite_column(
    pool: &SqlitePool,
    table_name: &str,
    column_name: &str,
    column_definition: &str,
) -> Result<()> {
    let query = format!("PRAGMA table_info({table_name})");
    let rows = sqlx::query_as::<_, (i64, String, String, i64, Option<String>, i64)>(&query)
        .fetch_all(pool)
        .await?;

    if rows.iter().any(|(_, name, _, _, _, _)| name == column_name) {
        return Ok(());
    }

    let alter = format!("ALTER TABLE {table_name} ADD COLUMN {column_definition}");
    sqlx::query(&alter).execute(pool).await?;
    Ok(())
}

pub(crate) async fn sqlite_object_type(
    pool: &SqlitePool,
    object_name: &str,
) -> Result<Option<String>> {
    let row = sqlx::query_as::<_, (String,)>("SELECT type FROM sqlite_master WHERE name = ?")
        .bind(object_name)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|(object_type,)| object_type))
}

pub(crate) async fn ensure_sqlite_column_if_table_exists(
    pool: &SqlitePool,
    table_name: &str,
    column_name: &str,
    column_definition: &str,
) -> Result<()> {
    if sqlite_object_type(pool, table_name).await?.as_deref() == Some("table") {
        ensure_sqlite_column(pool, table_name, column_name, column_definition).await?;
    }
    Ok(())
}

pub(crate) async fn sqlite_table_columns(
    pool: &SqlitePool,
    table_name: &str,
) -> Result<Vec<String>> {
    let query = format!("PRAGMA table_info({table_name})");
    let rows = sqlx::query_as::<_, (i64, String, String, i64, Option<String>, i64)>(&query)
        .fetch_all(pool)
        .await?;
    Ok(rows.into_iter().map(|(_, name, _, _, _, _)| name).collect())
}

pub(crate) async fn migrate_sqlite_legacy_table_with_common_columns(
    pool: &SqlitePool,
    legacy_table_name: &str,
    canonical_table_name: &str,
) -> Result<()> {
    if sqlite_object_type(pool, legacy_table_name)
        .await?
        .as_deref()
        != Some("table")
    {
        return Ok(());
    }

    let legacy_columns = sqlite_table_columns(pool, legacy_table_name).await?;
    let canonical_columns = sqlite_table_columns(pool, canonical_table_name).await?;
    let common_columns: Vec<String> = canonical_columns
        .into_iter()
        .filter(|column_name| legacy_columns.contains(column_name))
        .collect();

    if !common_columns.is_empty() {
        let column_list = common_columns.join(", ");
        let insert = format!(
            "INSERT OR IGNORE INTO {canonical_table_name} ({column_list})
             SELECT {column_list} FROM {legacy_table_name}"
        );
        sqlx::query(&insert).execute(pool).await?;
    }

    let drop_table = format!("DROP TABLE {legacy_table_name}");
    sqlx::query(&drop_table).execute(pool).await?;
    Ok(())
}

pub(crate) async fn recreate_sqlite_compatibility_view(
    pool: &SqlitePool,
    legacy_name: &str,
    select_sql: &str,
) -> Result<()> {
    match sqlite_object_type(pool, legacy_name).await?.as_deref() {
        Some("table") => {
            let drop_table = format!("DROP TABLE {legacy_name}");
            sqlx::query(&drop_table).execute(pool).await?;
        }
        Some("view") => {
            let drop_view = format!("DROP VIEW {legacy_name}");
            sqlx::query(&drop_view).execute(pool).await?;
        }
        _ => {}
    }

    let create_view = format!("CREATE VIEW {legacy_name} AS {select_sql}");
    sqlx::query(&create_view).execute(pool).await?;
    Ok(())
}

pub(crate) fn encode_extension_config(config: &Value) -> Result<String> {
    Ok(serde_json::to_string(config)?)
}

pub(crate) fn decode_extension_config(config_json: &str) -> Result<Value> {
    Ok(serde_json::from_str(config_json)?)
}

pub(crate) fn encode_routing_assessments(
    assessments: &[RoutingCandidateAssessment],
) -> Result<String> {
    Ok(serde_json::to_string(assessments)?)
}

pub(crate) fn decode_routing_assessments(
    assessments_json: &str,
) -> Result<Vec<RoutingCandidateAssessment>> {
    Ok(serde_json::from_str(assessments_json)?)
}

pub(crate) fn encode_string_list(values: &[String]) -> Result<String> {
    Ok(serde_json::to_string(values)?)
}

pub(crate) fn decode_string_list(values_json: &str) -> Result<Vec<String>> {
    Ok(serde_json::from_str(values_json)?)
}

pub(crate) fn current_timestamp_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| i64::try_from(duration.as_millis()).unwrap_or(i64::MAX))
        .unwrap_or_default()
}

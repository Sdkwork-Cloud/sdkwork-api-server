use super::*;

pub(super) async fn assert_pg_column(
    pool: &PgPool,
    table_name: &str,
    column_name: &str,
    data_type: &str,
    nullable: bool,
    default_contains: Option<&str>,
) {
    let row: (String, String, Option<String>) = sqlx::query_as(
        "select data_type, is_nullable, column_default
         from information_schema.columns
         where table_schema = 'public'
           and table_name = $1
           and column_name = $2",
    )
    .bind(table_name)
    .bind(column_name)
    .fetch_one(pool)
    .await
    .unwrap();

    assert_eq!(row.0, data_type);
    assert_eq!(row.1 == "YES", nullable);
    if let Some(expected) = default_contains {
        assert!(
            row.2
                .as_deref()
                .is_some_and(|value| value.contains(expected)),
            "expected default for {table_name}.{column_name} to contain {expected:?}, got {:?}",
            row.2
        );
    }
}

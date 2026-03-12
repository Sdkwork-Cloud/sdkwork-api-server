use sdkwork_api_storage_core::StorageDialect;

#[test]
fn dialect_reports_sqlite_name() {
    assert_eq!(StorageDialect::Sqlite.as_str(), "sqlite");
}

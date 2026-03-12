use sdkwork_api_storage_libsql::dialect_name;

#[test]
fn reports_libsql_name() {
    assert_eq!(dialect_name(), "libsql");
}

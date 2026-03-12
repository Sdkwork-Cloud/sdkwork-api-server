use sdkwork_api_storage_postgres::dialect_name;

#[test]
fn reports_postgres_name() {
    assert_eq!(dialect_name(), "postgres");
}

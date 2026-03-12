use sdkwork_api_storage_mysql::dialect_name;

#[test]
fn reports_mysql_name() {
    assert_eq!(dialect_name(), "mysql");
}

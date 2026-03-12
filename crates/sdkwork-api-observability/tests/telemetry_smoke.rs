use sdkwork_api_observability::service_name;

#[test]
fn exposes_service_name() {
    assert_eq!(service_name("gateway-service"), "gateway-service");
}

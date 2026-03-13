use sdkwork_api_app_gateway::create_fine_tuning_job;

#[test]
fn returns_fine_tuning_job_object() {
    let response = create_fine_tuning_job("tenant-1", "project-1", "gpt-4.1-mini").unwrap();
    assert_eq!(response.object, "fine_tuning.job");
    assert_eq!(response.model, "gpt-4.1-mini");
}

use sdkwork_api_app_gateway::create_fine_tuning_job;
use sdkwork_api_contract_openai::fine_tuning::CreateFineTuningJobRequest;

#[test]
fn returns_fine_tuning_job_object() {
    let request = CreateFineTuningJobRequest::new("file_local_0000000000000001", "gpt-4.1-mini");
    let response = create_fine_tuning_job("tenant-1", "project-1", &request).unwrap();
    assert_eq!(response.object, "fine_tuning.job");
    assert_eq!(response.model, "gpt-4.1-mini");
    assert!(response.id.starts_with("ftjob_local_"));
}

#[test]
fn lists_fine_tuning_jobs() {
    let response = sdkwork_api_app_gateway::list_fine_tuning_jobs("tenant-1", "project-1").unwrap();
    assert_eq!(response.object, "list");
    assert!(response.data.is_empty());
}

#[test]
fn retrieve_requires_persisted_fine_tuning_job_state() {
    let error = sdkwork_api_app_gateway::get_fine_tuning_job(
        "tenant-1",
        "project-1",
        "ftjob_local_0000000000000001",
    )
    .unwrap_err();
    assert!(error.to_string().contains("not found"));
}

#[test]
fn cancel_requires_persisted_fine_tuning_job_state() {
    let error = sdkwork_api_app_gateway::cancel_fine_tuning_job(
        "tenant-1",
        "project-1",
        "ftjob_local_0000000000000001",
    )
    .unwrap_err();
    assert!(error.to_string().contains("not found"));
}

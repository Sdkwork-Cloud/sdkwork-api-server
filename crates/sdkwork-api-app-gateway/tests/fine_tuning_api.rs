use sdkwork_api_app_gateway::{
    cancel_fine_tuning_job, create_fine_tuning_job, get_fine_tuning_job, list_fine_tuning_jobs,
};
use sdkwork_api_contract_openai::fine_tuning::CreateFineTuningJobRequest;

fn assert_error_contains<T: std::fmt::Debug, E: std::fmt::Display>(
    result: Result<T, E>,
    expected: &str,
) {
    let error = result.expect_err("expected error");
    assert!(
        error.to_string().contains(expected),
        "expected error containing `{expected}`, got `{error}`"
    );
}

#[test]
fn local_fine_tuning_fallback_requires_upstream_provider() {
    let request = CreateFineTuningJobRequest::new("file_local_0000000000000001", "gpt-4.1-mini");
    assert_error_contains(
        create_fine_tuning_job("tenant-1", "project-1", &request),
        "Local fine-tuning job fallback is not supported",
    );
    assert_error_contains(
        list_fine_tuning_jobs("tenant-1", "project-1"),
        "Local fine-tuning job listing fallback is not supported",
    );
}

#[test]
fn local_fine_tuning_fallback_requires_persisted_job_state() {
    assert_error_contains(
        get_fine_tuning_job("tenant-1", "project-1", "ftjob_local_0000000000000001"),
        "fine tuning job not found",
    );
    assert_error_contains(
        cancel_fine_tuning_job("tenant-1", "project-1", "ftjob_local_0000000000000001"),
        "fine tuning job not found",
    );
}

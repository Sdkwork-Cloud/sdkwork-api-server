use sdkwork_api_app_gateway::create_eval;
use sdkwork_api_contract_openai::evals::CreateEvalRequest;

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
fn local_eval_fallback_requires_upstream_provider() {
    let request = CreateEvalRequest::new("qa-benchmark", "file_local_0000000000000001");
    assert_error_contains(
        create_eval("tenant-1", "project-1", &request),
        "Local eval fallback is not supported",
    );
}

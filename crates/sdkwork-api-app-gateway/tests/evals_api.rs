use sdkwork_api_app_gateway::create_eval;
use sdkwork_api_contract_openai::evals::CreateEvalRequest;

#[test]
fn returns_eval_object() {
    let request = CreateEvalRequest::new("qa-benchmark", "file_local_0000000000000001");
    let response = create_eval("tenant-1", "project-1", &request).unwrap();
    assert_eq!(response.object, "eval");
    assert_eq!(response.name, "qa-benchmark");
}

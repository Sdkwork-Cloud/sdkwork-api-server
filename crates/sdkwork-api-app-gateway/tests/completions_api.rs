use sdkwork_api_app_gateway::create_completion;

#[test]
fn completion_requires_generation_backend() {
    let error = create_completion("tenant-1", "project-1", "gpt-3.5-turbo-instruct").unwrap_err();
    assert!(error.to_string().contains("not supported"));
}

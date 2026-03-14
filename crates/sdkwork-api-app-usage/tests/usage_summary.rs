use sdkwork_api_app_usage::summarize_usage_records;
use sdkwork_api_domain_usage::UsageRecord;

#[test]
fn summarizes_usage_records_by_project_provider_and_model() {
    let records = vec![
        UsageRecord::new("project-1", "gpt-4.1", "provider-openai"),
        UsageRecord::new("project-1", "gpt-4.1", "provider-openai"),
        UsageRecord::new("project-1", "text-embedding-3-large", "provider-openai"),
        UsageRecord::new("project-2", "gpt-4.1", "provider-openrouter"),
    ];

    let summary = summarize_usage_records(&records);

    assert_eq!(summary.total_requests, 4);
    assert_eq!(summary.project_count, 2);
    assert_eq!(summary.model_count, 2);
    assert_eq!(summary.provider_count, 2);

    assert_eq!(summary.projects.len(), 2);
    assert_eq!(summary.projects[0].project_id, "project-1");
    assert_eq!(summary.projects[0].request_count, 3);
    assert_eq!(summary.projects[1].project_id, "project-2");
    assert_eq!(summary.projects[1].request_count, 1);

    assert_eq!(summary.providers.len(), 2);
    assert_eq!(summary.providers[0].provider, "provider-openai");
    assert_eq!(summary.providers[0].request_count, 3);
    assert_eq!(summary.providers[0].project_count, 1);
    assert_eq!(summary.providers[1].provider, "provider-openrouter");
    assert_eq!(summary.providers[1].request_count, 1);
    assert_eq!(summary.providers[1].project_count, 1);

    assert_eq!(summary.models.len(), 2);
    assert_eq!(summary.models[0].model, "gpt-4.1");
    assert_eq!(summary.models[0].request_count, 3);
    assert_eq!(summary.models[0].provider_count, 2);
    assert_eq!(summary.models[1].model, "text-embedding-3-large");
    assert_eq!(summary.models[1].request_count, 1);
    assert_eq!(summary.models[1].provider_count, 1);
}

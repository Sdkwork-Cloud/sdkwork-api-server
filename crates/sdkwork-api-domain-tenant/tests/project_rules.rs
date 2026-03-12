use sdkwork_api_domain_tenant::Project;

#[test]
fn project_belongs_to_tenant() {
    let project = Project::new("tenant-1", "project-1", "Gateway");
    assert_eq!(project.tenant_id, "tenant-1");
}

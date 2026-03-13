use anyhow::Result;
use sdkwork_api_domain_tenant::{Project, Tenant};
use sdkwork_api_storage_core::AdminStore;

pub fn service_name() -> &'static str {
    "tenant-service"
}

pub fn create_tenant(id: &str, name: &str) -> Result<Tenant> {
    Ok(Tenant::new(id, name))
}

pub fn create_project(tenant_id: &str, id: &str, name: &str) -> Result<Project> {
    Ok(Project::new(tenant_id, id, name))
}

pub async fn persist_tenant(store: &dyn AdminStore, id: &str, name: &str) -> Result<Tenant> {
    let tenant = create_tenant(id, name)?;
    store.insert_tenant(&tenant).await
}

pub async fn list_tenants(store: &dyn AdminStore) -> Result<Vec<Tenant>> {
    store.list_tenants().await
}

pub async fn persist_project(
    store: &dyn AdminStore,
    tenant_id: &str,
    id: &str,
    name: &str,
) -> Result<Project> {
    let project = create_project(tenant_id, id, name)?;
    store.insert_project(&project).await
}

pub async fn list_projects(store: &dyn AdminStore) -> Result<Vec<Project>> {
    store.list_projects().await
}

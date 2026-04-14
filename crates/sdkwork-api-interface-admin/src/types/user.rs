use super::*;

fn default_user_active() -> bool {
    true
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct UpsertOperatorUserRequest {
    #[serde(default)]
    pub(crate) id: Option<String>,
    pub(crate) email: String,
    pub(crate) display_name: String,
    #[serde(default)]
    pub(crate) password: Option<String>,
    #[serde(default)]
    pub(crate) role: Option<AdminUserRole>,
    #[serde(default = "default_user_active")]
    pub(crate) active: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct UpsertPortalUserRequest {
    #[serde(default)]
    pub(crate) id: Option<String>,
    pub(crate) email: String,
    pub(crate) display_name: String,
    #[serde(default)]
    pub(crate) password: Option<String>,
    pub(crate) workspace_tenant_id: String,
    pub(crate) workspace_project_id: String,
    #[serde(default = "default_user_active")]
    pub(crate) active: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct UpdateUserStatusRequest {
    pub(crate) active: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct ResetUserPasswordRequest {
    pub(crate) new_password: String,
}

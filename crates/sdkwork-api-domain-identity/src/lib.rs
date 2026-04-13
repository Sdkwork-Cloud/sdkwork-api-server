use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayApiKey {
    pub tenant_id: String,
    pub project_id: String,
    pub environment: String,
    revoked: bool,
}

impl GatewayApiKey {
    pub fn new(
        tenant_id: impl Into<String>,
        project_id: impl Into<String>,
        environment: impl Into<String>,
    ) -> Self {
        Self {
            tenant_id: tenant_id.into(),
            project_id: project_id.into(),
            environment: environment.into(),
            revoked: false,
        }
    }

    pub fn revoke(&mut self) {
        self.revoked = true;
    }

    pub fn is_active(&self) -> bool {
        !self.revoked
    }
}

pub trait GatewayApiKeyRepository: Send + Sync {
    fn save(&self, key: &GatewayApiKey) -> Result<(), String>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GatewayApiKeyRecord {
    pub tenant_id: String,
    pub project_id: String,
    pub environment: String,
    pub hashed_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key_group_id: Option<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub key_prefix: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    pub created_at_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_used_at_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at_ms: Option<u64>,
    pub active: bool,
}

impl GatewayApiKeyRecord {
    pub fn new(
        tenant_id: impl Into<String>,
        project_id: impl Into<String>,
        environment: impl Into<String>,
        hashed_key: impl Into<String>,
    ) -> Self {
        Self {
            tenant_id: tenant_id.into(),
            project_id: project_id.into(),
            environment: environment.into(),
            hashed_key: hashed_key.into(),
            api_key_group_id: None,
            key_prefix: String::new(),
            label: String::new(),
            notes: None,
            created_at_ms: 0,
            last_used_at_ms: None,
            expires_at_ms: None,
            active: true,
        }
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    pub fn with_api_key_group_id(mut self, api_key_group_id: impl Into<String>) -> Self {
        self.api_key_group_id = Some(api_key_group_id.into());
        self
    }

    pub fn with_api_key_group_id_option(mut self, api_key_group_id: Option<String>) -> Self {
        self.api_key_group_id = api_key_group_id;
        self
    }

    pub fn with_key_prefix(mut self, key_prefix: impl Into<String>) -> Self {
        self.key_prefix = key_prefix.into();
        self
    }

    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }

    pub fn with_notes_option(mut self, notes: Option<String>) -> Self {
        self.notes = notes;
        self
    }

    pub fn with_created_at_ms(mut self, created_at_ms: u64) -> Self {
        self.created_at_ms = created_at_ms;
        self
    }

    pub fn with_last_used_at_ms(mut self, last_used_at_ms: u64) -> Self {
        self.last_used_at_ms = Some(last_used_at_ms);
        self
    }

    pub fn with_last_used_at_ms_option(mut self, last_used_at_ms: Option<u64>) -> Self {
        self.last_used_at_ms = last_used_at_ms;
        self
    }

    pub fn with_expires_at_ms(mut self, expires_at_ms: u64) -> Self {
        self.expires_at_ms = Some(expires_at_ms);
        self
    }

    pub fn with_expires_at_ms_option(mut self, expires_at_ms: Option<u64>) -> Self {
        self.expires_at_ms = expires_at_ms;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiKeyGroupRecord {
    pub group_id: String,
    pub tenant_id: String,
    pub project_id: String,
    pub environment: String,
    pub name: String,
    pub slug: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_capability_scope: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_routing_profile_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_accounting_mode: Option<String>,
    pub active: bool,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl ApiKeyGroupRecord {
    pub fn new(
        group_id: impl Into<String>,
        tenant_id: impl Into<String>,
        project_id: impl Into<String>,
        environment: impl Into<String>,
        name: impl Into<String>,
        slug: impl Into<String>,
    ) -> Self {
        Self {
            group_id: group_id.into(),
            tenant_id: tenant_id.into(),
            project_id: project_id.into(),
            environment: environment.into(),
            name: name.into(),
            slug: slug.into(),
            description: None,
            color: None,
            default_capability_scope: None,
            default_routing_profile_id: None,
            default_accounting_mode: None,
            active: true,
            created_at_ms: 0,
            updated_at_ms: 0,
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_description_option(mut self, description: Option<String>) -> Self {
        self.description = description;
        self
    }

    pub fn with_color(mut self, color: impl Into<String>) -> Self {
        self.color = Some(color.into());
        self
    }

    pub fn with_color_option(mut self, color: Option<String>) -> Self {
        self.color = color;
        self
    }

    pub fn with_default_capability_scope(
        mut self,
        default_capability_scope: impl Into<String>,
    ) -> Self {
        self.default_capability_scope = Some(default_capability_scope.into());
        self
    }

    pub fn with_default_capability_scope_option(
        mut self,
        default_capability_scope: Option<String>,
    ) -> Self {
        self.default_capability_scope = default_capability_scope;
        self
    }

    pub fn with_default_routing_profile_id(
        mut self,
        default_routing_profile_id: impl Into<String>,
    ) -> Self {
        self.default_routing_profile_id = Some(default_routing_profile_id.into());
        self
    }

    pub fn with_default_routing_profile_id_option(
        mut self,
        default_routing_profile_id: Option<String>,
    ) -> Self {
        self.default_routing_profile_id = default_routing_profile_id;
        self
    }

    pub fn with_default_accounting_mode(
        mut self,
        default_accounting_mode: impl Into<String>,
    ) -> Self {
        self.default_accounting_mode = Some(default_accounting_mode.into());
        self
    }

    pub fn with_default_accounting_mode_option(
        mut self,
        default_accounting_mode: Option<String>,
    ) -> Self {
        self.default_accounting_mode = default_accounting_mode;
        self
    }

    pub fn with_active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    pub fn with_created_at_ms(mut self, created_at_ms: u64) -> Self {
        self.created_at_ms = created_at_ms;
        self
    }

    pub fn with_updated_at_ms(mut self, updated_at_ms: u64) -> Self {
        self.updated_at_ms = updated_at_ms;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortalUserRecord {
    pub id: String,
    pub email: String,
    pub display_name: String,
    pub password_salt: String,
    pub password_hash: String,
    pub workspace_tenant_id: String,
    pub workspace_project_id: String,
    pub active: bool,
    pub created_at_ms: u64,
}

impl PortalUserRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        email: impl Into<String>,
        display_name: impl Into<String>,
        password_salt: impl Into<String>,
        password_hash: impl Into<String>,
        workspace_tenant_id: impl Into<String>,
        workspace_project_id: impl Into<String>,
        active: bool,
        created_at_ms: u64,
    ) -> Self {
        Self {
            id: id.into(),
            email: email.into(),
            display_name: display_name.into(),
            password_salt: password_salt.into(),
            password_hash: password_hash.into(),
            workspace_tenant_id: workspace_tenant_id.into(),
            workspace_project_id: workspace_project_id.into(),
            active,
            created_at_ms,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortalWorkspaceMembershipRecord {
    pub membership_id: String,
    pub user_id: String,
    pub tenant_id: String,
    pub project_id: String,
    pub role: String,
    pub created_at_ms: u64,
}

impl PortalWorkspaceMembershipRecord {
    pub fn new(
        membership_id: impl Into<String>,
        user_id: impl Into<String>,
        tenant_id: impl Into<String>,
        project_id: impl Into<String>,
        created_at_ms: u64,
    ) -> Self {
        Self {
            membership_id: membership_id.into(),
            user_id: user_id.into(),
            tenant_id: tenant_id.into(),
            project_id: project_id.into(),
            role: "member".to_owned(),
            created_at_ms,
        }
    }

    pub fn with_role(mut self, role: impl Into<String>) -> Self {
        self.role = role.into();
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdminUserRecord {
    pub id: String,
    pub email: String,
    pub display_name: String,
    pub password_salt: String,
    pub password_hash: String,
    pub role: AdminUserRole,
    pub active: bool,
    pub created_at_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminUserRole {
    SuperAdmin,
    PlatformOperator,
    FinanceOperator,
    ReadOnlyOperator,
}

impl AdminUserRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SuperAdmin => "super_admin",
            Self::PlatformOperator => "platform_operator",
            Self::FinanceOperator => "finance_operator",
            Self::ReadOnlyOperator => "read_only_operator",
        }
    }
}

impl Default for AdminUserRole {
    fn default() -> Self {
        Self::SuperAdmin
    }
}

impl FromStr for AdminUserRole {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim() {
            "super_admin" => Ok(Self::SuperAdmin),
            "platform_operator" => Ok(Self::PlatformOperator),
            "finance_operator" => Ok(Self::FinanceOperator),
            "read_only_operator" => Ok(Self::ReadOnlyOperator),
            _ => Err(
                "admin role must be one of super_admin, platform_operator, finance_operator, read_only_operator"
                    .to_owned(),
            ),
        }
    }
}

impl AdminUserRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        email: impl Into<String>,
        display_name: impl Into<String>,
        password_salt: impl Into<String>,
        password_hash: impl Into<String>,
        role: AdminUserRole,
        active: bool,
        created_at_ms: u64,
    ) -> Self {
        Self {
            id: id.into(),
            email: email.into(),
            display_name: display_name.into(),
            password_salt: password_salt.into(),
            password_hash: password_hash.into(),
            role,
            active,
            created_at_ms,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortalUserProfile {
    pub id: String,
    pub email: String,
    pub display_name: String,
    pub workspace_tenant_id: String,
    pub workspace_project_id: String,
    pub active: bool,
    pub created_at_ms: u64,
}

impl From<&PortalUserRecord> for PortalUserProfile {
    fn from(value: &PortalUserRecord) -> Self {
        Self {
            id: value.id.clone(),
            email: value.email.clone(),
            display_name: value.display_name.clone(),
            workspace_tenant_id: value.workspace_tenant_id.clone(),
            workspace_project_id: value.workspace_project_id.clone(),
            active: value.active,
            created_at_ms: value.created_at_ms,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdminUserProfile {
    pub id: String,
    pub email: String,
    pub display_name: String,
    pub role: AdminUserRole,
    pub active: bool,
    pub created_at_ms: u64,
}

impl From<&AdminUserRecord> for AdminUserProfile {
    fn from(value: &AdminUserRecord) -> Self {
        Self {
            id: value.id.clone(),
            email: value.email.clone(),
            display_name: value.display_name.clone(),
            role: value.role,
            active: value.active,
            created_at_ms: value.created_at_ms,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdminAuditEventRecord {
    pub event_id: String,
    pub actor_user_id: String,
    pub actor_email: String,
    pub actor_role: AdminUserRole,
    pub action: String,
    pub resource_type: String,
    pub resource_id: String,
    pub approval_scope: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_tenant_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_project_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_provider_id: Option<String>,
    pub created_at_ms: u64,
}

impl AdminAuditEventRecord {
    pub fn new(
        event_id: impl Into<String>,
        actor_user_id: impl Into<String>,
        actor_email: impl Into<String>,
        actor_role: AdminUserRole,
        action: impl Into<String>,
        resource_type: impl Into<String>,
        resource_id: impl Into<String>,
        approval_scope: impl Into<String>,
        created_at_ms: u64,
    ) -> Self {
        Self {
            event_id: event_id.into(),
            actor_user_id: actor_user_id.into(),
            actor_email: actor_email.into(),
            actor_role,
            action: action.into(),
            resource_type: resource_type.into(),
            resource_id: resource_id.into(),
            approval_scope: approval_scope.into(),
            target_tenant_id: None,
            target_project_id: None,
            target_provider_id: None,
            created_at_ms,
        }
    }

    pub fn with_target_tenant_id(mut self, target_tenant_id: impl Into<String>) -> Self {
        self.target_tenant_id = Some(target_tenant_id.into());
        self
    }

    pub fn with_target_tenant_id_option(mut self, target_tenant_id: Option<String>) -> Self {
        self.target_tenant_id = target_tenant_id;
        self
    }

    pub fn with_target_project_id(mut self, target_project_id: impl Into<String>) -> Self {
        self.target_project_id = Some(target_project_id.into());
        self
    }

    pub fn with_target_project_id_option(mut self, target_project_id: Option<String>) -> Self {
        self.target_project_id = target_project_id;
        self
    }

    pub fn with_target_provider_id(mut self, target_provider_id: impl Into<String>) -> Self {
        self.target_provider_id = Some(target_provider_id.into());
        self
    }

    pub fn with_target_provider_id_option(mut self, target_provider_id: Option<String>) -> Self {
        self.target_provider_id = target_provider_id;
        self
    }
}

pub type TenantId = u64;
pub type OrganizationId = u64;
pub type UserId = u64;
pub type ApiKeyId = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GatewayAuthType {
    ApiKey,
    Jwt,
}

impl GatewayAuthType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ApiKey => "api_key",
            Self::Jwt => "jwt",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GatewayAuthSubject {
    pub tenant_id: TenantId,
    pub organization_id: OrganizationId,
    pub user_id: UserId,
    pub auth_type: GatewayAuthType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key_id: Option<ApiKeyId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jwt_subject: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    pub request_principal: String,
}

impl GatewayAuthSubject {
    pub fn for_jwt(
        tenant_id: TenantId,
        organization_id: OrganizationId,
        user_id: UserId,
        jwt_subject: impl Into<String>,
    ) -> Self {
        let jwt_subject = jwt_subject.into();
        Self {
            tenant_id,
            organization_id,
            user_id,
            auth_type: GatewayAuthType::Jwt,
            api_key_id: None,
            api_key_hash: None,
            request_principal: format!("jwt:{jwt_subject}"),
            jwt_subject: Some(jwt_subject),
            platform: None,
            owner: None,
        }
    }

    pub fn for_api_key(
        tenant_id: TenantId,
        organization_id: OrganizationId,
        user_id: UserId,
        api_key_id: ApiKeyId,
        api_key_hash: impl Into<String>,
    ) -> Self {
        Self {
            tenant_id,
            organization_id,
            user_id,
            auth_type: GatewayAuthType::ApiKey,
            api_key_id: Some(api_key_id),
            api_key_hash: Some(api_key_hash.into()),
            jwt_subject: None,
            platform: None,
            owner: None,
            request_principal: format!("api_key:{api_key_id}"),
        }
    }

    pub fn with_platform(mut self, platform: impl Into<String>) -> Self {
        self.platform = Some(platform.into());
        self
    }

    pub fn with_owner(mut self, owner: impl Into<String>) -> Self {
        self.owner = Some(owner.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityUserRecord {
    pub user_id: UserId,
    pub tenant_id: TenantId,
    pub organization_id: OrganizationId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_user_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    pub status: String,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl IdentityUserRecord {
    pub fn new(user_id: UserId, tenant_id: TenantId, organization_id: OrganizationId) -> Self {
        Self {
            user_id,
            tenant_id,
            organization_id,
            external_user_ref: None,
            username: None,
            display_name: None,
            email: None,
            status: "active".to_owned(),
            created_at_ms: 0,
            updated_at_ms: 0,
        }
    }

    pub fn with_external_user_ref(mut self, external_user_ref: Option<String>) -> Self {
        self.external_user_ref = external_user_ref;
        self
    }

    pub fn with_username(mut self, username: Option<String>) -> Self {
        self.username = username;
        self
    }

    pub fn with_display_name(mut self, display_name: Option<String>) -> Self {
        self.display_name = display_name;
        self
    }

    pub fn with_email(mut self, email: Option<String>) -> Self {
        self.email = email;
        self
    }

    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.status = status.into();
        self
    }

    pub fn with_created_at_ms(mut self, created_at_ms: u64) -> Self {
        self.created_at_ms = created_at_ms;
        self
    }

    pub fn with_updated_at_ms(mut self, updated_at_ms: u64) -> Self {
        self.updated_at_ms = updated_at_ms;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalApiKeyRecord {
    pub api_key_id: ApiKeyId,
    pub tenant_id: TenantId,
    pub organization_id: OrganizationId,
    pub user_id: UserId,
    pub key_prefix: String,
    pub key_hash: String,
    pub display_name: String,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_used_at_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rotated_from_api_key_id: Option<ApiKeyId>,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl CanonicalApiKeyRecord {
    pub fn new(
        api_key_id: ApiKeyId,
        tenant_id: TenantId,
        organization_id: OrganizationId,
        user_id: UserId,
        key_hash: impl Into<String>,
    ) -> Self {
        Self {
            api_key_id,
            tenant_id,
            organization_id,
            user_id,
            key_prefix: String::new(),
            key_hash: key_hash.into(),
            display_name: String::new(),
            status: "active".to_owned(),
            expires_at_ms: None,
            last_used_at_ms: None,
            rotated_from_api_key_id: None,
            created_at_ms: 0,
            updated_at_ms: 0,
        }
    }

    pub fn with_key_prefix(mut self, key_prefix: impl Into<String>) -> Self {
        self.key_prefix = key_prefix.into();
        self
    }

    pub fn with_display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = display_name.into();
        self
    }

    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.status = status.into();
        self
    }

    pub fn with_expires_at_ms(mut self, expires_at_ms: Option<u64>) -> Self {
        self.expires_at_ms = expires_at_ms;
        self
    }

    pub fn with_last_used_at_ms(mut self, last_used_at_ms: Option<u64>) -> Self {
        self.last_used_at_ms = last_used_at_ms;
        self
    }

    pub fn with_rotated_from_api_key_id(
        mut self,
        rotated_from_api_key_id: Option<ApiKeyId>,
    ) -> Self {
        self.rotated_from_api_key_id = rotated_from_api_key_id;
        self
    }

    pub fn with_created_at_ms(mut self, created_at_ms: u64) -> Self {
        self.created_at_ms = created_at_ms;
        self
    }

    pub fn with_updated_at_ms(mut self, updated_at_ms: u64) -> Self {
        self.updated_at_ms = updated_at_ms;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityBindingRecord {
    pub identity_binding_id: u64,
    pub tenant_id: TenantId,
    pub organization_id: OrganizationId,
    pub user_id: UserId,
    pub binding_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issuer: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_ref: Option<String>,
    pub status: String,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl IdentityBindingRecord {
    pub fn new(
        identity_binding_id: u64,
        tenant_id: TenantId,
        organization_id: OrganizationId,
        user_id: UserId,
        binding_type: impl Into<String>,
    ) -> Self {
        Self {
            identity_binding_id,
            tenant_id,
            organization_id,
            user_id,
            binding_type: binding_type.into(),
            issuer: None,
            subject: None,
            platform: None,
            owner: None,
            external_ref: None,
            status: "active".to_owned(),
            created_at_ms: 0,
            updated_at_ms: 0,
        }
    }

    pub fn with_issuer(mut self, issuer: Option<String>) -> Self {
        self.issuer = issuer;
        self
    }

    pub fn with_subject(mut self, subject: Option<String>) -> Self {
        self.subject = subject;
        self
    }

    pub fn with_platform(mut self, platform: Option<String>) -> Self {
        self.platform = platform;
        self
    }

    pub fn with_owner(mut self, owner: Option<String>) -> Self {
        self.owner = owner;
        self
    }

    pub fn with_external_ref(mut self, external_ref: Option<String>) -> Self {
        self.external_ref = external_ref;
        self
    }

    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.status = status.into();
        self
    }

    pub fn with_created_at_ms(mut self, created_at_ms: u64) -> Self {
        self.created_at_ms = created_at_ms;
        self
    }

    pub fn with_updated_at_ms(mut self, updated_at_ms: u64) -> Self {
        self.updated_at_ms = updated_at_ms;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tenant {
    pub id: String,
    pub name: String,
}

impl Tenant {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    pub tenant_id: String,
    pub id: String,
    pub name: String,
}

impl Project {
    pub fn new(
        tenant_id: impl Into<String>,
        id: impl Into<String>,
        name: impl Into<String>,
    ) -> Self {
        Self {
            tenant_id: tenant_id.into(),
            id: id.into(),
            name: name.into(),
        }
    }
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateContainerRequest {
    pub name: String,
}

impl CreateContainerRequest {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ContainerObject {
    pub id: String,
    pub object: &'static str,
    pub name: String,
    pub status: &'static str,
}

impl ContainerObject {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            object: "container",
            name: name.into(),
            status: "running",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ListContainersResponse {
    pub object: &'static str,
    pub data: Vec<ContainerObject>,
}

impl ListContainersResponse {
    pub fn new(data: Vec<ContainerObject>) -> Self {
        Self {
            object: "list",
            data,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DeleteContainerResponse {
    pub id: String,
    pub object: &'static str,
    pub deleted: bool,
}

impl DeleteContainerResponse {
    pub fn deleted(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            object: "container.deleted",
            deleted: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateContainerFileRequest {
    pub file_id: String,
}

impl CreateContainerFileRequest {
    pub fn new(file_id: impl Into<String>) -> Self {
        Self {
            file_id: file_id.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ContainerFileObject {
    pub id: String,
    pub object: &'static str,
    pub container_id: String,
}

impl ContainerFileObject {
    pub fn new(id: impl Into<String>, container_id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            object: "container.file",
            container_id: container_id.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ListContainerFilesResponse {
    pub object: &'static str,
    pub data: Vec<ContainerFileObject>,
}

impl ListContainerFilesResponse {
    pub fn new(data: Vec<ContainerFileObject>) -> Self {
        Self {
            object: "list",
            data,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DeleteContainerFileResponse {
    pub id: String,
    pub object: &'static str,
    pub deleted: bool,
}

impl DeleteContainerFileResponse {
    pub fn deleted(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            object: "container.file.deleted",
            deleted: true,
        }
    }
}

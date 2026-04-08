use super::*;

#[derive(Debug, Deserialize)]
pub(crate) struct RegisterRequest {
    pub(crate) email: String,
    pub(crate) password: String,
    pub(crate) display_name: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct LoginRequest {
    pub(crate) email: String,
    pub(crate) password: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ChangePasswordRequest {
    pub(crate) current_password: String,
    pub(crate) new_password: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct ErrorResponse {
    pub(crate) error: ErrorBody,
}

#[derive(Debug, Serialize)]
pub(crate) struct ErrorBody {
    pub(crate) message: String,
}

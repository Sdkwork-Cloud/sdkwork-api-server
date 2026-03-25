use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    env, fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ApiKeySetupClientId {
    Codex,
    ClaudeCode,
    Opencode,
    Openclaw,
    Gemini,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ApiKeySetupCompatibility {
    Openai,
    Anthropic,
    Gemini,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ApiKeySetupInstallMode {
    Standard,
    Env,
    Both,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ApiKeySetupEnvScope {
    User,
    System,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeySetupModel {
    pub id: String,
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeySetupProvider {
    pub id: String,
    pub channel_id: String,
    pub name: String,
    pub base_url: String,
    pub api_key: String,
    pub compatibility: ApiKeySetupCompatibility,
    pub models: Vec<ApiKeySetupModel>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeySetupOpenClaw {
    pub instance_ids: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeySetupRequest {
    pub client_id: ApiKeySetupClientId,
    pub install_mode: Option<ApiKeySetupInstallMode>,
    pub env_scope: Option<ApiKeySetupEnvScope>,
    pub provider: ApiKeySetupProvider,
    pub open_claw: Option<ApiKeySetupOpenClaw>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ApiKeySetupFileAction {
    Created,
    Updated,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeySetupWrittenFile {
    pub path: String,
    pub action: ApiKeySetupFileAction,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeySetupEnvironmentTarget {
    pub scope: ApiKeySetupEnvScope,
    pub shell: String,
    pub target: String,
    pub variables: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeySetupResult {
    pub client_id: ApiKeySetupClientId,
    pub written_files: Vec<ApiKeySetupWrittenFile>,
    pub updated_environments: Vec<ApiKeySetupEnvironmentTarget>,
    pub updated_instance_ids: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeyInstance {
    pub id: String,
    pub label: String,
    pub detail: Option<String>,
}

fn resolve_home_dir() -> Result<PathBuf, String> {
    env::var_os("USERPROFILE")
        .or_else(|| env::var_os("HOME"))
        .map(PathBuf::from)
        .ok_or_else(|| "Could not resolve the current user home directory.".to_string())
}

fn install_mode(request: &ApiKeySetupRequest) -> ApiKeySetupInstallMode {
    request
        .install_mode
        .clone()
        .unwrap_or(ApiKeySetupInstallMode::Standard)
}

fn env_scope(request: &ApiKeySetupRequest) -> ApiKeySetupEnvScope {
    request
        .env_scope
        .clone()
        .unwrap_or(ApiKeySetupEnvScope::User)
}

fn primary_model(provider: &ApiKeySetupProvider) -> Result<&ApiKeySetupModel, String> {
    provider
        .models
        .first()
        .ok_or_else(|| "At least one model is required for quick setup.".to_string())
}

fn write_file(path: &Path, content: String) -> Result<ApiKeySetupWrittenFile, String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    let action = if path.exists() {
        ApiKeySetupFileAction::Updated
    } else {
        ApiKeySetupFileAction::Created
    };
    fs::write(path, content).map_err(|error| error.to_string())?;

    Ok(ApiKeySetupWrittenFile {
        path: path.display().to_string(),
        action,
    })
}

fn build_env_assignments(request: &ApiKeySetupRequest) -> Vec<String> {
    match request.client_id {
        ApiKeySetupClientId::Codex | ApiKeySetupClientId::Opencode => vec![
            format!("OPENAI_API_KEY=\"{}\"", request.provider.api_key),
            format!("OPENAI_BASE_URL=\"{}\"", request.provider.base_url),
        ],
        ApiKeySetupClientId::ClaudeCode => vec![
            format!("ANTHROPIC_AUTH_TOKEN=\"{}\"", request.provider.api_key),
            format!("ANTHROPIC_BASE_URL=\"{}\"", request.provider.base_url),
        ],
        ApiKeySetupClientId::Gemini => vec![
            format!("GEMINI_API_KEY=\"{}\"", request.provider.api_key),
            format!(
                "GOOGLE_GEMINI_BASE_URL=\"{}\"",
                request.provider.base_url
            ),
            "GEMINI_API_KEY_AUTH_MECHANISM=\"bearer\"".to_string(),
        ],
        ApiKeySetupClientId::Openclaw => Vec::new(),
    }
}

fn write_env_file(
    home_dir: &Path,
    request: &ApiKeySetupRequest,
) -> Result<Option<ApiKeySetupEnvironmentTarget>, String> {
    let assignments = build_env_assignments(request);
    if assignments.is_empty() {
        return Ok(None);
    }

    let is_windows = env::consts::OS.eq_ignore_ascii_case("windows");
    let scope = env_scope(request);
    let target = home_dir
        .join(".sdkwork-router")
        .join(match (is_windows, scope.clone()) {
            (true, ApiKeySetupEnvScope::System) => "system-env.ps1",
            (true, ApiKeySetupEnvScope::User) => "user-env.ps1",
            (false, ApiKeySetupEnvScope::System) => "system-env.sh",
            (false, ApiKeySetupEnvScope::User) => "user-env.sh",
        });
    let content = if is_windows {
        assignments
            .iter()
            .map(|item| {
                let (key, value) = item.split_once('=').unwrap_or((item.as_str(), "\"\""));
                format!("$env:{key} = {value}")
            })
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        assignments
            .iter()
            .map(|item| format!("export {item}"))
            .collect::<Vec<_>>()
            .join("\n")
    };

    write_file(&target, format!("{content}\n"))?;

    Ok(Some(ApiKeySetupEnvironmentTarget {
        scope,
        shell: if is_windows { "powershell" } else { "sh" }.to_string(),
        target: target.display().to_string(),
        variables: assignments,
    }))
}

#[tauri::command]
pub fn install_api_router_client_setup(request: ApiKeySetupRequest) -> Result<ApiKeySetupResult, String> {
    let home_dir = resolve_home_dir()?;
    let mode = install_mode(&request);
    let model = primary_model(&request.provider)?;
    let mut written_files = Vec::new();
    let mut updated_environments = Vec::new();
    let mut updated_instance_ids = Vec::new();

    if matches!(mode, ApiKeySetupInstallMode::Env | ApiKeySetupInstallMode::Both) {
        if let Some(target) = write_env_file(&home_dir, &request)? {
            updated_environments.push(target);
        }
    }

    if matches!(mode, ApiKeySetupInstallMode::Env) {
        return Ok(ApiKeySetupResult {
            client_id: request.client_id,
            written_files,
            updated_environments,
            updated_instance_ids,
        });
    }

    match request.client_id {
        ApiKeySetupClientId::Codex => {
            written_files.push(write_file(
                &home_dir.join(".codex").join("config.toml"),
                format!(
                    "model = \"{}\"\nmodel_provider = \"api_router\"\n\n[model_providers.api_router]\nname = \"{}\"\nbase_url = \"{}\"\nwire_api = \"responses\"\nrequires_openai_auth = true\n",
                    model.id, request.provider.name, request.provider.base_url
                ),
            )?);
            written_files.push(write_file(
                &home_dir.join(".codex").join("auth.json"),
                serde_json::to_string_pretty(&json!({
                    "auth_mode": "apikey",
                    "OPENAI_API_KEY": request.provider.api_key,
                }))
                .map_err(|error| error.to_string())?,
            )?);
        }
        ApiKeySetupClientId::ClaudeCode => {
            written_files.push(write_file(
                &home_dir.join(".claude").join("settings.json"),
                serde_json::to_string_pretty(&json!({
                    "$schema": "https://json.schemastore.org/claude-code-settings.json",
                    "model": model.id,
                    "env": {
                        "ANTHROPIC_AUTH_TOKEN": request.provider.api_key,
                        "ANTHROPIC_BASE_URL": request.provider.base_url,
                    }
                }))
                .map_err(|error| error.to_string())?,
            )?);
        }
        ApiKeySetupClientId::Opencode => {
            written_files.push(write_file(
                &home_dir.join(".config").join("opencode").join("opencode.json"),
                serde_json::to_string_pretty(&json!({
                    "$schema": "https://opencode.ai/config.json",
                    "provider": {
                        "api-router": {
                            "npm": "@ai-sdk/openai",
                            "name": request.provider.name,
                            "options": { "baseURL": request.provider.base_url },
                            "models": {
                                model.id.clone(): { "name": format!("{} / {}", request.provider.name, model.name) }
                            }
                        }
                    },
                    "model": format!("api-router/{}", model.id),
                }))
                .map_err(|error| error.to_string())?,
            )?);
            written_files.push(write_file(
                &home_dir.join(".local").join("share").join("opencode").join("auth.json"),
                serde_json::to_string_pretty(&json!({
                    "api-router": { "type": "api", "key": request.provider.api_key }
                }))
                .map_err(|error| error.to_string())?,
            )?);
        }
        ApiKeySetupClientId::Gemini => {
            written_files.push(write_file(
                &home_dir.join(".gemini").join("settings.json"),
                serde_json::to_string_pretty(&json!({
                    "model": { "name": model.id },
                    "security": { "auth": { "selectedType": "gemini-api-key" } }
                }))
                .map_err(|error| error.to_string())?,
            )?);
            written_files.push(write_file(
                &home_dir.join(".gemini").join(".env"),
                format!(
                    "GEMINI_API_KEY=\"{}\"\nGOOGLE_GEMINI_BASE_URL=\"{}\"\nGEMINI_API_KEY_AUTH_MECHANISM=\"bearer\"\n",
                    request.provider.api_key, request.provider.base_url
                ),
            )?);
        }
        ApiKeySetupClientId::Openclaw => {
            let open_claw = request
                .open_claw
                .ok_or_else(|| "OpenClaw setup requires at least one selected instance.".to_string())?;
            if open_claw.instance_ids.is_empty() {
                return Err("OpenClaw setup requires at least one selected instance.".to_string());
            }

            for instance_id in open_claw.instance_ids {
                written_files.push(write_file(
                    &home_dir
                        .join(".openclaw")
                        .join("instances")
                        .join(&instance_id)
                        .join("providers")
                        .join("provider-api-router.json"),
                    serde_json::to_string_pretty(&json!({
                        "provider": "api-router",
                        "endpoint": request.provider.base_url,
                        "apiKey": request.provider.api_key,
                        "defaultModelId": model.id,
                        "label": request.provider.name,
                    }))
                    .map_err(|error| error.to_string())?,
                )?);
                updated_instance_ids.push(instance_id);
            }
        }
    }

    Ok(ApiKeySetupResult {
        client_id: request.client_id,
        written_files,
        updated_environments,
        updated_instance_ids,
    })
}

#[tauri::command]
pub fn list_api_key_instances() -> Result<Vec<ApiKeyInstance>, String> {
    let home_dir = resolve_home_dir()?;
    let instances_root = home_dir.join(".openclaw").join("instances");
    if !instances_root.exists() {
        return Ok(Vec::new());
    }

    let mut instances = Vec::new();
    for entry in fs::read_dir(instances_root).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let id = entry.file_name().to_string_lossy().to_string();
        instances.push(ApiKeyInstance {
            label: id.clone(),
            detail: Some("OpenClaw instance".to_string()),
            id,
        });
    }

    instances.sort_by(|left, right| left.label.cmp(&right.label));
    Ok(instances)
}

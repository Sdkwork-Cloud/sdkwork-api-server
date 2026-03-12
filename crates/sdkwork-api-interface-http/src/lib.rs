use axum::{
    extract::Json as ExtractJson,
    extract::State,
    http::header,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use sdkwork_api_app_gateway::create_chat_completion;
use sdkwork_api_app_gateway::list_models;
use sdkwork_api_app_gateway::{create_embedding, create_response, list_models_from_store};
use sdkwork_api_contract_openai::streaming::SseFrame;
use sdkwork_api_storage_sqlite::SqliteAdminStore;
use serde::Deserialize;
use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct GatewayApiState {
    store: SqliteAdminStore,
}

impl GatewayApiState {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            store: SqliteAdminStore::new(pool),
        }
    }
}

pub fn gateway_router() -> Router {
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/v1/models", get(list_models_handler))
        .route("/v1/chat/completions", post(chat_completions_handler))
        .route("/v1/responses", post(responses_handler))
        .route("/v1/embeddings", post(embeddings_handler))
}

pub fn gateway_router_with_pool(pool: SqlitePool) -> Router {
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/v1/models", get(list_models_from_store_handler))
        .route("/v1/chat/completions", post(chat_completions_handler))
        .route("/v1/responses", post(responses_handler))
        .route("/v1/embeddings", post(embeddings_handler))
        .with_state(GatewayApiState::new(pool))
}

async fn list_models_handler() -> Json<sdkwork_api_contract_openai::models::ListModelsResponse> {
    Json(list_models("tenant-1", "project-1").expect("models response"))
}

async fn list_models_from_store_handler(
    State(state): State<GatewayApiState>,
) -> Result<Json<sdkwork_api_contract_openai::models::ListModelsResponse>, Response> {
    list_models_from_store(&state.store, "tenant-1", "project-1")
        .await
        .map(Json)
        .map_err(|_| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "failed to load models",
            )
                .into_response()
        })
}

#[derive(Debug, Deserialize)]
struct ChatCompletionsRequest {
    model: String,
    #[allow(dead_code)]
    stream: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct ResponsesRequest {
    model: String,
}

#[derive(Debug, Deserialize)]
struct EmbeddingsRequest {
    model: String,
}

async fn chat_completions_handler(
    ExtractJson(request): ExtractJson<ChatCompletionsRequest>,
) -> Response {
    if request.stream.unwrap_or(false) {
        let body = format!(
            "{}{}",
            SseFrame::data("{\"id\":\"chatcmpl_1\",\"object\":\"chat.completion.chunk\"}"),
            SseFrame::data("[DONE]")
        );
        ([(header::CONTENT_TYPE, "text/event-stream")], body).into_response()
    } else {
        Json(
            create_chat_completion("tenant-1", "project-1", &request.model)
                .expect("chat completion"),
        )
        .into_response()
    }
}

async fn responses_handler(
    ExtractJson(request): ExtractJson<ResponsesRequest>,
) -> Json<sdkwork_api_contract_openai::responses::ResponseObject> {
    Json(create_response("tenant-1", "project-1", &request.model).expect("response"))
}

async fn embeddings_handler(
    ExtractJson(request): ExtractJson<EmbeddingsRequest>,
) -> Json<sdkwork_api_contract_openai::embeddings::CreateEmbeddingResponse> {
    Json(create_embedding("tenant-1", "project-1", &request.model).expect("embedding"))
}

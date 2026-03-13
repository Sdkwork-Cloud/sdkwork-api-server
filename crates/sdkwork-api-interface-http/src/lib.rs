use axum::{
    extract::Json as ExtractJson,
    extract::State,
    http::header,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use sdkwork_api_app_billing::persist_ledger_entry;
use sdkwork_api_app_gateway::create_chat_completion;
use sdkwork_api_app_gateway::list_models;
use sdkwork_api_app_gateway::{
    create_embedding, create_response, list_models_from_store, relay_chat_completion_from_store,
    relay_embedding_from_store, relay_response_from_store,
};
use sdkwork_api_app_routing::simulate_route_with_store;
use sdkwork_api_app_usage::persist_usage_record;
use sdkwork_api_contract_openai::chat_completions::CreateChatCompletionRequest;
use sdkwork_api_contract_openai::embeddings::CreateEmbeddingRequest;
use sdkwork_api_contract_openai::responses::CreateResponseRequest;
use sdkwork_api_contract_openai::streaming::SseFrame;
use sdkwork_api_storage_sqlite::SqliteAdminStore;
use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct GatewayApiState {
    store: SqliteAdminStore,
    credential_master_key: String,
}

impl GatewayApiState {
    pub fn new(pool: SqlitePool) -> Self {
        Self::with_master_key(pool, "local-dev-master-key")
    }

    pub fn with_master_key(pool: SqlitePool, credential_master_key: impl Into<String>) -> Self {
        Self {
            store: SqliteAdminStore::new(pool),
            credential_master_key: credential_master_key.into(),
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
    gateway_router_with_pool_and_master_key(pool, "local-dev-master-key")
}

pub fn gateway_router_with_pool_and_master_key(
    pool: SqlitePool,
    credential_master_key: impl Into<String>,
) -> Router {
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/v1/models", get(list_models_from_store_handler))
        .route(
            "/v1/chat/completions",
            post(chat_completions_with_state_handler),
        )
        .route("/v1/responses", post(responses_with_state_handler))
        .route("/v1/embeddings", post(embeddings_with_state_handler))
        .with_state(GatewayApiState::with_master_key(
            pool,
            credential_master_key,
        ))
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

async fn chat_completions_handler(
    ExtractJson(request): ExtractJson<CreateChatCompletionRequest>,
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
    ExtractJson(request): ExtractJson<CreateResponseRequest>,
) -> Json<sdkwork_api_contract_openai::responses::ResponseObject> {
    Json(create_response("tenant-1", "project-1", &request.model).expect("response"))
}

async fn embeddings_handler(
    ExtractJson(request): ExtractJson<CreateEmbeddingRequest>,
) -> Json<sdkwork_api_contract_openai::embeddings::CreateEmbeddingResponse> {
    Json(create_embedding("tenant-1", "project-1", &request.model).expect("embedding"))
}

async fn chat_completions_with_state_handler(
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateChatCompletionRequest>,
) -> Response {
    if !request.stream.unwrap_or(false) {
        match relay_chat_completion_from_store(
            &state.store,
            &state.credential_master_key,
            "tenant-1",
            "project-1",
            &request,
        )
        .await
        {
            Ok(Some(response)) => {
                let usage_result = record_gateway_usage(
                    &state.store,
                    "chat_completion",
                    &request.model,
                    100,
                    0.10,
                )
                .await;
                if usage_result.is_err() {
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to record usage",
                    )
                        .into_response();
                }

                return Json(response).into_response();
            }
            Ok(None) => {}
            Err(_) => {
                return (
                    axum::http::StatusCode::BAD_GATEWAY,
                    "failed to relay upstream chat completion",
                )
                    .into_response();
            }
        }
    }

    let usage_result =
        record_gateway_usage(&state.store, "chat_completion", &request.model, 100, 0.10).await;
    if usage_result.is_err() {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

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

async fn responses_with_state_handler(
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateResponseRequest>,
) -> Response {
    match relay_response_from_store(
        &state.store,
        &state.credential_master_key,
        "tenant-1",
        "project-1",
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage(&state.store, "responses", &request.model, 120, 0.12)
                .await
                .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_GATEWAY,
                "failed to relay upstream response",
            )
                .into_response();
        }
    }

    if record_gateway_usage(&state.store, "responses", &request.model, 120, 0.12)
        .await
        .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(create_response("tenant-1", "project-1", &request.model).expect("response"))
        .into_response()
}

async fn embeddings_with_state_handler(
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateEmbeddingRequest>,
) -> Response {
    match relay_embedding_from_store(
        &state.store,
        &state.credential_master_key,
        "tenant-1",
        "project-1",
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage(&state.store, "embeddings", &request.model, 10, 0.01)
                .await
                .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_GATEWAY,
                "failed to relay upstream embedding",
            )
                .into_response();
        }
    }

    if record_gateway_usage(&state.store, "embeddings", &request.model, 10, 0.01)
        .await
        .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(create_embedding("tenant-1", "project-1", &request.model).expect("embedding"))
        .into_response()
}

async fn record_gateway_usage(
    store: &SqliteAdminStore,
    capability: &str,
    model: &str,
    units: u64,
    amount: f64,
) -> anyhow::Result<()> {
    let decision = simulate_route_with_store(store, capability, model).await?;
    persist_usage_record(store, "project-1", model, &decision.selected_provider_id).await?;
    persist_ledger_entry(store, "project-1", units, amount).await?;
    Ok(())
}

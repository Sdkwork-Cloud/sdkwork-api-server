#[path = "gateway_openapi_paths_models_chat.rs"]
mod paths_models_chat;
#[path = "gateway_openapi_paths_media.rs"]
mod paths_media;
#[path = "gateway_openapi_paths_assistants_threads.rs"]
mod paths_assistants_threads;
#[path = "gateway_openapi_paths_files_batches.rs"]
mod paths_files_batches;
#[path = "gateway_openapi_paths_vector_compat.rs"]
mod paths_vector_compat;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "SDKWORK Gateway API",
        version = env!("CARGO_PKG_VERSION"),
        description = "OpenAPI 3.1 schema generated directly from the current gateway router implementation."
    ),
    modifiers(&GatewayApiDocModifier),
    tags(
        (name = "system", description = "Gateway health and system-facing routes."),
        (name = "models", description = "Model listing and model metadata routes."),
        (name = "chat", description = "OpenAI-compatible chat completion routes."),
        (name = "completions", description = "OpenAI-compatible text completion routes."),
        (name = "responses", description = "OpenAI-compatible response generation routes."),
        (name = "conversations", description = "OpenAI-compatible conversation and conversation item routes."),
        (name = "embeddings", description = "Embedding generation routes."),
        (name = "moderations", description = "Moderation and safety evaluation routes."),
        (name = "images", description = "Image generation, edit, and variation routes."),
        (name = "audio", description = "Audio transcription, translation, speech, and voice routes."),
        (name = "files", description = "File upload, listing, and retrieval routes."),
        (name = "uploads", description = "Multi-part upload lifecycle routes."),
        (name = "batches", description = "Batch execution submission and management routes."),
        (name = "vector-stores", description = "Vector store search and file management routes."),
        (name = "assistants", description = "Assistant creation and retrieval routes."),
        (name = "threads", description = "Assistant thread and message management routes."),
        (name = "runs", description = "Assistant run orchestration and run step routes."),
        (name = "realtime", description = "Realtime session bootstrap routes."),
        (name = "compatibility", description = "Anthropic and Gemini compatibility routes.")
    )
)]
struct GatewayApiDoc;

struct GatewayApiDocModifier;

impl Modify for GatewayApiDocModifier {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        openapi.servers = Some(vec![Server::new("/")]);
        openapi
            .components
            .get_or_insert_with(utoipa::openapi::Components::new)
            .add_security_scheme(
                "bearerAuth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("API Key")
                        .build(),
                ),
            );
    }
}


mod openapi_paths {
    pub(super) use super::paths_assistants_threads::*;
    pub(super) use super::paths_files_batches::*;
    pub(super) use super::paths_media::*;
    pub(super) use super::paths_models_chat::*;
    pub(super) use super::paths_vector_compat::*;
}

fn gateway_openapi() -> utoipa::openapi::OpenApi {
    OpenApiRouter::<()>::with_openapi(GatewayApiDoc::openapi())
        .routes(routes!(openapi_paths::health))
        .routes(routes!(openapi_paths::list_models))
        .routes(routes!(openapi_paths::get_model))
        .routes(routes!(openapi_paths::chat_completions))
        .routes(routes!(openapi_paths::completions))
        .routes(routes!(openapi_paths::responses))
        .routes(routes!(openapi_paths::responses_input_tokens))
        .routes(routes!(openapi_paths::responses_compact))
        .routes(routes!(openapi_paths::response_get))
        .routes(routes!(openapi_paths::response_delete))
        .routes(routes!(openapi_paths::response_input_items))
        .routes(routes!(openapi_paths::response_cancel))
        .routes(routes!(openapi_paths::embeddings))
        .routes(routes!(openapi_paths::moderations))
        .routes(routes!(openapi_paths::image_generations))
        .routes(routes!(openapi_paths::image_edits))
        .routes(routes!(openapi_paths::image_variations))
        .routes(routes!(openapi_paths::transcriptions))
        .routes(routes!(openapi_paths::translations))
        .routes(routes!(openapi_paths::audio_speech))
        .routes(routes!(openapi_paths::audio_voices))
        .routes(routes!(openapi_paths::audio_voice_consents))
        .routes(routes!(openapi_paths::assistants_list))
        .routes(routes!(openapi_paths::assistants_create))
        .routes(routes!(openapi_paths::assistants_get))
        .routes(routes!(openapi_paths::conversations_list))
        .routes(routes!(openapi_paths::conversations_create))
        .routes(routes!(openapi_paths::conversation_get))
        .routes(routes!(openapi_paths::conversation_update))
        .routes(routes!(openapi_paths::conversation_delete))
        .routes(routes!(openapi_paths::conversation_items_list))
        .routes(routes!(openapi_paths::conversation_items_create))
        .routes(routes!(openapi_paths::conversation_item_get))
        .routes(routes!(openapi_paths::conversation_item_delete))
        .routes(routes!(openapi_paths::threads_create))
        .routes(routes!(openapi_paths::thread_get))
        .routes(routes!(openapi_paths::thread_update))
        .routes(routes!(openapi_paths::thread_delete))
        .routes(routes!(openapi_paths::thread_messages_list))
        .routes(routes!(openapi_paths::thread_messages_create))
        .routes(routes!(openapi_paths::thread_message_get))
        .routes(routes!(openapi_paths::thread_message_update))
        .routes(routes!(openapi_paths::thread_message_delete))
        .routes(routes!(openapi_paths::thread_and_run_create))
        .routes(routes!(openapi_paths::thread_runs_list))
        .routes(routes!(openapi_paths::thread_runs_create))
        .routes(routes!(openapi_paths::thread_run_get))
        .routes(routes!(openapi_paths::thread_run_update))
        .routes(routes!(openapi_paths::thread_run_cancel))
        .routes(routes!(openapi_paths::thread_run_submit_tool_outputs))
        .routes(routes!(openapi_paths::thread_run_steps_list))
        .routes(routes!(openapi_paths::thread_run_step_get))
        .routes(routes!(openapi_paths::realtime_sessions))
        .routes(routes!(openapi_paths::files_list))
        .routes(routes!(openapi_paths::files_create))
        .routes(routes!(openapi_paths::file_get))
        .routes(routes!(openapi_paths::file_delete))
        .routes(routes!(openapi_paths::uploads_create))
        .routes(routes!(openapi_paths::upload_parts_create))
        .routes(routes!(openapi_paths::upload_complete))
        .routes(routes!(openapi_paths::upload_cancel))
        .routes(routes!(openapi_paths::batches_list))
        .routes(routes!(openapi_paths::batches_create))
        .routes(routes!(openapi_paths::batch_get))
        .routes(routes!(openapi_paths::batch_cancel))
        .routes(routes!(openapi_paths::vector_stores_list))
        .routes(routes!(openapi_paths::vector_stores_create))
        .routes(routes!(openapi_paths::vector_store_get))
        .routes(routes!(openapi_paths::vector_store_update))
        .routes(routes!(openapi_paths::vector_store_delete))
        .routes(routes!(openapi_paths::vector_store_search))
        .routes(routes!(openapi_paths::vector_store_files_list))
        .routes(routes!(openapi_paths::vector_store_files_create))
        .routes(routes!(openapi_paths::vector_store_file_get))
        .routes(routes!(openapi_paths::vector_store_file_delete))
        .routes(routes!(openapi_paths::vector_store_file_batches_create))
        .routes(routes!(openapi_paths::vector_store_file_batch_get))
        .routes(routes!(openapi_paths::vector_store_file_batch_cancel))
        .routes(routes!(openapi_paths::vector_store_file_batch_files_list))
        .routes(routes!(openapi_paths::anthropic_messages))
        .routes(routes!(openapi_paths::anthropic_count_tokens))
        .routes(routes!(openapi_paths::gemini_models_compat))
        .into_openapi()
}

async fn gateway_openapi_handler() -> Json<utoipa::openapi::OpenApi> {
    Json(gateway_openapi())
}

async fn gateway_docs_index_handler() -> Html<String> {
    Html(
        r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>SDKWORK Gateway API</title>
    <style>
      :root {
        color-scheme: light dark;
        font-family: Inter, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
      }

      body {
        margin: 0;
        background: #f5f7fb;
        color: #101828;
      }

      .shell {
        display: grid;
        min-height: 100vh;
        grid-template-rows: auto 1fr;
      }

      .hero {
        padding: 20px 24px 16px;
        border-bottom: 1px solid rgba(15, 23, 42, 0.08);
        background: rgba(255, 255, 255, 0.96);
      }

      .eyebrow {
        margin: 0 0 8px;
        font-size: 12px;
        font-weight: 700;
        letter-spacing: 0.12em;
        text-transform: uppercase;
        color: #475467;
      }

      h1 {
        margin: 0 0 8px;
        font-size: 28px;
        line-height: 1.1;
      }

      p {
        margin: 0;
        font-size: 14px;
        line-height: 1.6;
        color: #475467;
      }

      code {
        padding: 2px 6px;
        border-radius: 999px;
        background: rgba(15, 23, 42, 0.06);
        font-size: 12px;
      }

      iframe {
        width: 100%;
        height: 100%;
        border: 0;
        background: white;
      }

      @media (prefers-color-scheme: dark) {
        body {
          background: #09090b;
          color: #fafafa;
        }

        .hero {
          background: rgba(24, 24, 27, 0.96);
          border-bottom-color: rgba(255, 255, 255, 0.08);
        }

        .eyebrow,
        p {
          color: #a1a1aa;
        }

        code {
          background: rgba(255, 255, 255, 0.08);
        }
      }
    </style>
  </head>
  <body>
    <main class="shell">
      <section class="hero">
        <p class="eyebrow">OpenAPI 3.1</p>
        <h1>SDKWORK Gateway API</h1>
        <p>Interactive documentation is backed by the live schema endpoint <code>/openapi.json</code>.</p>
      </section>
      <iframe src="/docs/ui/" title="SDKWORK Gateway API"></iframe>
    </main>
  </body>
</html>"#
            .to_string(),
    )
}

fn gateway_docs_router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/openapi.json", get(gateway_openapi_handler))
        .route("/docs", get(gateway_docs_index_handler))
        .merge(SwaggerUi::new("/docs/ui/").config(SwaggerUiConfig::new([
            SwaggerUiUrl::with_primary("SDKWORK Gateway API", "/openapi.json", true),
        ])))
}


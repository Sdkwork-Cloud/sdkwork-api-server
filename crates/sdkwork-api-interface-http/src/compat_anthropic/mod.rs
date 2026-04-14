use anyhow::{Context, Result};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use sdkwork_api_contract_openai::chat_completions::{
    ChatMessageInput, CreateChatCompletionRequest,
};
use sdkwork_api_contract_openai::responses::CountResponseInputTokensRequest;
use sdkwork_api_provider_core::ProviderStreamOutput;
use serde_json::{json, Map, Value};

use crate::compat_streaming::{sse_named_event_frame, transform_openai_sse_stream, OpenAiSseEvent};

mod error_responses;
mod request_mapping;
mod response_mapping;
mod stream_mapping;

pub use error_responses::{anthropic_bad_gateway_response, anthropic_invalid_request_response};
pub use request_mapping::{anthropic_count_tokens_request, anthropic_request_to_chat_completion};
pub use response_mapping::{openai_chat_response_to_anthropic, openai_count_tokens_to_anthropic};
pub use stream_mapping::anthropic_stream_from_openai;

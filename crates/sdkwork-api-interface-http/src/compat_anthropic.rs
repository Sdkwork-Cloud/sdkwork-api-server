use std::collections::BTreeMap;

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

pub fn anthropic_invalid_request_response(message: impl Into<String>) -> Response {
    anthropic_error_response(
        StatusCode::BAD_REQUEST,
        "invalid_request_error",
        message.into(),
    )
}

pub fn anthropic_bad_gateway_response(message: impl Into<String>) -> Response {
    anthropic_error_response(StatusCode::BAD_GATEWAY, "api_error", message.into())
}

pub fn anthropic_error_response(status: StatusCode, error_type: &str, message: String) -> Response {
    (
        status,
        Json(json!({
            "type": "error",
            "error": {
                "type": error_type,
                "message": message
            }
        })),
    )
        .into_response()
}

pub fn anthropic_request_to_chat_completion(
    payload: &Value,
) -> Result<CreateChatCompletionRequest> {
    let object = payload
        .as_object()
        .context("anthropic request body must be a JSON object")?;
    let model = required_string(object, "model")?.to_owned();
    let stream = object.get("stream").and_then(Value::as_bool);

    let mut extra = passthrough_fields(
        object,
        &[
            "model",
            "messages",
            "system",
            "stream",
            "tools",
            "tool_choice",
            "max_tokens",
            "stop_sequences",
        ],
    );

    if let Some(max_tokens) = object.get("max_tokens").cloned() {
        extra.insert("max_tokens".to_owned(), max_tokens);
    }
    if let Some(stop_sequences) = object.get("stop_sequences").cloned() {
        extra.insert("stop".to_owned(), stop_sequences);
    }
    if let Some(tool_choice) = object.get("tool_choice") {
        extra.insert(
            "tool_choice".to_owned(),
            anthropic_tool_choice_to_openai(tool_choice),
        );
    }
    if let Some(tools) = object.get("tools") {
        let openai_tools = anthropic_tools_to_openai(tools);
        if let Some(array) = openai_tools.as_array() {
            if !array.is_empty() {
                extra.insert("tools".to_owned(), openai_tools);
            }
        }
    }

    let mut messages = Vec::new();
    if let Some(system) = object.get("system") {
        let system_text = anthropic_system_text(system);
        if !system_text.is_empty() {
            messages.push(ChatMessageInput {
                role: "system".to_owned(),
                content: Value::String(system_text),
                extra: Map::new(),
            });
        }
    }
    if let Some(input_messages) = object.get("messages").and_then(Value::as_array) {
        messages.extend(anthropic_messages_to_openai(input_messages)?);
    }

    Ok(CreateChatCompletionRequest {
        model,
        messages,
        stream,
        extra,
    })
}

pub fn anthropic_count_tokens_request(payload: &Value) -> Result<CountResponseInputTokensRequest> {
    let object = payload
        .as_object()
        .context("anthropic count_tokens body must be a JSON object")?;
    Ok(CountResponseInputTokensRequest::new(
        required_string(object, "model")?,
        payload.clone(),
    ))
}

pub fn openai_chat_response_to_anthropic(response: &Value) -> Value {
    let choice = response
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|choices| choices.first())
        .cloned()
        .unwrap_or_else(|| json!({}));
    let message = choice.get("message").cloned().unwrap_or_else(|| json!({}));
    let mut content = openai_message_to_anthropic_content(&message);
    if content.is_empty() {
        content.push(json!({
            "type": "text",
            "text": ""
        }));
    }

    let usage = response.get("usage").cloned().unwrap_or_else(|| json!({}));
    let input_tokens = usage
        .get("prompt_tokens")
        .and_then(Value::as_u64)
        .or_else(|| usage.get("input_tokens").and_then(Value::as_u64))
        .unwrap_or(0);
    let output_tokens = usage
        .get("completion_tokens")
        .and_then(Value::as_u64)
        .or_else(|| usage.get("output_tokens").and_then(Value::as_u64))
        .unwrap_or(0);

    json!({
        "id": response.get("id").cloned().unwrap_or_else(|| Value::String("msg_1".to_owned())),
        "type": "message",
        "role": "assistant",
        "model": response.get("model").cloned().unwrap_or_else(|| Value::String(String::new())),
        "content": content,
        "stop_reason": choice
            .get("finish_reason")
            .and_then(Value::as_str)
            .map(openai_finish_reason_to_anthropic)
            .unwrap_or(Value::Null),
        "stop_sequence": Value::Null,
        "usage": {
            "input_tokens": input_tokens,
            "output_tokens": output_tokens
        }
    })
}

pub fn openai_count_tokens_to_anthropic(response: &Value) -> Value {
    json!({
        "input_tokens": response
            .get("input_tokens")
            .and_then(Value::as_u64)
            .unwrap_or(0)
    })
}

pub fn anthropic_stream_from_openai(response: ProviderStreamOutput) -> ProviderStreamOutput {
    transform_openai_sse_stream(response, AnthropicStreamState::default(), |state, event| {
        state.map_event(event)
    })
}

#[derive(Default)]
struct AnthropicStreamState {
    started: bool,
    finished: bool,
    text_block_open: bool,
    message_id: String,
    model: String,
    pending_tool_calls: BTreeMap<usize, ToolCallBuffer>,
}

impl AnthropicStreamState {
    fn map_event(&mut self, event: OpenAiSseEvent) -> Vec<String> {
        match event {
            OpenAiSseEvent::Json(value) => self.map_json_event(value),
            OpenAiSseEvent::Done => self.finish(Value::Null),
        }
    }

    fn map_json_event(&mut self, value: Value) -> Vec<String> {
        if self.message_id.is_empty() {
            self.message_id = value
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or("msg_stream")
                .to_owned();
        }
        if self.model.is_empty() {
            self.model = value
                .get("model")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned();
        }

        let mut frames = Vec::new();
        if !self.started {
            self.started = true;
            frames.push(sse_named_event_frame(
                "message_start",
                &json!({
                    "type": "message_start",
                    "message": {
                        "id": self.message_id,
                        "type": "message",
                        "role": "assistant",
                        "model": self.model,
                        "content": [],
                        "stop_reason": Value::Null,
                        "stop_sequence": Value::Null,
                        "usage": {
                            "input_tokens": 0,
                            "output_tokens": 0
                        }
                    }
                }),
            ));
        }

        let Some(choice) = value
            .get("choices")
            .and_then(Value::as_array)
            .and_then(|choices| choices.first())
        else {
            return frames;
        };

        if let Some(delta_text) = choice
            .get("delta")
            .and_then(|delta| delta.get("content"))
            .and_then(Value::as_str)
        {
            if !delta_text.is_empty() {
                if !self.text_block_open {
                    self.text_block_open = true;
                    frames.push(sse_named_event_frame(
                        "content_block_start",
                        &json!({
                            "type": "content_block_start",
                            "index": 0,
                            "content_block": {
                                "type": "text",
                                "text": ""
                            }
                        }),
                    ));
                }
                frames.push(sse_named_event_frame(
                    "content_block_delta",
                    &json!({
                        "type": "content_block_delta",
                        "index": 0,
                        "delta": {
                            "type": "text_delta",
                            "text": delta_text
                        }
                    }),
                ));
            }
        }

        if let Some(tool_calls) = choice
            .get("delta")
            .and_then(|delta| delta.get("tool_calls"))
            .and_then(Value::as_array)
        {
            for tool_call in tool_calls {
                let index = tool_call.get("index").and_then(Value::as_u64).unwrap_or(0) as usize;
                let entry = self.pending_tool_calls.entry(index).or_default();
                if entry.id.is_empty() {
                    entry.id = tool_call
                        .get("id")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_owned();
                }
                if let Some(function) = tool_call.get("function") {
                    if entry.name.is_empty() {
                        entry.name = function
                            .get("name")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_owned();
                    }
                    if let Some(arguments) = function.get("arguments").and_then(Value::as_str) {
                        entry.arguments.push_str(arguments);
                    }
                }
            }
        }

        if let Some(finish_reason) = choice.get("finish_reason").and_then(Value::as_str) {
            frames.extend(self.finish(openai_finish_reason_to_anthropic(finish_reason)));
        }

        frames
    }

    fn finish(&mut self, stop_reason: Value) -> Vec<String> {
        if self.finished || !self.started {
            return Vec::new();
        }
        self.finished = true;

        let mut frames = Vec::new();
        if self.text_block_open {
            frames.push(sse_named_event_frame(
                "content_block_stop",
                &json!({
                    "type": "content_block_stop",
                    "index": 0
                }),
            ));
            self.text_block_open = false;
        }

        if !self.pending_tool_calls.is_empty() {
            let base_index = if frames.is_empty() { 0 } else { 1 };
            for (offset, (_, tool_call)) in self.pending_tool_calls.iter().enumerate() {
                let input = serde_json::from_str::<Value>(&tool_call.arguments)
                    .unwrap_or_else(|_| Value::String(tool_call.arguments.clone()));
                let index = base_index + offset;
                frames.push(sse_named_event_frame(
                    "content_block_start",
                    &json!({
                        "type": "content_block_start",
                        "index": index,
                        "content_block": {
                            "type": "tool_use",
                            "id": tool_call.id,
                            "name": tool_call.name,
                            "input": input
                        }
                    }),
                ));
                frames.push(sse_named_event_frame(
                    "content_block_stop",
                    &json!({
                        "type": "content_block_stop",
                        "index": index
                    }),
                ));
            }
            self.pending_tool_calls.clear();
        }

        frames.push(sse_named_event_frame(
            "message_delta",
            &json!({
                "type": "message_delta",
                "delta": {
                    "stop_reason": stop_reason,
                    "stop_sequence": Value::Null
                },
                "usage": {
                    "output_tokens": 0
                }
            }),
        ));
        frames.push(sse_named_event_frame(
            "message_stop",
            &json!({
                "type": "message_stop"
            }),
        ));

        frames
    }
}

#[derive(Default)]
struct ToolCallBuffer {
    id: String,
    name: String,
    arguments: String,
}

fn anthropic_messages_to_openai(messages: &[Value]) -> Result<Vec<ChatMessageInput>> {
    let mut result = Vec::new();

    for message in messages {
        let object = message
            .as_object()
            .context("anthropic message entries must be JSON objects")?;
        let role = required_string(object, "role")?;
        let content = object.get("content").cloned().unwrap_or(Value::Null);

        match role {
            "user" => translate_anthropic_user_message(&content, &mut result)?,
            "assistant" => translate_anthropic_assistant_message(&content, &mut result)?,
            "system" => result.push(ChatMessageInput {
                role: "system".to_owned(),
                content: Value::String(extract_text_from_value(&content)),
                extra: Map::new(),
            }),
            _ => {
                result.push(ChatMessageInput {
                    role: role.to_owned(),
                    content,
                    extra: Map::new(),
                });
            }
        }
    }

    Ok(result)
}

fn translate_anthropic_user_message(
    content: &Value,
    out: &mut Vec<ChatMessageInput>,
) -> Result<()> {
    match content {
        Value::String(text) => {
            out.push(ChatMessageInput {
                role: "user".to_owned(),
                content: Value::String(text.clone()),
                extra: Map::new(),
            });
        }
        Value::Array(blocks) => {
            let mut content_parts = Vec::new();
            let mut tool_messages = Vec::new();

            for block in blocks {
                let block_type = block
                    .get("type")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                match block_type {
                    "text" => {
                        if let Some(text) = block.get("text").and_then(Value::as_str) {
                            content_parts.push(Value::String(text.to_owned()));
                        }
                    }
                    "image" => {
                        if let Some(image_part) = anthropic_image_block_to_openai(block) {
                            content_parts.push(image_part);
                        }
                    }
                    "tool_result" => {
                        let tool_call_id = block
                            .get("tool_use_id")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_owned();
                        let mut extra = Map::new();
                        extra.insert("tool_call_id".to_owned(), Value::String(tool_call_id));
                        if let Some(is_error) = block.get("is_error").cloned() {
                            extra.insert("anthropic_is_error".to_owned(), is_error);
                        }
                        tool_messages.push(ChatMessageInput {
                            role: "tool".to_owned(),
                            content: Value::String(extract_text_from_value(
                                block.get("content").unwrap_or(&Value::Null),
                            )),
                            extra,
                        });
                    }
                    _ => {}
                }
            }

            if !content_parts.is_empty() {
                out.push(ChatMessageInput {
                    role: "user".to_owned(),
                    content: collapse_message_content(content_parts),
                    extra: Map::new(),
                });
            }
            out.extend(tool_messages);
        }
        _ => {}
    }

    Ok(())
}

fn translate_anthropic_assistant_message(
    content: &Value,
    out: &mut Vec<ChatMessageInput>,
) -> Result<()> {
    match content {
        Value::String(text) => {
            out.push(ChatMessageInput {
                role: "assistant".to_owned(),
                content: Value::String(text.clone()),
                extra: Map::new(),
            });
        }
        Value::Array(blocks) => {
            let mut content_parts = Vec::new();
            let mut tool_calls = Vec::new();

            for block in blocks {
                let block_type = block
                    .get("type")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                match block_type {
                    "text" => {
                        if let Some(text) = block.get("text").and_then(Value::as_str) {
                            content_parts.push(Value::String(text.to_owned()));
                        }
                    }
                    "tool_use" => {
                        let name = block
                            .get("name")
                            .and_then(Value::as_str)
                            .unwrap_or_default();
                        let id = block.get("id").and_then(Value::as_str).unwrap_or_default();
                        let input = block.get("input").cloned().unwrap_or_else(|| json!({}));
                        tool_calls.push(json!({
                            "id": id,
                            "type": "function",
                            "function": {
                                "name": name,
                                "arguments": serde_json::to_string(&input).unwrap_or_else(|_| "{}".to_owned())
                            }
                        }));
                    }
                    _ => {}
                }
            }

            if !content_parts.is_empty() || !tool_calls.is_empty() {
                let mut extra = Map::new();
                if !tool_calls.is_empty() {
                    extra.insert("tool_calls".to_owned(), Value::Array(tool_calls));
                }
                out.push(ChatMessageInput {
                    role: "assistant".to_owned(),
                    content: if content_parts.is_empty() {
                        Value::String(String::new())
                    } else {
                        collapse_message_content(content_parts)
                    },
                    extra,
                });
            }
        }
        _ => {}
    }

    Ok(())
}

fn anthropic_tool_choice_to_openai(tool_choice: &Value) -> Value {
    match tool_choice {
        Value::Object(object) => match object.get("type").and_then(Value::as_str) {
            Some("any") => Value::String("required".to_owned()),
            Some("auto") => Value::String("auto".to_owned()),
            Some("tool") => {
                let name = object
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned();
                json!({
                    "type": "function",
                    "function": {
                        "name": name
                    }
                })
            }
            _ => tool_choice.clone(),
        },
        _ => tool_choice.clone(),
    }
}

fn anthropic_tools_to_openai(tools: &Value) -> Value {
    let Some(tools) = tools.as_array() else {
        return Value::Array(Vec::new());
    };

    Value::Array(
        tools
            .iter()
            .map(|tool| {
                json!({
                    "type": "function",
                    "function": {
                        "name": tool.get("name").cloned().unwrap_or_else(|| Value::String(String::new())),
                        "description": tool.get("description").cloned().unwrap_or(Value::Null),
                        "parameters": tool
                            .get("input_schema")
                            .cloned()
                            .unwrap_or_else(|| json!({"type":"object","properties":{}}))
                    }
                })
            })
            .collect(),
    )
}

fn anthropic_system_text(system: &Value) -> String {
    match system {
        Value::String(text) => text.clone(),
        Value::Array(blocks) => blocks
            .iter()
            .filter_map(|block| block.get("text").and_then(Value::as_str))
            .collect::<Vec<_>>()
            .join("\n"),
        _ => String::new(),
    }
}

fn anthropic_image_block_to_openai(block: &Value) -> Option<Value> {
    let source = block.get("source")?;
    let media_type = source.get("media_type").and_then(Value::as_str)?;
    let data = source.get("data").and_then(Value::as_str)?;
    Some(json!({
        "type": "image_url",
        "image_url": {
            "url": format!("data:{media_type};base64,{data}")
        }
    }))
}

fn openai_message_to_anthropic_content(message: &Value) -> Vec<Value> {
    let mut content = Vec::new();

    append_openai_content_as_anthropic_blocks(message.get("content"), &mut content);

    if let Some(tool_calls) = message.get("tool_calls").and_then(Value::as_array) {
        for tool_call in tool_calls {
            let id = tool_call
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let function = tool_call.get("function").unwrap_or(&Value::Null);
            let name = function
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let arguments = function
                .get("arguments")
                .and_then(Value::as_str)
                .unwrap_or("{}");
            let input = serde_json::from_str::<Value>(arguments)
                .unwrap_or_else(|_| Value::String(arguments.to_owned()));
            content.push(json!({
                "type": "tool_use",
                "id": id,
                "name": name,
                "input": input
            }));
        }
    }

    content
}

fn append_openai_content_as_anthropic_blocks(content: Option<&Value>, out: &mut Vec<Value>) {
    let Some(content) = content else {
        return;
    };
    match content {
        Value::String(text) => {
            if !text.is_empty() {
                out.push(json!({
                    "type": "text",
                    "text": text
                }));
            }
        }
        Value::Array(parts) => {
            for part in parts {
                match part {
                    Value::String(text) => out.push(json!({
                        "type": "text",
                        "text": text
                    })),
                    Value::Object(object) => {
                        let part_type = object
                            .get("type")
                            .and_then(Value::as_str)
                            .unwrap_or_default();
                        if matches!(part_type, "text" | "input_text" | "output_text") {
                            if let Some(text) = object.get("text").and_then(Value::as_str) {
                                out.push(json!({
                                    "type": "text",
                                    "text": text
                                }));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
}

fn openai_finish_reason_to_anthropic(finish_reason: &str) -> Value {
    match finish_reason {
        "stop" => Value::String("end_turn".to_owned()),
        "length" => Value::String("max_tokens".to_owned()),
        "tool_calls" | "function_call" => Value::String("tool_use".to_owned()),
        _ => Value::Null,
    }
}

fn collapse_message_content(parts: Vec<Value>) -> Value {
    if parts.iter().all(Value::is_string) {
        let text = parts
            .iter()
            .filter_map(Value::as_str)
            .collect::<Vec<_>>()
            .join("\n");
        Value::String(text)
    } else {
        Value::Array(
            parts
                .into_iter()
                .map(|part| match part {
                    Value::String(text) => json!({
                        "type": "text",
                        "text": text
                    }),
                    other => other,
                })
                .collect(),
        )
    }
}

fn extract_text_from_value(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Array(items) => items
            .iter()
            .filter_map(|item| {
                item.as_str().map(ToOwned::to_owned).or_else(|| {
                    item.get("text")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned)
                })
            })
            .collect::<Vec<_>>()
            .join("\n"),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

fn passthrough_fields(source: &Map<String, Value>, known_fields: &[&str]) -> Map<String, Value> {
    source
        .iter()
        .filter(|(key, _)| !known_fields.contains(&key.as_str()))
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect()
}

fn required_string<'a>(object: &'a Map<String, Value>, key: &str) -> Result<&'a str> {
    object
        .get(key)
        .and_then(Value::as_str)
        .with_context(|| format!("missing or invalid string field `{key}`"))
}

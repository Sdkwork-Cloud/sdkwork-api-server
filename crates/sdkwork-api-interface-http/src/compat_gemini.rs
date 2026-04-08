use std::collections::{BTreeMap, HashMap, VecDeque};

use anyhow::{Context, Result};
use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use sdkwork_api_contract_openai::chat_completions::{
    ChatMessageInput, CreateChatCompletionRequest,
};
use sdkwork_api_contract_openai::responses::CountResponseInputTokensRequest;
use sdkwork_api_provider_core::ProviderStreamOutput;
use serde_json::{Map, Value, json};

use crate::compat_streaming::{OpenAiSseEvent, sse_data_frame, transform_openai_sse_stream};

pub fn gemini_invalid_request_response(message: impl Into<String>) -> Response {
    gemini_error_response(StatusCode::BAD_REQUEST, "INVALID_ARGUMENT", message.into())
}

pub fn gemini_bad_gateway_response(message: impl Into<String>) -> Response {
    gemini_error_response(StatusCode::BAD_GATEWAY, "BAD_GATEWAY", message.into())
}

pub fn gemini_error_response(status: StatusCode, google_status: &str, message: String) -> Response {
    (
        status,
        Json(json!({
            "error": {
                "code": status.as_u16(),
                "message": message,
                "status": google_status
            }
        })),
    )
        .into_response()
}

pub fn gemini_request_to_chat_completion(
    model: &str,
    payload: &Value,
) -> Result<CreateChatCompletionRequest> {
    let object = payload
        .as_object()
        .context("gemini request body must be a JSON object")?;

    let mut extra = passthrough_fields(
        object,
        &["contents", "tools", "generationConfig", "systemInstruction"],
    );

    if let Some(generation_config) = object.get("generationConfig").and_then(Value::as_object) {
        if let Some(max_output_tokens) = generation_config.get("maxOutputTokens").cloned() {
            extra.insert("max_tokens".to_owned(), max_output_tokens);
        }
        if let Some(candidate_count) = generation_config.get("candidateCount").cloned() {
            extra.insert("n".to_owned(), candidate_count);
        }
        if let Some(stop_sequences) = generation_config.get("stopSequences").cloned() {
            extra.insert("stop".to_owned(), stop_sequences);
        }
        for key in [
            "temperature",
            "topP",
            "topK",
            "presencePenalty",
            "frequencyPenalty",
        ] {
            if let Some(value) = generation_config.get(key).cloned() {
                extra.insert(generation_key_to_openai(key).to_owned(), value);
            }
        }
        for (key, value) in generation_config {
            if !matches!(
                key.as_str(),
                "maxOutputTokens"
                    | "candidateCount"
                    | "stopSequences"
                    | "temperature"
                    | "topP"
                    | "topK"
                    | "presencePenalty"
                    | "frequencyPenalty"
            ) {
                extra.insert(key.clone(), value.clone());
            }
        }
    }

    if let Some(tools) = object.get("tools") {
        let openai_tools = gemini_tools_to_openai(tools);
        if let Some(array) = openai_tools.as_array() {
            if !array.is_empty() {
                extra.insert("tools".to_owned(), openai_tools);
            }
        }
    }

    let mut messages = Vec::new();
    if let Some(system_instruction) = object.get("systemInstruction") {
        let text = gemini_system_instruction_text(system_instruction);
        if !text.is_empty() {
            messages.push(ChatMessageInput {
                role: "system".to_owned(),
                content: Value::String(text),
                extra: Map::new(),
            });
        }
    }

    if let Some(contents) = object.get("contents").and_then(Value::as_array) {
        messages.extend(gemini_contents_to_openai(contents)?);
    }

    Ok(CreateChatCompletionRequest {
        model: model.to_owned(),
        messages,
        stream: None,
        extra,
    })
}

pub fn gemini_count_tokens_request(
    model: &str,
    payload: &Value,
) -> CountResponseInputTokensRequest {
    CountResponseInputTokensRequest::new(model, payload.clone())
}

pub fn openai_chat_response_to_gemini(response: &Value) -> Value {
    let choices = response
        .get("choices")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let candidates = choices
        .iter()
        .enumerate()
        .map(|(index, choice)| {
            let message = choice.get("message").cloned().unwrap_or_else(|| json!({}));
            let parts = openai_message_to_gemini_parts(&message);
            json!({
                "index": index,
                "content": {
                    "role": "model",
                    "parts": if parts.is_empty() {
                        vec![json!({"text": ""})]
                    } else {
                        parts
                    }
                },
                "finishReason": choice
                    .get("finish_reason")
                    .and_then(Value::as_str)
                    .map(openai_finish_reason_to_gemini)
                    .unwrap_or("STOP")
            })
        })
        .collect::<Vec<_>>();

    let usage = response.get("usage").cloned().unwrap_or_else(|| json!({}));
    let prompt_tokens = usage
        .get("prompt_tokens")
        .and_then(Value::as_u64)
        .or_else(|| usage.get("input_tokens").and_then(Value::as_u64))
        .unwrap_or(0);
    let candidate_tokens = usage
        .get("completion_tokens")
        .and_then(Value::as_u64)
        .or_else(|| usage.get("output_tokens").and_then(Value::as_u64))
        .unwrap_or(0);
    let total_tokens = usage
        .get("total_tokens")
        .and_then(Value::as_u64)
        .unwrap_or(prompt_tokens.saturating_add(candidate_tokens));

    json!({
        "candidates": candidates,
        "usageMetadata": {
            "promptTokenCount": prompt_tokens,
            "candidatesTokenCount": candidate_tokens,
            "totalTokenCount": total_tokens
        }
    })
}

pub fn openai_count_tokens_to_gemini(response: &Value) -> Value {
    json!({
        "totalTokens": response
            .get("input_tokens")
            .and_then(Value::as_u64)
            .unwrap_or(0)
    })
}

pub fn gemini_stream_from_openai(response: ProviderStreamOutput) -> ProviderStreamOutput {
    transform_openai_sse_stream(response, GeminiStreamState::default(), |state, event| {
        state.map_event(event)
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GeminiCompatAction {
    GenerateContent,
    StreamGenerateContent,
    CountTokens,
}

pub fn parse_gemini_compat_tail(tail: &str) -> Option<(String, GeminiCompatAction)> {
    let (model, action) = tail.split_once(':')?;
    let action = match action {
        "generateContent" => GeminiCompatAction::GenerateContent,
        "streamGenerateContent" => GeminiCompatAction::StreamGenerateContent,
        "countTokens" => GeminiCompatAction::CountTokens,
        _ => return None,
    };
    Some((model.to_owned(), action))
}

#[derive(Default)]
struct GeminiStreamState {
    model: String,
    pending_tool_calls: BTreeMap<usize, ToolCallBuffer>,
}

impl GeminiStreamState {
    fn map_event(&mut self, event: OpenAiSseEvent) -> Vec<String> {
        match event {
            OpenAiSseEvent::Json(value) => self.map_json_event(value),
            OpenAiSseEvent::Done => Vec::new(),
        }
    }

    fn map_json_event(&mut self, value: Value) -> Vec<String> {
        if self.model.is_empty() {
            self.model = value
                .get("model")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned();
        }

        let Some(choice) = value
            .get("choices")
            .and_then(Value::as_array)
            .and_then(|choices| choices.first())
        else {
            return Vec::new();
        };

        let mut frames = Vec::new();
        let mut finish_reason = choice
            .get("finish_reason")
            .and_then(Value::as_str)
            .map(openai_finish_reason_to_gemini);

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

        if let Some(delta_text) = choice
            .get("delta")
            .and_then(|delta| delta.get("content"))
            .and_then(Value::as_str)
        {
            if !delta_text.is_empty() {
                let mut candidate = json!({
                    "content": {
                        "role": "model",
                        "parts": [
                            { "text": delta_text }
                        ]
                    }
                });
                if let Some(reason) = finish_reason.take() {
                    candidate["finishReason"] = Value::String(reason.to_owned());
                }
                frames.push(sse_data_frame(&json!({
                    "candidates": [candidate]
                })));
            }
        }

        if let Some(reason) = finish_reason {
            if !self.pending_tool_calls.is_empty() {
                let parts = self
                    .pending_tool_calls
                    .values()
                    .map(|tool_call| {
                        let args = serde_json::from_str::<Value>(&tool_call.arguments)
                            .unwrap_or_else(|_| Value::String(tool_call.arguments.clone()));
                        json!({
                            "functionCall": {
                                "name": tool_call.name,
                                "args": args
                            }
                        })
                    })
                    .collect::<Vec<_>>();
                frames.push(sse_data_frame(&json!({
                    "candidates": [{
                        "content": {
                            "role": "model",
                            "parts": parts
                        },
                        "finishReason": reason
                    }]
                })));
                self.pending_tool_calls.clear();
            } else if frames.is_empty() {
                frames.push(sse_data_frame(&json!({
                    "candidates": [{
                        "content": {
                            "role": "model",
                            "parts": [
                                { "text": "" }
                            ]
                        },
                        "finishReason": reason
                    }]
                })));
            }
        }

        frames
    }
}

#[derive(Default)]
struct ToolCallBuffer {
    id: String,
    name: String,
    arguments: String,
}

fn gemini_contents_to_openai(contents: &[Value]) -> Result<Vec<ChatMessageInput>> {
    let mut messages = Vec::new();
    let mut call_counters: HashMap<String, usize> = HashMap::new();
    let mut outstanding_calls: HashMap<String, VecDeque<String>> = HashMap::new();

    for content in contents {
        let object = content
            .as_object()
            .context("gemini content entries must be JSON objects")?;
        let role = object.get("role").and_then(Value::as_str).unwrap_or("user");
        let parts = object
            .get("parts")
            .and_then(Value::as_array)
            .context("gemini content parts must be an array")?;

        match role {
            "model" => {
                let mut text_segments = Vec::new();
                let mut tool_calls = Vec::new();

                for part in parts {
                    if let Some(text) = part.get("text").and_then(Value::as_str) {
                        text_segments.push(text.to_owned());
                        continue;
                    }

                    let Some(function_call) = part.get("functionCall") else {
                        continue;
                    };
                    let name = function_call
                        .get("name")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_owned();
                    let call_id = next_tool_call_id(&name, &mut call_counters);
                    outstanding_calls
                        .entry(name.clone())
                        .or_default()
                        .push_back(call_id.clone());
                    let args = function_call
                        .get("args")
                        .cloned()
                        .unwrap_or_else(|| json!({}));
                    tool_calls.push(json!({
                        "id": call_id,
                        "type": "function",
                        "function": {
                            "name": name,
                            "arguments": serde_json::to_string(&args).unwrap_or_else(|_| "{}".to_owned())
                        }
                    }));
                }

                if !text_segments.is_empty() || !tool_calls.is_empty() {
                    let mut extra = Map::new();
                    if !tool_calls.is_empty() {
                        extra.insert("tool_calls".to_owned(), Value::Array(tool_calls));
                    }
                    messages.push(ChatMessageInput {
                        role: "assistant".to_owned(),
                        content: Value::String(text_segments.join("\n")),
                        extra,
                    });
                }
            }
            _ => {
                let mut text_segments = Vec::new();
                let mut tool_messages = Vec::new();

                for part in parts {
                    if let Some(text) = part.get("text").and_then(Value::as_str) {
                        text_segments.push(text.to_owned());
                        continue;
                    }

                    let Some(function_response) = part.get("functionResponse") else {
                        continue;
                    };
                    let name = function_response
                        .get("name")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_owned();
                    let tool_call_id = outstanding_calls
                        .get_mut(&name)
                        .and_then(|calls| calls.pop_front())
                        .unwrap_or_else(|| next_tool_call_id(&name, &mut call_counters));
                    let mut extra = Map::new();
                    extra.insert("tool_call_id".to_owned(), Value::String(tool_call_id));
                    extra.insert("name".to_owned(), Value::String(name));
                    tool_messages.push(ChatMessageInput {
                        role: "tool".to_owned(),
                        content: Value::String(
                            function_response
                                .get("response")
                                .map(extract_text_from_value)
                                .unwrap_or_default(),
                        ),
                        extra,
                    });
                }

                if !text_segments.is_empty() {
                    messages.push(ChatMessageInput {
                        role: "user".to_owned(),
                        content: Value::String(text_segments.join("\n")),
                        extra: Map::new(),
                    });
                }
                messages.extend(tool_messages);
            }
        }
    }

    Ok(messages)
}

fn gemini_system_instruction_text(system_instruction: &Value) -> String {
    if let Some(parts) = system_instruction.get("parts").and_then(Value::as_array) {
        parts
            .iter()
            .filter_map(|part| part.get("text").and_then(Value::as_str))
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        extract_text_from_value(system_instruction)
    }
}

fn gemini_tools_to_openai(tools: &Value) -> Value {
    let Some(tools) = tools.as_array() else {
        return Value::Array(Vec::new());
    };

    let mut openai_tools = Vec::new();
    for tool in tools {
        let Some(declarations) = tool.get("functionDeclarations").and_then(Value::as_array) else {
            continue;
        };
        for declaration in declarations {
            openai_tools.push(json!({
                "type": "function",
                "function": {
                    "name": declaration.get("name").cloned().unwrap_or_else(|| Value::String(String::new())),
                    "description": declaration.get("description").cloned().unwrap_or(Value::Null),
                    "parameters": declaration
                        .get("parameters")
                        .cloned()
                        .unwrap_or_else(|| json!({"type":"object","properties":{}}))
                }
            }));
        }
    }

    Value::Array(openai_tools)
}

fn openai_message_to_gemini_parts(message: &Value) -> Vec<Value> {
    let mut parts = Vec::new();

    append_openai_content_as_gemini_parts(message.get("content"), &mut parts);

    if let Some(tool_calls) = message.get("tool_calls").and_then(Value::as_array) {
        for tool_call in tool_calls {
            let function = tool_call.get("function").unwrap_or(&Value::Null);
            let name = function
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let args = function
                .get("arguments")
                .and_then(Value::as_str)
                .unwrap_or("{}");
            let parsed_args = serde_json::from_str::<Value>(args)
                .unwrap_or_else(|_| Value::String(args.to_owned()));
            parts.push(json!({
                "functionCall": {
                    "name": name,
                    "args": parsed_args
                }
            }));
        }
    }

    parts
}

fn append_openai_content_as_gemini_parts(content: Option<&Value>, out: &mut Vec<Value>) {
    let Some(content) = content else {
        return;
    };

    match content {
        Value::String(text) => {
            if !text.is_empty() {
                out.push(json!({ "text": text }));
            }
        }
        Value::Array(parts) => {
            for part in parts {
                match part {
                    Value::String(text) => out.push(json!({ "text": text })),
                    Value::Object(object) => {
                        let part_type = object
                            .get("type")
                            .and_then(Value::as_str)
                            .unwrap_or_default();
                        if matches!(part_type, "text" | "input_text" | "output_text") {
                            if let Some(text) = object.get("text").and_then(Value::as_str) {
                                out.push(json!({ "text": text }));
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

fn openai_finish_reason_to_gemini(finish_reason: &str) -> &'static str {
    match finish_reason {
        "length" => "MAX_TOKENS",
        "content_filter" => "SAFETY",
        _ => "STOP",
    }
}

fn generation_key_to_openai(key: &str) -> &str {
    match key {
        "topP" => "top_p",
        "presencePenalty" => "presence_penalty",
        "frequencyPenalty" => "frequency_penalty",
        _ => key,
    }
}

fn next_tool_call_id(name: &str, counters: &mut HashMap<String, usize>) -> String {
    let counter = counters.entry(name.to_owned()).or_insert(0);
    *counter += 1;
    format!("gemini_tool_call_{}_{}", sanitize_identifier(name), counter)
}

fn sanitize_identifier(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect::<String>();
    if sanitized.is_empty() {
        "tool".to_owned()
    } else {
        sanitized
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

#[cfg(test)]
mod tests {
    use super::{GeminiCompatAction, parse_gemini_compat_tail};

    #[test]
    fn parses_generate_content_tail() {
        assert_eq!(
            parse_gemini_compat_tail("gemini-2.5-pro:generateContent"),
            Some((
                "gemini-2.5-pro".to_owned(),
                GeminiCompatAction::GenerateContent,
            ))
        );
    }

    #[test]
    fn parses_stream_generate_content_tail() {
        assert_eq!(
            parse_gemini_compat_tail("gemini-2.5-pro:streamGenerateContent"),
            Some((
                "gemini-2.5-pro".to_owned(),
                GeminiCompatAction::StreamGenerateContent,
            ))
        );
    }

    #[test]
    fn parses_count_tokens_tail_with_empty_model() {
        assert_eq!(
            parse_gemini_compat_tail(":countTokens"),
            Some((String::new(), GeminiCompatAction::CountTokens))
        );
    }

    #[test]
    fn rejects_unknown_action() {
        assert_eq!(parse_gemini_compat_tail("gemini-2.5-pro:unsupported"), None);
    }
}

use super::*;

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

pub(crate) fn openai_finish_reason_to_gemini(finish_reason: &str) -> &'static str {
    match finish_reason {
        "length" => "MAX_TOKENS",
        "content_filter" => "SAFETY",
        _ => "STOP",
    }
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

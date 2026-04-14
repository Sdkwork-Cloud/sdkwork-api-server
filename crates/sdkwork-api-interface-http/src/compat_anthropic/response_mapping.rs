use super::*;

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
        "id": response
            .get("id")
            .cloned()
            .unwrap_or_else(|| Value::String(String::new())),
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

pub(crate) fn openai_finish_reason_to_anthropic(finish_reason: &str) -> Value {
    match finish_reason {
        "stop" => Value::String("end_turn".to_owned()),
        "length" => Value::String("max_tokens".to_owned()),
        "tool_calls" | "function_call" => Value::String("tool_use".to_owned()),
        _ => Value::Null,
    }
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

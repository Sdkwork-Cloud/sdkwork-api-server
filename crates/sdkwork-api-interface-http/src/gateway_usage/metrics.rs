use super::*;

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct TokenUsageMetrics {
    pub(crate) input_tokens: u64,
    pub(crate) output_tokens: u64,
    pub(crate) total_tokens: u64,
}

fn json_u64(value: Option<&Value>) -> Option<u64> {
    value.and_then(|value| value.as_u64())
}

pub(crate) fn extract_token_usage_metrics(response: &Value) -> Option<TokenUsageMetrics> {
    if let Some(usage) = response.get("usage") {
        let input_tokens = json_u64(usage.get("prompt_tokens"))
            .or_else(|| json_u64(usage.get("input_tokens")))
            .unwrap_or(0);
        let output_tokens = json_u64(usage.get("completion_tokens"))
            .or_else(|| json_u64(usage.get("output_tokens")))
            .unwrap_or(0);
        let total_tokens = json_u64(usage.get("total_tokens"))
            .unwrap_or_else(|| input_tokens.saturating_add(output_tokens));

        if input_tokens > 0 || output_tokens > 0 || total_tokens > 0 {
            return Some(TokenUsageMetrics {
                input_tokens,
                output_tokens,
                total_tokens,
            });
        }
    }

    let input_tokens = json_u64(response.get("input_tokens")).unwrap_or(0);
    let output_tokens = json_u64(response.get("output_tokens")).unwrap_or(0);
    let total_tokens = json_u64(response.get("total_tokens"))
        .unwrap_or_else(|| input_tokens.saturating_add(output_tokens));

    if input_tokens > 0 || output_tokens > 0 || total_tokens > 0 {
        return Some(TokenUsageMetrics {
            input_tokens,
            output_tokens,
            total_tokens,
        });
    }

    None
}

pub(crate) fn response_usage_id_or_single_data_item_id(response: &Value) -> Option<&str> {
    response.get("id").and_then(Value::as_str).or_else(|| {
        match response
            .get("data")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
        {
            Some([item]) => item.get("id").and_then(Value::as_str),
            _ => None,
        }
    })
}

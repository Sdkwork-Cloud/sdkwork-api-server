#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct TokenUsageMetrics {
    pub(crate) input_tokens: u64,
    pub(crate) output_tokens: u64,
    pub(crate) total_tokens: u64,
}

pub(crate) fn response_usage_id_or_single_data_item_id(
    response: &serde_json::Value,
) -> Option<&str> {
    crate::gateway_commercial::response_usage_id_or_single_data_item_id(response)
}

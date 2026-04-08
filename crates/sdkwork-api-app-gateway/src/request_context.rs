use super::*;

pub async fn with_request_routing_region<T, F>(requested_region: Option<String>, future: F) -> T
where
    F: Future<Output = T>,
{
    REQUEST_ROUTING_REGION
        .scope(
            requested_region.and_then(|region| normalize_routing_region(&region)),
            future,
        )
        .await
}

pub async fn with_request_api_key_group_id<T, F>(api_key_group_id: Option<String>, future: F) -> T
where
    F: Future<Output = T>,
{
    REQUEST_API_KEY_GROUP_ID
        .scope(
            api_key_group_id.and_then(|group_id| normalize_api_key_group_id(&group_id)),
            future,
        )
        .await
}

pub fn current_request_routing_region() -> Option<String> {
    REQUEST_ROUTING_REGION.try_with(Clone::clone).ok().flatten()
}

pub fn current_request_api_key_group_id() -> Option<String> {
    REQUEST_API_KEY_GROUP_ID
        .try_with(Clone::clone)
        .ok()
        .flatten()
}

fn normalize_routing_region(region: &str) -> Option<String> {
    let normalized = region.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

fn normalize_api_key_group_id(api_key_group_id: &str) -> Option<String> {
    let normalized = api_key_group_id.trim();
    if normalized.is_empty() {
        None
    } else {
        Some(normalized.to_owned())
    }
}

use super::*;

pub(crate) fn current_time_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| u64::try_from(duration.as_millis()).unwrap_or(u64::MAX))
        .unwrap_or(0)
}

pub(crate) fn generate_selection_seed() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| {
            let nanos = duration.as_nanos();
            (nanos ^ u128::from(std::process::id())) as u64
        })
        .unwrap_or_else(|_| u64::from(std::process::id()))
}

pub(crate) fn normalize_optional_text(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

pub(crate) fn normalize_region_option(region: Option<&str>) -> Option<String> {
    region.and_then(normalize_region)
}

pub(crate) fn normalize_region(region: &str) -> Option<String> {
    let normalized = region.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

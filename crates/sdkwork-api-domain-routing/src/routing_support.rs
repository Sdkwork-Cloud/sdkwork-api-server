use super::*;

pub(crate) fn default_enabled() -> bool {
    true
}

pub(crate) fn is_false(value: &bool) -> bool {
    !*value
}

pub(crate) fn dedup_preserving_order(values: Vec<String>) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let mut deduped = Vec::with_capacity(values.len());
    for value in values {
        if seen.insert(value.clone()) {
            deduped.push(value);
        }
    }
    deduped
}

pub(crate) fn glob_matches(pattern: &str, input: &str) -> bool {
    glob_matches_bytes(pattern.as_bytes(), input.as_bytes())
}

fn glob_matches_bytes(pattern: &[u8], input: &[u8]) -> bool {
    if pattern.is_empty() {
        return input.is_empty();
    }

    match pattern[0] {
        b'*' => {
            glob_matches_bytes(&pattern[1..], input)
                || (!input.is_empty() && glob_matches_bytes(pattern, &input[1..]))
        }
        byte => {
            !input.is_empty() && byte == input[0] && glob_matches_bytes(&pattern[1..], &input[1..])
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum GeminiCompatAction {
    GenerateContent,
    StreamGenerateContent,
    CountTokens,
}

pub(crate) fn parse_gemini_compat_tail(tail: &str) -> Option<(String, GeminiCompatAction)> {
    let tail = tail.trim_start_matches('/');
    let (model, action) = tail.split_once(':')?;
    let action = match action {
        "generateContent" => GeminiCompatAction::GenerateContent,
        "streamGenerateContent" => GeminiCompatAction::StreamGenerateContent,
        "countTokens" => GeminiCompatAction::CountTokens,
        _ => return None,
    };
    Some((model.to_owned(), action))
}

use super::*;

#[test]
fn builtin_host_registers_current_provider_extensions() {
    let host = builtin_extension_host();

    assert!(host.manifest("sdkwork.provider.openai.official").is_some());
    assert!(host.manifest("sdkwork.provider.openrouter").is_some());
    assert!(host.manifest("sdkwork.provider.ollama").is_some());

    assert!(host
        .resolve_provider("openai", "http://localhost")
        .is_some());
    assert!(host
        .resolve_provider("openrouter", "http://localhost")
        .is_some());
    assert!(host
        .resolve_provider("ollama", "http://localhost")
        .is_some());
}

#[serial(extension_env)]
#[test]
fn builtin_host_resolves_provider_by_extension_id() {
    let host = builtin_extension_host();

    assert!(host
        .resolve_provider("sdkwork.provider.openai.official", "http://localhost")
        .is_some());
    assert!(host
        .resolve_provider("sdkwork.provider.openrouter", "http://localhost")
        .is_some());
    assert!(host
        .resolve_provider("sdkwork.provider.ollama", "http://localhost")
        .is_some());
}

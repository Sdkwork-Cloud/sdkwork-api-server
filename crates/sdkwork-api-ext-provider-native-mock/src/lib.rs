use std::ffi::CString;
use std::os::raw::c_char;
use std::sync::OnceLock;

use sdkwork_api_extension_abi::{
    free_raw_c_string, from_raw_c_str, into_raw_c_string, ProviderInvocation,
    ProviderInvocationResult, ProviderStreamInvocationResult, ProviderStreamWriter,
    SDKWORK_EXTENSION_ABI_VERSION,
};
use sdkwork_api_extension_core::{
    CapabilityDescriptor, CompatibilityLevel, ExtensionKind, ExtensionManifest,
    ExtensionPermission, ExtensionProtocol, ExtensionRuntime,
};

pub const FIXTURE_EXTENSION_ID: &str = "sdkwork.provider.native.mock";

fn manifest_json() -> &'static CString {
    static MANIFEST_JSON: OnceLock<CString> = OnceLock::new();
    MANIFEST_JSON.get_or_init(|| {
        CString::new(
            serde_json::to_string(
                &ExtensionManifest::new(
                    FIXTURE_EXTENSION_ID,
                    ExtensionKind::Provider,
                    "0.1.0",
                    ExtensionRuntime::NativeDynamic,
                )
                .with_display_name("Native Mock")
                .with_protocol(ExtensionProtocol::OpenAi)
                .with_channel_binding("sdkwork.channel.openai")
                .with_permission(ExtensionPermission::NetworkOutbound)
                .with_capability(CapabilityDescriptor::new(
                    "chat.completions.create",
                    CompatibilityLevel::Native,
                ))
                .with_capability(CapabilityDescriptor::new(
                    "chat.completions.stream",
                    CompatibilityLevel::Native,
                )),
            )
            .expect("manifest json"),
        )
        .expect("manifest c string")
    })
}

#[no_mangle]
pub extern "C" fn sdkwork_extension_abi_version() -> u32 {
    SDKWORK_EXTENSION_ABI_VERSION
}

#[no_mangle]
pub extern "C" fn sdkwork_extension_manifest_json() -> *const c_char {
    manifest_json().as_ptr()
}

#[no_mangle]
pub extern "C" fn sdkwork_extension_provider_execute_json(payload: *const c_char) -> *mut c_char {
    let invocation = unsafe { from_raw_c_str(payload) }
        .and_then(|payload| serde_json::from_str::<ProviderInvocation>(&payload).ok());

    let result = match invocation {
        Some(invocation)
            if invocation.operation == "chat.completions.create" && !invocation.expects_stream =>
        {
            ProviderInvocationResult::json(serde_json::json!({
                "id": "chatcmpl_native_dynamic",
                "object": "chat.completion",
                "model": invocation.body["model"],
                "choices": [],
                "provider": "native_dynamic"
            }))
        }
        Some(invocation) if invocation.expects_stream => {
            ProviderInvocationResult::unsupported("stream output is not implemented in the fixture")
        }
        Some(invocation) => ProviderInvocationResult::unsupported(format!(
            "operation {} is not implemented in the fixture",
            invocation.operation
        )),
        None => ProviderInvocationResult::error("invalid invocation payload"),
    };

    into_raw_c_string(serde_json::to_string(&result).expect("result json"))
}

#[no_mangle]
pub extern "C" fn sdkwork_extension_provider_execute_stream_json(
    payload: *const c_char,
    writer: *const ProviderStreamWriter,
) -> *mut c_char {
    let invocation = unsafe { from_raw_c_str(payload) }
        .and_then(|payload| serde_json::from_str::<ProviderInvocation>(&payload).ok());
    let writer = unsafe { writer.as_ref() };

    let result = match (invocation, writer) {
        (Some(invocation), Some(writer))
            if invocation.operation == "chat.completions.create" && invocation.expects_stream =>
        {
            let content_type = "text/event-stream";
            let chunk = serde_json::json!({
                "id": "chatcmpl_native_dynamic_stream",
                "object": "chat.completion.chunk",
                "model": invocation.body["model"],
                "choices": [{
                    "index": 0,
                    "delta": {
                        "role": "assistant",
                        "content": "hello from native dynamic"
                    },
                    "finish_reason": serde_json::Value::Null
                }]
            })
            .to_string();
            let first_frame = format!("data: {chunk}\n\n");
            let done_frame = "data: [DONE]\n\n";

            if !writer.set_content_type(content_type) {
                ProviderStreamInvocationResult::error(
                    "host stream receiver closed before content type was set",
                )
            } else if !writer.write_chunk(first_frame.as_bytes())
                || !writer.write_chunk(done_frame.as_bytes())
            {
                ProviderStreamInvocationResult::error(
                    "host stream receiver closed before all chunks were written",
                )
            } else {
                ProviderStreamInvocationResult::streamed(content_type)
            }
        }
        (Some(invocation), Some(_)) => ProviderStreamInvocationResult::unsupported(format!(
            "stream operation {} is not implemented in the fixture",
            invocation.operation
        )),
        (_, None) => ProviderStreamInvocationResult::error("stream writer is missing"),
        (None, Some(_)) => ProviderStreamInvocationResult::error("invalid invocation payload"),
    };

    into_raw_c_string(serde_json::to_string(&result).expect("result json"))
}

#[no_mangle]
pub extern "C" fn sdkwork_extension_free_string(ptr: *mut c_char) {
    unsafe { free_raw_c_string(ptr) }
}

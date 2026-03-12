use sdkwork_api_contract_openai::streaming::SseFrame;

#[test]
fn formats_sse_frame() {
    let frame = SseFrame::data("{\"ok\":true}");
    assert_eq!(frame.to_string(), "data: {\"ok\":true}\n\n");
}

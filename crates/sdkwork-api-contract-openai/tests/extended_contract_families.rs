use sdkwork_api_contract_openai::assistants::{AssistantObject, CreateAssistantRequest};
use sdkwork_api_contract_openai::audio::{CreateTranscriptionRequest, TranscriptionObject};
use sdkwork_api_contract_openai::batches::{BatchObject, CreateBatchRequest};
use sdkwork_api_contract_openai::evals::{CreateEvalRequest, EvalObject};
use sdkwork_api_contract_openai::files::{CreateFileRequest, FileObject};
use sdkwork_api_contract_openai::images::{CreateImageRequest, ImageObject, ImagesResponse};
use sdkwork_api_contract_openai::moderations::{CreateModerationRequest, ModerationResponse};
use sdkwork_api_contract_openai::realtime::{CreateRealtimeSessionRequest, RealtimeSessionObject};
use sdkwork_api_contract_openai::uploads::{CreateUploadRequest, UploadObject};
use sdkwork_api_contract_openai::vector_stores::{
    CreateVectorStoreRequest, VectorStoreFileObject, VectorStoreObject,
};
use sdkwork_api_contract_openai::webhooks::{CreateWebhookRequest, WebhookObject};

#[test]
fn serializes_file_contracts() {
    let request = CreateFileRequest::new("fine-tune", "train.jsonl");
    let request_json = serde_json::to_value(request).unwrap();
    assert_eq!(request_json["purpose"], "fine-tune");
    assert_eq!(request_json["filename"], "train.jsonl");

    let file = FileObject::new("file_1", "train.jsonl", "fine-tune");
    let json = serde_json::to_value(file).unwrap();
    assert_eq!(json["object"], "file");
    assert_eq!(json["purpose"], "fine-tune");
}

#[test]
fn serializes_upload_contracts() {
    let request = CreateUploadRequest::new("batch", "input.jsonl", 1024);
    let request_json = serde_json::to_value(request).unwrap();
    assert_eq!(request_json["purpose"], "batch");
    assert_eq!(request_json["bytes"], 1024);

    let upload = UploadObject::new("upload_1", "input.jsonl", "batch");
    let json = serde_json::to_value(upload).unwrap();
    assert_eq!(json["object"], "upload");
    assert_eq!(json["status"], "pending");
}

#[test]
fn serializes_audio_contracts() {
    let request = CreateTranscriptionRequest::new("gpt-4o-mini-transcribe", "file_1");
    let request_json = serde_json::to_value(request).unwrap();
    assert_eq!(request_json["model"], "gpt-4o-mini-transcribe");
    assert_eq!(request_json["file_id"], "file_1");

    let transcription = TranscriptionObject::new("hello world");
    let json = serde_json::to_value(transcription).unwrap();
    assert_eq!(json["text"], "hello world");
}

#[test]
fn serializes_image_contracts() {
    let request = CreateImageRequest::new("gpt-image-1", "draw a lighthouse");
    let request_json = serde_json::to_value(request).unwrap();
    assert_eq!(request_json["model"], "gpt-image-1");
    assert_eq!(request_json["prompt"], "draw a lighthouse");

    let response = ImagesResponse::new(vec![ImageObject::base64("encoded-image")]);
    let json = serde_json::to_value(response).unwrap();
    assert_eq!(json["data"][0]["b64_json"], "encoded-image");
}

#[test]
fn serializes_moderation_contracts() {
    let request = CreateModerationRequest::new("omni-moderation-latest", "hi");
    let request_json = serde_json::to_value(request).unwrap();
    assert_eq!(request_json["model"], "omni-moderation-latest");
    assert_eq!(request_json["input"], "hi");

    let response = ModerationResponse::flagged("omni-moderation-latest");
    let json = serde_json::to_value(response).unwrap();
    assert_eq!(json["results"][0]["flagged"], true);
}

#[test]
fn serializes_realtime_contracts() {
    let request = CreateRealtimeSessionRequest::new("gpt-4o-realtime-preview");
    let request_json = serde_json::to_value(request).unwrap();
    assert_eq!(request_json["model"], "gpt-4o-realtime-preview");

    let session = RealtimeSessionObject::new("sess_1", "gpt-4o-realtime-preview");
    let json = serde_json::to_value(session).unwrap();
    assert_eq!(json["object"], "realtime.session");
}

#[test]
fn serializes_assistant_contracts() {
    let request = CreateAssistantRequest::new("Support", "gpt-4.1");
    let request_json = serde_json::to_value(request).unwrap();
    assert_eq!(request_json["name"], "Support");
    assert_eq!(request_json["model"], "gpt-4.1");

    let assistant = AssistantObject::new("asst_1", "Support", "gpt-4.1");
    let json = serde_json::to_value(assistant).unwrap();
    assert_eq!(json["object"], "assistant");
}

#[test]
fn serializes_vector_store_contracts() {
    let request = CreateVectorStoreRequest::new("Knowledge Base");
    let request_json = serde_json::to_value(request).unwrap();
    assert_eq!(request_json["name"], "Knowledge Base");

    let store = VectorStoreObject::new("vs_1", "Knowledge Base");
    let store_json = serde_json::to_value(store).unwrap();
    assert_eq!(store_json["object"], "vector_store");

    let file = VectorStoreFileObject::new("file_1");
    let file_json = serde_json::to_value(file).unwrap();
    assert_eq!(file_json["object"], "vector_store.file");
}

#[test]
fn serializes_batch_contracts() {
    let request = CreateBatchRequest::new("file_1", "/v1/responses", "24h");
    let request_json = serde_json::to_value(request).unwrap();
    assert_eq!(request_json["input_file_id"], "file_1");
    assert_eq!(request_json["endpoint"], "/v1/responses");

    let batch = BatchObject::new("batch_1", "/v1/responses", "file_1");
    let json = serde_json::to_value(batch).unwrap();
    assert_eq!(json["object"], "batch");
}

#[test]
fn serializes_webhook_contracts() {
    let request =
        CreateWebhookRequest::new("https://example.com/webhook", vec!["response.completed"]);
    let request_json = serde_json::to_value(request).unwrap();
    assert_eq!(request_json["url"], "https://example.com/webhook");
    assert_eq!(request_json["events"][0], "response.completed");

    let webhook = WebhookObject::new("wh_1", "https://example.com/webhook");
    let json = serde_json::to_value(webhook).unwrap();
    assert_eq!(json["object"], "webhook_endpoint");
}

#[test]
fn serializes_eval_contracts() {
    let request = CreateEvalRequest::new("qa-benchmark", "file_1");
    let request_json = serde_json::to_value(request).unwrap();
    assert_eq!(request_json["name"], "qa-benchmark");
    assert_eq!(request_json["data_source_config"]["type"], "file");

    let eval = EvalObject::new("eval_1", "qa-benchmark");
    let json = serde_json::to_value(eval).unwrap();
    assert_eq!(json["object"], "eval");
}

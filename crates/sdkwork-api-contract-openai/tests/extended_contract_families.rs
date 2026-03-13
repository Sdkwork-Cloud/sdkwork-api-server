use sdkwork_api_contract_openai::assistants::{AssistantObject, CreateAssistantRequest};
use sdkwork_api_contract_openai::audio::{CreateTranscriptionRequest, TranscriptionObject};
use sdkwork_api_contract_openai::batches::{BatchObject, CreateBatchRequest, ListBatchesResponse};
use sdkwork_api_contract_openai::evals::{CreateEvalRequest, EvalObject};
use sdkwork_api_contract_openai::files::{
    CreateFileRequest, DeleteFileResponse, FileObject, ListFilesResponse,
};
use sdkwork_api_contract_openai::fine_tuning::{
    CreateFineTuningJobRequest, FineTuningJobObject, ListFineTuningJobsResponse,
};
use sdkwork_api_contract_openai::images::{CreateImageRequest, ImageObject, ImagesResponse};
use sdkwork_api_contract_openai::moderations::{CreateModerationRequest, ModerationResponse};
use sdkwork_api_contract_openai::realtime::{CreateRealtimeSessionRequest, RealtimeSessionObject};
use sdkwork_api_contract_openai::uploads::{
    AddUploadPartRequest, CompleteUploadRequest, CreateUploadRequest, UploadObject,
    UploadPartObject,
};
use sdkwork_api_contract_openai::vector_stores::{
    CreateVectorStoreFileRequest, CreateVectorStoreRequest, DeleteVectorStoreFileResponse,
    DeleteVectorStoreResponse, ListVectorStoreFilesResponse, ListVectorStoresResponse,
    UpdateVectorStoreRequest, VectorStoreFileObject, VectorStoreObject,
};
use sdkwork_api_contract_openai::webhooks::{CreateWebhookRequest, WebhookObject};

#[test]
fn serializes_file_contracts() {
    let request = CreateFileRequest::new("fine-tune", "train.jsonl", b"{}".to_vec());
    assert_eq!(request.purpose, "fine-tune");
    assert_eq!(request.filename, "train.jsonl");
    assert_eq!(request.bytes, b"{}".to_vec());
    assert_eq!(request.content_type, None);

    let file = FileObject::new("file_1", "train.jsonl", "fine-tune");
    let json = serde_json::to_value(file).unwrap();
    assert_eq!(json["object"], "file");
    assert_eq!(json["purpose"], "fine-tune");

    let list = ListFilesResponse::new(vec![FileObject::with_bytes(
        "file_1",
        "train.jsonl",
        "fine-tune",
        2,
    )]);
    let list_json = serde_json::to_value(list).unwrap();
    assert_eq!(list_json["object"], "list");
    assert_eq!(list_json["data"][0]["id"], "file_1");

    let deleted = DeleteFileResponse::deleted("file_1");
    let deleted_json = serde_json::to_value(deleted).unwrap();
    assert_eq!(deleted_json["deleted"], true);
}

#[test]
fn serializes_upload_contracts() {
    let request = CreateUploadRequest::new("batch", "input.jsonl", "application/jsonl", 1024);
    let request_json = serde_json::to_value(request).unwrap();
    assert_eq!(request_json["purpose"], "batch");
    assert_eq!(request_json["bytes"], 1024);
    assert_eq!(request_json["mime_type"], "application/jsonl");

    let upload = UploadObject::new("upload_1", "input.jsonl", "batch");
    let json = serde_json::to_value(upload).unwrap();
    assert_eq!(json["object"], "upload");
    assert_eq!(json["status"], "pending");

    let part_request = AddUploadPartRequest::new("upload_1", b"{}".to_vec());
    assert_eq!(part_request.upload_id, "upload_1");
    assert_eq!(part_request.data, b"{}".to_vec());

    let part = UploadPartObject::new("part_1", "upload_1");
    let part_json = serde_json::to_value(part).unwrap();
    assert_eq!(part_json["object"], "upload.part");

    let complete_request = CompleteUploadRequest::new("upload_1", vec!["part_1"]);
    let complete_json = serde_json::to_value(complete_request).unwrap();
    assert_eq!(complete_json["part_ids"][0], "part_1");

    let cancelled_upload = UploadObject::cancelled(
        "upload_1",
        "input.jsonl",
        "batch",
        "application/jsonl",
        1024,
        vec!["part_1".to_owned()],
    );
    let cancelled_json = serde_json::to_value(cancelled_upload).unwrap();
    assert_eq!(cancelled_json["status"], "cancelled");
}

#[test]
fn serializes_fine_tuning_contracts() {
    let request = CreateFineTuningJobRequest::new("file_1", "gpt-4.1-mini");
    let request_json = serde_json::to_value(request).unwrap();
    assert_eq!(request_json["training_file"], "file_1");

    let job = FineTuningJobObject::new("ftjob_1", "gpt-4.1-mini");
    let job_json = serde_json::to_value(job).unwrap();
    assert_eq!(job_json["object"], "fine_tuning.job");

    let list =
        ListFineTuningJobsResponse::new(vec![FineTuningJobObject::new("ftjob_1", "gpt-4.1-mini")]);
    let list_json = serde_json::to_value(list).unwrap();
    assert_eq!(list_json["object"], "list");

    let cancelled = FineTuningJobObject::cancelled("ftjob_1", "gpt-4.1-mini");
    let cancelled_json = serde_json::to_value(cancelled).unwrap();
    assert_eq!(cancelled_json["status"], "cancelled");
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

    let update = UpdateVectorStoreRequest::new("Knowledge Base Updated");
    let update_json = serde_json::to_value(update).unwrap();
    assert_eq!(update_json["name"], "Knowledge Base Updated");

    let store = VectorStoreObject::new("vs_1", "Knowledge Base");
    let store_json = serde_json::to_value(store).unwrap();
    assert_eq!(store_json["object"], "vector_store");

    let list =
        ListVectorStoresResponse::new(vec![VectorStoreObject::new("vs_1", "Knowledge Base")]);
    let list_json = serde_json::to_value(list).unwrap();
    assert_eq!(list_json["object"], "list");
    assert_eq!(list_json["data"][0]["id"], "vs_1");

    let deleted = DeleteVectorStoreResponse::deleted("vs_1");
    let deleted_json = serde_json::to_value(deleted).unwrap();
    assert_eq!(deleted_json["object"], "vector_store.deleted");
    assert_eq!(deleted_json["deleted"], true);

    let file_request = CreateVectorStoreFileRequest::new("file_1");
    let file_request_json = serde_json::to_value(file_request).unwrap();
    assert_eq!(file_request_json["file_id"], "file_1");

    let file = VectorStoreFileObject::new("file_1");
    let file_json = serde_json::to_value(file).unwrap();
    assert_eq!(file_json["object"], "vector_store.file");

    let file_list = ListVectorStoreFilesResponse::new(vec![VectorStoreFileObject::new("file_1")]);
    let file_list_json = serde_json::to_value(file_list).unwrap();
    assert_eq!(file_list_json["object"], "list");
    assert_eq!(file_list_json["data"][0]["id"], "file_1");

    let deleted_file = DeleteVectorStoreFileResponse::deleted("file_1");
    let deleted_file_json = serde_json::to_value(deleted_file).unwrap();
    assert_eq!(deleted_file_json["object"], "vector_store.file.deleted");
    assert_eq!(deleted_file_json["deleted"], true);
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

    let list =
        ListBatchesResponse::new(vec![BatchObject::new("batch_1", "/v1/responses", "file_1")]);
    let list_json = serde_json::to_value(list).unwrap();
    assert_eq!(list_json["object"], "list");

    let cancelled = BatchObject::cancelled("batch_1", "/v1/responses", "file_1");
    let cancelled_json = serde_json::to_value(cancelled).unwrap();
    assert_eq!(cancelled_json["status"], "cancelled");
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

use sdkwork_api_contract_openai::audio::{
    CreateSpeechRequest, CreateTranslationRequest, SpeechResponse, TranslationObject,
};
use sdkwork_api_contract_openai::completions::{CompletionObject, CreateCompletionRequest};
use sdkwork_api_contract_openai::fine_tuning::{CreateFineTuningJobRequest, FineTuningJobObject};
use sdkwork_api_contract_openai::videos::{CreateVideoRequest, VideoObject, VideosResponse};

#[test]
fn serializes_legacy_completions_contracts() {
    let request = CreateCompletionRequest::new("gpt-3.5-turbo-instruct", "Hello");
    let request_json = serde_json::to_value(request).unwrap();
    assert_eq!(request_json["model"], "gpt-3.5-turbo-instruct");
    assert_eq!(request_json["prompt"], "Hello");

    let completion = CompletionObject::new("cmpl_1", "Hello back");
    let json = serde_json::to_value(completion).unwrap();
    assert_eq!(json["object"], "text_completion");
    assert_eq!(json["choices"][0]["text"], "Hello back");
}

#[test]
fn serializes_fine_tuning_contracts() {
    let request = CreateFineTuningJobRequest::new("file_1", "gpt-4.1-mini");
    let request_json = serde_json::to_value(request).unwrap();
    assert_eq!(request_json["training_file"], "file_1");
    assert_eq!(request_json["model"], "gpt-4.1-mini");

    let job = FineTuningJobObject::new("ftjob_1", "gpt-4.1-mini");
    let json = serde_json::to_value(job).unwrap();
    assert_eq!(json["object"], "fine_tuning.job");
}

#[test]
fn serializes_video_contracts() {
    let request = CreateVideoRequest::new("sora-1", "A short cinematic flyover");
    let request_json = serde_json::to_value(request).unwrap();
    assert_eq!(request_json["model"], "sora-1");
    assert_eq!(request_json["prompt"], "A short cinematic flyover");

    let response = VideosResponse::new(vec![VideoObject::new(
        "video_1",
        "https://example.com/video.mp4",
    )]);
    let json = serde_json::to_value(response).unwrap();
    assert_eq!(json["data"][0]["object"], "video");
}

#[test]
fn serializes_audio_speech_and_translation_contracts() {
    let speech_request = CreateSpeechRequest::new("gpt-4o-mini-tts", "Nova", "Hello");
    let speech_json = serde_json::to_value(speech_request).unwrap();
    assert_eq!(speech_json["model"], "gpt-4o-mini-tts");
    assert_eq!(speech_json["voice"], "Nova");

    let speech_response = SpeechResponse::new("mp3", "base64-audio");
    let speech_response_json = serde_json::to_value(speech_response).unwrap();
    assert_eq!(speech_response_json["format"], "mp3");

    let translation_request = CreateTranslationRequest::new("gpt-4o-mini-transcribe", "file_1");
    let translation_json = serde_json::to_value(translation_request).unwrap();
    assert_eq!(translation_json["file_id"], "file_1");

    let translation = TranslationObject::new("translated text");
    let translation_object_json = serde_json::to_value(translation).unwrap();
    assert_eq!(translation_object_json["text"], "translated text");
}

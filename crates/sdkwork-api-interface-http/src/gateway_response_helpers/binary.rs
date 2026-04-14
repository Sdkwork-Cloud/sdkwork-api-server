use super::*;

fn local_binary_response_build_error_response(resource: &str) -> Response {
    bad_gateway_openai_response(format!("failed to build local {resource} response"))
}

fn local_speech_audio_decode_error_response() -> Response {
    bad_gateway_openai_response("failed to decode local speech audio")
}

pub(crate) fn local_file_content_response(
    tenant_id: &str,
    project_id: &str,
    file_id: &str,
) -> Response {
    match file_content(tenant_id, project_id, file_id) {
        Ok(bytes) => match Response::builder()
            .status(axum::http::StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/jsonl")
            .body(Body::from(bytes))
        {
            Ok(response) => response,
            Err(_) => local_binary_response_build_error_response("file content"),
        },
        Err(error) => local_gateway_error_response(error, "Requested file was not found."),
    }
}

pub(crate) fn local_container_file_content_response(
    tenant_id: &str,
    project_id: &str,
    container_id: &str,
    file_id: &str,
) -> Response {
    match sdkwork_api_app_gateway::container_file_content(
        tenant_id,
        project_id,
        container_id,
        file_id,
    ) {
        Ok(bytes) => match Response::builder()
            .status(axum::http::StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/octet-stream")
            .body(Body::from(bytes))
        {
            Ok(response) => response,
            Err(_) => local_binary_response_build_error_response("container file content"),
        },
        Err(error) => {
            local_gateway_error_response(error, "Requested container file was not found.")
        }
    }
}

pub(crate) fn local_video_content_response(
    tenant_id: &str,
    project_id: &str,
    video_id: &str,
) -> Response {
    match video_content(tenant_id, project_id, video_id) {
        Ok(bytes) => match Response::builder()
            .status(axum::http::StatusCode::OK)
            .header(header::CONTENT_TYPE, "video/mp4")
            .body(Body::from(bytes))
        {
            Ok(response) => response,
            Err(_) => local_binary_response_build_error_response("video content"),
        },
        Err(error) => local_gateway_error_response(error, "Requested video was not found."),
    }
}

pub(crate) fn local_music_content_response(
    tenant_id: &str,
    project_id: &str,
    music_id: &str,
) -> Response {
    match music_content(tenant_id, project_id, music_id) {
        Ok(bytes) => match Response::builder()
            .status(axum::http::StatusCode::OK)
            .header(header::CONTENT_TYPE, "audio/mpeg")
            .body(Body::from(bytes))
        {
            Ok(response) => response,
            Err(_) => local_binary_response_build_error_response("music content"),
        },
        Err(error) => local_gateway_error_response(error, "Requested music asset was not found."),
    }
}

pub(crate) fn local_speech_response(
    tenant_id: &str,
    project_id: &str,
    request: &CreateSpeechRequest,
) -> Response {
    let speech = match create_speech_response(tenant_id, project_id, request) {
        Ok(speech) => speech,
        Err(error) => {
            let message = error.to_string();
            if local_gateway_error_is_invalid_request(&message) {
                return invalid_request_openai_response(message, "invalid_audio_request");
            }
            return bad_gateway_openai_response("failed to process local speech fallback");
        }
    };
    if request.stream_format.as_deref() == Some("sse") {
        let delta = serde_json::json!({
            "type":"response.output_audio.delta",
            "delta": speech.audio_base64,
            "format": speech.format,
        })
        .to_string();
        let done = serde_json::json!({
            "type":"response.completed"
        })
        .to_string();
        let body = format!("{}{}", SseFrame::data(&delta), SseFrame::data(&done));
        return ([(header::CONTENT_TYPE, "text/event-stream")], body).into_response();
    }

    let bytes = match STANDARD.decode(speech.audio_base64.as_bytes()) {
        Ok(bytes) => bytes,
        Err(_) => return local_speech_audio_decode_error_response(),
    };

    match Response::builder()
        .status(axum::http::StatusCode::OK)
        .header(header::CONTENT_TYPE, speech_content_type(&speech.format))
        .body(Body::from(bytes))
    {
        Ok(response) => response,
        Err(_) => local_binary_response_build_error_response("speech audio"),
    }
}

fn speech_content_type(format: &str) -> &'static str {
    match format {
        "mp3" => "audio/mpeg",
        "opus" => "audio/opus",
        "aac" => "audio/aac",
        "flac" => "audio/flac",
        "pcm" => "audio/pcm",
        _ => "audio/wav",
    }
}

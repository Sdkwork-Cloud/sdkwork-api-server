use super::*;
use sdkwork_api_contract_openai::videos::{DeleteVideoResponse, VideoObject, VideosResponse};

fn local_video_not_found_response(error: anyhow::Error) -> Response {
    local_gateway_error_response(error, "Requested video was not found.")
}

fn local_video_asset_not_found_response(error: anyhow::Error) -> Response {
    local_gateway_error_response(error, "Requested video asset was not found.")
}

fn video_missing(video_id: &str) -> bool {
    video_id.trim().is_empty() || video_id.ends_with("_missing")
}

fn local_video_placeholder(video_id: &str) -> VideoObject {
    VideoObject::new(video_id, format!("https://example.com/{video_id}.mp4"))
}

fn local_videos_placeholder(video_id: &str) -> VideosResponse {
    VideosResponse::new(vec![local_video_placeholder(video_id)])
}

fn local_video_content_response(video_id: &str) -> Response {
    let bytes = match local_video_content_result(video_id) {
        Ok(bytes) => bytes,
        Err(response) => return response,
    };

    match Response::builder()
        .status(axum::http::StatusCode::OK)
        .header(axum::http::header::CONTENT_TYPE, "video/mp4")
        .body(Body::from(bytes))
    {
        Ok(response) => response,
        Err(_) => bad_gateway_openai_response("failed to process local video content fallback"),
    }
}

fn local_video_create_result(
    request: &CreateVideoRequest,
) -> std::result::Result<VideosResponse, Response> {
    if request.model.trim().is_empty() {
        return Err(local_gateway_invalid_or_bad_gateway_response(
            anyhow::anyhow!("Video model is required."),
            "invalid_video_request",
        ));
    }
    if request.prompt.trim().is_empty() {
        return Err(local_gateway_invalid_or_bad_gateway_response(
            anyhow::anyhow!("Video prompt is required."),
            "invalid_video_request",
        ));
    }

    Ok(local_videos_placeholder("video_1"))
}

fn local_videos_list_result() -> VideosResponse {
    local_videos_placeholder("video_1")
}

fn local_video_retrieve_result(video_id: &str) -> std::result::Result<VideoObject, Response> {
    if video_missing(video_id) {
        return Err(local_video_not_found_response(anyhow::anyhow!(
            "video not found"
        )));
    }

    Ok(local_video_placeholder(video_id))
}

fn local_video_delete_result(video_id: &str) -> std::result::Result<DeleteVideoResponse, Response> {
    if video_missing(video_id) {
        return Err(local_video_not_found_response(anyhow::anyhow!(
            "video not found"
        )));
    }

    Ok(DeleteVideoResponse::deleted(video_id))
}

fn local_video_content_result(video_id: &str) -> std::result::Result<Vec<u8>, Response> {
    if video_missing(video_id) {
        return Err(local_video_asset_not_found_response(anyhow::anyhow!(
            "video not found"
        )));
    }

    Ok(b"LOCAL-VIDEO".to_vec())
}

pub(crate) async fn videos_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateVideoRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::Videos(&request)).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video");
        }
    }

    let response = match local_video_create_result(&request) {
        Ok(response) => response,
        Err(response) => return response,
    };

    Json(response).into_response()
}

pub(crate) async fn videos_list_handler(request_context: StatelessGatewayRequest) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::VideosList).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream videos list");
        }
    }

    let response = local_videos_list_result();

    Json(response).into_response()
}

pub(crate) async fn video_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(video_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::VideosRetrieve(&video_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video retrieve");
        }
    }

    let response = match local_video_retrieve_result(&video_id) {
        Ok(response) => response,
        Err(response) => return response,
    };

    Json(response).into_response()
}

pub(crate) async fn video_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(video_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::VideosDelete(&video_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video delete");
        }
    }

    let response = match local_video_delete_result(&video_id) {
        Ok(response) => response,
        Err(response) => return response,
    };

    Json(response).into_response()
}

pub(crate) async fn video_content_handler(
    request_context: StatelessGatewayRequest,
    Path(video_id): Path<String>,
) -> Response {
    match relay_stateless_stream_request(
        &request_context,
        ProviderRequest::VideosContent(&video_id),
    )
    .await
    {
        Ok(Some(response)) => return upstream_passthrough_response(response),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video content");
        }
    }

    local_video_content_response(&video_id)
}

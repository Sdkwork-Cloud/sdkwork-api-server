use super::*;
use crate::gateway_provider_mirror_relay::{
    relay_provider_mirror_json_request, ProviderMirrorJsonRelayOutcome,
};

const GOOGLE_VEO_MIRROR_PROTOCOL_IDENTITY: &str = "google-veo";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GoogleVeoAction {
    PredictLongRunning,
    FetchPredictOperation,
    Predict,
}

fn google_veo_stateless_bad_gateway_response(action: &str) -> Response {
    bad_gateway_openai_response(format!(
        "failed to relay upstream google veo {action} request"
    ))
}

fn google_veo_invalid_request_response(message: impl Into<String>) -> Response {
    invalid_request_openai_response(message, "invalid_video_request")
}

fn parse_google_veo_models_tail(tail: &str) -> Option<(String, GoogleVeoAction)> {
    let (model, action) = tail.split_once(':')?;
    let action = match action.trim() {
        "predictLongRunning" => GoogleVeoAction::PredictLongRunning,
        "fetchPredictOperation" => GoogleVeoAction::FetchPredictOperation,
        "predict" => GoogleVeoAction::Predict,
        _ => return None,
    };
    let model = model.trim();
    if model.is_empty() {
        return None;
    }
    Some((model.to_owned(), action))
}

pub(crate) async fn video_google_veo_models_action_handler(
    request_context: StatelessGatewayRequest,
    Path((_project, _location, tail)): Path<(String, String, String)>,
    request: Request<Body>,
) -> Response {
    let Some((_model, action)) = parse_google_veo_models_tail(&tail) else {
        return google_veo_invalid_request_response("unsupported google veo models action");
    };
    let Some(upstream) = request_context.upstream() else {
        return google_veo_stateless_bad_gateway_response("models action");
    };

    let action_name = match action {
        GoogleVeoAction::PredictLongRunning => "predictLongRunning",
        GoogleVeoAction::FetchPredictOperation => "fetchPredictOperation",
        GoogleVeoAction::Predict => {
            return super::music_google::relay_stateless_google_music_predict_request(
                request_context,
                request,
            )
            .await
        }
    };
    if upstream.mirror_protocol_identity() != GOOGLE_VEO_MIRROR_PROTOCOL_IDENTITY {
        return google_veo_stateless_bad_gateway_response(action_name);
    }
    match relay_provider_mirror_json_request(
        GOOGLE_VEO_MIRROR_PROTOCOL_IDENTITY,
        upstream.base_url(),
        upstream.api_key(),
        request,
    )
    .await
    {
        Ok(ProviderMirrorJsonRelayOutcome::Json(response)) => Json(response).into_response(),
        Ok(ProviderMirrorJsonRelayOutcome::Error(response)) => response,
        Err(_) => google_veo_stateless_bad_gateway_response(action_name),
    }
}

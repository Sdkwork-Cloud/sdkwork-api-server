use super::*;

fn local_eval_error_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_eval_request",
        "Requested eval was not found.",
    )
}

pub(crate) async fn evals_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateEvalRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::Evals(&request)).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream eval");
        }
    }

    let response = match create_eval(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    ) {
        Ok(response) => response,
        Err(error) => return local_eval_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn evals_list_handler(request_context: StatelessGatewayRequest) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::EvalsList).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream evals list");
        }
    }

    let response = match list_evals(request_context.tenant_id(), request_context.project_id()) {
        Ok(response) => response,
        Err(error) => return local_eval_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn eval_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(eval_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::EvalsRetrieve(&eval_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream eval retrieve");
        }
    }

    let response = match get_eval(
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_eval_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn eval_update_handler(
    request_context: StatelessGatewayRequest,
    Path(eval_id): Path<String>,
    ExtractJson(request): ExtractJson<UpdateEvalRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::EvalsUpdate(&eval_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream eval update");
        }
    }

    let response = match update_eval(
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
        &request,
    ) {
        Ok(response) => response,
        Err(error) => return local_eval_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn eval_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(eval_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::EvalsDelete(&eval_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream eval delete");
        }
    }

    let response = match delete_eval(
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_eval_error_response(error),
    };

    Json(response).into_response()
}

use super::*;

fn local_eval_run_output_item_error_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_eval_request",
        "Requested eval run output item was not found.",
    )
}

pub(crate) async fn eval_run_output_items_list_handler(
    request_context: StatelessGatewayRequest,
    Path((eval_id, run_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::EvalRunOutputItemsList(&eval_id, &run_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream eval run output items list",
            );
        }
    }

    let response = match sdkwork_api_app_gateway::list_eval_run_output_items(
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
        &run_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_eval_run_output_item_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn eval_run_output_item_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path((eval_id, run_id, output_item_id)): Path<(String, String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::EvalRunOutputItemsRetrieve(&eval_id, &run_id, &output_item_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream eval run output item retrieve",
            );
        }
    }

    let response = match sdkwork_api_app_gateway::get_eval_run_output_item(
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
        &run_id,
        &output_item_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_eval_run_output_item_error_response(error),
    };

    Json(response).into_response()
}

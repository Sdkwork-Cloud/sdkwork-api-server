use super::*;

fn local_fine_tuning_job_error_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_fine_tuning_request",
        "Requested fine tuning job was not found.",
    )
}

pub(crate) async fn fine_tuning_jobs_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateFineTuningJobRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::FineTuningJobs(&request))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream fine tuning job");
        }
    }

    let response = match create_fine_tuning_job(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    ) {
        Ok(response) => response,
        Err(error) => return local_fine_tuning_job_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn fine_tuning_jobs_list_handler(
    request_context: StatelessGatewayRequest,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::FineTuningJobsList).await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream fine tuning jobs list");
        }
    }

    let response =
        match list_fine_tuning_jobs(request_context.tenant_id(), request_context.project_id()) {
            Ok(response) => response,
            Err(error) => return local_fine_tuning_job_error_response(error),
        };

    Json(response).into_response()
}

pub(crate) async fn fine_tuning_job_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(fine_tuning_job_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::FineTuningJobsRetrieve(&fine_tuning_job_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream fine tuning job retrieve",
            );
        }
    }

    let response = match get_fine_tuning_job(
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuning_job_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_fine_tuning_job_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn fine_tuning_job_cancel_handler(
    request_context: StatelessGatewayRequest,
    Path(fine_tuning_job_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::FineTuningJobsCancel(&fine_tuning_job_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream fine tuning job cancel");
        }
    }

    let response = match cancel_fine_tuning_job(
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuning_job_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_fine_tuning_job_error_response(error),
    };

    Json(response).into_response()
}

use super::*;

pub(crate) fn apply_stateless_management_routes(
    router: Router<StatelessGatewayContext>,
) -> Router<StatelessGatewayContext> {
    router
        .route(
            "/v1/fine_tuning/jobs",
            get(fine_tuning_jobs_list_handler).post(fine_tuning_jobs_handler),
        )
        .route(
            "/v1/fine_tuning/jobs/{fine_tuning_job_id}",
            get(fine_tuning_job_retrieve_handler),
        )
        .route(
            "/v1/fine_tuning/jobs/{fine_tuning_job_id}/cancel",
            post(fine_tuning_job_cancel_handler),
        )
        .route(
            "/v1/fine_tuning/jobs/{fine_tuning_job_id}/events",
            get(fine_tuning_job_events_handler),
        )
        .route(
            "/v1/fine_tuning/jobs/{fine_tuning_job_id}/checkpoints",
            get(fine_tuning_job_checkpoints_handler),
        )
        .route(
            "/v1/fine_tuning/jobs/{fine_tuning_job_id}/pause",
            post(fine_tuning_job_pause_handler),
        )
        .route(
            "/v1/fine_tuning/jobs/{fine_tuning_job_id}/resume",
            post(fine_tuning_job_resume_handler),
        )
        .route(
            "/v1/fine_tuning/checkpoints/{fine_tuned_model_checkpoint}/permissions",
            get(fine_tuning_checkpoint_permissions_list_handler)
                .post(fine_tuning_checkpoint_permissions_handler),
        )
        .route(
            "/v1/fine_tuning/checkpoints/{fine_tuned_model_checkpoint}/permissions/{permission_id}",
            axum::routing::delete(fine_tuning_checkpoint_permission_delete_handler),
        )
        .route(
            "/v1/assistants",
            get(assistants_list_handler).post(assistants_handler),
        )
        .route(
            "/v1/assistants/{assistant_id}",
            get(assistant_retrieve_handler)
                .post(assistant_update_handler)
                .delete(assistant_delete_handler),
        )
        .route(
            "/v1/webhooks",
            get(webhooks_list_handler).post(webhooks_handler),
        )
        .route(
            "/v1/webhooks/{webhook_id}",
            get(webhook_retrieve_handler)
                .post(webhook_update_handler)
                .delete(webhook_delete_handler),
        )
        .route("/v1/realtime/sessions", post(realtime_sessions_handler))
}

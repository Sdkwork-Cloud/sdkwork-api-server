use super::*;

pub(crate) fn apply_stateless_eval_and_vector_routes(
    router: Router<StatelessGatewayContext>,
) -> Router<StatelessGatewayContext> {
    router
        .route("/v1/evals", get(evals_list_handler).post(evals_handler))
        .route(
            "/v1/evals/{eval_id}",
            get(eval_retrieve_handler)
                .post(eval_update_handler)
                .delete(eval_delete_handler),
        )
        .route(
            "/v1/evals/{eval_id}/runs",
            get(eval_runs_list_handler).post(eval_runs_handler),
        )
        .route(
            "/v1/evals/{eval_id}/runs/{run_id}",
            get(eval_run_retrieve_handler).delete(eval_run_delete_handler),
        )
        .route(
            "/v1/evals/{eval_id}/runs/{run_id}/cancel",
            post(eval_run_cancel_handler),
        )
        .route(
            "/v1/evals/{eval_id}/runs/{run_id}/output_items",
            get(eval_run_output_items_list_handler),
        )
        .route(
            "/v1/evals/{eval_id}/runs/{run_id}/output_items/{output_item_id}",
            get(eval_run_output_item_retrieve_handler),
        )
        .route(
            "/v1/batches",
            get(batches_list_handler).post(batches_handler),
        )
        .route("/v1/batches/{batch_id}", get(batch_retrieve_handler))
        .route("/v1/batches/{batch_id}/cancel", post(batch_cancel_handler))
        .route(
            "/v1/vector_stores",
            get(vector_stores_list_handler).post(vector_stores_handler),
        )
        .route(
            "/v1/vector_stores/{vector_store_id}",
            get(vector_store_retrieve_handler)
                .post(vector_store_update_handler)
                .delete(vector_store_delete_handler),
        )
        .route(
            "/v1/vector_stores/{vector_store_id}/search",
            post(vector_store_search_handler),
        )
        .route(
            "/v1/vector_stores/{vector_store_id}/files",
            get(vector_store_files_list_handler).post(vector_store_files_handler),
        )
        .route(
            "/v1/vector_stores/{vector_store_id}/files/{file_id}",
            get(vector_store_file_retrieve_handler).delete(vector_store_file_delete_handler),
        )
        .route(
            "/v1/vector_stores/{vector_store_id}/file_batches",
            post(vector_store_file_batches_handler),
        )
        .route(
            "/v1/vector_stores/{vector_store_id}/file_batches/{batch_id}",
            get(vector_store_file_batch_retrieve_handler),
        )
        .route(
            "/v1/vector_stores/{vector_store_id}/file_batches/{batch_id}/cancel",
            post(vector_store_file_batch_cancel_handler),
        )
        .route(
            "/v1/vector_stores/{vector_store_id}/file_batches/{batch_id}/files",
            get(vector_store_file_batch_files_handler),
        )
}

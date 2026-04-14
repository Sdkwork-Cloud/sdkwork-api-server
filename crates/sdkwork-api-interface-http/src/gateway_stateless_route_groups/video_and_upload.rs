use super::*;

pub(crate) fn apply_stateless_video_and_upload_routes(
    router: Router<StatelessGatewayContext>,
) -> Router<StatelessGatewayContext> {
    router
        .route("/v1/videos", get(videos_list_handler).post(videos_handler))
        .route(
            "/v1/videos/{video_id}",
            get(video_retrieve_handler).delete(video_delete_handler),
        )
        .route("/v1/videos/{video_id}/content", get(video_content_handler))
        .route("/v1/videos/{video_id}/remix", post(video_remix_handler))
        .route(
            "/v1/videos/characters",
            post(video_character_create_handler),
        )
        .route(
            "/v1/videos/characters/{character_id}",
            get(video_character_retrieve_canonical_handler),
        )
        .route("/v1/videos/edits", post(video_edits_handler))
        .route("/v1/videos/extensions", post(video_extensions_handler))
        .route(
            "/v1/videos/{video_id}/characters",
            get(video_characters_list_handler),
        )
        .route(
            "/v1/videos/{video_id}/characters/{character_id}",
            get(video_character_retrieve_handler).post(video_character_update_handler),
        )
        .route("/v1/videos/{video_id}/extend", post(video_extend_handler))
        .route("/v1/uploads", post(uploads_handler))
        .route("/v1/uploads/{upload_id}/parts", post(upload_parts_handler))
        .route(
            "/v1/uploads/{upload_id}/complete",
            post(upload_complete_handler),
        )
        .route(
            "/v1/uploads/{upload_id}/cancel",
            post(upload_cancel_handler),
        )
}

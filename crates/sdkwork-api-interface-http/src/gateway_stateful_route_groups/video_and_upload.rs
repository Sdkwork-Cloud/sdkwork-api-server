use super::*;

pub(crate) fn apply_stateful_video_and_upload_routes(
    router: Router<GatewayApiState>,
) -> Router<GatewayApiState> {
    router
        .route(
            "/v1/videos",
            get(videos_list_with_state_handler).post(videos_with_state_handler),
        )
        .route(
            "/v1/videos/{video_id}",
            get(video_retrieve_with_state_handler).delete(video_delete_with_state_handler),
        )
        .route(
            "/v1/videos/{video_id}/content",
            get(video_content_with_state_handler),
        )
        .route(
            "/v1/videos/{video_id}/remix",
            post(video_remix_with_state_handler),
        )
        .route(
            "/v1/videos/characters",
            post(video_character_create_with_state_handler),
        )
        .route(
            "/v1/videos/characters/{character_id}",
            get(video_character_retrieve_canonical_with_state_handler),
        )
        .route("/v1/videos/edits", post(video_edits_with_state_handler))
        .route(
            "/v1/videos/extensions",
            post(video_extensions_with_state_handler),
        )
        .route(
            "/v1/videos/{video_id}/characters",
            get(video_characters_list_with_state_handler),
        )
        .route(
            "/v1/videos/{video_id}/characters/{character_id}",
            get(video_character_retrieve_with_state_handler)
                .post(video_character_update_with_state_handler),
        )
        .route(
            "/v1/videos/{video_id}/extend",
            post(video_extend_with_state_handler),
        )
        .route("/v1/uploads", post(uploads_with_state_handler))
        .route(
            "/v1/uploads/{upload_id}/parts",
            post(upload_parts_with_state_handler),
        )
        .route(
            "/v1/uploads/{upload_id}/complete",
            post(upload_complete_with_state_handler),
        )
        .route(
            "/v1/uploads/{upload_id}/cancel",
            post(upload_cancel_with_state_handler),
        )
}

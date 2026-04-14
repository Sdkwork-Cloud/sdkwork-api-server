use super::*;

pub(crate) fn apply_stateful_inference_and_storage_routes(
    router: Router<GatewayApiState>,
) -> Router<GatewayApiState> {
    router
        .route("/v1/embeddings", post(embeddings_with_state_handler))
        .route("/v1/moderations", post(moderations_with_state_handler))
        .route(
            "/v1/images/generations",
            post(image_generations_with_state_handler),
        )
        .route("/v1/images/edits", post(image_edits_with_state_handler))
        .route(
            "/v1/images/variations",
            post(image_variations_with_state_handler),
        )
        .route(
            "/v1/audio/transcriptions",
            post(transcriptions_with_state_handler),
        )
        .route(
            "/v1/audio/translations",
            post(translations_with_state_handler),
        )
        .route("/v1/audio/speech", post(audio_speech_with_state_handler))
        .route("/v1/audio/voices", get(audio_voices_with_state_handler))
        .route(
            "/v1/audio/voice_consents",
            post(audio_voice_consents_with_state_handler),
        )
        .route(
            "/v1/music",
            get(music_list_with_state_handler).post(music_with_state_handler),
        )
        .route("/v1/music/lyrics", post(music_lyrics_with_state_handler))
        .route(
            "/v1/music/{music_id}",
            get(music_retrieve_with_state_handler).delete(music_delete_with_state_handler),
        )
        .route(
            "/v1/music/{music_id}/content",
            get(music_content_with_state_handler),
        )
        .route(
            "/v1/containers",
            get(containers_list_with_state_handler).post(containers_with_state_handler),
        )
        .route(
            "/v1/containers/{container_id}",
            get(container_retrieve_with_state_handler).delete(container_delete_with_state_handler),
        )
        .route(
            "/v1/containers/{container_id}/files",
            get(container_files_list_with_state_handler).post(container_files_with_state_handler),
        )
        .route(
            "/v1/containers/{container_id}/files/{file_id}",
            get(container_file_retrieve_with_state_handler)
                .delete(container_file_delete_with_state_handler),
        )
        .route(
            "/v1/containers/{container_id}/files/{file_id}/content",
            get(container_file_content_with_state_handler),
        )
        .route(
            "/v1/files",
            get(files_list_with_state_handler).post(files_with_state_handler),
        )
        .route(
            "/v1/files/{file_id}",
            get(file_retrieve_with_state_handler).delete(file_delete_with_state_handler),
        )
        .route(
            "/v1/files/{file_id}/content",
            get(file_content_with_state_handler),
        )
}

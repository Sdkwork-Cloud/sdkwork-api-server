use super::*;

pub(crate) fn apply_stateless_inference_and_storage_routes(
    router: Router<StatelessGatewayContext>,
) -> Router<StatelessGatewayContext> {
    router
        .route("/v1/embeddings", post(embeddings_handler))
        .route("/v1/moderations", post(moderations_handler))
        .route("/v1/images/generations", post(image_generations_handler))
        .route("/v1/images/edits", post(image_edits_handler))
        .route("/v1/images/variations", post(image_variations_handler))
        .route("/v1/audio/transcriptions", post(transcriptions_handler))
        .route("/v1/audio/translations", post(translations_handler))
        .route("/v1/audio/speech", post(audio_speech_handler))
        .route("/v1/audio/voices", get(audio_voices_handler))
        .route(
            "/v1/audio/voice_consents",
            post(audio_voice_consents_handler),
        )
        .route("/v1/music", get(music_list_handler).post(music_handler))
        .route("/v1/music/lyrics", post(music_lyrics_handler))
        .route(
            "/v1/music/{music_id}",
            get(music_retrieve_handler).delete(music_delete_handler),
        )
        .route("/v1/music/{music_id}/content", get(music_content_handler))
        .route(
            "/v1/containers",
            get(containers_list_handler).post(containers_handler),
        )
        .route(
            "/v1/containers/{container_id}",
            get(container_retrieve_handler).delete(container_delete_handler),
        )
        .route(
            "/v1/containers/{container_id}/files",
            get(container_files_list_handler).post(container_files_handler),
        )
        .route(
            "/v1/containers/{container_id}/files/{file_id}",
            get(container_file_retrieve_handler).delete(container_file_delete_handler),
        )
        .route(
            "/v1/containers/{container_id}/files/{file_id}/content",
            get(container_file_content_handler),
        )
        .route("/v1/files", get(files_list_handler).post(files_handler))
        .route(
            "/v1/files/{file_id}",
            get(file_retrieve_handler).delete(file_delete_handler),
        )
        .route("/v1/files/{file_id}/content", get(file_content_handler))
}

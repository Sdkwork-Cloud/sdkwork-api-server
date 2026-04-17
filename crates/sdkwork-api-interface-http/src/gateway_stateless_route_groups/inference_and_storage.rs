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
        .route(
            "/api/v1/services/aigc/image-generation/generation",
            post(dashscope_image_generation_handler),
        )
        .route(
            "/api/v3/images/generations",
            post(volcengine_image_generation_handler),
        )
        .route(
            "/api/v1/services/aigc/video-generation/video-synthesis",
            post(dashscope_video_synthesis_handler),
        )
        .route(
            "/v1/projects/{project}/locations/{location}/publishers/google/models/{*tail}",
            post(video_google_veo_models_action_handler),
        )
        .route(
            "/api/v1/contents/generations/tasks",
            post(video_volcengine_task_create_handler),
        )
        .route(
            "/api/v1/contents/generations/tasks/{id}",
            get(video_volcengine_task_get_handler),
        )
        .route(
            "/api/v1/tasks/{task_id}",
            get(dashscope_image_task_get_handler),
        )
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
            "/v1/music_generation",
            post(music_minimax_generation_handler),
        )
        .route("/v1/lyrics_generation", post(music_minimax_lyrics_handler))
        .route(
            "/v1/video_generation",
            post(video_minimax_generation_handler),
        )
        .route(
            "/v1/query/video_generation",
            get(video_minimax_generation_query_handler),
        )
        .route("/ent/v2/text2video", post(video_vidu_text2video_handler))
        .route("/ent/v2/img2video", post(video_vidu_img2video_handler))
        .route(
            "/ent/v2/reference2video",
            post(video_vidu_reference2video_handler),
        )
        .route(
            "/ent/v2/tasks/{id}/creations",
            get(video_vidu_task_creations_handler),
        )
        .route(
            "/ent/v2/tasks/{id}/cancel",
            post(video_vidu_task_cancel_handler),
        )
        .route("/api/v1/generate", post(music_suno_generate_handler))
        .route(
            "/api/v1/generate/record-info",
            get(music_suno_generate_record_info_handler),
        )
        .route("/api/v1/lyrics", post(music_suno_lyrics_handler))
        .route(
            "/api/v1/lyrics/record-info",
            get(music_suno_lyrics_record_info_handler),
        )
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
            "/v1/files/retrieve",
            get(video_minimax_file_retrieve_handler),
        )
        .route(
            "/v1/files/{file_id}",
            get(file_retrieve_handler).delete(file_delete_handler),
        )
        .route("/v1/files/{file_id}/content", get(file_content_handler))
}

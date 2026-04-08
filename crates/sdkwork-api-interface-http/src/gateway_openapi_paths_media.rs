use super::*;

    #[utoipa::path(
        post,
        path = "/v1/images/generations",
        tag = "images",
        request_body = CreateImageRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Image generation result.", body = ImagesResponse),
            (status = 400, description = "Invalid image generation payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the image generation.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn image_generations() {}

    #[utoipa::path(
        post,
        path = "/v1/images/edits",
        tag = "images",
        request_body(
            content = CreateImageEditRequest,
            content_type = "multipart/form-data",
            description = "Multipart image edit payload."
        ),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Image edit result.", body = ImagesResponse),
            (status = 400, description = "Invalid image edit payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to edit the image.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn image_edits() {}

    #[utoipa::path(
        post,
        path = "/v1/images/variations",
        tag = "images",
        request_body(
            content = CreateImageVariationRequest,
            content_type = "multipart/form-data",
            description = "Multipart image variation payload."
        ),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Image variation result.", body = ImagesResponse),
            (status = 400, description = "Invalid image variation payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the image variation.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn image_variations() {}

    #[utoipa::path(
        post,
        path = "/v1/audio/transcriptions",
        tag = "audio",
        request_body = CreateTranscriptionRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Audio transcription result.", body = TranscriptionObject),
            (status = 400, description = "Invalid transcription payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the transcription.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn transcriptions() {}

    #[utoipa::path(
        post,
        path = "/v1/audio/translations",
        tag = "audio",
        request_body = CreateTranslationRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Audio translation result.", body = TranslationObject),
            (status = 400, description = "Invalid translation payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the translation.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn translations() {}

    #[utoipa::path(
        post,
        path = "/v1/audio/speech",
        tag = "audio",
        request_body = CreateSpeechRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Audio speech synthesis result.", body = SpeechResponse),
            (status = 400, description = "Invalid speech synthesis payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to synthesize speech.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn audio_speech() {}

    #[utoipa::path(
        get,
        path = "/v1/audio/voices",
        tag = "audio",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Available audio voices.", body = ListVoicesResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load the voice catalog.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn audio_voices() {}

    #[utoipa::path(
        post,
        path = "/v1/audio/voice_consents",
        tag = "audio",
        request_body = CreateVoiceConsentRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Audio voice consent approval result.", body = VoiceConsentObject),
            (status = 400, description = "Invalid audio voice consent payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the audio voice consent.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn audio_voice_consents() {}

use super::*;

#[utoipa::path(
        get,
        path = "/v1/vector_stores",
        tag = "vector-stores",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible vector stores.", body = ListVectorStoresResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load vector stores.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn vector_stores_list() {}

#[utoipa::path(
        post,
        path = "/v1/vector_stores",
        tag = "vector-stores",
        request_body = CreateVectorStoreRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Created vector store.", body = VectorStoreObject),
            (status = 400, description = "Invalid vector store payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the vector store.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn vector_stores_create() {}

#[utoipa::path(
        get,
        path = "/v1/vector_stores/{vector_store_id}",
        tag = "vector-stores",
        params(("vector_store_id" = String, Path, description = "Vector store identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible vector store metadata.", body = VectorStoreObject),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested vector store was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load the vector store.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn vector_store_get() {}

#[utoipa::path(
        post,
        path = "/v1/vector_stores/{vector_store_id}",
        tag = "vector-stores",
        params(("vector_store_id" = String, Path, description = "Vector store identifier.")),
        request_body = UpdateVectorStoreRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Updated vector store.", body = VectorStoreObject),
            (status = 400, description = "Invalid vector store update payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested vector store was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to update the vector store.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn vector_store_update() {}

#[utoipa::path(
        delete,
        path = "/v1/vector_stores/{vector_store_id}",
        tag = "vector-stores",
        params(("vector_store_id" = String, Path, description = "Vector store identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Deleted vector store.", body = DeleteVectorStoreResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested vector store was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to delete the vector store.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn vector_store_delete() {}

#[utoipa::path(
        post,
        path = "/v1/vector_stores/{vector_store_id}/search",
        tag = "vector-stores",
        params(("vector_store_id" = String, Path, description = "Vector store identifier.")),
        request_body = SearchVectorStoreRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Vector store search result.", body = SearchVectorStoreResponse),
            (status = 400, description = "Invalid vector store search payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested vector store was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to search the vector store.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn vector_store_search() {}

#[utoipa::path(
        get,
        path = "/v1/vector_stores/{vector_store_id}/files",
        tag = "vector-stores",
        params(("vector_store_id" = String, Path, description = "Vector store identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible vector store files.", body = ListVectorStoreFilesResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested vector store was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load vector store files.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn vector_store_files_list() {}

#[utoipa::path(
        post,
        path = "/v1/vector_stores/{vector_store_id}/files",
        tag = "vector-stores",
        params(("vector_store_id" = String, Path, description = "Vector store identifier.")),
        request_body = CreateVectorStoreFileRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Created vector store file link.", body = VectorStoreFileObject),
            (status = 400, description = "Invalid vector store file payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested vector store was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the vector store file link.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn vector_store_files_create() {}

#[utoipa::path(
        get,
        path = "/v1/vector_stores/{vector_store_id}/files/{file_id}",
        tag = "vector-stores",
        params(
            ("vector_store_id" = String, Path, description = "Vector store identifier."),
            ("file_id" = String, Path, description = "File identifier.")
        ),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible vector store file metadata.", body = VectorStoreFileObject),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested vector store file was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load the vector store file.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn vector_store_file_get() {}

#[utoipa::path(
        delete,
        path = "/v1/vector_stores/{vector_store_id}/files/{file_id}",
        tag = "vector-stores",
        params(
            ("vector_store_id" = String, Path, description = "Vector store identifier."),
            ("file_id" = String, Path, description = "File identifier.")
        ),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Deleted vector store file link.", body = DeleteVectorStoreFileResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested vector store file was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to delete the vector store file link.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn vector_store_file_delete() {}

#[utoipa::path(
        post,
        path = "/v1/vector_stores/{vector_store_id}/file_batches",
        tag = "vector-stores",
        params(("vector_store_id" = String, Path, description = "Vector store identifier.")),
        request_body = CreateVectorStoreFileBatchRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Created vector store file batch.", body = VectorStoreFileBatchObject),
            (status = 400, description = "Invalid vector store file batch payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested vector store was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the vector store file batch.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn vector_store_file_batches_create() {}

#[utoipa::path(
        get,
        path = "/v1/vector_stores/{vector_store_id}/file_batches/{batch_id}",
        tag = "vector-stores",
        params(
            ("vector_store_id" = String, Path, description = "Vector store identifier."),
            ("batch_id" = String, Path, description = "Vector store file batch identifier.")
        ),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible vector store file batch metadata.", body = VectorStoreFileBatchObject),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested vector store file batch was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load the vector store file batch.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn vector_store_file_batch_get() {}

#[utoipa::path(
        post,
        path = "/v1/vector_stores/{vector_store_id}/file_batches/{batch_id}/cancel",
        tag = "vector-stores",
        params(
            ("vector_store_id" = String, Path, description = "Vector store identifier."),
            ("batch_id" = String, Path, description = "Vector store file batch identifier.")
        ),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Cancelled vector store file batch.", body = VectorStoreFileBatchObject),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested vector store file batch was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to cancel the vector store file batch.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn vector_store_file_batch_cancel() {}

#[utoipa::path(
        get,
        path = "/v1/vector_stores/{vector_store_id}/file_batches/{batch_id}/files",
        tag = "vector-stores",
        params(
            ("vector_store_id" = String, Path, description = "Vector store identifier."),
            ("batch_id" = String, Path, description = "Vector store file batch identifier.")
        ),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible vector store file batch files.", body = ListVectorStoreFilesResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested vector store file batch was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load vector store file batch files.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn vector_store_file_batch_files_list() {}

#[utoipa::path(
        post,
        path = "/v1/messages",
        tag = "compatibility",
        request_body = Value,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Anthropic-compatible message result.", body = Value),
            (status = 400, description = "Invalid Anthropic compatibility payload.", body = Value),
            (status = 401, description = "Missing or invalid gateway API key.", body = Value),
            (status = 500, description = "Gateway failed to serve the Anthropic compatibility route.", body = Value)
        )
    )]
pub(crate) async fn anthropic_messages() {}

#[utoipa::path(
        post,
        path = "/v1/messages/count_tokens",
        tag = "compatibility",
        request_body = Value,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Anthropic-compatible token count result.", body = Value),
            (status = 400, description = "Invalid Anthropic token count payload.", body = Value),
            (status = 401, description = "Missing or invalid gateway API key.", body = Value),
            (status = 500, description = "Gateway failed to serve the Anthropic token count route.", body = Value)
        )
    )]
pub(crate) async fn anthropic_count_tokens() {}

#[utoipa::path(
        post,
        path = "/v1beta/models/{tail}",
        tag = "compatibility",
        params(("tail" = String, Path, description = "Gemini compatibility route suffix.")),
        request_body = Value,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Gemini-compatible route result.", body = Value),
            (status = 400, description = "Invalid Gemini compatibility payload.", body = Value),
            (status = 401, description = "Missing or invalid gateway API key.", body = Value),
            (status = 500, description = "Gateway failed to serve the Gemini compatibility route.", body = Value)
        )
    )]
pub(crate) async fn gemini_models_compat() {}

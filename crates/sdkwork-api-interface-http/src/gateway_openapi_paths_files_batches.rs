use super::*;

#[utoipa::path(
        get,
        path = "/v1/files",
        tag = "files",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible files.", body = ListFilesResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load files.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn files_list() {}

#[utoipa::path(
        post,
        path = "/v1/files",
        tag = "files",
        request_body(
            content = CreateFileRequest,
            content_type = "multipart/form-data",
            description = "Multipart file upload payload."
        ),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Created file.", body = FileObject),
            (status = 400, description = "Invalid file upload payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the file.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn files_create() {}

#[utoipa::path(
        get,
        path = "/v1/files/{file_id}",
        tag = "files",
        params(("file_id" = String, Path, description = "File identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible file metadata.", body = FileObject),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested file was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load the file.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn file_get() {}

#[utoipa::path(
        delete,
        path = "/v1/files/{file_id}",
        tag = "files",
        params(("file_id" = String, Path, description = "File identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Deleted file.", body = DeleteFileResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested file was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to delete the file.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn file_delete() {}

#[utoipa::path(
        post,
        path = "/v1/uploads",
        tag = "uploads",
        request_body = CreateUploadRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Created upload session.", body = UploadObject),
            (status = 400, description = "Invalid upload payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the upload session.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn uploads_create() {}

#[utoipa::path(
        post,
        path = "/v1/uploads/{upload_id}/parts",
        tag = "uploads",
        params(("upload_id" = String, Path, description = "Upload session identifier.")),
        request_body(
            content = AddUploadPartRequest,
            content_type = "multipart/form-data",
            description = "Multipart upload part payload."
        ),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Created upload part.", body = UploadPartObject),
            (status = 400, description = "Invalid upload part payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to add the upload part.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn upload_parts_create() {}

#[utoipa::path(
        post,
        path = "/v1/uploads/{upload_id}/complete",
        tag = "uploads",
        params(("upload_id" = String, Path, description = "Upload session identifier.")),
        request_body = CompleteUploadRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Completed upload session.", body = UploadObject),
            (status = 400, description = "Invalid upload completion payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested upload session was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to complete the upload session.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn upload_complete() {}

#[utoipa::path(
        post,
        path = "/v1/uploads/{upload_id}/cancel",
        tag = "uploads",
        params(("upload_id" = String, Path, description = "Upload session identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Cancelled upload session.", body = UploadObject),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested upload session was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to cancel the upload session.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn upload_cancel() {}

#[utoipa::path(
        get,
        path = "/v1/batches",
        tag = "batches",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible batches.", body = ListBatchesResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load batches.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn batches_list() {}

#[utoipa::path(
        post,
        path = "/v1/batches",
        tag = "batches",
        request_body = CreateBatchRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Created batch.", body = BatchObject),
            (status = 400, description = "Invalid batch payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the batch.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn batches_create() {}

#[utoipa::path(
        get,
        path = "/v1/batches/{batch_id}",
        tag = "batches",
        params(("batch_id" = String, Path, description = "Batch identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible batch metadata.", body = BatchObject),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested batch was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load the batch.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn batch_get() {}

#[utoipa::path(
        post,
        path = "/v1/batches/{batch_id}/cancel",
        tag = "batches",
        params(("batch_id" = String, Path, description = "Batch identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Cancelled batch.", body = BatchObject),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested batch was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to cancel the batch.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn batch_cancel() {}

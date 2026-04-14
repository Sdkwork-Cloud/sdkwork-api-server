use super::errors::{bad_multipart, missing_multipart_field};
use super::*;

pub(crate) async fn parse_upload_part_request(
    upload_id: String,
    mut multipart: Multipart,
) -> Result<AddUploadPartRequest, Response> {
    let mut data = None;
    let mut filename = None;
    let mut content_type = None;

    while let Some(field) = multipart.next_field().await.map_err(bad_multipart)? {
        if field.name() == Some("data") {
            filename = field.file_name().map(ToOwned::to_owned);
            content_type = field.content_type().map(ToOwned::to_owned);
            data = Some(field.bytes().await.map_err(bad_multipart)?.to_vec());
        }
    }

    let mut request =
        AddUploadPartRequest::new(upload_id, data.ok_or_else(missing_multipart_field)?);
    if let Some(filename) = filename {
        request = request.with_filename(filename);
    }
    if let Some(content_type) = content_type {
        request = request.with_content_type(content_type);
    }
    Ok(request)
}

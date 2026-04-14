use super::errors::{bad_multipart, missing_multipart_field};
use super::*;

pub(crate) async fn parse_image_edit_request(
    mut multipart: Multipart,
) -> Result<CreateImageEditRequest, Response> {
    let mut model = None;
    let mut prompt = None;
    let mut image = None;
    let mut mask = None;
    let mut n = None;
    let mut quality = None;
    let mut response_format = None;
    let mut size = None;
    let mut user = None;

    while let Some(field) = multipart.next_field().await.map_err(bad_multipart)? {
        match field.name() {
            Some("model") => model = Some(field.text().await.map_err(bad_multipart)?),
            Some("prompt") => prompt = Some(field.text().await.map_err(bad_multipart)?),
            Some("image") => image = Some(parse_image_upload_field(field).await?),
            Some("mask") => mask = Some(parse_image_upload_field(field).await?),
            Some("n") => {
                n = Some(
                    parse_u32_field(field.text().await.map_err(bad_multipart)?).map_err(
                        |message| (axum::http::StatusCode::BAD_REQUEST, message).into_response(),
                    )?,
                )
            }
            Some("quality") => quality = Some(field.text().await.map_err(bad_multipart)?),
            Some("response_format") => {
                response_format = Some(field.text().await.map_err(bad_multipart)?)
            }
            Some("size") => size = Some(field.text().await.map_err(bad_multipart)?),
            Some("user") => user = Some(field.text().await.map_err(bad_multipart)?),
            _ => {}
        }
    }

    let mut request = CreateImageEditRequest::new(
        prompt.ok_or_else(missing_multipart_field)?,
        image.ok_or_else(missing_multipart_field)?,
    );
    if let Some(model) = model {
        request = request.with_model(model);
    }
    if let Some(mask) = mask {
        request = request.with_mask(mask);
    }
    request.n = n;
    request.quality = quality;
    request.response_format = response_format;
    request.size = size;
    request.user = user;

    Ok(request)
}

pub(crate) async fn parse_image_variation_request(
    mut multipart: Multipart,
) -> Result<CreateImageVariationRequest, Response> {
    let mut model = None;
    let mut image = None;
    let mut n = None;
    let mut response_format = None;
    let mut size = None;
    let mut user = None;

    while let Some(field) = multipart.next_field().await.map_err(bad_multipart)? {
        match field.name() {
            Some("model") => model = Some(field.text().await.map_err(bad_multipart)?),
            Some("image") => image = Some(parse_image_upload_field(field).await?),
            Some("n") => {
                n = Some(
                    parse_u32_field(field.text().await.map_err(bad_multipart)?).map_err(
                        |message| (axum::http::StatusCode::BAD_REQUEST, message).into_response(),
                    )?,
                )
            }
            Some("response_format") => {
                response_format = Some(field.text().await.map_err(bad_multipart)?)
            }
            Some("size") => size = Some(field.text().await.map_err(bad_multipart)?),
            Some("user") => user = Some(field.text().await.map_err(bad_multipart)?),
            _ => {}
        }
    }

    let mut request = CreateImageVariationRequest::new(image.ok_or_else(missing_multipart_field)?);
    if let Some(model) = model {
        request = request.with_model(model);
    }
    request.n = n;
    request.response_format = response_format;
    request.size = size;
    request.user = user;

    Ok(request)
}

async fn parse_image_upload_field(
    field: axum::extract::multipart::Field<'_>,
) -> Result<ImageUpload, Response> {
    let filename = field
        .file_name()
        .map(ToOwned::to_owned)
        .ok_or_else(missing_multipart_field)?;
    let content_type = field.content_type().map(ToOwned::to_owned);
    let bytes = field.bytes().await.map_err(bad_multipart)?.to_vec();
    let mut upload = ImageUpload::new(filename, bytes);
    if let Some(content_type) = content_type {
        upload = upload.with_content_type(content_type);
    }
    Ok(upload)
}

fn parse_u32_field(value: String) -> Result<u32, &'static str> {
    value
        .parse::<u32>()
        .map_err(|_| "invalid numeric multipart field")
}

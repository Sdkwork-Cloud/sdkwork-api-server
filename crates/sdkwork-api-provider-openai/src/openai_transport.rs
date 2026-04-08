use super::*;

impl OpenAiProviderAdapter {
    pub async fn webhooks(&self, api_key: &str, request: &CreateWebhookRequest) -> Result<Value> {
        self.post_json("/v1/webhooks", api_key, request).await
    }

    pub async fn list_webhooks(&self, api_key: &str) -> Result<Value> {
        self.get_json("/v1/webhooks", api_key).await
    }

    pub async fn retrieve_webhook(&self, api_key: &str, webhook_id: &str) -> Result<Value> {
        self.get_json(&format!("/v1/webhooks/{webhook_id}"), api_key)
            .await
    }

    pub async fn update_webhook(
        &self,
        api_key: &str,
        webhook_id: &str,
        request: &UpdateWebhookRequest,
    ) -> Result<Value> {
        self.post_json(&format!("/v1/webhooks/{webhook_id}"), api_key, request)
            .await
    }

    pub async fn delete_webhook(&self, api_key: &str, webhook_id: &str) -> Result<Value> {
        self.delete_json(&format!("/v1/webhooks/{webhook_id}"), api_key)
            .await
    }

    pub(crate) async fn post_json<T: serde::Serialize>(
        &self,
        path: &str,
        api_key: &str,
        request: &T,
    ) -> Result<Value> {
        let response = self
            .authorized_request(reqwest::Method::POST, path, api_key)
            .json(request)
            .send()
            .await?;
        let response = self.require_success(response).await?;

        Ok(response.json::<Value>().await?)
    }

    pub(crate) async fn post_stream<T: serde::Serialize>(
        &self,
        path: &str,
        api_key: &str,
        request: &T,
    ) -> Result<ProviderStreamOutput> {
        let response = self
            .authorized_request(reqwest::Method::POST, path, api_key)
            .json(request)
            .send()
            .await?;
        let response = self.require_success(response).await?;

        Ok(ProviderStreamOutput::from_reqwest_response(response))
    }

    pub(crate) async fn get_json(&self, path: &str, api_key: &str) -> Result<Value> {
        let response = self
            .authorized_request(reqwest::Method::GET, path, api_key)
            .send()
            .await?;
        let response = self.require_success(response).await?;

        Ok(response.json::<Value>().await?)
    }

    pub(crate) async fn get_stream(&self, path: &str, api_key: &str) -> Result<ProviderStreamOutput> {
        let response = self
            .authorized_request(reqwest::Method::GET, path, api_key)
            .send()
            .await?;
        let response = self.require_success(response).await?;

        Ok(ProviderStreamOutput::from_reqwest_response(response))
    }

    pub(crate) async fn delete_json(&self, path: &str, api_key: &str) -> Result<Value> {
        let response = self
            .authorized_request(reqwest::Method::DELETE, path, api_key)
            .send()
            .await?;
        let response = self.require_success(response).await?;

        Ok(response.json::<Value>().await?)
    }

    pub(crate) async fn post_multipart_json(
        &self,
        path: &str,
        api_key: &str,
        form: reqwest::multipart::Form,
    ) -> Result<Value> {
        let response = self
            .authorized_request(reqwest::Method::POST, path, api_key)
            .multipart(form)
            .send()
            .await?;
        let response = self.require_success(response).await?;

        Ok(response.json::<Value>().await?)
    }

    pub(crate) async fn post_empty_json(&self, path: &str, api_key: &str) -> Result<Value> {
        let response = self
            .authorized_request(reqwest::Method::POST, path, api_key)
            .send()
            .await?;
        let response = self.require_success(response).await?;

        Ok(response.json::<Value>().await?)
    }

    pub(crate) async fn require_success(&self, response: reqwest::Response) -> Result<reqwest::Response> {
        if response.status().is_success() {
            return Ok(response);
        }

        let status = response.status();
        let retry_after = retry_after_from_headers(response.headers());
        let body_excerpt = response
            .text()
            .await
            .ok()
            .map(|body| body.chars().take(512).collect::<String>())
            .filter(|body| !body.trim().is_empty());
        Err(ProviderHttpError::new(
            Some(status),
            retry_after.map(|retry_after| retry_after.secs),
            retry_after.map(|retry_after| retry_after.source),
            body_excerpt,
        )
        .into())
    }

    fn authorized_request(
        &self,
        method: reqwest::Method,
        path: &str,
        api_key: &str,
    ) -> RequestBuilder {
        let mut builder = self
            .client
            .request(method, format!("{}{}", self.base_url, path))
            .bearer_auth(api_key);
        for (name, value) in &self.request_headers {
            builder = builder.header(name, value);
        }

        self.apply_openai_compat_headers(path, builder)
    }

    fn apply_openai_compat_headers(&self, path: &str, builder: RequestBuilder) -> RequestBuilder {
        if path.starts_with("/v1/assistants") || path.starts_with("/v1/threads") {
            builder.header("OpenAI-Beta", "assistants=v2")
        } else {
            builder
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RetryAfterHint {
    secs: u64,
    source: ProviderRetryAfterSource,
}

fn retry_after_from_headers(headers: &reqwest::header::HeaderMap) -> Option<RetryAfterHint> {
    headers
        .get(reqwest::header::RETRY_AFTER)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| retry_after_from_value(value.trim()))
}

fn retry_after_from_value(value: &str) -> Option<RetryAfterHint> {
    if let Ok(seconds) = value.parse::<u64>() {
        return Some(RetryAfterHint {
            secs: seconds,
            source: ProviderRetryAfterSource::Seconds,
        });
    }

    let retry_at = httpdate::parse_http_date(value).ok()?;
    let delay = retry_at.duration_since(SystemTime::now()).ok()?;
    Some(RetryAfterHint {
        secs: delay.as_secs().max(1),
        source: ProviderRetryAfterSource::HttpDate,
    })
}

pub(crate) fn multipart_file_part(
    bytes: Vec<u8>,
    filename: &str,
    content_type: Option<&str>,
) -> reqwest::multipart::Part {
    let mut part = reqwest::multipart::Part::bytes(bytes).file_name(filename.to_owned());
    if let Some(content_type) = content_type {
        part = part
            .mime_str(content_type)
            .expect("valid multipart content type");
    }
    part
}

pub(crate) fn add_optional_text_field(
    form: reqwest::multipart::Form,
    name: &str,
    value: Option<&str>,
) -> reqwest::multipart::Form {
    match value {
        Some(value) => form.text(name.to_owned(), value.to_owned()),
        None => form,
    }
}

pub(crate) fn add_optional_number_field(
    form: reqwest::multipart::Form,
    name: &str,
    value: Option<u32>,
) -> reqwest::multipart::Form {
    match value {
        Some(value) => form.text(name.to_owned(), value.to_string()),
        None => form,
    }
}

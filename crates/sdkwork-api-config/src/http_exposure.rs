use super::*;

impl Default for HttpExposureConfig {
    fn default() -> Self {
        Self {
            metrics_bearer_token: DEFAULT_METRICS_BEARER_TOKEN.to_owned(),
            browser_allowed_origins: default_browser_allowed_origins(),
        }
    }
}

impl HttpExposureConfig {
    pub fn from_env() -> Result<Self> {
        Self::from_pairs(std::env::vars())
    }

    pub fn from_pairs<I, K, V>(pairs: I) -> Result<Self>
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        let values = collect_pairs(pairs);
        let mut config = Self::default();
        if let Some(value) = values.get(SDKWORK_METRICS_BEARER_TOKEN) {
            config.metrics_bearer_token = value.clone();
        }
        if let Some(value) = values.get(SDKWORK_BROWSER_ALLOWED_ORIGINS) {
            config.browser_allowed_origins =
                parse_string_list_env(value, SDKWORK_BROWSER_ALLOWED_ORIGINS)?;
        }
        Ok(config)
    }
}


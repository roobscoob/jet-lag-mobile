use futures_util::StreamExt;

use crate::resource::{bundle::ResourceBundle, reference::ResourceReference};

pub struct ResourceFetcher {
    base_url: String,
    client: reqwest::Client,
}

impl ResourceFetcher {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            client: reqwest::Client::new(),
        }
    }

    /// Fetch bundle metadata (the list of resource references)
    pub async fn fetch_bundle(&self, bundle_id: &str) -> Result<ResourceBundle, FetchError> {
        let url = format!("{}/bundles/{}", self.base_url, bundle_id);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| FetchError::Network(e.to_string()))?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(FetchError::NotFound);
        }

        if !response.status().is_success() {
            return Err(FetchError::Network(format!("HTTP {}", response.status())));
        }

        let bundle: ResourceBundle = response
            .json()
            .await
            .map_err(|e| FetchError::InvalidData(e.to_string()))?;

        Ok(bundle)
    }

    /// Fetch the resource data as a streaming byte stream
    pub async fn fetch_resource(
        &self,
        reference: &ResourceReference,
    ) -> Result<impl futures_core::Stream<Item = Result<bytes::Bytes, FetchError>>, FetchError>
    {
        let url = format!("{}/resources/{}", self.base_url, reference.id());

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| FetchError::Network(e.to_string()))?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(FetchError::NotFound);
        }

        if !response.status().is_success() {
            return Err(FetchError::Network(format!("HTTP {}", response.status())));
        }

        Ok(response
            .bytes_stream()
            .map(|result| result.map_err(|e| FetchError::Network(e.to_string()))))
    }
}

#[derive(Debug)]
pub enum FetchError {
    Io(std::io::Error),
    NotFound,
    Network(String),
    InvalidData(String),
}

impl std::fmt::Display for FetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FetchError::Io(e) => write!(f, "IO error: {}", e),
            FetchError::NotFound => write!(f, "Resource not found"),
            FetchError::Network(msg) => write!(f, "Network error: {}", msg),
            FetchError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
        }
    }
}

impl std::error::Error for FetchError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            FetchError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for FetchError {
    fn from(e: std::io::Error) -> Self {
        FetchError::Io(e)
    }
}

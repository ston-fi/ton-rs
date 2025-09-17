use crate::errors::TonError;
use crate::tep::metadata::loader::ipfs_loader::{IpfsLoader, IpfsLoaderError};
use crate::tep::metadata::Metadata;
use crate::tep::metadata::MetadataExternal;
use crate::tep::metadata::MetadataInternal;
use crate::tep::metadata::{MetadataContent, META_URI};
use reqwest::header;
use reqwest::header::HeaderValue;
use reqwest::StatusCode;
use std::num::ParseIntError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MetaLoaderError {
    #[error("Unsupported content layout (Metadata content: {0:?})")]
    ContentLayoutUnsupported(Box<MetadataContent>),

    #[error("Failed to load jetton metadata (URI: {uri}, response status code: {status})")]
    LoadMetadataFailed { uri: String, status: StatusCode },

    #[error("Serde_json Error ({0})")]
    SerdeJsonError(#[from] serde_json::Error),

    #[error("IpfsLoaderError ({0})")]
    IpfsLoaderError(#[from] IpfsLoaderError),

    #[error("Transport error ({0})")]
    TransportError(#[from] reqwest::Error),

    #[error("Internal error ({0})")]
    InternalError(String),

    #[error("Parse int error ({0})")]
    ParseIntError(#[from] ParseIntError),

    #[error(transparent)]
    MetadataParseError(#[from] TonError),
}

pub struct MetaLoader {
    http_client: reqwest::Client,
    ipfs_loader: IpfsLoader,
}

pub struct MetaLoaderBuilder {
    http_client: Option<reqwest::Client>,
    ipfs_loader: Option<IpfsLoader>,
}

impl MetaLoaderBuilder {
    fn new() -> Self {
        MetaLoaderBuilder {
            http_client: None,
            ipfs_loader: None,
        }
    }

    pub fn with_http_client(mut self, http_client: reqwest::Client) -> Self {
        self.http_client = Some(http_client);
        self
    }
    pub fn with_ipfs_loader(mut self, ipfs_loader: IpfsLoader) -> Self {
        self.ipfs_loader = Some(ipfs_loader);
        self
    }

    pub fn build(self) -> Result<MetaLoader, MetaLoaderError> {
        let http_client = self.http_client.unwrap_or_else(|| {
            let mut headers = header::HeaderMap::new();
            headers.insert("user-agent", HeaderValue::from_static("tonlib-rs/1.x"));
            headers.insert("accept", HeaderValue::from_static("*/*"));
            reqwest::Client::builder().default_headers(headers).build().unwrap()
        });

        let ipfs_loader = match self.ipfs_loader {
            Some(ipfs_loader) => ipfs_loader,
            None => IpfsLoader::builder().with_client(http_client.clone()).build()?,
        };

        Ok(MetaLoader {
            http_client,
            ipfs_loader,
        })
    }
}

impl MetaLoader {
    pub fn builder() -> MetaLoaderBuilder { MetaLoaderBuilder::new() }

    pub async fn load_external_meta(&self, uri: &str) -> Result<String, MetaLoaderError> {
        log::trace!("Downloading metadata from {}", uri);
        let meta_str: String = if uri.starts_with("ipfs://") {
            let path: String = uri.chars().skip(7).collect();
            self.ipfs_loader.load_utf8_lossy(path.as_str()).await?
        } else {
            let resp = self.http_client.get(uri).send().await?;
            if resp.status().is_success() {
                resp.text().await?
            } else {
                return Err(MetaLoaderError::LoadMetadataFailed {
                    uri: uri.to_string(),
                    status: resp.status(),
                });
            }
        };

        Ok(meta_str)
    }

    pub async fn load<T: Metadata>(&self, content: &MetadataContent) -> Result<T, MetaLoaderError> {
        match content {
            MetadataContent::External(MetadataExternal { uri }) => {
                let json = self.load_external_meta(&uri.as_str()).await?;
                Ok(T::from_json(&json)?)
            }
            MetadataContent::Internal(MetadataInternal { data: dict }) => {
                let uri = match dict.get(&META_URI) {
                    Some(uri) => uri,
                    None => return Ok(T::from_dict(dict)?),
                };
                let uri_str = uri.as_str();

                let json = match self.load_external_meta(&uri_str).await {
                    Ok(json) => json,
                    Err(err) => {
                        log::warn!(
                            "Failed to load metadata from internal META_URI {uri_str}: {err}, use internal data only"
                        );
                        return Ok(T::from_dict(dict)?);
                    }
                };
                Ok(T::from_data(dict, Some(&json))?)
            }
            content => Err(MetaLoaderError::ContentLayoutUnsupported(Box::new(content.clone()))),
        }
    }
}

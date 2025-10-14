mod builder;
mod ipfs_loader;

pub use ipfs_loader::*;

use crate::errors::{MetaLoaderError, TonResult};
use crate::tep::metadata::meta_loader::builder::Builder;
use crate::tep::metadata::Metadata;
use crate::tep::metadata::MetadataExternal;
use crate::tep::metadata::MetadataInternal;
use crate::tep::metadata::{MetadataContent, META_URI};

pub struct MetaLoader {
    http_loader: reqwest::Client,
    ipfs_loader: IpfsLoader,
}

impl MetaLoader {
    pub fn builder() -> Builder { Builder::new() }

    pub async fn load_external_meta(&self, uri: &str) -> TonResult<String> {
        log::trace!("Downloading metadata from {}", uri);
        let meta_str: String = if uri.starts_with("ipfs://") {
            let path: String = uri.chars().skip(7).collect();
            self.ipfs_loader.load_utf8_lossy(path.as_str()).await?
        } else {
            let resp = self.http_loader.get(uri).send().await?;
            if resp.status().is_success() {
                resp.text().await?
            } else {
                return Err(MetaLoaderError::LoadMetadataFailed {
                    uri: uri.to_string(),
                    status: resp.status(),
                }
                .into());
            }
        };

        Ok(meta_str)
    }

    pub async fn load<T: Metadata>(&self, content: &MetadataContent) -> TonResult<T> {
        match content {
            MetadataContent::External(MetadataExternal { uri }) => {
                let json = self.load_external_meta(&uri.as_str()).await?;
                Ok(T::from_json(&json)?)
            }
            MetadataContent::Internal(MetadataInternal { data: dict }) => {
                let uri = match dict.get(&META_URI) {
                    Some(uri) => uri,
                    None => return T::from_dict(dict),
                };
                let uri_str = uri.as_str();

                let json = match self.load_external_meta(&uri_str).await {
                    Ok(json) => json,
                    Err(err) => {
                        log::warn!(
                            "Failed to load metadata from internal META_URI {uri_str}: {err}, use internal data only"
                        );
                        return T::from_dict(dict);
                    }
                };
                Ok(T::from_data(dict, Some(&json))?)
            }
            content => Err(MetaLoaderError::ContentLayoutUnsupported(Box::new(content.clone())).into()),
        }
    }
}

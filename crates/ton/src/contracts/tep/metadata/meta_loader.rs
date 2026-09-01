//! HTTP and IPFS metadata loading.

pub mod builder;
pub mod ipfs_loader;

use crate::contracts::tep::metadata::meta_loader::builder::Builder;
use crate::contracts::tep::metadata::meta_loader::ipfs_loader::IpfsLoader;
use crate::contracts::tep::metadata::metadata_content::{MetadataContent, MetadataExternal, MetadataInternal};
use crate::contracts::tep::metadata::metadata_fields::META_URI;
use crate::contracts::tep::metadata::traits::Metadata;
use crate::errors::{MetaLoaderError, TonResult};

pub struct MetaLoader {
    http_loader: reqwest::Client,
    ipfs_loader: IpfsLoader,
    semichain_external_metadata_required: bool,
}

impl MetaLoader {
    /// Creates a metadata-loader builder.
    ///
    /// The default IPFS transport uses the best-effort `https://ipfs.io/ipfs` public gateway. Production
    /// applications should configure their own gateway with [`Builder::with_ipfs_base_url`].
    pub fn builder() -> Builder {
        Builder::new()
    }

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

    /// Loads metadata according to its TEP-64 content layout.
    ///
    /// Semi-chain metadata falls back to its on-chain fields when the external document cannot be loaded unless
    /// [`Builder::with_semichain_external_metadata_required`] requires that document.
    ///
    /// # Errors
    ///
    /// Returns an error when the content layout is unsupported or required metadata cannot be loaded or parsed.
    pub async fn load<T: Metadata>(&self, content: &MetadataContent) -> TonResult<T> {
        match content {
            MetadataContent::External(MetadataExternal { uri }) => {
                let json = self.load_external_meta(&uri.as_str()).await?;
                Ok(T::from_json(&json)?)
            },
            MetadataContent::Internal(MetadataInternal { data: dict }) => {
                let uri = match dict.get(&META_URI) {
                    Some(uri) => uri,
                    None => return T::from_dict(dict),
                };
                let uri_str = uri.as_str();

                let json = match self.load_external_meta(&uri_str).await {
                    Ok(json) => json,
                    Err(err) => {
                        if self.semichain_external_metadata_required {
                            return Err(err);
                        }
                        log::warn!(
                            "Failed to load metadata from internal META_URI {uri_str}: {err}, use internal data only"
                        );
                        return T::from_dict(dict);
                    },
                };
                Ok(T::from_data(dict, Some(&json))?)
            },
            content => Err(MetaLoaderError::ContentLayoutUnsupported(Box::new(content.clone())).into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::tep::metadata::metadata_content::MetadataDict;
    use crate::contracts::tep::snake_data::SnakeData;
    use crate::errors::TonError;

    #[derive(Debug, PartialEq, Eq)]
    struct TestMetadata {
        external_loaded: bool,
    }

    impl Metadata for TestMetadata {
        fn from_data(_dict: &MetadataDict, json: Option<&str>) -> Result<Self, TonError> {
            Ok(Self {
                external_loaded: json.is_some(),
            })
        }
    }

    #[tokio::test]
    async fn test_semichain_external_metadata_required() -> anyhow::Result<()> {
        let content = MetadataContent::Internal(MetadataInternal {
            data: MetadataDict::from([(**META_URI, SnakeData::from("unsupported://metadata").into())]),
        });

        let fallback_loader = MetaLoader::builder().build()?;
        let fallback_metadata: TestMetadata = fallback_loader.load(&content).await?;
        assert_eq!(fallback_metadata, TestMetadata { external_loaded: false });

        let required_loader = MetaLoader::builder().with_semichain_external_metadata_required(true).build()?;
        let required_result = required_loader.load::<TestMetadata>(&content).await;
        assert!(matches!(required_result, Err(TonError::TransportError(_))));

        Ok(())
    }
}

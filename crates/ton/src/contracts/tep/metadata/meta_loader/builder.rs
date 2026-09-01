//! Builder for metadata loaders.

use crate::contracts::tep::metadata::meta_loader::MetaLoader;
use crate::contracts::tep::metadata::meta_loader::ipfs_loader::IpfsConnectionType;
use crate::contracts::tep::metadata::meta_loader::ipfs_loader::IpfsLoader;
use crate::errors::TonResult;
use derive_setters::Setters;
use reqwest::header;
use reqwest::header::{HeaderMap, HeaderValue};

/// Configures HTTP and IPFS metadata loading.
///
/// IPFS URIs use the IPFS Foundation's best-effort public gateway by default. Production applications should
/// configure infrastructure they control with [`Builder::with_ipfs_base_url`].
#[derive(Setters)]
#[setters(prefix = "with_", strip_option)]
pub struct Builder {
    /// IPFS transport used for `ipfs://` metadata.
    ipfs_connection_type: IpfsConnectionType,
    /// Base URL of the HTTP gateway or IPFS node.
    ipfs_base_url: String,
    /// HTTP client shared by HTTP and IPFS metadata requests.
    http_client: Option<reqwest::Client>,
    /// Requires semi-chain metadata to include its external document instead of falling back to on-chain fields.
    semichain_external_metadata_required: bool,
}

impl Builder {
    pub(super) fn new() -> Self {
        Self {
            http_client: None,
            ipfs_connection_type: IpfsConnectionType::HttpGateway,
            ipfs_base_url: "https://ipfs.io/ipfs".to_string(),
            semichain_external_metadata_required: false,
        }
    }

    pub fn build(self) -> TonResult<MetaLoader> {
        let http_client = match self.http_client {
            Some(client) => client,
            None => {
                let headers = HeaderMap::from_iter([
                    (header::USER_AGENT, HeaderValue::from_static("tonlib-rs/1.x")),
                    (header::ACCEPT, HeaderValue::from_static("*/*")),
                ]);
                reqwest::Client::builder().default_headers(headers).build()?
            },
        };

        let ipfs_loader = IpfsLoader {
            connection_type: self.ipfs_connection_type,
            base_url: self.ipfs_base_url,
            client: http_client.clone(),
        };

        Ok(MetaLoader {
            http_loader: http_client,
            ipfs_loader,
            semichain_external_metadata_required: self.semichain_external_metadata_required,
        })
    }
}

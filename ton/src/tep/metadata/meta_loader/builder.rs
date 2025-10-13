use crate::errors::TonResult;
use crate::tep::metadata::meta_loader::ipfs_loader::IpfsLoader;
use crate::tep::metadata::{IpfsConnectionType, MetaLoader};
use derive_setters::Setters;
use reqwest::header;
use reqwest::header::{HeaderMap, HeaderValue};

#[derive(Setters)]
#[setters(prefix = "with_", strip_option)]
pub struct Builder {
    ipfs_connection_type: IpfsConnectionType,
    ipfs_base_url: String,
    http_client: Option<reqwest::Client>,
}

impl Builder {
    pub(super) fn new() -> Self {
        Self {
            http_client: None,
            ipfs_connection_type: IpfsConnectionType::HttpGateway,
            ipfs_base_url: "https://cloudflare-ipfs.com/ipfs/".to_string(),
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
            }
        };

        let ipfs_loader = IpfsLoader {
            connection_type: self.ipfs_connection_type,
            base_url: self.ipfs_base_url,
            client: http_client.clone(),
        };

        Ok(MetaLoader {
            http_loader: http_client,
            ipfs_loader,
        })
    }
}

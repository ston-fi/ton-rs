use std::fmt::Debug;

use crate::errors::{MetaLoaderError, TonResult};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum IpfsConnectionType {
    HttpGateway,
    IpfsNode,
}

pub(super) struct IpfsLoader {
    pub(super) connection_type: IpfsConnectionType,
    pub(super) base_url: String,
    pub(super) client: reqwest::Client,
}

impl IpfsLoader {
    pub(super) async fn load(&self, path: &str) -> TonResult<Vec<u8>> {
        let response = match self.connection_type {
            IpfsConnectionType::HttpGateway => {
                let full_url = format!("{}/{}", self.base_url, path);
                self.client.get(full_url).send().await?
            }
            IpfsConnectionType::IpfsNode => {
                let full_url = format!("{}/api/v0/cat?arg={}", self.base_url, path);
                self.client.post(full_url).send().await?
            }
        };
        let status = response.status();
        if status.is_success() {
            let bytes = response.bytes().await?.to_vec();
            Ok(bytes)
        } else {
            const MAX_MESSAGE_SIZE: usize = 200;
            let body = String::from_utf8_lossy(&response.bytes().await?).to_string();
            let msg = if body.len() > MAX_MESSAGE_SIZE {
                format!("{}...", &body[0..MAX_MESSAGE_SIZE - 3])
            } else {
                body.clone()
            };

            Err(MetaLoaderError::IpfsLoadError {
                path: path.to_string(),
                status,
                msg,
            }
            .into())
        }
    }

    pub(super) async fn load_utf8_lossy(&self, path: &str) -> TonResult<String> {
        let bytes = self.load(path).await?;
        let str = String::from_utf8_lossy(&bytes).to_string();
        Ok(str)
    }
}

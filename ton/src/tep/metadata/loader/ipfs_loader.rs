use reqwest::StatusCode;
use std::fmt::Debug;
use thiserror::Error;

use serde::{Deserialize, Serialize};

#[derive(Debug, Error)]
pub enum IpfsLoaderError {
    #[error("Failed to load IPFS object (path: {path}, status: {status}, message: {message})")]
    IpfsLoadObjectFailed {
        path: String,
        status: StatusCode,
        message: String,
    },

    #[error("Transport error: {0}")]
    TransportError(#[from] reqwest::Error),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum IpfsConnectionType {
    HttpGateway,
    IpfsNode,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub struct IpfsLoaderConfig {
    connection_type: IpfsConnectionType,
    base_url: String,
}

impl IpfsLoaderConfig {
    pub fn http_gateway(url: &str) -> IpfsLoaderConfig {
        IpfsLoaderConfig {
            connection_type: IpfsConnectionType::HttpGateway,
            base_url: url.to_string(),
        }
    }

    pub fn ipfs_node(url: &str) -> IpfsLoaderConfig {
        IpfsLoaderConfig {
            connection_type: IpfsConnectionType::IpfsNode,
            base_url: url.to_string(),
        }
    }
}

impl Default for IpfsLoaderConfig {
    fn default() -> Self { Self::http_gateway("https://cloudflare-ipfs.com/ipfs/") }
}

#[derive(Default)]
pub struct IpfsLoaderBuilder {
    ipfs_loader_config: Option<IpfsLoaderConfig>,
    client: Option<reqwest::Client>,
}

impl IpfsLoaderBuilder {
    pub fn with_config(mut self, config: IpfsLoaderConfig) -> Self {
        self.ipfs_loader_config = Some(config);
        self
    }

    pub fn with_client(mut self, client: reqwest::Client) -> Self {
        self.client = Some(client);
        self
    }

    pub fn build(self) -> Result<IpfsLoader, IpfsLoaderError> {
        let config = self.ipfs_loader_config.unwrap_or_default();
        Ok(IpfsLoader {
            connection_type: config.connection_type,
            base_url: config.base_url,
            client: self.client.unwrap_or(reqwest::Client::builder().build()?),
        })
    }
}

#[derive(Clone)]
pub struct IpfsLoader {
    connection_type: IpfsConnectionType,
    base_url: String,
    client: reqwest::Client,
}

impl Default for IpfsLoader {
    fn default() -> Self {
        let cfg = IpfsLoaderConfig::default();
        Self {
            connection_type: cfg.connection_type,
            base_url: cfg.base_url,
            client: Default::default(),
        }
    }
}

impl IpfsLoader {
    pub fn builder() -> IpfsLoaderBuilder { IpfsLoaderBuilder::default() }

    pub async fn load(&self, path: &str) -> Result<Vec<u8>, IpfsLoaderError> {
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
            let message = if body.len() > MAX_MESSAGE_SIZE {
                format!("{}...", &body[0..MAX_MESSAGE_SIZE - 3])
            } else {
                body.clone()
            };

            Err(IpfsLoaderError::IpfsLoadObjectFailed {
                path: path.to_string(),
                status,
                message,
            })
        }
    }

    pub async fn load_utf8_lossy(&self, path: &str) -> Result<String, IpfsLoaderError> {
        let bytes = self.load(path).await?;
        let str = String::from_utf8_lossy(&bytes).to_string();
        Ok(str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static CONFIG_JSON: &str = r#"
    {
      "connection_type": "http_gateway",
      "base_url": "http://example.com/"
    }
    "#;

    #[test]
    fn test_config_deserialization() -> anyhow::Result<()> {
        let config: IpfsLoaderConfig = serde_json::from_str(CONFIG_JSON)?;
        assert_eq!(config.connection_type, IpfsConnectionType::HttpGateway);
        assert_eq!(config.base_url, "http://example.com/");
        Ok(())
    }
}

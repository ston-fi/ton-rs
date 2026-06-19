use crate::cell::TonHash;
use crate::types::*;
use ::serde::de::Error;
use ::serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;

impl Serialize for TonHash {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serde_ton_hash_hex::serialize(self, serializer)
    }
}

impl<'de> Deserialize<'de> for TonHash {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        serde_ton_hash_hex::deserialize(deserializer)
    }
}

impl Serialize for TonAddress {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serde_ton_address_base64_url::serialize(self, serializer)
    }
}

impl<'de> Deserialize<'de> for TonAddress {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        serde_ton_address_base64_url::deserialize(deserializer)
    }
}

// TonHash
pub mod serde_ton_hash_hex {
    use super::*;

    pub fn serialize<S: Serializer>(hash: &TonHash, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(hash.to_hex().as_str())
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<TonHash, D::Error> {
        TonHash::from_str(&String::deserialize(deserializer)?).map_err(Error::custom)
    }
}

pub mod serde_ton_hash_base64 {
    use super::*;

    pub fn serialize<S: Serializer>(hash: &TonHash, se: S) -> Result<S::Ok, S::Error> {
        se.serialize_str(hash.to_base64().as_str())
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<TonHash, D::Error> {
        TonHash::from_str(&String::deserialize(deserializer)?).map_err(Error::custom)
    }
}

// Option<TonHash>
pub mod serde_ton_hash_hex_opt {
    use super::*;

    pub fn serialize<S: Serializer>(hash: &Option<TonHash>, serializer: S) -> Result<S::Ok, S::Error> {
        match hash {
            Some(h) => serde_ton_hash_hex::serialize(h, serializer),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<TonHash>, D::Error> {
        let opt_val = Option::<serde_json::Value>::deserialize(deserializer)?;
        opt_val.map(serde_ton_hash_hex::deserialize).transpose().map_err(Error::custom)
    }
}

pub mod serde_ton_hash_base64_opt {
    use super::*;

    pub fn serialize<S: Serializer>(hash: &Option<TonHash>, serializer: S) -> Result<S::Ok, S::Error> {
        match hash {
            Some(h) => serde_ton_hash_base64::serialize(h, serializer),
            None => serializer.serialize_none(),
        }
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<TonHash>, D::Error> {
        let opt_val = Option::<serde_json::Value>::deserialize(deserializer)?;
        opt_val.map(serde_ton_hash_base64::deserialize).transpose().map_err(Error::custom)
    }
}

// Vec<TonHash>
pub mod serde_ton_hash_vec_base64 {
    pub use super::*;

    pub fn serialize<S: Serializer>(data: &[TonHash], serializer: S) -> Result<S::Ok, S::Error> {
        let base64_strings: Vec<String> = data.iter().map(|h| h.to_base64()).collect();
        base64_strings.serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<TonHash>, D::Error> {
        let base64_vec: Vec<String> = Vec::deserialize(deserializer)?;
        base64_vec.into_iter().map(|s| TonHash::from_str(&s).map_err(Error::custom)).collect()
    }
}

// TonAddress
pub mod serde_ton_address_hex {
    pub use super::*;

    pub fn serialize<S: Serializer>(address: &TonAddress, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(address.to_hex().as_str())
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<TonAddress, D::Error> {
        TonAddress::from_str(&String::deserialize(deserializer)?).map_err(Error::custom)
    }
}

pub mod serde_ton_address_base64_url {
    pub use super::*;

    pub fn serialize<S: Serializer>(address: &TonAddress, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&address.to_base64(true, true, true))
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<TonAddress, D::Error> {
        TonAddress::from_str(&String::deserialize(deserializer)?).map_err(Error::custom)
    }
}

pub mod serde_ton_address_base64_url_testnet {
    pub use super::*;

    pub fn serialize<S: Serializer>(address: &TonAddress, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&address.to_base64(false, true, true))
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<TonAddress, D::Error> {
        TonAddress::from_str(&String::deserialize(deserializer)?).map_err(Error::custom)
    }
}

// Option<TonAddress>
pub mod serde_ton_address_hex_opt {
    pub use super::*;

    pub fn serialize<S: Serializer>(address: &Option<TonAddress>, serializer: S) -> Result<S::Ok, S::Error> {
        match address {
            Some(addr) => serde_ton_address_hex::serialize(addr, serializer),
            None => serializer.serialize_none(),
        }
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<TonAddress>, D::Error> {
        let opt_val = Option::<serde_json::Value>::deserialize(deserializer)?;
        opt_val.map(serde_ton_address_hex::deserialize).transpose().map_err(Error::custom)
    }
}

pub mod serde_ton_address_base64_url_opt {
    pub use super::*;

    pub fn serialize<S: Serializer>(address: &Option<TonAddress>, serializer: S) -> Result<S::Ok, S::Error> {
        match address {
            Some(addr) => serde_ton_address_base64_url::serialize(addr, serializer),
            None => serializer.serialize_none(),
        }
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<TonAddress>, D::Error> {
        let opt_val = Option::<serde_json::Value>::deserialize(deserializer)?;
        opt_val.map(serde_ton_address_base64_url::deserialize).transpose().map_err(Error::custom)
    }
}

pub mod serde_ton_address_base64_url_testnet_opt {
    pub use super::*;

    pub fn serialize<S: Serializer>(address: &Option<TonAddress>, serializer: S) -> Result<S::Ok, S::Error> {
        match address {
            Some(addr) => serde_ton_address_base64_url_testnet::serialize(addr, serializer),
            None => serializer.serialize_none(),
        }
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<TonAddress>, D::Error> {
        let opt_val = Option::<serde_json::Value>::deserialize(deserializer)?;
        opt_val.map(serde_ton_address_base64_url_testnet::deserialize).transpose().map_err(Error::custom)
    }
}

// TxLTHash
pub mod serde_tx_lt_hash_json {
    use super::*;

    pub fn serialize<S: Serializer>(data: &TxLTHash, serializer: S) -> Result<S::Ok, S::Error> {
        let json_val = serde_json::json!({
            "lt": data.lt.to_string(),
            "hash": data.hash.to_base64(),
        });
        json_val.serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<TxLTHash, D::Error> {
        let json_val: serde_json::Value = Deserialize::deserialize(deserializer)?;
        let lt = json_val
            .get("lt")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::custom("Missing or invalid 'lt' field"))?
            .parse::<i64>()
            .map_err(Error::custom)?;
        let hash = json_val
            .get("hash")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::custom("Missing or invalid 'hash' field"))?;
        let hash = TonHash::from_str(hash).map_err(Error::custom)?;
        Ok(TxLTHash { lt, hash })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde_ton_hash() -> anyhow::Result<()> {
        use serde_json::json;

        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct TestStruct {
            #[serde(with = "serde_ton_hash_base64")]
            hash: TonHash,
            #[serde(with = "serde_ton_hash_vec_base64")]
            hash_vec: Vec<TonHash>,
            #[serde(with = "serde_ton_hash_hex_opt")]
            hash_opt: Option<TonHash>,
        }

        let val = TestStruct {
            hash: TonHash::from_slice(&[1u8; 32])?,
            hash_vec: vec![TonHash::from_slice(&[2u8; 32])?, TonHash::from_slice(&[3u8; 32])?],
            hash_opt: Some(TonHash::from_slice(&[3u8; 32])?),
        };
        let val_json_str = serde_json::to_string(&val)?;
        let val_json = serde_json::from_str::<serde_json::Value>(&val_json_str)?;

        let expected_json = json!({
            "hash": "AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE=",
            "hash_vec": [
                "AgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgI=",
                "AwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwM="
            ],
            "hash_opt": "0303030303030303030303030303030303030303030303030303030303030303",
        });
        assert_eq!(val_json, expected_json);
        let parsed_val = serde_json::from_str::<TestStruct>(&val_json.to_string())?;
        assert_eq!(parsed_val, val);
        Ok(())
    }

    #[test]
    fn test_default_serde_ton_address_ton_hash() -> anyhow::Result<()> {
        use serde_json::json;

        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct TestStruct {
            address: TonAddress,
            hash: TonHash,
            hash_opt: Option<TonHash>,
        }

        let address = TonAddress::from_str("EQCGScrZe1xbyWqWDvdI6mzP-GAcAWFv6ZXuaJOuSqemxku4")?;
        let hash = TonHash::from_str("16befdc4512ca3ffaa2919e1f0d7635588edcb9fa7d3990fe83e89275c291cc7")?;

        let val = TestStruct {
            address: address.clone(),
            hash: hash.clone(),
            hash_opt: Some(hash.clone()),
        };

        let val_json_str = serde_json::to_string(&val)?;
        let val_json = serde_json::from_str::<serde_json::Value>(&val_json_str)?;
        let expected_json = json!({
            "address": "EQCGScrZe1xbyWqWDvdI6mzP-GAcAWFv6ZXuaJOuSqemxku4",
            "hash": "16befdc4512ca3ffaa2919e1f0d7635588edcb9fa7d3990fe83e89275c291cc7",
            "hash_opt": "16befdc4512ca3ffaa2919e1f0d7635588edcb9fa7d3990fe83e89275c291cc7",
        });
        assert_eq!(val_json, expected_json);

        let parsed_val = serde_json::from_str::<TestStruct>(&val_json.to_string())?;
        assert_eq!(parsed_val, val);

        let val_none = TestStruct {
            address,
            hash,
            hash_opt: None,
        };
        let val_none_json = serde_json::from_str::<serde_json::Value>(&serde_json::to_string(&val_none)?)?;
        assert_eq!(
            val_none_json,
            json!({
                "address": "EQCGScrZe1xbyWqWDvdI6mzP-GAcAWFv6ZXuaJOuSqemxku4",
                "hash": "16befdc4512ca3ffaa2919e1f0d7635588edcb9fa7d3990fe83e89275c291cc7",
                "hash_opt": null,
            })
        );
        let parsed_none = serde_json::from_str::<TestStruct>(&val_none_json.to_string())?;
        assert_eq!(parsed_none, val_none);
        Ok(())
    }
}

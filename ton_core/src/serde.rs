use crate::cell::TonHash;
use crate::types::*;
use ::serde::de::Error;
use ::serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;

// TonHash
pub mod serde_ton_hash_base64 {
    use super::*;

    pub fn serialize<S: Serializer>(hash: &TonHash, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(hash.to_base64().as_str())
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<TonHash, D::Error> {
        TonHash::from_str(&String::deserialize(deserializer)?).map_err(Error::custom)
    }
}

pub mod serde_ton_hash_hex {
    use super::*;

    pub fn serialize<S: Serializer>(hash: &TonHash, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(hash.to_hex().as_str())
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<TonHash, D::Error> {
        TonHash::from_str(&String::deserialize(deserializer)?).map_err(Error::custom)
    }
}

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
        }

        let val = TestStruct {
            hash: TonHash::from_slice(&[1u8; 32])?,
            hash_vec: vec![TonHash::from_slice(&[2u8; 32])?, TonHash::from_slice(&[3u8; 32])?],
        };
        let val_json = serde_json::to_string(&val)?;
        let expected = json!({
            "hash": "AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE=",
            "hash_vec": [
                "AgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgI=",
                "AwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwM="
            ]
        })
        .to_string();
        assert_eq!(val_json, expected);
        Ok(())
    }
}

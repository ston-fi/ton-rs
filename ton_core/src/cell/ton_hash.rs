use crate::bail_ton_core_data;
use crate::cell::TonCellNum;
use crate::errors::TonCoreError;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use std::hash::Hash;

#[derive(Clone, PartialEq, Hash, Eq, Ord, PartialOrd)]
pub struct TonHash(TonHashData);

#[derive(Clone)]
enum TonHashData {
    Slice([u8; 32]),
    Vec(Vec<u8>),
}

impl TonHash {
    pub const BYTES_LEN: usize = 32;
    pub const BITS_LEN: usize = 256;
    pub const ZERO: TonHash = TonHash::from_slice_sized(&[0u8; 32]);

    pub const fn from_slice_sized(data: &[u8; 32]) -> Self { Self(TonHashData::Slice(*data)) }

    pub fn from_slice(data: &[u8]) -> Result<Self, TonCoreError> {
        check_bytes_len(data)?;
        Ok(Self::from_slice_sized(data[..32].try_into().unwrap()))
    }

    pub fn from_vec(data: Vec<u8>) -> Result<Self, TonCoreError> {
        check_bytes_len(&data)?;
        Ok(Self(TonHashData::Vec(data)))
    }

    pub fn from_num<T: TonCellNum>(num: &T) -> Result<Self, TonCoreError> {
        if T::IS_PRIMITIVE {
            return Err(TonCoreError::data("TonHash", "Can't create from primitive type (not enough bytes)"));
        }
        Self::from_slice(&num.tcn_to_bytes())
    }

    pub fn as_slice(&self) -> &[u8] { self.0.as_slice() }

    pub fn as_slice_sized(&self) -> &[u8; 32] {
        match &self.0 {
            TonHashData::Slice(data) => data,
            TonHashData::Vec(data) => data.as_slice().try_into().unwrap(),
        }
    }

    pub fn as_slice_sized_mut(&mut self) -> &mut [u8; 32] {
        match &mut self.0 {
            TonHashData::Slice(data) => data,
            TonHashData::Vec(data) => data.as_mut_slice().try_into().unwrap(),
        }
    }

    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        match &mut self.0 {
            TonHashData::Slice(data) => data,
            TonHashData::Vec(data) => data.as_mut_slice(),
        }
    }

    pub fn to_vec(&self) -> Vec<u8> { self.as_slice().to_vec() }
    pub fn to_hex(&self) -> String { hex::encode(self.as_slice()) }
    pub fn to_base64(&self) -> String { BASE64_STANDARD.encode(self.as_slice()) }

    pub fn into_vec(self) -> Vec<u8> {
        match self.0 {
            TonHashData::Slice(data) => data.to_vec(),
            TonHashData::Vec(data) => data,
        }
    }
}

impl Default for TonHash {
    fn default() -> Self { TonHash::ZERO }
}

impl TonHashData {
    fn as_slice(&self) -> &[u8] {
        match self {
            TonHashData::Slice(data) => data.as_slice(),
            TonHashData::Vec(data) => data.as_slice(),
        }
    }
}

fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<TonHash, TonCoreError> {
    let bytes = hex::decode(hex)?;
    check_bytes_len(&bytes)?;
    Ok(TonHash(TonHashData::Vec(bytes)))
}

fn from_base64<T: AsRef<[u8]>>(base64: T) -> Result<TonHash, TonCoreError> {
    TonHash::from_vec(BASE64_STANDARD.decode(base64)?)
}

fn check_bytes_len(bytes: &[u8]) -> Result<(), TonCoreError> {
    if bytes.len() != TonHash::BYTES_LEN {
        bail_ton_core_data!("expected bytes_len {}, but got {}", TonHash::BYTES_LEN, bytes.len());
    }
    Ok(())
}

#[rustfmt::skip]
mod traits_impl {
    use std::fmt::{Debug, Display, Formatter, UpperHex};
    use std::fmt::Result as FmtResult;
    use std::hash::Hash;
    use std::str::FromStr;
    use crate::cell::ton_hash::{from_base64, from_hex, TonHash, TonHashData};
    use crate::errors::TonCoreError;


    impl From<[u8; 32]> for TonHash { fn from(data: [u8; 32]) -> Self { Self(TonHashData::Slice(data)) } }
    impl From<&[u8; 32]> for TonHash { fn from(data: &[u8; 32]) -> Self { Self(TonHashData::Slice(*data)) } }
    impl FromStr for TonHash {
        type Err = TonCoreError;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            if s.len() == 64 {
                return from_hex(s);
            }
            from_base64(s)
        }
    }

    impl AsRef<[u8]> for TonHash { fn as_ref(&self) -> &[u8] { self.as_slice() } }
    impl UpperHex for TonHash { fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult { write!(f, "{}", self.to_hex().to_uppercase()) } }
    impl Display for TonHash { fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult { write!(f, "{self:X}") } }
    impl Debug for TonHash { fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult { write!(f, "TonHash[{self:X}]") } }
    
    // Must implement it manually, because we don't distinguish between Vec and Slice
    impl Eq for TonHashData {}
    impl PartialEq for TonHashData { fn eq(&self, other: &Self) -> bool { self.as_slice() == other.as_slice() } }
    impl Hash for TonHashData { fn hash<H: std::hash::Hasher>(&self, state: &mut H) { state.write(self.as_slice()); } }
    impl Ord for TonHashData {fn cmp(&self, other: &Self) -> std::cmp::Ordering {self.as_slice().cmp(other.as_slice()) } }
    impl PartialOrd for TonHashData {fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> { Some(self.cmp(other)) } }
}

#[cfg(feature = "serde")]
mod serde {
    use crate::cell::TonHash;
    use serde::de::Error;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::str::FromStr;

    pub mod serde_ton_hash_base64 {
        pub use super::*;

        pub fn serialize<S: Serializer>(hash: &TonHash, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.serialize_str(hash.to_base64().as_str())
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
}
#[cfg(feature = "serde")]
pub use serde::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::str::FromStr;
    use tokio_test::assert_err;

    #[test]
    fn test_ton_hash_display() -> anyhow::Result<()> {
        let data = [255u8; 32];
        let hash = TonHash::from(data);
        assert_eq!(format!("{hash}"), "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF");
        assert_eq!(format!("{hash:X}"), "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF");
        Ok(())
    }

    #[test]
    fn test_ton_hash_from() -> anyhow::Result<()> {
        let data = [1u8; 32];
        let hash = TonHash::from(data);
        assert_eq!(hash.as_slice(), &data);
        Ok(())
    }

    #[test]
    fn test_ton_hash_slice() -> anyhow::Result<()> {
        let data = [1u8; 32];
        let hash = TonHash::from_slice(&data)?;
        assert_eq!(hash.as_slice(), &data);

        let wrong_data = [1u8; 31];
        assert_err!(TonHash::from_slice(&wrong_data));
        Ok(())
    }

    #[test]
    fn test_ton_hash_from_slice_sized() -> anyhow::Result<()> {
        let data = [1u8; 32];
        let hash = TonHash::from_slice_sized(&data);
        assert_eq!(hash.as_slice(), &data);
        Ok(())
    }

    #[test]
    fn test_ton_hash_from_vec() -> anyhow::Result<()> {
        let data = vec![1u8; 32];
        let hash = TonHash::from_vec(data.clone())?;
        assert_eq!(hash.as_slice(), &data);
        Ok(())
    }

    #[test]
    fn test_ton_hash_from_hex() -> anyhow::Result<()> {
        let data = [255u8; 32];
        let hash = TonHash::from_str("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF")?;
        assert_eq!(hash.as_slice(), &data);
        Ok(())
    }

    #[test]
    fn test_ton_hash_from_base64() -> anyhow::Result<()> {
        let data = [
            159, 115, 38, 150, 26, 81, 188, 250, 200, 211, 46, 142, 240, 183, 144, 7, 187, 83, 144, 183, 68, 163, 90,
            117, 106, 189, 241, 66, 113, 59, 99, 240,
        ];
        let base64_str = "n3MmlhpRvPrI0y6O8LeQB7tTkLdEo1p1ar3xQnE7Y/A=";
        let hash = TonHash::from_str(base64_str)?;
        assert_eq!(hash.as_slice(), &data);
        Ok(())
    }

    #[test]
    fn test_ton_hash_data_hash_eq_impl() -> anyhow::Result<()> {
        let data1 = [1u8; 32];
        let data2 = vec![1u8; 32];
        let hash1 = TonHash::from_slice_sized(&data1);
        let hash2 = TonHash::from_vec(data2)?;
        assert_eq!(hash1, hash2);
        let storage = HashSet::from([hash1, hash2]);
        assert_eq!(storage.len(), 1);
        Ok(())
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serde_ton_hash() -> anyhow::Result<()> {
        use ::serde::{Deserialize, Serialize};
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

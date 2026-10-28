use crate::bail_ton_core_data;
use crate::errors::TonCoreError;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use fastnum::{I512, U256};

#[derive(Clone, Copy, PartialEq, Hash, Eq, Ord, PartialOrd)]
pub struct TonHash([u8; 32]);

impl TonHash {
    pub const BYTES_LEN: usize = 32;
    pub const BITS_LEN: usize = 256;
    pub const ZERO: TonHash = TonHash::from_slice_sized(&[0u8; 32]);

    pub const fn from_slice_sized(data: &[u8; 32]) -> Self {
        Self(*data)
    }

    pub fn from_slice(data: &[u8]) -> Result<Self, TonCoreError> {
        let data = <[u8; 32]>::try_from(data).map_err(|_| bytes_len_error(data.len()))?;
        Ok(Self(data))
    }

    pub fn from_vec(data: Vec<u8>) -> Result<Self, TonCoreError> {
        let bytes_len = data.len();
        let data = <[u8; 32]>::try_from(data).map_err(|_| bytes_len_error(bytes_len))?;
        Ok(Self(data))
    }

    pub fn from_u256(num: &U256) -> Result<Self, TonCoreError> {
        let slice = &mut [0u8; 32];
        let num_bytes = num.to_radix_le(256);
        let offset = 32 - num_bytes.len();
        slice[offset..].copy_from_slice(&num_bytes);
        Ok(Self::from_slice_sized(slice))
    }

    pub fn from_i512(num: &I512) -> Result<Self, TonCoreError> {
        let num_bytes = num.to_radix_le(256);
        if num_bytes.len() > 32 {
            bail_ton_core_data!("I512 value is too large to fit into TonHash");
        }
        TonHash::from_u256(&U256::from_radix_le(&num_bytes, 256).unwrap())
    }

    pub fn to_u256(&self) -> U256 {
        // unwrap is save: TonHash always has 32 bytes
        U256::from_radix_le(self.as_slice(), 256).unwrap()
    }

    pub fn to_i512(&self) -> I512 {
        // unwrap is save: TonHash always has 32 bytes
        I512::from_radix_le(self.as_slice(), 256).unwrap()
    }

    pub fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }

    pub fn as_slice_sized(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn as_slice_sized_mut(&mut self) -> &mut [u8; 32] {
        &mut self.0
    }

    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        self.0.as_mut_slice()
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.as_slice().to_vec()
    }
    pub fn to_hex(&self) -> String {
        hex::encode(self.as_slice())
    }
    pub fn to_base64(&self) -> String {
        BASE64_STANDARD.encode(self.as_slice())
    }

    pub fn into_vec(self) -> Vec<u8> {
        self.0.to_vec()
    }

    pub fn rewrite_first_bits<T: Into<u128>>(&mut self, value: T, bits_len: usize) -> Result<(), TonCoreError> {
        if bits_len == 0 {
            return Ok(());
        }
        if bits_len > 128 {
            return Err(TonCoreError::data("TonHash", "bits_len must be in range 0..=128"));
        }

        let value_u128 = value.into();
        let shifted = value_u128 << (128 - bits_len);
        let value_bytes = shifted.to_be_bytes();

        if !crate::bits_utils::BitsUtils::rewrite(&value_bytes, 0, self.as_slice_mut(), 0, bits_len) {
            bail_ton_core_data!("Failed to rewrite first {} bits of TonHash", bits_len);
        }
        Ok(())
    }
}

fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<TonHash, TonCoreError> {
    let bytes = hex::decode(hex)?;
    TonHash::from_vec(bytes)
}

fn from_base64<T: AsRef<[u8]>>(base64: T) -> Result<TonHash, TonCoreError> {
    TonHash::from_vec(BASE64_STANDARD.decode(base64)?)
}

fn bytes_len_error(bytes_len: usize) -> TonCoreError {
    TonCoreError::data(module_path!(), format!("expected bytes_len {}, but got {bytes_len}", TonHash::BYTES_LEN))
}

#[rustfmt::skip]
mod traits_impl {
    use super::*;
    use std::fmt::{Debug, Display, Formatter, UpperHex};
    use std::fmt::Result as FmtResult;
    use std::str::FromStr;
    use crate::bail_ton_core_data;
    use crate::errors::TonCoreError;

    impl Default for TonHash { fn default() -> Self { TonHash::ZERO } }
    impl From<[u8; 32]> for TonHash { fn from(data: [u8; 32]) -> Self { Self(data) } }
    impl From<&[u8; 32]> for TonHash { fn from(data: &[u8; 32]) -> Self { Self(*data) } }
    impl FromStr for TonHash {
        type Err = TonCoreError;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            if s.len() == 64 {
                return from_hex(s);
            }
            if s.len() == 66 && (s.starts_with("0x") || s.starts_with("0X")) {
                return from_hex(&s[2..]);
            }
            match from_base64(s) {
                Ok(hash) => Ok(hash),
                Err(_) => bail_ton_core_data!("Parsing TonHash failed for str: {s}"),
            }
        }
    }
    impl TryFrom<String> for TonHash {
        type Error = TonCoreError;
        fn try_from(value: String) -> Result<Self, Self::Error> { TonHash::from_str(&value) }
    }
    impl TryFrom<&str> for TonHash {
        type Error = TonCoreError;
        fn try_from(value: &str) -> Result<Self, Self::Error> { TonHash::from_str(value) }
    }

    impl AsRef<[u8]> for TonHash { fn as_ref(&self) -> &[u8] { self.as_slice() } }
    impl UpperHex for TonHash { fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult { write!(f, "{}", self.to_hex().to_uppercase()) } }
    impl Display for TonHash { fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult { write!(f, "{self:X}") } }
    impl Debug for TonHash { fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult { write!(f, "TonHash[{self:X}]") } }
}

#[cfg(test)]
#[path = "../tests/cell/test_ton_hash.rs"]
mod tests;

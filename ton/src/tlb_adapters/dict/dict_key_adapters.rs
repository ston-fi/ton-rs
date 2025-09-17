use crate::bail_ton;
use crate::errors::TonError;
use num_bigint::BigUint;
use ton_lib_core::cell::TonCell;
use ton_lib_core::cell::TonHash;
use ton_lib_core::traits::tlb::TLB;
use ton_lib_core::types::tlb_core::MsgAddressInt;
use ton_lib_core::types::TonAddress;

pub trait DictKeyAdapter<K> {
    fn make_key(src_key: &K) -> Result<BigUint, TonError>;
    fn extract_key(dict_key: &BigUint) -> Result<K, TonError>;
}

pub struct DictKeyAdapterTonHash; // properly tested in LibsDict & account
pub struct DictKeyAdapterInto;
pub struct DictKeyAdapterAddress;
pub struct DictKeyAdapterString; // TODO is not covered by tests

impl DictKeyAdapter<TonHash> for DictKeyAdapterTonHash {
    fn make_key(src_key: &TonHash) -> Result<BigUint, TonError> { Ok(BigUint::from_bytes_be(src_key.as_slice())) }

    fn extract_key(dict_key: &BigUint) -> Result<TonHash, TonError> {
        let mut hash_bytes = vec![0; TonHash::BYTES_LEN];
        let key_bytes = dict_key.to_bytes_be();
        if key_bytes.len() > TonHash::BYTES_LEN {
            let err_str = format!(
                "dict key is too long: expected={}, given={}, key={}",
                TonHash::BYTES_LEN,
                key_bytes.len(),
                dict_key
            );
            return Err(TonError::Custom(err_str));
        }
        let offset = TonHash::BYTES_LEN - key_bytes.len();
        hash_bytes.as_mut_slice()[offset..].copy_from_slice(key_bytes.as_slice());
        Ok(TonHash::from_slice(&hash_bytes)?)
    }
}

impl DictKeyAdapter<MsgAddressInt> for DictKeyAdapterAddress {
    fn make_key(src_key: &MsgAddressInt) -> Result<BigUint, TonError> {
        let cell = src_key.to_cell()?;
        Ok(BigUint::from_bytes_le(&cell.data))
    }

    fn extract_key(dict_key: &BigUint) -> Result<MsgAddressInt, TonError> {
        let mut builder = TonCell::builder();
        builder.write_num(dict_key, 267)?;
        Ok(MsgAddressInt::from_cell(&builder.build()?)?)
    }
}

impl DictKeyAdapter<TonAddress> for DictKeyAdapterAddress {
    fn make_key(src_key: &TonAddress) -> Result<BigUint, TonError> {
        let cell = src_key.to_cell()?;
        Ok(BigUint::from_bytes_le(&cell.data))
    }

    fn extract_key(dict_key: &BigUint) -> Result<TonAddress, TonError> {
        let mut builder = TonCell::builder();
        builder.write_num(dict_key, 267)?;
        Ok(TonAddress::from_cell(&builder.build()?)?)
    }
}

impl<T: Clone + Into<BigUint> + TryFrom<BigUint>> DictKeyAdapter<T> for DictKeyAdapterInto {
    fn make_key(src_key: &T) -> Result<BigUint, TonError> { Ok(src_key.clone().into()) }

    fn extract_key(dict_key: &BigUint) -> Result<T, TonError> {
        match T::try_from(dict_key.clone()) {
            Ok(key) => Ok(key),
            Err(_) => bail_ton!("fail to extract dict key"),
        }
    }
}

impl DictKeyAdapter<String> for DictKeyAdapterString {
    fn make_key(src_key: &String) -> Result<BigUint, TonError> {
        let bytes = src_key.as_bytes();
        Ok(BigUint::from_bytes_le(bytes))
    }

    fn extract_key(dict_key: &BigUint) -> Result<String, TonError> {
        let bytes = dict_key.to_bytes_le();
        Ok(String::from_utf8(bytes)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_dict_key_adapter_ton_hash() -> anyhow::Result<()> {
        let dict_key = DictKeyAdapterTonHash::make_key(&TonHash::ZERO)?;
        assert_eq!(dict_key, 0u32.into());
        assert_eq!(DictKeyAdapterTonHash::extract_key(&dict_key)?, TonHash::ZERO);

        let dict_key = DictKeyAdapterTonHash::make_key(&TonHash::from([0b1010_1010; 32]))?;
        assert_eq!(
            dict_key,
            BigUint::from_str("77194726158210796949047323339125271902179989777093709359638389338608753093290")?
        );
        assert_eq!(DictKeyAdapterTonHash::extract_key(&dict_key)?, TonHash::from([0b1010_1010; 32]));
        Ok(())
    }
}

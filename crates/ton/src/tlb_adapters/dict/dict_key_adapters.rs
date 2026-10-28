use crate::bail_ton;
use crate::errors::TonError;
use num_bigint::{BigInt, BigUint, Sign};
use num_traits::One;
use std::marker::PhantomData;
use ton_core::cell::TonCell;
use ton_core::cell::TonHash;
use ton_core::traits::tlb::TLB;
use ton_core::types::TonAddress;
use ton_core::types::tlb_core::MsgAddressInt;

pub trait DictKeyAdapter {
    type KeyType;
    fn make_key(src_key: &Self::KeyType) -> Result<BigUint, TonError>;
    fn extract_key(dict_key: &BigUint) -> Result<Self::KeyType, TonError>;
}

pub struct DictKeyAdapterTonHash; // properly tested in LibsDict & account_types
pub struct DictKeyAdapterUint<T>(PhantomData<T>);
/// Signed dictionary keys encoded as `KEY_BITS_LEN`-bit two's complement.
/// Conversion rejects zero width and keys outside the representable range.
pub struct DictKeyAdapterInt<const KEY_BITS_LEN: usize, T>(PhantomData<T>);
pub struct DictKeyAdapterMsgAddress;
pub struct DictKeyAdapterTonAddress;
pub struct DictKeyAdapterString; // TODO is not covered by tests

impl DictKeyAdapter for DictKeyAdapterTonHash {
    type KeyType = TonHash;
    fn make_key(src_key: &TonHash) -> Result<BigUint, TonError> {
        Ok(BigUint::from_bytes_be(src_key.as_slice()))
    }

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

impl DictKeyAdapter for DictKeyAdapterMsgAddress {
    type KeyType = MsgAddressInt;
    fn make_key(src_key: &MsgAddressInt) -> Result<BigUint, TonError> {
        let cell = src_key.to_cell()?;
        Ok(cell.parser().read_num(cell.data_len_bits())?)
    }

    fn extract_key(dict_key: &BigUint) -> Result<MsgAddressInt, TonError> {
        let mut builder = TonCell::builder();
        builder.write_num(dict_key, 267)?;
        Ok(MsgAddressInt::from_cell(&builder.build()?)?)
    }
}

impl DictKeyAdapter for DictKeyAdapterTonAddress {
    type KeyType = TonAddress;
    fn make_key(src_key: &TonAddress) -> Result<BigUint, TonError> {
        let cell = src_key.to_cell()?;
        Ok(cell.parser().read_num(cell.data_len_bits())?)
    }

    fn extract_key(dict_key: &BigUint) -> Result<TonAddress, TonError> {
        let mut builder = TonCell::builder();
        builder.write_num(dict_key, 267)?;
        Ok(TonAddress::from_cell(&builder.build()?)?)
    }
}

impl<T: Clone + Into<BigUint> + TryFrom<BigUint>> DictKeyAdapter for DictKeyAdapterUint<T> {
    type KeyType = T;
    fn make_key(src_key: &Self::KeyType) -> Result<BigUint, TonError> {
        Ok(src_key.clone().into())
    }

    fn extract_key(dict_key: &BigUint) -> Result<Self::KeyType, TonError> {
        match T::try_from(dict_key.clone()) {
            Ok(key) => Ok(key),
            Err(_) => bail_ton!("fail to extract dict key"),
        }
    }
}

impl<const KEY_BITS_LEN: usize, T> DictKeyAdapter for DictKeyAdapterInt<KEY_BITS_LEN, T>
where
    T: Clone + Into<BigInt> + TryFrom<BigInt>,
{
    type KeyType = T;

    fn make_key(src_key: &Self::KeyType) -> Result<BigUint, TonError> {
        if KEY_BITS_LEN == 0 {
            bail_ton!("signed dictionary key width must be nonzero");
        }
        let big_int: BigInt = src_key.clone().into();
        let limit = BigInt::one() << (KEY_BITS_LEN - 1);
        if big_int < -&limit || big_int >= limit {
            bail_ton!("signed dictionary key {big_int} exceeds {KEY_BITS_LEN} bits");
        }

        let (sign, magnitude) = big_int.into_parts();
        if sign == Sign::Minus { Ok((BigUint::one() << KEY_BITS_LEN) - magnitude) } else { Ok(magnitude) }
    }

    fn extract_key(dict_key: &BigUint) -> Result<Self::KeyType, TonError> {
        if KEY_BITS_LEN == 0 {
            bail_ton!("signed dictionary key width must be nonzero");
        }
        if dict_key.bits() > KEY_BITS_LEN as u64 {
            bail_ton!("dictionary key {dict_key} exceeds {KEY_BITS_LEN} bits");
        }
        let sign_bit = BigUint::one() << (KEY_BITS_LEN - 1);
        let big_int = if dict_key >= &sign_bit {
            // interpret as negative: x - 2^bits
            BigInt::from(dict_key.clone()) - (BigInt::one() << KEY_BITS_LEN)
        } else {
            BigInt::from(dict_key.clone())
        };
        match T::try_from(big_int.clone()) {
            Ok(key) => Ok(key),
            Err(_) => bail_ton!("fail to extract dict key from {big_int} ({KEY_BITS_LEN} bits)"),
        }
    }
}

impl DictKeyAdapter for DictKeyAdapterString {
    type KeyType = String;
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
#[path = "../../tests/tlb_adapters/dict/test_dict_key_adapters.rs"]
mod tests;

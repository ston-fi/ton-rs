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
pub struct DictKeyAdapterInt<const KEY_BITS_LEN: usize, T>(PhantomData<T>);
pub struct DictKeyAdapterMsgAddress;
pub struct DictKeyAdapterTonAddress;
pub struct DictKeyAdapterString; // TODO is not covered by tests

impl DictKeyAdapter for DictKeyAdapterTonHash {
    type KeyType = TonHash;
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
    fn make_key(src_key: &Self::KeyType) -> Result<BigUint, TonError> { Ok(src_key.clone().into()) }

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
        let big_int: BigInt = src_key.clone().into();

        let big_uint = if big_int.sign() == Sign::Minus {
            // compute 2^bits + x  (since x is negative)
            let modulo = BigUint::one() << KEY_BITS_LEN;
            let abs_val = (-big_int).to_biguint().unwrap();
            &modulo - &abs_val
        } else {
            big_int.to_biguint().unwrap()
        };

        Ok(big_uint)
    }

    fn extract_key(dict_key: &BigUint) -> Result<Self::KeyType, TonError> {
        let sign_bit = BigUint::one() << (KEY_BITS_LEN - 1);

        let big_int = if dict_key >= &sign_bit {
            // interpret as negative: x - 2^bits
            let modulo = BigUint::one() << KEY_BITS_LEN;
            BigInt::from_biguint(Sign::Minus, &modulo - dict_key)
        } else {
            BigInt::from_biguint(Sign::Plus, dict_key.clone())
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
mod tests {
    use super::*;
    use crate::tlb_adapters::{DictValAdapterTLB, TLBHashMap};
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

    #[test]
    fn test_dict_key_adapter_uint() -> anyhow::Result<()> {
        for val in [0u32, 1, 13, 190, 9999999] {
            let dict_key = DictKeyAdapterUint::make_key(&val)?;
            let extracted_val = DictKeyAdapterUint::<u32>::extract_key(&dict_key)?;
            assert_eq!(val, extracted_val);
        }
        Ok(())
    }

    #[test]
    fn test_dict_key_adapter_int() -> anyhow::Result<()> {
        for val in [-9999999i32, -190, -13, -1, 0, 1, 13, 190, 9999999] {
            let dict_key = DictKeyAdapterInt::<30, _>::make_key(&val)?;
            let extracted_val = DictKeyAdapterInt::<30, i32>::extract_key(&dict_key)?;
            assert_eq!(val, extracted_val);
        }
        Ok(())
    }

    #[test]
    fn test_dict_key_adapter_signed_key_parse_tlb() -> anyhow::Result<()> {
        let dict_hex = "b5ee9c72010207010001e600020120010200e7ae3626d0000000000000000000000000000000000000000000000000000046ec8cd22d8bffffffffffffffffffffb913732dd27800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002020277030400e7a69d930000000000000000000000000000000000000000000000000000046ec8cd22d880000000000000000000046ec8cd22d8800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002002038ddc050600e5b14500000000000000000000000000000000000000000000000000001c0c3e8aba24400000000000000000001c0c3e8aba24400000000000000000000000000000000002f23e52bc009da2a23b462da0fa694000000000000000000000000000000000000e3288b94abb3942fb8a96aeac2fe000e5b2b800000000000000000000000000000000000000000000000000001c0c3e8aba247fffffffffffffffffffe3f3c17545dbc000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000020";
        let dict_cell = TonCell::from_boc_hex(dict_hex)?;
        let dict = TLBHashMap::<DictKeyAdapterInt<24, i32>, DictValAdapterTLB<TonCell>>::new(24)
            .read(&mut dict_cell.parser())?;
        assert_eq!(dict.len(), 4);
        for key in &[-34080, -39660, -887220, 887220] {
            assert!(dict.contains_key(key), "key {key} not found, available keys: {:?}", dict.keys());
        }

        let mut builder = TonCell::builder();
        TLBHashMap::<DictKeyAdapterInt<24, i32>, DictValAdapterTLB<TonCell>>::new(24).write(&mut builder, &dict)?;
        let serialized = builder.build()?.to_boc_hex()?;
        assert_eq!(dict_hex, serialized);
        Ok(())
    }
}

use std::num::ParseIntError;
use thiserror::Error;

use crate::{cell::TonHash, types::TonAddress};

const EXTRA_CURRENCY_BASE_HASH: [u8; TonHash::BYTES_LEN] = {
    let mut prefix = [0u8; TonHash::BYTES_LEN];

    // First bit marks that this is extra currency
    prefix[0] = 1 << 7;

    prefix
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct TonExtraCurrencyId(u32);

impl TonExtraCurrencyId {
    pub fn new(id: u32) -> TonExtraCurrencyId { TonExtraCurrencyId(id) }

    pub fn to_address(id: TonExtraCurrencyId) -> TonAddress {
        let mut hash_part = EXTRA_CURRENCY_BASE_HASH;

        let id_be_bytes = u32::from(id).to_be_bytes();

        let ([.., b3_mut, b2_mut, b1_mut, b0_mut], [b3, b2, b1, b0]) = (&mut hash_part, id_be_bytes);
        *b0_mut = b0;
        *b1_mut = b1;
        *b2_mut = b2;
        *b3_mut = b3;

        TonAddress::new(0, hash_part.into())
    }

    pub fn from_address(address: &TonAddress) -> Option<TonExtraCurrencyId> {
        if EXTRA_CURRENCY_BASE_HASH[0..(TonHash::BYTES_LEN - 4)] != address.hash.as_slice()[0..(TonHash::BYTES_LEN - 4)]
        {
            return None;
        };

        let id_bytes = match address.hash.as_slice() {
            [.., b3, b2, b1, b0] => [*b3, *b2, *b1, *b0],
            _ => return None,
        };

        let id = u32::from_be_bytes(id_bytes).into();

        Some(id)
    }
}

mod traits_impl {
    use std::fmt::Display;
    use std::ops::Deref;
    use std::str::FromStr;

    use crate::types::{TonExtraCurrencyId, TonExtraCurrencyIdParseError};

    impl Display for TonExtraCurrencyId {
        fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { self.0.fmt(formatter) }
    }
    impl Deref for TonExtraCurrencyId {
        type Target = u32;
        fn deref(&self) -> &Self::Target { &self.0 }
    }
    impl From<u32> for TonExtraCurrencyId {
        fn from(id: u32) -> Self { TonExtraCurrencyId::new(id) }
    }
    impl From<TonExtraCurrencyId> for u32 {
        fn from(value: TonExtraCurrencyId) -> Self { value.0 }
    }
    impl FromStr for TonExtraCurrencyId {
        type Err = TonExtraCurrencyIdParseError;
        fn from_str(string: &str) -> Result<Self, Self::Err> { Ok(u32::from_str(string)?.into()) }
    }
}

#[derive(Debug, Error)]
pub enum TonExtraCurrencyIdParseError {
    #[error("not an unsigned 32-bit number")]
    ParseIntError(#[from] ParseIntError),
}

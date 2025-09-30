use crate::{cell::TonHash, errors::TonCoreError, types::TonAddress};

const EXTRA_CURRENCY_BASE_HASH: TonHash = {
    let mut prefix = [0u8; TonHash::BYTES_LEN];

    // First bit marks that this is extra currency
    prefix[0] = 1 << 7;

    TonHash::from_slice_sized(&prefix)
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct TonExtraCurrencyId(u32);

impl TonExtraCurrencyId {
    pub fn new(id: u32) -> TonExtraCurrencyId { TonExtraCurrencyId(id) }

    pub fn to_address(&self) -> TonAddress {
        let mut hash_part = EXTRA_CURRENCY_BASE_HASH;

        let id_be_bytes = self.to_be_bytes();

        let ([.., b3_mut, b2_mut, b1_mut, b0_mut], [b3, b2, b1, b0]) = (hash_part.as_slice_sized_mut(), id_be_bytes);
        *b0_mut = b0;
        *b1_mut = b1;
        *b2_mut = b2;
        *b3_mut = b3;

        TonAddress::new(0, hash_part)
    }

    pub fn from_address(address: &TonAddress) -> Result<TonExtraCurrencyId, TonCoreError> {
        if EXTRA_CURRENCY_BASE_HASH.as_slice_sized()[0..(TonHash::BYTES_LEN - 4)]
            != address.hash.as_slice()[0..(TonHash::BYTES_LEN - 4)]
        {
            return Err(TonCoreError::data("TonExtraCurrencyId", "Address hash mismatch"));
        };

        let id_bytes = match address.hash.as_slice() {
            [.., b3, b2, b1, b0] => [*b3, *b2, *b1, *b0],
            _ => return Err(TonCoreError::data("TonExtraCurrencyId", "Not enough bytes in address hash")),
        };

        Ok(Self(u32::from_be_bytes(id_bytes)))
    }
}

mod traits_impl {
    use std::fmt::Display;
    use std::ops::Deref;
    use std::str::FromStr;

    use crate::errors::TonCoreError;
    use crate::types::TonExtraCurrencyId;

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
        type Err = TonCoreError;
        fn from_str(string: &str) -> Result<Self, Self::Err> { Ok(u32::from_str(string)?.into()) }
    }
}

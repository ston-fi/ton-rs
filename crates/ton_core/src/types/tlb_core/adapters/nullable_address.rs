use crate::cell::{CellBuilder, CellParser};
use crate::errors::TonCoreResult;
use crate::types::TonAddress;
use crate::types::tlb_core::adapters::InternalAddress;

/// Encodes `Option<TonAddress>` like Tolk's `address?` type, without a `Maybe` bit.
///
/// `None` is `addr_none` (`00`); `Some(address)` is `addr_std` without anycast.
/// In particular, `Some(TonAddress::ZERO)` is internal `0:0`, distinct from `None`.
/// The default `Option<TonAddress>` codec instead adds a `Maybe` bit and maps
/// the zero address to `addr_none`.
///
/// Present values follow [`InternalAddress`]: reading rejects external,
/// variable-length and anycast addresses; writing rejects workchains outside
/// `int8`. This covers nullable standard internal addresses, not every form of
/// [`MsgAddress`](crate::types::tlb_core::MsgAddress). The adapter consumes only
/// its inline field; the enclosing record owns complete-consumption checks.
///
/// ```
/// # use ton_core::{cell, errors, traits};
/// use ton_core::TLB;
/// use ton_core::traits::tlb::TLB;
/// use ton_core::types::TonAddress;
/// use ton_core::types::tlb_core::adapters::NullableAddress;
///
/// #[derive(Debug, PartialEq, TLB)]
/// struct Payment {
///     #[tlb(adapter = "NullableAddress")]
///     recipient: Option<TonAddress>,
///     query_id: u64,
/// }
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// for (recipient, address_bits) in [(None, 2), (Some(TonAddress::ZERO), 267)] {
///     let payment = Payment { recipient, query_id: 7 };
///     let cell = payment.to_cell()?;
///     assert_eq!(cell.data_len_bits(), address_bits + 64);
///     assert_eq!(Payment::from_cell(&cell)?, payment);
/// }
/// # Ok(())
/// # }
/// ```
pub struct NullableAddress;

impl NullableAddress {
    /// Reads `addr_none` as `None`, or a standard internal address as `Some`.
    pub fn read(&self, parser: &mut CellParser) -> TonCoreResult<Option<TonAddress>> {
        let mut lookahead = parser.clone();
        if lookahead.read_num::<u8>(2)? == 0 {
            parser.read_num::<u8>(2)?;
            return Ok(None);
        }
        Ok(Some(InternalAddress.read(parser)?))
    }

    /// Writes Tolk `address?`, preserving `Some(0:0)` without a `Maybe` bit.
    pub fn write(&self, builder: &mut CellBuilder, value: &Option<TonAddress>) -> TonCoreResult<()> {
        match value {
            Some(address) => InternalAddress.write(builder, address),
            None => builder.write_num(&0u8, 2),
        }
    }
}

use crate::bail_ton_core_data;
use crate::cell::{CellBuilder, CellParser};
use crate::errors::{TonCoreError, TonCoreResult};
use crate::traits::tlb::TLB;
use crate::types::TonAddress;
use crate::types::tlb_core::{MsgAddress, MsgAddressInt, MsgAddressIntStd};

/// Encodes a `TonAddress` as `addr_std`, including `TonAddress::ZERO` as internal `0:0`.
///
/// Unlike the default `TonAddress` codec, this adapter never writes `addr_none`.
/// Reading rejects null, external, variable-length and anycast addresses. Writing
/// rejects workchains outside the signed 8-bit range. It consumes only the address;
/// the enclosing record owns any complete-consumption check.
///
/// ```
/// # use ton_core::{cell, errors, traits};
/// use ton_core::TLB;
/// use ton_core::traits::tlb::TLB;
/// use ton_core::types::TonAddress;
/// use ton_core::types::tlb_core::adapters::InternalAddress;
///
/// #[derive(Debug, PartialEq, TLB)]
/// struct Payment {
///     #[tlb(adapter = "InternalAddress")]
///     recipient: TonAddress,
///     query_id: u64,
/// }
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let payment = Payment { recipient: TonAddress::ZERO, query_id: 7 };
/// let cell = payment.to_cell()?;
/// assert_eq!(cell.data_len_bits(), 267 + 64);
/// assert_eq!(Payment::from_cell(&cell)?, payment);
/// # Ok(())
/// # }
/// ```
pub struct InternalAddress;

impl InternalAddress {
    /// Reads a standard internal address without anycast, preserving internal `0:0`.
    pub fn read(&self, parser: &mut CellParser) -> TonCoreResult<TonAddress> {
        match MsgAddress::read(parser)? {
            MsgAddress::Int(MsgAddressInt::Std(addr)) if addr.anycast.is_none() => {
                Ok(TonAddress::new(i32::from(addr.workchain), addr.address))
            },
            _ => bail_ton_core_data!("Expected a standard internal address without anycast"),
        }
    }

    /// Writes `addr_std` without anycast; fails if the workchain does not fit in `int8`.
    pub fn write(&self, builder: &mut CellBuilder, value: &TonAddress) -> TonCoreResult<()> {
        let workchain = i8::try_from(value.workchain)
            .map_err(|_| TonCoreError::data(module_path!(), "Internal address workchain exceeds int8"))?;
        MsgAddressIntStd {
            anycast: None,
            workchain,
            address: value.hash,
        }
        .write(builder)
    }
}

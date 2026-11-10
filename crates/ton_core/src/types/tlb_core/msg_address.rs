use crate::cell::CellBuilder;
use crate::cell::CellParser;
use crate::cell::TonHash;
use crate::errors::TonCoreError;
use crate::traits::tlb::{TLB, TLBPrefix};
use crate::types::tlb_core::VarLenBits;
use std::convert::Into;
use ton_macros::TLB;

/// TON message address.
///
/// Schema: <https://github.com/ton-blockchain/ton/blob/59a8cf0ae5c3062d14ec4c89a04fee80b5fd05c1/crypto/block/block.tlb#L100>
///
/// Structural equality preserves the address variant and its encoded fields.
/// Address-bearing records can derive `Eq` without normalizing none to standard zero:
///
/// ```
/// use ton_core::types::{TonAddress, tlb_core::MsgAddress};
///
/// #[derive(Debug, PartialEq, Eq)]
/// struct AddressRecord {
///     recipient: MsgAddress,
/// }
///
/// let absent = AddressRecord { recipient: MsgAddress::NONE };
/// let zero = AddressRecord { recipient: TonAddress::ZERO.to_msg_address_int().into() };
/// assert_ne!(absent, zero);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, TLB)]
#[non_exhaustive]
pub enum MsgAddress {
    Int(MsgAddressInt),
    Ext(MsgAddressExt),
}

/// External TON message address.
#[derive(Debug, Clone, PartialEq, Eq, TLB)]
#[non_exhaustive]
pub enum MsgAddressExt {
    None(MsgAddressNone),
    Extern(MsgAddressExtern),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, TLB)]
#[tlb(prefix = 0b00, bits_len = 2)]
pub struct MsgAddressNone;

#[derive(Debug, Clone, PartialEq, Eq, TLB)]
#[tlb(prefix = 0b01, bits_len = 2)]
pub struct MsgAddressExtern {
    pub address: VarLenBits<Vec<u8>, 9>,
}

/// Internal TON message address.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, TLB)]
#[non_exhaustive]
pub enum MsgAddressInt {
    Std(MsgAddressIntStd),
    Var(MsgAddressIntVar),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, TLB)]
#[tlb(prefix = 0b10, bits_len = 2)]
pub struct MsgAddressIntStd {
    pub anycast: Option<Anycast>,
    pub workchain: i8,
    pub address: TonHash,
}

// peculiar object - addr_bits_len is separated from addr value,
// so TLB must be implemented manually
#[derive(Debug, Clone, Eq, Hash, Ord, PartialOrd, PartialEq)]
pub struct MsgAddressIntVar {
    pub anycast: Option<Anycast>,
    pub addr_bits_len: u32, // 9 bit
    pub workchain: i32,
    pub address: Vec<u8>,
}

impl TLB for MsgAddressIntVar {
    const PREFIX: TLBPrefix = TLBPrefix::new(0b11, 2);

    fn read_definition(parser: &mut CellParser) -> Result<Self, TonCoreError> {
        let anycast = TLB::read(parser)?;
        let addr_bits_len = parser.read_num(9)?;
        let workchain = TLB::read(parser)?;
        let address = parser.read_bits(addr_bits_len as usize)?;
        Ok(Self {
            anycast,
            addr_bits_len,
            workchain,
            address,
        })
    }

    fn write_definition(&self, builder: &mut CellBuilder) -> Result<(), TonCoreError> {
        self.anycast.write(builder)?;
        builder.write_num(&self.addr_bits_len, 9)?;
        self.workchain.write(builder)?;
        builder.write_bits(&self.address, self.addr_bits_len as usize)?;
        Ok(())
    }
}

impl MsgAddress {
    pub const NONE: MsgAddress = MsgAddress::Ext(MsgAddressExt::NONE);

    /// Checks that the fields are encodable and anycast depth is in `1..=30`.
    ///
    /// This preserves the address representation. It does not establish contract
    /// acceptance or whether a particular VM version permits the address.
    ///
    /// # Errors
    /// Returns an error for invalid anycast depth or fields that do not fit their TL-B layout.
    pub fn validate(&self) -> Result<(), TonCoreError> {
        let anycast = match self {
            Self::Int(MsgAddressInt::Std(address)) => address.anycast.as_ref(),
            Self::Int(MsgAddressInt::Var(address)) => address.anycast.as_ref(),
            _ => None,
        };
        if anycast.is_some_and(|a| !(1..=30).contains(&a.rewrite_pfx.bits_len)) {
            return Err(TonCoreError::data(module_path!(), "MsgAddress anycast depth must be in 1..=30"));
        }
        self.to_cell()?;
        Ok(())
    }
}

impl MsgAddressExt {
    pub const NONE: MsgAddressExt = MsgAddressExt::None(MsgAddressNone);

    /// Creates an external address using exactly `bits_len` bits from `address`.
    pub fn new<T: Into<Vec<u8>>>(address: T, bits_len: usize) -> Self {
        MsgAddressExt::Extern(MsgAddressExtern {
            address: VarLenBits::new(address, bits_len),
        })
    }
}

impl MsgAddressInt {
    /// Returns the workchain ID.
    pub fn wc(&self) -> i32 {
        match self {
            MsgAddressInt::Std(addr) => addr.workchain as i32,
            MsgAddressInt::Var(addr) => addr.workchain,
        }
    }
    /// Returns the address bytes.
    pub fn address_hash(&self) -> &[u8] {
        match self {
            MsgAddressInt::Std(addr) => addr.address.as_slice(),
            MsgAddressInt::Var(addr) => &addr.address,
        }
    }
}

/// Allows easily convert enum variants to parent type - Extra converters (are not derived automatically)
#[rustfmt::skip]
mod from_impl {
    use super::*;
    impl From<MsgAddressNone> for MsgAddress { fn from(value: MsgAddressNone) -> Self { Self::Ext(value.into()) } }
    impl From<MsgAddressExtern> for MsgAddress { fn from(value: MsgAddressExtern) -> Self { Self::Ext(value.into()) } }
    impl From<MsgAddressIntStd> for MsgAddress { fn from(value: MsgAddressIntStd) -> Self { Self::Int(value.into()) } }
    impl From<MsgAddressIntVar> for MsgAddress { fn from(value: MsgAddressIntVar) -> Self { Self::Int(value.into()) } }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, TLB)]
pub struct Anycast {
    pub rewrite_pfx: VarLenBits<Vec<u8>, 5>,
}

impl Anycast {
    /// Creates an anycast prefix using `depth` bits from `rewrite_pfx`.
    pub fn new(depth: u32, rewrite_pfx: Vec<u8>) -> Self {
        Self {
            rewrite_pfx: VarLenBits::new(rewrite_pfx, depth as usize),
        }
    }
}

#[cfg(test)]
#[path = "../../tests/types/tlb_core/test_msg_address.rs"]
mod tests;

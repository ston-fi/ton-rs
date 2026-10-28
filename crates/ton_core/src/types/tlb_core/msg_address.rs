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
#[derive(Debug, Clone, PartialEq, TLB)]
#[non_exhaustive]
pub enum MsgAddress {
    Int(MsgAddressInt),
    Ext(MsgAddressExt),
}

/// External TON message address.
#[derive(Debug, Clone, PartialEq, TLB)]
#[non_exhaustive]
pub enum MsgAddressExt {
    None(MsgAddressNone),
    Extern(MsgAddressExtern),
}

#[derive(Debug, Clone, Copy, PartialEq, TLB)]
#[tlb(prefix = 0b00, bits_len = 2)]
pub struct MsgAddressNone;

#[derive(Debug, Clone, PartialEq, TLB)]
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

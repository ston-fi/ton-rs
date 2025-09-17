use crate::cell::CellBuilder;
use crate::cell::CellParser;
use crate::cell::TonHash;
use crate::errors::TonCoreError;
use crate::traits::tlb::{TLBPrefix, TLB};
use crate::types::tlb_core::VarLenBits;
use std::convert::Into;
use ton_lib_macros::TLB;

/// https://github.com/ton-blockchain/ton/blob/59a8cf0ae5c3062d14ec4c89a04fee80b5fd05c1/crypto/block/block.tlb#L100
/// Implemented in _core because TonAddress depends on it
#[derive(Debug, Clone, PartialEq, TLB)]
pub enum MsgAddress {
    Int(MsgAddressInt),
    Ext(MsgAddressExt),
}

// Ext
#[derive(Debug, Clone, PartialEq, TLB)]
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

// Int
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, TLB)]
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

    pub fn new<T: Into<Vec<u8>>>(address: T, bits_len: usize) -> Self {
        MsgAddressExt::Extern(MsgAddressExtern {
            address: VarLenBits::new(address, bits_len),
        })
    }
}

impl MsgAddressInt {
    pub fn wc(&self) -> i32 {
        match self {
            MsgAddressInt::Std(addr) => addr.workchain as i32,
            MsgAddressInt::Var(addr) => addr.workchain,
        }
    }
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
    pub fn new(depth: u32, rewrite_pfx: Vec<u8>) -> Self {
        Self {
            rewrite_pfx: VarLenBits::new(rewrite_pfx, depth as usize),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test::assert_ok;

    #[test]
    fn test_block_tlb_msg_address_read_write() -> anyhow::Result<()> {
        // Anyhow read/write is covered under the hood
        let boc =
            "b5ee9c7201010101002800004bbe031053100134ea6c68e2f2cee9619bdd2732493f3a1361eccd7c5267a9eb3c5dcebc533bb6";
        let parsed = MsgAddress::from_boc_hex(boc)?;
        let expected = MsgAddressIntStd {
            anycast: Some(Anycast::new(30, vec![3, 16, 83, 16])),
            workchain: 0,
            address: [
                77, 58, 155, 26, 56, 188, 179, 186, 88, 102, 247, 73, 204, 146, 79, 206, 132, 216, 123, 51, 95, 20,
                153, 234, 122, 207, 23, 115, 175, 20, 206, 237,
            ]
            .into(),
        };
        assert_eq!(parsed, expected.into());

        let serial_cell = parsed.to_cell()?;
        let parsed_back = assert_ok!(MsgAddress::from_cell(&serial_cell));
        assert_eq!(parsed_back, parsed);
        Ok(())
    }

    #[test]
    fn test_block_tlb_msg_address_int_read_i8_workchain() -> anyhow::Result<()> {
        let boc = "b5ee9c720101010100240000439fe00000000000000000000000000000000000000000000000000000000000000010";
        let parsed = assert_ok!(MsgAddress::from_boc_hex(boc));

        let expected = MsgAddressIntStd {
            anycast: None,
            workchain: -1,
            address: TonHash::from([0; 32]),
        };
        assert_eq!(parsed, expected.into());

        // don't support same layout, so check deserialized data again
        let serial_cell = parsed.to_cell()?;
        let parsed_back = assert_ok!(MsgAddress::from_cell(&serial_cell));
        assert_eq!(parsed, parsed_back);
        Ok(())
    }

    #[test]
    fn test_block_tlb_msg_address_int_read() -> anyhow::Result<()> {
        let boc = "b5ee9c720101010100240000439fe00000000000000000000000000000000000000000000000000000000000000010";
        let parsed = assert_ok!(MsgAddressInt::from_boc_hex(boc));

        let expected = MsgAddressIntStd {
            anycast: None,
            workchain: -1,
            address: TonHash::from([0; 32]),
        };
        assert_eq!(parsed, expected.into());

        // don't support same layout, so check deserialized data again
        let serial_cell = parsed.to_cell()?;
        let parsed_back = assert_ok!(MsgAddressInt::from_cell(&serial_cell));
        assert_eq!(parsed, parsed_back);
        Ok(())
    }

    #[test]
    fn test_block_tlb_msg_address_none() -> anyhow::Result<()> {
        let addr: MsgAddress = MsgAddressNone {}.into();
        let cell = addr.to_cell()?;
        let parsed = MsgAddress::from_cell(&cell)?;
        assert_eq!(parsed, addr);

        let addr_none = MsgAddressNone {};
        let addr_none_cell = addr_none.to_cell()?;
        let parsed_none = MsgAddress::from_cell(&addr_none_cell)?;
        let msg_addr_none: MsgAddress = addr_none.into();
        assert_eq!(parsed_none, msg_addr_none);
        Ok(())
    }
}

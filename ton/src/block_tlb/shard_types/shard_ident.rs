use std::fmt::{Debug, Display, Formatter};
use ton_lib_core::bits_utils::BitsUtils;
use ton_lib_core::cell::{CellBuilder, CellParser};
use ton_lib_core::constants::{TON_MASTERCHAIN, TON_MAX_SPLIT_DEPTH, TON_SHARD_FULL};
use ton_lib_core::errors::TonCoreError;
use ton_lib_core::traits::tlb::{TLBPrefix, TLB};
use ton_lib_core::types::tlb_core::MsgAddressInt;
use ton_lib_core::{bail_ton_core, bail_ton_core_data};

#[derive(Clone, Eq, Hash, PartialEq, Default)]
pub struct ShardPfx {
    pub value: u64,
    pub bits_len: u32,
}

// TLBType implementation is quite tricky, it doesn't keep shard as is
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct ShardIdent {
    pub workchain: i32,
    pub shard: u64,
}

impl ShardPfx {
    pub fn to_shard(&self) -> u64 {
        let tag = 1u64 << (63 - self.bits_len);
        (self.value & (!tag).wrapping_add(1)) | tag
    }
}

impl Debug for ShardPfx {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ShardPfx(value: 0x{:016X}, bits_len: {})", self.value, self.bits_len)
    }
}

impl ShardIdent {
    pub fn new(workchain: i32, shard: u64) -> Self { Self { workchain, shard } }
    pub fn from_pfx(workchain: i32, shard_pfx: &ShardPfx) -> Self {
        Self {
            workchain,
            shard: shard_pfx.to_shard(),
        }
    }
    pub fn new_mc() -> Self { Self::new(TON_MASTERCHAIN, TON_SHARD_FULL) }
    pub fn prefix_len(&self) -> u32 { 63u32 - self.shard.trailing_zeros() }
    pub fn split(&self) -> Result<(ShardIdent, ShardIdent), TonCoreError> {
        let lower_bits = self.prefix_lower_bits() >> 1;
        if lower_bits & (!0 >> (TON_MAX_SPLIT_DEPTH + 1)) != 0 {
            bail_ton_core!("Can't split shard {}, because of max split depth is {TON_MAX_SPLIT_DEPTH}", self.shard);
        }
        Ok((
            ShardIdent::new(self.workchain, self.shard - lower_bits),
            ShardIdent::new(self.workchain, self.shard + lower_bits),
        ))
    }
    pub fn merge(&self) -> Result<ShardIdent, TonCoreError> {
        if self.shard == TON_SHARD_FULL {
            bail_ton_core!("Can't merge shard SHARD_FULL ({})", self.shard);
        }
        let lower_bits = self.prefix_lower_bits();
        Ok(ShardIdent::new(self.workchain, (self.shard - lower_bits) | (lower_bits << 1)))
    }

    pub fn contains_addr(&self, addr: &MsgAddressInt) -> bool {
        if addr.wc() != self.workchain {
            return false;
        }
        if self.shard == TON_SHARD_FULL {
            return true;
        }
        let pfx_len_bits = self.prefix_len();
        BitsUtils::equal(&self.shard.to_be_bytes(), addr.address_hash(), pfx_len_bits as usize)
    }

    fn prefix_lower_bits(&self) -> u64 { self.shard & (!self.shard).wrapping_add(1) }
}

impl Default for ShardIdent {
    fn default() -> Self { Self::new_mc() }
}

impl TLB for ShardIdent {
    const PREFIX: TLBPrefix = TLBPrefix::new(0b00, 2);

    fn read_definition(parser: &mut CellParser) -> Result<Self, TonCoreError> {
        let pfx_bits_len: u32 = parser.read_num(6)?;
        if pfx_bits_len > TON_MAX_SPLIT_DEPTH as u32 {
            bail_ton_core_data!("expecting prefix_len <= {TON_MAX_SPLIT_DEPTH}, got {pfx_bits_len}");
        }
        let wc = parser.read_num(32)?;
        let shard_prefix: u64 = parser.read_num(64)?;
        let shard_pfx = ShardPfx {
            value: shard_prefix,
            bits_len: pfx_bits_len,
        };
        Ok(Self::from_pfx(wc, &shard_pfx))
    }

    fn write_definition(&self, builder: &mut CellBuilder) -> Result<(), TonCoreError> {
        if self.shard == 0 {
            bail_ton_core_data!("shard can't be 0");
        }
        builder.write_num(&self.prefix_len(), 6)?;
        self.workchain.write(builder)?;
        let prefix = self.shard - (self.shard & (!self.shard).wrapping_add(1));
        builder.write_num(&prefix, 64)?;
        Ok(())
    }
}

impl Debug for ShardIdent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { write!(f, "{self}") }
}

impl Display for ShardIdent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ShardIdent(wc: {}, shard: 0x{:016X})", self.workchain, self.shard)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use ton_lib_core::cell::TonCell;
    use ton_lib_core::types::TonAddress;

    #[test]
    fn test_block_tlb_shard_ident_master() -> anyhow::Result<()> {
        // master_block: shard:(shard_ident shard_pfx_bits:0 workchain_id:-1 shard_prefix:0)
        // https://explorer.toncoin.org/viewblock?workchain=-1&shard=8000000000000000&seqno=46991999&roothash=CBEBAA6AC4270C987C90C5ED930FF37F9B73C705999585D6D8C1C5E9FA3DD6E3&filehash=A660FB144506617B0C0132B92CF41E436A6A87E924FD97C769C5D6E36320327B
        let mut builder = TonCell::builder();
        builder.write_num(&0u32, 2)?; // ShardIdent prefix
        builder.write_num(&0u32, 6)?; // shard_pfx_bits
        builder.write_num(&TON_MASTERCHAIN, 32)?; // workchain
        builder.write_num(&0u64, 64)?; // shard_prefix
        let mc_shard_ident_cell = builder.build()?;
        let mc_shard_ident_parsed = ShardIdent::from_cell(&mc_shard_ident_cell)?;
        assert_eq!(mc_shard_ident_parsed.workchain, TON_MASTERCHAIN);
        assert_eq!(mc_shard_ident_parsed.shard, TON_SHARD_FULL);
        let mc_shard_ident_cell_serial = mc_shard_ident_parsed.to_cell()?;
        assert_eq!(mc_shard_ident_cell, mc_shard_ident_cell_serial);
        Ok(())
    }

    #[test]
    fn test_block_tlb_shard_ident_shard() -> anyhow::Result<()> {
        // shard_block: shard:(shard_ident shard_pfx_bits:2 workchain_id:0 shard_prefix:4611686018427387904)
        // https://explorer.toncoin.org/viewblock?workchain=0&shard=6000000000000000&seqno=52111590&roothash=D350895E85FFD081F564E5D138F374A9B52B53AEE0035B07CE5A5D6388B73B45&filehash=16944A89B7DC24BCF46AC434D5E29717B0905FDBCB523FACAFE40547BF2E7DB9
        let mut builder = TonCell::builder();
        builder.write_num(&0u32, 2)?; // ShardIdent prefix
        builder.write_num(&2u32, 6)?; // shard_pfx_bits
        builder.write_num(&0i32, 32)?; // workchain
        builder.write_num(&4611686018427387904u64, 64)?; // shard_prefix, 0x4000000000000000u64
        let shard_shard_ident_cell = builder.build()?;
        let mc_shard_ident_parsed = ShardIdent::from_cell(&shard_shard_ident_cell)?;
        assert_eq!(mc_shard_ident_parsed.workchain, 0);
        assert_eq!(mc_shard_ident_parsed.shard, 0x6000000000000000);
        let mc_shard_ident_cell_serial = mc_shard_ident_parsed.to_cell()?;
        assert_eq!(shard_shard_ident_cell, mc_shard_ident_cell_serial);
        Ok(())
    }

    #[test]
    fn test_block_tlb_shard_ident_split_merge() -> anyhow::Result<()> {
        let shard_ident = ShardIdent::new(0, 0x8000000000000000);
        let (left, right) = shard_ident.split()?;
        assert_eq!(left.workchain, 0);
        assert_eq!(left.shard, 0x4000000000000000);
        assert_eq!(right.workchain, 0);
        assert_eq!(right.shard, 0xC000000000000000);

        let (left2, right2) = left.split()?;
        assert_eq!(left2.shard, 0x2000000000000000);
        assert_eq!(right2.shard, 0x6000000000000000);

        let (left3, right3) = right.split()?;
        assert_eq!(left3.shard, 0xa000000000000000);
        assert_eq!(right3.shard, 0xe000000000000000);

        assert_eq!(left3.merge()?, right);
        assert_eq!(right3.merge()?, right);

        assert_eq!(left2.merge()?, left);
        assert_eq!(right2.merge()?, left);

        assert_eq!(left.merge()?, shard_ident);
        assert_eq!(right.merge()?, shard_ident);
        Ok(())
    }

    #[test]
    fn test_shard_ident_contains_addr() -> anyhow::Result<()> {
        let addr = TonAddress::from_str("EQDc_nrm5oOVCVQM8GRJ5q_hr1jgpNQjsGkIGE-uztt26_Ep")?;

        let shard_ident = ShardIdent::new(0, 0x8000000000000000);
        assert!(shard_ident.contains_addr(&addr.to_msg_address_int()));
        let (left, right) = shard_ident.split()?;
        assert!(!left.contains_addr(&addr.to_msg_address_int()), "shard: {left}, addr: {}", addr.to_hex());
        assert!(right.contains_addr(&addr.to_msg_address_int()), "shard: {right}, addr: {}", addr.to_hex());
        Ok(())
    }
}

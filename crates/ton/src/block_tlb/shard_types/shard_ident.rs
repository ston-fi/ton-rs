use std::fmt::{Debug, Display, Formatter};
use ton_core::bits_utils::BitsUtils;
use ton_core::cell::{CellBuilder, CellParser};
use ton_core::constants::{TON_MASTERCHAIN, TON_MAX_SPLIT_DEPTH, TON_SHARD_FULL};
use ton_core::errors::TonCoreError;
use ton_core::traits::tlb::{TLB, TLBPrefix};
use ton_core::types::tlb_core::MsgAddressInt;
use ton_core::{bail_ton_core, bail_ton_core_data};

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
    /// Creates an identifier from the exact workchain and tagged shard values.
    pub fn new(workchain: i32, shard: u64) -> Self {
        Self { workchain, shard }
    }
    pub fn from_pfx(workchain: i32, shard_pfx: &ShardPfx) -> Self {
        Self {
            workchain,
            shard: shard_pfx.to_shard(),
        }
    }
    pub fn new_mc() -> Self {
        Self::new(TON_MASTERCHAIN, TON_SHARD_FULL)
    }
    pub fn prefix_len(&self) -> u32 {
        63u32 - self.shard.trailing_zeros()
    }
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

    fn prefix_lower_bits(&self) -> u64 {
        self.shard & (!self.shard).wrapping_add(1)
    }
}

impl Default for ShardIdent {
    fn default() -> Self {
        Self::new_mc()
    }
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
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl Display for ShardIdent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ShardIdent(wc: {}, shard: 0x{:016X})", self.workchain, self.shard)
    }
}

#[cfg(test)]
#[path = "../../tests/block_tlb/shard_types/test_shard_ident.rs"]
mod tests;

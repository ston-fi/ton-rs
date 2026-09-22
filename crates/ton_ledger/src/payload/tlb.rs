// Fixed private wire records: fields and prefixes define serialization.
use ton::ton_core::{
    TLB,
    cell::{TonCell, TonHash},
    types::tlb_core::{MsgAddress, TLBCoins, TLBRef, adapters::ConstLen},
};
#[derive(TLB)]
#[tlb(prefix = 0x7258a69b, bits_len = 32, ensure_empty = true)]
pub(super) struct Whitelist {
    pub query: u64,
    pub address: MsgAddress,
}
#[derive(TLB)]
#[tlb(prefix = 0x1001, bits_len = 32, ensure_empty = true)]
pub(super) struct Validator {
    pub query: u64,
    pub address: MsgAddress,
}
#[derive(TLB)]
#[tlb(prefix = 0x1000, bits_len = 32, ensure_empty = true)]
pub(super) struct Withdraw {
    pub query: u64,
    pub amount: TLBCoins,
}
#[derive(TLB)]
#[tlb(prefix = 8, bits_len = 32, ensure_empty = true)]
pub(super) struct Swap {
    pub query: u64,
    pub id: TonHash,
}
#[derive(TLB)]
#[tlb(prefix = 0x7bcd1fef, bits_len = 32, ensure_empty = true)]
pub(super) struct WhalesDeposit {
    pub query: u64,
    pub gas: TLBCoins,
}
#[derive(TLB)]
#[tlb(prefix = 0xda803efd, bits_len = 32, ensure_empty = true)]
pub(super) struct WhalesWithdraw {
    pub query: u64,
    pub gas: TLBCoins,
    pub amount: TLBCoins,
}
#[derive(TLB)]
#[tlb(prefix = 0xa7733acd, bits_len = 32, ensure_empty = true)]
pub(super) struct Vesting {
    pub query: u64,
    pub mode: u8,
    pub message: TLBRef<TonCell>,
}

#[derive(TLB)]
#[tlb(prefix = 0x69fb306c, bits_len = 32, ensure_empty = true)]
pub(super) struct Vote {
    pub query: u64,
    pub address: MsgAddress,
    #[tlb(bits_len = 48)]
    pub expiration: u64,
    pub vote: bool,
    pub confirm: bool,
}

// This optional field has no presence bit on-chain; it occupies the remaining
// 64 bits when present. Ordinary Option<T> would change its wire format.
pub(super) struct TrailingAppId(pub Option<u64>);
impl ton::ton_core::traits::tlb::TLB for TrailingAppId {
    fn read_definition(p: &mut ton::ton_core::cell::CellParser) -> Result<Self, ton::ton_core::errors::TonCoreError> {
        let value = match p.data_bits_left()? {
            0 => None,
            64 => Some(u64::read(p)?),
            _ => return Err(ton::ton_core::errors::TonCoreError::Custom("invalid trailing app ID".into())),
        };
        p.ensure_empty()?;
        Ok(Self(value))
    }
    fn write_definition(
        &self,
        b: &mut ton::ton_core::cell::CellBuilder,
    ) -> Result<(), ton::ton_core::errors::TonCoreError> {
        if let Some(v) = self.0 {
            v.write(b)?;
        }
        Ok(())
    }
}
#[derive(TLB)]
#[tlb(prefix = 0x47d54391, bits_len = 32, ensure_empty = true)]
pub(super) struct TonstakersDeposit {
    pub query: u64,
    pub app: TrailingAppId,
}

// Fixed private wire records: fields and prefixes define serialization.
use ton::contracts::tep::nft::nft_transfer_msg::NFTTransferMsg;
use ton::ton_core::{
    TLB,
    cell::{CellBuilder, CellParser, TonCell, TonHash},
    errors::{TonCoreError, TonCoreResult},
    traits::tlb::{TLB, TLBPrefix},
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
impl TLB for TrailingAppId {
    fn read_definition(parser: &mut CellParser) -> TonCoreResult<Self> {
        let value = match parser.data_bits_left()? {
            0 => None,
            64 => Some(u64::read(parser)?),
            _ => return Err(TonCoreError::Custom("invalid trailing app ID".into())),
        };
        parser.ensure_empty()?;
        Ok(Self(value))
    }
    fn write_definition(&self, builder: &mut CellBuilder) -> TonCoreResult<()> {
        if let Some(app_id) = self.0 {
            app_id.write(builder)?;
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

#[derive(TLB)]
#[tlb(prefix = 0, bits_len = 32, ensure_empty = true)]
pub(super) struct Comment {
    pub content: TonCell,
}

#[derive(TLB)]
#[tlb(prefix = 0x4eb1f0f9, bits_len = 32, ensure_empty = true)]
pub(super) struct DnsChangeRecord {
    pub query_id: u64,
    pub key: TonHash,
    pub record: TrailingRecordRef,
}

// DNS uses an optional trailing reference without a presence bit.
pub(super) struct TrailingRecordRef(pub Option<TonCell>);
impl TLB for TrailingRecordRef {
    fn read_definition(parser: &mut CellParser) -> TonCoreResult<Self> {
        match parser.refs_left() {
            0 => Ok(Self(None)),
            1 => Ok(Self(Some(parser.read_next_ref()?.clone()))),
            _ => Err(TonCoreError::Custom("DNS record has multiple references".into())),
        }
    }

    fn write_definition(&self, builder: &mut CellBuilder) -> TonCoreResult<()> {
        if let Some(record) = &self.0 {
            builder.write_ref(record.clone())?;
        }
        Ok(())
    }
}

#[derive(TLB)]
#[tlb(prefix = 0x9fd3, bits_len = 16, ensure_empty = true)]
pub(super) struct DnsWalletRecord {
    pub address: MsgAddress,
    pub capabilities: DnsCapabilities,
}

// Firmware accepts no list, an empty list, or one wallet capability.
#[derive(TLB)]
pub(super) enum DnsCapabilities {
    Absent(NoDnsCapabilities),
    Present(DnsCapabilityList),
}

#[derive(TLB)]
#[tlb(prefix = 0, bits_len = 8)]
pub(super) struct NoDnsCapabilities;

#[derive(TLB)]
#[tlb(prefix = 1, bits_len = 8)]
pub(super) struct DnsCapabilityList {
    pub wallet: Option<DnsWalletCapability>,
}

#[derive(TLB)]
#[tlb(prefix = 0x2177, bits_len = 16)]
pub(super) struct DnsWalletCapability {
    pub end: DnsCapabilityEnd,
}

#[derive(TLB)]
#[tlb(prefix = 0, bits_len = 1)]
pub(super) struct DnsCapabilityEnd;

// NFTTransferMsg stores TonAddress, whose TLB writer normalizes zero to
// addr_none. This private adapter preserves the standard addresses reconstructed
// by firmware while retaining the existing message type and fields.
pub(super) struct NftTransfer(pub NFTTransferMsg);
impl TLB for NftTransfer {
    const PREFIX: TLBPrefix = NFTTransferMsg::PREFIX;

    fn read_definition(parser: &mut CellParser) -> TonCoreResult<Self> {
        NFTTransferMsg::read_definition(parser).map(Self)
    }

    fn write_definition(&self, builder: &mut CellBuilder) -> TonCoreResult<()> {
        let message = &self.0;
        message.query_id.write(builder)?;
        message.new_owner.to_msg_address_int().write(builder)?;
        message.response_dst.to_msg_address_int().write(builder)?;
        message.custom_payload.write(builder)?;
        message.forward_ton_amount.write(builder)?;
        message.forward_payload.write(builder)
    }
}

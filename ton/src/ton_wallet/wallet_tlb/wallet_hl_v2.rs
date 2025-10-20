use ton_core::cell::{TonCell, TonHash};
use ton_core::types::tlb_core::TLBRef;
use ton_core::TLB;

/// WalletVersion::HighloadV2R2, not tested
#[derive(Clone, Debug, TLB)]
pub struct WalletHLV2R2Data {
    pub wallet_id: i32,
    pub last_cleaned_time: u64,
    pub public_key: TonHash,
    pub queries: Option<TLBRef<TonCell>>,
}

impl WalletHLV2R2Data {
    pub fn new(wallet_id: i32, public_key: TonHash) -> Self {
        Self {
            wallet_id,
            last_cleaned_time: 0,
            public_key,
            queries: None,
        }
    }
}

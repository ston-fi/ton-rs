use super::*;
use ton_core::cell::TonCell;
use ton_core::TLB;

#[derive(Debug, Clone, PartialEq, TLB)]
pub enum JettonWalletMsgBody {
    Burn(JettonBurnMsg<TonCell>),
    BurnNotification(JettonBurnNotification),
    InternalTransfer(JettonInternalTransferMsg<TonCell>),
    Transfer(JettonTransferMsg<TonCell>),
    TransferNotification(JettonTransferNotificationMsg<TonCell>),
}

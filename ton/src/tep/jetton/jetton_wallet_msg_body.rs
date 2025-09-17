use super::*;
use ton_lib_core::TLB;

#[derive(Debug, Clone, PartialEq, TLB)]
pub enum JettonWalletMsgBody {
    Burn(JettonBurnMsg),
    BurnNotification(JettonBurnNotification),
    InternalTransfer(JettonInternalTransferMsg),
    Transfer(JettonTransferMsg),
    TransferNotification(JettonTransferNotificationMsg),
}

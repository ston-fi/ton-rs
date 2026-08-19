//! Typed Jetton wallet message bodies.

use super::jetton_burn_msg::JettonBurnMsg;
use super::jetton_burn_notification::JettonBurnNotification;
use super::jetton_internal_transfer_msg::JettonInternalTransferMsg;
use super::jetton_transfer_msg::JettonTransferMsg;
use super::jetton_transfer_notification_msg::JettonTransferNotificationMsg;
use ton_core::TLB;

#[derive(Debug, Clone, PartialEq, TLB)]
pub enum JettonWalletMsgBody {
    Burn(JettonBurnMsg),
    BurnNotification(JettonBurnNotification),
    InternalTransfer(JettonInternalTransferMsg),
    Transfer(JettonTransferMsg),
    TransferNotification(JettonTransferNotificationMsg),
}

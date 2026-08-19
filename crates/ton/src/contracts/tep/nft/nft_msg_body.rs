//! Typed NFT message bodies.

use crate::contracts::tep::excesses_msg::ExcessesMsg;
use crate::contracts::tep::nft::nft_get_static_data_msg::NFTGetStaticDataMsg;
use crate::contracts::tep::nft::nft_ownership_assigned_msg::NFTOwnershipAssignedMsg;
use crate::contracts::tep::nft::nft_report_static_data_msg::NFTReportStaticDataMsg;
use crate::contracts::tep::nft::nft_transfer_msg::NFTTransferMsg;
use ton_core::TLB;

#[derive(Clone, Debug, PartialEq, TLB)]
pub enum NFTMsgBody {
    Excesses(ExcessesMsg),
    GetStaticData(NFTGetStaticDataMsg),
    OwnershipAssigned(NFTOwnershipAssignedMsg),
    ReportStaticData(NFTReportStaticDataMsg),
    Transfer(NFTTransferMsg),
}

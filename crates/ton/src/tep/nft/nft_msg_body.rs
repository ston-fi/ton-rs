use crate::tep::excesses_msg::ExcessesMsg;
use crate::tep::nft::*;
use ton_core::TLB;

#[derive(Clone, Debug, PartialEq, TLB)]
pub enum NFTMsgBody {
    Excesses(ExcessesMsg),
    GetStaticData(NFTGetStaticDataMsg),
    OwnershipAssigned(NFTOwnershipAssignedMsg),
    ReportStaticData(NFTReportStaticDataMsg),
    Transfer(NFTTransferMsg),
}

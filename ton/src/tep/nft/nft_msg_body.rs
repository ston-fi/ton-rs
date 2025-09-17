use crate::tep::nft::*;
use ton_lib_core::TLB;

#[derive(Clone, Debug, PartialEq, TLB)]
pub enum NFTMsgBody {
    Excesses(NFTExcessesMsg),
    GetStaticData(NFTGetStaticDataMsg),
    OwnershipAssigned(NFTOwnershipAssignedMsg),
    ReportStaticData(NFTReportStaticDataMsg),
    Transfer(NFTTransferMsg),
}

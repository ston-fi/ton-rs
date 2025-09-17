use crate::tep::sbt::sbt_destroy_msg::SbtDestroyMsg;
use crate::tep::sbt::sbt_owner_info_msg::SbtOwnerInfoMsg;
use crate::tep::sbt::sbt_ownership_proof_msg::SbtOwnershipProofMsg;
use crate::tep::sbt::sbt_prove_ownership_msg::SbtProveOwnershipMsg;
use crate::tep::sbt::sbt_request_owner_msg::SbtRequestOwnerMsg;
use crate::tep::sbt::sbt_revoke_msg::SbtRevokeMsg;
use ton_lib_core::TLB;

#[derive(Clone, Debug, PartialEq, TLB)]
pub enum SbtMsgBody {
    Destroy(SbtDestroyMsg),
    OwnerInfo(SbtOwnerInfoMsg),
    OwnershipProof(SbtOwnershipProofMsg),
    ProveOwnership(SbtProveOwnershipMsg),
    RequestOwner(SbtRequestOwnerMsg),
    Revoke(SbtRevokeMsg),
}

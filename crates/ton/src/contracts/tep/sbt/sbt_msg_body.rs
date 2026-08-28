//! Typed soulbound-token message bodies.

use crate::contracts::tep::sbt::sbt_destroy_msg::SbtDestroyMsg;
use crate::contracts::tep::sbt::sbt_owner_info_msg::SbtOwnerInfoMsg;
use crate::contracts::tep::sbt::sbt_ownership_proof_msg::SbtOwnershipProofMsg;
use crate::contracts::tep::sbt::sbt_prove_ownership_msg::SbtProveOwnershipMsg;
use crate::contracts::tep::sbt::sbt_request_owner_msg::SbtRequestOwnerMsg;
use crate::contracts::tep::sbt::sbt_revoke_msg::SbtRevokeMsg;
use ton_core::TLB;

/// Supported soulbound-token message body.
#[derive(Clone, Debug, PartialEq, TLB)]
#[non_exhaustive]
pub enum SbtMsgBody {
    Destroy(SbtDestroyMsg),
    OwnerInfo(SbtOwnerInfoMsg),
    OwnershipProof(SbtOwnershipProofMsg),
    ProveOwnership(SbtProveOwnershipMsg),
    RequestOwner(SbtRequestOwnerMsg),
    Revoke(SbtRevokeMsg),
}

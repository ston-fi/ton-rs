use crate::block_tlb::*;
use crate::tlb_adapters::ConstLen;
use crate::tlb_adapters::TLBRef;
use crate::tlb_adapters::TLBRefOpt;
use std::ops::Deref;
use ton_lib_core::cell::TonHash;
use ton_lib_core::TLB;

// https://github.com/ton-blockchain/ton/blob/ed4682066978f69ffa38dd98912ca77d4f660f66/crypto/block/block.tlb#L353
#[derive(Debug, Clone, PartialEq, TLB)]
pub enum TxDescr {
    Ord(TxDescrOrd),
    Storage(TxDescrStorage),
    TickTock(TxDescrTickTock),
    SplitPrepare(TxDescrSplitPrepare),
    SplitInstall(TxDescrSplitInstall),
    MergePrepare(TxDescrMergePrepare),
    MergeInstall(TxDescrMergeInstall),
}

#[derive(Default, Debug, Clone, PartialEq, TLB)]
#[tlb(prefix = 0b0000, bits_len = 4)]
pub struct TxDescrOrd {
    pub credit_first: bool,
    pub storage_phase: Option<TrStoragePhase>,
    pub credit_phase: Option<TrCreditPhase>,
    pub compute_phase: TrComputePhase,
    #[tlb(adapter = "TLBRefOpt")]
    pub action: Option<TrActionPhase>,
    pub aborted: bool,
    pub bounce: Option<TrBouncePhase>,
    pub destroyed: bool,
}

#[derive(Debug, Clone, PartialEq, TLB)]
#[tlb(prefix = 0b0001, bits_len = 4)]
pub struct TxDescrStorage {
    pub storage_phase: TrStoragePhase,
}

#[derive(Debug, Clone, PartialEq, TLB)]
#[tlb(prefix = 0b001, bits_len = 3)]
pub struct TxDescrTickTock {
    pub is_tock: bool,
    pub storage_phase: TrStoragePhase,
    pub compute_phase: TrComputePhase,
    #[tlb(adapter = "TLBRefOpt")]
    pub action: Option<TrActionPhase>,
    pub aborted: bool,
    pub destroyed: bool,
}

#[derive(Debug, Clone, PartialEq, TLB)]
#[tlb(prefix = 0b0110, bits_len = 4)]
pub struct TxDescrSplitPrepare {
    pub split_info: SplitMergeInfo,
    pub storage_phase: Option<TrStoragePhase>,
    pub compute_phase: TrComputePhase,
    #[tlb(adapter = "TLBRefOpt")]
    pub action: Option<TrActionPhase>,
    pub aborted: bool,
    pub destroyed: bool,
}

#[derive(Debug, Clone, PartialEq, TLB)]
#[tlb(prefix = 0b0101, bits_len = 4)]
pub struct TxDescrSplitInstall {
    pub split_info: SplitMergeInfo,
    #[tlb(adapter = "TLBRef")]
    pub prepare_tx: Box<Tx>,
    pub installed: bool,
}

#[derive(Debug, Clone, PartialEq, TLB)]
#[tlb(prefix = 0b0110, bits_len = 4)]
pub struct TxDescrMergePrepare {
    pub split_info: SplitMergeInfo,
    pub storage_phase: TrStoragePhase,
    pub aborted: bool,
}

#[derive(Debug, Clone, PartialEq, TLB)]
#[tlb(prefix = 0b0111, bits_len = 4)]
pub struct TxDescrMergeInstall {
    pub split_info: SplitMergeInfo,
    #[tlb(adapter = "TLBRef")]
    pub prepare_tx: Box<Tx>,
    #[tlb(adapter = "TLBRefOpt")]
    pub storage_phase: Option<TrStoragePhase>,
    #[tlb(adapter = "TLBRefOpt")]
    pub credit_phase: Option<TrCreditPhase>,
    pub compute_phase: TrComputePhase,
    #[tlb(adapter = "TLBRefOpt")]
    pub action: Option<TrActionPhase>,
    pub aborted: bool,
    pub destroyed: bool,
}

#[derive(Debug, Clone, PartialEq, TLB)]
pub struct SplitMergeInfo {
    #[tlb(bits_len = 6)]
    pub cur_shard_pfx_len: u8,
    #[tlb(bits_len = 6)]
    pub acc_split_depth: u8,
    pub this_addr: TonHash,
    pub sibling_addr: TonHash,
}

impl Default for TxDescr {
    fn default() -> Self { TxDescrOrd::default().into() }
}

impl TxDescr {
    pub fn compute_phase(&self) -> Option<&TrComputePhaseVM> {
        let compute_phase = match &self {
            TxDescr::Ord(descr) => descr.compute_phase.as_vm(),
            TxDescr::Storage(_) => return None,
            TxDescr::TickTock(descr) => descr.compute_phase.as_vm(),
            TxDescr::SplitPrepare(_) => return None,
            TxDescr::SplitInstall(_) => return None,
            TxDescr::MergePrepare(_) => return None,
            TxDescr::MergeInstall(_) => return None,
        };
        compute_phase.map(|phase| phase.deref())
    }

    pub fn storage_phase(&self) -> Option<&TrStoragePhase> {
        match &self {
            TxDescr::Ord(descr) => descr.storage_phase.as_ref(),
            TxDescr::Storage(descr) => Some(&descr.storage_phase),
            TxDescr::TickTock(descr) => Some(&descr.storage_phase),
            TxDescr::SplitPrepare(descr) => descr.storage_phase.as_ref(),
            TxDescr::SplitInstall(_) => None,
            TxDescr::MergePrepare(descr) => Some(&descr.storage_phase),
            TxDescr::MergeInstall(descr) => descr.storage_phase.as_ref(),
        }
    }

    pub fn exit_code(&self) -> Option<i32> { self.compute_phase().map(|x| x.compute_phase_vm_info.exit_code) }
}

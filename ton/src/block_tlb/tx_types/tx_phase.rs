use crate::block_tlb::*;
use crate::tlb_adapters::TLBRef;
use ton_lib_core::cell::TonHash;
use ton_lib_core::types::tlb_core::VarLenBytes;
use ton_lib_core::TLB;

#[derive(Clone, Debug, PartialEq, TLB)]
pub struct TrStoragePhase {
    pub storage_fees_collected: Coins,
    pub storage_fees_due: Option<Coins>,
    pub status_change: AccStatusChange,
}

#[derive(Clone, Debug, PartialEq, TLB)]
pub enum TrComputePhase {
    Skipped(TrComputePhaseSkipped),
    #[rustfmt::skip]
    VM(Box::<TrComputePhaseVM>),
}

impl Default for TrComputePhase {
    fn default() -> Self {
        TrComputePhase::Skipped(TrComputePhaseSkipped {
            reason: TxComputeSkipReasonNoState.into(),
        })
    }
}

#[derive(Clone, Debug, PartialEq, TLB)]
pub struct TrActionPhase {
    pub success: bool,
    pub valid: bool,
    pub no_funds: bool,
    pub status_change: AccStatusChange,
    pub total_fwd_fees: Option<Coins>,
    pub total_action_fees: Option<Coins>,
    pub result_code: i32,
    pub result_arg: Option<i32>,
    pub tot_actions: u16,
    pub spec_actions: u16,
    pub skipped_actions: u16,
    pub msgs_created: u16,
    pub action_list_hash: TonHash,
    pub tot_msg_size: StorageUsedShort,
}

#[derive(Clone, Debug, PartialEq, TLB)]
pub enum TrBouncePhase {
    NegFunds(TrBouncePhaseNegFunds),
    NoFunds(TrBouncePhaseNoFunds),
    Ok(TrBouncePhaseOk),
}

#[derive(Clone, Debug, PartialEq, TLB)]
pub struct StorageUsedShort {
    pub cells: VarLenBytes<u64, 3>,
    pub bits: VarLenBytes<u64, 3>,
}

#[derive(Clone, Debug, PartialEq, TLB)]
#[tlb(prefix = 0b0, bits_len = 1)]
pub struct TrComputePhaseSkipped {
    pub reason: TxComputeSkipReason,
}

#[derive(Clone, Debug, PartialEq, TLB)]
#[tlb(prefix = 0b1, bits_len = 1)]
pub struct TrComputePhaseVM {
    pub success: bool,
    pub msg_state_used: bool,
    pub account_activated: bool,
    pub gas_fees: Coins,
    #[tlb(adapter = "TLBRef")]
    pub compute_phase_vm_info: ComputePhaseVMInfo,
}

#[derive(Clone, Debug, PartialEq, TLB)]
pub struct ComputePhaseVMInfo {
    pub gas_used: VarLenBytes<u64, 3>,
    pub gas_limit: VarLenBytes<u64, 3>,
    pub gas_credit: Option<VarLenBytes<u64, 2>>,
    pub mode: i8,
    pub exit_code: i32,
    pub exit_arg: Option<i32>,
    pub vm_steps: u32,
    pub vm_init_state_hash: TonHash,
    pub vm_final_state_hash: TonHash,
}

#[derive(Clone, Debug, PartialEq, TLB)]
pub enum AccStatusChange {
    Unchanged(AccStatusChangeUnchanged), // x -> x
    Frozen(AccStatusChangeFrozen),       // init -> frozen
    Deleted(AccStatusChangeDeleted),     // frozen -> deleted
}

#[derive(Clone, Debug, PartialEq, TLB)]
#[tlb(prefix = 0b0, bits_len = 1)]
pub struct AccStatusChangeUnchanged;

#[derive(Clone, Debug, PartialEq, TLB)]
#[tlb(prefix = 0b10, bits_len = 2)]
pub struct AccStatusChangeFrozen;

#[derive(Clone, Debug, PartialEq, TLB)]
#[tlb(prefix = 0b11, bits_len = 2)]
pub struct AccStatusChangeDeleted;

#[derive(Clone, Debug, PartialEq, TLB)]
pub struct TrCreditPhase {
    pub due_fees_collected: Option<Coins>,
    pub credit: CurrencyCollection,
}

#[derive(Clone, Debug, PartialEq, TLB)]
#[tlb(prefix = 0b00, bits_len = 2)]
pub struct TrBouncePhaseNegFunds;

#[derive(Clone, Debug, PartialEq, TLB)]
#[tlb(prefix = 0b01, bits_len = 2)]
pub struct TrBouncePhaseNoFunds {
    pub msg_size: StorageUsedShort,
    pub req_fwd_fee: Coins,
}

#[derive(Clone, Debug, PartialEq, TLB)]
#[tlb(prefix = 0b1, bits_len = 1)]
pub struct TrBouncePhaseOk {
    pub msg_size: StorageUsedShort,
    pub msg_fees: Coins,
    pub fws_fees: Coins,
}

#[derive(Clone, Debug, PartialEq, TLB)]
pub enum TxComputeSkipReason {
    NoState(TxComputeSkipReasonNoState),
    BadState(TxComputeSkipReasonBadState),
    NoGas(TxComputeSkipReasonNoGas),
    Suspended(TxComputeSkipReasonSuspended),
}

#[derive(Clone, Debug, PartialEq, TLB)]
#[tlb(prefix = 0b00, bits_len = 2)]
pub struct TxComputeSkipReasonNoState;

#[derive(Clone, Debug, PartialEq, TLB)]
#[tlb(prefix = 0b01, bits_len = 2)]
pub struct TxComputeSkipReasonBadState;

#[derive(Clone, Debug, PartialEq, TLB)]
#[tlb(prefix = 0b10, bits_len = 2)]
pub struct TxComputeSkipReasonNoGas;

#[derive(Clone, Debug, PartialEq, TLB)]
#[tlb(prefix = 0b110, bits_len = 3)]
pub struct TxComputeSkipReasonSuspended;

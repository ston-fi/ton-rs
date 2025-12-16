use crate::block_tlb::TVMStack;
use crate::errors::TonResult;
use crate::tep::tvm_results::tvm_result::TVMResult;
use fastnum::I512;
use ton_core::TVMResult;
use ton_macros::TLB;

#[derive(Debug, Clone, PartialEq, TVMResult, TLB)]
#[tvm_result(ensure_empty = true)]
pub struct GetDisplayMultiplierResult {
    pub numerator: I512,
    pub denominator: I512,
}

// TVMResult trait implementation tested in assert_jetton_master_scaled_ui

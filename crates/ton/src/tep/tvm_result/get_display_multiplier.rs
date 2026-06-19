use fastnum::I512;
use ton_macros::FromTVMStack;

#[derive(Debug, Clone, PartialEq, FromTVMStack)]
#[from_tvm_stack(ensure_empty = true)]
pub struct GetDisplayMultiplierResult {
    pub numerator: I512,
    pub denominator: I512,
}

// TVMType trait implementation tested in assert_jetton_master_scaled_ui

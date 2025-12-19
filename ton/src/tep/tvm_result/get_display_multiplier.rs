use fastnum::I512;
use ton_macros::TVMType;

#[derive(Debug, Clone, PartialEq, TVMType)]
#[tvm_type(ensure_empty = true)]
pub struct GetDisplayMultiplierResult {
    pub numerator: I512,
    pub denominator: I512,
}

// TVMType trait implementation tested in assert_jetton_master_scaled_ui

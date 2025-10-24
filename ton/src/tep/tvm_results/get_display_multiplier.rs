use crate::block_tlb::TVMStack;
use crate::tep::tvm_results::tvm_result::TVMResult;
use num_bigint::BigInt;
use ton_core::errors::TonCoreError;

#[derive(Debug, Clone, PartialEq)]
pub struct GetDisplayMultiplierResult {
    pub numerator: BigInt,
    pub denominator: BigInt,
}

// tested in assert_jetton_master_scaled_ui
impl TVMResult for GetDisplayMultiplierResult {
    fn from_stack(stack: &mut TVMStack) -> Result<Self, TonCoreError> {
        let denominator = stack.pop_int_or_tiny_int()?;
        let numerator = stack.pop_int_or_tiny_int()?;
        let result = Self { numerator, denominator };
        Ok(result)
    }
}

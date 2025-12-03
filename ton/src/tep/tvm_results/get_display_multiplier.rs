use crate::block_tlb::TVMStack;
use crate::errors::TonResult;
use crate::tep::tvm_results::tvm_result::TVMResult;
use fastnum::I512;

#[derive(Debug, Clone, PartialEq)]
pub struct GetDisplayMultiplierResult {
    pub numerator: I512,
    pub denominator: I512,
}

// tested in assert_jetton_master_scaled_ui
impl TVMResult for GetDisplayMultiplierResult {
    fn from_stack(stack: &mut TVMStack) -> TonResult<Self> {
        let denominator = stack.pop_number()?;
        let numerator = stack.pop_number()?;
        let result = Self { numerator, denominator };
        Ok(result)
    }
}

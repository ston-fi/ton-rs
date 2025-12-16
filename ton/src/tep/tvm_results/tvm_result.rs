use crate::block_tlb::TVMStack;
use crate::errors::TonResult;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use fastnum::I512;
use std::sync::Arc;
use ton_core::cell::TonCell;
use ton_core::traits::tlb::TLB;

#[rustfmt::skip]
pub trait TVMResult: Sized {
    /// stack must be parsed in reverse order compare to tonviewer results
    fn from_stack(stack: &mut TVMStack) -> TonResult<Self>;
    fn from_stack_boc<T: Into<Arc<Vec<u8>>>>(boc: T) -> TonResult<Self> { Self::from_stack(&mut TVMStack::from_boc(boc)?) }
    fn from_stack_boc_hex(boc: &str) -> TonResult<Self> { Self::from_stack_boc(hex::decode(boc)?) }
    fn from_stack_boc_base64(boc: &str) -> TonResult<Self> { Self::from_stack_boc(BASE64_STANDARD.decode(boc)?) }
}

mod trait_impl {
    use crate::tep::metadata::MetadataContent;

    use super::*;
    use ton_core::types::{Coins, TonAddress};

    impl TVMResult for bool {
        fn from_stack(stack: &mut TVMStack) -> TonResult<Self> { Ok(stack.pop_num()? != I512::ZERO) }
    }

    impl TVMResult for i64 {
        fn from_stack(stack: &mut TVMStack) -> TonResult<Self> { stack.pop_tiny_int() }
    }

    impl TVMResult for u32 {
        fn from_stack(stack: &mut TVMStack) -> TonResult<Self> { stack.pop_tiny_int() }
    }


    impl TVMResult for I512 {
        fn from_stack(stack: &mut TVMStack) -> TonResult<Self> { stack.pop_num() }
    }

    impl TVMResult for TonCell {
        fn from_stack(stack: &mut TVMStack) -> TonResult<Self> { stack.pop_cell() }
    }

    impl TVMResult for TonAddress {
        fn from_stack(stack: &mut TVMStack) -> TonResult<Self> { Ok(TonAddress::from_cell(&stack.pop_cell()?)?) }
    }

    impl TVMResult for MetadataContent {
        fn from_stack(stack: &mut TVMStack) -> TonResult<Self> { Ok(MetadataContent::from_cell(&stack.pop_cell()?)?) }
    }

    impl TVMResult for Coins {
        fn from_stack(stack: &mut TVMStack) -> TonResult<Self> { Ok(Coins::try_from(stack.pop_num()?)?) }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::block_tlb::TVMStack;
    use crate::errors::TonResult;
    use crate::tep::tvm_results::TVMResult;
    use tokio_test::assert_err;
    use ton_core::traits::tlb::TLB;
    use ton_core::types::TonAddress;
    use ton_macros::TVMResult;

    #[derive(TVMResult, Debug)]
    #[tvm_result(ensure_empty = true)]
    pub struct TestStruct {
        pub field1: i64,
        pub field2: TonAddress,
        pub field3: bool,
    }

    #[test]
    pub fn tvm_result_macros() -> anyhow::Result<()> {
        let mut stack = TVMStack::new(vec![]);
        stack.push_tiny_int(1);
        stack.push_cell(TonAddress::from_str("EQBiMfDMivebQb052Z6yR3jHrmwNhw1kQ5bcAUOBYsK_VPuK")?.to_cell()?);
        stack.push_tiny_int(0);

        let test_struct = TestStruct::from_stack(&mut stack)?;

        assert_eq!(test_struct.field1, 1i64);
        assert_eq!(test_struct.field2, TonAddress::from_str("EQBiMfDMivebQb052Z6yR3jHrmwNhw1kQ5bcAUOBYsK_VPuK")?);
        assert_eq!(test_struct.field3, false);

        assert_err!(assert_ensure_empty_for_stack());

        Ok(())
    }

    fn assert_ensure_empty_for_stack() -> anyhow::Result<()> {
        let mut stack = TVMStack::new(vec![]);
        stack.push_tiny_int(1);
        stack.push_tiny_int(1);
        stack.push_tiny_int(1);
        stack.push_cell(TonAddress::from_str("EQBiMfDMivebQb052Z6yR3jHrmwNhw1kQ5bcAUOBYsK_VPuK")?.to_cell()?);
        stack.push_tiny_int(0);

        let test_struct = TestStruct::from_stack(&mut stack)?;

        assert_eq!(test_struct.field1, 1i64);
        assert_eq!(test_struct.field2, TonAddress::from_str("EQBiMfDMivebQb052Z6yR3jHrmwNhw1kQ5bcAUOBYsK_VPuK")?);
        assert_eq!(test_struct.field3, false);

        Ok(())
    }
}

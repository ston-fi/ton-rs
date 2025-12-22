use crate::block_tlb::TVMStack;
use crate::errors::TonResult;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use std::sync::Arc;
use ton_core::traits::tlb::TLB;

/// Trait allows reading data from TVMStack
#[rustfmt::skip]
pub trait TVMType: Sized {
    /// stack must be parsed in reverse order compare to tonviewer results
    fn from_stack(stack: &mut TVMStack) -> TonResult<Self>;
    fn from_stack_boc<T: Into<Arc<Vec<u8>>>>(boc: T) -> TonResult<Self> { Self::from_stack(&mut TVMStack::from_boc(boc)?) }
    fn from_stack_boc_hex(boc: &str) -> TonResult<Self> { Self::from_stack_boc(hex::decode(boc)?) }
    fn from_stack_boc_base64(boc: &str) -> TonResult<Self> { Self::from_stack_boc(BASE64_STANDARD.decode(boc)?) }
}

pub trait PushToStack {
    fn push_to_stack(&self, stack: &mut TVMStack) -> TonResult<()>;
}

/// Implementations of TVMType for base classes
mod tvm_type_impls {
    use fastnum::I512;
    use ton_core::cell::{TonCell, TonHash};

    use super::*;
    use crate::errors::TonError;
    use ton_core::types::tlb_core::TLBCoins;
    use ton_core::types::{Coins, TonAddress};

    impl TVMType for bool {
        fn from_stack(stack: &mut TVMStack) -> TonResult<Self> { Ok(stack.pop_num()? != I512::ZERO) }
    }

    impl TVMType for i64 {
        fn from_stack(stack: &mut TVMStack) -> TonResult<Self> { stack.pop_tiny_int() }
    }

    impl TVMType for u32 {
        fn from_stack(stack: &mut TVMStack) -> TonResult<Self> {
            stack.pop_num().and_then(|num| {
                num.try_into().map_err(|_| TonError::UnexpectedValue {
                    expected: "u32".to_string(),
                    actual: format!("num {}", num),
                })
            })
        }
    }

    impl TVMType for I512 {
        fn from_stack(stack: &mut TVMStack) -> TonResult<Self> { stack.pop_num() }
    }

    impl TVMType for TonCell {
        fn from_stack(stack: &mut TVMStack) -> TonResult<Self> { stack.pop_cell() }
    }

    impl TVMType for TonAddress {
        fn from_stack(stack: &mut TVMStack) -> TonResult<Self> { Ok(TonAddress::from_cell(&stack.pop_cell()?)?) }
    }

    impl TVMType for TonHash {
        // only int representation is supported, reading from TonCell can be added later if needed
        fn from_stack(stack: &mut TVMStack) -> TonResult<Self> { Ok(TonHash::from_num(&stack.pop_int()?)?) }
    }

    impl TVMType for Coins {
        fn from_stack(stack: &mut TVMStack) -> TonResult<Self> { Ok(Coins::try_from(stack.pop_num()?)?) }
    }

    impl TVMType for TLBCoins {
        // only num representation is supported, reading from TonCell can be added later if needed
        fn from_stack(stack: &mut TVMStack) -> TonResult<Self> { Ok(TLBCoins::from_num(&stack.pop_num()?)?) }
    }
}

/// Implementations of TVMType for base classes
mod push_to_stack_impls {
    use super::*;
    use fastnum::I512;
    use ton_core::cell::{TonCell};
    use ton_core::types::TonAddress;

    impl PushToStack for bool {
        fn push_to_stack(&self, stack: &mut TVMStack) -> TonResult<()> {
            stack.push_tiny_int(if *self { 1 } else { 0 });
            Ok(())
        }
    }

    impl PushToStack for i64 {
        fn push_to_stack(&self, stack: &mut TVMStack) -> TonResult<()> {
            stack.push_tiny_int(*self);
            Ok(())
        }
    }

    impl PushToStack for I512 {
        fn push_to_stack(&self, stack: &mut TVMStack) -> TonResult<()> {
            stack.push_int(*self);
            Ok(())
        }
    }

    impl PushToStack for TonAddress {
        fn push_to_stack(&self, stack: &mut TVMStack) -> TonResult<()> {
            stack.push_cell_slice(self.to_cell()?);
            Ok(())
        }
    }

    impl PushToStack for TonCell {
        fn push_to_stack(&self, stack: &mut TVMStack) -> TonResult<()> {
            stack.push_cell(self.clone());
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::block_tlb::{TVMStack, TVMType};
    use std::str::FromStr;
    use tokio_test::assert_err;
    use ton_core::traits::tlb::TLB;
    use ton_core::types::TonAddress;
    use ton_macros::TVMType;

    #[derive(TVMType, Debug)]
    #[tvm_type(ensure_empty = true)]
    pub struct TestStruct {
        pub field1: i64,
        pub field2: TonAddress,
        pub field3: bool,
    }

    #[test]
    pub fn test_tvm_type_macros_work() -> anyhow::Result<()> {
        let mut stack = TVMStack::new(vec![]);
        stack.push_tiny_int(1);
        stack.push_cell(TonAddress::from_str("EQBiMfDMivebQb052Z6yR3jHrmwNhw1kQ5bcAUOBYsK_VPuK")?.to_cell()?);
        stack.push_tiny_int(0);

        let test_struct = TestStruct::from_stack(&mut stack)?;

        assert_eq!(test_struct.field1, 1i64);
        assert_eq!(test_struct.field2, TonAddress::from_str("EQBiMfDMivebQb052Z6yR3jHrmwNhw1kQ5bcAUOBYsK_VPuK")?);
        assert!(!test_struct.field3);
        Ok(())
    }

    #[test]
    pub fn test_tvm_type_ensure_empty() -> anyhow::Result<()> {
        let mut stack = TVMStack::new(vec![]);
        stack.push_tiny_int(1);
        stack.push_tiny_int(1);
        stack.push_tiny_int(1);
        stack.push_cell(TonAddress::from_str("EQBiMfDMivebQb052Z6yR3jHrmwNhw1kQ5bcAUOBYsK_VPuK")?.to_cell()?);
        stack.push_tiny_int(0);

        assert_err!(TestStruct::from_stack(&mut stack));
        Ok(())
    }
}

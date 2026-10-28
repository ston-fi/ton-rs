use crate::block_tlb::TVMStack;
use crate::errors::TonResult;

/// Trait allows pushing data to TVMStack
pub trait ToTVMStack {
    fn to_stack(&self, stack: &mut TVMStack) -> TonResult<()>;
}

/// Implementations of TVMType for base classes
mod to_tvm_stack_impls {
    use super::*;
    use crate::errors::TonError;
    use fastnum::I512;
    use ton_core::cell::TonCell;
    use ton_core::traits::tlb::TLB;
    use ton_core::types::TonAddress;
    use ton_core::types::tlb_core::MsgAddress;

    impl ToTVMStack for bool {
        fn to_stack(&self, stack: &mut TVMStack) -> TonResult<()> {
            stack.push_tiny_int(if *self { 1 } else { 0 });
            Ok(())
        }
    }

    impl ToTVMStack for i64 {
        fn to_stack(&self, stack: &mut TVMStack) -> TonResult<()> {
            stack.push_tiny_int(*self);
            Ok(())
        }
    }

    macro_rules! to_tvm_stack_primitives_impl {
        ($($t:ty),*) => {
            $(
                impl ToTVMStack for $t {
                    fn to_stack(&self, stack: &mut TVMStack) -> TonResult<()> {
                        let value = I512::try_from(*self).map_err(|_| TonError::UnexpectedValue {
                            expected: "TVM integer".into(),
                            actual: self.to_string(),
                        })?;
                        stack.push_int(value);
                        Ok(())
                    }
                }
            )*
        };
    }
    to_tvm_stack_primitives_impl!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i128, isize);

    impl ToTVMStack for I512 {
        fn to_stack(&self, stack: &mut TVMStack) -> TonResult<()> {
            stack.push_int(*self);
            Ok(())
        }
    }

    impl ToTVMStack for TonAddress {
        fn to_stack(&self, stack: &mut TVMStack) -> TonResult<()> {
            stack.push_cell_slice(self.to_cell()?);
            Ok(())
        }
    }

    impl ToTVMStack for MsgAddress {
        fn to_stack(&self, stack: &mut TVMStack) -> TonResult<()> {
            stack.push_cell_slice(self.to_cell()?);
            Ok(())
        }
    }

    impl ToTVMStack for TonCell {
        fn to_stack(&self, stack: &mut TVMStack) -> TonResult<()> {
            stack.push_cell(self.clone());
            Ok(())
        }
    }
}

#[cfg(test)]
#[path = "../../tests/block_tlb/tvm_types/test_to_tvm_stack.rs"]
mod tests;

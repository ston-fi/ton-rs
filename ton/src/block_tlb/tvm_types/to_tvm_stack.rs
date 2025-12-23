use crate::block_tlb::TVMStack;
use crate::errors::TonResult;

/// Trait allows pushing data to TVMStack
pub trait ToTVMStack {
    fn to_stack(&self, stack: &mut TVMStack) -> TonResult<()>;
}

/// Implementations of TVMType for base classes
mod to_tvm_stack_impls {
    use super::*;
    use fastnum::I512;
    use ton_core::cell::TonCell;
    use ton_core::traits::tlb::TLB;
    use ton_core::types::TonAddress;

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

    impl ToTVMStack for TonCell {
        fn to_stack(&self, stack: &mut TVMStack) -> TonResult<()> {
            stack.push_cell(self.clone());
            Ok(())
        }
    }
}

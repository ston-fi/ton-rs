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
mod tests {
    use crate::block_tlb::{FromTVMStack, TVMStack, TVMStackValue, ToTVMStack};
    use std::str::FromStr;
    use ton_core::types::TonAddress;
    use ton_core::types::tlb_core::{MsgAddress, MsgAddressExt};

    #[test]
    fn test_msg_address_stack_round_trip_preserves_variants() -> anyhow::Result<()> {
        let addresses = [
            MsgAddress::NONE,
            TonAddress::from_str("EQBiMfDMivebQb052Z6yR3jHrmwNhw1kQ5bcAUOBYsK_VPuK")?.to_msg_address(),
            MsgAddressExt::new(vec![0b1010_0000], 4).into(),
        ];

        for address in addresses {
            let mut stack = TVMStack::default();
            address.to_stack(&mut stack)?;
            assert!(matches!(stack.last(), Some(TVMStackValue::CellSlice(_))));
            assert_eq!(MsgAddress::from_stack(&mut stack)?, address);
            assert!(stack.is_empty());
        }
        Ok(())
    }
}

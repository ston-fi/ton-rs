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

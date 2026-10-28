use super::*;
use std::ops::Deref;

#[test]
fn test_block_tlb_out_list_send_msg_action_manual_build() -> anyhow::Result<()> {
    let actions_cnt = 10;
    let mut actions = vec![];
    for i in 0..actions_cnt {
        let act = OutAction::SendMsg(OutActionSendMsg {
            mode: i as u8,
            out_msg: TonCell::empty().clone().into(),
        });
        actions.push(act);
    }

    let out_list = OutList::new(actions);
    let serial_cell = out_list.to_cell()?;
    let parsed_back = OutList::from_cell(&serial_cell)?;
    assert_eq!(out_list, parsed_back);
    Ok(())
}

#[test]
fn test_block_tlb_out_list_send_msg_action_bc_data() -> anyhow::Result<()> {
    let cell = TonCell::from_boc_hex(
        "b5ee9c72010104010084000181bc04889cb28b36a3a00810e363a413763ec34860bf0fce552c5d36e37289fafd442f1983d740f92378919d969dd530aec92d258a0779fb371d4659f10ca1b3826001020a0ec3c86d0302030000006642007847b4630eb08d9f486fe846d5496878556dfd5a084f82a9a3fb01224e67c84c187a120000000000000000000000000000",
    )?;
    let mut parser = cell.parser();
    assert!(parser.read_bit()?);
    let out_list = TLBRef::<OutList>::read(&mut parser)?;

    // validate parsed data
    assert_eq!(out_list.actions.len(), 1);

    // validate serialization
    let serial = out_list.to_cell()?;
    let parsed_back = OutList::from_cell(&serial)?;
    assert_eq!(out_list.deref(), &parsed_back);
    Ok(())
}

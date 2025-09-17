use crate::block_tlb::{OutAction, OutActionSendMsg, OutList};
use ton_lib_core::bail_ton_core;
use ton_lib_core::cell::{CellBuilder, CellParser, TonCellRef};
use ton_lib_core::errors::TonCoreError;
use ton_lib_core::traits::tlb::TLB;

pub(super) fn write_up_to_4_msgs(
    dst: &mut CellBuilder,
    msgs: &[TonCellRef],
    msgs_modes: &[u8],
) -> Result<(), TonCoreError> {
    validate_msgs_count(msgs, msgs_modes, 4)?;
    for (msg, mode) in msgs.iter().zip(msgs_modes.iter()) {
        mode.write(dst)?;
        msg.write(dst)?;
    }
    Ok(())
}

pub(super) fn read_up_to_4_msgs(parser: &mut CellParser) -> Result<(Vec<u8>, Vec<TonCellRef>), TonCoreError> {
    let msgs_cnt = parser.cell.refs.len();
    let mut msgs_modes = Vec::with_capacity(msgs_cnt);
    let mut msgs = Vec::with_capacity(msgs_cnt);
    for _ in 0..msgs_cnt {
        msgs_modes.push(TLB::read(parser)?);
        msgs.push(TLB::read(parser)?);
    }
    Ok((msgs_modes, msgs))
}
pub(super) fn validate_msgs_count(msgs: &[TonCellRef], msgs_modes: &[u8], max_cnt: usize) -> Result<(), TonCoreError> {
    if msgs.len() > max_cnt || msgs_modes.len() != msgs.len() {
        bail_ton_core!("wrong msgs: modes_len={}, msgs_len={}, max_len={max_cnt}", msgs_modes.len(), msgs.len());
    }
    Ok(())
}

// V5 support
// https://github.com/ton-blockchain/wallet-contract-v5/blob/88557ebc33047a95207f6e47ac8aadb102dff744/types.tlb#L26
#[derive(Debug, PartialEq, Clone)]
pub(super) struct InnerRequest {
    out_actions: Option<OutList>, // there is Option<TLBRef<OutList>>, but we don't support such description in TLBDerive
                                  // other_actions: Option<()> unsupported
}

impl TLB for InnerRequest {
    fn read_definition(parser: &mut CellParser) -> Result<Self, TonCoreError> {
        if !parser.read_bit()? {
            return Ok(Self { out_actions: None });
        }
        let out_actions = TLB::from_cell(parser.read_next_ref()?)?;
        if parser.read_bit()? {
            bail_ton_core!("other_actions parsing is unsupported");
        }
        Ok(Self {
            out_actions: Some(out_actions),
        })
    }

    fn write_definition(&self, builder: &mut CellBuilder) -> Result<(), TonCoreError> {
        builder.write_bit(self.out_actions.is_some())?;
        if let Some(actions) = &self.out_actions {
            builder.write_ref(actions.to_cell_ref()?)?;
        }
        builder.write_bit(false)?; // other_actions are not supported
        Ok(())
    }
}

pub(super) fn parse_inner_request(request: InnerRequest) -> Result<(Vec<TonCellRef>, Vec<u8>), TonCoreError> {
    let out_list = match request.out_actions {
        Some(out_list) => out_list,
        None => return Ok((vec![], vec![])),
    };
    let mut msgs = vec![];
    let mut msgs_modes = vec![];
    for action in out_list.actions {
        if let OutAction::SendMsg(action_send_msg) = &action {
            msgs.push(action_send_msg.out_msg.clone());
            msgs_modes.push(action_send_msg.mode);
        } else {
            bail_ton_core!("Unsupported OutAction: {action:?}");
        }
    }

    Ok((msgs, msgs_modes))
}

pub(super) fn build_inner_request(msgs: &[TonCellRef], msgs_modes: &[u8]) -> Result<InnerRequest, TonCoreError> {
    if msgs.is_empty() {
        return Ok(InnerRequest { out_actions: None });
    }

    validate_msgs_count(msgs, msgs_modes, 255)?;
    let mut actions = vec![];
    for (msg, mode) in msgs.iter().zip(msgs_modes.iter()) {
        let action = OutActionSendMsg {
            mode: *mode,
            out_msg: msg.clone(),
        };
        actions.push(OutAction::SendMsg(action));
    }

    let out_list = OutList::new(actions);

    Ok(InnerRequest {
        out_actions: Some(out_list),
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use tokio_test::{assert_err, assert_ok};
    use ton_lib_core::cell::TonCell;

    #[test]
    fn test_write_up_to_4_msgs() -> anyhow::Result<()> {
        let mut builder = TonCell::builder();
        let msgs = vec![TonCell::EMPTY.into_ref(), TonCell::EMPTY.into_ref()];
        let msgs_modes = vec![1, 2];
        assert_ok!(write_up_to_4_msgs(&mut builder, &msgs, &msgs_modes));

        let mut builder = TonCell::builder();
        let msgs = vec![TonCell::EMPTY.into_ref(); 4];
        let msgs_modes = vec![1, 2, 3, 4];
        assert_ok!(write_up_to_4_msgs(&mut builder, &msgs, &msgs_modes));

        let mut builder = TonCell::builder();
        let msgs = vec![TonCell::EMPTY.into_ref(); 4];
        let msgs_modes = vec![1, 2, 3, 4, 5];
        assert_err!(write_up_to_4_msgs(&mut builder, &msgs, &msgs_modes));

        Ok(())
    }
}

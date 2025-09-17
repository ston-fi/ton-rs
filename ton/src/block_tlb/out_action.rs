use crate::block_tlb::CurrencyCollection;
use crate::tlb_adapters::ConstLen;
use ton_lib_core::cell::{CellBuilder, CellParser, TonCell, TonCellRef, TonHash};
use ton_lib_core::errors::TonCoreError;
use ton_lib_core::traits::tlb::TLB;
use ton_lib_core::types::tlb_core::TLBEither;
use ton_lib_core::TLB;

// https://github.com/ton-blockchain/ton/blob/2a68c8610bf28b43b2019a479a70d0606c2a0aa1/crypto/block/block.tlb#L399
#[derive(Debug, PartialEq, Clone, Default)]
pub struct OutList {
    pub actions: Vec<OutAction>,
}

// https://github.com/ton-blockchain/ton/blob/2a68c8610bf28b43b2019a479a70d0606c2a0aa1/crypto/block/block.tlb#L408
#[derive(Debug, PartialEq, Clone, TLB)]
pub enum OutAction {
    SendMsg(OutActionSendMsg),
    SetCode(OutActionSetCode),
    ReserveCurrency(OutActionReserveCurrency),
    ChangeLibrary(OutActionChangeLibrary),
}

#[derive(Debug, PartialEq, Clone, TLB)]
#[tlb(prefix = 0x0ec3c86d, bits_len = 32)]
pub struct OutActionSendMsg {
    pub mode: u8,
    pub out_msg: TonCellRef,
}

#[derive(Debug, PartialEq, Clone, TLB)]
#[tlb(prefix = 0xad4de08e, bits_len = 32)]
pub struct OutActionSetCode {
    pub new_code: TonCellRef,
}

#[derive(Debug, PartialEq, Clone, TLB)]
#[tlb(prefix = 0x36e6b809, bits_len = 32)]
pub struct OutActionReserveCurrency {
    pub mode: u8,
    pub currency_collection: CurrencyCollection,
}

#[derive(Debug, PartialEq, Clone, TLB)]
#[tlb(prefix = 0x26fa1dd4, bits_len = 32)]
pub struct OutActionChangeLibrary {
    #[tlb(bits_len = 7)]
    pub mode: u8,
    pub library: TLBEither<TonHash, TonCellRef>,
}

impl OutList {
    pub fn new(actions: Vec<OutAction>) -> Self { Self { actions } }
}

impl TLB for OutList {
    fn read_definition(parser: &mut CellParser) -> Result<Self, TonCoreError> {
        if parser.data_bits_remaining()? == 0 {
            return Ok(Self::default());
        }
        let mut cur_ref = parser.read_next_ref()?.clone();
        let mut actions = vec![TLB::read(parser)?];
        while !cur_ref.data.is_empty() {
            let mut cur_parser = cur_ref.parser();
            let next_ref = cur_parser.read_next_ref()?.clone();
            actions.push(TLB::read(&mut cur_parser)?);
            cur_ref = next_ref;
        }
        actions.reverse();
        Ok(Self { actions })
    }

    fn write_definition(&self, builder: &mut CellBuilder) -> Result<(), TonCoreError> {
        if self.actions.is_empty() {
            return Ok(());
        }
        let mut cur_cell = TonCell::EMPTY;
        for action in &self.actions {
            let mut parent_builder = TonCell::builder();
            parent_builder.write_ref(cur_cell.into_ref())?;
            action.write(&mut parent_builder)?;
            cur_cell = parent_builder.build()?;
        }
        cur_cell.write(builder)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::tlb_adapters::TLBRef;

    #[test]
    fn test_block_tlb_out_list_send_msg_action_manual_build() -> anyhow::Result<()> {
        let actions_cnt = 10;
        let mut actions = vec![];
        for i in 0..actions_cnt {
            let act = OutAction::SendMsg(OutActionSendMsg {
                mode: i as u8,
                out_msg: TonCell::EMPTY.into_ref(),
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
        let cell = TonCell::from_boc_hex("b5ee9c72010104010084000181bc04889cb28b36a3a00810e363a413763ec34860bf0fce552c5d36e37289fafd442f1983d740f92378919d969dd530aec92d258a0779fb371d4659f10ca1b3826001020a0ec3c86d0302030000006642007847b4630eb08d9f486fe846d5496878556dfd5a084f82a9a3fb01224e67c84c187a120000000000000000000000000000")?;
        let mut parser = cell.parser();
        assert!(parser.read_bit()?);
        let out_list: OutList = TLBRef::new().read(&mut parser)?;

        // validate parsed data
        assert_eq!(out_list.actions.len(), 1);

        // validate serialization
        let serial = out_list.to_cell()?;
        let parsed_back = OutList::from_cell(&serial)?;
        assert_eq!(out_list, parsed_back);
        Ok(())
    }
}

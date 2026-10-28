use crate::block_tlb::CurrencyCollection;
use crate::ton_core::types::tlb_core::adapters::ConstLen;
use ton_core::TLB;
use ton_core::cell::{CellBuilder, CellParser, TonCell, TonHash};
use ton_core::errors::TonCoreError;
use ton_core::traits::tlb::TLB;
use ton_core::types::tlb_core::{TLBEither, TLBRef};

// https://github.com/ton-blockchain/ton/blob/2a68c8610bf28b43b2019a479a70d0606c2a0aa1/crypto/block/block.tlb#L399
#[derive(Debug, PartialEq, Clone, Default)]
pub struct OutList {
    pub actions: Vec<OutAction>,
}

// https://github.com/ton-blockchain/ton/blob/2a68c8610bf28b43b2019a479a70d0606c2a0aa1/crypto/block/block.tlb#L408
/// Outbound action produced by a contract.
#[derive(Debug, PartialEq, Clone, TLB)]
#[non_exhaustive]
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
    pub out_msg: TLBRef<TonCell>,
}

#[derive(Debug, PartialEq, Clone, TLB)]
#[tlb(prefix = 0xad4de08e, bits_len = 32)]
pub struct OutActionSetCode {
    pub new_code: TLBRef<TonCell>,
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
    pub library: TLBEither<TonHash, TLBRef<TonCell>>,
}

impl OutList {
    /// Creates a list containing `actions` in execution order.
    pub fn new(actions: Vec<OutAction>) -> Self {
        Self { actions }
    }
}

impl TLB for OutList {
    fn read_definition(parser: &mut CellParser) -> Result<Self, TonCoreError> {
        if parser.data_bits_left()? == 0 {
            return Ok(Self::default());
        }
        let mut cur_ref = parser.read_next_ref()?.clone();
        let mut actions = vec![TLB::read(parser)?];
        while cur_ref.data_len_bits() != 0 {
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
        let mut cur_cell = TonCell::empty().to_owned();
        for action in &self.actions {
            let mut parent_builder = TonCell::builder();
            parent_builder.write_ref(cur_cell)?;
            action.write(&mut parent_builder)?;
            cur_cell = parent_builder.build()?;
        }
        cur_cell.write(builder)?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "../tests/block_tlb/test_out_action.rs"]
mod test;

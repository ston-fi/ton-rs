use crate::block_tlb::*;
use crate::tlb_adapters::*;
use crate::ton_core::types::tlb_core::adapters::ConstLen;
use std::collections::HashMap;
use ton_core::cell::*;
use ton_core::errors::TonCoreError;
use ton_core::traits::tlb::TLB;
use ton_core::types::tlb_core::TLBRef;
use ton_core::{TLB, bail_ton_core_data};

// https://github.com/ton-blockchain/ton/blob/ed4682066978f69ffa38dd98912ca77d4f660f66/crypto/block/block.tlb#L291
#[derive(Default, Clone, Debug, PartialEq, TLB)]
#[tlb(prefix = 0b0111, bits_len = 4)]
pub struct Tx {
    pub account_addr: TonHash,
    pub lt: u64,
    pub prev_tx_hash: TonHash,
    pub prev_tx_lt: u64,
    pub now: u32,
    #[tlb(bits_len = 15)]
    pub out_msgs_cnt: u16,
    pub orig_status: AccountStatus,
    pub end_status: AccountStatus,
    pub msgs: TLBRef<TxMsgs>,
    pub total_fees: CurrencyCollection,
    pub state_update: TLBRef<HashUpdate>,
    pub descr: TLBRef<TxDescr>,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct TxMsgs {
    pub in_msg: Option<Msg>,
    pub out_msgs: Vec<Msg>,
}

impl TLB for TxMsgs {
    fn read_definition(parser: &mut CellParser) -> Result<Self, TonCoreError> {
        let in_msg: Option<TLBRef<Msg>> = TLB::read(parser)?;
        let mut out_msgs_map: HashMap<u32, TLBRef<Msg>> =
            TLBHashMapE::<DictKeyAdapterUint<_>, DictValAdapterTLB<_>>::new(15).read(parser)?;
        let mut out_msgs = Vec::with_capacity(out_msgs_map.len());
        // it's important to keep the order
        for msg_index in 0..out_msgs_map.len() as u32 {
            let msg = match out_msgs_map.remove(&msg_index) {
                Some(msg) => msg.into_inner(),
                None => bail_ton_core_data!("Missing out message with index {msg_index}"),
            };
            out_msgs.push(msg);
        }
        Ok(Self {
            in_msg: in_msg.map(|x| x.into_inner()),
            out_msgs,
        })
    }

    fn write_definition(&self, builder: &mut CellBuilder) -> Result<(), TonCoreError> {
        if let Some(in_msg) = self.in_msg.as_ref() {
            builder.write_bit(true)?;
            builder.write_ref(in_msg.to_cell()?)?;
        } else {
            builder.write_bit(false)?;
        }
        let mut out_msgs_map = HashMap::with_capacity(self.out_msgs.len());
        for (pos, msg) in self.out_msgs.iter().enumerate() {
            out_msgs_map.insert(pos as u32, TLBRef::new(msg.clone().to_owned()));
        }
        TLBHashMapE::<DictKeyAdapterUint<_>, DictValAdapterTLB<_>>::new(15).write(builder, &out_msgs_map)?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../tests/block_tlb/tx_types/test_tx.rs"]
mod tests;

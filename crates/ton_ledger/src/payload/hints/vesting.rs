use super::{Hint, LedgerHintEncode, unsupported_record};
use crate::{
    error::{TonLedgerError, TonLedgerResult},
    payload::{
        encoding, exact,
        tlb::{Comment, Vesting},
    },
    protocol,
    signing::SigningPolicy,
};
use ton::{
    block_tlb::{CommonMsgInfo, CommonMsgInfoInt, Msg},
    ton_core::{
        traits::tlb::TLB,
        types::tlb_core::{EitherRefLayout, TLBCoins},
    },
};

impl LedgerHintEncode for Vesting {
    fn encode_hint(&self, policy: SigningPolicy) -> TonLedgerResult<Hint> {
        let internal_message: Msg = exact(&self.message)?;
        let CommonMsgInfo::Int(info) = &internal_message.info else {
            return Err(TonLedgerError::OpaquePayload);
        };
        let destination = protocol::standard_address(&info.dst)?;
        let comment: Comment = exact(&internal_message.body.value).map_err(unsupported_record)?;
        let text = comment.encode_hint(policy)?.data;
        let mut reconstructed = Msg::new(
            CommonMsgInfoInt::new(destination.to_msg_address_int().into(), TLBCoins::new(info.value.coins.to_u128())),
            internal_message.body.value.clone(),
        );
        reconstructed.body.layout = EitherRefLayout::Native;
        if reconstructed.to_cell()?.cell_hash()? != self.message.cell_hash()? {
            return Err(TonLedgerError::Invalid("vesting message differs from firmware reconstruction"));
        }
        let mut output = Vec::new();
        encoding::query_id(&mut output, &self.query, policy)?;
        output.push(self.mode);
        protocol::address(&mut output, &destination)?;
        protocol::coins(&mut output, info.value.coins.to_u128())?;
        output.push(text.len() as u8);
        output.extend(text);
        Ok(Hint { id: 13, data: output })
    }
}

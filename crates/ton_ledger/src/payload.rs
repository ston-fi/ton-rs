mod encoding;
mod hints;
mod message;
mod tlb;
use crate::{
    error::{TonLedgerError, TonLedgerResult},
    protocol,
    signing::SigningPolicy,
};
use ton::{
    block_tlb::{CommonMsgInfo, CommonMsgInfoInt, Msg},
    ton_core::{
        cell::TonCell,
        traits::tlb::TLB,
        types::tlb_core::{EitherRefLayout, TLBCoins, TLBEitherRef},
    },
    ton_wallet::{WalletV3ExtMsgBody, WalletV4ExtMsgBody, WalletVersion},
};

pub(crate) fn exact<T: TLB>(cell: &TonCell) -> TonLedgerResult<T> {
    let mut parser = cell.parser();
    let value = T::read(&mut parser)?;
    parser.ensure_empty()?;
    if value.to_cell()?.cell_hash()? != cell.cell_hash()? {
        return Err(TonLedgerError::Invalid("noncanonical or unrepresentable cell layout"));
    }
    Ok(value)
}
pub(crate) fn transaction(
    version: WalletVersion,
    wallet_id: i32,
    body: &TonCell,
    policy: SigningPolicy,
) -> TonLedgerResult<Vec<u8>> {
    let (id, seqno, expiry, modes, msgs) = match version {
        WalletVersion::V3R2 => {
            let wallet_body: WalletV3ExtMsgBody = exact(body)?;
            (
                wallet_body.subwallet_id,
                wallet_body.msg_seqno,
                wallet_body.valid_until,
                wallet_body.msgs_modes,
                wallet_body.msgs,
            )
        },
        WalletVersion::V4R2 => {
            let wallet_body: WalletV4ExtMsgBody = exact(body)?;
            if wallet_body.opcode != 0 {
                return Err(TonLedgerError::Invalid("V4 opcode must be zero"));
            }
            (
                wallet_body.subwallet_id,
                wallet_body.msg_seqno,
                wallet_body.valid_until,
                wallet_body.msgs_modes,
                wallet_body.msgs,
            )
        },
        _ => return Err(TonLedgerError::UnsupportedWallet),
    };
    if id != wallet_id || msgs.len() != 1 || modes.len() != 1 {
        return Err(TonLedgerError::Invalid("wallet ID or message count mismatch"));
    }
    let mode = modes[0];
    if mode & !0xe3 != 0 || mode & 0xc0 == 0xc0 {
        return Err(TonLedgerError::Invalid("unsupported send mode"));
    }
    let msg: Msg = exact(&msgs[0])?;
    let CommonMsgInfo::Int(info) = &msg.info else {
        return Err(TonLedgerError::Invalid("internal message required"));
    };
    let dst = protocol::standard_address(&info.dst)?;
    let has_payload = msg.body.layout == EitherRefLayout::ToRef;
    if !has_payload && (msg.body.value.data_len_bits() != 0 || !msg.body.value.refs().is_empty()) {
        return Err(TonLedgerError::Invalid("nonempty payload must be a reference"));
    }
    let mut canonical = Msg::new(
        CommonMsgInfoInt {
            bounce: info.bounce,
            ..CommonMsgInfoInt::new(dst.to_msg_address_int().into(), TLBCoins::new(info.value.coins.to_u128()))
        },
        msg.body.value.clone(),
    );
    canonical.body.layout = if has_payload { EitherRefLayout::ToRef } else { EitherRefLayout::ToCell };
    if let Some(init) = &msg.init {
        canonical.init = Some(TLBEitherRef::new_ref(init.value.clone()));
    }
    if canonical.to_cell()?.cell_hash()? != msgs[0].cell_hash()? {
        return Err(TonLedgerError::Invalid("firmware cannot reconstruct this internal message exactly"));
    }
    if msg.init.is_some() && policy != SigningPolicy::AllowOpaque {
        return Err(TonLedgerError::OpaquePayload);
    }
    let mut out = vec![1];
    out.extend(wallet_id.to_be_bytes());
    out.push(u8::from(version == WalletVersion::V4R2));
    out.extend(seqno.to_be_bytes());
    out.extend(expiry.to_be_bytes());
    protocol::coins(&mut out, info.value.coins.to_u128())?;
    protocol::address(&mut out, &dst)?;
    out.extend([u8::from(info.bounce), mode]);
    out.push(u8::from(msg.init.is_some()));
    if let Some(init) = &msg.init {
        protocol::cell_ref(&mut out, &init.value.to_cell()?)?;
    }
    out.push(u8::from(has_payload));
    if has_payload {
        protocol::cell_ref(&mut out, &msg.body.value)?;
        out.extend(hints::encode(&msg.body.value, policy)?);
    } else {
        out.push(0);
    }
    if out.len() > 510 {
        return Err(TonLedgerError::Invalid("transaction exceeds firmware 510-byte limit"));
    }
    Ok(out)
}
#[cfg(test)]
mod _test_payload;

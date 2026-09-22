use super::{encoding, exact, hints::LedgerHintEncode, tlb::*};
use crate::{
    error::{TonLedgerError, TonLedgerResult},
    protocol,
    signing::SigningPolicy,
};
use sha2::{Digest, Sha256};
use ton::{
    block_tlb::{CommonMsgInfo, CommonMsgInfoInt, Msg},
    contracts::tep::{
        jetton::{jetton_burn_msg::JettonBurnMsg, jetton_transfer_msg::JettonTransferMsg},
        nft::nft_transfer_msg::NFTTransferMsg,
    },
    ton_core::{
        cell::TonCell,
        traits::tlb::{TLB, TLBPrefix},
        types::tlb_core::{EitherRefLayout, MsgAddress, TLBCoins},
    },
};

pub(super) fn hints(cell: &TonCell, policy: SigningPolicy) -> TonLedgerResult<Vec<u8>> {
    match recognize(cell, policy) {
        Ok((id, data)) => {
            let mut out = vec![1];
            out.extend(id.to_be_bytes());
            out.extend((data.len() as u16).to_be_bytes());
            out.extend(data);
            Ok(out)
        },
        Err(_) if policy == SigningPolicy::AllowOpaque => Ok(vec![0]),
        Err(e) => Err(e),
    }
}
fn comment(cell: &TonCell) -> TonLedgerResult<Vec<u8>> {
    let mut p = cell.parser();
    if u32::read(&mut p)? != 0 {
        return Err(TonLedgerError::OpaquePayload);
    }
    let bits = p.data_bits_left()?;
    if bits % 8 != 0 {
        return Err(TonLedgerError::OpaquePayload);
    }
    let data = p.read_bits(bits)?;
    p.ensure_empty()?;
    protocol::printable(&data, 120)?;
    Ok(data)
}
fn encode<T: TLB + LedgerHintEncode>(cell: &TonCell, policy: SigningPolicy) -> TonLedgerResult<(u32, Vec<u8>)> {
    let message: T = exact(cell)?;
    Ok((T::HINT_ID, message.encode_hint(policy)?))
}
fn recognize(cell: &TonCell, policy: SigningPolicy) -> TonLedgerResult<(u32, Vec<u8>)> {
    let mut p = cell.parser();
    let prefix = TLBPrefix::new(u32::read(&mut p)? as usize, 32);
    let mut out = vec![];
    let id = match prefix {
        TLBPrefix { value: 0, .. } => {
            out = comment(cell)?;
            0
        },
        <JettonTransferMsg as TLB>::PREFIX => return encode::<JettonTransferMsg>(cell, policy),
        <NFTTransferMsg as TLB>::PREFIX => {
            // TonAddress serializes the zero address as addr_none. Reconstruct
            // these fields explicitly so standard zero addresses stay standard.
            let mut parser = cell.parser();
            let m = NFTTransferMsg::read(&mut parser)?;
            parser.ensure_empty()?;
            let mut canonical = TonCell::builder();
            NFTTransferMsg::write_prefix(&mut canonical)?;
            m.query_id.write(&mut canonical)?;
            m.new_owner.to_msg_address_int().write(&mut canonical)?;
            m.response_dst.to_msg_address_int().write(&mut canonical)?;
            m.custom_payload.write(&mut canonical)?;
            m.forward_ton_amount.write(&mut canonical)?;
            m.forward_payload.write(&mut canonical)?;
            if canonical.build()?.cell_hash()? != cell.cell_hash()? {
                return Err(TonLedgerError::Invalid("NFT address/layout is not representable"));
            }
            return Ok((NFTTransferMsg::HINT_ID, m.encode_hint(policy)?));
        },
        <JettonBurnMsg as TLB>::PREFIX => return encode::<JettonBurnMsg>(cell, policy),
        <Whitelist as TLB>::PREFIX => return encode::<Whitelist>(cell, policy),
        <Withdraw as TLB>::PREFIX => return encode::<Withdraw>(cell, policy),
        <Validator as TLB>::PREFIX => return encode::<Validator>(cell, policy),
        <TonstakersDeposit as TLB>::PREFIX => return encode::<TonstakersDeposit>(cell, policy),
        <Vote as TLB>::PREFIX => return encode::<Vote>(cell, policy),
        TLBPrefix { value: 0x4eb1f0f9, .. } => {
            encoding::query_id(&mut out, &u64::read(&mut p)?, policy)?;
            let key = p.read_bits(256)?;
            let value = if p.refs_left() == 1 { Some(p.read_next_ref()?.clone()) } else { None };
            p.ensure_empty()?;
            let wallet_key = Sha256::digest(b"wallet");
            let is_wallet = key.as_slice() == wallet_key.as_slice();
            out.extend([u8::from(value.is_some()), u8::from(!is_wallet)]);
            if is_wallet {
                if let Some(c) = &value {
                    let mut r = c.parser();
                    if u16::read(&mut r)? != 0x9fd3 {
                        return Err(TonLedgerError::OpaquePayload);
                    }
                    encoding::address(&mut out, &MsgAddress::read(&mut r)?, policy)?;
                    let flags = u8::read(&mut r)?;
                    if flags > 1 {
                        return Err(TonLedgerError::OpaquePayload);
                    }
                    out.push(flags);
                    if flags == 1 {
                        let cap = r.read_bit()?;
                        out.push(u8::from(cap));
                        if cap && u16::read(&mut r)? != 0x2177 {
                            return Err(TonLedgerError::OpaquePayload);
                        }
                        if cap && r.read_bit()? {
                            return Err(TonLedgerError::OpaquePayload);
                        }
                    }
                    r.ensure_empty()?;
                }
            } else {
                encoding::opaque(policy)?;
                out.extend(key);
                if let Some(c) = &value {
                    protocol::cell_ref(&mut out, c)?;
                }
            }
            9
        },
        <Swap as TLB>::PREFIX => return encode::<Swap>(cell, policy),
        <WhalesDeposit as TLB>::PREFIX => return encode::<WhalesDeposit>(cell, policy),
        <WhalesWithdraw as TLB>::PREFIX => return encode::<WhalesWithdraw>(cell, policy),
        <Vesting as TLB>::PREFIX => {
            let m: Vesting = exact(cell)?;
            let msg: Msg = exact(&m.message)?;
            let CommonMsgInfo::Int(info) = &msg.info else {
                return Err(TonLedgerError::OpaquePayload);
            };
            let dst = protocol::standard_address(&info.dst)?;
            let text = comment(&msg.body.value)?;
            let mut canonical = Msg::new(
                CommonMsgInfoInt::new(dst.to_msg_address_int().into(), TLBCoins::new(info.value.coins.to_u128())),
                msg.body.value.clone(),
            );
            canonical.body.layout = EitherRefLayout::Native;
            if canonical.to_cell()?.cell_hash()? != m.message.cell_hash()? {
                return Err(TonLedgerError::Invalid("vesting message differs from firmware reconstruction"));
            }
            encoding::query_id(&mut out, &m.query, policy)?;
            out.push(m.mode);
            protocol::address(&mut out, &dst)?;
            protocol::coins(&mut out, info.value.coins.to_u128())?;
            out.push(text.len() as u8);
            out.extend(text);
            13
        },
        _ => return Err(TonLedgerError::OpaquePayload),
    };
    Ok((id, out))
}

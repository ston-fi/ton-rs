//! Checked byte encoders shared by Ledger hint field mappings.
use super::tlb::TrailingAppId;
use crate::{
    error::{TonLedgerError, TonLedgerResult},
    protocol,
    signing::SigningPolicy,
};
use ton::ton_core::{
    cell::{TonCell, TonHash},
    traits::tlb::TLB,
    types::{
        TonAddress,
        tlb_core::{EitherRefLayout, MsgAddress, TLBCoins, TLBEitherRef, TLBRef},
    },
};

pub(super) fn query_id(out: &mut Vec<u8>, value: &u64, _policy: SigningPolicy) -> TonLedgerResult<()> {
    out.push(u8::from(*value != 0));
    if *value != 0 {
        out.extend(value.to_be_bytes());
    }
    Ok(())
}

pub(super) fn positive_query_id(out: &mut Vec<u8>, value: &u64, _policy: SigningPolicy) -> TonLedgerResult<()> {
    if *value == 0 {
        return Err(TonLedgerError::Invalid("TonWhales query ID must be positive"));
    }
    out.extend(value.to_be_bytes());
    Ok(())
}

pub(super) fn coins(out: &mut Vec<u8>, value: &TLBCoins, _policy: SigningPolicy) -> TonLedgerResult<()> {
    if value.to_cell()?.cell_hash()? != TLBCoins::new(value.to_u128()).to_cell()?.cell_hash()? {
        return Err(TonLedgerError::Invalid("nonminimal coin encoding"));
    }
    protocol::coins(out, value.to_u128())
}

pub(super) fn address(out: &mut Vec<u8>, value: &MsgAddress, _policy: SigningPolicy) -> TonLedgerResult<()> {
    protocol::address(out, &protocol::standard_address(value)?)
}

pub(super) fn ton_address(out: &mut Vec<u8>, value: &TonAddress, _policy: SigningPolicy) -> TonLedgerResult<()> {
    protocol::address(out, value)
}

pub(super) fn custom_payload(
    out: &mut Vec<u8>,
    value: &Option<TLBRef<TonCell>>,
    policy: SigningPolicy,
) -> TonLedgerResult<()> {
    out.push(u8::from(value.is_some()));
    if let Some(cell) = value {
        opaque(policy)?;
        protocol::cell_ref(out, cell)?;
    }
    Ok(())
}

pub(super) fn burn_custom_payload(
    out: &mut Vec<u8>,
    value: &Option<TLBRef<TonCell>>,
    policy: SigningPolicy,
) -> TonLedgerResult<()> {
    if let Some(cell) = value
        && cell.refs().is_empty()
        && cell.data_len_bits() % 8 == 0
        && cell.data_len_bits() <= 32 * 8
    {
        let bytes = cell.parser().read_bits(cell.data_len_bits())?;
        out.extend([2, bytes.len() as u8]);
        out.extend(bytes);
        Ok(())
    } else {
        custom_payload(out, value, policy)
    }
}

pub(super) fn forward_payload(
    out: &mut Vec<u8>,
    value: &TLBEitherRef<TonCell>,
    policy: SigningPolicy,
) -> TonLedgerResult<()> {
    if value.layout == EitherRefLayout::ToRef {
        opaque(policy)?;
        out.push(1);
        protocol::cell_ref(out, &value.value)?;
    } else if value.value.data_len_bits() == 0 && value.value.refs().is_empty() {
        out.push(0);
    } else {
        return Err(TonLedgerError::Invalid("inline forward payload cannot be represented by a hint"));
    }
    Ok(())
}

pub(super) fn trailing_app_id(out: &mut Vec<u8>, value: &TrailingAppId, _policy: SigningPolicy) -> TonLedgerResult<()> {
    out.push(u8::from(value.0.is_some()));
    if let Some(id) = value.0 {
        out.extend(id.to_be_bytes());
    }
    Ok(())
}

pub(super) fn uint48(out: &mut Vec<u8>, value: &u64, _policy: SigningPolicy) -> TonLedgerResult<()> {
    protocol::uint48(out, *value)
}

pub(super) fn boolean(out: &mut Vec<u8>, value: &bool, _policy: SigningPolicy) -> TonLedgerResult<()> {
    out.push(u8::from(*value));
    Ok(())
}

pub(super) fn hash(out: &mut Vec<u8>, value: &TonHash, _policy: SigningPolicy) -> TonLedgerResult<()> {
    out.extend(value.as_slice());
    Ok(())
}

pub(super) fn opaque(policy: SigningPolicy) -> TonLedgerResult<()> {
    if policy != SigningPolicy::AllowOpaque { Err(TonLedgerError::OpaquePayload) } else { Ok(()) }
}

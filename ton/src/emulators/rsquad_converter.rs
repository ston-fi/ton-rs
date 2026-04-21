use crate::block_tlb::{TVMCell, TVMCellSlice, TVMInt, TVMStack, TVMStackValue, TVMTinyInt, TVMTuple};
use crate::emulators::emul_bc_config::EmulBCConfig;
use crate::errors::{TonError, TonResult};
use fastnum::I512;
use rsquad_ton_block as ton_block;
use rsquad_ton_vm as ton_vm;
use std::str::FromStr;
use ton_block::{Cell, ConfigParams, Serializable, SliceData, UInt256, read_single_root_boc, write_boc};
use ton_core::cell::{TonCell, TonHash};
use ton_core::traits::tlb::TLB;

pub(in crate::emulators) fn config_params_from_emul_config(config: &EmulBCConfig) -> TonResult<ConfigParams> {
    let cell = read_single_root_boc(&config.to_boc()?)?;
    Ok(ConfigParams::with_root(cell)?)
}

pub(in crate::emulators) fn cell_to_rsquad_cell(cell: &TonCell) -> TonResult<Cell> {
    Ok(read_single_root_boc(&cell.to_boc()?)?)
}

pub(in crate::emulators) fn rsquad_cell_to_cell(cell: &Cell) -> TonResult<TonCell> {
    Ok(TonCell::from_boc(write_boc(cell)?)?)
}

pub(in crate::emulators) fn ton_hash_to_uint256(hash: &TonHash) -> TonResult<UInt256> {
    UInt256::from_str(&hash.to_hex()).map_err(|err| TonError::Custom(err.to_string()))
}

pub(in crate::emulators) fn ton_address_to_slice(address_hex: &str) -> TonResult<SliceData> {
    let address = ton_block::MsgAddressInt::from_str(address_hex)?;
    Ok(address.write_to_bitstring()?)
}

pub(in crate::emulators) fn tvm_stack_to_rsquad(stack_boc: &[u8]) -> TonResult<ton_vm::stack::Stack> {
    let stack = TVMStack::from_boc(stack_boc.to_vec())?;
    let storage = stack.iter().map(stack_value_to_rsquad).collect::<TonResult<Vec<_>>>()?;
    Ok(ton_vm::stack::Stack::with_storage(storage))
}

pub(in crate::emulators) fn rsquad_stack_to_tvm_boc(stack: &ton_vm::stack::Stack) -> TonResult<Vec<u8>> {
    let values = stack.iter().map(stack_item_to_tvm).collect::<TonResult<Vec<_>>>()?;
    Ok(TVMStack::new(values).to_boc()?)
}

pub(in crate::emulators) fn prev_blocks_info_from_boc(
    boc: Option<&[u8]>,
) -> TonResult<ton_vm::smart_contract_info::PrevBlocksInfo> {
    let Some(boc) = boc else {
        return Ok(Default::default());
    };
    let value = TVMStackValue::from_boc(boc.to_vec())?;
    Ok(ton_vm::smart_contract_info::PrevBlocksInfo::Tuple(stack_value_to_rsquad(&value)?))
}

fn stack_value_to_rsquad(value: &TVMStackValue) -> TonResult<ton_vm::stack::StackItem> {
    use ton_vm::stack::{StackItem, integer::IntegerData};

    match value {
        TVMStackValue::Null(_) => Ok(StackItem::None),
        TVMStackValue::TinyInt(value) => Ok(StackItem::int(value.value)),
        TVMStackValue::Int(value) => {
            let parsed =
                IntegerData::from_str(&value.value.to_string()).map_err(|err| TonError::Custom(err.to_string()))?;
            Ok(StackItem::integer(parsed))
        }
        TVMStackValue::Nan(_) => Ok(StackItem::nan()),
        TVMStackValue::Cell(TVMCell { value }) => Ok(StackItem::cell(cell_to_rsquad_cell(value)?)),
        TVMStackValue::CellSlice(value) => {
            let slice_cell = cell_to_rsquad_cell(&value.to_cell()?)?;
            Ok(StackItem::slice(SliceData::load_cell(slice_cell)?))
        }
        TVMStackValue::Tuple(value) => {
            Ok(StackItem::tuple(value.iter().map(stack_value_to_rsquad).collect::<TonResult<Vec<_>>>()?))
        }
        TVMStackValue::Builder(_) | TVMStackValue::Cont(_) => {
            Err(TonError::Custom(format!("rustemulator does not support TVM stack value conversion for {value:?}")))
        }
    }
}

fn stack_item_to_tvm(value: &ton_vm::stack::StackItem) -> TonResult<TVMStackValue> {
    use ton_vm::stack::StackItem;

    match value {
        StackItem::None => Ok(TVMStackValue::Null(crate::block_tlb::TVMNull)),
        StackItem::Integer(value) => {
            if value.is_nan() {
                Ok(TVMStackValue::Nan(crate::block_tlb::TVMNan))
            } else {
                let integer = I512::from_str(&value.to_string()).map_err(|err| TonError::Custom(err.to_string()))?;
                if let Ok(number) = integer.to_i64() {
                    Ok(TVMStackValue::TinyInt(TVMTinyInt { value: number }))
                } else {
                    Ok(TVMStackValue::Int(TVMInt { value: integer }))
                }
            }
        }
        StackItem::Cell(cell) => Ok(TVMStackValue::Cell(TVMCell {
            value: rsquad_cell_to_cell(cell)?.into(),
        })),
        StackItem::Slice(slice) => {
            let cell = slice.clone().into_cell()?;
            Ok(TVMStackValue::CellSlice(TVMCellSlice::from_cell(rsquad_cell_to_cell(&cell)?)))
        }
        StackItem::Tuple(values) => Ok(TVMStackValue::Tuple(TVMTuple::new(
            values.iter().map(stack_item_to_tvm).collect::<TonResult<Vec<_>>>()?,
        ))),
        StackItem::Builder(_) | StackItem::Continuation(_) => {
            Err(TonError::Custom(format!("rustemulator does not support TVM stack item conversion for {value:?}")))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{rsquad_stack_to_tvm_boc, tvm_stack_to_rsquad};
    use crate::block_tlb::{TVMStack, TVMTuple};
    use ton_core::traits::tlb::TLB;
    use ton_core::types::TonAddress;

    #[test]
    fn test_tvm_stack_roundtrip_through_rsquad_adapter() -> anyhow::Result<()> {
        let mut tuple = TVMTuple::default();
        tuple.push_tiny_int(7);
        tuple.push_cell_slice(TonAddress::ZERO.to_cell()?);

        let mut stack = TVMStack::default();
        stack.push_tiny_int(1);
        stack.push_int(2.into());
        stack.push_cell_slice(TonAddress::ZERO.to_cell()?);
        stack.push_tuple(tuple);

        let rsquad_stack = tvm_stack_to_rsquad(&stack.to_boc()?)?;
        let roundtrip = TVMStack::from_boc(rsquad_stack_to_tvm_boc(&rsquad_stack)?)?;
        assert_eq!(stack.len(), roundtrip.len());
        Ok(())
    }
}

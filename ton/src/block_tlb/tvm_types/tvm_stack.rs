use crate::block_tlb::{TVMCell, TVMCellSlice, TVMInt, TVMStackValue, TVMTinyInt, TVMTuple};
use crate::errors::TonError;
use num_bigint::BigInt;
use num_traits::FromPrimitive;
use std::fmt::Display;
use std::ops::{Deref, DerefMut};
use ton_lib_core::cell::{CellBuilder, CellParser, TonCell, TonCellRef};
use ton_lib_core::errors::TonCoreError;
use ton_lib_core::traits::tlb::TLB;

macro_rules! extract_stack_val {
    ($maybe_result:expr, $variant:ident) => {
        match $maybe_result {
            None => Err(TonError::TVMStackEmpty),
            Some(TVMStackValue::$variant(val)) => Ok(val.value),
            Some(rest) => Err(TonError::TVMStackWrongType(stringify!($variant).to_string(), format!("{rest:?}"))),
        }
    };
}

// https://github.com/ton-blockchain/ton/blob/ed4682066978f69ffa38dd98912ca77d4f660f66/crypto/block/block.tlb#L864
// Doesn't implement tlb schema directly for convenience purposes
#[derive(Debug, Clone, Default)]
pub struct TVMStack(Vec<TVMStackValue>);

impl Deref for TVMStack {
    type Target = Vec<TVMStackValue>;
    fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for TVMStack {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl TVMStack {
    pub const EMPTY_BOC: &'static [u8] = &[181, 238, 156, 114, 1, 1, 1, 1, 0, 5, 0, 0, 6, 0, 0, 0];
    pub const EMPTY: Self = Self(Vec::new());

    pub fn new(items: Vec<TVMStackValue>) -> Self { Self(items) }

    pub fn push_tiny_int(&mut self, value: i64) { self.push(TVMStackValue::TinyInt(TVMTinyInt { value })); }
    pub fn push_int(&mut self, value: BigInt) { self.push(TVMStackValue::Int(TVMInt { value })); }
    pub fn push_cell(&mut self, value: TonCellRef) { self.push(TVMStackValue::Cell(TVMCell { value })); }
    pub fn push_cell_slice(&mut self, cell: TonCellRef) {
        self.push(TVMStackValue::CellSlice(TVMCellSlice::from_cell(cell)));
    }
    pub fn push_tuple(&mut self, tuple: TVMTuple) { self.push(TVMStackValue::Tuple(tuple)); }

    pub fn pop_checked(&mut self) -> Result<TVMStackValue, TonError> {
        match self.pop() {
            None => Err(TonError::TVMStackEmpty),
            Some(value) => Ok(value),
        }
    }

    pub fn pop_tiny_int(&mut self) -> Result<i64, TonError> { extract_stack_val!(self.pop(), TinyInt) }
    pub fn pop_int(&mut self) -> Result<BigInt, TonError> { extract_stack_val!(self.pop(), Int) }

    pub fn pop_int_or_tiny_int(&mut self) -> Result<BigInt, TonError> {
        match self.pop_checked()? {
            TVMStackValue::Int(inner) => Ok(inner.value),
            TVMStackValue::TinyInt(inner) => {
                BigInt::from_i64(inner.value).ok_or(TonError::Custom("Unabled to convert BigInt from i64".to_string()))
            }
            other => Err(TonError::TVMStackWrongType(String::from("TinyInt or Int type"), other.to_string()))?,
        }
    }

    // extract cell & cell_slice
    pub fn pop_cell(&mut self) -> Result<TonCellRef, TonError> {
        match self.pop() {
            None => Err(TonError::TVMStackEmpty),
            Some(TVMStackValue::Cell(cell)) => Ok(cell.value),
            Some(TVMStackValue::CellSlice(slice)) => Ok(slice.value),
            _ => Err(TonError::TVMStackWrongType("Cell".to_string(), format!("{self:?}"))),
        }
    }
    pub fn pop_tuple(&mut self) -> Result<TVMTuple, TonError> {
        match self.pop() {
            None => Err(TonError::TVMStackEmpty),
            Some(TVMStackValue::Tuple(tuple)) => Ok(tuple),
            Some(rest) => Err(TonError::TVMStackWrongType("Tuple".to_string(), format!("{rest:?}"))),
        }
    }
}

impl TLB for TVMStack {
    fn read_definition(parser: &mut CellParser) -> Result<Self, TonCoreError> {
        let depth: u32 = parser.read_num(24)?;
        if depth == 0 {
            return Ok(Self::default());
        }

        let mut vm_stack = Self::default();
        let mut rest = parser.read_next_ref()?.clone();
        vm_stack.push(TLB::read(parser)?);
        for _ in 1..depth {
            let mut rest_parser = rest.parser();
            let new_rest = rest_parser.read_next_ref()?.clone(); // read "rest" first
            vm_stack.push(TLB::read(&mut rest_parser)?); // then read "item" itself
            rest = new_rest;
        }
        vm_stack.reverse();
        Ok(vm_stack)
    }

    fn write_definition(&self, builder: &mut CellBuilder) -> Result<(), TonCoreError> {
        builder.write_num(&self.0.len(), 24)?;
        if self.0.is_empty() {
            return Ok(());
        }
        let mut cur_rest = TonCell::EMPTY;
        // we fill cell chain from the end
        for item in self.0.iter() {
            let mut rest_builder = TonCell::builder();
            rest_builder.write_ref(cur_rest.into_ref())?; // write "rest" first
            item.write(&mut rest_builder)?; // then write "item" itself
            cur_rest = rest_builder.build()?
        }
        builder.write_cell(&cur_rest)
    }
}

impl Display for TVMStack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "===VMStack===")?;
        for item in self.0.iter().rev() {
            writeln!(f, "{item}, ")?;
        }
        write!(f, "======")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use tokio_test::assert_ok;
    use ton_lib_core::types::TonAddress;

    #[test]
    fn test_vm_stack_empty() -> anyhow::Result<()> {
        let stack = TVMStack::default();
        let cell = stack.to_cell()?;
        let mut parser = cell.parser();
        let depth: u32 = parser.read_num(24)?;
        assert_eq!(depth, 0);
        assert_ok!(parser.ensure_empty());

        let stack_parsed = TVMStack::from_cell(&cell)?;
        assert!(stack_parsed.is_empty());
        Ok(())
    }

    #[test]
    fn test_vm_stack_tiny_int() -> anyhow::Result<()> {
        let mut stack = TVMStack::new(vec![]);
        stack.push_tiny_int(1);
        let stack_cell = stack.to_cell()?;

        let mut parser = stack_cell.parser();
        let depth: u32 = parser.read_num(24)?;
        assert_eq!(depth, 1);
        assert_eq!(parser.read_next_ref()?.deref(), &TonCell::EMPTY);

        match TVMStackValue::read(&mut parser)? {
            TVMStackValue::TinyInt(val) => assert_eq!(val.value, 1),
            _ => panic!("Expected TinyInt"),
        }

        let mut stack_parsed = TVMStack::from_cell(&stack_cell)?;
        assert_eq!(stack_parsed.len(), 1);
        assert_eq!(stack_parsed.pop_tiny_int()?, 1);
        Ok(())
    }

    #[test]
    fn test_vm_stack_cell_slice() -> anyhow::Result<()> {
        let mut stack = TVMStack::new(vec![]);
        stack.push_cell_slice(TonAddress::ZERO.to_cell_ref()?);
        let stack_cell = stack.to_cell()?;

        let mut parser = stack_cell.parser();
        let depth: u32 = parser.read_num(24)?;
        assert_eq!(depth, 1);
        assert_eq!(parser.read_next_ref()?.deref(), &TonCell::EMPTY);

        match TVMStackValue::read(&mut parser)? {
            TVMStackValue::CellSlice(val) => assert_eq!(val.value.deref(), &TonAddress::ZERO.to_cell()?),
            _ => panic!("Expected CellSlice"),
        }

        let mut stack_parsed = TVMStack::from_cell(&stack_cell)?;
        assert_eq!(stack_parsed.len(), 1);
        assert_eq!(stack_parsed.pop_cell()?.deref(), &TonAddress::ZERO.to_cell()?);
        Ok(())
    }

    #[test]
    fn test_vm_stack_deep_3() -> anyhow::Result<()> {
        let mut stack = TVMStack::new(vec![]);
        stack.push_tiny_int(1);
        stack.push_int(2.into());
        stack.push_cell_slice(TonAddress::ZERO.to_cell_ref()?);
        let stack_cell = stack.to_cell()?;

        let mut deep1_parser = stack_cell.parser();
        let depth: u32 = deep1_parser.read_num(24)?;
        assert_eq!(depth, 3);

        let rest1 = deep1_parser.read_next_ref()?.clone();
        match TVMStackValue::read(&mut deep1_parser)? {
            TVMStackValue::CellSlice(val) => assert_eq!(val.value.deref(), &TonAddress::ZERO.to_cell()?),
            _ => panic!("Expected CellSlice"),
        }

        let mut deep2_parser = rest1.parser();
        let rest2 = deep2_parser.read_next_ref()?.clone();
        match TVMStackValue::read(&mut deep2_parser)? {
            TVMStackValue::Int(val) => assert_eq!(val.value, 2.into()),
            _ => panic!("Expected Int"),
        }

        let mut deep3_parser = rest2.parser();
        let rest3 = deep3_parser.read_next_ref()?.clone();
        match TVMStackValue::read(&mut deep3_parser)? {
            TVMStackValue::TinyInt(val) => assert_eq!(val.value, 1),
            _ => panic!("Expected TinyInt"),
        }

        assert_eq!(rest3.deref(), &TonCell::EMPTY);

        let mut stack_parsed = TVMStack::from_cell(&stack_cell)?;
        assert_eq!(stack_parsed.len(), 3);
        assert_eq!(stack_parsed.pop_cell()?.deref(), &TonAddress::ZERO.to_cell()?);
        assert_eq!(stack_parsed.pop_int()?, 2.into());
        assert_eq!(stack_parsed.pop_tiny_int()?, 1);
        Ok(())
    }

    #[test]
    fn test_vm_stack_complex_with_tupl() -> anyhow::Result<()> {
        let stack_boc_base64 = "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";
        let stack = TVMStack::from_boc_b64(stack_boc_base64)?;
        let stack_serial = stack.to_boc_base64()?;
        // assert_eq!(stack_boc_base64, stack_serial); // TODO doesn't work
        let stack_parsed_back = TVMStack::from_boc_b64(&stack_serial)?;
        assert_eq!(stack.cell_hash()?, stack_parsed_back.cell_hash()?);
        Ok(())
    }
}

use crate::block_tlb::{TVMCell, TVMCellSlice, TVMInt, TVMStackValue, TVMTinyInt, TVMTuple};
use crate::errors::{TonError, TonResult};
use fastnum::I512;
use std::fmt::Display;
use std::ops::{Deref, DerefMut};
use ton_core::cell::{CellBuilder, CellParser, TonCell};
use ton_core::errors::TonCoreResult;
use ton_core::traits::tlb::TLB;

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
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TVMStack(Vec<TVMStackValue>);

impl Deref for TVMStack {
    type Target = Vec<TVMStackValue>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TVMStack {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl TVMStack {
    pub const EMPTY_BOC: &'static [u8] = &[181, 238, 156, 114, 1, 1, 1, 1, 0, 5, 0, 0, 6, 0, 0, 0];
    pub const EMPTY: Self = Self(Vec::new());

    /// Creates a stack preserving the supplied bottom-to-top item order.
    pub fn new(items: Vec<TVMStackValue>) -> Self {
        Self(items)
    }

    pub fn ensure_empty(&self) -> TonResult<()> {
        if !self.is_empty() {
            return Err(TonError::TVMStackNotEmpty);
        }

        Ok(())
    }

    pub fn push_tiny_int(&mut self, value: i64) {
        self.push(TVMStackValue::TinyInt(TVMTinyInt { value }));
    }
    pub fn push_int(&mut self, value: I512) {
        self.push(TVMStackValue::Int(TVMInt { value }));
    }
    pub fn push_cell(&mut self, value: TonCell) {
        self.push(TVMStackValue::Cell(TVMCell { value: value.into() }));
    }
    pub fn push_cell_slice(&mut self, cell: TonCell) {
        self.push(TVMStackValue::CellSlice(TVMCellSlice::from_cell(cell)));
    }
    pub fn push_tuple(&mut self, value: TVMTuple) {
        self.push(TVMStackValue::Tuple(value));
    }

    pub fn pop_checked(&mut self) -> TonResult<TVMStackValue> {
        match self.pop() {
            None => Err(TonError::TVMStackEmpty),
            Some(value) => Ok(value),
        }
    }

    pub fn pop_tiny_int(&mut self) -> TonResult<i64> {
        extract_stack_val!(self.pop(), TinyInt)
    }
    pub fn pop_int(&mut self) -> TonResult<I512> {
        extract_stack_val!(self.pop(), Int)
    }
    /// Pops Int or TinyInt as I512
    pub fn pop_num(&mut self) -> TonResult<I512> {
        match self.pop_checked()? {
            TVMStackValue::Int(inner) => Ok(inner.value),
            TVMStackValue::TinyInt(inner) => Ok(I512::from_i64(inner.value)),
            other => Err(TonError::TVMStackWrongType(String::from("TinyInt or Int type"), other.to_string()))?,
        }
    }

    /// Pops Cell or CellSlice as TonCell
    pub fn pop_cell(&mut self) -> TonResult<TonCell> {
        match self.pop() {
            None => Err(TonError::TVMStackEmpty),
            Some(TVMStackValue::Cell(cell)) => Ok(cell.value.into_inner()),
            Some(TVMStackValue::CellSlice(slice)) => Ok(slice.to_cell()?),
            Some(other) => Err(TonError::TVMStackWrongType("Cell".to_string(), format!("{other:?}"))),
        }
    }
    pub fn pop_tuple(&mut self) -> TonResult<TVMTuple> {
        match self.pop() {
            None => Err(TonError::TVMStackEmpty),
            Some(TVMStackValue::Tuple(tuple)) => Ok(tuple),
            Some(rest) => Err(TonError::TVMStackWrongType("Tuple".to_string(), format!("{rest:?}"))),
        }
    }
}

impl TLB for TVMStack {
    fn read_definition(parser: &mut CellParser) -> TonCoreResult<Self> {
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

    fn write_definition(&self, builder: &mut CellBuilder) -> TonCoreResult<()> {
        builder.write_num(&self.0.len(), 24)?;
        if self.0.is_empty() {
            return Ok(());
        }
        let mut cur_rest = TonCell::empty().to_owned();
        // we fill cell chain from the end
        for item in self.0.iter() {
            let mut rest_builder = TonCell::builder();
            rest_builder.write_ref(cur_rest)?; // write "rest" first
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
#[path = "../../tests/block_tlb/tvm_types/test_tvm_stack.rs"]
mod tests;

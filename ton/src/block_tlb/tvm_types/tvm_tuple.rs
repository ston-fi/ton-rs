use crate::block_tlb::{TVMCell, TVMCellSlice, TVMInt, TVMStackValue, TVMTinyInt, TVMTupleSized};
use crate::errors::{TonError, TonResult};
use fastnum::I512;
use std::ops::{Deref, DerefMut};
use ton_core::cell::{CellBuilder, CellParser, TonCell};
use ton_core::errors::{TonCoreError, TonCoreResult};
use ton_core::traits::tlb::TLB;

macro_rules! extract_tuple_val {
    ($maybe_result:expr, $variant:ident) => {
        match &$maybe_result {
            None => Err(TonError::TVMStackEmpty),
            Some(TVMStackValue::$variant(val)) => Ok(&val.value),
            Some(rest) => Err(TonError::TVMStackWrongType(stringify!($variant).to_string(), format!("{rest:?}"))),
        }
    };
}

// https://github.com/ton-blockchain/ton/blob/master/crypto/block/block.tlb#L872C30-L872C40
// Doesn't implement tlb schema directly for convenience purposes
// Very similar with VMStackValue, but random access to underlying values
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TVMTuple(Vec<TVMStackValue>);

impl Deref for TVMTuple {
    type Target = Vec<TVMStackValue>;
    fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for TVMTuple {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

#[rustfmt::skip]
impl TVMTuple {
    pub fn new(items: Vec<TVMStackValue>) -> Self { Self(items) }

    pub fn push_tiny_int(&mut self, value: i64) { self.push(TVMStackValue::TinyInt(TVMTinyInt { value })); }
    pub fn push_int(&mut self, value: I512) { self.push(TVMStackValue::Int(TVMInt { value })); }
    pub fn push_cell(&mut self, value: TonCell) { self.push(TVMStackValue::Cell(TVMCell { value: value.into() })); }
    pub fn push_cell_slice(&mut self, cell: TonCell) { self.push(TVMStackValue::CellSlice(TVMCellSlice::from_cell(cell))); }
    pub fn push_tuple(&mut self, value: TVMTuple) {
        let tuple_sized = TVMTupleSized {
            len: value.len() as u16,
            value,
        };
        self.push(TVMStackValue::Tuple(tuple_sized))
    }

    pub fn get_tiny_int(&self, index: usize) -> TonResult<&i64> { extract_tuple_val!(self.get(index), TinyInt) }
    pub fn get_int(&self, index: usize) -> TonResult<&I512> { extract_tuple_val!(self.get(index), Int) }
    pub fn get_cell(&self, index: usize) -> TonResult<&TonCell> {
        match self.get(index) {
            None => Err(TonError::TVMStackEmpty),
            Some(TVMStackValue::Cell(val)) => Ok(&val.value),
            Some(TVMStackValue::CellSlice(val)) => Ok(&val.value),
            Some(rest) => Err(TonError::TVMStackWrongType("Cell | CellSlice".to_string(), format!("{rest:?}"))),
        }
    }
    pub fn get_cell_slice(&self, index: usize) -> TonResult<&TonCell> { extract_tuple_val!(self.get(index), CellSlice) }
    pub fn get_tuple(&self, index: usize) -> TonResult<&TVMTuple> { extract_tuple_val!(self.get(index), Tuple) }
}

impl TLB for TVMTuple {
    fn read_definition(parser: &mut CellParser) -> TonCoreResult<Self> {
        let mut data = Vec::new();
        read_tuple(parser, &mut data)?;
        Ok(TVMTuple(data))
    }

    fn write_definition(&self, builder: &mut CellBuilder) -> TonCoreResult<()> {
        write_tuple(builder, self)?;
        Ok(())
    }
}

// vm_tuple_nil$_ = VmTuple 0;
// vm_tuple_tcons$_ {n:#} head:(VmTupleRef n) tail:^VmStackValue = VmTuple (n + 1);
fn read_tuple(parser: &mut CellParser, data: &mut Vec<TVMStackValue>) -> TonCoreResult<()> {
    // println!("entered read_tuple with {} bits, {} refs", parser.data_bits_left()?, parser.refs_left());
    // handling VmTuple 0;
    if parser.refs_left() == 0 {
        return Ok(());
    }
    read_tuple_ref(parser, data)?;
    let tail = TVMStackValue::from_cell(parser.read_next_ref()?)?;
    data.push(tail);
    Ok(())
}

// vm_tupref_nil$_ = VmTupleRef 0;
// vm_tupref_single$_ entry:^VmStackValue = VmTupleRef 1;
// vm_tupref_any$_ {n:#} ref:^(VmTuple (n + 2)) = VmTupleRef (n + 2);
fn read_tuple_ref(parser: &mut CellParser, data: &mut Vec<TVMStackValue>) -> TonCoreResult<()> {
    // println!("entered read_tuple_ref with {} bits, {} refs", parser.data_bits_left()?, parser.refs_left());
    // handling VmTupleRef 0. Remaining ref is tail from VmTuple 1
    if parser.refs_left() == 1 {
        return Ok(());
    }

    // ^VmStackValue or ^(VmTuple (n + 2))
    let mut next_ref_parser = parser.read_next_ref()?.parser();

    // If reference has data, it's VmStackValue (VmStackValue always has prefix in data)
    if next_ref_parser.data_bits_left()? > 0 {
        data.push(TVMStackValue::read(&mut next_ref_parser)?);
        return Ok(());
    }
    read_tuple(&mut next_ref_parser, data)?;
    Ok(())
}

fn write_tuple(builder: &mut CellBuilder, data: &[TVMStackValue]) -> TonCoreResult<()> {
    if data.is_empty() {
        return Ok(());
    }
    write_tuple_ref(builder, &data[..data.len() - 1])?;
    builder.write_ref(data.last().unwrap().to_cell()?) // unwrap is safe: data is not empty
}

// vm_tupref_nil$_ = VmTupleRef 0;
// vm_tupref_single$_ entry:^VmStackValue = VmTupleRef 1;
// vm_tupref_any$_ {n:#} ref:^(VmTuple (n + 2)) = VmTupleRef (n + 2);
fn write_tuple_ref(builder: &mut CellBuilder, data: &[TVMStackValue]) -> TonCoreResult<()> {
    if data.is_empty() {
        return Ok(());
    }
    if data.len() == 1 {
        return builder.write_ref(data[0].to_cell()?);
    }
    let mut rest_builder = TonCell::builder();
    write_tuple(&mut rest_builder, &data)?;
    builder.write_ref(rest_builder.build()?)
}

impl Into<TVMStackValue> for TVMTuple {
    fn into(self) -> TVMStackValue {
        let wrapper = TVMTupleSized {
            len: self.len() as u16,
            value: self,
        };
        TVMStackValue::Tuple(wrapper)
    }
}

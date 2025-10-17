use ton_lib_core::cell::CellBuilder;
use ton_lib_core::cell::CellParser;
use ton_lib_core::cell::TonCellNum;
use ton_lib_core::errors::TonCoreError;
use ton_lib_core::traits::tlb::TLB;

pub trait DictValAdapter {
    type ValType;
    fn write(builder: &mut CellBuilder, val: &Self::ValType) -> Result<(), TonCoreError>;
    fn read(parser: &mut CellParser) -> Result<Self::ValType, TonCoreError>;
}

pub struct DictValAdapterTLB<T: TLB>(std::marker::PhantomData<T>);
pub struct DictValAdapterNum<T, const BITS_LEN: usize>(std::marker::PhantomData<T>);

impl<T: TLB> DictValAdapter for DictValAdapterTLB<T> {
    type ValType = T;
    fn write(builder: &mut CellBuilder, val: &T) -> Result<(), TonCoreError> { val.write(builder) }
    fn read(parser: &mut CellParser) -> Result<T, TonCoreError> { T::read(parser) }
}

impl<T: TonCellNum, const BITS_LEN: usize> DictValAdapter for DictValAdapterNum<T, BITS_LEN> {
    type ValType = T;
    fn write(builder: &mut CellBuilder, val: &T) -> Result<(), TonCoreError> { builder.write_num(val, BITS_LEN) }
    fn read(parser: &mut CellParser) -> Result<T, TonCoreError> { parser.read_num(BITS_LEN) }
}

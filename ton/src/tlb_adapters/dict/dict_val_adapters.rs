use ton_lib_core::cell::CellBuilder;
use ton_lib_core::cell::CellParser;
use ton_lib_core::cell::TonCellNum;
use ton_lib_core::errors::TonCoreError;
use ton_lib_core::traits::tlb::TLB;

pub trait DictValAdapter<T> {
    fn write(builder: &mut CellBuilder, val: &T) -> Result<(), TonCoreError>;
    fn read(parser: &mut CellParser) -> Result<T, TonCoreError>;
}

pub struct DictValAdapterTLB;
pub struct DictValAdapterNum<const BITS_LEN: usize>;

impl<T: TLB> DictValAdapter<T> for DictValAdapterTLB {
    fn write(builder: &mut CellBuilder, val: &T) -> Result<(), TonCoreError> { val.write(builder) }
    fn read(parser: &mut CellParser) -> Result<T, TonCoreError> { T::read(parser) }
}

impl<T: TonCellNum, const BITS_LEN: usize> DictValAdapter<T> for DictValAdapterNum<BITS_LEN> {
    fn write(builder: &mut CellBuilder, val: &T) -> Result<(), TonCoreError> { builder.write_num(val, BITS_LEN) }
    fn read(parser: &mut CellParser) -> Result<T, TonCoreError> { parser.read_num(BITS_LEN) }
}

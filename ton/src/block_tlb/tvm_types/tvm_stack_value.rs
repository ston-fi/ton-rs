use crate::block_tlb::{TVMCellSlice, TVMStack, TVMTuple};
use crate::tlb_adapters::DictKeyAdapterUint;
use crate::tlb_adapters::DictValAdapterTLB;
use crate::tlb_adapters::TLBHashMap;
use crate::ton_lib_core::types::tlb_core::adapters::ConstLen;
use num_bigint::BigInt;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;
use std::sync::Arc;
use ton_lib_core::cell::TonCell;
use ton_lib_core::types::tlb_core::TLBRef;
use ton_lib_core::TLB;

#[derive(Clone, TLB)]
pub enum TVMStackValue {
    Null(TVMNull),
    TinyInt(TVMTinyInt),
    Int(TVMInt),
    Nan(TVMNan),
    Cell(TVMCell),
    CellSlice(TVMCellSlice),
    Builder(TVMBuilder), // TODO is not tested
    Cont(TVMCont),       // TODO is not tested
    Tuple(TVMTuple),
}

#[derive(Debug, Clone, TLB)]
#[tlb(prefix = 0x00, bits_len = 8)]
pub struct TVMNull;

#[derive(Debug, Clone, TLB)]
#[tlb(prefix = 0x01, bits_len = 8)]
pub struct TVMTinyInt {
    pub value: i64,
}

// vm_stk_int#0201_ value:int257 = VmStackValue; means 0x0201 without latest bit ==> 0000001000000000
#[derive(Debug, Clone, TLB)]
#[tlb(prefix = 0x0100, bits_len = 15)]
pub struct TVMInt {
    #[tlb(bits_len = 257)]
    pub value: BigInt,
}

#[derive(Debug, Clone, TLB)]
#[tlb(prefix = 0x02ff, bits_len = 16)]
pub struct TVMNan;

#[derive(Debug, Clone, TLB)]
#[tlb(prefix = 0x03, bits_len = 8)]
pub struct TVMCell {
    pub value: TLBRef<TonCell>,
}

#[derive(Debug, Clone, TLB)]
#[tlb(prefix = 0x05, bits_len = 8)]
pub struct TVMBuilder {
    pub cell: TLBRef<TonCell>,
}

#[derive(Debug, Clone, TLB)]
pub enum TVMCont {
    Std(VMContStd),
    Envelope(TVMContEnvelope),
    Quit(VMContQuit),
    QuitExc(TVMContQuitExc),
    Repeat(VMContRepeat),
    Until(VMContUntil),
    Again(VMContAgain),
    WhileCond(VMContWhileCond),
    WhileBody(VMContWhileBody),
    PushInt(VMContPushInt),
}

#[derive(Debug, Clone, TLB)]
pub struct VMControlData {
    #[tlb(bits_len = 13)]
    pub nargs: Option<u16>,
    pub stack: Option<Arc<TVMStack>>,
    pub save: VMSaveList,
    pub cp: Option<i16>,
}

#[derive(Debug, Clone, TLB)]
pub struct VMSaveList {
    #[tlb(adapter = "TLBHashMap::<DictKeyAdapterUint<_>, DictValAdapterTLB<_>>::new(4)")]
    pub cregs: HashMap<u8, TVMStackValue>,
}

#[derive(Debug, Clone, TLB)]
#[tlb(prefix = 0x00, bits_len = 8)]
pub struct VMContStd {
    pub data: Arc<VMControlData>,
    pub code: Arc<TVMCellSlice>,
}

#[derive(Debug, Clone, TLB)]
#[tlb(prefix = 0x01, bits_len = 8)]
pub struct TVMContEnvelope {
    pub data: VMControlData,
    pub next: Arc<TLBRef<TVMCont>>,
}

#[derive(Debug, Clone, TLB)]
#[tlb(prefix = 0x1000, bits_len = 16)]
pub struct VMContQuit {
    pub exit_code: i32,
}

#[derive(Debug, Clone, TLB)]
#[tlb(prefix = 0x1001, bits_len = 16)]
pub struct TVMContQuitExc {}

#[derive(Debug, Clone, TLB)]
#[tlb(prefix = 0x10100, bits_len = 20)]
pub struct VMContRepeat {
    #[tlb(bits_len = 63)]
    pub count: u64,
    pub body: Arc<TLBRef<TVMCont>>,
    pub after: Arc<TLBRef<TVMCont>>,
}

#[derive(Debug, Clone, TLB)]
#[tlb(prefix = 0x110000, bits_len = 24)]
pub struct VMContUntil {
    pub body: Arc<TLBRef<TVMCont>>,
    pub after: Arc<TLBRef<TVMCont>>,
}

#[derive(Debug, Clone, TLB)]
#[tlb(prefix = 0x110001, bits_len = 24)]
pub struct VMContAgain {
    pub body: Arc<TLBRef<TVMCont>>,
}

#[derive(Debug, Clone, TLB)]
#[tlb(prefix = 0x110010, bits_len = 24)]
pub struct VMContWhileCond {
    pub cond: Arc<TLBRef<TVMCont>>,
    pub body: Arc<TLBRef<TVMCont>>,
    pub after: Arc<TLBRef<TVMCont>>,
}

#[derive(Debug, Clone, TLB)]
#[tlb(prefix = 0x110011, bits_len = 24)]
pub struct VMContWhileBody {
    pub cond: Arc<TLBRef<TVMCont>>,
    pub body: Arc<TLBRef<TVMCont>>,
    pub after: Arc<TLBRef<TVMCont>>,
}

#[derive(Debug, Clone, TLB)]
#[tlb(prefix = 0x1111, bits_len = 16)]
pub struct VMContPushInt {
    pub value: i32,
    pub next: Arc<TLBRef<TVMCont>>,
}

impl Debug for TVMStackValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { write!(f, "{self}") }
}

impl Display for TVMStackValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use Deref;
        match self {
            TVMStackValue::Null(_) => write!(f, "Null"),
            TVMStackValue::TinyInt(v) => write!(f, "TinyInt({})", v.value),
            TVMStackValue::Int(v) => write!(f, "Int({})", v.value),
            TVMStackValue::Nan(_) => write!(f, "Nan"),
            TVMStackValue::Cell(v) => write!(f, "Cell({})", v.value.deref()),
            TVMStackValue::CellSlice(v) => write!(f, "CellSlice({})", v.value),
            TVMStackValue::Builder(_) => write!(f, "Builder"),
            TVMStackValue::Cont(_) => write!(f, "Cont"),
            TVMStackValue::Tuple(v) => write!(f, "Tuple[{v:?}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ton_lib_core::traits::tlb::TLB;

    #[test]
    fn test_tvm_tiny_int_serialization() -> anyhow::Result<()> {
        let tiny_int = TVMTinyInt { value: 1 };
        let cell = tiny_int.to_cell()?;

        eprintln!("TinyInt cell BOC: {}", cell.to_boc_base64()?);

        let parsed = TVMTinyInt::from_cell(&cell)?;
        eprintln!("Parsed value: {}", parsed.value);

        assert_eq!(parsed.value, 1);
        Ok(())
    }

    #[test]
    fn test_tvm_stack_value_enum_serialization() -> anyhow::Result<()> {
        let stack_value = TVMStackValue::TinyInt(TVMTinyInt { value: 1 });
        let cell = stack_value.to_cell()?;

        eprintln!("TVMStackValue cell BOC: {}", cell.to_boc_base64()?);

        let parsed = TVMStackValue::from_cell(&cell)?;
        eprintln!("Parsed value: {:?}", parsed);

        match parsed {
            TVMStackValue::TinyInt(val) => assert_eq!(val.value, 1),
            _ => panic!("Expected TinyInt, got {:?}", parsed),
        }
        Ok(())
    }
}

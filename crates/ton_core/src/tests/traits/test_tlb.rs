use super::*;
use tokio_test::{assert_err, assert_ok};
use ton_macros::TLB;

#[derive(Debug)]
struct FailingTLBObjWithRef(u8);

impl TLB for FailingTLBObjWithRef {
    fn read_definition(parser: &mut CellParser) -> TonCoreResult<Self> {
        parser.read_next_ref()?;
        let byte = u8::read_definition(parser)?;
        if byte == 0x01 { bail_ton_core_data!("Failed reading definition") } else { Ok(FailingTLBObjWithRef(byte)) }
    }

    fn write_definition(&self, builder: &mut CellBuilder) -> TonCoreResult<()> {
        builder.write_ref(TonCell::empty().clone())?;
        self.0.write_definition(builder)
    }
}

#[test]
fn test_tlb_derive() -> anyhow::Result<()> {
    #[derive(TLB)]
    #[tlb(prefix = 0x01, bits_len = 8)]
    struct TestTLBObject(u32);
    let cell = TestTLBObject(42).to_cell()?;
    let parsed = TestTLBObject::from_cell(&cell)?;
    assert_eq!(parsed.0, 42);

    let parser = &mut cell.parser();
    let prefix: u32 = parser.read_num(8)?;
    assert_eq!(prefix, 0x01);

    let data: u32 = parser.read_num(32)?;
    assert_eq!(data, 42);
    Ok(())
}

#[test]
fn test_tlb_derive_const_prefix() -> anyhow::Result<()> {
    const PREFIX: usize = 0x02;

    #[derive(TLB)]
    #[tlb(prefix = PREFIX, bits_len = 8)]
    struct TestTLBObject(u32);
    let cell = TestTLBObject(42).to_cell()?;
    let parsed = TestTLBObject::from_cell(&cell)?;
    assert_eq!(parsed.0, 42);
    Ok(())
}

#[test]
fn test_failing_definition_read_restores_refs() -> anyhow::Result<()> {
    let valid = FailingTLBObjWithRef(0);
    let cell = valid.to_cell()?;
    let mut parser = cell.parser();
    let position = parser.get_position()?;

    assert_ok!(FailingTLBObjWithRef::read(&mut parser));
    let after_ok = parser.get_position()?;
    assert_eq!(after_ok.bits_offset, position.bits_offset + 8);
    assert_eq!(after_ok.next_ref_pos, position.next_ref_pos + 1);

    let invalid = FailingTLBObjWithRef(1);
    let cell = invalid.to_cell()?;
    let mut parser = cell.parser();
    let parser_position = parser.get_position()?;

    assert_err!(FailingTLBObjWithRef::read(&mut parser));
    let after_err = parser.get_position()?;
    assert_eq!(after_err, parser_position);

    Ok(())
}

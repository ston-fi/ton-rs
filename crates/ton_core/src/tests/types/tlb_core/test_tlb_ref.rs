use super::*;
use ton_macros::TLB;

#[derive(TLB, PartialEq, Debug)]
struct TestStruct {
    pub a: TLBRef<u8>,
    pub b: TLBRef<u8>,
}

#[test]
fn test_tlb_ref_opt_derive() -> anyhow::Result<()> {
    let expected = TestStruct {
        a: 255.into(),
        b: 255.into(),
    };
    let cell = expected.to_cell()?;
    assert_eq!(cell.refs().len(), 2);
    assert_eq!(cell.refs()[0].underlying_storage(), vec![255]);
    assert_eq!(cell.refs()[1].underlying_storage(), vec![255]);
    let parsed = TestStruct::from_cell(&cell)?;
    assert_eq!(parsed, expected);
    Ok(())
}

#[test]
fn test_tlb_ref_serde_as_cell() -> anyhow::Result<()> {
    let mut cell_builder = TonCell::builder();
    cell_builder.write_bits([0b00110000, 0b00111001], 16)?;
    let cell = cell_builder.build()?;
    let cell_ref = TLBRef::new(cell.clone());
    assert_eq!(cell, cell_ref.clone().into_inner());

    assert_eq!(cell, cell_ref.to_cell()?);

    let cell_boc = cell.to_boc()?;
    let cell_ref_boc = cell_ref.to_boc()?;
    assert_eq!(cell_boc, cell_ref_boc);

    let parsed_ref = TLBRef::<TonCell>::from_boc(cell_boc)?;
    assert_eq!(parsed_ref.into_inner(), cell);
    let parsed_cell = TonCell::from_boc(cell_ref_boc)?;
    assert_eq!(parsed_cell, cell);
    Ok(())
}

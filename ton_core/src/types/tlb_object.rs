use crate::cell::TonCellRef;
use crate::traits::tlb::TLB;

// use crate::cell::build_parse::builder::CellBuilder;
// use crate::cell::build_parse::parser::CellParser;
// use crate::cell::ton_cell::TonCell;
// use crate::errors::TonLibError;
//
/// Can be use for Lazy loading of TLB objects
#[allow(unused)]
#[derive(Debug, Clone)]
pub struct TLBObject<T: TLB> {
    plain: Option<T>,
    cell_ref: Option<TonCellRef>,
}
//
// impl<T: TLBType> TLBObject<T> {
//     pub fn from_cell(cell: &TonCell) -> Self {
//         Self {
//             plain: None,
//             cell: Some(cell.clone()),
//         }
//     }
//
//     pub fn from_plain(plain: T) -> Self {
//         Self {
//             plain: Some(plain),
//             cell: None,
//         }
//     }
//
//     pub fn read(&self) -> Result<T, TonLibError> {
//         if let Some(plain) = &self.plain {
//             return Ok(plain.clone());
//         }
//         if let Some(cell) = &self.cell {
//             return T::from_cell(cell);
//         }
//         Err(TonLibError::TLBObjectNoValue("TLBObject::read".to_string()))
//     }
//
//     pub fn read_inplace(&mut self) -> Result<&T, TonLibError> {
//         if self.plain.is_none() {
//             self.plain = Some(self.read()?);
//         }
//         Ok(self.plain.as_ref().unwrap())
//     }
// }
//
// impl<T: TLBType> TLBType for TLBObject<T> {
//     fn read_definition(parser: &mut CellParser) -> Result<Self, TonLibError> {
//         let cell = parser.read_cell()?;
//         Ok(Self {
//             plain: None,
//             cell: Some(cell),
//         })
//     }
//
//     fn write_definition(&self, builder: &mut CellBuilder) -> Result<(), TonLibError> {
//         if let Some(cell) = &self.cell {
//             return cell.write(builder);
//         }
//         if let Some(plain) = &self.plain {
//             return plain.write(builder);
//         }
//         Err(TonLibError::TLBObjectNoValue("TLBObject::<TLBType>::write_def".to_string()))
//     }
// }

use crate::cell::cell_meta::CellMeta;
use crate::cell::cell_meta::CellType;
use crate::cell::ton_hash::TonHash;
use crate::cell::{CellBuilder, CellParser, LevelMask};
use crate::errors::TonCoreError;
use smallvec::SmallVec;
use std::ops::Deref;
use std::sync::{Arc, LazyLock};

/// ```rust
/// use ton_lib_core::cell::TonCell;
/// let mut builder = TonCell::builder();
/// builder.write_bits([1,2,3], 24).unwrap();
/// let cell = builder.build().unwrap();
/// assert_eq!(cell.data, vec![1, 2, 3]);
/// let mut parser = cell.parser();
/// let data = parser.read_bits(24).unwrap();
/// assert_eq!(data, [1, 2, 3]);
/// ```
#[derive(Clone)]
pub struct TonCell {
    pub(super) cell_data: Arc<CellData>,
    pub(super) borders: CellBorders, // absolute borders for cell_data
    pub(super) meta: Arc<CellMeta>,
}

impl TonCell {
    pub const MAX_DATA_BITS_LEN: usize = 1023;
    pub const MAX_REFS_COUNT: usize = 4;
    pub const EMPTY_CELL_HASH: TonHash = TonHash::from_slice_sized(&[
        150, 162, 150, 210, 36, 242, 133, 198, 123, 238, 147, 195, 15, 138, 48, 145, 87, 240, 218, 163, 93, 197, 184,
        126, 65, 11, 120, 99, 10, 9, 207, 199,
    ]);
    pub const EMPTY_BOC: &'static [u8] = &[181, 238, 156, 114, 1, 1, 1, 1, 0, 2, 0, 0, 0];

    pub fn empty() -> &'static Self { EMPTY_CELL.deref() }

    pub fn builder() -> CellBuilder { CellBuilder::new(CellType::Ordinary) }
    pub fn builder_typed(cell_type: CellType) -> CellBuilder { CellBuilder::new(cell_type) }
    pub fn parser(&self) -> CellParser { CellParser::new(self) }

    pub fn cell_type(&self) -> CellType {
        if self.is_sliced() {
            CellType::Ordinary
        } else {
            self.cell_data.cell_type
        }
    }
    pub fn level_mask(&self) -> LevelMask { self.meta.level_mask(self) }
    pub fn hash(&self) -> Result<&TonHash, TonCoreError> { self.hash_for_level(LevelMask::MAX_LEVEL) }
    pub fn depth(&self) -> Result<u16, TonCoreError> { self.depth_for_level(LevelMask::MAX_LEVEL) }
    pub fn refs(&self) -> &[TonCell] {
        &self.cell_data.refs[self.borders.start_ref as usize..self.borders.end_ref as usize]
    }
    pub fn data_bits_len(&self) -> usize { (self.borders.end_bit - self.borders.start_bit) as usize }

    pub fn hash_for_level(&self, level: LevelMask) -> Result<&TonHash, TonCoreError> {
        self.meta.hash_for_level(self, level)
    }
    pub fn depth_for_level(&self, level: LevelMask) -> Result<u16, TonCoreError> {
        self.meta.depth_for_level(self, level)
    }

    pub(crate) fn underlying_storage(&self) -> &[u8] { &self.cell_data.data_storage }
    fn is_sliced(&self) -> bool {
        self.borders.start_bit != self.cell_data.start_bit
            || self.borders.end_bit as usize != self.cell_data.end_bit as usize
            || self.borders.start_ref != 0
            || self.borders.end_ref as usize != self.cell_data.refs.len()
    }
}

pub(super) type TonCellStorage = SmallVec<[TonCell; TonCell::MAX_REFS_COUNT]>;
pub(super) struct CellData {
    pub cell_type: CellType,
    pub data_storage: Arc<Vec<u8>>, // shared between cell-tree deserialized from BoC
    pub start_bit: u16,             // TODO do we need it?
    pub end_bit: u16,               // exclusive // TODO do we need it?
    pub refs: TonCellStorage,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct CellBorders {
    pub start_bit: u16,
    pub end_bit: u16, // exclusive
    pub start_ref: u8,
    pub end_ref: u8, // exclusive
}

static EMPTY_CELL_DATA: LazyLock<Arc<CellData>> = LazyLock::new(|| {
    Arc::new(CellData {
        cell_type: CellType::Ordinary,
        data_storage: Arc::new(vec![]),
        start_bit: 0,
        end_bit: 0,
        refs: SmallVec::new(),
    })
});

static EMPTY_CELL: LazyLock<TonCell> = LazyLock::new(|| TonCell {
    cell_data: EMPTY_CELL_DATA.to_owned(),
    borders: CellBorders {
        start_bit: 0,
        end_bit: 0,
        start_ref: 0,
        end_ref: 0,
    },
    meta: Arc::new(CellMeta::default()),
});

#[rustfmt::skip]
mod traits_impl {
    use std::fmt::{Debug, Display, Formatter};
    use crate::cell::{TonCell};

    // TonCell
    impl PartialEq for TonCell { fn eq(&self, other: &Self) -> bool { self.hash().is_ok() && other.hash().is_ok() && self.hash().unwrap() == other.hash().unwrap() } }
    impl Eq for TonCell {}
    impl Display for TonCell { fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { f.write_fmt(format_args!("ololo not implemented")) } }
    impl Debug for TonCell { fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { write!(f, "{self}") } }

}

// fn write_cell_display(f: &mut Formatter<'_>, cell: &TonCell, indent_level: usize) -> std::fmt::Result {
//     use std::fmt::Write;
//     let indent = "    ".repeat(indent_level);
//     // Generate the data display string
//     let mut data_display = cell.data.iter().fold(String::new(), |mut res, byte| {
//         let _ = write!(res, "{byte:02X}");
//         res
//     });
//     // completion tag
//     if cell.data_bits_len % 8 != 0 {
//         data_display.push('_');
//     }
//
//     if data_display.is_empty() {
//         data_display.push_str("");
//     };
//
//     if cell.refs.is_empty() {
//         // Compact format for cells without references
//         writeln!(
//             f,
//             "{indent}Cell {{type: {:?}, lm: {}, data: [{data_display}], bit_len: {}, refs ({}): []}}",
//             cell.cell_type,
//             cell.level_mask(),
//             cell.data_bits_len,
//             cell.refs.len()
//         )
//     } else {
//         // Full format for cells with references
//         writeln!(
//             f,
//             "{indent}Cell x{{type: {:?}, lm: {}, data: [{data_display}], bit_len: {}, refs({}): [",
//             cell.cell_type,
//             cell.level_mask(),
//             cell.data_bits_len,
//             cell.refs.len()
//         )?;
//         for i in 0..cell.refs.len() {
//             write_cell_display(f, cell.refs[i].deref(), indent_level + 1)?;
//         }
//         writeln!(f, "{indent}]}}")
//     }
// }
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_ton_cell_create() {
//         let child = TonCell {
//             cell_type: CellType::Ordinary,
//             data: vec![0x01, 0x02, 0x03],
//             data_bits_len: 24,
//             refs: TonCellStorage::new(),
//             meta: CellMeta::EMPTY_CELL_META,
//         }
//         .into_ref();
//
//         let _cell = TonCell {
//             cell_type: CellType::Ordinary,
//             data: vec![0x04, 0x05, 0x06],
//             data_bits_len: 24,
//             refs: TonCellStorage::from([child]),
//             meta: CellMeta::EMPTY_CELL_META,
//         };
//     }
// }

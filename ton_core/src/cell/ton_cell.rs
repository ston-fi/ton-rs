use crate::cell::meta::CellMeta;
use crate::cell::meta::CellType;
use crate::cell::meta::LevelMask;
use crate::cell::ton_hash::TonHash;
use crate::cell::CellBuilder;
use crate::cell::CellParser;
use crate::errors::TonCoreError;
use std::fmt::Formatter;
use std::ops::Deref;
use std::sync::Arc;

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
    pub cell_type: CellType,
    pub data: Vec<u8>,
    pub data_bits_len: usize,
    pub refs: TonCellStorage,
    pub(crate) meta: CellMeta,
}

impl TonCell {
    pub const MAX_DATA_BITS_LEN: usize = 1023;
    pub const MAX_REFS_COUNT: usize = 4;
    pub const EMPTY: Self = TonCell {
        cell_type: CellType::Ordinary,
        data: vec![],
        data_bits_len: 0,
        refs: TonCellStorage::new(),
        meta: CellMeta::EMPTY_CELL_META,
    };
    pub const EMPTY_CELL_HASH: TonHash = TonHash::from_slice_sized(&[
        150, 162, 150, 210, 36, 242, 133, 198, 123, 238, 147, 195, 15, 138, 48, 145, 87, 240, 218, 163, 93, 197, 184,
        126, 65, 11, 120, 99, 10, 9, 207, 199,
    ]);
    pub const EMPTY_BOC: &'static [u8] = &[181, 238, 156, 114, 1, 1, 1, 1, 0, 2, 0, 0, 0];

    pub fn builder() -> CellBuilder { CellBuilder::new(CellType::Ordinary) }
    pub fn builder_typed(cell_type: CellType) -> CellBuilder { CellBuilder::new(cell_type) }
    pub fn parser(&'_ self) -> CellParser<'_> { CellParser::new(self) }

    pub fn level_mask(&self) -> LevelMask { self.meta.level_mask(self) }

    pub fn hash(&self) -> Result<&TonHash, TonCoreError> { self.hash_for_level(LevelMask::MAX_LEVEL) }

    pub fn depth(&self) -> Result<u16, TonCoreError> { self.depth_for_level(LevelMask::MAX_LEVEL) }

    pub fn hash_for_level(&self, level: LevelMask) -> Result<&TonHash, TonCoreError> {
        self.meta.hash_for_level(self, level)
    }

    pub fn depth_for_level(&self, level: LevelMask) -> Result<u16, TonCoreError> {
        self.meta.depth_for_level(self, level)
    }

    pub fn into_ref(self) -> TonCellRef { TonCellRef(self.into()) }
}

// TonCelRef
#[derive(Clone, PartialEq, Eq)]
pub struct TonCellRef(Arc<TonCell>);
impl TonCellRef {
    pub fn into_arc(self) -> Arc<TonCell> { self.0 }
}

pub type TonCellStorage = Vec<TonCellRef>;

#[rustfmt::skip]
mod traits_impl {
    use std::fmt::{Debug, Display, Formatter};
    use std::ops::Deref;
    use crate::cell::ton_cell::write_cell_display;
    use crate::cell::{TonCell, TonCellRef};

    // TonCell
    impl PartialEq for TonCell { fn eq(&self, other: &Self) -> bool { self.hash().is_ok() && other.hash().is_ok() && self.hash().unwrap() == other.hash().unwrap() } }
    impl Eq for TonCell {}
    impl Display for TonCell { fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { write_cell_display(f, self, 0) } }
    impl Debug for TonCell { fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { write!(f, "{self}") } }

    // TonCellRef
    impl Deref for TonCellRef { type Target = TonCell; fn deref(&self) -> &Self::Target { &self.0 } }
    impl Display for TonCellRef { fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { write_cell_display(f, self.deref(), 0) } }
    impl Debug for TonCellRef { fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { write!(f, "{self}") } }
}

fn write_cell_display(f: &mut Formatter<'_>, cell: &TonCell, indent_level: usize) -> std::fmt::Result {
    use std::fmt::Write;
    let indent = "    ".repeat(indent_level);
    // Generate the data display string
    let mut data_display = cell.data.iter().fold(String::new(), |mut res, byte| {
        let _ = write!(res, "{byte:02X}");
        res
    });
    // completion tag
    if cell.data_bits_len % 8 != 0 {
        data_display.push('_');
    }

    if data_display.is_empty() {
        data_display.push_str("");
    };

    if cell.refs.is_empty() {
        // Compact format for cells without references
        writeln!(
            f,
            "{indent}Cell {{type: {:?}, lm: {}, data: [{data_display}], bit_len: {}, refs ({}): []}}",
            cell.cell_type,
            cell.level_mask(),
            cell.data_bits_len,
            cell.refs.len()
        )
    } else {
        // Full format for cells with references
        writeln!(
            f,
            "{indent}Cell x{{type: {:?}, lm: {}, data: [{data_display}], bit_len: {}, refs({}): [",
            cell.cell_type,
            cell.level_mask(),
            cell.data_bits_len,
            cell.refs.len()
        )?;
        for i in 0..cell.refs.len() {
            write_cell_display(f, cell.refs[i].deref(), indent_level + 1)?;
        }
        writeln!(f, "{indent}]}}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_owned_create() {
        let child = TonCell {
            cell_type: CellType::Ordinary,
            data: vec![0x01, 0x02, 0x03],
            data_bits_len: 24,
            refs: TonCellStorage::new(),
            meta: CellMeta::EMPTY_CELL_META,
        }
        .into_ref();

        let _cell = TonCell {
            cell_type: CellType::Ordinary,
            data: vec![0x04, 0x05, 0x06],
            data_bits_len: 24,
            refs: TonCellStorage::from([child]),
            meta: CellMeta::EMPTY_CELL_META,
        };
    }
}

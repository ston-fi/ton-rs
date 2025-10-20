use crate::bail_ton_core_data;
use crate::bits_utils::BitsUtils;
use crate::cell::cell_builder::INITIAL_STORAGE_CAPACITY;
use crate::cell::cell_meta::CellMeta;
use crate::cell::cell_meta::CellType;
use crate::cell::ton_hash::TonHash;
use crate::cell::{CellBuilder, CellParser, LevelMask};
use crate::errors::TonCoreError;
use bitstream_io::{BigEndian, BitReader, BitWriter, ByteReader};
use smallvec::SmallVec;
use std::fmt::Formatter;
use std::io::Cursor;
use std::ops::Deref;
use std::sync::{Arc, LazyLock};

/// ```rust
/// let mut builder = ton_core::cell::TonCell::builder();
/// builder.write_bits([1,2,3], 24).unwrap();
/// let cell = builder.build().unwrap();
/// assert_eq!(cell.data_len_bits(), 24);
/// let mut parser = cell.parser();
/// let data = parser.read_bits(24).unwrap();
/// assert_eq!(data, [1, 2, 3]);
/// ```
#[derive(Clone)]
pub struct TonCell {
    pub(super) cell_type: CellType,
    pub(super) cell_data: Arc<CellData>,
    pub(super) borders: CellBorders, // absolute borders of cell_data
    pub(super) meta: Arc<CellMeta>,
}

impl TonCell {
    pub const MAX_DATA_LEN_BITS: usize = 1023;
    pub const MAX_REFS_COUNT: usize = 4;
    pub const EMPTY_CELL_HASH: TonHash = TonHash::from_slice_sized(&[
        150, 162, 150, 210, 36, 242, 133, 198, 123, 238, 147, 195, 15, 138, 48, 145, 87, 240, 218, 163, 93, 197, 184,
        126, 65, 11, 120, 99, 10, 9, 207, 199,
    ]);
    pub const EMPTY_BOC: &'static [u8] = &[181, 238, 156, 114, 1, 1, 1, 1, 0, 2, 0, 0, 0];

    pub fn empty() -> &'static Self { EMPTY_CELL.deref() }

    pub fn builder() -> CellBuilder { CellBuilder::new(CellType::Ordinary, INITIAL_STORAGE_CAPACITY) }
    pub fn builder_extra(cell_type: CellType, initial_capacity: usize) -> CellBuilder {
        CellBuilder::new(cell_type, initial_capacity)
    }

    // Borders are relative to origin cell
    pub fn slice(&self, borders: CellBorders) -> Result<Self, TonCoreError> {
        let new_cell_borders = CellBorders {
            start_bit: borders.start_bit + self.borders.start_bit,
            end_bit: borders.end_bit + self.borders.start_bit,
            start_ref: borders.start_ref + self.borders.start_ref,
            end_ref: borders.end_ref + self.borders.start_ref,
        };

        if new_cell_borders.end_bit > self.borders.end_bit || new_cell_borders.end_ref > self.borders.end_ref {
            bail_ton_core_data!(
                "Can't build slice:\nslice_borders={:?}\ncell_borders={:?}\nnew_cell_borders={:?}",
                borders,
                self.borders,
                new_cell_borders
            );
        }

        let (cell_type, meta) = if new_cell_borders == self.borders {
            (self.cell_type, self.meta.clone())
        } else {
            (CellType::Ordinary, Arc::new(CellMeta::default()))
        };
        Ok(TonCell {
            cell_type,
            cell_data: self.cell_data.clone(),
            borders: new_cell_borders,
            meta,
        })
    }

    pub fn parser(&'_ self) -> CellParser<'_> { CellParser::new(self) }

    pub fn cell_type(&self) -> CellType { self.cell_type }
    pub fn level_mask(&self) -> LevelMask { self.meta.level_mask(self) }
    pub fn hash(&self) -> Result<&TonHash, TonCoreError> { self.hash_for_level(LevelMask::MAX_LEVEL) }
    pub fn depth(&self) -> Result<u16, TonCoreError> { self.depth_for_level(LevelMask::MAX_LEVEL) }
    pub fn refs(&self) -> &[TonCell] {
        &self.cell_data.refs[self.borders.start_ref as usize..self.borders.end_ref as usize]
    }
    pub fn data_len_bits(&self) -> usize { self.borders.end_bit - self.borders.start_bit }

    pub fn hash_for_level(&self, level: LevelMask) -> Result<&TonHash, TonCoreError> {
        self.meta.hash_for_level(self, level)
    }
    pub fn depth_for_level(&self, level: LevelMask) -> Result<u16, TonCoreError> {
        self.meta.depth_for_level(self, level)
    }

    #[cfg(test)]
    pub(crate) fn underlying_storage(&self) -> &[u8] { &self.cell_data.data_storage }
}

pub(super) type RefStorage = SmallVec<[TonCell; TonCell::MAX_REFS_COUNT]>;

pub(super) struct CellData {
    pub data_storage: Arc<Vec<u8>>, // shared between cell-tree deserialized from BoC
    pub refs: RefStorage,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CellBorders {
    pub start_bit: usize,
    pub end_bit: usize, // exclusive
    pub start_ref: u8,
    pub end_ref: u8, // exclusive
}

static EMPTY_CELL: LazyLock<TonCell> = LazyLock::new(|| TonCell {
    cell_type: CellType::Ordinary,
    cell_data: Arc::new(CellData {
        data_storage: Arc::new(vec![]),
        refs: RefStorage::new(),
    }),
    borders: CellBorders {
        start_bit: 0,
        end_bit: 0,
        start_ref: 0,
        end_ref: 0,
    },
    meta: Arc::new(CellMeta::default()),
});

pub(super) type CellBytesReader<'a> = ByteReader<Cursor<&'a [u8]>, BigEndian>;
pub(super) type CellBitsReader<'a> = BitReader<Cursor<&'a [u8]>, BigEndian>;
pub(super) type CellBitWriter = BitWriter<Vec<u8>, BigEndian>;

#[rustfmt::skip]
mod traits_impl {
    use std::fmt::{Debug, Display, Formatter};
    use crate::cell::{TonCell};
    use crate::cell::ton_cell::write_cell_display;

    // TonCell
    impl PartialEq for TonCell { fn eq(&self, other: &Self) -> bool { self.hash().is_ok() && other.hash().is_ok() && self.hash().unwrap() == other.hash().unwrap() } }
    impl Eq for TonCell {}
    impl Display for TonCell { fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { write_cell_display(f, self, 0) } }
    // expensive
    impl Debug for TonCell { fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { write!(f, "{self}") } }
}

fn write_cell_display(f: &mut Formatter<'_>, cell: &TonCell, indent_level: usize) -> std::fmt::Result {
    use std::fmt::Write;
    let indent = "    ".repeat(indent_level);
    let mut cell_data = vec![0; cell.data_len_bits().div_ceil(8)];
    BitsUtils::read_with_offset(
        &cell.cell_data.data_storage,
        &mut cell_data,
        cell.borders.start_bit,
        cell.data_len_bits(),
    );
    // Generate the data display string
    let mut data_display = cell_data.iter().fold(String::new(), |mut res, byte| {
        let _ = write!(res, "{byte:02X}");
        res
    });
    // completion tag
    if cell.data_len_bits() % 8 != 0 {
        data_display.push('_');
    }

    if data_display.is_empty() {
        data_display.push_str("");
    };

    if cell.refs().is_empty() {
        // Compact format for cells without references
        writeln!(
            f,
            "{indent}Cell {{type: {:?}, lm: {}, data: [{data_display}], bit_len: {}, refs ({}): []}}",
            cell.cell_type,
            cell.level_mask(),
            cell.data_len_bits(),
            cell.refs().len()
        )
    } else {
        // Full format for cells with references
        writeln!(
            f,
            "{indent}Cell x{{type: {:?}, lm: {}, data: [{data_display}], bit_len: {}, refs({}): [",
            cell.cell_type,
            cell.level_mask(),
            cell.data_len_bits(),
            cell.refs().len()
        )?;
        for cell_ref in cell.refs() {
            write_cell_display(f, cell_ref, indent_level + 1)?;
        }
        writeln!(f, "{indent}]}}")
    }
}

#[cfg(test)]
mod tests {
    use crate::cell::{CellBorders, TonCell};

    #[test]
    fn test_ton_cell_slice() -> anyhow::Result<()> {
        let mut builder = TonCell::builder();
        builder.write_bits([1, 2, 3], 24)?;

        for i in 0..4 {
            let mut c_builder = TonCell::builder();
            c_builder.write_num(&i, 8)?;
            builder.write_ref(c_builder.build()?)?;
        }
        let cell = builder.build()?;
        assert_eq!(cell.underlying_storage(), &[1, 2, 3]);
        assert_eq!(cell.data_len_bits(), 24);
        assert_eq!(cell.refs().len(), 4);

        let slice = cell.slice(CellBorders {
            start_bit: 8,
            end_bit: 16,
            start_ref: 1,
            end_ref: 3,
        })?;
        assert_eq!(slice.underlying_storage(), &[1, 2, 3]);
        assert_eq!(slice.data_len_bits(), 8);
        assert_eq!(slice.refs().len(), 2);
        assert_eq!(slice.refs()[0].underlying_storage(), &[1]);
        assert_eq!(slice.refs()[1].underlying_storage(), &[2]);
        Ok(())
    }
}

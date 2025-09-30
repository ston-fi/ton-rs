use crate::cell::{CellType, LevelMask};

pub const GENERIC_BOC_MAGIC: u32 = 0xb5ee9c72;
/// `cells` must be topologically sorted.
#[derive(PartialEq, Debug, Clone)]
pub struct RawBoC {
    pub cells: Vec<RawCell>,
    pub roots_position: Vec<usize>,
}

/// References are stored as indices in BagOfCells.
#[derive(PartialEq, Debug, Clone)]
pub struct RawCell {
    pub cell_type: CellType,
    pub data: Vec<u8>,
    pub data_bits_len: usize,
    pub refs_positions: Vec<usize>,
    pub level_mask: LevelMask,
}

mod cell_meta_builder;
mod cell_type;
mod level_mask;

pub use cell_type::*;
pub use level_mask::*;

use crate::cell::cell_meta::cell_meta_builder::CellMetaBuilder;
use crate::cell::ton_cell::TonCell;
use crate::cell::ton_hash::TonHash;
use crate::errors::TonCoreError;
use once_cell;
use once_cell::sync::OnceCell;
use smallvec::SmallVec;
use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq)]
pub struct CellMeta {
    pub(crate) level_mask: OnceCell<LevelMask>,
    pub(crate) hashes_depths: OnceCell<HashesDepthsStorage>,
}

impl CellMeta {
    pub(crate) const DEPTH_BYTES: usize = 2;
    pub(crate) fn validate(&self, cell: &TonCell) -> Result<(), TonCoreError> { CellMetaBuilder::new(cell).validate() }

    pub(crate) fn level_mask(&self, cell: &TonCell) -> LevelMask {
        *cell.meta.level_mask.get_or_init(|| {
            let cell_refs = cell.refs();
            let mut queue = VecDeque::with_capacity(cell_refs.len());
            for cell_ref in cell_refs.iter().filter(|x| !x.meta.level_initialized()) {
                queue.push_back((cell_ref, 0));
            }

            while let Some((cur_cell, cur_ref_pos)) = queue.pop_front() {
                if let Some(child) = cur_cell.refs().get(cur_ref_pos) {
                    queue.push_front((cur_cell, cur_ref_pos + 1));
                    if !child.meta.level_initialized() {
                        queue.push_front((child, 0));
                    }
                } else {
                    let _ = cur_cell.level_mask();
                }
            }
            CellMetaBuilder::new(cell).calc_level_mask()
        })
    }

    pub(crate) fn hash_for_level(&self, cell: &TonCell, level: LevelMask) -> Result<&TonHash, TonCoreError> {
        let hashes = self.get_hashes_depths(cell)?.0;
        Ok(&hashes[level.mask() as usize])
    }
    pub(crate) fn depth_for_level(&self, cell: &TonCell, level: LevelMask) -> Result<u16, TonCoreError> {
        let depths = &self.get_hashes_depths(cell)?.1;
        Ok(depths[level.mask() as usize])
    }

    fn get_hashes_depths(&self, cell: &TonCell) -> Result<(&[TonHash], &[u16]), TonCoreError> {
        let data = self.hashes_depths.get_or_try_init(|| {
            let level_mask = self.level_mask(cell);
            let cell_refs = cell.refs();
            let mut queue = VecDeque::with_capacity(cell_refs.len());
            for cell_ref in cell.refs() {
                if !cell_ref.meta.hash_initialized() {
                    queue.push_back((cell_ref, 0));
                }
            }
            while let Some((cur_cell, cur_ref_pos)) = queue.pop_front() {
                if let Some(child) = cur_cell.refs().get(cur_ref_pos) {
                    queue.push_front((cur_cell, cur_ref_pos + 1));
                    if !child.meta.hash_initialized() {
                        queue.push_front((child, 0));
                    }
                } else {
                    let _ = cur_cell.hash()?; // just to calc it
                }
            }
            CellMetaBuilder::new(cell).calc_hashes_and_depths(level_mask)
        });
        data.map(|(hashes, depths)| (hashes.as_slice(), depths.as_slice()))
    }

    fn level_initialized(&self) -> bool { self.level_mask.get().is_some() }

    fn hash_initialized(&self) -> bool { self.hashes_depths.get().is_some() }
}

impl Default for CellMeta {
    fn default() -> Self {
        Self {
            level_mask: OnceCell::new(),
            hashes_depths: OnceCell::new(),
        }
    }
}

pub(super) type HashesDepthsStorage = (SmallVec<[TonHash; 4]>, SmallVec<[u16; 4]>);

// static EMPTY_CELL_META: LazyLock<Arc<CellMeta>> = LazyLock::new(|| {
//     Arc::new(CellMeta {
//         level_mask: LevelMask::new(0).into(),
//         hashes_depths: (SmallVec::from_elem(TonCell::EMPTY_CELL_HASH, 4), SmallVec::from_elem(0, 4)).into(),
//     })
// });

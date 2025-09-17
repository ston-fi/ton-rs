use crate::cell::meta::cell_meta_builder::CellMetaBuilder;
use crate::cell::meta::level_mask::LevelMask;
use crate::cell::ton_cell::TonCell;
use crate::cell::ton_hash::TonHash;
use crate::errors::TonCoreError;
use once_cell;
use once_cell::sync::OnceCell;
use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CellMeta {
    level_mask: LazyLevelMask,
    hashes_depths: LazyHashesDepths,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum LazyLevelMask {
    Static(LevelMask),
    Dynamic(OnceCell<LevelMask>),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum LazyHashesDepths {
    Static(([TonHash; 4], [u16; 4])),
    Dynamic(OnceCell<(Vec<TonHash>, Vec<u16>)>),
}

impl CellMeta {
    pub(crate) const DEPTH_BYTES: usize = 2;

    pub(crate) const EMPTY_CELL_META: CellMeta = CellMeta {
        level_mask: LazyLevelMask::Static(LevelMask::new(0)),
        hashes_depths: LazyHashesDepths::Static(([TonCell::EMPTY_CELL_HASH; 4], [0; 4])),
    };

    pub(crate) fn validate(&self, cell: &TonCell) -> Result<(), TonCoreError> { CellMetaBuilder::new(cell).validate() }

    pub(crate) fn level_mask(&self, cell: &TonCell) -> LevelMask {
        match &self.level_mask {
            LazyLevelMask::Static(mask) => *mask,
            LazyLevelMask::Dynamic(once) => {
                if let Some(lm) = once.get() {
                    return *lm;
                }
                *once.get_or_init(|| {
                    let mut queue = VecDeque::with_capacity(cell.refs.len());
                    for cell_ref in cell.refs.iter().filter(|x| x.meta.level_initialized()) {
                        if !cell_ref.meta.level_initialized() {
                            queue.push_back((cell_ref, 0));
                        }
                    }
                    while !queue.is_empty() {
                        let (cur_cell, cur_ref) = queue.pop_front().unwrap();
                        if let Some(child) = cur_cell.refs.get(cur_ref) {
                            queue.push_front((cur_cell, cur_ref + 1));
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
        }
    }

    pub(crate) fn hash_for_level(&self, cell: &TonCell, level: LevelMask) -> Result<&TonHash, TonCoreError> {
        let hashes = &self.get_hashes_depths(cell)?.0;
        Ok(&hashes[level.mask() as usize])
    }
    pub(crate) fn depth_for_level(&self, cell: &TonCell, level: LevelMask) -> Result<u16, TonCoreError> {
        let depths = &self.get_hashes_depths(cell)?.1;
        Ok(depths[level.mask() as usize])
    }

    fn get_hashes_depths(&self, cell: &TonCell) -> Result<(&[TonHash], &[u16]), TonCoreError> {
        let data = match &self.hashes_depths {
            LazyHashesDepths::Static(hashes_depths) => return Ok((&hashes_depths.0, &hashes_depths.1)),
            LazyHashesDepths::Dynamic(once) => {
                if let Some(data) = once.get() {
                    return Ok((data.0.as_slice(), data.1.as_slice()));
                }

                once.get_or_try_init(|| {
                    let level_mask = self.level_mask(cell);
                    let mut queue = VecDeque::with_capacity(cell.refs.len());
                    for cell_ref in cell.refs.iter().filter(|x| x.meta.hash_initialized()) {
                        queue.push_back((cell_ref, 0));
                    }
                    while !queue.is_empty() {
                        let (cur_cell, cur_ref) = queue.pop_front().unwrap();
                        if let Some(child) = cur_cell.refs.get(cur_ref) {
                            queue.push_front((cur_cell, cur_ref + 1));
                            if !child.meta.hash_initialized() {
                                queue.push_front((child, 0));
                            }
                        } else {
                            let _ = cur_cell.hash()?;
                        }
                    }
                    CellMetaBuilder::new(cell).calc_hashes_and_depths(level_mask)
                })
            }
        };
        data.map(|(hashes, depths)| (hashes.as_slice(), depths.as_slice()))
    }

    fn level_initialized(&self) -> bool {
        match &self.hashes_depths {
            LazyHashesDepths::Static(_) => true,
            LazyHashesDepths::Dynamic(once) => once.get().is_some(),
        }
    }

    fn hash_initialized(&self) -> bool {
        match &self.level_mask {
            LazyLevelMask::Static(_) => true,
            LazyLevelMask::Dynamic(once) => once.get().is_some(),
        }
    }
}

impl Default for CellMeta {
    fn default() -> Self {
        Self {
            level_mask: LazyLevelMask::Dynamic(OnceCell::new()),
            hashes_depths: LazyHashesDepths::Dynamic(OnceCell::new()),
        }
    }
}

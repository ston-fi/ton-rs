use crate::cell::{CellMeta, TonCell, TonCellRef};
use crate::errors::TonCoreError;

use super::RawBoC;

impl RawBoC {
    //Based on https://github.com/toncenter/tonweb/blob/c2d5d0fc23d2aec55a0412940ce6e580344a288c/src/boc/Cell.js#L198
    pub fn into_ton_cells(self) -> Result<Vec<TonCellRef>, TonCoreError> {
        let cells_len = self.cells.len();
        let mut cells: Vec<TonCellRef> = Vec::with_capacity(cells_len);

        for (cell_index, cell_raw) in self.cells.into_iter().enumerate().rev() {
            let mut refs = Vec::with_capacity(cell_raw.refs_positions.len());
            for ref_index in cell_raw.refs_positions {
                if ref_index <= cell_index {
                    return Err(TonCoreError::Custom("ref to parent cell detected".to_string()));
                }
                refs.push(cells[cells_len - 1 - ref_index].clone());
            }

            let cell_ref = TonCell {
                cell_type: cell_raw.cell_type,
                data: cell_raw.data,
                data_bits_len: cell_raw.data_bits_len,
                refs,
                meta: CellMeta::default(),
            }
            .into_ref();
            cells.push(cell_ref);
        }

        let mut roots = Vec::with_capacity(self.roots_position.len());
        for root_index in self.roots_position {
            roots.push(cells[cells_len - 1 - root_index].clone());
        }
        Ok(roots)
    }
}

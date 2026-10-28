use crate::cell::TonCell;
use crate::cell::TonHash;
use crate::cell::cell_meta::CellType;
use crate::errors::TonCoreError;
use std::collections::{HashSet, VecDeque};

pub struct TonCellUtils;

impl TonCellUtils {
    /// Runs over cell trees, collect library_cells and extracts corresponding lib_ids.
    pub fn extract_lib_ids<'a, I>(cells_iter: I) -> Result<HashSet<TonHash>, TonCoreError>
    where
        I: IntoIterator<Item = &'a TonCell>,
    {
        let mut result = HashSet::new();
        let mut visited = HashSet::new();

        let mut queue = VecDeque::from_iter(cells_iter);

        while let Some(cell) = queue.pop_front() {
            if !visited.insert(cell.hash()?) {
                continue;
            }
            if let Some(lib_id) = Self::read_lib_id(cell)? {
                result.insert(lib_id);
            }
            queue.extend(cell.refs());
        }
        Ok(result)
    }

    // Read lib_ids from library_cell
    pub fn read_lib_id(cell: &TonCell) -> Result<Option<TonHash>, TonCoreError> {
        if cell.cell_type() != CellType::LibraryRef {
            return Ok(None);
        }
        let mut parser = cell.parser();
        parser.read_bits(8)?; // skip exotic tag

        Ok(Some(TonHash::from_vec(parser.read_bits(TonHash::BITS_LEN)?)?))
    }
}

#[cfg(test)]
#[path = "../tests/cell/test_ton_cell_utils.rs"]
mod tests;

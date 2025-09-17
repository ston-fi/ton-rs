use crate::cell::meta::CellType;
use crate::cell::TonCell;
use crate::cell::TonHash;
use crate::errors::TonCoreError;
use std::collections::{HashSet, VecDeque};
use std::ops::Deref;

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
            queue.extend(cell.refs.iter().map(Deref::deref));
        }
        Ok(result)
    }

    // Read lib_ids from library_cell
    pub fn read_lib_id(cell: &TonCell) -> Result<Option<TonHash>, TonCoreError> {
        if cell.cell_type != CellType::LibraryRef {
            return Ok(None);
        }
        Ok(Some(TonHash::from_slice(&cell.data[1..=32])?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::ton_hash::TonHash;
    use crate::traits::tlb::TLB;
    use std::str::FromStr;

    #[test]
    fn test_extract_lib_ids_stonfi_router() -> Result<(), TonCoreError> {
        let expected_lib_id = TonHash::from_str("57DE63D28E4D3608E0C02D437A7B50EF5F28F36A4821A047FD663CE63F4597EC")?;
        let code = TonCell::from_boc_hex(
            "b5ee9c720101010100230008420257de63d28e4d3608e0c02d437a7b50ef5f28f36a4821a047fd663ce63f4597ec",
        )?;
        let cells = vec![&code];
        let lib_ids = TonCellUtils::extract_lib_ids(cells)?;
        assert_eq!(lib_ids, HashSet::from([expected_lib_id.clone()]));

        // check accepted formats
        let code_ref = code.clone().into_ref();
        let cells = vec![code_ref.deref(), &code];
        let lib_ids = TonCellUtils::extract_lib_ids(cells)?;
        assert_eq!(lib_ids, HashSet::from([expected_lib_id.clone()]));
        Ok(())
    }

    #[test]
    fn test_extract_lib_ids_stonfi_pton() -> Result<(), TonCoreError> {
        // https://tonviewer.com/EQBnGWMCf3-FZZq1W4IWcWiGAc3PHuZ0_H-7sad2oY00o83S
        let code = TonCell::from_boc_hex(
            "b5ee9c7201010101002300084202d29017573b8132be742e9c02dabe2311fb3df9f077e661d3ee24d431058b8830",
        )?;
        let data = TonCell::from_boc_hex("b5ee9c7201010301005d000208000000000102084202cd88e6f3c2a9cf01bb003a2837ec0d92c19685ed1dbfffd94a545dcfdf0a14d900600168747470733a2f2f7374617469632e73746f6e2e66692f6a6574746f6e2f746f6e2d70726f78792d76322e6a736f6e")?;
        let cells = vec![&code, &data];
        let lib_ids = TonCellUtils::extract_lib_ids(cells)?.into_iter().collect::<HashSet<_>>();
        assert_eq!(
            lib_ids,
            HashSet::from([
                TonHash::from_str("CD88E6F3C2A9CF01BB003A2837EC0D92C19685ED1DBFFFD94A545DCFDF0A14D9")?,
                TonHash::from_str("D29017573B8132BE742E9C02DABE2311FB3DF9F077E661D3EE24D431058B8830")?
            ])
        );
        Ok(())
    }
}

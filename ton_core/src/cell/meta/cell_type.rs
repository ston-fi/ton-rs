use crate::bail_ton_core_data;
use crate::errors::TonCoreError;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CellType {
    Ordinary,
    PrunedBranch,
    LibraryRef,
    MerkleProof,
    MerkleUpdate,
}

impl CellType {
    // https://docs.ton.org/v3/documentation/data-formats/tlb/exotic-cells
    pub fn new_exotic(byte: u8) -> Result<CellType, TonCoreError> {
        let cell_type = match byte {
            0x01 => Self::PrunedBranch,
            0x02 => Self::LibraryRef,
            0x03 => Self::MerkleProof,
            0x04 => Self::MerkleUpdate,
            _ => bail_ton_core_data!("Unknown exotic type with first byte={byte}"),
        };
        Ok(cell_type)
    }

    pub fn is_exotic(&self) -> bool { self != &CellType::Ordinary }
}

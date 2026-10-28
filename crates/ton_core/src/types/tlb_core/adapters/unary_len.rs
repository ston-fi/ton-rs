use crate::cell::{CellBuilder, CellParser};
use crate::errors::TonCoreError;
use crate::traits::tlb::TLB;
use std::ops::{Deref, DerefMut};

/// <https://github.com/ton-blockchain/ton/blob/ed4682066978f69ffa38dd98912ca77d4f660f66/crypto/block/block.tlb#L33>
///
/// Sequence of N 1-bits followed by a 0-bit, where N is the length
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnaryLen(pub usize);

impl TLB for UnaryLen {
    fn read_definition(parser: &mut CellParser) -> Result<Self, TonCoreError> {
        let mut bits_len = 0;
        while parser.read_bit()? {
            bits_len += 1;
        }
        Ok(UnaryLen(bits_len))
    }

    fn write_definition(&self, dst: &mut CellBuilder) -> Result<(), TonCoreError> {
        for _ in 0..self.0 {
            dst.write_bit(true)?;
        }
        dst.write_bit(false)
    }
}

impl Deref for UnaryLen {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for UnaryLen {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
#[path = "../../../tests/types/tlb_core/adapters/test_unary_len.rs"]
mod tests;

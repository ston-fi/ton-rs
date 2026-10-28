use crate::cell::CellBuilder;
use crate::cell::CellParser;
use crate::errors::TonCoreError;
use crate::traits::tlb::TLB;

/// Either X Y
///
/// <https://github.com/ton-blockchain/ton/blame/cac968f77dfa5a14e63db40190bda549f0eaf746/crypto/block/block.tlb#L10>
#[derive(Clone, Debug, PartialEq)]
pub enum TLBEither<L, R> {
    Left(L),
    Right(R),
}

impl<L: TLB, R: TLB> TLB for TLBEither<L, R> {
    fn read_definition(parser: &mut CellParser) -> Result<Self, TonCoreError> {
        match parser.read_bit()? {
            false => Ok(Self::Left(L::read(parser)?)),
            true => Ok(Self::Right(R::read(parser)?)),
        }
    }

    fn write_definition(&self, builder: &mut CellBuilder) -> Result<(), TonCoreError> {
        match self {
            TLBEither::Left(left) => {
                builder.write_bit(false)?;
                left.write(builder)
            },
            TLBEither::Right(right) => {
                builder.write_bit(true)?;
                right.write(builder)
            },
        }
    }
}

#[cfg(test)]
#[path = "../../tests/types/tlb_core/test_either.rs"]
mod tests;

use crate::cell::CellBuilder;
use crate::cell::CellParser;
use crate::errors::TonCoreError;
use crate::traits::tlb::TLB;

/// Either X ^X
///
/// <https://github.com/ton-blockchain/ton/blame/cac968f77dfa5a14e63db40190bda549f0eaf746/crypto/block/block.tlb#L10>
#[derive(Clone, Debug)]
pub struct TLBEitherRef<T> {
    pub value: T,
    pub layout: EitherRefLayout,
}

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum EitherRefLayout {
    ToCell,
    ToRef,
    Native,
}

impl<T> TLBEitherRef<T> {
    /// Creates a value whose inline/reference layout is chosen during serialization.
    pub fn new(value: T) -> Self {
        Self {
            value,
            layout: EitherRefLayout::Native,
        }
    }

    /// Creates a value that is always serialized by reference.
    pub fn new_ref(value: T) -> Self {
        Self {
            value,
            layout: EitherRefLayout::ToRef,
        }
    }
    /// Creates a value with an explicit serialization layout.
    pub fn new_with_layout(value: T, layout: EitherRefLayout) -> Self {
        Self { value, layout }
    }
    pub fn into_inner(self) -> T {
        self.value
    }
}

impl<T: TLB> TLB for TLBEitherRef<T> {
    fn read_definition(parser: &mut CellParser) -> Result<Self, TonCoreError> {
        let val = match parser.read_bit()? {
            false => TLBEitherRef {
                value: TLB::read(parser)?,
                layout: EitherRefLayout::ToCell,
            },
            true => TLBEitherRef {
                value: TLB::from_cell(parser.read_next_ref()?)?,
                layout: EitherRefLayout::ToRef,
            },
        };
        Ok(val)
    }

    fn write_definition(&self, builder: &mut CellBuilder) -> Result<(), TonCoreError> {
        let cell = self.value.to_cell()?;
        let serial_layout = match self.layout {
            EitherRefLayout::ToCell => EitherRefLayout::ToCell,
            EitherRefLayout::ToRef => EitherRefLayout::ToRef,
            EitherRefLayout::Native => {
                // strictly <, 1 more bit is reserver for layout marker
                if cell.data_len_bits() < builder.data_bits_left() && cell.refs().len() <= builder.refs_left() {
                    EitherRefLayout::ToCell
                } else {
                    EitherRefLayout::ToRef
                }
            },
        };
        match serial_layout {
            EitherRefLayout::ToCell => {
                builder.write_bit(false)?;
                builder.write_cell(&cell)?;
            },
            EitherRefLayout::ToRef => {
                builder.write_bit(true)?;
                builder.write_ref(cell)?;
            },
            _ => unreachable!("Invalid EitherRefLayout value"),
        };
        Ok(())
    }
}

#[rustfmt::skip]
mod traits_impl {
    use std::ops::{Deref, DerefMut};
    use crate::types::tlb_core::TLBEitherRef;

    impl<T> Deref for TLBEitherRef<T> { type Target = T; fn deref(&self) -> &Self::Target { &self.value }}
    impl<T> DerefMut for TLBEitherRef<T> { fn deref_mut(&mut self) -> &mut Self::Target { &mut self.value } }
    impl<T: PartialEq> PartialEq for TLBEitherRef<T> { fn eq(&self, other: &Self) -> bool { self.value == other.value }}
    impl<T> From<T> for TLBEitherRef<T> { fn from(value: T) -> Self { Self::new(value) } }
}

#[cfg(test)]
#[path = "../../tests/types/tlb_core/test_either_ref.rs"]
mod tests;

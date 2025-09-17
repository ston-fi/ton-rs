use crate::cell::CellBuilder;
use crate::cell::CellParser;
use crate::errors::TonCoreError;
use crate::traits::tlb::TLB;
use std::ops::{Deref, DerefMut};

/// Either X ^X
///
/// https://github.com/ton-blockchain/ton/blame/cac968f77dfa5a14e63db40190bda549f0eaf746/crypto/block/block.tlb#L10
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
    pub fn new(value: T) -> Self {
        Self {
            value,
            layout: EitherRefLayout::Native,
        }
    }

    pub fn new_with_layout(value: T, layout: EitherRefLayout) -> Self { Self { value, layout } }
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
                if cell.data_bits_len < builder.data_bits_left() {
                    EitherRefLayout::ToCell
                } else {
                    EitherRefLayout::ToRef
                }
            }
        };
        match serial_layout {
            EitherRefLayout::ToCell => {
                builder.write_bit(false)?;
                builder.write_cell(&cell)?;
            }
            EitherRefLayout::ToRef => {
                builder.write_bit(true)?;
                builder.write_ref(cell.into_ref())?;
            }
            _ => unreachable!("Invalid EitherRefLayout value"),
        };
        Ok(())
    }
}

impl<T> Deref for TLBEitherRef<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target { &self.value }
}
impl<T> DerefMut for TLBEitherRef<T> {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.value }
}
impl<T: PartialEq> PartialEq for TLBEitherRef<T> {
    fn eq(&self, other: &Self) -> bool { self.value == other.value }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::TonCell;
    use tokio_test::assert_ok;
    use ton_lib_macros::TLB;

    #[derive(Debug, PartialEq, TLB)]
    struct TestType1(i32);

    #[derive(Debug, PartialEq, TLB)]
    struct TestType2(i64);

    #[test]
    fn test_either_ref() -> anyhow::Result<()> {
        let obj1 = TLBEitherRef {
            value: TestType1(1),
            layout: EitherRefLayout::ToCell,
        };

        let obj2 = TLBEitherRef {
            value: TestType2(2),
            layout: EitherRefLayout::ToRef,
        };

        let obj3 = TLBEitherRef {
            value: TestType1(3),
            layout: EitherRefLayout::Native,
        };

        let mut builder = TonCell::builder();
        obj1.write(&mut builder)?;
        obj2.write(&mut builder)?;
        obj3.write(&mut builder)?;
        let cell = builder.build()?;
        let mut parser = cell.parser();
        let parsed_obj1 = TLBEitherRef::<TestType1>::read(&mut parser)?;
        let parsed_obj2 = TLBEitherRef::<TestType2>::read(&mut parser)?;
        let parsed_obj3 = TLBEitherRef::<TestType1>::read(&mut parser)?;
        assert_eq!(obj1, parsed_obj1);
        assert_eq!(parsed_obj1.layout, EitherRefLayout::ToCell);
        assert_eq!(obj2, parsed_obj2);
        assert_eq!(parsed_obj2.layout, EitherRefLayout::ToRef);

        assert_eq!(obj3.value, parsed_obj3.value);
        assert_eq!(parsed_obj1.layout, EitherRefLayout::ToCell);
        Ok(())
    }

    #[test]
    fn test_either_ref_recursive() -> anyhow::Result<()> {
        #[derive(Debug, PartialEq, Clone)]
        enum List {
            Empty,
            Some(Box<Item>),
        }

        #[derive(Debug, PartialEq, Clone, TLB)]
        struct Item {
            next: TLBEitherRef<TonCell>,
            number1: u128,
            number2: u128,
            number3: u128,
        }

        impl TLB for List {
            fn read_definition(parser: &mut CellParser) -> Result<Self, TonCoreError> {
                match parser.data_bits_remaining()? {
                    0 => Ok(Self::Empty),
                    _ => Ok(Self::Some(TLB::read(parser)?)),
                }
            }

            fn write_definition(&self, dst: &mut CellBuilder) -> Result<(), TonCoreError> {
                match self {
                    List::Empty => {}
                    List::Some(item) => item.write(dst)?,
                }
                Ok(())
            }
        }

        let new_list = List::Some(Box::from(Item {
            next: TLBEitherRef {
                value: List::Some(Box::from(Item {
                    next: TLBEitherRef {
                        value: List::Empty.to_cell()?,
                        layout: EitherRefLayout::Native,
                    },
                    number1: 1,
                    number2: 1,
                    number3: 1,
                }))
                .to_cell()?,
                layout: EitherRefLayout::Native,
            },
            number1: 1,
            number2: 2,
            number3: 3,
        }));

        assert_ok!(new_list.to_cell());
        Ok(())
    }
}

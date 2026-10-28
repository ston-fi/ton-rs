use std::fmt::Display;
use std::ops::{BitOr, BitOrAssign, Shr, Sub};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd)]
pub struct LevelMask(u8);

impl LevelMask {
    pub const MIN_LEVEL: LevelMask = LevelMask(0);
    pub const MAX_LEVEL: LevelMask = LevelMask(3);

    pub const fn new(mask: u8) -> Self {
        Self(mask)
    }
    pub const fn level(&self) -> u8 {
        8 - self.0.leading_zeros() as u8
    }
    pub const fn mask(&self) -> u8 {
        self.0
    }
    pub const fn hash_index(&self) -> usize {
        self.0.count_ones() as usize
    }
    pub const fn hash_count(&self) -> usize {
        self.hash_index() + 1
    }
    pub const fn is_significant(&self, level: u8) -> bool {
        level == 0 || !(self.0 >> (level - 1)).is_multiple_of(2)
    }
    pub const fn apply(&self, level: u8) -> Self {
        LevelMask(self.0 & ((1u8 << level) - 1))
    }
}

impl From<LevelMask> for u8 {
    fn from(val: LevelMask) -> Self {
        val.0
    }
}

impl<T: Into<u8>> BitOr<T> for LevelMask {
    type Output = Self;
    fn bitor(self, rhs: T) -> Self::Output {
        LevelMask(self.0 | rhs.into())
    }
}

impl<T: Into<u8>> BitOrAssign<T> for LevelMask {
    fn bitor_assign(&mut self, rhs: T) {
        self.0 |= rhs.into();
    }
}

impl<T: Into<u8>> Shr<T> for LevelMask {
    type Output = Self;
    fn shr(self, rhs: T) -> Self::Output {
        LevelMask(self.0 >> rhs.into())
    }
}

impl Display for LevelMask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<T: Into<u8>> Sub<T> for LevelMask {
    type Output = LevelMask;
    fn sub(self, rhs: T) -> Self::Output {
        LevelMask(self.0 - rhs.into())
    }
}

#[cfg(test)]
#[path = "../../tests/cell/cell_meta/test_level_mask.rs"]
mod tests;

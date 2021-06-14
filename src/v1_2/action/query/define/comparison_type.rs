use crate::v1_2::error::define::QueryComparisonTypeError;

pub const INEQUAL: u8 = 0;
pub const EQUAL: u8 = 1;
pub const LESS_THAN: u8 = 2;
pub const LESS_THAN_OR_EQUAL: u8 = 3;
pub const GREATER_THAN: u8 = 4;
pub const GREATER_THAN_OR_EQUAL: u8 = 5;

#[repr(u8)]
#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum QueryComparisonType {
    Inequal = INEQUAL,
    Equal = EQUAL,
    LessThan = LESS_THAN,
    LessThanOrEqual = LESS_THAN_OR_EQUAL,
    GreaterThan = GREATER_THAN,
    GreaterThanOrEqual = GREATER_THAN_OR_EQUAL,
    Rfu6 = 6,
    Rfu7 = 7,
}
impl QueryComparisonType {
    /// # Safety
    /// You are to warrant that n is encoded on 3 bits only.
    /// That means n <= 0x7.
    pub const unsafe fn from_unchecked(n: u8) -> Self {
        match n {
            INEQUAL => Self::Inequal,
            EQUAL => Self::Equal,
            LESS_THAN => Self::LessThan,
            LESS_THAN_OR_EQUAL => Self::LessThanOrEqual,
            GREATER_THAN => Self::GreaterThan,
            GREATER_THAN_OR_EQUAL => Self::GreaterThanOrEqual,
            6 => Self::Rfu6,
            7 => Self::Rfu7,
            // Should never occured if used safely
            _ => Self::Inequal,
        }
    }

    /// # Errors
    /// Returns an error if n > 7
    pub const fn from(n: u8) -> Result<Self, QueryComparisonTypeError> {
        Ok(match n {
            INEQUAL => Self::Inequal,
            EQUAL => Self::Equal,
            LESS_THAN => Self::LessThan,
            LESS_THAN_OR_EQUAL => Self::LessThanOrEqual,
            GREATER_THAN => Self::GreaterThan,
            GREATER_THAN_OR_EQUAL => Self::GreaterThanOrEqual,
            6 => Self::Rfu6,
            7 => Self::Rfu7,
            _ => return Err(QueryComparisonTypeError::Invalid),
        })
    }
}

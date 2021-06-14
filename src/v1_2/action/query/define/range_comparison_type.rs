use crate::v1_2::error::define::QueryRangeComparisonTypeError;

pub const NOT_IN_RANGE: u8 = 0;
pub const IN_RANGE: u8 = 1;

#[repr(u8)]
#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum QueryRangeComparisonType {
    NotInRange = NOT_IN_RANGE,
    InRange = IN_RANGE,
    Rfu2 = 2,
    Rfu3 = 3,
    Rfu4 = 4,
    Rfu5 = 5,
    Rfu6 = 6,
    Rfu7 = 7,
}
impl QueryRangeComparisonType {
    /// # Safety
    /// You are to warrant that n is encoded on 3 bits only.
    /// That means n <= 0x7.
    pub const unsafe fn from_unchecked(n: u8) -> Self {
        match n {
            NOT_IN_RANGE => Self::NotInRange,
            IN_RANGE => Self::InRange,
            2 => Self::Rfu2,
            3 => Self::Rfu3,
            4 => Self::Rfu4,
            5 => Self::Rfu5,
            6 => Self::Rfu6,
            7 => Self::Rfu7,
            // Should never occured if used safely
            _ => Self::NotInRange,
        }
    }

    /// # Errors
    /// Returns an error if n > 7
    pub const fn from(n: u8) -> Result<Self, QueryRangeComparisonTypeError> {
        Ok(match n {
            NOT_IN_RANGE => Self::NotInRange,
            IN_RANGE => Self::InRange,
            2 => Self::Rfu2,
            3 => Self::Rfu3,
            4 => Self::Rfu4,
            5 => Self::Rfu5,
            6 => Self::Rfu6,
            7 => Self::Rfu7,
            _ => return Err(QueryRangeComparisonTypeError::Invalid),
        })
    }
}

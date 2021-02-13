#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum QueryComparisonType {
    Inequal = 0,
    Equal = 1,
    LessThan = 2,
    LessThanOrEqual = 3,
    GreaterThan = 4,
    GreaterThanOrEqual = 5,
    Rfu6 = 6,
    Rfu7 = 7,
}
impl QueryComparisonType {
    /// # Safety
    /// You are to warrant that n is encoded on 3 bits only.
    /// That means n <= 0x7.
    pub const unsafe fn from_unchecked(n: u8) -> Self {
        match n {
            0 => Self::Inequal,
            1 => Self::Equal,
            2 => Self::LessThan,
            3 => Self::LessThanOrEqual,
            4 => Self::GreaterThan,
            5 => Self::GreaterThanOrEqual,
            6 => Self::Rfu6,
            7 => Self::Rfu7,
            // Should never occured if used safely
            _ => Self::Inequal,
        }
    }

    /// # Errors
    /// Returns an error if n > 7
    pub const fn from(n: u8) -> Result<Self, ()> {
        Ok(match n {
            0 => Self::Inequal,
            1 => Self::Equal,
            2 => Self::LessThan,
            3 => Self::LessThanOrEqual,
            4 => Self::GreaterThan,
            5 => Self::GreaterThanOrEqual,
            6 => Self::Rfu6,
            7 => Self::Rfu7,
            _ => return Err(()),
        })
    }
}

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum QueryRangeComparisonType {
    NotInRange = 0,
    InRange = 1,
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
            0 => Self::NotInRange,
            1 => Self::InRange,
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
    pub const fn from(n: u8) -> Result<Self, ()> {
        Ok(match n {
            0 => Self::NotInRange,
            1 => Self::InRange,
            2 => Self::Rfu2,
            3 => Self::Rfu3,
            4 => Self::Rfu4,
            5 => Self::Rfu5,
            6 => Self::Rfu6,
            7 => Self::Rfu7,
            _ => return Err(()),
        })
    }
}

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum QueryCode {
    // NonVoid = 0,
    // ComparisonWithZero = 1,
    ComparisonWithValue = 2,
    // ComparisonWithOtherFile = 3,
    ComparisonWithRange = 4,
    // StringTokenSearch = 7,
}
impl QueryCode {
    /// # Errors
    /// Returns an error if the query code is unknown
    pub const fn from(n: u8) -> Result<Self, ()> {
        #[cfg_attr(not(feature = "decode_query"), allow(unreachable_code))]
        Ok(match n {
            // 0 => QueryCode::NonVoid,
            // 1 => QueryCode::ComparisonWithZero,
            #[cfg(feature = "decode_query_compare_with_value")]
            2 => QueryCode::ComparisonWithValue,
            // 3 => QueryCode::ComparisonWithOtherFile,
            #[cfg(feature = "decode_query_compare_with_range")]
            4 => QueryCode::ComparisonWithRange,
            // 7 => QueryCode::StringTokenSearch,
            // TODO This should be an enumeration of the queries instead of all_queries, in case
            // they are selected manually.
            #[cfg_attr(not(feature = "all_queries"), allow(unreachable_patterns))]
            _ => return Err(()),
        })
    }
}

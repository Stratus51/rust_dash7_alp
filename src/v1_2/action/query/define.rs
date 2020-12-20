#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum QueryComparisonType {
    Inequal = 0,
    Equal = 1,
    LessThan = 2,
    LessThanOrEqual = 3,
    GreaterThan = 4,
    GreaterThanOrEqual = 5,
    Rfu0 = 6,
    Rfu1 = 7,
}
impl QueryComparisonType {
    /// # Safety
    /// You are to warrant that n is encoded on 3 bits only.
    /// That means n <= 0x7.
    pub const unsafe fn from_unchecked(n: u8) -> Self {
        match n {
            0 => QueryComparisonType::Inequal,
            1 => QueryComparisonType::Equal,
            2 => QueryComparisonType::LessThan,
            3 => QueryComparisonType::LessThanOrEqual,
            4 => QueryComparisonType::GreaterThan,
            5 => QueryComparisonType::GreaterThanOrEqual,
            6 => QueryComparisonType::Rfu0,
            7 => QueryComparisonType::Rfu1,
            // Should never occured if used safely
            _ => QueryComparisonType::Inequal,
        }
    }

    pub const fn from(n: u8) -> Result<Self, u8> {
        Ok(match n {
            0 => QueryComparisonType::Inequal,
            1 => QueryComparisonType::Equal,
            2 => QueryComparisonType::LessThan,
            3 => QueryComparisonType::LessThanOrEqual,
            4 => QueryComparisonType::GreaterThan,
            5 => QueryComparisonType::GreaterThanOrEqual,
            6 => QueryComparisonType::Rfu0,
            7 => QueryComparisonType::Rfu1,
            x => return Err(x),
        })
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum QueryRangeComparisonType {
    NotInRange = 0,
    InRange = 1,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum QueryCode {
    // NonVoid = 0,
    // ComparisonWithZero = 1,
    ComparisonWithValue = 2,
    // ComparisonWithOtherFile = 3,
    // BitmapRangeComparison = 4,
    // StringTokenSearch = 7,
}
impl QueryCode {
    pub const fn from(n: u8) -> Result<Self, u8> {
        Ok(match n {
            // 0 => QueryCode::NonVoid,
            // 1 => QueryCode::ComparisonWithZero,
            2 => QueryCode::ComparisonWithValue,
            // 3 => QueryCode::ComparisonWithOtherFile,
            // 4 => QueryCode::BitmapRangeComparison,
            // 7 => QueryCode::StringTokenSearch,
            x => return Err(x),
        })
    }
}

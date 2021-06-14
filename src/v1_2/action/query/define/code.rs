use crate::v1_2::error::define::QueryCodeError;

pub const NON_VOID: u8 = 0;
pub const COMPARISON_WITH_ZERO: u8 = 1;
pub const COMPARISON_WITH_VALUE: u8 = 2;
pub const COMPARISON_WITH_OTHER_FILE: u8 = 3;
pub const COMPARISON_WITH_RANGE: u8 = 4;
pub const STRING_TOKEN_SEARCH: u8 = 7;

#[repr(u8)]
#[cfg(feature = "decode_query")]
#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum QueryCode {
    // NonVoid = NON_VOID,
    // ComparisonWithZero = COMPARISON_WITH_ZERO,
    #[cfg(feature = "decode_query_compare_with_value")]
    ComparisonWithValue = COMPARISON_WITH_VALUE,
    // ComparisonWithOtherFile = COMPARISON_WITH_OTHER_FILE,
    #[cfg(feature = "decode_query_compare_with_range")]
    ComparisonWithRange = COMPARISON_WITH_RANGE,
    // StringTokenSearch = STRING_TOKEN_SEARCH,
}

#[cfg(feature = "decode_query")]
impl QueryCode {
    /// # Errors
    /// Returns an error if the query code is unknown
    pub const fn from(n: u8) -> Result<Self, QueryCodeError> {
        #[cfg_attr(not(feature = "decode_query"), allow(unreachable_code))]
        Ok(match n {
            // NON_VOID => QueryCode::NonVoid,
            // COMPARISON_WITH_ZERO => QueryCode::ComparisonWithZero,
            #[cfg(feature = "decode_query_compare_with_value")]
            COMPARISON_WITH_VALUE => QueryCode::ComparisonWithValue,
            // COMPARISON_WITH_OTHER_FILE => QueryCode::ComparisonWithOtherFile,
            #[cfg(feature = "decode_query_compare_with_range")]
            COMPARISON_WITH_RANGE => QueryCode::ComparisonWithRange,
            // STRING_TOKEN_SEARCH => QueryCode::StringTokenSearch,
            n if n <= 7 => return Err(QueryCodeError::Unsupported { code: n }),
            // TODO This should be an enumeration of the queries instead of all_queries, in case
            // they are selected manually.
            #[cfg_attr(not(feature = "all_queries"), allow(unreachable_patterns))]
            _ => return Err(QueryCodeError::Invalid),
        })
    }
}

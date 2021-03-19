pub mod comparison_with_range;

use crate::decodable::{MissingByteErrorBuilder, SizeError};
use comparison_with_range::{QueryRangeError, QueryRangeSizeError};

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct UnsupportedQueryCode<'data> {
    /// Parsed query code
    pub code: u8,
    /// Remaining bytes starting with the byte containing the query code
    /// because it may contain some query specific data.
    pub remaining_data: &'data [u8],
}

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum QueryError<'data> {
    UnsupportedQueryCode(UnsupportedQueryCode<'data>),
    /// The query is a range query and the encoded range is invalid because, range.start > range.end
    BadEncodedRange,
}
impl<'data> From<UnsupportedQueryCode<'data>> for QueryError<'data> {
    fn from(e: UnsupportedQueryCode<'data>) -> Self {
        Self::UnsupportedQueryCode(e)
    }
}
impl<'data> From<QueryRangeError> for QueryError<'data> {
    fn from(e: QueryRangeError) -> Self {
        match e {
            QueryRangeError::BadEncodedRange => Self::BadEncodedRange,
        }
    }
}

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum QuerySizeError<'data> {
    UnsupportedQueryCode(UnsupportedQueryCode<'data>),
    /// The query is a range query and the encoded range is invalid because, range.start > range.end
    BadEncodedRange,
    MissingBytes,
}
impl<'data> From<QueryError<'data>> for QuerySizeError<'data> {
    fn from(e: QueryError<'data>) -> Self {
        match e {
            QueryError::BadEncodedRange => Self::BadEncodedRange,
            QueryError::UnsupportedQueryCode(e) => Self::UnsupportedQueryCode(e),
        }
    }
}
impl<'data> From<QueryRangeSizeError> for QuerySizeError<'data> {
    fn from(e: QueryRangeSizeError) -> Self {
        match e {
            QueryRangeSizeError::BadEncodedRange => Self::BadEncodedRange,
            QueryRangeSizeError::MissingBytes => Self::MissingBytes,
        }
    }
}
impl<'data> From<SizeError> for QuerySizeError<'data> {
    fn from(_: SizeError) -> Self {
        Self::MissingBytes
    }
}
impl<'data> MissingByteErrorBuilder for QuerySizeError<'data> {
    fn missing_bytes() -> Self {
        Self::MissingBytes
    }
}

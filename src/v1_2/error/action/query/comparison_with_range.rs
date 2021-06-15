use crate::decodable::MissingByteErrorBuilder;

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum QueryRangeError {
    /// The encoded range is invalid because, range.start > range.end
    BadEncodedRange,
}

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum QueryRangeSizeError {
    /// The encoded range is invalid because, range.start > range.end
    BadEncodedRange,
    MissingBytes,
}
impl From<QueryRangeError> for QueryRangeSizeError {
    fn from(e: QueryRangeError) -> Self {
        match e {
            QueryRangeError::BadEncodedRange => Self::BadEncodedRange,
        }
    }
}
impl MissingByteErrorBuilder for QueryRangeSizeError {
    fn missing_bytes() -> Self {
        Self::MissingBytes
    }
}

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum QueryRangeSetError {
    /// The given range does not have the same compare length as the encoded one.
    CompareLengthMismatch,
    /// The bitmap bit size calculated with the given range does not match the size of the encoded
    /// bitmap.
    BitmapBitSizeMismatch,
    /// The given range is invalid because, range.start > range.end
    BadGivenRange,
    /// The encoded range is invalid because, range.start > range.end
    BadEncodedRange,
    /// The range cannot be encoded with the given compare_length
    CompareLengthTooSmall,
}

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum QueryRangeSetLooselyError {
    /// The boundaries + bitmap size does not match the current one.
    ByteSizeMismatch,
    /// The given range is invalid because, range.start > range.end
    BadGivenRange,
    /// The encoded range is invalid because, range.start > range.end
    BadEncodedRange,
    /// The range cannot be encoded with the given compare_length
    CompareLengthTooSmall,
}

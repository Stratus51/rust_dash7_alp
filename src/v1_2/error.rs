use crate::decodable::{MissingByteErrorBuilder, SizeError};
// TODO Split this file into submodules. It is too crowded.

// ============================================================
// Defines
// ============================================================
#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum OpCodeError {
    Unsupported { code: u8 },
    Invalid,
}

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum QueryComparisonTypeError {
    Invalid,
}

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum QueryRangeComparisonTypeError {
    Invalid,
}

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum QueryCodeError {
    Unsupported { code: u8 },
    Invalid,
}

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum StatusExtensionError {
    Unsupported { ext: u8 },
    Invalid,
}

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum InterfaceIdError {
    Unsupported { id: u8 },
}

// ============================================================
// Operands
// ============================================================
// TODO These errors containing the data pointer are cool.
// But they currently force a loss of mutability of mutable decode error.
// That is sad, but fixing it requires quite some refactoring and some degree of code duplication.
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
}

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

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct UnsupportedExtension<'data> {
    /// Parsed status extension field
    pub extension: u8,
    /// Remaining bytes starting after the ALP action opcode byte because
    /// there is nothing left to parse in the first byte.
    pub remaining_data: &'data [u8],
}

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct UnsupportedInterfaceId<'data> {
    /// Parsed status extension field
    pub id: u8,
    /// Remaining bytes starting after the interface ID byte
    pub remaining_data: &'data [u8],
}

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum StatusInterfaceSizeError<'data> {
    UnsupportedInterfaceId(UnsupportedInterfaceId<'data>),
    MissingBytes,
}
impl<'data> From<UnsupportedInterfaceId<'data>> for StatusInterfaceSizeError<'data> {
    fn from(e: UnsupportedInterfaceId<'data>) -> Self {
        Self::UnsupportedInterfaceId(e)
    }
}
impl<'data> MissingByteErrorBuilder for StatusInterfaceSizeError<'data> {
    fn missing_bytes() -> Self {
        Self::MissingBytes
    }
}

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum StatusDecodeError<'data> {
    /// The decoded query contains an unknown query code.
    UnsupportedExtension(UnsupportedExtension<'data>),
    /// The input data contains an unknown interface ID
    UnsupportedInterfaceId(UnsupportedInterfaceId<'data>),
}
impl<'data> From<UnsupportedExtension<'data>> for StatusDecodeError<'data> {
    fn from(e: UnsupportedExtension<'data>) -> Self {
        Self::UnsupportedExtension(e)
    }
}
impl<'data> From<UnsupportedInterfaceId<'data>> for StatusDecodeError<'data> {
    fn from(e: UnsupportedInterfaceId<'data>) -> Self {
        Self::UnsupportedInterfaceId(e)
    }
}

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum StatusSizeError<'data> {
    /// The decoded query contains an unknown query code.
    UnsupportedExtension(UnsupportedExtension<'data>),
    /// The input data contains an unknown interface ID
    UnsupportedInterfaceId(UnsupportedInterfaceId<'data>),
    MissingBytes,
}

impl<'data> From<StatusInterfaceSizeError<'data>> for StatusSizeError<'data> {
    fn from(e: StatusInterfaceSizeError<'data>) -> Self {
        match e {
            StatusInterfaceSizeError::MissingBytes => Self::MissingBytes,
            StatusInterfaceSizeError::UnsupportedInterfaceId(e) => Self::UnsupportedInterfaceId(e),
        }
    }
}
impl<'data> From<StatusDecodeError<'data>> for StatusSizeError<'data> {
    fn from(e: StatusDecodeError<'data>) -> Self {
        match e {
            StatusDecodeError::UnsupportedExtension(e) => Self::UnsupportedExtension(e),
            StatusDecodeError::UnsupportedInterfaceId(e) => Self::UnsupportedInterfaceId(e),
        }
    }
}
impl<'data> From<UnsupportedExtension<'data>> for StatusSizeError<'data> {
    fn from(e: UnsupportedExtension<'data>) -> Self {
        Self::UnsupportedExtension(e)
    }
}
impl<'data> MissingByteErrorBuilder for StatusSizeError<'data> {
    fn missing_bytes() -> Self {
        Self::MissingBytes
    }
}

// ============================================================
// Action
// ============================================================
#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct UnsupportedOpCode<'data> {
    /// Parsed op_code field
    pub op_code: u8,
    /// Remaining bytes starting with the op_code byte
    pub remaining_data: &'data [u8],
}

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum ActionDecodeError<'data> {
    UnsupportedOpCode(UnsupportedOpCode<'data>),
    Query(QueryError<'data>),
    Status(StatusDecodeError<'data>),
}

impl<'data> From<StatusDecodeError<'data>> for ActionDecodeError<'data> {
    fn from(e: StatusDecodeError<'data>) -> Self {
        Self::Status(e)
    }
}

impl<'data> From<QueryError<'data>> for ActionDecodeError<'data> {
    fn from(e: QueryError<'data>) -> Self {
        Self::Query(e)
    }
}

impl<'data> From<UnsupportedOpCode<'data>> for ActionDecodeError<'data> {
    fn from(e: UnsupportedOpCode<'data>) -> Self {
        Self::UnsupportedOpCode(e)
    }
}

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum ActionSizeError<'data> {
    UnsupportedOpCode(UnsupportedOpCode<'data>),
    Query(QueryError<'data>),
    Status(StatusDecodeError<'data>),
    MissingBytes,
}

impl<'data> From<StatusDecodeError<'data>> for ActionSizeError<'data> {
    fn from(e: StatusDecodeError<'data>) -> Self {
        Self::Status(e)
    }
}

impl<'data> From<QueryError<'data>> for ActionSizeError<'data> {
    fn from(e: QueryError<'data>) -> Self {
        Self::Query(e)
    }
}

impl<'data> From<StatusSizeError<'data>> for ActionSizeError<'data> {
    fn from(e: StatusSizeError<'data>) -> Self {
        match e {
            StatusSizeError::MissingBytes => Self::MissingBytes,
            StatusSizeError::UnsupportedExtension(e) => {
                Self::Status(StatusDecodeError::UnsupportedExtension(e))
            }
            StatusSizeError::UnsupportedInterfaceId(e) => {
                Self::Status(StatusDecodeError::UnsupportedInterfaceId(e))
            }
        }
    }
}

impl<'data> From<QuerySizeError<'data>> for ActionSizeError<'data> {
    fn from(e: QuerySizeError<'data>) -> Self {
        match e {
            QuerySizeError::MissingBytes => Self::MissingBytes,
            QuerySizeError::UnsupportedQueryCode(e) => {
                Self::Query(QueryError::UnsupportedQueryCode(e))
            }
            QuerySizeError::BadEncodedRange => Self::Query(QueryError::BadEncodedRange),
        }
    }
}

impl<'data> From<UnsupportedOpCode<'data>> for ActionSizeError<'data> {
    fn from(e: UnsupportedOpCode<'data>) -> Self {
        Self::UnsupportedOpCode(e)
    }
}

impl<'data> From<SizeError> for ActionSizeError<'data> {
    fn from(_: SizeError) -> Self {
        Self::MissingBytes
    }
}

impl<'data> From<ActionDecodeError<'data>> for ActionSizeError<'data> {
    fn from(e: ActionDecodeError<'data>) -> Self {
        match e {
            ActionDecodeError::UnsupportedOpCode(e) => Self::UnsupportedOpCode(e),
            ActionDecodeError::Query(e) => Self::Query(e),
            ActionDecodeError::Status(e) => Self::Status(e),
        }
    }
}

impl<'data> MissingByteErrorBuilder for ActionSizeError<'data> {
    fn missing_bytes() -> Self {
        Self::MissingBytes
    }
}

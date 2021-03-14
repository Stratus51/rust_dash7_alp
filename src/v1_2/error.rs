use crate::decodable::{MissingByteErrorBuilder, SizeError};

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
pub struct UnsupportedQueryCode<'item, 'data> {
    /// Parsed query code
    pub code: u8,
    /// Remaining bytes starting with the byte containing the query code
    /// because it may contain some query specific data.
    pub remaining_data: &'item &'data [u8],
}

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum QuerySizeError<'item, 'data> {
    MissingBytes,
    UnsupportedQueryCode(UnsupportedQueryCode<'item, 'data>),
}
impl<'item, 'data> From<UnsupportedQueryCode<'item, 'data>> for QuerySizeError<'item, 'data> {
    fn from(e: UnsupportedQueryCode<'item, 'data>) -> Self {
        Self::UnsupportedQueryCode(e)
    }
}
impl<'item, 'data> MissingByteErrorBuilder for QuerySizeError<'item, 'data> {
    fn missing_bytes() -> Self {
        Self::MissingBytes
    }
}

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct UnsupportedExtension<'item, 'data> {
    /// Parsed status extension field
    pub extension: u8,
    /// Remaining bytes starting after the ALP action opcode byte because
    /// there is nothing left to parse in the first byte.
    pub remaining_data: &'item &'data [u8],
}

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct UnsupportedInterfaceId<'item, 'data> {
    /// Parsed status extension field
    pub id: u8,
    /// Remaining bytes starting after the interface ID byte
    pub remaining_data: &'item &'data [u8],
}

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum StatusInterfaceSizeError<'item, 'data> {
    MissingBytes,
    UnsupportedInterfaceId(UnsupportedInterfaceId<'item, 'data>),
}
impl<'item, 'data> From<UnsupportedInterfaceId<'item, 'data>>
    for StatusInterfaceSizeError<'item, 'data>
{
    fn from(e: UnsupportedInterfaceId<'item, 'data>) -> Self {
        Self::UnsupportedInterfaceId(e)
    }
}
impl<'item, 'data> MissingByteErrorBuilder for StatusInterfaceSizeError<'item, 'data> {
    fn missing_bytes() -> Self {
        Self::MissingBytes
    }
}

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum StatusDecodeError<'item, 'data> {
    /// The decoded query contains an unknown query code.
    UnsupportedExtension(UnsupportedExtension<'item, 'data>),
    /// The input data contains an unknown interface ID
    UnsupportedInterfaceId(UnsupportedInterfaceId<'item, 'data>),
}
impl<'item, 'data> From<UnsupportedExtension<'item, 'data>> for StatusDecodeError<'item, 'data> {
    fn from(e: UnsupportedExtension<'item, 'data>) -> Self {
        Self::UnsupportedExtension(e)
    }
}
impl<'item, 'data> From<UnsupportedInterfaceId<'item, 'data>> for StatusDecodeError<'item, 'data> {
    fn from(e: UnsupportedInterfaceId<'item, 'data>) -> Self {
        Self::UnsupportedInterfaceId(e)
    }
}

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum StatusSizeError<'item, 'data> {
    MissingBytes,
    /// The decoded query contains an unknown query code.
    UnsupportedExtension(UnsupportedExtension<'item, 'data>),
    /// The input data contains an unknown interface ID
    UnsupportedInterfaceId(UnsupportedInterfaceId<'item, 'data>),
}

impl<'item, 'data> From<StatusInterfaceSizeError<'item, 'data>> for StatusSizeError<'item, 'data> {
    fn from(e: StatusInterfaceSizeError<'item, 'data>) -> Self {
        match e {
            StatusInterfaceSizeError::MissingBytes => Self::MissingBytes,
            StatusInterfaceSizeError::UnsupportedInterfaceId(e) => Self::UnsupportedInterfaceId(e),
        }
    }
}
impl<'item, 'data> From<StatusDecodeError<'item, 'data>> for StatusSizeError<'item, 'data> {
    fn from(e: StatusDecodeError<'item, 'data>) -> Self {
        match e {
            StatusDecodeError::UnsupportedExtension(e) => Self::UnsupportedExtension(e),
            StatusDecodeError::UnsupportedInterfaceId(e) => Self::UnsupportedInterfaceId(e),
        }
    }
}
impl<'item, 'data> From<UnsupportedExtension<'item, 'data>> for StatusSizeError<'item, 'data> {
    fn from(e: UnsupportedExtension<'item, 'data>) -> Self {
        Self::UnsupportedExtension(e)
    }
}
impl<'item, 'data> MissingByteErrorBuilder for StatusSizeError<'item, 'data> {
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
pub struct UnsupportedOpCode<'item, 'data> {
    /// Parsed op_code field
    pub op_code: u8,
    /// Remaining bytes starting with the op_code byte
    pub remaining_data: &'item &'data [u8],
}

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum ActionDecodeError<'item, 'data> {
    UnsupportedOpCode(UnsupportedOpCode<'item, 'data>),
    Query(UnsupportedQueryCode<'item, 'data>),
    Status(StatusDecodeError<'item, 'data>),
}

impl<'item, 'data> From<StatusDecodeError<'item, 'data>> for ActionDecodeError<'item, 'data> {
    fn from(e: StatusDecodeError<'item, 'data>) -> Self {
        Self::Status(e)
    }
}

impl<'item, 'data> From<UnsupportedQueryCode<'item, 'data>> for ActionDecodeError<'item, 'data> {
    fn from(e: UnsupportedQueryCode<'item, 'data>) -> Self {
        Self::Query(e)
    }
}

impl<'item, 'data> From<UnsupportedOpCode<'item, 'data>> for ActionDecodeError<'item, 'data> {
    fn from(e: UnsupportedOpCode<'item, 'data>) -> Self {
        Self::UnsupportedOpCode(e)
    }
}

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum ActionSizeError<'item, 'data> {
    MissingBytes,
    UnsupportedOpCode(UnsupportedOpCode<'item, 'data>),
    Query(UnsupportedQueryCode<'item, 'data>),
    Status(StatusDecodeError<'item, 'data>),
}

impl<'item, 'data> From<StatusDecodeError<'item, 'data>> for ActionSizeError<'item, 'data> {
    fn from(e: StatusDecodeError<'item, 'data>) -> Self {
        Self::Status(e)
    }
}

impl<'item, 'data> From<UnsupportedQueryCode<'item, 'data>> for ActionSizeError<'item, 'data> {
    fn from(e: UnsupportedQueryCode<'item, 'data>) -> Self {
        Self::Query(e)
    }
}

impl<'item, 'data> From<StatusSizeError<'item, 'data>> for ActionSizeError<'item, 'data> {
    fn from(e: StatusSizeError<'item, 'data>) -> Self {
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

impl<'item, 'data> From<QuerySizeError<'item, 'data>> for ActionSizeError<'item, 'data> {
    fn from(e: QuerySizeError<'item, 'data>) -> Self {
        match e {
            QuerySizeError::MissingBytes => Self::MissingBytes,
            QuerySizeError::UnsupportedQueryCode(e) => Self::Query(e),
        }
    }
}

impl<'item, 'data> From<UnsupportedOpCode<'item, 'data>> for ActionSizeError<'item, 'data> {
    fn from(e: UnsupportedOpCode<'item, 'data>) -> Self {
        Self::UnsupportedOpCode(e)
    }
}

impl<'item, 'data> From<SizeError> for ActionSizeError<'item, 'data> {
    fn from(_: SizeError) -> Self {
        Self::MissingBytes
    }
}

impl<'item, 'data> From<ActionDecodeError<'item, 'data>> for ActionSizeError<'item, 'data> {
    fn from(e: ActionDecodeError<'item, 'data>) -> Self {
        match e {
            ActionDecodeError::UnsupportedOpCode(e) => Self::UnsupportedOpCode(e),
            ActionDecodeError::Query(e) => Self::Query(e),
            ActionDecodeError::Status(e) => Self::Status(e),
        }
    }
}

impl<'item, 'data> MissingByteErrorBuilder for ActionSizeError<'item, 'data> {
    fn missing_bytes() -> Self {
        Self::MissingBytes
    }
}

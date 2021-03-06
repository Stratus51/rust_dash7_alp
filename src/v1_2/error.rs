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
pub enum QuerySizeError<'data> {
    MissingBytes,
    UnsupportedQueryCode(UnsupportedQueryCode<'data>),
}
impl<'data> From<UnsupportedQueryCode<'data>> for QuerySizeError<'data> {
    fn from(e: UnsupportedQueryCode<'data>) -> Self {
        Self::UnsupportedQueryCode(e)
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
    MissingBytes,
    UnsupportedInterfaceId(UnsupportedInterfaceId<'data>),
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
    MissingBytes,
    /// The decoded query contains an unknown query code.
    UnsupportedExtension(UnsupportedExtension<'data>),
    /// The input data contains an unknown interface ID
    UnsupportedInterfaceId(UnsupportedInterfaceId<'data>),
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
    Query(UnsupportedQueryCode<'data>),
    Status(StatusDecodeError<'data>),
}

impl<'data> From<StatusDecodeError<'data>> for ActionDecodeError<'data> {
    fn from(e: StatusDecodeError<'data>) -> Self {
        Self::Status(e)
    }
}

impl<'data> From<UnsupportedQueryCode<'data>> for ActionDecodeError<'data> {
    fn from(e: UnsupportedQueryCode<'data>) -> Self {
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
    MissingBytes,
    UnsupportedOpCode(UnsupportedOpCode<'data>),
    Query(UnsupportedQueryCode<'data>),
    Status(StatusDecodeError<'data>),
}

impl<'data> From<StatusDecodeError<'data>> for ActionSizeError<'data> {
    fn from(e: StatusDecodeError<'data>) -> Self {
        Self::Status(e)
    }
}

impl<'data> From<UnsupportedQueryCode<'data>> for ActionSizeError<'data> {
    fn from(e: UnsupportedQueryCode<'data>) -> Self {
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
            QuerySizeError::UnsupportedQueryCode(e) => Self::Query(e),
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

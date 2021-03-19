pub mod query;
pub mod status;

use crate::decodable::{MissingByteErrorBuilder, SizeError};
use query::{QueryError, QuerySizeError};
use status::{StatusDecodeError, StatusSizeError};

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

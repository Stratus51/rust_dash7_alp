pub mod interface;

use crate::decodable::MissingByteErrorBuilder;
use interface::{StatusInterfaceSizeError, UnsupportedInterfaceId};

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

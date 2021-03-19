use crate::decodable::MissingByteErrorBuilder;

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

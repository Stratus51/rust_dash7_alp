#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum BasicDecodeError {
    /// The input data is missing bytes to be decoded into the wanted item.
    MissingBytes(usize),
}

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum QueryDecodeError {
    /// The decoded query contains an unknown query code.
    UnknownQueryCode(u8),
    /// The input data is missing bytes to be decoded into the wanted item.
    MissingBytes(usize),
}

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum QueryOperandDecodeError {
    /// The decoded query contains an unknown query code.
    UnknownQueryCode(u8),
    /// The input data is missing bytes to be decoded into the wanted item.
    MissingBytes(usize),
}

impl From<QueryOperandDecodeError> for QueryDecodeError {
    fn from(e: QueryOperandDecodeError) -> Self {
        match e {
            QueryOperandDecodeError::UnknownQueryCode(c) => Self::UnknownQueryCode(c),
            QueryOperandDecodeError::MissingBytes(n) => Self::MissingBytes(n),
        }
    }
}

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct UnknownQueryCode<'data> {
    /// Parsed query code
    pub code: u8,
    /// Remaining bytes starting with the byte containing the query code
    /// because it may contain some query specific data.
    pub remaining_data: &'data [u8],
}

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct PtrUnknownQueryCode<'data> {
    /// Parsed query code
    pub code: u8,
    /// Remaining bytes starting with the byte containing the query code
    /// because it may contain some query specific data.
    pub remaining_data: *const u8,
    pub phantom: core::marker::PhantomData<&'data ()>,
}

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum QueryActionDecodeError<'data> {
    /// The decoded query contains an unknown query code.
    UnknownQueryCode(UnknownQueryCode<'data>),
    /// The input data is missing bytes to be decoded into the wanted item.
    MissingBytes(usize),
}

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct UnknownExtension<'data> {
    /// Parsed status extension field
    pub extension: u8,
    /// Remaining bytes starting after the ALP action opcode byte because
    /// there is nothing left to parse in the first byte.
    pub remaining_data: &'data [u8],
}

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct PtrUnknownExtension<'data> {
    /// Parsed status extension field
    pub extension: u8,
    /// Remaining bytes starting after the ALP action opcode byte because
    /// there is nothing left to parse in the first byte.
    pub remaining_data: *const u8,
    pub phantom: core::marker::PhantomData<&'data ()>,
}

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct UnknownInterfaceId<'data> {
    /// Parsed status extension field
    pub id: u8,
    /// Remaining bytes starting after the interface ID byte
    pub remaining_data: &'data [u8],
}

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct PtrUnknownInterfaceId<'data> {
    /// Parsed status extension field
    pub id: u8,
    /// Remaining bytes starting after the interface ID byte
    pub remaining_data: *const u8,
    pub phantom: core::marker::PhantomData<&'data ()>,
}

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum StatusDecodeError<'data> {
    /// The decoded query contains an unknown query code.
    UnknownExtension(UnknownExtension<'data>),
    /// The input data is missing bytes to be decoded into the wanted item.
    MissingBytes(usize),
    /// The input data contains an unknown interface ID
    UnknownInterfaceId(UnknownInterfaceId<'data>),
}

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum UncheckedStatusDecodeError<'data> {
    /// The decoded query contains an unknown query code.
    UnknownExtension(UnknownExtension<'data>),
    /// The input data contains an unknown interface ID
    UnknownInterfaceId(UnknownInterfaceId<'data>),
}

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum PtrUncheckedStatusDecodeError<'data> {
    /// The decoded query contains an unknown query code.
    UnknownExtension(PtrUnknownExtension<'data>),
    /// The input data contains an unknown interface ID
    UnknownInterfaceId(PtrUnknownInterfaceId<'data>),
}

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum StatusInterfaceDecodeError {
    /// The input data is missing bytes to be decoded into the wanted item.
    MissingBytes(usize),
    /// The input data contains an unknown interface ID
    UnknownInterfaceId(u8),
}

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum ActionDecodeError<'data> {
    UnknownActionCode(u8),
    MissingBytes(usize),
    UnknownQueryCode(UnknownQueryCode<'data>),
    UnknownStatusExtension(UnknownExtension<'data>),
    UnknownStatusInterfaceId(UnknownInterfaceId<'data>),
}

impl<'data> From<UncheckedStatusDecodeError<'data>> for ActionDecodeError<'data> {
    fn from(e: UncheckedStatusDecodeError<'data>) -> Self {
        match e {
            UncheckedStatusDecodeError::UnknownExtension(e) => Self::UnknownStatusExtension(e),
            UncheckedStatusDecodeError::UnknownInterfaceId(e) => Self::UnknownStatusInterfaceId(e),
        }
    }
}

impl<'data> From<UnknownQueryCode<'data>> for ActionDecodeError<'data> {
    fn from(e: UnknownQueryCode<'data>) -> Self {
        Self::UnknownQueryCode(e)
    }
}

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum PtrActionDecodeError<'data> {
    UnknownActionCode(u8),
    UnknownQueryCode(PtrUnknownQueryCode<'data>),
    UnknownStatusExtension(PtrUnknownExtension<'data>),
    UnknownStatusInterfaceId(PtrUnknownInterfaceId<'data>),
}

impl<'data> From<PtrUncheckedStatusDecodeError<'data>> for PtrActionDecodeError<'data> {
    fn from(e: PtrUncheckedStatusDecodeError<'data>) -> Self {
        match e {
            PtrUncheckedStatusDecodeError::UnknownExtension(e) => Self::UnknownStatusExtension(e),
            PtrUncheckedStatusDecodeError::UnknownInterfaceId(e) => {
                Self::UnknownStatusInterfaceId(e)
            }
        }
    }
}

impl<'data> From<PtrUnknownQueryCode<'data>> for PtrActionDecodeError<'data> {
    fn from(e: PtrUnknownQueryCode<'data>) -> Self {
        Self::UnknownQueryCode(e)
    }
}

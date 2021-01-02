#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum BasicDecodeError {
    /// The input data is missing bytes to be decoded into the wanted item.
    MissingBytes(usize),
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum QueryDecodeError {
    /// The decoded query contains an unknown query code.
    UnknownQueryCode(u8),
    /// The input data is missing bytes to be decoded into the wanted item.
    MissingBytes(usize),
}

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

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct UnknownQueryCode<'data> {
    /// Parsed query code
    pub code: u8,
    /// Remaining bytes starting with the byte containing the query code
    /// because it may contain some query specific data.
    pub remaining_data: &'data [u8],
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct PtrUnknownQueryCode<'data> {
    /// Parsed query code
    pub code: u8,
    /// Remaining bytes starting with the byte containing the query code
    /// because it may contain some query specific data.
    pub remaining_data: *const u8,
    pub phantom: core::marker::PhantomData<&'data ()>,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum QueryActionDecodeError<'data> {
    /// The decoded query contains an unknown query code.
    UnknownQueryCode(UnknownQueryCode<'data>),
    /// The input data is missing bytes to be decoded into the wanted item.
    MissingBytes(usize),
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct UnknownExtension<'data> {
    /// Parsed status extension field
    pub extension: u8,
    /// Remaining bytes starting after the ALP action opcode byte because
    /// there is nothing left to parse in the first byte.
    pub remaining_data: &'data [u8],
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct PtrUnknownExtension<'data> {
    /// Parsed status extension field
    pub extension: u8,
    /// Remaining bytes starting after the ALP action opcode byte because
    /// there is nothing left to parse in the first byte.
    pub remaining_data: *const u8,
    pub phantom: core::marker::PhantomData<&'data ()>,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct UnknownInterfaceId<'data> {
    /// Parsed status extension field
    pub id: u8,
    /// Remaining bytes starting after the interface ID byte
    pub remaining_data: &'data [u8],
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct PtrUnknownInterfaceId<'data> {
    /// Parsed status extension field
    pub id: u8,
    /// Remaining bytes starting after the interface ID byte
    pub remaining_data: *const u8,
    pub phantom: core::marker::PhantomData<&'data ()>,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum StatusDecodeError<'data> {
    /// The decoded query contains an unknown query code.
    UnknownExtension(UnknownExtension<'data>),
    /// The input data is missing bytes to be decoded into the wanted item.
    MissingBytes(usize),
    /// The input data contains an unknown interface ID
    UnknownInterfaceId(UnknownInterfaceId<'data>),
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum UncheckedStatusDecodeError<'data> {
    /// The decoded query contains an unknown query code.
    UnknownExtension(UnknownExtension<'data>),
    /// The input data contains an unknown interface ID
    UnknownInterfaceId(UnknownInterfaceId<'data>),
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum PtrUncheckedStatusDecodeError<'data> {
    /// The decoded query contains an unknown query code.
    UnknownExtension(PtrUnknownExtension<'data>),
    /// The input data contains an unknown interface ID
    UnknownInterfaceId(PtrUnknownInterfaceId<'data>),
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum StatusInterfaceDecodeError {
    /// The input data is missing bytes to be decoded into the wanted item.
    MissingBytes(usize),
    /// The input data contains an unknown interface ID
    UnknownInterfaceId(u8),
}

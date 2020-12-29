#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum BasicDecodeError {
    /// The input data is missing bytes to be decoded into the wanted item.
    MissingBytes(usize),
    /// The input contains an opcode that does not match the item you tried to
    /// decode.
    BadOpCode,
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
pub enum QueryActionDecodeError {
    /// The decoded query contains an unknown query code.
    UnknownQueryCode { code: u8, offset: usize },
    /// The input data is missing bytes to be decoded into the wanted item.
    MissingBytes(usize),
    /// The input contains an opcode that does not match the item you tried to
    /// decode.
    BadOpCode,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum StatusDecodeError {
    /// The decoded query contains an unknown query code.
    UnknownExtension { extension: u8, offset: usize },
    /// The input data is missing bytes to be decoded into the wanted item.
    MissingBytes(usize),
    /// The input contains an opcode that does not match the item you tried to
    /// decode.
    BadOpCode,
    /// The input data contains an unknown interface ID
    // TODO This offset needs to be replaced with a reference to the remaining data instead.
    // It would require a unique operation from the error generator to build a slice, then it
    // will be forwarded as is, and the next parsers only need to care about that last bit
    // of data anyway
    UnknownInterfaceId { id: u8, offset: usize },
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum UncheckedStatusDecodeError {
    /// The decoded query contains an unknown query code.
    UnknownExtension { extension: u8, offset: usize },
    /// The input data contains an unknown interface ID
    // TODO This offset needs to be replaced with a reference to the remaining data instead.
    // It would require a unique operation from the error generator to build a slice, then it
    // will be forwarded as is, and the next parsers only need to care about that last bit
    // of data anyway
    UnknownInterfaceId { id: u8, offset: usize },
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum StatusInterfaceDecodeError {
    /// The input data is missing bytes to be decoded into the wanted item.
    MissingBytes(usize),
    /// The input data contains an unknown interface ID
    UnknownInterfaceId(u8),
}

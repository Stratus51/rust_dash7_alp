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
    /// The decoded query contains an invalid query code.
    BadQueryCode(u8),
    /// The input data is missing bytes to be decoded into the wanted item.
    MissingBytes(usize),
    /// The input contains an opcode that does not match the item you tried to
    /// decode.
    // TODO Add the bad op code to the item so that the error handling
    // code does not have to parse it again.
    BadOpCode,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum QueryOperandDecodeError {
    /// The decoded query contains an invalid query code.
    BadQueryCode(u8),
    /// The input data is missing bytes to be decoded into the wanted item.
    MissingBytes(usize),
}

impl From<QueryOperandDecodeError> for QueryDecodeError {
    fn from(e: QueryOperandDecodeError) -> Self {
        match e {
            QueryOperandDecodeError::BadQueryCode(c) => Self::BadQueryCode(c),
            QueryOperandDecodeError::MissingBytes(n) => Self::MissingBytes(n),
        }
    }
}

// TODO Move to v1_2

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum BasicDecodeError {
    /// The input data is missing bytes to be decoded into the wanted item
    MissingBytes(usize),
    /// The input contains an opcode that does not match the item you tried to
    /// decode
    BadOpCode,
}

/// Result of a successful byte array decoding
#[derive(Clone, Debug, PartialEq)]
pub struct ParseValue<T> {
    /// Decoded value
    pub value: T,
    /// Number of bytes consumed to parse the value
    pub size: usize,
}
impl<T> ParseValue<T> {
    /// Internal method
    pub fn map_value<R, F: Fn(T) -> R>(self, f: F) -> ParseValue<R> {
        let ParseValue { value, size } = self;
        ParseValue {
            value: f(value),
            size,
        }
    }
    /// Internal method
    pub fn map<R, F: Fn(T, usize) -> (R, usize)>(self, f: F) -> ParseValue<R> {
        let ParseValue { value, size } = self;
        let (value, size) = f(value, size);
        ParseValue { value, size }
    }
}

use crate::Enum;

/// This represents the cases where the data cannot be parsed because of its content (thus will
/// never be parseable).
#[derive(Clone, Debug, PartialEq)]
pub enum ParseError {
    /// One of the values parsed is impossible. Thus the data is non-decodable.
    /// The returned values indicate what was the type being parsed and the impossible value
    /// associated.
    ImpossibleValue { en: Enum, value: u8 },
}

/// Parsing Fails
/// This represents the cases where the parsing is impossible with the currently given data.
#[derive(Clone, Debug, PartialEq)]
pub enum ParseFail {
    /// Data is still missing to complete the parsing. This failure returns the minimum number of
    /// bytes required to finish this parsing.
    MissingBytes(usize),
    /// A parsing error occured due to the data content. This means that the data is unparseable.
    Error { error: ParseError, offset: usize },
}
impl ParseFail {
    /// Internal method
    pub fn inc_offset(self, n: usize) -> Self {
        match self {
            ParseFail::Error { error, offset } => ParseFail::Error {
                error,
                offset: offset + n,
            },
            x => x,
        }
    }
}
/// Result returned by a decoding attempt.
pub type ParseResult<T> = Result<ParseValue<T>, ParseFail>;

pub trait ParseResultExtension {
    /// Internal method
    fn inc_offset(self, n: usize) -> Self;
}

impl<T> ParseResultExtension for ParseResult<T> {
    /// Internal method
    fn inc_offset(self, n: usize) -> Self {
        self.map_err(|e| match e {
            ParseFail::Error { error, offset } => ParseFail::Error {
                error,
                offset: offset + n,
            },
            x => x,
        })
    }
}

/// Trait implemented by any item that is encodable to a byte array and decodable from a byte
/// array.
pub trait Codec {
    /// Computes the number of bytes required to encode the item.
    fn encoded_size(&self) -> usize;

    /// Encode the item into a given byte array.
    /// # Safety
    /// You have to ensure there is enough space in the given array (compared to what
    /// [encoded_size](#encoded_size) returns) or this method will panic.
    /// # Panics
    /// Panics if the given `out` array is too small.
    unsafe fn encode(&self, out: &mut [u8]) -> usize;

    /// Attempt to decode a byte array to produce an item.
    /// May return the item with the bytes consumed, a request for more bytes or a parsing error
    fn decode(out: &[u8]) -> ParseResult<Self>
    where
        Self: std::marker::Sized;

    /// Allocate a byte array of the right size and encode the item in it.
    fn encode_to_box(&self) -> Box<[u8]> {
        let mut data = vec![0; self.encoded_size()].into_boxed_slice();
        unsafe { self.encode(&mut data) };
        data
    }
}

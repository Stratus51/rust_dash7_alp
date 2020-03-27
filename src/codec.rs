#[derive(Clone, Debug, PartialEq)]
pub struct ParseValue<T> {
    pub value: T,
    pub size: usize,
}
impl<T> ParseValue<T> {
    pub fn map_value<R, F: Fn(T) -> R>(self, f: F) -> ParseValue<R> {
        let ParseValue { value, size } = self;
        ParseValue {
            value: f(value),
            size,
        }
    }
    pub fn map<R, F: Fn(T, usize) -> (R, usize)>(self, f: F) -> ParseValue<R> {
        let ParseValue { value, size } = self;
        let (value, size) = f(value, size);
        ParseValue { value, size }
    }
}

use crate::Enum;

#[derive(Clone, Debug, PartialEq)]
pub enum ParseError {
    UnknownEnumVariant { en: Enum, value: u8 },
}

#[derive(Clone, Debug, PartialEq)]
pub enum ParseFail {
    MissingBytes(Option<usize>),
    Error { error: ParseError, offset: usize },
}
impl ParseFail {
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
pub type ParseResult<T> = Result<ParseValue<T>, ParseFail>;

pub trait ParseResultExtension {
    fn inc_offset(self, n: usize) -> Self;
}

impl<T> ParseResultExtension for ParseResult<T> {
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

pub trait Codec {
    fn encoded_size(&self) -> usize;
    unsafe fn encode(&self, out: &mut [u8]) -> usize;
    fn decode(out: &[u8]) -> ParseResult<Self>
    where
        Self: std::marker::Sized;
    fn encode_to_box(&self) -> Box<[u8]> {
        let mut data = vec![0; self.encoded_size()].into_boxed_slice();
        unsafe { self.encode(&mut data) };
        data
    }
}

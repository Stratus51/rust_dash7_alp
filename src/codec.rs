#[derive(Clone, Debug, PartialEq)]
pub struct ParseValue<T> {
    pub value: T,
    // TODO Rename: data_size? size?
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

#[derive(Clone, Debug, PartialEq)]
pub enum ParseError {
    MissingBytes(Option<usize>),
}
pub type ParseResult<T> = Result<ParseValue<T>, ParseError>;

// TODO Rename to Codec, encode, decode
pub trait Codec {
    fn encoded_size(&self) -> usize;
    fn encode(&self, out: &mut [u8]) -> usize;
    fn decode(out: &[u8]) -> ParseResult<Self>
    where
        Self: std::marker::Sized;
    fn encode_to_box(&self) -> Box<[u8]> {
        let mut data = vec![0; self.encoded_size()].into_boxed_slice();
        self.encode(&mut data);
        data
    }
}

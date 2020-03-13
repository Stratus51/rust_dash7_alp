#[derive(Clone, Debug, PartialEq)]
pub struct ParseValue<T> {
    pub value: T,
    // TODO Rename: data_size? size?
    pub data_read: usize,
}
impl<T> ParseValue<T> {
    pub fn map_value<R, F: Fn(T) -> R>(self, f: F) -> ParseValue<R> {
        let ParseValue { value, data_read } = self;
        ParseValue {
            value: f(value),
            data_read,
        }
    }
    pub fn map<R, F: Fn(T, usize) -> (R, usize)>(self, f: F) -> ParseValue<R> {
        let ParseValue { value, data_read } = self;
        let (value, data_read) = f(value, data_read);
        ParseValue { value, data_read }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ParseError {
    MissingBytes(Option<usize>),
}
pub type ParseResult<T> = Result<ParseValue<T>, ParseError>;

// TODO Rename to Codec, encode, decode
pub trait Serializable {
    fn serialized_size(&self) -> usize;
    fn serialize(&self, out: &mut [u8]) -> usize;
    fn deserialize(out: &[u8]) -> ParseResult<Self>
    where
        Self: std::marker::Sized;
    fn serialize_to_box(&self) -> Box<[u8]> {
        let mut data = vec![0; self.serialized_size()].into_boxed_slice();
        self.serialize(&mut data);
        data
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq)]
pub struct WithOffset<T> {
    pub offset: usize,
    pub value: T,
}

impl<T> WithOffset<T> {
    pub fn new(offset: usize, value: T) -> Self {
        Self { offset, value }
    }

    pub fn new_head(value: T) -> Self {
        Self::new(0, value)
    }

    pub fn shift(mut self, n: usize) -> Self {
        self.offset += n;
        self
    }

    pub fn map_value<U, F: FnOnce(T) -> U>(self, f: F) -> WithOffset<U> {
        let Self { offset, value } = self;
        WithOffset {
            offset,
            value: f(value),
        }
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq)]
pub struct WithSize<T> {
    pub size: usize,
    pub value: T,
}

impl<T> WithSize<T> {
    pub fn new(size: usize, value: T) -> Self {
        Self { size, value }
    }

    pub fn add(&mut self, n: usize) {
        self.size += n;
    }

    pub fn map_value<U, F: FnOnce(T) -> U>(self, f: F) -> WithSize<U> {
        let Self { size, value } = self;
        WithSize {
            size,
            value: f(value),
        }
    }
}

// TODO Bad name
#[derive(Debug, Copy, Clone, Hash, PartialEq)]
pub enum StdError {
    MissingBytes(usize),
}

/// Trait implemented by any item that is encodable to a byte array and decodable from a byte
/// array.
pub trait Codec: core::marker::Sized {
    type Error;

    /// Computes the number of bytes required to encode the item.
    fn encoded_size(&self) -> usize;

    /// Encode the item into a given byte array.
    /// # Safety
    /// You have to ensure there is enough space in the given array (compared to what
    /// [encoded_size](#encoded_size) returns) or this method will panic.
    /// # Panics
    /// Panics if the given `out` array is too small.
    unsafe fn encode_in(&self, out: &mut [u8]) -> usize;

    /// Attempt to decode a byte array to produce an item.
    /// May return the item with the bytes consumed, a request for more bytes or a parsing error
    fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>>;

    /// Allocate a byte array of the right size and encode the item in it.
    fn encode(&self) -> Box<[u8]> {
        let mut data = vec![0; self.encoded_size()].into_boxed_slice();
        unsafe { self.encode_in(&mut data) };
        data
    }
}

// TODO Not working because of lifetimes on DecodableBuffer.complete_decoding()
pub trait Decodable<Buffer: DecodableBuffer<Item>, Item = Self> {
    type Error;

    /// Creates a decodable item from a data pointer without checking the data size.
    ///
    /// This method is meant to allow unchecked cross language wrapper libraries
    /// to implement an unchecked call without having to build a fake slice with
    /// a fake size.
    ///
    /// It is not meant to be used inside a Rust library/binary.
    ///
    /// # Safety
    /// You are to check that data is not empty and that data.len() >=
    /// [DecodableVarint.size()](struct.DecodableVarint.html#method.size)
    /// (the expected byte size of the returned DecodableItem).
    ///
    /// Failing that might result in reading and interpreting data outside the given
    /// array.
    unsafe fn start_decoding_ptr(data: *const u8) -> Buffer;

    /// Creates a decodable item without checking the data size.
    ///
    /// # Safety
    /// You are to check that data is not empty and that data.len() >=
    /// [DecodableVarint.size()](struct.DecodableWriteFileData.html#method.size)
    /// (the expected byte size of the returned DecodableItem).
    ///
    /// Failing that might result in reading and interpreting data outside the given
    /// array.
    unsafe fn start_decoding_unchecked(data: &[u8]) -> Buffer {
        Self::start_decoding_ptr(data.as_ptr())
    }

    /// Creates a decodable item.
    ///
    /// This decodable item allows each parts of the item independently.
    fn start_decoding(data: &[u8]) -> Result<Buffer, Self::Error>;

    /// Decodes the Item from a data pointer.
    ///
    /// Returns the decoded data and the number of bytes consumed to produce it.
    ///
    /// This method is meant to allow unchecked cross language wrapper libraries
    /// to implement an unchecked call without having to build a fake slice with
    /// a fake size.
    ///
    /// It is not meant to be used inside a Rust library/binary.
    ///
    /// # Safety
    /// You are to check that data is not empty and that data.len() >=
    /// [DecodableVarint.size()](struct.DecodableVarint.html#method.size)
    /// (the expected byte size of the returned DecodableItem).
    ///
    /// Failing that will result in reading and interpreting data outside the given
    /// array.
    unsafe fn decode_ptr(data: *const u8) -> (Item, usize) {
        Self::start_decoding_ptr(data).complete_decoding()
    }

    /// Decodes the Item from bytes.
    ///
    /// Returns the decoded data and the number of bytes consumed to produce it.
    ///
    /// # Safety
    /// You are to check that data is not empty and that data.len() >=
    /// [DecodableWriteFileData.size()](struct.DecodableWriteFileData.html#method.size)
    /// (the expected byte size of the returned DecodableItem).
    ///
    /// Failing that will result in reading and interpreting data outside the given
    /// array.
    unsafe fn decode_unchecked(data: &[u8]) -> (Item, usize) {
        Self::start_decoding_unchecked(data).complete_decoding()
    }

    /// Decodes the item from bytes.
    ///
    /// On success, returns the decoded data and the number of bytes consumed
    /// to produce it.
    fn decode(data: &[u8]) -> Result<(Item, usize), Self::Error>;
}

pub trait DecodableBuffer<T> {
    /// Decodes the size of the Item in bytes
    fn size(&self) -> usize;

    /// Fully decode the Item
    ///
    /// Returns the decoded data and the number of bytes consumed to produce it.
    fn complete_decoding(&self) -> (T, usize);
}

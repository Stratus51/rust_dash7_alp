pub struct WithConsumedBytes<T> {
    pub item: T,
    pub consumed_bytes: usize,
}

pub trait EncodedData<'data, DecodedData: Sized + 'data> {
    fn from_data(data: &'data [u8]) -> Self;
    fn from_data_ptr(data: *const u8) -> Self;

    /// Decodes the size of the Item in bytes
    ///
    /// # Safety
    /// This might require reading data bytes that may be outside the valid data to be calculated.
    unsafe fn expected_size(&self) -> usize;

    /// Safely checks whether the given data_size is bigger than the decoded object expected size
    /// and return the expected size.
    ///
    /// # Errors
    /// Fails if the data_size is smaller than the required data size to decode the object.
    fn smaller_than(&self, data_size: usize) -> Result<usize, usize>;

    /// Fully decodes the Item.
    fn complete_decoding(&self) -> WithConsumedBytes<DecodedData>;
}

pub trait Decodable<'data>: Sized + 'data {
    type Data: EncodedData<'data, Self>;

    /// Creates an encoded item handle from a data pointer without checking the data size.
    ///
    /// This method is meant to allow unchecked cross language wrapper libraries
    /// to implement an unchecked call without having to build a fake slice with
    /// a fake size.
    ///
    /// It is not meant to be used inside a Rust library/binary.
    ///
    /// # Safety
    /// You are to check that:
    /// - The decodable object fits in the given data:
    /// [`decodable.smaller_than(data.len())`](struct.DecodableVarint.html#method.smaller_than)
    ///
    /// Failing that might result in reading and interpreting data outside the given
    /// array (depending on what is done with the resulting object).
    unsafe fn start_decoding_ptr(data: *const u8) -> Self::Data {
        Self::Data::from_data_ptr(data)
    }

    /// Creates an encoded item handle without checking the data size.
    ///
    /// # Safety
    /// You are to check that:
    /// - The decodable object fits in the given data:
    /// [`decodable.smaller_than(data.len())`](struct.DecodableVarint.html#method.smaller_than)
    ///
    /// Failing that might result in reading and interpreting data outside the given
    /// array (depending on what is done with the resulting object).
    unsafe fn start_decoding_unchecked(data: &'data [u8]) -> Self::Data {
        Self::Data::from_data(data)
    }

    /// Returns an encoded item handle.
    ///
    /// This decodable item allows each parts of the item to be decoded independently.
    ///
    /// # Errors
    /// - Fails if data is smaller then the decoded expected size.
    fn start_decoding(data: &'data [u8]) -> Result<WithConsumedBytes<Self::Data>, usize> {
        let ret = unsafe { Self::start_decoding_unchecked(data) };
        let size = ret.smaller_than(data.len())?;
        Ok(WithConsumedBytes {
            item: ret,
            consumed_bytes: size,
        })
    }

    /// Decodes an item from a data pointer.
    ///
    /// This method is meant to allow unchecked cross language wrapper libraries
    /// to implement an unchecked call without having to build a fake slice with
    /// a fake size.
    ///
    /// It is not meant to be used inside a Rust library/binary.
    ///
    /// # Safety
    /// May attempt to read bytes after the end of the array.
    ///
    /// You are to check that:
    /// - The decodable object fits in the given data:
    /// [`decodable.smaller_than(data.len())`](struct.DecodableVarint.html#method.smaller_than)
    ///
    /// Failing that will result in reading and interpreting data outside the given
    /// array.
    unsafe fn decode_ptr(data: *const u8) -> WithConsumedBytes<Self> {
        Self::start_decoding_ptr(data).complete_decoding()
    }

    /// Decodes an item from raw data.
    ///
    /// # Safety
    /// May attempt to read bytes after the end of the array.
    ///
    /// You are to check that:
    /// - The decodable object fits in the given data:
    /// [`decodable.smaller_than(data.len())`](struct.DecodableVarint.html#method.smaller_than)
    ///
    /// Failing that will result in reading and interpreting data outside the given
    /// array.
    unsafe fn decode_unchecked(data: &'data [u8]) -> WithConsumedBytes<Self> {
        Self::start_decoding_unchecked(data).complete_decoding()
    }

    /// Decodes an item from raw data.
    ///
    /// # Errors
    /// Fails if the input data is too small to decode and requires the minimum
    /// number of bytes required to continue decoding.
    fn decode(data: &'data [u8]) -> Result<WithConsumedBytes<Self>, usize> {
        match Self::start_decoding(data) {
            Ok(v) => Ok(v.item.complete_decoding()),
            Err(e) => Err(e),
        }
    }
}

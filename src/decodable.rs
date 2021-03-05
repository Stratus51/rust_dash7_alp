#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct WithByteSize<T> {
    pub item: T,
    pub byte_size: usize,
}

impl<T> WithByteSize<T> {
    pub fn map<U, F>(self, f: F) -> WithByteSize<U>
    where
        F: Fn(T) -> U,
    {
        let Self { item, byte_size } = self;
        WithByteSize {
            item: f(item),
            byte_size,
        }
    }
}

/// Array of bytes that represents an item (DecodedData)
pub trait EncodedData<'data> {
    type DecodedData: Sized + 'data;

    /// # Safety
    /// Requires the data to contain at least one byte.
    unsafe fn from_data_ref(data: &'data [u8]) -> Self;

    /// # Safety
    /// Requires the data to contain at least one byte.
    unsafe fn from_data_ptr(data: *const u8) -> Self;

    /// Decodes the size of the item in bytes
    ///
    /// # Safety
    /// This might require reading data bytes that may be outside the valid data to be calculated.
    unsafe fn expected_size(&self) -> usize;

    /// Safely checks whether the given data_size is bigger than the decoded object expected size
    /// and return the expected size.
    ///
    /// # Errors
    /// Fails if the data_size is smaller than the required data size to decode the object.
    /// On error, returns the minimum bytes required to continue parsing the size of this item.
    fn smaller_than(&self, data_size: usize) -> Result<usize, usize>;

    /// Fully decodes the Item.
    fn complete_decoding(&self) -> WithByteSize<Self::DecodedData>;
}

/// Item that can always be decoded from bytes (provided there is enough data)
pub trait Decodable<'data>: Sized + 'data {
    type Data: EncodedData<'data, DecodedData = Self>;

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
    /// - The data is not empty.
    /// - The data verify some item specific size conditions if any are stated (like on
    /// [StatusRef](../v1_2/action/status/enum.StatusRef.html#impl-FailableDecodable%3C%27data%3E)).
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
        Self::Data::from_data_ref(data)
    }

    /// Returns an encoded item handle.
    ///
    /// This decodable item allows each parts of the item to be decoded independently.
    ///
    /// # Errors
    /// - Fails if data is smaller then the decoded expected size.
    fn start_decoding(data: &'data [u8]) -> Result<WithByteSize<Self::Data>, usize> {
        let ret = unsafe { Self::start_decoding_unchecked(data) };
        let size = ret.smaller_than(data.len())?;
        Ok(WithByteSize {
            item: ret,
            byte_size: size,
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
    unsafe fn decode_ptr(data: *const u8) -> WithByteSize<Self> {
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
    unsafe fn decode_unchecked(data: &'data [u8]) -> WithByteSize<Self> {
        Self::start_decoding_unchecked(data).complete_decoding()
    }

    /// Decodes an item from raw data.
    ///
    /// # Errors
    /// Fails if the input data is too small to decode and requires the minimum
    /// number of bytes required to continue decoding.
    fn decode(data: &'data [u8]) -> Result<WithByteSize<Self>, usize> {
        Self::start_decoding(data).map(|v| v.item.complete_decoding())
    }
}

/// Array of bytes that represents an item (DecodedData)
pub trait FailableEncodedData<'data> {
    type RefError: Clone + Eq + PartialEq + Ord + PartialOrd + core::hash::Hash + core::fmt::Debug;
    type PtrError: Clone + Eq + PartialEq + Ord + PartialOrd + core::hash::Hash + core::fmt::Debug;
    type DecodedData: Sized + 'data;

    /// Parse the first byte to check whether that data is parseable.
    ///
    /// # Safety
    /// Requires the data to contain at least one byte.
    ///
    /// # Errors
    /// The data is not parseable.
    unsafe fn from_data_ref(data: &'data [u8]) -> Result<Self, Self::RefError>
    where
        Self: Sized;

    /// Parse the first byte to check whether that data is parseable.
    ///
    /// # Safety
    /// Requires the data to contain at least one byte.
    ///
    /// # Errors
    /// The data is not parseable.
    unsafe fn from_data_ptr(data: *const u8) -> Result<Self, Self::PtrError>
    where
        Self: Sized;

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
    /// On error, returns the minimum bytes required to continue parsing the size of this item.
    fn smaller_than(&self, data_size: usize) -> Result<usize, usize>;

    /// Fully decodes the Item.
    fn complete_decoding(&self) -> WithByteSize<Self::DecodedData>;
}

pub type FailableEncodedDataRefError<'data, T> = <T as FailableEncodedData<'data>>::RefError;
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum FailableDecodingRefError<'data, T: FailableEncodedData<'data, DecodedData = U>, U> {
    DataSize(usize),
    Decode(FailableEncodedDataRefError<'data, T>),
}
impl<'data, T: FailableEncodedData<'data, DecodedData = U>, U> core::fmt::Debug
    for FailableDecodingRefError<'data, T, U>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::DataSize(size) => write!(f, "DataSize({})", size),
            Self::Decode(error) => write!(f, "Decode({:?})", error),
        }
    }
}

pub type FailableEncodedDataPtrError<'data, T> = <T as FailableEncodedData<'data>>::PtrError;
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum FailableDecodingPtrError<'data, T: FailableEncodedData<'data, DecodedData = U>, U> {
    DataSize(usize),
    Decode(FailableEncodedDataPtrError<'data, T>),
}

/// Item that may be decoded from coherent bytes.
pub trait FailableDecodable<'data>: Sized + 'data {
    type Data: FailableEncodedData<'data, DecodedData = Self>;

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
    /// - The data is not empty.
    /// - The data verify some item specific size conditions if any are stated (like on
    /// [StatusRef](../v1_2/action/status/enum.StatusRef.html#impl-FailableDecodable%3C%27data%3E)).
    /// - The decodable object fits in the given data:
    /// [`decodable.smaller_than(data.len())`](struct.DecodableVarint.html#method.smaller_than)
    ///
    /// Failing that might result in reading and interpreting data outside the given
    /// array (depending on what is done with the resulting object).
    ///
    /// # Errors
    /// Fails if the data is not parseable.
    unsafe fn start_decoding_ptr(
        data: *const u8,
    ) -> Result<Self::Data, FailableEncodedDataPtrError<'data, Self::Data>> {
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
    ///
    /// # Errors
    /// Fails if the data is not parseable.
    unsafe fn start_decoding_unchecked(
        data: &'data [u8],
    ) -> Result<Self::Data, FailableEncodedDataRefError<'data, Self::Data>> {
        Self::Data::from_data_ref(data)
    }

    /// Returns an encoded item handle.
    ///
    /// This decodable item allows each parts of the item to be decoded independently.
    ///
    /// # Errors
    /// - Fails if data is smaller then the decoded expected size.
    /// - Fails if the data is not parseable.
    fn start_decoding(
        data: &'data [u8],
    ) -> Result<WithByteSize<Self::Data>, FailableDecodingRefError<'data, Self::Data, Self>> {
        let ret = unsafe {
            Self::start_decoding_unchecked(data).map_err(FailableDecodingRefError::Decode)?
        };
        let size = ret
            .smaller_than(data.len())
            .map_err(FailableDecodingRefError::DataSize)?;
        Ok(WithByteSize {
            item: ret,
            byte_size: size,
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
    ///
    /// # Errors
    /// Fails if the data is not parseable.
    unsafe fn decode_ptr(
        data: *const u8,
    ) -> Result<WithByteSize<Self>, FailableEncodedDataPtrError<'data, Self::Data>> {
        Self::start_decoding_ptr(data).map(|v| v.complete_decoding())
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
    ///
    /// # Errors
    /// Fails if the data is not parseable.
    unsafe fn decode_unchecked(
        data: &'data [u8],
    ) -> Result<WithByteSize<Self>, FailableEncodedDataRefError<'data, Self::Data>> {
        Self::start_decoding_unchecked(data).map(|v| v.complete_decoding())
    }

    /// Decodes an item from raw data.
    ///
    /// # Errors
    /// - Fails if the input data is too small to decode and requires the minimum
    /// number of bytes required to continue decoding.
    /// - Fails if the data is not parseable.
    fn decode(
        data: &'data [u8],
    ) -> Result<WithByteSize<Self>, FailableDecodingRefError<'data, Self::Data, Self>> {
        Self::start_decoding(data).map(|v| v.item.complete_decoding())
    }
}
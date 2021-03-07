#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum SizeError {
    MissingBytes,
}

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
    /// This method was not made to be called directly. Please see the [Decodable](trait.Decodable)
    /// API.
    unsafe fn new(data: &'data [u8]) -> Self;

    /// Safely calculates what the size in bytes of the item we are decoding should be.
    ///
    /// # Errors
    /// Fails if:
    /// - The data is too small.
    /// - The decoded item contains unsupported elements.
    /// - The data is not coherent.
    fn encoded_size(&self) -> Result<usize, SizeError>;

    /// Fully decodes the Item.
    fn complete_decoding(&self) -> WithByteSize<Self::DecodedData>;
}

/// Item that can always be decoded from bytes (provided there is enough data)
pub trait Decodable<'data>: Sized + 'data {
    type Data: EncodedData<'data, DecodedData = Self>;

    /// Creates an encoded item handle without checking the data size.
    ///
    /// # Safety
    /// You are to check that:
    /// - The decodable object fits in the given data:
    /// [`decodable.encoded_size()`](trait.Decodable.html#method.size)
    ///
    /// Failing that might result in reading and interpreting data outside the given
    /// array (depending on what is done with the resulting object).
    unsafe fn start_decoding_unchecked(data: &'data [u8]) -> Self::Data {
        Self::Data::new(data)
    }

    /// Returns an encoded item handle.
    ///
    /// This encoded item handle allows each parts of the item to be decoded independently.
    ///
    /// # Errors
    /// - Fails if data is smaller then the decoded item's expected size.
    fn start_decoding(data: &'data [u8]) -> Result<WithByteSize<Self::Data>, SizeError> {
        let ret = unsafe { Self::start_decoding_unchecked(data) };
        let size = ret.encoded_size()?;
        if size > data.len() {
            return Err(SizeError::MissingBytes);
        }
        Ok(WithByteSize {
            item: ret,
            byte_size: size,
        })
    }

    /// Decodes an item from raw data.
    ///
    /// # Safety
    /// May attempt to read bytes after the end of the array.
    ///
    /// You are to check that:
    /// - The decodable object fits in the given data:
    /// [`decodable.encoded_size()`](struct.Decodable.html#method.size)
    ///
    /// Failing that will result in reading and interpreting data outside the given
    /// array.
    unsafe fn decode_unchecked(data: &'data [u8]) -> WithByteSize<Self> {
        Self::start_decoding_unchecked(data).complete_decoding()
    }

    /// Decodes an item from raw data.
    ///
    /// # Errors
    /// Fails if the input data is too small to decode.
    fn decode(data: &'data [u8]) -> Result<WithByteSize<Self>, SizeError> {
        Self::start_decoding(data).map(|v| v.item.complete_decoding())
    }
}

pub trait MissingByteErrorBuilder {
    fn missing_bytes() -> Self;
}

/// Array of bytes that represents an item (DecodedData)
pub trait FailableEncodedData<'data> {
    type DecodeError: Clone
        + Eq
        + PartialEq
        + Ord
        + PartialOrd
        + core::hash::Hash
        + core::fmt::Debug;
    type SizeError: Clone
        + Eq
        + PartialEq
        + Ord
        + PartialOrd
        + core::hash::Hash
        + core::fmt::Debug
        + MissingByteErrorBuilder;
    type DecodedData: Sized + 'data;

    /// # Safety
    /// This method was not made to be called directly. Please see the [Decodable](trait.Decodable)
    /// API.
    unsafe fn new(data: &'data [u8]) -> Self;

    /// Safely calculates what the size in bytes of the item we are decoding should be.
    ///
    /// # Errors
    /// Fails if:
    /// - The data is too small.
    /// - The decoded item contains unsupported elements.
    /// - The data is not coherent.
    fn encoded_size(&self) -> Result<usize, Self::SizeError>;

    /// Fully decodes the Item.
    ///
    /// # Errors
    /// Fails if the data is not decodable.
    /// This currently means that either the data is incoherent or it contains unsupported
    /// items.
    ///
    /// This method does not check data boundaries, as it assumes those have been previously done.
    fn complete_decoding(&self) -> Result<WithByteSize<Self::DecodedData>, Self::DecodeError>;
}

pub type FailableEncodedDataSizeError<'data, T> = <T as FailableEncodedData<'data>>::SizeError;
pub type FailableEncodedDataDecodeError<'data, T> = <T as FailableEncodedData<'data>>::DecodeError;

/// Item that may be decoded from coherent bytes.
pub trait FailableDecodable<'data>: Sized + 'data {
    type Data: FailableEncodedData<'data, DecodedData = Self>;
    type FullDecodeError: Clone
        + Eq
        + PartialEq
        + Ord
        + PartialOrd
        + core::hash::Hash
        + core::fmt::Debug
        + From<FailableEncodedDataSizeError<'data, Self::Data>>
        + From<FailableEncodedDataDecodeError<'data, Self::Data>>;

    /// Creates an encoded item handle without checking the data size.
    ///
    /// # Safety
    /// You are to check that:
    /// - The decodable object fits in the given data:
    /// [`decodable.encoded_size()`](trait.Decodable.html#method.size)
    ///
    /// Failing that might result in reading and interpreting data outside the given
    /// array (depending on what is done with the resulting object).
    ///
    /// # Errors
    /// Fails if the data is not parseable.
    unsafe fn start_decoding_unchecked(data: &'data [u8]) -> Self::Data {
        Self::Data::new(data)
    }

    /// Returns an encoded item handle.
    ///
    /// This encoded item handle allows each parts of the item to be decoded independently.
    ///
    /// # Errors
    /// - Fails if data is smaller then the decoded expected size.
    /// - Fails if the data is not parseable.
    fn start_decoding(
        data: &'data [u8],
    ) -> Result<WithByteSize<Self::Data>, FailableEncodedDataSizeError<'data, Self::Data>> {
        let ret = unsafe { Self::start_decoding_unchecked(data) };
        let size = ret.encoded_size()?;
        if size > data.len() {
            return Err(<FailableEncodedDataSizeError<'data, Self::Data> as MissingByteErrorBuilder>::missing_bytes());
        }
        Ok(WithByteSize {
            item: ret,
            byte_size: size,
        })
    }

    /// Decodes an item from raw data.
    ///
    /// # Safety
    /// May attempt to read bytes after the end of the array.
    ///
    /// You are to check that:
    /// - The decodable object fits in the given data:
    /// [`decodable.encoded_size()`](struct.Decodable.html#method.size)
    ///
    /// Failing that will result in reading and interpreting data outside the given
    /// array.
    ///
    /// # Errors
    /// Fails if the data is not parseable.
    unsafe fn decode_unchecked(
        data: &'data [u8],
    ) -> Result<WithByteSize<Self>, FailableEncodedDataDecodeError<'data, Self::Data>> {
        Self::start_decoding_unchecked(data).complete_decoding()
    }

    /// Decodes an item from raw data.
    ///
    /// # Errors
    /// - Fails if the input data is too small to decode.
    /// - Fails if the data is not parseable.
    fn decode(data: &'data [u8]) -> Result<WithByteSize<Self>, Self::FullDecodeError> {
        Self::start_decoding(data)
            .map_err(|e| e.into())
            .and_then(|v| v.item.complete_decoding().map_err(|e| e.into()))
    }
}

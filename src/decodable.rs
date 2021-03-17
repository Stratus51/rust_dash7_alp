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

#[macro_export]
macro_rules! make_decodable {
    ($struct_name:ident, $encoded_data:ident, $encoded_data_mut:ident) => {
        impl<'item, 'data> $struct_name<'item, 'data> {
            /// Creates an encoded item handle without checking the data size.
            ///
            /// # Safety
            /// You are to check that:
            /// - The decodable object fits in the given data:
            /// [`decodable.encoded_size()`](trait.Decodable.html#method.size)
            ///
            /// Failing that might result in reading and interpreting data outside the given
            /// array (depending on what is done with the resulting object).
            pub unsafe fn start_decoding_unchecked<'result>(
                data: &'data [u8],
            ) -> $encoded_data<'result, 'data> {
                $encoded_data::new(data)
            }

            /// Returns an encoded item handle.
            ///
            /// This encoded item handle allows each parts of the item to be decoded independently.
            ///
            /// # Errors
            /// - Fails if data is smaller then the decoded item's expected size.
            pub fn start_decoding<'result>(
                data: &'data [u8],
            ) -> Result<WithByteSize<$encoded_data<'result, 'data>>, SizeError> {
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

            /// Creates a mutable encoded item handle without checking the data size.
            ///
            /// # Safety
            /// You are to check that:
            /// - The decodable object fits in the given data:
            /// [`decodable.encoded_size()`](trait.Decodable.html#method.size)
            ///
            /// Failing that might result in reading and interpreting data outside the given
            /// array (depending on what is done with the resulting object).
            pub unsafe fn start_decoding_unchecked_mut<'result>(
                data: &'data mut [u8],
            ) -> $encoded_data_mut<'result, 'data> {
                $encoded_data_mut::new(data)
            }

            /// Returns a mutable encoded item handle.
            ///
            /// This encoded item handle allows each parts of the item to be decoded independently.
            ///
            /// # Errors
            /// - Fails if data is smaller then the decoded item's expected size.
            pub fn start_decoding_mut<'result>(
                data: &'data mut [u8],
            ) -> Result<WithByteSize<$encoded_data_mut<'result, 'data>>, SizeError> {
                let ret = unsafe { Self::start_decoding_unchecked_mut(data) };
                let size = ret.encoded_size()?;
                if size > data.len() {
                    return Err(SizeError::MissingBytes);
                }
                Ok(WithByteSize {
                    item: ret,
                    byte_size: size,
                })
            }

            // TODO Should a mut encodable result in a mut ref object?

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
            pub unsafe fn decode_unchecked<'result>(
                data: &'data [u8],
            ) -> WithByteSize<$struct_name<'result, 'data>> {
                Self::start_decoding_unchecked(data).complete_decoding()
            }

            /// Decodes an item from raw data.
            ///
            /// # Errors
            /// Fails if the input data is too small to decode.
            pub fn decode<'result>(
                data: &'data [u8],
            ) -> Result<WithByteSize<$struct_name<'result, 'data>>, SizeError> {
                Self::start_decoding(data).map(|v| v.item.complete_decoding())
            }
        }
    };
}

pub trait MissingByteErrorBuilder {
    fn missing_bytes() -> Self;
}

#[macro_export]
macro_rules! make_failable_decodable {
    ($struct_name:ident, $encoded_data:ident, $encoded_data_mut:ident,
     $size_error: ident, $decode_error: ident, $full_error: ident) => {
        use crate::decodable::{ MissingByteErrorBuilder};
        impl<'item, 'data> $struct_name<'item, 'data> {
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
        pub unsafe fn start_decoding_unchecked<'result>(data: &'data [u8]) -> $encoded_data<'result, 'data> {
            $encoded_data::new(data)
        }

        /// Returns an encoded item handle.
        ///
        /// This encoded item handle allows each parts of the item to be decoded independently.
        ///
        /// # Errors
        /// - Fails if data is smaller then the decoded expected size.
        /// - Fails if the data is not parseable.
        pub fn start_decoding<'result>(
            data: &'data [u8],
        ) -> Result<WithByteSize<$encoded_data<'result, 'data>>, $size_error<'result, 'data>>
        {
            let ret = unsafe { Self::start_decoding_unchecked(data) };
            let size = ret.encoded_size()?;
            if size > data.len() {
                return Err(<$size_error as MissingByteErrorBuilder>::missing_bytes());
            }
            Ok(WithByteSize {
                item: ret,
                byte_size: size,
            })
        }

        /// Creates a mutable encoded item handle without checking the data size.
        ///
        /// # Safety
        /// You are to check that:
        /// - The decodable object fits in the given data:
        /// [`decodable.encoded_size()`](trait.Decodable.html#method.size)
        ///
        /// Failing that might result in reading and interpreting data outside the given
        /// array (depending on what is done with the resulting object).
        pub unsafe fn start_decoding_unchecked_mut<'result>(data: &'data mut [u8])
            -> $encoded_data_mut<'result, 'data> {
            $encoded_data_mut::new(data)
        }

        /// Returns a mutable encoded item handle.
        ///
        /// This encoded item handle allows each parts of the item to be decoded independently.
        ///
        /// # Errors
        /// - Fails if data is smaller then the decoded item's expected size.
        pub fn start_decoding_mut<'result>(
            data: &'data mut [u8],
        ) -> Result<
            WithByteSize<$encoded_data_mut<'result, 'data>>,
            $size_error<'result, 'data>,
        > {
            let ret = unsafe { Self::start_decoding_unchecked_mut(data) };
            let size = ret.encoded_size()?;
            if size > data.len() {
                return Err(<$size_error as MissingByteErrorBuilder>::missing_bytes());
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
        pub unsafe fn decode_unchecked<'result>(
            data: &'data [u8],
        ) -> Result<WithByteSize<$struct_name<'result, 'data>>, $decode_error<'result, 'data>>
        {
            Self::start_decoding_unchecked(data).complete_decoding()
        }

        /// Decodes an item from raw data.
        ///
        /// # Errors
        /// - Fails if the input data is too small to decode.
        /// - Fails if the data is not parseable.
        pub fn decode<'result>(data: &'data [u8])
            -> Result<WithByteSize<$struct_name<'result, 'data>>, $full_error<'result, 'data>> {
            Self::start_decoding(data)
                .map_err(|e| e.into())
                .and_then(|v| v.item.complete_decoding().map_err(|e| e.into()))
        }
        }
    }
}

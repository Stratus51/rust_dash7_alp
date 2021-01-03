pub mod action_query;
pub mod break_query;
pub mod comparison_with_range;
pub mod comparison_with_value;
pub mod define;

pub use action_query::{ActionQuery, DecodableActionQuery};
pub use comparison_with_range::{ComparisonWithRange, DecodableComparisonWithRange};
pub use comparison_with_value::{ComparisonWithValue, DecodableComparisonWithValue};

use super::super::error::QueryDecodeError;
use define::QueryCode;

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Query<'item> {
    // NonVoid(non_void::NonVoid),
    // ComparisonWithZero(ComparisonWithZero),
    ComparisonWithValue(ComparisonWithValue<'item>),
    // ComparisonWithOtherFile(ComparisonWithOtherFile),
    ComparisonWithRange(ComparisonWithRange<'item>),
    // StringTokenSearch(StringTokenSearch),
}

impl<'item> Query<'item> {
    /// Encodes the Item into a data pointer without checking the size of the
    /// receiving byte array.
    ///
    /// This method is meant to allow unchecked cross language wrapper libraries
    /// to implement an unchecked call without having to build a fake slice with
    /// a fake size.
    ///
    /// It is not meant to be used inside a Rust library/binary.
    ///
    /// # Safety
    /// You are responsible for checking that `out.len()` >= [`self.size()`](#method.size).
    ///
    /// Failing that will result in the program writing out of bound in
    /// random parts of your memory.
    pub unsafe fn encode_in_ptr(&self, out: *mut u8) -> usize {
        match self {
            Self::ComparisonWithValue(query) => query.encode_in_ptr(out),
            Self::ComparisonWithRange(query) => query.encode_in_ptr(out),
        }
    }

    /// Encodes the Item without checking the size of the receiving
    /// byte array.
    ///
    /// # Safety
    /// You are responsible for checking that `out.len()` >= [`self.size()`](#method.size).
    ///
    /// Failing that will result in the program writing out of bound in
    /// random parts of your memory.
    pub unsafe fn encode_in_unchecked(&self, out: &mut [u8]) -> usize {
        self.encode_in_ptr(out.as_mut_ptr())
    }

    /// Encodes the value into pre allocated array.
    ///
    /// # Errors
    /// Fails if the pre allocated array is smaller than [self.size()](#method.size)
    /// returning the number of input bytes required.
    pub fn encode_in(&self, out: &mut [u8]) -> Result<usize, usize> {
        let size = self.size();
        if out.len() >= size {
            Ok(unsafe { self.encode_in_ptr(out.as_mut_ptr()) })
        } else {
            Err(size)
        }
    }

    /// Size in bytes of the encoded equivalent of the item.
    pub fn size(&self) -> usize {
        match self {
            Self::ComparisonWithValue(query) => query.size(),
            Self::ComparisonWithRange(query) => query.size(),
        }
    }

    /// Creates a decodable item from a data pointer without checking the data size.
    ///
    /// This method is meant to allow unchecked cross language wrapper libraries
    /// to implement an unchecked call without having to build a fake slice with
    /// a fake size.
    ///
    /// It is not meant to be used inside a Rust library/binary.
    ///
    /// # Errors
    /// Fails if the decoded data contains an invalid querycode. Returning the querycode.
    ///
    /// # Safety
    /// You are to check that:
    /// - The decodable object fits in the given data:
    /// [`decodable.smaller_than(data.len())`](struct.DecodableQuery.html#method.smaller_than)
    ///
    /// Failing that might result in reading and interpreting data outside the given
    /// array (depending on what is done with the resulting object).
    pub unsafe fn start_decoding_ptr<'data>(data: *const u8) -> Result<DecodableQuery<'data>, u8> {
        DecodableQuery::from_ptr(data)
    }

    /// Creates a decodable item without checking the data size.
    ///
    /// # Errors
    /// Fails if the decoded data contains an invalid querycode. Returning the querycode.
    ///
    /// # Safety
    /// You are to check that:
    /// - The decodable object fits in the given data:
    /// [`decodable.smaller_than(data.len())`](struct.DecodableQuery.html#method.smaller_than)
    ///
    /// Failing that might result in reading and interpreting data outside the given
    /// array (depending on what is done with the resulting object).
    pub unsafe fn start_decoding_unchecked(data: &[u8]) -> Result<DecodableQuery, u8> {
        DecodableQuery::new(data)
    }

    /// Returns a Decodable object and its expected byte size.
    ///
    /// This decodable item allows each parts of the item to be decoded independently.
    ///
    /// # Errors
    /// - Fails if first byte of the data contains an invalid querycode.
    /// - Fails if data is smaller then the decoded expected size.
    pub fn start_decoding(data: &[u8]) -> Result<(DecodableQuery, usize), QueryDecodeError> {
        if data.is_empty() {
            return Err(QueryDecodeError::MissingBytes(1));
        }
        let ret = unsafe {
            Self::start_decoding_unchecked(data).map_err(QueryDecodeError::UnknownQueryCode)?
        };
        let size = ret
            .smaller_than(data.len())
            .map_err(QueryDecodeError::MissingBytes)?;
        Ok((ret, size))
    }

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
    /// # Errors
    /// Fails if first byte of the data contains an invalid querycode.
    ///
    /// # Safety
    /// You are to check that:
    /// - The first byte contains this action's querycode.
    /// - The resulting size of the data consumed is smaller than the size of the
    /// decoded data.
    ///
    /// Failing that might result in reading and interpreting data outside the given
    /// array (depending on what is done with the resulting object).
    pub unsafe fn decode_ptr(data: *const u8) -> Result<(Self, usize), QueryDecodeError> {
        Ok(Self::start_decoding_ptr(data)
            .map_err(QueryDecodeError::UnknownQueryCode)?
            .complete_decoding())
    }

    /// Decodes the Item from bytes.
    ///
    /// Returns the decoded data and the number of bytes consumed to produce it.
    ///
    /// # Errors
    /// Fails if first byte of the data contains an invalid querycode.
    ///
    /// # Safety
    /// You are to check that:
    /// - The first byte contains this action's querycode.
    /// - The resulting size of the data consumed is smaller than the size of the
    /// decoded data.
    ///
    /// Failing that might result in reading and interpreting data outside the given
    /// array (depending on what is done with the resulting object).
    pub unsafe fn decode_unchecked(data: &'item [u8]) -> Result<(Self, usize), QueryDecodeError> {
        Ok(Self::start_decoding_unchecked(data)
            .map_err(QueryDecodeError::UnknownQueryCode)?
            .complete_decoding())
    }

    /// Decodes the item from bytes.
    ///
    /// # Errors
    /// - Fails if first byte of the data contains an invalid querycode.
    /// - Fails if data is smaller then the decoded expected size.
    pub fn decode(data: &'item [u8]) -> Result<(Self, usize), QueryDecodeError> {
        Ok(Self::start_decoding(data)?.0.complete_decoding())
    }
}

pub enum DecodableQuery<'data> {
    // NonVoid(non_void::NonVoid),
    // ComparisonWithZero(ComparisonWithZero),
    ComparisonWithValue(DecodableComparisonWithValue<'data>),
    // ComparisonWithOtherFile(ComparisonWithOtherFile),
    ComparisonWithRange(DecodableComparisonWithRange<'data>),
    // StringTokenSearch(StringTokenSearch),
}

impl<'data> DecodableQuery<'data> {
    /// # Errors
    /// Fails if the querycode is invalid. Returning the querycode.
    ///
    /// # Safety
    /// The data has to contain at least one byte.
    pub unsafe fn new(data: &'data [u8]) -> Result<Self, u8> {
        Self::from_ptr(data.as_ptr())
    }

    /// # Errors
    /// Fails if the querycode is invalid. Returning the querycode.
    ///
    /// # Safety
    /// The data has to contain at least one byte.
    unsafe fn from_ptr(data: *const u8) -> Result<Self, u8> {
        let code = (*data.offset(0) >> 5) & 0x07;
        let query_code = match QueryCode::from(code) {
            Ok(code) => code,
            Err(_) => return Err(code),
        };
        Ok(match query_code {
            QueryCode::ComparisonWithValue => {
                DecodableQuery::ComparisonWithValue(ComparisonWithValue::start_decoding_ptr(data))
            }
            QueryCode::ComparisonWithRange => {
                DecodableQuery::ComparisonWithRange(ComparisonWithRange::start_decoding_ptr(data))
            }
        })
    }

    /// Decodes the size of the Item in bytes
    ///
    /// # Safety
    /// This requires reading the data bytes that may be out of bound to be calculate.
    pub unsafe fn expected_size(&self) -> usize {
        match self {
            Self::ComparisonWithValue(d) => d.expected_size(),
            Self::ComparisonWithRange(d) => d.expected_size(),
        }
    }

    /// Checks whether the given data_size is bigger than the decoded object expected size.
    ///
    /// On success, returns the size of the decoded object.
    ///
    /// # Errors
    /// Fails if the data_size is smaller than the required data size to decode the object.
    pub fn smaller_than(&self, data_size: usize) -> Result<usize, usize> {
        match self {
            Self::ComparisonWithValue(d) => d.smaller_than(data_size),
            Self::ComparisonWithRange(d) => d.smaller_than(data_size),
        }
    }

    /// Fully decode the Item
    ///
    /// Returns the decoded data and the number of bytes consumed to produce it.
    pub fn complete_decoding(&self) -> (Query<'data>, usize) {
        match self {
            Self::ComparisonWithValue(d) => {
                let (op, size) = d.complete_decoding();
                (Query::ComparisonWithValue(op), size)
            }
            Self::ComparisonWithRange(d) => {
                let (op, size) = d.complete_decoding();
                (Query::ComparisonWithRange(op), size)
            }
        }
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::unwrap_in_result, clippy::panic, clippy::expect_used)]
    use super::define::QueryComparisonType;
    use super::*;
    use crate::define::{EncodableData, FileId, MaskedValue};
    use crate::varint::Varint;

    #[test]
    fn known() {
        fn test(op: Query, data: &[u8]) {
            // Test op.encode_in() == data
            let mut encoded = [0_u8; 64];
            let size = op.encode_in(&mut encoded).unwrap();
            assert_eq!(size, data.len());
            assert_eq!(&encoded[..size], data);

            // Test decode(data) == op
            let (ret, size) = Query::decode(data).unwrap();
            assert_eq!(size, data.len());
            assert_eq!(ret, op);

            // Test partial_decode == op
            let (decoder, expected_size) = Query::start_decoding(data).unwrap();
            assert_eq!(expected_size, size);
            assert_eq!(unsafe { decoder.expected_size() }, size);
            assert_eq!(decoder.smaller_than(data.len()).unwrap(), size);
        }
        test(
            Query::ComparisonWithValue(ComparisonWithValue {
                signed_data: false,
                comparison_type: QueryComparisonType::GreaterThan,
                compare_value: MaskedValue::new(
                    EncodableData::new(&[0x0A, 0x0B, 0x0C, 0x0D]).unwrap(),
                    Some(&[0x00, 0xFF, 0x0F, 0xFF]),
                )
                .unwrap(),
                file_id: FileId::new(0x88),
                offset: Varint::new(0xFF).unwrap(),
            }),
            &[
                0x40 | 0x10 | 0x04,
                0x04,
                0x00,
                0xFF,
                0x0F,
                0xFF,
                0x0A,
                0x0B,
                0x0C,
                0x0D,
                0x88,
                0x40,
                0xFF,
            ],
        );
        test(
            Query::ComparisonWithRange(ComparisonWithRange {
                signed_data: false,
                comparison_type: define::QueryRangeComparisonType::NotInRange,
                range: crate::define::MaskedRange::new(50, 66, Some(&[0x33, 0x22])).unwrap(),
                file_id: FileId::new(0x88),
                offset: Varint::new(0xFF).unwrap(),
            }),
            &[0x80 | 0x10, 0x01, 50, 66, 0x33, 0x22, 0x88, 0x40, 0xFF],
        );
    }

    #[test]
    fn consistence() {
        const TOT_SIZE: usize = 1 + 1 + 3 + 3 + 1 + 3;
        let op = Query::ComparisonWithValue(ComparisonWithValue {
            signed_data: true,
            comparison_type: QueryComparisonType::GreaterThanOrEqual,
            compare_value: MaskedValue::new(
                EncodableData::new(&[0x00, 0x43, 0x02]).unwrap(),
                Some(&[0x44, 0x88, 0x11]),
            )
            .unwrap(),
            file_id: FileId::new(0xFF),
            offset: Varint::new(0x3F_FF_00).unwrap(),
        });

        // Test decode(op.encode_in()) == op
        let mut encoded = [0_u8; TOT_SIZE];
        let size_encoded = op.encode_in(&mut encoded).unwrap();
        let (ret, size_decoded) = Query::decode(&encoded).unwrap();
        assert_eq!(size_encoded, size_decoded);
        assert_eq!(ret, op);

        // Test decode(data).encode_in() == data
        let mut encoded2 = [0_u8; TOT_SIZE];
        let size_encoded2 = op.encode_in(&mut encoded2).unwrap();
        assert_eq!(size_encoded, size_encoded2);
        assert_eq!(encoded2[..size_encoded2], encoded[..size_encoded]);
    }
}

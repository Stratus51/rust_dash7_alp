pub mod action_query;
pub mod break_query;
pub mod comparison_with_range;
pub mod comparison_with_value;
pub mod define;

#[cfg(feature = "query_compare_with_range")]
use comparison_with_range::ComparisonWithRangeRef;
#[cfg(feature = "decode_query_compare_with_range")]
use comparison_with_range::EncodedComparisonWithRange;
#[cfg(feature = "query_compare_with_value")]
use comparison_with_value::ComparisonWithValueRef;
#[cfg(feature = "decode_query_compare_with_value")]
use comparison_with_value::EncodedComparisonWithValue;

#[cfg(feature = "query_compare_with_range")]
#[cfg(feature = "alloc")]
use comparison_with_range::ComparisonWithRange;
#[cfg(feature = "query_compare_with_value")]
#[cfg(feature = "alloc")]
use comparison_with_value::ComparisonWithValue;

#[cfg(feature = "decode_query")]
use define::QueryCode;

#[cfg(feature = "decode_query")]
use crate::decodable::{
    Decodable, EncodedData, FailableDecodable, FailableEncodedData, SizeError, WithByteSize,
};
#[cfg(feature = "decode_query")]
use crate::v1_2::error::UnknownQueryCode;

#[cfg(feature = "query")]
#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum QueryRef<'item> {
    // NonVoid(non_void::NonVoid),
    // ComparisonWithZero(ComparisonWithZero),
    #[cfg(feature = "query_compare_with_value")]
    ComparisonWithValue(ComparisonWithValueRef<'item>),
    // ComparisonWithOtherFile(ComparisonWithOtherFile),
    #[cfg(feature = "query_compare_with_range")]
    ComparisonWithRange(ComparisonWithRangeRef<'item>),
    // StringTokenSearch(StringTokenSearch),
}

#[cfg(feature = "query")]
impl<'item> QueryRef<'item> {
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
            #[cfg(feature = "query_compare_with_value")]
            Self::ComparisonWithValue(query) => query.encode_in_ptr(out),
            #[cfg(feature = "query_compare_with_range")]
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
            #[cfg(feature = "query_compare_with_value")]
            Self::ComparisonWithValue(query) => query.size(),
            #[cfg(feature = "query_compare_with_range")]
            Self::ComparisonWithRange(query) => query.size(),
        }
    }

    // TODO Move inside when comparison without alloc exists
    #[cfg(feature = "alloc")]
    pub fn to_owned(&self) -> Query {
        match self {
            #[cfg(feature = "query_compare_with_value")]
            Self::ComparisonWithValue(query) => Query::ComparisonWithValue(query.to_owned()),
            #[cfg(feature = "query_compare_with_range")]
            Self::ComparisonWithRange(query) => Query::ComparisonWithRange(query.to_owned()),
        }
    }
}

#[cfg(feature = "decode_query")]
#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum DecodedQueryRef<'item> {
    // NonVoid(non_void::NonVoid),
    // ComparisonWithZero(ComparisonWithZero),
    #[cfg(feature = "decode_query_compare_with_value")]
    ComparisonWithValue(ComparisonWithValueRef<'item>),
    // ComparisonWithOtherFile(ComparisonWithOtherFile),
    #[cfg(feature = "decode_query_compare_with_range")]
    ComparisonWithRange(ComparisonWithRangeRef<'item>),
    // StringTokenSearch(StringTokenSearch),
}

#[cfg(feature = "decode_query")]
impl<'item> DecodedQueryRef<'item> {
    pub fn as_encodable(self) -> QueryRef<'item> {
        self.into()
    }
}

#[cfg(feature = "decode_query")]
impl<'item> From<DecodedQueryRef<'item>> for QueryRef<'item> {
    fn from(decoded: DecodedQueryRef<'item>) -> Self {
        match decoded {
            #[cfg(feature = "decode_query_compare_with_value")]
            DecodedQueryRef::ComparisonWithValue(query) => QueryRef::ComparisonWithValue(query),
            #[cfg(feature = "decode_query_compare_with_range")]
            DecodedQueryRef::ComparisonWithRange(query) => QueryRef::ComparisonWithRange(query),
        }
    }
}

#[cfg(feature = "decode_query")]
pub enum EncodedQuery<'data> {
    // NonVoid(non_void::NonVoid),
    // ComparisonWithZero(ComparisonWithZero),
    #[cfg(feature = "decode_query_compare_with_value")]
    ComparisonWithValue(EncodedComparisonWithValue<'data>),
    // ComparisonWithOtherFile(ComparisonWithOtherFile),
    #[cfg(feature = "decode_query_compare_with_range")]
    ComparisonWithRange(EncodedComparisonWithRange<'data>),
    // StringTokenSearch(StringTokenSearch),
}

#[cfg(feature = "decode_query")]
impl<'data> EncodedQuery<'data> {
    /// # Safety
    /// You are to warrant, somehow, that the input byte array contains a complete item.
    /// Else this might result in out of bound reads, and absurd results.
    pub unsafe fn size_unchecked(&self) -> usize {
        match self {
            #[cfg(feature = "decode_query_compare_with_value")]
            Self::ComparisonWithValue(d) => d.size_unchecked(),
            #[cfg(feature = "decode_query_compare_with_range")]
            Self::ComparisonWithRange(d) => d.size_unchecked(),
        }
    }
}

#[cfg(feature = "decode_query")]
impl<'data> FailableEncodedData<'data> for EncodedQuery<'data> {
    type Error = UnknownQueryCode<'data>;
    type DecodedData = DecodedQueryRef<'data>;

    unsafe fn new(data: &'data [u8]) -> Result<Self, Self::Error> {
        let code = (*data.get_unchecked(0) >> 5) & 0x07;
        let query_code = match QueryCode::from(code) {
            Ok(code) => code,
            Err(_) => {
                return Err(UnknownQueryCode {
                    code,
                    remaining_data: &data[1..],
                })
            }
        };
        Ok(match query_code {
            #[cfg(feature = "decode_query_compare_with_value")]
            QueryCode::ComparisonWithValue => EncodedQuery::ComparisonWithValue(
                ComparisonWithValueRef::start_decoding_unchecked(data),
            ),
            #[cfg(feature = "decode_query_compare_with_range")]
            QueryCode::ComparisonWithRange => EncodedQuery::ComparisonWithRange(
                ComparisonWithRangeRef::start_decoding_unchecked(data),
            ),
            #[cfg(not(all(
                feature = "decode_query_compare_with_range",
                feature = "decode_query_compare_with_value"
            )))]
            _ => {
                return Err(UnknownQueryCode {
                    code,
                    remaining_data: &data[1..],
                })
            }
        })
    }

    fn size(&self) -> Result<usize, SizeError> {
        match self {
            #[cfg(feature = "decode_query_compare_with_value")]
            Self::ComparisonWithValue(d) => d.size(),
            #[cfg(feature = "decode_query_compare_with_range")]
            Self::ComparisonWithRange(d) => d.size(),
        }
    }

    fn complete_decoding(&self) -> WithByteSize<DecodedQueryRef<'data>> {
        match self {
            #[cfg(feature = "decode_query_compare_with_value")]
            Self::ComparisonWithValue(d) => {
                let WithByteSize {
                    item: op,
                    byte_size: size,
                } = d.complete_decoding();
                WithByteSize {
                    item: DecodedQueryRef::ComparisonWithValue(op),
                    byte_size: size,
                }
            }
            #[cfg(feature = "decode_query_compare_with_range")]
            Self::ComparisonWithRange(d) => {
                let WithByteSize {
                    item: op,
                    byte_size: size,
                } = d.complete_decoding();
                WithByteSize {
                    item: DecodedQueryRef::ComparisonWithRange(op),
                    byte_size: size,
                }
            }
        }
    }
}

#[cfg(feature = "decode_query")]
impl<'data> FailableDecodable<'data> for DecodedQueryRef<'data> {
    type Data = EncodedQuery<'data>;
}

// TODO This alloc condition could be lighter and be required only for query variants that really
// need allocation.
#[cfg(feature = "query")]
#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[cfg(feature = "alloc")]
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Query {
    // NonVoid(non_void::NonVoid),
    // ComparisonWithZero(ComparisonWithZero),
    #[cfg(feature = "query_compare_with_value")]
    ComparisonWithValue(ComparisonWithValue),
    // ComparisonWithOtherFile(ComparisonWithOtherFile),
    #[cfg(feature = "query_compare_with_range")]
    ComparisonWithRange(ComparisonWithRange),
    // StringTokenSearch(StringTokenSearch),
}

#[cfg(feature = "query")]
#[cfg(feature = "alloc")]
impl Query {
    pub fn as_ref(&self) -> QueryRef {
        match self {
            #[cfg(feature = "query_compare_with_value")]
            Self::ComparisonWithValue(query) => QueryRef::ComparisonWithValue(query.as_ref()),
            #[cfg(feature = "query_compare_with_range")]
            Self::ComparisonWithRange(query) => QueryRef::ComparisonWithRange(query.as_ref()),
        }
    }
}

#[cfg(feature = "decode_query")]
#[cfg(test)]
mod test {
    #![allow(clippy::unwrap_in_result, clippy::panic, clippy::expect_used)]
    #[cfg(feature = "decode_query_compare_with_value")]
    use super::define::QueryComparisonType;
    use super::*;
    use crate::define::FileId;
    #[cfg(feature = "decode_query_compare_with_value")]
    use crate::define::{EncodableDataRef, MaskedValueRef};
    use crate::varint::Varint;

    #[test]
    fn known() {
        fn test(op: QueryRef, data: &[u8]) {
            // Test op.encode_in() == data
            let mut encoded = [0_u8; 64];
            let size = op.encode_in(&mut encoded).unwrap();
            assert_eq!(size, data.len());
            assert_eq!(&encoded[..size], data);

            // Test decode(data) == op
            let WithByteSize {
                item: ret,
                byte_size: size,
            } = DecodedQueryRef::decode(data).unwrap();
            assert_eq!(size, data.len());
            assert_eq!(ret.as_encodable(), op);

            // Test partial_decode == op
            let WithByteSize {
                item: decoder,
                byte_size: expected_size,
            } = DecodedQueryRef::start_decoding(data).unwrap();
            assert_eq!(expected_size, size);
            assert_eq!(unsafe { decoder.size_unchecked() }, size);
            assert_eq!(decoder.size().unwrap(), size);
        }
        #[cfg(feature = "decode_query_compare_with_value")]
        test(
            QueryRef::ComparisonWithValue(ComparisonWithValueRef {
                signed_data: false,
                comparison_type: QueryComparisonType::GreaterThan,
                compare_value: MaskedValueRef::new(
                    EncodableDataRef::new(&[0x0A, 0x0B, 0x0C, 0x0D]).unwrap(),
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
        #[cfg(feature = "decode_query_compare_with_range")]
        test(
            QueryRef::ComparisonWithRange(ComparisonWithRangeRef {
                signed_data: false,
                comparison_type: define::QueryRangeComparisonType::NotInRange,
                range: crate::define::MaskedRangeRef::new(50, 66, Some(&[0x33, 0x22])).unwrap(),
                file_id: FileId::new(0x88),
                offset: Varint::new(0xFF).unwrap(),
            }),
            &[0x80 | 0x10, 0x01, 50, 66, 0x33, 0x22, 0x88, 0x40, 0xFF],
        );
    }

    #[cfg(feature = "decode_query_compare_with_value")]
    #[test]
    fn consistence() {
        const TOT_SIZE: usize = 1 + 1 + 3 + 3 + 1 + 3;
        let op = QueryRef::ComparisonWithValue(ComparisonWithValueRef {
            signed_data: true,
            comparison_type: QueryComparisonType::GreaterThanOrEqual,
            compare_value: MaskedValueRef::new(
                EncodableDataRef::new(&[0x00, 0x43, 0x02]).unwrap(),
                Some(&[0x44, 0x88, 0x11]),
            )
            .unwrap(),
            file_id: FileId::new(0xFF),
            offset: Varint::new(0x3F_FF_00).unwrap(),
        });

        // Test decode(op.encode_in()) == op
        let mut encoded = [0_u8; TOT_SIZE];
        let size_encoded = op.encode_in(&mut encoded).unwrap();
        let WithByteSize {
            item: ret,
            byte_size: size_decoded,
        } = DecodedQueryRef::decode(&encoded).unwrap();
        assert_eq!(size_encoded, size_decoded);
        assert_eq!(ret.as_encodable(), op);

        // Test decode(data).encode_in() == data
        let mut encoded2 = [0_u8; TOT_SIZE];
        let size_encoded2 = op.encode_in(&mut encoded2).unwrap();
        assert_eq!(size_encoded, size_encoded2);
        assert_eq!(encoded2[..size_encoded2], encoded[..size_encoded]);
    }
}

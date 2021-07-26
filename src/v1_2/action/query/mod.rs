pub mod action_query;
pub mod break_query;
pub mod comparison_with_range;
pub mod comparison_with_value;
pub mod define;

#[cfg(feature = "query_compare_with_range")]
use comparison_with_range::ComparisonWithRangeRef;
#[cfg(feature = "decode_query_compare_with_range")]
use comparison_with_range::{EncodedComparisonWithRange, EncodedComparisonWithRangeMut};
#[cfg(feature = "query_compare_with_value")]
use comparison_with_value::ComparisonWithValueRef;
#[cfg(feature = "decode_query_compare_with_value")]
use comparison_with_value::{EncodedComparisonWithValue, EncodedComparisonWithValueMut};

#[cfg(feature = "query_compare_with_range")]
#[cfg(feature = "alloc")]
use comparison_with_range::ComparisonWithRange;
#[cfg(feature = "query_compare_with_value")]
#[cfg(feature = "alloc")]
use comparison_with_value::ComparisonWithValue;

#[cfg(feature = "query")]
use define::code::QueryCode;

#[cfg(any(feature = "decode_query_compare_with_value"))]
use crate::decodable::{Decodable, EncodedData};

#[cfg(feature = "decode_query")]
use crate::decodable::{FailableDecodable, FailableEncodedData, WithByteSize};
#[cfg(feature = "query")]
use crate::encodable::Encodable;
#[cfg(feature = "decode_query")]
use crate::v1_2::error::action::query::{QueryError, QuerySizeError, UnsupportedQueryCode};

#[cfg(feature = "query")]
#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum QueryRef<'data> {
    // NonVoid(non_void::NonVoid),
    // ComparisonWithZero(ComparisonWithZero),
    #[cfg(feature = "query_compare_with_value")]
    ComparisonWithValue(ComparisonWithValueRef<'data>),
    // ComparisonWithOtherFile(ComparisonWithOtherFile),
    #[cfg(feature = "query_compare_with_range")]
    ComparisonWithRange(ComparisonWithRangeRef<'data>),
    // StringTokenSearch(StringTokenSearch),
}

#[cfg(feature = "query")]
impl<'data> Encodable for QueryRef<'data> {
    unsafe fn encode_in_ptr(&self, out: *mut u8) -> usize {
        match self {
            #[cfg(feature = "query_compare_with_value")]
            Self::ComparisonWithValue(query) => query.encode_in_ptr(out),
            #[cfg(feature = "query_compare_with_range")]
            Self::ComparisonWithRange(query) => query.encode_in_ptr(out),
        }
    }

    fn encoded_size(&self) -> usize {
        match self {
            #[cfg(feature = "query_compare_with_value")]
            Self::ComparisonWithValue(query) => query.encoded_size(),
            #[cfg(feature = "query_compare_with_range")]
            Self::ComparisonWithRange(query) => query.encoded_size(),
        }
    }
}

#[cfg(feature = "query")]
impl<'data> QueryRef<'data> {
    pub fn query_code(&self) -> QueryCode {
        match self {
            #[cfg(feature = "query_compare_with_value")]
            Self::ComparisonWithValue(_) => QueryCode::ComparisonWithValue,
            #[cfg(feature = "query_compare_with_range")]
            Self::ComparisonWithRange(_) => QueryCode::ComparisonWithRange,
        }
    }

    // TODO Move inside when comparison without alloc exists
    #[cfg(feature = "alloc")]
    #[allow(clippy::wrong_self_convention)]
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
pub enum DecodedQueryRef<'data> {
    // NonVoid(non_void::NonVoid),
    // ComparisonWithZero(ComparisonWithZero),
    #[cfg(feature = "decode_query_compare_with_value")]
    ComparisonWithValue(ComparisonWithValueRef<'data>),
    // ComparisonWithOtherFile(ComparisonWithOtherFile),
    #[cfg(feature = "decode_query_compare_with_range")]
    ComparisonWithRange(ComparisonWithRangeRef<'data>),
    // StringTokenSearch(StringTokenSearch),
}

#[cfg(feature = "decode_query")]
impl<'data> DecodedQueryRef<'data> {
    pub fn query_code(&self) -> QueryCode {
        match self {
            #[cfg(feature = "decode_query_compare_with_value")]
            Self::ComparisonWithValue(_) => QueryCode::ComparisonWithValue,
            #[cfg(feature = "decode_query_compare_with_range")]
            Self::ComparisonWithRange(_) => QueryCode::ComparisonWithRange,
        }
    }

    pub fn as_encodable(self) -> QueryRef<'data> {
        self.into()
    }
}

#[cfg(feature = "decode_query")]
impl<'data> From<DecodedQueryRef<'data>> for QueryRef<'data> {
    fn from(decoded: DecodedQueryRef<'data>) -> Self {
        match decoded {
            #[cfg(feature = "decode_query_compare_with_value")]
            DecodedQueryRef::ComparisonWithValue(query) => QueryRef::ComparisonWithValue(query),
            #[cfg(feature = "decode_query_compare_with_range")]
            DecodedQueryRef::ComparisonWithRange(query) => QueryRef::ComparisonWithRange(query),
        }
    }
}

#[cfg(feature = "decode_query")]
pub enum ValidEncodedQuery<'data> {
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
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct EncodedQuery<'data> {
    data: &'data [u8],
}

#[cfg(feature = "decode_query")]
impl<'data> EncodedQuery<'data> {
    /// # Errors
    /// Fails if the query code is unsupported.
    pub fn query_code(&self) -> Result<QueryCode, UnsupportedQueryCode<'data>> {
        unsafe {
            let code = (*self.data.get_unchecked(0) >> 5) & 0x07;
            QueryCode::from(code).map_err(|_| UnsupportedQueryCode {
                code,
                remaining_data: self.data.get_unchecked(1..),
            })
        }
    }

    /// # Errors
    /// Fails if the query code is unsupported.
    pub fn operand(&self) -> Result<ValidEncodedQuery<'data>, UnsupportedQueryCode<'data>> {
        unsafe {
            Ok(match self.query_code()? {
                #[cfg(feature = "decode_query_compare_with_value")]
                QueryCode::ComparisonWithValue => ValidEncodedQuery::ComparisonWithValue(
                    ComparisonWithValueRef::start_decoding_unchecked(self.data),
                ),
                #[cfg(feature = "decode_query_compare_with_range")]
                QueryCode::ComparisonWithRange => ValidEncodedQuery::ComparisonWithRange(
                    ComparisonWithRangeRef::start_decoding_unchecked(self.data),
                ),
            })
        }
    }
}

#[cfg(feature = "decode_query")]
impl<'data> FailableEncodedData<'data> for EncodedQuery<'data> {
    type SourceData = &'data [u8];
    type SizeError = QuerySizeError<'data>;
    type DecodeError = QueryError<'data>;
    type DecodedData = DecodedQueryRef<'data>;

    unsafe fn new(data: Self::SourceData) -> Self {
        Self { data }
    }

    fn encoded_size(&self) -> Result<usize, Self::SizeError> {
        Ok(
            match self
                .operand()
                .map_err(QuerySizeError::UnsupportedQueryCode)?
            {
                #[cfg(feature = "decode_query_compare_with_value")]
                ValidEncodedQuery::ComparisonWithValue(d) => d.encoded_size()?,
                #[cfg(feature = "decode_query_compare_with_range")]
                ValidEncodedQuery::ComparisonWithRange(d) => d.encoded_size()?,
            },
        )
    }

    fn complete_decoding(&self) -> Result<WithByteSize<Self::DecodedData>, Self::DecodeError> {
        Ok(match self.operand()? {
            #[cfg(feature = "decode_query_compare_with_value")]
            ValidEncodedQuery::ComparisonWithValue(d) => {
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
            ValidEncodedQuery::ComparisonWithRange(d) => {
                let WithByteSize {
                    item: op,
                    byte_size: size,
                } = d.complete_decoding()?;
                WithByteSize {
                    item: DecodedQueryRef::ComparisonWithRange(op),
                    byte_size: size,
                }
            }
        })
    }
}

#[cfg(feature = "decode_query")]
pub enum ValidEncodedQueryMut<'data> {
    // NonVoid(non_void::NonVoid),
    // ComparisonWithZero(ComparisonWithZero),
    #[cfg(feature = "decode_query_compare_with_value")]
    ComparisonWithValue(EncodedComparisonWithValueMut<'data>),
    // ComparisonWithOtherFile(ComparisonWithOtherFile),
    #[cfg(feature = "decode_query_compare_with_range")]
    ComparisonWithRange(EncodedComparisonWithRangeMut<'data>),
    // StringTokenSearch(StringTokenSearch),
}

#[cfg(feature = "decode_query")]
#[derive(Eq, PartialEq, Debug)]
pub struct EncodedQueryMut<'data> {
    data: &'data mut [u8],
}

#[cfg(feature = "decode_query")]
crate::make_downcastable!(EncodedQueryMut, EncodedQuery);

#[cfg(feature = "decode_query")]
impl<'data> EncodedQueryMut<'data> {
    /// # Errors
    /// Fails if the query code is unsupported.
    pub fn query_code(&self) -> Result<QueryCode, UnsupportedQueryCode<'data>> {
        self.borrow().query_code()
    }

    /// # Errors
    /// Fails if the query code is unsupported.
    pub fn operand(&self) -> Result<ValidEncodedQuery<'data>, UnsupportedQueryCode<'data>> {
        self.borrow().operand()
    }

    /// Changes the query code, and thus the query type of this query.
    ///
    /// # Safety
    /// This will break:
    /// - the whole query structure except maybe for the "length" parameter.
    ///
    /// It also breaks the payload after this action.
    ///
    /// Only use it if you are sure about what you are doing.
    pub unsafe fn set_query_code(&mut self, code: QueryCode) {
        *self.data.get_unchecked_mut(0) =
            (*self.data.get_unchecked(0) & 0x1F) | ((code as u8) << 5);
    }

    /// # Errors
    /// Fails if the query code is unsupported.
    pub fn operand_mut(&mut self) -> Result<ValidEncodedQueryMut, UnsupportedQueryCode<'data>> {
        unsafe {
            Ok(match self.query_code()? {
                #[cfg(feature = "decode_query_compare_with_value")]
                QueryCode::ComparisonWithValue => ValidEncodedQueryMut::ComparisonWithValue(
                    ComparisonWithValueRef::start_decoding_unchecked_mut(self.data),
                ),
                #[cfg(feature = "decode_query_compare_with_range")]
                QueryCode::ComparisonWithRange => ValidEncodedQueryMut::ComparisonWithRange(
                    ComparisonWithRangeRef::start_decoding_unchecked_mut(self.data),
                ),
            })
        }
    }
}

#[cfg(feature = "decode_query")]
impl<'data> FailableEncodedData<'data> for EncodedQueryMut<'data> {
    type SourceData = &'data mut [u8];
    type SizeError = QuerySizeError<'data>;
    type DecodeError = QueryError<'data>;
    type DecodedData = DecodedQueryRef<'data>;

    unsafe fn new(data: Self::SourceData) -> Self {
        Self { data }
    }

    fn encoded_size(&self) -> Result<usize, Self::SizeError> {
        self.borrow().encoded_size()
    }

    fn complete_decoding(&self) -> Result<WithByteSize<Self::DecodedData>, Self::DecodeError> {
        self.borrow().complete_decoding()
    }
}

#[cfg(feature = "decode_query")]
impl<'data> FailableDecodable<'data> for DecodedQueryRef<'data> {
    type Data = EncodedQuery<'data>;
    type DataMut = EncodedQueryMut<'data>;
    type FullDecodeError = QuerySizeError<'data>;
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
    pub fn borrow(&self) -> QueryRef {
        match self {
            #[cfg(feature = "query_compare_with_value")]
            Self::ComparisonWithValue(query) => QueryRef::ComparisonWithValue(query.borrow()),
            #[cfg(feature = "query_compare_with_range")]
            Self::ComparisonWithRange(query) => QueryRef::ComparisonWithRange(query.borrow()),
        }
    }
}

#[cfg(feature = "decode_query")]
#[cfg(test)]
mod test {
    #![allow(clippy::unwrap_in_result, clippy::panic, clippy::expect_used)]
    #[cfg(feature = "decode_query_compare_with_value")]
    use super::define::comparison_type::QueryComparisonType;
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
            assert_eq!(decoder.encoded_size().unwrap(), size);
            assert_eq!(decoder.query_code().unwrap(), op.query_code());

            // Test partial mutability
            let WithByteSize {
                item: mut decoder_mut,
                byte_size: expected_size,
            } = DecodedQueryRef::start_decoding_mut(&mut encoded).unwrap();
            assert_eq!(expected_size, size);

            match decoder_mut.operand_mut().unwrap() {
                #[cfg(feature = "decode_query_compare_with_value")]
                ValidEncodedQueryMut::ComparisonWithValue(mut decoder_mut) => {
                    let original = decoder_mut.signed_data();
                    let new_signed_data = !original;
                    assert!(new_signed_data != original);
                    decoder_mut.set_signed_data(new_signed_data);
                    assert_eq!(decoder_mut.signed_data(), new_signed_data);
                }
                #[cfg(feature = "decode_query_compare_with_range")]
                ValidEncodedQueryMut::ComparisonWithRange(mut decoder_mut) => {
                    let original = decoder_mut.signed_data();
                    let new_signed_data = !original;
                    assert!(new_signed_data != original);
                    decoder_mut.set_signed_data(new_signed_data);
                    assert_eq!(decoder_mut.signed_data(), new_signed_data);
                }
            }

            // Unsafe mutations
            #[cfg(all(
                feature = "decode_query_compare_with_value",
                feature = "decode_query_compare_with_range"
            ))]
            {
                let original = decoder_mut.query_code().unwrap();
                let target = if let QueryCode::ComparisonWithValue = original {
                    QueryCode::ComparisonWithRange
                } else {
                    QueryCode::ComparisonWithValue
                };
                assert!(target != original);
                unsafe { decoder_mut.set_query_code(target) };
                assert_eq!(decoder_mut.query_code().unwrap(), target);
            }

            // Check undecodability of shorter payload
            for i in 1..data.len() {
                assert_eq!(
                    DecodedQueryRef::start_decoding(&data[..i]),
                    Err(QuerySizeError::MissingBytes)
                );
            }

            // Check unencodability in shorter arrays
            for i in 0..data.len() {
                let mut array = vec![0; i];
                let ret = op.encode_in(&mut array);
                let missing = ret.unwrap_err();
                assert_eq!(missing, data.len());
            }
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
                comparison_type:
                    define::range_comparison_type::QueryRangeComparisonType::NotInRange,
                range: crate::define::MaskedRangeRef::new(
                    Varint::new(1).unwrap(),
                    50,
                    66,
                    Some(&[0x33, 0x22]),
                )
                .unwrap(),
                file_id: FileId::new(0x88),
                offset: Varint::new(0xFF).unwrap(),
            }),
            &[0x80 | 0x10, 0x01, 50, 66, 0x33, 0x22, 0x88, 0x40, 0xFF],
        );
    }

    #[test]
    fn errors() {
        let data = [0xC0, 0x11];
        assert_eq!(
            DecodedQueryRef::start_decoding(&data),
            Err(QuerySizeError::UnsupportedQueryCode(UnsupportedQueryCode {
                // TODO This might be a supported query code!
                code: 6,
                remaining_data: &[0x11],
            }))
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

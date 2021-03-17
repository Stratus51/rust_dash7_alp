#[cfg(feature = "decode_query")]
use crate::decodable::{FailableEncodedData, WithByteSize};
#[cfg(feature = "query")]
use crate::encodable::Encodable;
#[cfg(feature = "query")]
use crate::v1_2::define::flag;
#[cfg(feature = "query")]
use crate::v1_2::define::op_code;
#[cfg(feature = "decode_query")]
use crate::v1_2::error::{QuerySizeError, UnsupportedQueryCode};

#[cfg(feature = "query")]
use super::QueryRef;
#[cfg(feature = "decode_query")]
use super::{DecodedQueryRef, EncodedQuery, EncodedQueryMut};

#[cfg(feature = "query")]
#[cfg(feature = "alloc")]
use super::Query;

// TODO Rename to be more generic, and be reused in break_query
/// Executes next action group depending on a condition
#[cfg(feature = "query")]
#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct ActionQueryRef<'item, 'data> {
    /// Group with next action
    pub group: bool,
    /// Ask for a response (a status)
    pub response: bool,
    /// Action condition
    pub query: QueryRef<'item, 'data>,
}

#[cfg(feature = "query")]
impl<'item, 'data> Encodable for ActionQueryRef<'item, 'data> {
    unsafe fn encode_in_ptr(&self, out: *mut u8) -> usize {
        let mut size = 0;
        *out.add(0) = op_code::ACTION_QUERY
            | if self.group { flag::GROUP } else { 0 }
            | if self.response { flag::RESPONSE } else { 0 };
        size += 1;
        size += self.query.encode_in_ptr(out.add(size));
        size
    }

    fn encoded_size(&self) -> usize {
        1 + self.query.encoded_size()
    }
}

#[cfg(feature = "query")]
impl<'item, 'data> ActionQueryRef<'item, 'data> {
    // TODO This is not always required once non alloc query are implemented
    #[cfg(feature = "alloc")]
    pub fn to_owned(&self) -> ActionQuery {
        ActionQuery {
            group: self.group,
            response: self.response,
            query: self.query.to_owned(),
        }
    }
}

#[cfg(feature = "decode_query")]
#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct DecodedActionQueryRef<'item, 'data> {
    /// Group with next action
    pub group: bool,
    /// Ask for a response (a status)
    pub response: bool,
    /// Action condition
    pub query: DecodedQueryRef<'item, 'data>,
}

#[cfg(feature = "decode_query")]
impl<'item, 'data> DecodedActionQueryRef<'item, 'data> {
    pub fn as_encodable<'result>(self) -> ActionQueryRef<'result, 'data> {
        self.into()
    }
}

#[cfg(feature = "decode_query")]
impl<'item, 'data> From<DecodedActionQueryRef<'item, 'data>> for ActionQueryRef<'item, 'data> {
    fn from(decoded: DecodedActionQueryRef<'item, 'data>) -> Self {
        Self {
            group: decoded.group,
            response: decoded.response,
            query: decoded.query.into(),
        }
    }
}

#[cfg(feature = "decode_query")]
pub struct EncodedActionQuery<'item, 'data> {
    data: &'item &'data [u8],
}

#[cfg(feature = "decode_query")]
impl<'item, 'data> EncodedActionQuery<'item, 'data> {
    pub fn group(&self) -> bool {
        unsafe { *self.data.get_unchecked(0) & flag::GROUP != 0 }
    }

    pub fn response(&self) -> bool {
        unsafe { *self.data.get_unchecked(0) & flag::RESPONSE != 0 }
    }

    pub fn query<'result>(&self) -> EncodedQuery<'result, 'data> {
        unsafe { DecodedQueryRef::start_decoding_unchecked(&self.data.get_unchecked(1..)) }
    }
}

#[cfg(feature = "decode_query")]
impl<'item, 'data> EncodedActionQuery<'item, 'data> {
    pub(crate) unsafe fn new(data: &'data [u8]) -> Self {
        Self { data }
    }

    pub fn encoded_size<'result>(&self) -> Result<usize, QuerySizeError<'result, 'data>> {
        self.query().encoded_size().map(|size| 1 + size)
    }

    pub fn complete_decoding<'result>(
        &self,
    ) -> Result<
        WithByteSize<DecodedActionQueryRef<'result, 'data>>,
        UnsupportedQueryCode<'result, 'data>,
    > {
        let WithByteSize {
            item: query,
            byte_size: query_size,
        } = self.query().complete_decoding()?;
        Ok(WithByteSize {
            item: DecodedActionQueryRef {
                group: self.group(),
                response: self.response(),
                query,
            },
            byte_size: 1 + query_size,
        })
    }
}

#[cfg(feature = "decode_query")]
pub struct EncodedActionQueryMut<'item, 'data> {
    data: &'item mut &'data mut [u8],
}

#[cfg(feature = "decode_query")]
impl<'item, 'data> EncodedActionQueryMut<'item, 'data> {
    pub fn as_ref<'result>(&self) -> EncodedActionQuery<'result, 'data> {
        unsafe { EncodedActionQuery::new(self.data) }
    }

    pub fn group(&self) -> bool {
        self.as_ref().group()
    }

    pub fn response(&self) -> bool {
        self.as_ref().response()
    }

    pub fn query<'result>(&self) -> EncodedQuery<'result, 'data> {
        self.as_ref().query()
    }

    pub fn set_group(&mut self, group: bool) {
        unsafe { *self.data.get_unchecked_mut(0) |= flag::GROUP }
    }

    pub fn set_response(&mut self, group: bool) {
        unsafe { *self.data.get_unchecked_mut(0) |= flag::RESPONSE }
    }

    pub fn query_mut<'result>(&'data mut self) -> EncodedQueryMut<'result, 'data> {
        unsafe { DecodedQueryRef::start_decoding_unchecked_mut(self.data.get_unchecked_mut(1..)) }
    }
}

#[cfg(feature = "decode_query")]
impl<'item, 'data> EncodedActionQueryMut<'item, 'data> {
    pub(crate) unsafe fn new(data: &'data mut [u8]) -> Self {
        Self { data }
    }

    pub fn encoded_size<'result>(&self) -> Result<usize, QuerySizeError<'result, 'data>> {
        self.as_ref().encoded_size()
    }

    pub fn complete_decoding<'result>(
        &self,
    ) -> Result<
        WithByteSize<DecodedActionQueryRef<'result, 'data>>,
        UnsupportedQueryCode<'result, 'data>,
    > {
        self.as_ref().complete_decoding()
    }
}

#[cfg(feature = "decode_query")]
crate::make_failable_decodable!(
    DecodedActionQueryRef,
    EncodedActionQuery,
    EncodedActionQueryMut,
    QuerySizeError,
    UnsupportedQueryCode,
    QuerySizeError
);

/// Executes next action group depending on a condition
#[cfg(feature = "query")]
#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[cfg(feature = "alloc")]
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct ActionQuery {
    /// Group with next action
    pub group: bool,
    /// Ask for a response (a status)
    pub response: bool,
    /// Action condition
    pub query: Query,
}

#[cfg(feature = "query")]
#[cfg(feature = "alloc")]
impl ActionQuery {
    pub fn as_ref(&self) -> ActionQueryRef {
        ActionQueryRef {
            group: self.group,
            response: self.response,
            query: self.query.as_ref(),
        }
    }
}

#[cfg(feature = "decode_action_query")]
#[cfg(feature = "decode_query_compare_with_value")]
#[cfg(test)]
mod test {
    #![allow(clippy::unwrap_in_result, clippy::panic, clippy::expect_used)]
    use super::super::{
        comparison_with_value::ComparisonWithValueRef, define::QueryComparisonType,
    };
    use super::*;
    use crate::{
        define::{EncodableDataRef, FileId, MaskedValueRef},
        varint::Varint,
    };

    #[test]
    fn known() {
        fn test(op: ActionQueryRef, data: &[u8]) {
            // Test op.encode_in() == data
            let mut encoded = [0_u8; 2 + 8];
            let size = op.encode_in(&mut encoded).unwrap();
            assert_eq!(size, data.len());
            assert_eq!(&encoded[..size], data);

            // Test decode(data) == op
            let WithByteSize {
                item: ret,
                byte_size: size,
            } = DecodedActionQueryRef::decode(data).unwrap();
            assert_eq!(size, data.len());
            assert_eq!(ret.as_encodable(), op);

            // Test partial_decode == op
            let WithByteSize {
                item: decoder,
                byte_size: expected_size,
            } = DecodedActionQueryRef::start_decoding(data).unwrap();
            assert_eq!(expected_size, size);
            assert_eq!(
                op,
                ActionQueryRef {
                    group: decoder.group(),
                    response: decoder.response(),
                    query: decoder
                        .query()
                        .complete_decoding()
                        .unwrap()
                        .item
                        .as_encodable(),
                }
            );
        }
        test(
            ActionQueryRef {
                group: false,
                response: true,
                query: QueryRef::ComparisonWithValue(ComparisonWithValueRef {
                    signed_data: true,
                    comparison_type: QueryComparisonType::Equal,
                    compare_value: MaskedValueRef::new(
                        EncodableDataRef::new(&[0x00, 0x01, 0x02]).unwrap(),
                        None,
                    )
                    .unwrap(),
                    file_id: FileId::new(0x42),
                    offset: Varint::new(0x40_00).unwrap(),
                }),
            },
            &[
                0x40 | 0x08,
                0x40 | 0x08 | 0x01,
                0x03,
                0x00,
                0x01,
                0x02,
                0x42,
                0x80,
                0x40,
                0x00,
            ],
        );
    }

    #[test]
    fn consistence() {
        const TOT_SIZE: usize = 10;
        let op = ActionQueryRef {
            group: true,
            response: false,
            query: QueryRef::ComparisonWithValue(ComparisonWithValueRef {
                signed_data: true,
                comparison_type: QueryComparisonType::Equal,
                compare_value: MaskedValueRef::new(
                    EncodableDataRef::new(&[0x00, 0x01, 0x02]).unwrap(),
                    None,
                )
                .unwrap(),
                file_id: FileId::new(0x42),
                offset: Varint::new(0x40_00).unwrap(),
            }),
        };

        // Test decode(op.encode_in()) == op
        let mut encoded = [0_u8; TOT_SIZE];
        let size_encoded = op.encode_in(&mut encoded).unwrap();
        let WithByteSize {
            item: ret,
            byte_size: size_decoded,
        } = DecodedActionQueryRef::decode(&encoded).unwrap();
        assert_eq!(size_encoded, size_decoded);
        assert_eq!(ret.as_encodable(), op);

        // Test decode(data).encode_in() == data
        let mut encoded2 = [0_u8; TOT_SIZE];
        let size_encoded2 = op.encode_in(&mut encoded2).unwrap();
        assert_eq!(size_encoded, size_encoded2);
        assert_eq!(encoded2[..size_encoded2], encoded[..size_encoded]);
    }
}

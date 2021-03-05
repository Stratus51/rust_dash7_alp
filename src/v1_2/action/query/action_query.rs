#[cfg(feature = "query")]
use super::super::super::define::flag;
#[cfg(feature = "query")]
use super::super::super::define::op_code::OpCode;
#[cfg(feature = "decode_query")]
use crate::decodable::{FailableDecodable, FailableEncodedData, WithByteSize};
#[cfg(feature = "decode_query")]
use crate::v1_2::error::{PtrUnknownQueryCode, UnknownQueryCode};

#[cfg(feature = "query")]
use super::QueryRef;
#[cfg(feature = "decode_query")]
use super::{DecodedQueryRef, EncodedQuery};

#[cfg(feature = "query")]
#[cfg(feature = "alloc")]
use super::Query;

/// Executes next action group depending on a condition
#[cfg(feature = "query")]
#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct ActionQueryRef<'item> {
    /// Group with next action
    pub group: bool,
    /// Ask for a response (a status)
    pub response: bool,
    /// Action condition
    pub query: QueryRef<'item>,
}

#[cfg(feature = "query")]
impl<'item> ActionQueryRef<'item> {
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
        let mut size = 0;
        *out.add(0) = OpCode::ActionQuery as u8
            | if self.group { flag::GROUP } else { 0 }
            | if self.response { flag::RESPONSE } else { 0 };
        size += 1;
        size += self.query.encode_in_ptr(out.add(size));
        size
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
        1 + self.query.size()
    }

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
pub struct DecodedActionQueryRef<'item> {
    /// Group with next action
    pub group: bool,
    /// Ask for a response (a status)
    pub response: bool,
    /// Action condition
    pub query: DecodedQueryRef<'item>,
}

#[cfg(feature = "decode_query")]
impl<'item> DecodedActionQueryRef<'item> {
    pub fn as_encodable(self) -> ActionQueryRef<'item> {
        self.into()
    }
}

#[cfg(feature = "decode_query")]
impl<'item> From<DecodedActionQueryRef<'item>> for ActionQueryRef<'item> {
    fn from(decoded: DecodedActionQueryRef<'item>) -> Self {
        Self {
            group: decoded.group,
            response: decoded.response,
            query: decoded.query.into(),
        }
    }
}

#[cfg(feature = "decode_query")]
pub struct EncodedActionQuery<'data> {
    data: *const u8,
    data_life: core::marker::PhantomData<&'data ()>,
    query: EncodedQuery<'data>,
}

#[cfg(feature = "decode_query")]
impl<'data> EncodedActionQuery<'data> {
    pub fn group(&self) -> bool {
        unsafe { *self.data.add(0) & flag::GROUP != 0 }
    }

    pub fn response(&self) -> bool {
        unsafe { *self.data.add(0) & flag::RESPONSE != 0 }
    }

    pub fn query(&self) -> &EncodedQuery<'data> {
        &self.query
    }
}

#[cfg(feature = "decode_query")]
impl<'data> FailableEncodedData<'data> for EncodedActionQuery<'data> {
    type RefError = UnknownQueryCode<'data>;
    type PtrError = PtrUnknownQueryCode<'data>;
    type DecodedData = DecodedActionQueryRef<'data>;

    unsafe fn from_data_ref(data: &'data [u8]) -> Result<Self, Self::RefError> {
        let query = DecodedQueryRef::start_decoding_unchecked(&data[1..])?;
        Ok(Self {
            data: data.as_ptr(),
            data_life: core::marker::PhantomData,
            query,
        })
    }

    unsafe fn from_data_ptr(data: *const u8) -> Result<Self, Self::PtrError> {
        let query = DecodedQueryRef::start_decoding_ptr(data.add(1))?;
        Ok(Self {
            data,
            data_life: core::marker::PhantomData,
            query,
        })
    }

    unsafe fn expected_size(&self) -> usize {
        1 + self.query.expected_size()
    }

    fn smaller_than(&self, data_size: usize) -> Result<usize, usize> {
        self.query
            .smaller_than(data_size - 1)
            .map(|size| 1 + size)
            .map_err(|size| 1 + size)
    }

    fn complete_decoding(&self) -> WithByteSize<DecodedActionQueryRef<'data>> {
        let WithByteSize {
            item: query,
            byte_size: query_size,
        } = self.query.complete_decoding();
        WithByteSize {
            item: DecodedActionQueryRef {
                group: self.group(),
                response: self.response(),
                query,
            },
            byte_size: 1 + query_size,
        }
    }
}

#[cfg(feature = "decode_query")]
impl<'data> FailableDecodable<'data> for DecodedActionQueryRef<'data> {
    type Data = EncodedActionQuery<'data>;
}

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
    use super::super::define::QueryComparisonType;
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
                    query: decoder.query().complete_decoding().item.as_encodable(),
                }
            );
        }
        test(
            ActionQueryRef {
                group: false,
                response: true,
                query: QueryRef::ComparisonWithValue(
                    super::super::comparison_with_value::ComparisonWithValueRef {
                        signed_data: true,
                        comparison_type: QueryComparisonType::Equal,
                        compare_value: MaskedValueRef::new(
                            EncodableDataRef::new(&[0x00, 0x01, 0x02]).unwrap(),
                            None,
                        )
                        .unwrap(),
                        file_id: FileId::new(0x42),
                        offset: Varint::new(0x40_00).unwrap(),
                    },
                ),
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
            query: QueryRef::ComparisonWithValue(
                super::super::comparison_with_value::ComparisonWithValueRef {
                    signed_data: true,
                    comparison_type: QueryComparisonType::Equal,
                    compare_value: MaskedValueRef::new(
                        EncodableDataRef::new(&[0x00, 0x01, 0x02]).unwrap(),
                        None,
                    )
                    .unwrap(),
                    file_id: FileId::new(0x42),
                    offset: Varint::new(0x40_00).unwrap(),
                },
            ),
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

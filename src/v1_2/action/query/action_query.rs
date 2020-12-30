use super::super::super::define::flag;
use super::super::super::define::op_code::OpCode;
use super::super::super::error::{
    PtrUnknownQueryCode, QueryActionDecodeError, QueryDecodeError, UnknownQueryCode,
};
use super::{DecodableQuery, Query};

/// Writes data to a file.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct ActionQuery<'item> {
    /// Group with next action
    pub group: bool,
    /// Ask for a response (a status)
    pub response: bool,
    /// Action condition
    pub query: Query<'item>,
}

impl<'item> ActionQuery<'item> {
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

    /// Creates a decodable item from a data pointer without checking the data size.
    ///
    /// This method is meant to allow unchecked cross language wrapper libraries
    /// to implement an unchecked call without having to build a fake slice with
    /// a fake size.
    ///
    /// It is not meant to be used inside a Rust library/binary.
    ///
    /// # Safety
    /// You are to check that:
    /// - The first byte contains this action's opcode.
    /// - The decodable object fits in the given data:
    /// [`decodable.smaller_than(data.len())`](struct.DecodableActionQuery.html#method.smaller_than)
    ///
    /// Failing that might result in reading and interpreting data outside the given
    /// array (depending on what is done with the resulting object).
    pub const unsafe fn start_decoding_ptr<'data>(data: *const u8) -> DecodableActionQuery<'data> {
        DecodableActionQuery::from_ptr(data)
    }

    /// Creates a decodable item without checking the data size.
    ///
    /// # Safety
    /// You are to check that:
    /// - The first byte contains this action's opcode.
    /// - The decodable object fits in the given data:
    /// [`decodable.smaller_than(data.len())`](struct.DecodableActionQuery.html#method.smaller_than)
    ///
    /// Failing that might result in reading and interpreting data outside the given
    /// array (depending on what is done with the resulting object).
    pub const unsafe fn start_decoding_unchecked(data: &[u8]) -> DecodableActionQuery {
        DecodableActionQuery::new(data)
    }

    /// Returns a Decodable object and its expected byte size.
    ///
    /// This decodable item allows each parts of the item to be decoded independently.
    ///
    /// # Errors
    /// - Fails if first byte of the data contains the wrong opcode.
    /// - Fails if data is smaller then the decoded expected size.
    pub fn start_decoding(
        data: &[u8],
    ) -> Result<(DecodableActionQuery, usize), QueryActionDecodeError> {
        if data.is_empty() {
            return Err(QueryActionDecodeError::MissingBytes(1));
        }
        if data[0] & 0x3F != OpCode::ActionQuery as u8 {
            return Err(QueryActionDecodeError::BadOpCode);
        }
        let ret = unsafe { Self::start_decoding_unchecked(data) };
        let size = ret.smaller_than(data.len()).map_err(|e| match e {
            QueryDecodeError::UnknownQueryCode(code) => {
                QueryActionDecodeError::UnknownQueryCode(UnknownQueryCode {
                    code,
                    remaining_data: unsafe { data.get_unchecked(1..) },
                })
            }
            QueryDecodeError::MissingBytes(n) => QueryActionDecodeError::MissingBytes(n),
        })?;
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
    /// Fails if the parsed data corresponds to an invalid querycode.
    /// Returns the invalid querycode.
    ///
    /// # Safety
    /// May attempt to read bytes after the end of the array.
    ///
    /// You are to check that:
    /// - The first byte contains this action's opcode.
    /// - The resulting size of the data consumed is smaller than the size of the
    /// decoded data.
    ///
    /// Failing that will result in reading and interpreting data outside the given
    /// array.
    pub unsafe fn decode_ptr(data: *const u8) -> Result<(Self, usize), PtrUnknownQueryCode<'item>> {
        Self::start_decoding_ptr(data)
            .complete_decoding()
            .map_err(|code| PtrUnknownQueryCode {
                code,
                remaining_data: data.add(1),
                phantom: core::marker::PhantomData,
            })
    }

    /// Decodes the Item from bytes.
    ///
    /// Returns the decoded data and the number of bytes consumed to produce it.
    ///
    /// # Errors
    /// Fails if the parsed data corresponds to an invalid querycode.
    /// Returns the invalid querycode.
    ///
    /// # Safety
    /// May attempt to read bytes after the end of the array.
    ///
    /// You are to check that:
    /// - The first byte contains this action's opcode.
    /// - The resulting size of the data consumed is smaller than the size of the
    /// decoded data.
    ///
    /// Failing that will result in reading and interpreting data outside the given
    /// array.
    pub unsafe fn decode_unchecked(data: &'item [u8]) -> Result<(Self, usize), UnknownQueryCode> {
        Self::start_decoding_unchecked(data)
            .complete_decoding()
            .map_err(|code| UnknownQueryCode {
                code,
                remaining_data: data.get_unchecked(1..),
            })
    }

    /// Decodes the item from bytes.
    ///
    /// On success, returns the decoded data and the number of bytes consumed
    /// to produce it.
    ///
    /// # Errors
    /// - Fails if first byte of the data contains the wrong opcode.
    /// - Fails if data is smaller then the decoded expected size.
    pub fn decode(data: &'item [u8]) -> Result<(Self, usize), QueryActionDecodeError> {
        Ok(Self::start_decoding(data)?
            .0
            .complete_decoding()
            // TODO This error should never happen as it should be triggered by `start_decoding`
            // first, when fetching the size of the operand.
            .map_err(|code| {
                QueryActionDecodeError::UnknownQueryCode(UnknownQueryCode {
                    code,
                    remaining_data: unsafe { data.get_unchecked(1..) },
                })
            })?)
    }
}

pub struct DecodableActionQuery<'data> {
    data: *const u8,
    data_life: core::marker::PhantomData<&'data ()>,
}

impl<'data> DecodableActionQuery<'data> {
    const fn new(data: &'data [u8]) -> Self {
        Self::from_ptr(data.as_ptr())
    }

    const fn from_ptr(data: *const u8) -> Self {
        Self {
            data,
            data_life: core::marker::PhantomData,
        }
    }

    /// Decodes the size of the Item in bytes
    ///
    /// # Errors
    /// Fails if the parsed data corresponds to an invalid querycode.
    /// Returns the invalid querycode.
    ///
    /// # Safety
    /// This requires reading the data bytes that may be out of bound to be calculate.
    pub unsafe fn expected_size(&self) -> Result<usize, u8> {
        Ok(1 + self.query()?.expected_size())
    }

    /// Checks whether the given data_size is bigger than the decoded object expected size.
    ///
    /// On success, returns the size of the decoded object.
    ///
    /// # Errors
    /// - Fails if the querycode is unknown
    /// - Fails if the data_size is smaller than the required data size to decode the object.
    pub fn smaller_than(&self, data_size: usize) -> Result<usize, QueryDecodeError> {
        self.query()
            .map_err(QueryDecodeError::UnknownQueryCode)?
            .smaller_than(data_size - 1)
            .map(|size| 1 + size)
            .map_err(|size| QueryDecodeError::MissingBytes(1 + size))
    }

    pub fn group(&self) -> bool {
        unsafe { *self.data.add(0) & flag::GROUP != 0 }
    }

    pub fn response(&self) -> bool {
        unsafe { *self.data.add(0) & flag::RESPONSE != 0 }
    }

    /// # Errors
    /// Fails if the parsed data corresponds to an invalid querycode.
    /// Returns the invalid querycode.
    pub fn query(&self) -> Result<DecodableQuery<'data>, u8> {
        unsafe { Query::start_decoding_ptr(self.data.add(1)) }
    }

    /// Fully decode the Item
    ///
    /// Returns the decoded data and the number of bytes consumed to produce it.
    ///
    /// # Errors
    /// Fails if the parsed data corresponds to an invalid querycode.
    /// Returns the invalid querycode.
    pub fn complete_decoding(&self) -> Result<(ActionQuery<'data>, usize), u8> {
        let (query, query_size) = self.query()?.complete_decoding();
        Ok((
            ActionQuery {
                group: self.group(),
                response: self.response(),
                query,
            },
            1 + query_size,
        ))
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::unwrap_in_result, clippy::panic, clippy::expect_used)]
    use super::super::define::QueryComparisonType;
    use super::*;
    use crate::{
        define::{EncodableData, FileId, MaskedValue},
        varint::Varint,
    };

    #[test]
    fn known() {
        fn test(op: ActionQuery, data: &[u8]) {
            // Test op.encode_in() == data
            let mut encoded = [0_u8; 2 + 8];
            let size = op.encode_in(&mut encoded).unwrap();
            assert_eq!(size, data.len());
            assert_eq!(&encoded[..size], data);

            // Test decode(data) == op
            let (ret, size) = ActionQuery::decode(data).unwrap();
            assert_eq!(size, data.len());
            assert_eq!(ret, op);

            // Test partial_decode == op
            let (decoder, expected_size) = ActionQuery::start_decoding(data).unwrap();
            assert_eq!(expected_size, size);
            assert_eq!(
                op,
                ActionQuery {
                    group: decoder.group(),
                    response: decoder.response(),
                    query: decoder.query().unwrap().complete_decoding().0,
                }
            );
        }
        test(
            ActionQuery {
                group: false,
                response: true,
                query: Query::ComparisonWithValue(
                    super::super::comparison_with_value::ComparisonWithValue {
                        signed_data: true,
                        comparison_type: QueryComparisonType::Equal,
                        compare_value: MaskedValue::new(
                            EncodableData::new(&[0x00, 0x01, 0x02]).unwrap(),
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
        let op = ActionQuery {
            group: true,
            response: false,
            query: Query::ComparisonWithValue(
                super::super::comparison_with_value::ComparisonWithValue {
                    signed_data: true,
                    comparison_type: QueryComparisonType::Equal,
                    compare_value: MaskedValue::new(
                        EncodableData::new(&[0x00, 0x01, 0x02]).unwrap(),
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
        let (ret, size_decoded) = ActionQuery::decode(&encoded).unwrap();
        assert_eq!(size_encoded, size_decoded);
        assert_eq!(ret, op);

        // Test decode(data).encode_in() == data
        let mut encoded2 = [0_u8; TOT_SIZE];
        let size_encoded2 = op.encode_in(&mut encoded2).unwrap();
        assert_eq!(size_encoded, size_encoded2);
        assert_eq!(encoded2[..size_encoded2], encoded[..size_encoded]);
    }
}

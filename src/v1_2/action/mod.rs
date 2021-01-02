pub mod chunk;
pub mod copy_file;
pub mod create_new_file;
pub mod delete_file;
pub mod execute_file;
pub mod exist_file;
pub mod flush_file;
pub mod forward;
pub mod indirect_forward;
pub mod logic;
pub mod nop;
pub mod permission_request;
pub mod query;
pub mod read_file_data;
pub mod read_file_properties;
pub mod request_tag;
pub mod response_tag;
pub mod restore_file;
pub mod return_file_data;
pub mod return_file_properties;
pub mod status;
pub mod verify_checksum;
pub mod write_file_data;
pub mod write_file_properties;

use crate::v1_2::define::op_code::OpCode;
use crate::v1_2::error::{ActionDecodeError, PtrActionDecodeError};

use nop::{DecodableNop, Nop};
use query::action_query::{ActionQuery, DecodableActionQuery};
use read_file_data::{DecodableReadFileData, ReadFileData};
use read_file_properties::{DecodableReadFileProperties, ReadFileProperties};
use status::{DecodableStatus, Status};
use write_file_data::{DecodableWriteFileData, WriteFileData};

// TODO SPEC: Why are some actions named "return". Removing that from the name would still
// be technically correct: The operand "File data" contains file data. Seems good enough.
// We can still keep the description mentionning it is supposed to be a response.
// But we could also generalize the description ... After all...
//
// This does not apply to tag response/request where knowing if it is a request or a
// response is important.

// TODO SPEC: Is BreakQuery still pertinent in v1.3 as it is equivalent to:
// [ActionQuery, Break]

// TODO Extension

/// An ALP Action
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Action<'item> {
    // Nop
    Nop(Nop),
    // Read
    ReadFileData(ReadFileData),
    ReadFileProperties(ReadFileProperties),

    // Write
    WriteFileData(WriteFileData<'item>),
    // WriteFileProperties(WriteFileProperties),
    ActionQuery(ActionQuery<'item>),
    // BreakQuery(BreakQuery),
    // TODO
    // PermissionRequest(PermissionRequest),
    // VerifyChecksum(VerifyChecksum),

    // // Management
    // ExistFile(ExistFile),
    // CreateNewFile(CreateNewFile),
    // DeleteFile(DeleteFile),
    // RestoreFile(RestoreFile),
    // FlushFile(FlushFile),
    // CopyFile(CopyFile),
    // ExecuteFile(ExecuteFile),

    // // Response
    // ReturnFileData(ReturnFileData),
    // ReturnFileProperties(ReturnFileProperties),
    Status(Status<'item>),
    // ResponseTag(ResponseTag),

    // // Special
    // Chunk(Chunk),
    // Logic(Logic),
    // TODO
    // Forward(Forward),
    // IndirectForward(IndirectForward),
    // RequestTag(RequestTag),

    // // TODO
    // Extension(Extension),
}

// TODO Put action decoding behind feature flags
// TODO Codec interface

impl<'item> Action<'item> {
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
            Self::Nop(action) => action.encode_in_ptr(out),
            Self::ReadFileData(action) => action.encode_in_ptr(out),
            Self::ReadFileProperties(action) => action.encode_in_ptr(out),
            Self::WriteFileData(action) => action.encode_in_ptr(out),
            Self::ActionQuery(action) => action.encode_in_ptr(out),
            Self::Status(action) => action.encode_in_ptr(out),
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
            Self::Nop(action) => action.size(),
            Self::ReadFileData(action) => action.size(),
            Self::ReadFileProperties(action) => action.size(),
            Self::WriteFileData(action) => action.size(),
            Self::ActionQuery(action) => action.size(),
            Self::Status(action) => action.size(),
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
    /// - Fails if the decoded data contains an invalid opcode. Returning the opcode.
    /// - Fails if one of the actions found is unparseable.
    ///
    /// # Safety
    /// You are to check that:
    /// - The decodable object fits in the given data:
    /// [`decodable.smaller_than(data.len())`](struct.DecodableAction.html#method.smaller_than)
    ///
    /// Failing that might result in reading and interpreting data outside the given
    /// array (depending on what is done with the resulting object).
    pub unsafe fn start_decoding_ptr<'data>(
        data: *const u8,
    ) -> Result<DecodableAction<'data>, PtrActionDecodeError<'data>> {
        DecodableAction::from_ptr(data)
    }

    /// Creates a decodable item without checking the data size.
    ///
    /// # Errors
    /// - Fails if the decoded data contains an invalid opcode. Returning the opcode.
    /// - Fails if one of the actions found is unparseable.
    ///
    /// # Safety
    /// You are to check that:
    /// - The decodable object fits in the given data:
    /// [`decodable.smaller_than(data.len())`](struct.DecodableAction.html#method.smaller_than)
    ///
    /// Failing that might result in reading and interpreting data outside the given
    /// array (depending on what is done with the resulting object).
    pub unsafe fn start_decoding_unchecked(
        data: &[u8],
    ) -> Result<DecodableAction, ActionDecodeError> {
        DecodableAction::new(data)
    }

    /// Returns a Decodable object and its expected byte size.
    ///
    /// This decodable item allows each parts of the item to be decoded independently.
    ///
    /// # Errors
    /// - Fails if first byte of the data contains an invalid opcode.
    /// - Fails if one of the actions found is unparseable.
    /// - Fails if data is smaller then the decoded expected size.
    pub fn start_decoding(data: &[u8]) -> Result<(DecodableAction, usize), ActionDecodeError> {
        if data.is_empty() {
            return Err(ActionDecodeError::MissingBytes(1));
        }
        let ret = unsafe { Self::start_decoding_unchecked(data)? };
        let size = ret
            .smaller_than(data.len())
            .map_err(ActionDecodeError::MissingBytes)?;
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
    /// - Fails if the decoded data contains an invalid opcode. Returning the opcode.
    /// - Fails if one of the actions found is unparseable.
    ///
    /// # Safety
    /// You are to check that:
    /// - The first byte contains this action's opcode.
    /// - The resulting size of the data consumed is smaller than the size of the
    /// decoded data.
    ///
    /// Failing that might result in reading and interpreting data outside the given
    /// array (depending on what is done with the resulting object).
    pub unsafe fn decode_ptr(
        data: *const u8,
    ) -> Result<(Self, usize), PtrActionDecodeError<'item>> {
        Ok(Self::start_decoding_ptr(data)?.complete_decoding())
    }

    /// Decodes the Item from bytes.
    ///
    /// Returns the decoded data and the number of bytes consumed to produce it.
    ///
    /// # Errors
    /// - Fails if the decoded data contains an invalid opcode. Returning the opcode.
    /// - Fails if one of the actions found is unparseable.
    ///
    /// # Safety
    /// You are to check that:
    /// - The first byte contains this action's opcode.
    /// - The resulting size of the data consumed is smaller than the size of the
    /// decoded data.
    ///
    /// Failing that might result in reading and interpreting data outside the given
    /// array (depending on what is done with the resulting object).
    pub unsafe fn decode_unchecked(data: &'item [u8]) -> Result<(Self, usize), ActionDecodeError> {
        Ok(Self::start_decoding_unchecked(data)?.complete_decoding())
    }

    /// Decodes the item from bytes.
    ///
    /// # Errors
    /// - Fails if first byte of the data contains an invalid opcode.
    /// - Fails if one of the actions found is unparseable.
    /// - Fails if data is smaller then the decoded expected size.
    pub fn decode(data: &'item [u8]) -> Result<(Self, usize), ActionDecodeError> {
        Ok(Self::start_decoding(data)?.0.complete_decoding())
    }
}

pub enum DecodableAction<'data> {
    Nop(DecodableNop<'data>),
    ReadFileData(DecodableReadFileData<'data>),
    ReadFileProperties(DecodableReadFileProperties<'data>),
    WriteFileData(DecodableWriteFileData<'data>),
    ActionQuery(DecodableActionQuery<'data>),
    Status(DecodableStatus<'data>),
}

impl<'data> DecodableAction<'data> {
    /// # Errors
    /// Fails if the opcode is invalid. Returning the opcode.
    ///
    /// # Safety
    /// The data has to contain at least one byte.
    pub unsafe fn new(data: &'data [u8]) -> Result<Self, ActionDecodeError<'data>> {
        let code = *data.get_unchecked(0) & 0x3F;
        let op_code = match OpCode::from(code) {
            Ok(code) => code,
            Err(_) => return Err(ActionDecodeError::UnknownActionCode(code)),
        };
        Ok(match op_code {
            OpCode::Nop => Self::Nop(Nop::start_decoding_unchecked(data)),
            OpCode::ReadFileData => {
                Self::ReadFileData(ReadFileData::start_decoding_unchecked(data))
            }
            OpCode::ReadFileProperties => {
                Self::ReadFileProperties(ReadFileProperties::start_decoding_unchecked(data))
            }
            OpCode::WriteFileData => {
                Self::WriteFileData(WriteFileData::start_decoding_unchecked(data))
            }
            OpCode::ActionQuery => {
                if data.len() < 2 {
                    return Err(ActionDecodeError::MissingBytes(2));
                }
                Self::ActionQuery(ActionQuery::start_decoding_unchecked(data)?)
            }
            OpCode::Status => Self::Status(Status::start_decoding_unchecked(data)?),
        })
    }

    /// # Errors
    /// Fails if the opcode is invalid. Returning the opcode.
    ///
    /// # Safety
    /// The data has to contain at least one byte.
    unsafe fn from_ptr(data: *const u8) -> Result<Self, PtrActionDecodeError<'data>> {
        let code = *data.offset(0) & 0x3F;
        let op_code = match OpCode::from(code) {
            Ok(code) => code,
            Err(_) => return Err(PtrActionDecodeError::UnknownActionCode(code)),
        };
        Ok(match op_code {
            OpCode::Nop => Self::Nop(Nop::start_decoding_ptr(data)),
            OpCode::ReadFileData => Self::ReadFileData(ReadFileData::start_decoding_ptr(data)),
            OpCode::ReadFileProperties => {
                Self::ReadFileProperties(ReadFileProperties::start_decoding_ptr(data))
            }
            OpCode::WriteFileData => Self::WriteFileData(WriteFileData::start_decoding_ptr(data)),
            OpCode::ActionQuery => Self::ActionQuery(ActionQuery::start_decoding_ptr(data)?),
            OpCode::Status => Self::Status(Status::start_decoding_ptr(data)?),
        })
    }

    /// Decodes the size of the Item in bytes
    ///
    /// # Safety
    /// This requires reading the data bytes that may be out of bound to be calculate.
    pub unsafe fn expected_size(&self) -> usize {
        match self {
            Self::Nop(action) => action.expected_size(),
            Self::ReadFileData(action) => action.expected_size(),
            Self::ReadFileProperties(action) => action.expected_size(),
            Self::WriteFileData(action) => action.expected_size(),
            Self::ActionQuery(action) => action.expected_size(),
            Self::Status(action) => action.expected_size(),
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
            Self::Nop(action) => action.smaller_than(data_size),
            Self::ReadFileData(action) => action.smaller_than(data_size),
            Self::ReadFileProperties(action) => action.smaller_than(data_size),
            Self::WriteFileData(action) => action.smaller_than(data_size),
            Self::ActionQuery(action) => action.smaller_than(data_size),
            Self::Status(action) => action.smaller_than(data_size),
        }
    }

    /// Fully decode the Item
    ///
    /// Returns the decoded data and the number of bytes consumed to produce it.
    pub fn complete_decoding(&self) -> (Action<'data>, usize) {
        match self {
            Self::Nop(action) => {
                let (action, size) = action.complete_decoding();
                (Action::Nop(action), size)
            }
            Self::ReadFileData(action) => {
                let (action, size) = action.complete_decoding();
                (Action::ReadFileData(action), size)
            }
            Self::ReadFileProperties(action) => {
                let (action, size) = action.complete_decoding();
                (Action::ReadFileProperties(action), size)
            }
            Self::WriteFileData(action) => {
                let (action, size) = action.complete_decoding();
                (Action::WriteFileData(action), size)
            }
            Self::ActionQuery(action) => {
                let (action, size) = action.complete_decoding();
                (Action::ActionQuery(action), size)
            }
            Self::Status(action) => {
                let (action, size) = action.complete_decoding();
                (Action::Status(action), size)
            }
        }
    }
}

// TODO Add action specific test cases to verify that smaller_than does its job correctly: No out
// of bound read.

#[cfg(test)]
mod test {
    #![allow(clippy::unwrap_in_result, clippy::panic, clippy::expect_used)]
    use super::*;
    use crate::define::{EncodableData, FileId, MaskedValue};
    use crate::v1_2::dash7::{
        addressee::{AccessClass, Addressee, AddresseeIdentifier, NlsMethod},
        interface_status::{AddresseeWithNlsState, Dash7InterfaceStatus},
    };
    use crate::varint::Varint;

    #[test]
    fn known() {
        fn test(op: Action, data: &[u8]) {
            // Test op.encode_in() == data
            let mut encoded = [0_u8; 64];
            let size = op.encode_in(&mut encoded).unwrap();
            assert_eq!(size, data.len());
            assert_eq!(&encoded[..size], data);

            // Test decode(data) == op
            let (ret, size) = Action::decode(data).unwrap();
            assert_eq!(size, data.len());
            assert_eq!(ret, op);

            // Test partial_decode == op
            let (decoder, expected_size) = Action::start_decoding(data).unwrap();
            assert_eq!(expected_size, size);
            assert_eq!(unsafe { decoder.expected_size() }, size);
            assert_eq!(decoder.smaller_than(data.len()).unwrap(), size);
        }
        test(
            Action::Nop(Nop {
                group: false,
                response: true,
            }),
            &[0x40],
        );
        test(
            Action::ReadFileProperties(ReadFileProperties {
                group: false,
                response: false,
                file_id: FileId::new(0xFF),
            }),
            &[0x02, 0xFF],
        );
        test(
            Action::ReadFileData(ReadFileData {
                group: false,
                response: false,
                file_id: FileId::new(0xFF),
                offset: Varint::new(0x3F_FF_FF_FF).unwrap(),
                length: Varint::new(0x3F_FF_FF_FF).unwrap(),
            }),
            &[0x01, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
        );
        test(
            Action::WriteFileData(WriteFileData {
                group: false,
                response: false,
                file_id: FileId::new(0xFF),
                offset: Varint::new(0x3F_FF_FF_FF).unwrap(),
                data: EncodableData::new(&[0xFF, 0xFE]).unwrap(),
            }),
            &[0x04, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x02, 0xFF, 0xFE],
        );
        test(
            Action::ActionQuery(ActionQuery {
                group: false,
                response: true,
                query: query::Query::ComparisonWithValue(
                    query::comparison_with_value::ComparisonWithValue {
                        signed_data: true,
                        comparison_type: query::define::QueryComparisonType::Equal,
                        compare_value: MaskedValue::new(
                            EncodableData::new(&[0x00, 0x01, 0x02]).unwrap(),
                            None,
                        )
                        .unwrap(),
                        file_id: FileId::new(0x42),
                        offset: Varint::new(0x40_00).unwrap(),
                    },
                ),
            }),
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
        test(
            Action::Status(Status::Interface(
                status::interface::StatusInterface::Dash7(Dash7InterfaceStatus {
                    ch_header: 0x1,
                    ch_idx: 0x2,
                    rxlev: 0x3,
                    lb: 0x4,
                    snr: 0x5,
                    status: 0x6,
                    token: 0x7,
                    seq: 0x8,
                    resp_to: 0x9,
                    addressee_with_nls_state: AddresseeWithNlsState::new(
                        Addressee {
                            nls_method: NlsMethod::None,
                            access_class: AccessClass(0xE1),
                            identifier: AddresseeIdentifier::Uid(&[
                                0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
                            ]),
                        },
                        None,
                    )
                    .unwrap(),
                }),
            )),
            &[
                34 | 0x40,
                0xD7,
                0x14,
                0x01,
                0x02,
                0x00,
                0x03,
                0x04,
                0x05,
                0x06,
                0x07,
                0x08,
                0x09,
                0x20,
                0xE1,
                0x00,
                0x11,
                0x22,
                0x33,
                0x44,
                0x55,
                0x66,
                0x77,
            ],
        );
    }

    #[test]
    fn consistence() {
        const TOT_SIZE: usize = 1 + 1 + 3 + 3 + 1 + 3;
        let op = Action::ActionQuery(ActionQuery {
            group: true,
            response: false,
            query: query::Query::ComparisonWithValue(
                query::comparison_with_value::ComparisonWithValue {
                    signed_data: true,
                    comparison_type: query::define::QueryComparisonType::Equal,
                    compare_value: MaskedValue::new(
                        EncodableData::new(&[0x00, 0x01, 0x02]).unwrap(),
                        None,
                    )
                    .unwrap(),
                    file_id: FileId::new(0x42),
                    offset: Varint::new(0x40_00).unwrap(),
                },
            ),
        });

        // Test decode(op.encode_in()) == op
        let mut encoded = [0_u8; TOT_SIZE];
        let size_encoded = op.encode_in(&mut encoded).unwrap();
        let (ret, size_decoded) = Action::decode(&encoded).unwrap();
        assert_eq!(size_encoded, size_decoded);
        assert_eq!(ret, op);

        // Test decode(data).encode_in() == data
        let mut encoded2 = [0_u8; TOT_SIZE];
        let size_encoded2 = op.encode_in(&mut encoded2).unwrap();
        assert_eq!(size_encoded, size_encoded2);
        assert_eq!(encoded2[..size_encoded2], encoded[..size_encoded]);
    }
}

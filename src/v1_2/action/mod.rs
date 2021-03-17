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

#[cfg(feature = "decode_action")]
use crate::v1_2::define::op_code::OpCode;
#[cfg(feature = "decode_action")]
use crate::v1_2::error::{ActionDecodeError, ActionSizeError, UnsupportedOpCode};

#[cfg(feature = "decode_nop")]
use nop::{EncodedNop, EncodedNopMut};
#[cfg(feature = "decode_action_query")]
use query::action_query::{DecodedActionQueryRef, EncodedActionQuery, EncodedActionQueryMut};
#[cfg(feature = "decode_read_file_data")]
use read_file_data::{EncodedReadFileData, EncodedReadFileDataMut};
#[cfg(feature = "decode_read_file_properties")]
use read_file_properties::{EncodedReadFileProperties, EncodedReadFilePropertiesMut};
#[cfg(feature = "decode_status")]
use status::{EncodedStatus, EncodedStatusMut};
#[cfg(feature = "decode_write_file_data")]
use write_file_data::{EncodedWriteFileData, EncodedWriteFileDataMut};

#[cfg(feature = "alloc")]
#[cfg(feature = "action_query")]
use query::action_query::ActionQuery;
#[cfg(feature = "alloc")]
#[cfg(feature = "write_file_data")]
use write_file_data::WriteFileData;

#[cfg(feature = "nop")]
use nop::{Nop, NopRef};
#[cfg(feature = "action_query")]
use query::action_query::ActionQueryRef;
#[cfg(feature = "read_file_data")]
use read_file_data::{ReadFileData, ReadFileDataRef};
#[cfg(feature = "read_file_properties")]
use read_file_properties::{ReadFileProperties, ReadFilePropertiesRef};
#[cfg(feature = "status")]
use status::{Status, StatusRef};
#[cfg(feature = "write_file_data")]
use write_file_data::WriteFileDataRef;

#[cfg(any(
    feature = "decode_nop",
    feature = "decode_read_file_data",
    feature = "decode_read_file_properties",
    feature = "decode_write_file_data"
))]
use crate::decodable::EncodedData;
#[cfg(feature = "decode_action")]
use crate::decodable::FailableEncodedData;
#[cfg(feature = "decode_action")]
use crate::decodable::WithByteSize;
use crate::encodable::Encodable;

// TODO SPEC: Why are some actions named "return". Removing that from the name would still
// be technically correct: The operand "File data" contains file data. Seems good enough.
// We can still keep the description mentionning it is supposed to be a response.
// But we could also generalize the description ... After all...
//
// This does not apply to tag response/request where knowing if it is a request or a
// response is important.

// TODO SPEC: Is BreakQuery still pertinent in v1.3 as it is equivalent to:
// [ActionQuery, Break]

/// An ALP Action
///
/// It does not own its data references.
#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum ActionRef<'item, 'data> {
    // Nop
    #[cfg(feature = "nop")]
    Nop(NopRef<'item, 'data>),
    // Read
    #[cfg(feature = "read_file_data")]
    ReadFileData(ReadFileDataRef<'item, 'data>),
    #[cfg(feature = "read_file_properties")]
    ReadFileProperties(ReadFilePropertiesRef<'item, 'data>),

    // Write
    #[cfg(feature = "write_file_data")]
    WriteFileData(WriteFileDataRef<'item, 'data>),
    // WriteFileProperties(WriteFileProperties),
    #[cfg(feature = "action_query")]
    ActionQuery(ActionQueryRef<'item, 'data>),
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
    #[cfg(feature = "status")]
    Status(StatusRef<'item, 'data>),
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

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ActionToOwnedError {
    RequiresAllocation,
}

impl<'item, 'data> Encodable for ActionRef<'item, 'data> {
    unsafe fn encode_in_ptr(&self, out: *mut u8) -> usize {
        match self {
            #[cfg(feature = "nop")]
            Self::Nop(action) => action.encode_in_ptr(out),
            #[cfg(feature = "read_file_data")]
            Self::ReadFileData(action) => action.encode_in_ptr(out),
            #[cfg(feature = "read_file_properties")]
            Self::ReadFileProperties(action) => action.encode_in_ptr(out),
            #[cfg(feature = "write_file_data")]
            Self::WriteFileData(action) => action.encode_in_ptr(out),
            #[cfg(feature = "action_query")]
            Self::ActionQuery(action) => action.encode_in_ptr(out),
            #[cfg(feature = "status")]
            Self::Status(action) => action.encode_in_ptr(out),
        }
    }

    fn encoded_size(&self) -> usize {
        match self {
            #[cfg(feature = "nop")]
            Self::Nop(action) => action.encoded_size(),
            #[cfg(feature = "read_file_data")]
            Self::ReadFileData(action) => action.encoded_size(),
            #[cfg(feature = "read_file_properties")]
            Self::ReadFileProperties(action) => action.encoded_size(),
            #[cfg(feature = "write_file_data")]
            Self::WriteFileData(action) => action.encoded_size(),
            #[cfg(feature = "action_query")]
            Self::ActionQuery(action) => action.encoded_size(),
            #[cfg(feature = "status")]
            Self::Status(action) => action.encoded_size(),
        }
    }
}

impl<'item, 'data> ActionRef<'item, 'data> {
    /// Copies all the reference based data to create a fully self contained
    /// object.
    ///
    /// # Errors
    /// Fails only if the alloc flag is not set and the owned action requires the
    /// alloc feature.
    pub fn to_owned(&self) -> Result<Action, ActionToOwnedError> {
        Ok(match self {
            #[cfg(feature = "nop")]
            Self::Nop(action) => Action::Nop(action.to_owned()),
            #[cfg(feature = "read_file_data")]
            Self::ReadFileData(action) => Action::ReadFileData(action.to_owned()),
            #[cfg(feature = "read_file_properties")]
            Self::ReadFileProperties(action) => Action::ReadFileProperties(action.to_owned()),
            #[cfg(feature = "write_file_data")]
            #[cfg(feature = "alloc")]
            Self::WriteFileData(action) => Action::WriteFileData(action.to_owned()),
            #[cfg(feature = "action_query")]
            #[cfg(feature = "alloc")]
            Self::ActionQuery(action) => Action::ActionQuery(action.to_owned()),
            #[cfg(feature = "status")]
            Self::Status(action) => Action::Status(action.to_owned()),
            // TODO This should be an enumeration of the actions instead of all_actions, in case
            // they are selected manually.
            #[cfg_attr(not(feature = "all_actions"), allow(unreachable_patterns))]
            #[allow(unreachable_patterns)]
            _ => return Err(ActionToOwnedError::RequiresAllocation),
        })
    }
}

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[cfg(feature = "decode_action")]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum DecodedActionRef<'item, 'data> {
    #[cfg(feature = "decode_nop")]
    // Nop
    Nop(NopRef<'item, 'data>),
    // Read
    #[cfg(feature = "decode_read_file_data")]
    ReadFileData(ReadFileDataRef<'item, 'data>),
    #[cfg(feature = "decode_read_file_properties")]
    ReadFileProperties(ReadFilePropertiesRef<'item, 'data>),

    // Write
    #[cfg(feature = "decode_write_file_data")]
    WriteFileData(WriteFileDataRef<'item, 'data>),
    // WriteFileProperties(WriteFileProperties),
    #[cfg(feature = "decode_action_query")]
    ActionQuery(DecodedActionQueryRef<'item, 'data>),
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
    #[cfg(feature = "decode_status")]
    Status(StatusRef<'item, 'data>),
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

#[cfg(feature = "decode_action")]
impl<'item, 'data> DecodedActionRef<'item, 'data> {
    pub fn as_encodable(self) -> ActionRef<'item, 'data> {
        self.into()
    }
}

#[cfg(feature = "decode_action")]
impl<'item, 'data> From<DecodedActionRef<'item, 'data>> for ActionRef<'item, 'data> {
    fn from(decoded: DecodedActionRef<'item, 'data>) -> Self {
        match decoded {
            #[cfg(feature = "decode_nop")]
            DecodedActionRef::Nop(action) => ActionRef::Nop(action),
            #[cfg(feature = "decode_read_file_data")]
            DecodedActionRef::ReadFileData(action) => ActionRef::ReadFileData(action),
            #[cfg(feature = "decode_read_file_properties")]
            DecodedActionRef::ReadFileProperties(action) => ActionRef::ReadFileProperties(action),
            #[cfg(all(feature = "decode_write_file_data"))]
            DecodedActionRef::WriteFileData(action) => ActionRef::WriteFileData(action),
            #[cfg(all(feature = "decode_action_query"))]
            DecodedActionRef::ActionQuery(action) => ActionRef::ActionQuery(action.as_encodable()),
            #[cfg(feature = "decode_status")]
            DecodedActionRef::Status(action) => ActionRef::Status(action),
        }
    }
}

#[cfg(feature = "decode_action")]
pub enum ValidEncodedAction<'item, 'data> {
    #[cfg(feature = "decode_nop")]
    Nop(EncodedNop<'item, 'data>),
    #[cfg(feature = "decode_read_file_data")]
    ReadFileData(EncodedReadFileData<'item, 'data>),
    #[cfg(feature = "decode_read_file_properties")]
    ReadFileProperties(EncodedReadFileProperties<'item, 'data>),
    #[cfg(feature = "decode_write_file_data")]
    WriteFileData(EncodedWriteFileData<'item, 'data>),
    #[cfg(feature = "decode_action_query")]
    ActionQuery(EncodedActionQuery<'item, 'data>),
    #[cfg(feature = "decode_status")]
    Status(EncodedStatus<'item, 'data>),
}

#[cfg(feature = "decode_action")]
impl<'item, 'data> EncodedAction<'item, 'data> {
    /// # Errors
    /// Fails if the op code is unsupported.
    pub fn op_code<'result>(&self) -> Result<OpCode, UnsupportedOpCode<'result, 'data>> {
        let code = unsafe { *self.data.get_unchecked(0) & 0x3F };
        OpCode::from(code).map_err(|_| UnsupportedOpCode {
            op_code: code,
            remaining_data: self.data,
        })
    }

    /// # Errors
    /// Fails if the op code is unsupported.
    pub fn action<'result>(
        &self,
    ) -> Result<ValidEncodedAction<'result, 'data>, UnsupportedOpCode<'result, 'data>> {
        unsafe {
            Ok(match self.op_code()? {
                #[cfg(feature = "decode_nop")]
                OpCode::Nop => ValidEncodedAction::Nop(NopRef::start_decoding_unchecked(self.data)),
                #[cfg(feature = "decode_read_file_data")]
                OpCode::ReadFileData => ValidEncodedAction::ReadFileData(
                    ReadFileDataRef::start_decoding_unchecked(self.data),
                ),
                #[cfg(feature = "decode_read_file_properties")]
                OpCode::ReadFileProperties => ValidEncodedAction::ReadFileProperties(
                    ReadFilePropertiesRef::start_decoding_unchecked(self.data),
                ),
                #[cfg(feature = "decode_write_file_data")]
                OpCode::WriteFileData => ValidEncodedAction::WriteFileData(
                    WriteFileDataRef::start_decoding_unchecked(self.data),
                ),
                #[cfg(feature = "decode_action_query")]
                OpCode::ActionQuery => ValidEncodedAction::ActionQuery(
                    DecodedActionQueryRef::start_decoding_unchecked(self.data),
                ),
                #[cfg(feature = "decode_status")]
                OpCode::Status => {
                    ValidEncodedAction::Status(StatusRef::start_decoding_unchecked(self.data))
                }
                op_code => {
                    return Err(UnsupportedOpCode {
                        op_code: op_code as u8,
                        remaining_data: self.data,
                    })
                }
            })
        }
    }
}

#[cfg(feature = "decode_action")]
pub struct EncodedAction<'item, 'data> {
    data: &'item &'data [u8],
}

#[cfg(feature = "decode_action")]
impl<'item, 'data> EncodedAction<'item, 'data> {
    pub(crate) unsafe fn new(data: &'data [u8]) -> Self {
        Self { data }
    }

    pub fn encoded_size<'result>(&self) -> Result<usize, ActionSizeError<'result, 'data>> {
        Ok(match self.action()? {
            #[cfg(feature = "decode_nop")]
            ValidEncodedAction::Nop(action) => action.encoded_size()?,
            #[cfg(feature = "decode_read_file_data")]
            ValidEncodedAction::ReadFileData(action) => action.encoded_size()?,
            #[cfg(feature = "decode_read_file_properties")]
            ValidEncodedAction::ReadFileProperties(action) => action.encoded_size()?,
            #[cfg(feature = "decode_write_file_data")]
            ValidEncodedAction::WriteFileData(action) => action.encoded_size()?,
            #[cfg(feature = "decode_action_query")]
            ValidEncodedAction::ActionQuery(action) => action.encoded_size()?,
            #[cfg(feature = "decode_status")]
            ValidEncodedAction::Status(action) => action.encoded_size()?,
        })
    }

    pub fn complete_decoding<'result>(
        &self,
    ) -> Result<WithByteSize<DecodedActionRef<'result, 'data>>, ActionDecodeError<'result, 'data>>
    {
        Ok(match self.action()? {
            #[cfg(feature = "decode_nop")]
            ValidEncodedAction::Nop(action) => {
                action.complete_decoding().map(DecodedActionRef::Nop)
            }
            #[cfg(feature = "decode_read_file_data")]
            ValidEncodedAction::ReadFileData(action) => action
                .complete_decoding()
                .map(DecodedActionRef::ReadFileData),
            #[cfg(feature = "decode_read_file_properties")]
            ValidEncodedAction::ReadFileProperties(action) => action
                .complete_decoding()
                .map(DecodedActionRef::ReadFileProperties),
            #[cfg(feature = "decode_write_file_data")]
            ValidEncodedAction::WriteFileData(action) => action
                .complete_decoding()
                .map(DecodedActionRef::WriteFileData),
            #[cfg(feature = "decode_action_query")]
            ValidEncodedAction::ActionQuery(action) => action
                .complete_decoding()?
                .map(DecodedActionRef::ActionQuery),
            #[cfg(feature = "decode_status")]
            ValidEncodedAction::Status(action) => {
                action.complete_decoding()?.map(DecodedActionRef::Status)
            }
        })
    }
}

#[cfg(feature = "decode_action")]
pub struct EncodedActionMut<'item, 'data> {
    data: &'item mut &'data mut [u8],
}

#[cfg(feature = "decode_action")]
pub enum ValidEncodedActionMut<'item, 'data> {
    #[cfg(feature = "decode_nop")]
    Nop(EncodedNopMut<'item, 'data>),
    #[cfg(feature = "decode_read_file_data")]
    ReadFileData(EncodedReadFileDataMut<'item, 'data>),
    #[cfg(feature = "decode_read_file_properties")]
    ReadFileProperties(EncodedReadFilePropertiesMut<'item, 'data>),
    #[cfg(feature = "decode_write_file_data")]
    WriteFileData(EncodedWriteFileDataMut<'item, 'data>),
    #[cfg(feature = "decode_action_query")]
    ActionQuery(EncodedActionQueryMut<'item, 'data>),
    #[cfg(feature = "decode_status")]
    Status(EncodedStatusMut<'item, 'data>),
}

#[cfg(feature = "decode_action")]
impl<'item, 'data> EncodedActionMut<'item, 'data> {
    pub const fn as_ref<'result>(&self) -> EncodedAction<'result, 'data> {
        EncodedAction::new(self.data)
    }

    /// # Errors
    /// Fails if the op code is unsupported.
    pub fn op_code<'result>(&self) -> Result<OpCode, UnsupportedOpCode<'result, 'data>> {
        self.as_ref().op_code()
    }

    /// # Errors
    /// Fails if the op code is unsupported.
    pub fn action<'result>(
        &self,
    ) -> Result<ValidEncodedAction<'result, 'data>, UnsupportedOpCode<'result, 'data>> {
        self.as_ref().action()
    }

    /// # Safety
    /// Changing this breaks the payload.
    /// It should never be used.
    /// Make sure that you change the following payload to keep it coherent.
    pub unsafe fn set_op_code(&mut self, op_code: OpCode) {
        *self.data.get_unchecked_mut(0) &= 0xC0;
        *self.data.get_unchecked_mut(0) |= op_code as u8;
    }

    /// # Errors
    /// Fails if the op code is unsupported.
    pub fn action_mut<'result>(
        &'data mut self,
    ) -> Result<ValidEncodedActionMut<'result, 'data>, UnsupportedOpCode<'result, 'data>> {
        unsafe {
            Ok(match self.op_code()? {
                #[cfg(feature = "decode_nop")]
                OpCode::Nop => {
                    ValidEncodedActionMut::Nop(NopRef::start_decoding_unchecked_mut(self.data))
                }
                #[cfg(feature = "decode_read_file_data")]
                OpCode::ReadFileData => ValidEncodedActionMut::ReadFileData(
                    ReadFileDataRef::start_decoding_unchecked_mut(self.data),
                ),
                #[cfg(feature = "decode_read_file_properties")]
                OpCode::ReadFileProperties => ValidEncodedActionMut::ReadFileProperties(
                    ReadFilePropertiesRef::start_decoding_unchecked_mut(self.data),
                ),
                #[cfg(feature = "decode_write_file_data")]
                OpCode::WriteFileData => ValidEncodedActionMut::WriteFileData(
                    WriteFileDataRef::start_decoding_unchecked_mut(self.data),
                ),
                #[cfg(feature = "decode_action_query")]
                OpCode::ActionQuery => ValidEncodedActionMut::ActionQuery(
                    DecodedActionQueryRef::start_decoding_unchecked_mut(self.data),
                ),
                #[cfg(feature = "decode_status")]
                OpCode::Status => ValidEncodedActionMut::Status(
                    StatusRef::start_decoding_unchecked_mut(self.data),
                ),
                op_code => {
                    return Err(UnsupportedOpCode {
                        op_code: op_code as u8,
                        remaining_data: &&self.data[..],
                    })
                }
            })
        }
    }
}

#[cfg(feature = "decode_action")]
impl<'item, 'data> EncodedActionMut<'item, 'data> {
    pub(crate) unsafe fn new(data: &'data mut [u8]) -> Self {
        Self { data: &mut data }
    }

    pub fn encoded_size<'result>(&self) -> Result<usize, ActionSizeError<'result, 'data>> {
        self.as_ref().encoded_size()
    }

    pub fn complete_decoding<'result>(
        &self,
    ) -> Result<WithByteSize<DecodedActionRef<'result, 'data>>, ActionDecodeError<'result, 'data>>
    {
        self.as_ref().complete_decoding()
    }
}

#[cfg(feature = "decode_action")]
crate::make_failable_decodable!(
    DecodedActionRef,
    EncodedAction,
    EncodedActionMut,
    ActionSizeError,
    ActionDecodeError,
    ActionSizeError
);

/// An Owned ALP Action
#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Action {
    // Nop
    #[cfg(feature = "nop")]
    Nop(Nop),
    // Read
    #[cfg(feature = "read_file_data")]
    ReadFileData(ReadFileData),
    #[cfg(feature = "read_file_properties")]
    ReadFileProperties(ReadFileProperties),

    // Write
    #[cfg(feature = "alloc")]
    #[cfg(feature = "write_file_data")]
    WriteFileData(WriteFileData),
    // WriteFileProperties(WriteFileProperties),
    #[cfg(feature = "alloc")]
    #[cfg(feature = "action_query")]
    ActionQuery(ActionQuery),
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
    #[cfg(feature = "status")]
    Status(Status),
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

#[cfg(feature = "decode_action")]
impl Action {
    pub fn as_ref(&self) -> ActionRef {
        match self {
            #[cfg(feature = "nop")]
            Self::Nop(action) => ActionRef::Nop(action.as_ref()),
            #[cfg(feature = "read_file_data")]
            Self::ReadFileData(action) => ActionRef::ReadFileData(action.as_ref()),
            #[cfg(feature = "read_file_properties")]
            Self::ReadFileProperties(action) => ActionRef::ReadFileProperties(action.as_ref()),
            #[cfg(feature = "alloc")]
            #[cfg(feature = "write_file_data")]
            Self::WriteFileData(action) => ActionRef::WriteFileData(action.as_ref()),
            #[cfg(feature = "alloc")]
            #[cfg(feature = "action_query")]
            Self::ActionQuery(action) => ActionRef::ActionQuery(action.as_ref()),
            #[cfg(feature = "status")]
            Self::Status(action) => ActionRef::Status(action.as_ref()),
        }
    }
}

// TODO Add action specific test cases to verify that smaller_than does its job correctly: No out
// of bound read.

#[cfg(feature = "decode_action")]
#[cfg(test)]
mod test {
    #![allow(clippy::unwrap_in_result, clippy::panic, clippy::expect_used)]
    use super::*;
    use crate::decodable::WithByteSize;
    #[cfg(any(
        all(
            feature = "decode_action_query",
            feature = "decode_query_compare_with_value"
        ),
        feature = "decode_write_file_data"
    ))]
    use crate::define::EncodableDataRef;
    #[cfg(any(
        feature = "decode_read_file_data",
        feature = "decode_read_file_properties",
        feature = "decode_write_file_data",
        all(
            feature = "decode_action_query",
            feature = "decode_query_compare_with_value"
        ),
    ))]
    use crate::define::FileId;
    #[cfg(all(
        feature = "decode_action_query",
        feature = "decode_query_compare_with_value"
    ))]
    use crate::define::MaskedValueRef;
    #[cfg(feature = "decode_status")]
    use crate::v1_2::dash7::{
        addressee::{AccessClass, AddresseeIdentifierRef, AddresseeRef, NlsMethod},
        interface_status::{AddresseeWithNlsStateRef, Dash7InterfaceStatusRef},
    };
    #[cfg(any(
        feature = "decode_read_file_data",
        feature = "decode_write_file_data",
        all(
            feature = "decode_action_query",
            feature = "decode_query_compare_with_value"
        ),
    ))]
    use crate::varint::Varint;

    #[test]
    fn known() {
        #[allow(dead_code)]
        fn test(op: ActionRef, data: &[u8]) {
            // Test op.encode_in() == data
            let mut encoded = [0_u8; 64];
            let size = op.encode_in(&mut encoded).unwrap();
            assert_eq!(size, data.len());
            assert_eq!(&encoded[..size], data);

            // Test decode(data) == op
            let WithByteSize {
                item: ret,
                byte_size: size,
            } = DecodedActionRef::decode(data).unwrap();
            assert_eq!(size, data.len());
            assert_eq!(ret.as_encodable(), op);

            // Test partial_decode == op
            let WithByteSize {
                item: decoder,
                byte_size: expected_size,
            } = DecodedActionRef::start_decoding(data).unwrap();
            assert_eq!(expected_size, size);
            // TODO Should this be supported?
            // assert_eq!(unsafe { decoder.encoded_size_unchecked() }, size);
            assert_eq!(decoder.encoded_size().unwrap(), size);
        }
        #[cfg(feature = "decode_nop")]
        test(ActionRef::Nop(NopRef::new(false, true)), &[0x40]);
        #[cfg(feature = "decode_read_file_data")]
        test(
            ActionRef::ReadFileData(ReadFileDataRef {
                group: false,
                response: false,
                file_id: FileId::new(0xFF),
                offset: Varint::new(0x3F_FF_FF_FF).unwrap(),
                length: Varint::new(0x3F_FF_FF_FF).unwrap(),
                phantom: core::marker::PhantomData,
            }),
            &[0x01, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
        );
        #[cfg(feature = "decode_read_file_properties")]
        test(
            ActionRef::ReadFileProperties(ReadFilePropertiesRef::new(
                false,
                false,
                FileId::new(0xFF),
            )),
            &[0x02, 0xFF],
        );
        #[cfg(feature = "decode_write_file_data")]
        test(
            ActionRef::WriteFileData(WriteFileDataRef {
                group: false,
                response: false,
                file_id: FileId::new(0xFF),
                offset: Varint::new(0x3F_FF_FF_FF).unwrap(),
                data: EncodableDataRef::new(&[0xFF, 0xFE]).unwrap(),
            }),
            &[0x04, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x02, 0xFF, 0xFE],
        );
        #[cfg(feature = "decode_action_query")]
        #[cfg(feature = "decode_query_compare_with_value")]
        test(
            ActionRef::ActionQuery(ActionQueryRef {
                group: false,
                response: true,
                query: query::QueryRef::ComparisonWithValue(
                    query::comparison_with_value::ComparisonWithValueRef {
                        signed_data: true,
                        comparison_type: query::define::QueryComparisonType::Equal,
                        compare_value: MaskedValueRef::new(
                            EncodableDataRef::new(&[0x00, 0x01, 0x02]).unwrap(),
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
        #[cfg(feature = "decode_status")]
        test(
            ActionRef::Status(StatusRef::Interface(
                status::interface::StatusInterfaceRef::Dash7(Dash7InterfaceStatusRef {
                    ch_header: 0x1,
                    ch_idx: 0x2,
                    rxlev: 0x3,
                    lb: 0x4,
                    snr: 0x5,
                    status: 0x6,
                    token: 0x7,
                    seq: 0x8,
                    resp_to: 0x9,
                    addressee_with_nls_state: AddresseeWithNlsStateRef::new(
                        AddresseeRef {
                            nls_method: NlsMethod::None,
                            access_class: AccessClass(0xE1),
                            identifier: AddresseeIdentifierRef::Uid(&[
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

    #[cfg(feature = "decode_nop")]
    #[test]
    fn consistence() {
        const TOT_SIZE: usize = 1;
        let op = ActionRef::Nop(NopRef::new(true, false));

        // Test decode(op.encode_in()) == op
        let mut encoded = [0_u8; TOT_SIZE];
        let size_encoded = op.encode_in(&mut encoded).unwrap();
        let WithByteSize {
            item: ret,
            byte_size: size_decoded,
        } = DecodedActionRef::decode(&encoded).unwrap();
        assert_eq!(size_encoded, size_decoded);
        assert_eq!(ret.as_encodable(), op);

        // Test decode(data).encode_in() == data
        let mut encoded2 = [0_u8; TOT_SIZE];
        let size_encoded2 = op.encode_in(&mut encoded2).unwrap();
        assert_eq!(size_encoded, size_encoded2);
        assert_eq!(encoded2[..size_encoded2], encoded[..size_encoded]);
    }
}

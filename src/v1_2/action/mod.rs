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
use crate::v1_2::error::{ActionDecodeError, PtrActionDecodeError};

#[cfg(feature = "decode_nop")]
use nop::DecodableNop;
#[cfg(feature = "decode_action_query")]
use query::action_query::{DecodableActionQuery, DecodedActionQueryRef};
#[cfg(feature = "decode_read_file_data")]
use read_file_data::DecodableReadFileData;
#[cfg(feature = "decode_read_file_properties")]
use read_file_properties::DecodableReadFileProperties;
#[cfg(feature = "decode_status")]
use status::DecodableStatus;
#[cfg(feature = "decode_write_file_data")]
use write_file_data::DecodableWriteFileData;

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

// TODO Find a way to get rid of the enum lifetime if there is no action
// requiring a lifetime

/// An ALP Action
///
/// It does not own its data references.
#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum ActionRef<'item> {
    // Nop
    #[cfg(feature = "nop")]
    Nop(NopRef<'item>),
    // Read
    #[cfg(feature = "read_file_data")]
    ReadFileData(ReadFileDataRef<'item>),
    #[cfg(feature = "read_file_properties")]
    ReadFileProperties(ReadFilePropertiesRef<'item>),

    // Write
    #[cfg(feature = "write_file_data")]
    WriteFileData(WriteFileDataRef<'item>),
    // WriteFileProperties(WriteFileProperties),
    #[cfg(feature = "action_query")]
    ActionQuery(ActionQueryRef<'item>),
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
    Status(StatusRef<'item>),
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

impl<'item> ActionRef<'item> {
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
            #[cfg(feature = "nop")]
            Self::Nop(action) => action.size(),
            #[cfg(feature = "read_file_data")]
            Self::ReadFileData(action) => action.size(),
            #[cfg(feature = "read_file_properties")]
            Self::ReadFileProperties(action) => action.size(),
            #[cfg(feature = "write_file_data")]
            Self::WriteFileData(action) => action.size(),
            #[cfg(feature = "action_query")]
            Self::ActionQuery(action) => action.size(),
            #[cfg(feature = "status")]
            Self::Status(action) => action.size(),
        }
    }

    /// Copies all the reference based data to create a fully self contained
    /// object.
    ///
    /// # Errors
    /// Fails only if the alloc flag is not set and the owned action requires the
    /// alloc feature.
    pub fn to_owned(&self) -> Result<Action, ()> {
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
            _ => return Err(()),
        })
    }
}

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[cfg(feature = "decode_action")]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum DecodedActionRef<'item> {
    #[cfg(feature = "decode_nop")]
    // Nop
    Nop(NopRef<'item>),
    // Read
    #[cfg(feature = "decode_read_file_data")]
    ReadFileData(ReadFileDataRef<'item>),
    #[cfg(feature = "decode_read_file_properties")]
    ReadFileProperties(ReadFilePropertiesRef<'item>),

    // Write
    #[cfg(feature = "decode_write_file_data")]
    WriteFileData(WriteFileDataRef<'item>),
    // WriteFileProperties(WriteFileProperties),
    #[cfg(feature = "decode_action_query")]
    ActionQuery(ActionQueryRef<'item>),
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
    Status(StatusRef<'item>),
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
impl<'item> DecodedActionRef<'item> {
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

    pub fn as_action(self) -> ActionRef<'item> {
        self.into()
    }
}

#[cfg(feature = "decode_action")]
impl<'item> From<DecodedActionRef<'item>> for ActionRef<'item> {
    fn from(decoded: DecodedActionRef<'item>) -> Self {
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
            DecodedActionRef::ActionQuery(action) => ActionRef::ActionQuery(action),
            #[cfg(feature = "decode_status")]
            DecodedActionRef::Status(action) => ActionRef::Status(action),
        }
    }
}

#[cfg(feature = "decode_action")]
pub enum DecodableAction<'data> {
    #[cfg(feature = "decode_nop")]
    Nop(DecodableNop<'data>),
    #[cfg(feature = "decode_read_file_data")]
    ReadFileData(DecodableReadFileData<'data>),
    #[cfg(feature = "decode_read_file_properties")]
    ReadFileProperties(DecodableReadFileProperties<'data>),
    #[cfg(feature = "decode_write_file_data")]
    WriteFileData(DecodableWriteFileData<'data>),
    #[cfg(feature = "decode_action_query")]
    ActionQuery(DecodableActionQuery<'data>),
    #[cfg(feature = "decode_status")]
    Status(DecodableStatus<'data>),
}

#[cfg(feature = "decode_action")]
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
            #[cfg(feature = "decode_nop")]
            OpCode::Nop => Self::Nop(NopRef::start_decoding_unchecked(data)),
            #[cfg(feature = "decode_read_file_data")]
            OpCode::ReadFileData => {
                Self::ReadFileData(ReadFileDataRef::start_decoding_unchecked(data))
            }
            #[cfg(feature = "decode_read_file_properties")]
            OpCode::ReadFileProperties => {
                Self::ReadFileProperties(ReadFilePropertiesRef::start_decoding_unchecked(data))
            }
            #[cfg(feature = "decode_write_file_data")]
            OpCode::WriteFileData => {
                Self::WriteFileData(WriteFileDataRef::start_decoding_unchecked(data))
            }
            #[cfg(feature = "decode_action_query")]
            OpCode::ActionQuery => {
                if data.len() < 2 {
                    return Err(ActionDecodeError::MissingBytes(2));
                }
                Self::ActionQuery(DecodedActionQueryRef::start_decoding_unchecked(data)?)
            }
            #[cfg(feature = "decode_status")]
            OpCode::Status => Self::Status(StatusRef::start_decoding_unchecked(data)?),
            _ => return Err(ActionDecodeError::UnknownActionCode(code)),
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
            #[cfg(feature = "decode_nop")]
            OpCode::Nop => Self::Nop(NopRef::start_decoding_ptr(data)),
            #[cfg(feature = "decode_read_file_data")]
            OpCode::ReadFileData => Self::ReadFileData(ReadFileDataRef::start_decoding_ptr(data)),
            #[cfg(feature = "decode_read_file_properties")]
            OpCode::ReadFileProperties => {
                Self::ReadFileProperties(ReadFilePropertiesRef::start_decoding_ptr(data))
            }
            #[cfg(feature = "decode_write_file_data")]
            OpCode::WriteFileData => {
                Self::WriteFileData(WriteFileDataRef::start_decoding_ptr(data))
            }
            #[cfg(feature = "decode_action_query")]
            OpCode::ActionQuery => {
                Self::ActionQuery(DecodedActionQueryRef::start_decoding_ptr(data)?)
            }
            #[cfg(feature = "decode_status")]
            OpCode::Status => Self::Status(StatusRef::start_decoding_ptr(data)?),
            _ => return Err(PtrActionDecodeError::UnknownActionCode(code)),
        })
    }

    /// Decodes the size of the Item in bytes
    ///
    /// # Safety
    /// This requires reading the data bytes that may be out of bound to be calculate.
    pub unsafe fn expected_size(&self) -> usize {
        match self {
            #[cfg(feature = "decode_nop")]
            Self::Nop(action) => action.expected_size(),
            #[cfg(feature = "decode_read_file_data")]
            Self::ReadFileData(action) => action.expected_size(),
            #[cfg(feature = "decode_read_file_properties")]
            Self::ReadFileProperties(action) => action.expected_size(),
            #[cfg(feature = "decode_write_file_data")]
            Self::WriteFileData(action) => action.expected_size(),
            #[cfg(feature = "decode_action_query")]
            Self::ActionQuery(action) => action.expected_size(),
            #[cfg(feature = "decode_status")]
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
            #[cfg(feature = "decode_nop")]
            Self::Nop(action) => action.smaller_than(data_size),
            #[cfg(feature = "decode_read_file_data")]
            Self::ReadFileData(action) => action.smaller_than(data_size),
            #[cfg(feature = "decode_read_file_properties")]
            Self::ReadFileProperties(action) => action.smaller_than(data_size),
            #[cfg(feature = "decode_write_file_data")]
            Self::WriteFileData(action) => action.smaller_than(data_size),
            #[cfg(feature = "decode_action_query")]
            Self::ActionQuery(action) => action.smaller_than(data_size),
            #[cfg(feature = "decode_status")]
            Self::Status(action) => action.smaller_than(data_size),
        }
    }

    /// Fully decode the Item
    ///
    /// Returns the decoded data and the number of bytes consumed to produce it.
    pub fn complete_decoding(&self) -> (DecodedActionRef<'data>, usize) {
        match self {
            #[cfg(feature = "decode_nop")]
            Self::Nop(action) => {
                let (action, size) = action.complete_decoding();
                (DecodedActionRef::Nop(action), size)
            }
            #[cfg(feature = "decode_read_file_data")]
            Self::ReadFileData(action) => {
                let (action, size) = action.complete_decoding();
                (DecodedActionRef::ReadFileData(action), size)
            }
            #[cfg(feature = "decode_read_file_properties")]
            Self::ReadFileProperties(action) => {
                let (action, size) = action.complete_decoding();
                (DecodedActionRef::ReadFileProperties(action), size)
            }
            #[cfg(feature = "decode_write_file_data")]
            Self::WriteFileData(action) => {
                let (action, size) = action.complete_decoding();
                (DecodedActionRef::WriteFileData(action), size)
            }
            #[cfg(feature = "decode_action_query")]
            Self::ActionQuery(action) => {
                let (action, size) = action.complete_decoding();
                (DecodedActionRef::ActionQuery(action.into()), size)
            }
            #[cfg(feature = "decode_status")]
            Self::Status(action) => {
                let (action, size) = action.complete_decoding();
                (DecodedActionRef::Status(action), size)
            }
        }
    }
}

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
            let (ret, size) = DecodedActionRef::decode(data).unwrap();
            assert_eq!(size, data.len());
            assert_eq!(ret.as_action(), op);

            // Test partial_decode == op
            let (decoder, expected_size) = DecodedActionRef::start_decoding(data).unwrap();
            assert_eq!(expected_size, size);
            assert_eq!(unsafe { decoder.expected_size() }, size);
            assert_eq!(decoder.smaller_than(data.len()).unwrap(), size);
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
        let (ret, size_decoded) = DecodedActionRef::decode(&encoded).unwrap();
        assert_eq!(size_encoded, size_decoded);
        assert_eq!(ret.as_action(), op);

        // Test decode(data).encode_in() == data
        let mut encoded2 = [0_u8; TOT_SIZE];
        let size_encoded2 = op.encode_in(&mut encoded2).unwrap();
        assert_eq!(size_encoded, size_encoded2);
        assert_eq!(encoded2[..size_encoded2], encoded[..size_encoded]);
    }
}

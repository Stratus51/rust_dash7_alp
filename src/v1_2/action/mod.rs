pub mod action_query;
pub mod break_query;
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

pub mod error;
pub mod flag;
pub mod op_code;

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
#[derive(Clone, Debug, PartialEq)]
pub enum Action<'a> {
    // Nop
    Nop(nop::Nop),
    // Read
    ReadFileData(read_file_data::ReadFileData),
    // ReadFileProperties(ReadFileProperties),

    // Write
    WriteFileData(write_file_data::WriteFileData<'a>),
    // WriteFileProperties(WriteFileProperties),
    // ActionQuery(ActionQuery),
    // BreakQuery(BreakQuery),
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
    // Status(Status),
    // ResponseTag(ResponseTag),

    // // Special
    // Chunk(Chunk),
    // Logic(Logic),
    // Forward(Forward),
    // IndirectForward(IndirectForward),
    // RequestTag(RequestTag),

    // // TODO
    // Extension(Extension),
}

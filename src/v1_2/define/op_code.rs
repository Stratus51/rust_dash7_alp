use crate::v1_2::error::define::OpCodeError;

// Nop
pub const NOP: u8 = 0;

// Read
pub const READ_FILE_DATA: u8 = 1;
pub const READ_FILE_PROPERTIES: u8 = 2;

// Write
pub const WRITE_FILE_DATA: u8 = 4;
pub const WRITE_FILE_PROPERTIES: u8 = 6;
pub const ACTION_QUERY: u8 = 8;
pub const BREAK_QUERY: u8 = 9;
pub const PERMISSION_REQUEST: u8 = 10;
pub const VERIFY_CHECKSUM: u8 = 11;

// Management
pub const EXIST_FILE: u8 = 16;
pub const CREATE_NEW_FILE: u8 = 17;
pub const DELETE_FILE: u8 = 18;
pub const RESTORE_FILE: u8 = 19;
pub const FLUSH_FILE: u8 = 20;
pub const COPY_FILE: u8 = 23;
pub const EXECUTE_FILE: u8 = 31;

// Response
pub const RETURN_FILE_DATA: u8 = 32;
pub const RETURN_FILE_PROPERTIES: u8 = 33;
pub const STATUS: u8 = 34;
pub const RESPONSE_TAG: u8 = 35;

// Special
pub const CHUNK: u8 = 48;
pub const LOGIC: u8 = 49;
pub const FORWARD: u8 = 50;
pub const INDIRECT_FORWARD: u8 = 51;
pub const REQUEST_TAG: u8 = 52;
pub const EXTENSION: u8 = 63;

#[repr(u8)]
#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum OpCode {
    // Nop
    #[cfg(any(feature = "nop", feature = "decode_nop"))]
    Nop = NOP,

    // Read
    #[cfg(any(feature = "read_file_data", feature = "decode_read_file_data"))]
    ReadFileData = READ_FILE_DATA,
    #[cfg(any(
        feature = "read_file_properties",
        feature = "decode_read_file_properties"
    ))]
    ReadFileProperties = READ_FILE_PROPERTIES,

    // Write
    #[cfg(any(feature = "write_file_data", feature = "decode_write_file_data"))]
    WriteFileData = WRITE_FILE_DATA,
    // WriteFileProperties = WRITE_FILE_PROPERTIES,
    #[cfg(any(feature = "action_query", feature = "decode_action_query"))]
    ActionQuery = ACTION_QUERY,
    // BreakQuery = BREAK_QUERY,
    // PermissionRequest = PERMISSION_REQUEST,
    // VerifyChecksum = VERIFY_CHECKSUM,

    // // Management
    // ExistFile = EXIST_FILE,
    // CreateNewFile = CREATE_NEW_FILE,
    // DeleteFile = DELETE_FILE,
    // RestoreFile = RESTORE_FILE,
    // FlushFile = FLUSH_FILE,
    // CopyFile = COPY_FILE,
    // ExecuteFile = EXECUTE_FILE,

    // // Response
    // ReturnFileData = RETURN_FILE_DATA,
    // ReturnFileProperties = RETURN_FILE_PROPERTIES,
    #[cfg(any(feature = "status", feature = "decode_status"))]
    Status = STATUS,
    // ResponseTag = RESPONSE_TAG,

    // // Special
    // Chunk = CHUNK,
    // Logic = LOGIC,
    // Forward = FORWARD,
    // IndirectForward = INDIRECT_FORWARD,
    // RequestTag = REQUEST_TAG,
    Extension = EXTENSION,
}

impl OpCode {
    #[cfg(any(feature = "nop", feature = "decode_nop"))]
    /// # Safety
    /// You are to warrant that n is encoded on 6 bits only.
    /// That means n <= 0x3F.
    pub const unsafe fn from_unchecked(n: u8) -> Self {
        match n {
            #[cfg(any(feature = "nop", feature = "decode_nop"))]
            NOP => Self::Nop,
            #[cfg(any(feature = "read_file_data", feature = "decode_read_file_data"))]
            READ_FILE_DATA => Self::ReadFileData,
            #[cfg(any(
                feature = "read_file_properties",
                feature = "decode_read_file_properties"
            ))]
            READ_FILE_PROPERTIES => Self::ReadFileProperties,
            #[cfg(any(feature = "write_file_data", feature = "decode_write_file_data"))]
            WRITE_FILE_DATA => Self::WriteFileData,
            // WRITE_FILE_PROPERTIES => Self::WriteFileProperties,
            #[cfg(any(feature = "action_query", feature = "decode_action_query"))]
            ACTION_QUERY => Self::ActionQuery,
            // BREAK_QUERY => Self::BreakQuery,
            // PERMISSION_REQUEST => Self::PermissionRequest,
            // VERIFY_CHECKSUM => Self::VerifyChecksum,
            // EXIST_FILE => Self::ExistFile,
            // CREATE_NEW_FILE => Self::CreateNewFile,
            // DELETE_FILE => Self::DeleteFile,
            // RESTORE_FILE => Self::RestoreFile,
            // FLUSH_FILE => Self::FlushFile,
            // COPY_FILE => Self::CopyFile,
            // EXECUTE_FILE => Self::ExecuteFile,
            // RETURN_FILE_DATA => Self::ReturnFileData,
            // RETURN_FILE_PROPERTIES => Self::ReturnFileProperties,
            #[cfg(any(feature = "status", feature = "decode_status"))]
            STATUS => Self::Status,
            // RESPONSE_TAG => Self::ResponseTag,
            // CHUNK => Self::Chunk,
            // LOGIC => Self::Logic,
            // FORWARD => Self::Forward,
            // INDIRECT_FORWARD => Self::IndirectForward,
            // REQUEST_TAG => Self::RequestTag,
            // EXTENSION => Self::Extension,
            // Should never occured if used safely
            _ => Self::Nop,
        }
    }

    /// # Errors
    /// Returns an error if n > 7
    pub const fn from(n: u8) -> Result<Self, OpCodeError> {
        Ok(match n {
            #[cfg(any(feature = "nop", feature = "decode_nop"))]
            NOP => Self::Nop,
            #[cfg(any(feature = "read_file_data", feature = "decode_read_file_data"))]
            READ_FILE_DATA => Self::ReadFileData,
            #[cfg(any(
                feature = "read_file_properties",
                feature = "decode_read_file_properties"
            ))]
            READ_FILE_PROPERTIES => Self::ReadFileProperties,
            #[cfg(any(feature = "write_file_data", feature = "decode_write_file_data"))]
            WRITE_FILE_DATA => Self::WriteFileData,
            // WRITE_FILE_PROPERTIES => Self::WriteFileProperties,
            #[cfg(any(feature = "action_query", feature = "decode_action_query"))]
            ACTION_QUERY => Self::ActionQuery,
            // BREAK_QUERY => Self::BreakQuery,
            // PERMISSION_REQUEST => Self::PermissionRequest,
            // VERIFY_CHECKSUM => Self::VerifyChecksum,
            // EXIST_FILE => Self::ExistFile,
            // CREATE_NEW_FILE => Self::CreateNewFile,
            // DELETE_FILE => Self::DeleteFile,
            // RESTORE_FILE => Self::RestoreFile,
            // FLUSH_FILE => Self::FlushFile,
            // COPY_FILE => Self::CopyFile,
            // EXECUTE_FILE => Self::ExecuteFile,
            // RETURN_FILE_DATA => Self::ReturnFileData,
            // RETURN_FILE_PROPERTIES => Self::ReturnFileProperties,
            #[cfg(any(feature = "status", feature = "decode_status"))]
            STATUS => Self::Status,
            // RESPONSE_TAG => Self::ResponseTag,
            // CHUNK => Self::Chunk,
            // LOGIC => Self::Logic,
            // FORWARD => Self::Forward,
            // INDIRECT_FORWARD => Self::IndirectForward,
            // REQUEST_TAG => Self::RequestTag,
            // EXTENSION => Self::Extension,
            n if n <= 0x3F => return Err(OpCodeError::Unsupported { code: n }),
            _ => return Err(OpCodeError::Invalid),
        })
    }
}

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum OpCode {
    // // Nop
    Nop = 0,

    // // Read
    ReadFileData = 1,
    ReadFileProperties = 2,

    // // Write
    WriteFileData = 4,
    // WriteFileProperties = 6,
    ActionQuery = 8,
    // BreakQuery = 9,
    // PermissionRequest = 10,
    // VerifyChecksum = 11,

    // // Management
    // ExistFile = 16,
    // CreateNewFile = 17,
    // DeleteFile = 18,
    // RestoreFile = 19,
    // FlushFile = 20,
    // CopyFile = 23,
    // ExecuteFile = 31,

    // // Response
    // ReturnFileData = 32,
    // ReturnFileProperties = 33,
    Status = 34,
    // ResponseTag = 35,

    // // Special
    // Chunk = 48,
    // Logic = 49,
    // Forward = 50,
    // IndirectForward = 51,
    // RequestTag = 52,
    Extension = 63,
}

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum OpCodeError {
    Unsupported { code: u8 },
    Invalid,
}

impl OpCode {
    /// # Safety
    /// You are to warrant that n is encoded on 6 bits only.
    /// That means n <= 0x3F.
    pub const unsafe fn from_unchecked(n: u8) -> Self {
        match n {
            0 => Self::Nop,
            1 => Self::ReadFileData,
            2 => Self::ReadFileProperties,
            4 => Self::WriteFileData,
            // 6 => Self::WriteFileProperties,
            8 => Self::ActionQuery,
            // 9 => Self::BreakQuery,
            // 10 => Self::PermissionRequest,
            // 11 => Self::VerifyChecksum,
            // 16 => Self::ExistFile,
            // 17 => Self::CreateNewFile,
            // 18 => Self::DeleteFile,
            // 19 => Self::RestoreFile,
            // 20 => Self::FlushFile,
            // 23 => Self::CopyFile,
            // 31 => Self::ExecuteFile,
            // 32 => Self::ReturnFileData,
            // 33 => Self::ReturnFileProperties,
            34 => Self::Status,
            // 35 => Self::ResponseTag,
            // 48 => Self::Chunk,
            // 49 => Self::Logic,
            // 50 => Self::Forward,
            // 51 => Self::IndirectForward,
            // 52 => Self::RequestTag,
            // 63 => Self::Extension,
            // Should never occured if used safely
            _ => Self::Nop,
        }
    }

    /// # Errors
    /// Returns an error if n > 7
    pub const fn from(n: u8) -> Result<Self, OpCodeError> {
        Ok(match n {
            0 => Self::Nop,
            1 => Self::ReadFileData,
            2 => Self::ReadFileProperties,
            4 => Self::WriteFileData,
            // 6 => Self::WriteFileProperties,
            8 => Self::ActionQuery,
            // 9 => Self::BreakQuery,
            // 10 => Self::PermissionRequest,
            // 11 => Self::VerifyChecksum,
            // 16 => Self::ExistFile,
            // 17 => Self::CreateNewFile,
            // 18 => Self::DeleteFile,
            // 19 => Self::RestoreFile,
            // 20 => Self::FlushFile,
            // 23 => Self::CopyFile,
            // 31 => Self::ExecuteFile,
            // 32 => Self::ReturnFileData,
            // 33 => Self::ReturnFileProperties,
            34 => Self::Status,
            // 35 => Self::ResponseTag,
            // 48 => Self::Chunk,
            // 49 => Self::Logic,
            // 50 => Self::Forward,
            // 51 => Self::IndirectForward,
            // 52 => Self::RequestTag,
            // 63 => Self::Extension,
            n if n <= 0x3F => return Err(OpCodeError::Unsupported { code: n }),
            _ => return Err(OpCodeError::Invalid),
        })
    }
}

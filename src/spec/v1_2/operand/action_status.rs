use crate::codec::{Codec, WithOffset, WithSize};
#[cfg(test)]
use crate::test_tools::test_item;
#[cfg(test)]
use hex_literal::hex;
use std::convert::TryInto;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StatusCode {
    Received = 1,
    Ok = 0,
    FileIdMissing = 0xff,
    CreateFileIdAlreadyExist = 0xfe,
    FileIsNotRestorable = 0xfd,
    InsufficientPermission = 0xfc,
    CreateFileLengthOverflow = 0xfb,
    CreateFileAllocationOverflow = 0xfa,
    WriteOffsetOverflow = 0xf9,
    WriteDataOverflow = 0xf8,
    WriteStorageUnavailable = 0xf7,
    UnknownOperation = 0xf6,
    OperandIncomplete = 0xf5,
    OperandWrongFormat = 0xf4,
    UnknownError = 0x80,
}
impl std::convert::TryFrom<u8> for StatusCode {
    type Error = u8;
    fn try_from(n: u8) -> Result<Self, Self::Error> {
        Ok(match n {
            1 => Self::Received,
            0 => Self::Ok,
            0xff => Self::FileIdMissing,
            0xfe => Self::CreateFileIdAlreadyExist,
            0xfd => Self::FileIsNotRestorable,
            0xfc => Self::InsufficientPermission,
            0xfb => Self::CreateFileLengthOverflow,
            0xfa => Self::CreateFileAllocationOverflow,
            0xf9 => Self::WriteOffsetOverflow,
            0xf8 => Self::WriteDataOverflow,
            0xf7 => Self::WriteStorageUnavailable,
            0xf6 => Self::UnknownOperation,
            0xf5 => Self::OperandIncomplete,
            0xf4 => Self::OperandWrongFormat,
            0x80 => Self::UnknownError,
            x => return Err(x),
        })
    }
}
impl std::fmt::Display for StatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Received => "RCV",
                Self::Ok => "OK",
                Self::FileIdMissing => "E_FID",
                Self::CreateFileIdAlreadyExist => "E_F_EXIST",
                Self::FileIsNotRestorable => "E_F_RST",
                Self::InsufficientPermission => "E_PRM",
                Self::CreateFileLengthOverflow => "E_NEW_LEN",
                Self::CreateFileAllocationOverflow => "E_NEW_ALLOC",
                Self::WriteOffsetOverflow => "E_W_OFF",
                Self::WriteDataOverflow => "E_W_DATA",
                Self::WriteStorageUnavailable => "E_W_STOR",
                Self::UnknownOperation => "E_UNK_OP",
                Self::OperandIncomplete => "E_INC",
                Self::OperandWrongFormat => "E_FMT",
                Self::UnknownError => "E_?",
            }
        )
    }
}

impl StatusCode {
    pub fn is_err(&self) -> bool {
        *self as u8 >= 0x80
    }
}

/// Result of an action in a previously sent request
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ActionStatus {
    /// Index of the ALP action associated with this status, in the original request as seen from
    /// the receiver side.
    // ALP_SPEC This is complicated to process because we have to known/possibly infer the position
    // of the action on the receiver side, and that we have to do that while also interpreting who
    // responded (the local modem won't have the same index as the distant device.).
    pub action_id: u8,
    /// Result code
    pub status: StatusCode,
}
impl std::fmt::Display for ActionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "a[{}]=>{}", self.action_id, self.status)
    }
}
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ActionStatusDecodingError {
    MissingBytes(usize),
    UnknownStatusCode(u8),
}
impl Codec for ActionStatus {
    type Error = ActionStatusDecodingError;
    fn encoded_size(&self) -> usize {
        1 + 1
    }
    unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
        out[0] = self.action_id;
        out[1] = self.status as u8;
        2
    }
    fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
        if out.len() < 2 {
            return Err(WithOffset::new_head(Self::Error::MissingBytes(
                2 - out.len(),
            )));
        }
        Ok(WithSize {
            value: Self {
                action_id: out[0],
                status: out[1]
                    .try_into()
                    .map_err(|e| WithOffset::new(1, Self::Error::UnknownStatusCode(e)))?,
            },
            size: 2,
        })
    }
}
#[test]
fn test_status_operand() {
    test_item(
        ActionStatus {
            action_id: 2,
            status: StatusCode::UnknownOperation,
        },
        &hex!("02 F6"),
    )
}

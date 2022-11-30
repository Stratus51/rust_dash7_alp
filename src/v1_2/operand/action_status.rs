use crate::codec::{Codec, StdError, WithOffset, WithSize};
#[cfg(test)]
use crate::test_tools::test_item;
#[cfg(test)]
use hex_literal::hex;

pub mod status {
    //! Status code that can be received as a result of some ALP actions.
    /// Action received and partially completed at response. To be completed after response
    pub const RECEIVED: u8 = 1;
    pub const OK: u8 = 0;
    pub const FILE_ID_MISSING: u8 = 0xFF;
    pub const CREATE_FILE_ID_ALREADY_EXIST: u8 = 0xFE;
    pub const FILE_IS_NOT_RESTORABLE: u8 = 0xFD;
    pub const INSUFFICIENT_PERMISSION: u8 = 0xFC;
    pub const CREATE_FILE_LENGTH_OVERFLOW: u8 = 0xFB;
    pub const CREATE_FILE_ALLOCATION_OVERFLOW: u8 = 0xFA; // ALP_SPEC: ??? Difference with the previous one?;
    pub const WRITE_OFFSET_OVERFLOW: u8 = 0xF9;
    pub const WRITE_DATA_OVERFLOW: u8 = 0xF8;
    pub const WRITE_STORAGE_UNAVAILABLE: u8 = 0xF7;
    pub const UNKNOWN_OPERATION: u8 = 0xF6;
    pub const OPERAND_INCOMPLETE: u8 = 0xF5;
    pub const OPERAND_WRONG_FORMAT: u8 = 0xF4;
    pub const UNKNOWN_ERROR: u8 = 0x80;
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
    pub status: u8,
}
impl std::fmt::Display for ActionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "a[{}]=>{}", self.action_id, self.status)
    }
}
impl Codec for ActionStatus {
    type Error = StdError;
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
                status: out[1],
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
            status: status::UNKNOWN_OPERATION,
        },
        &hex!("02 F6"),
    )
}

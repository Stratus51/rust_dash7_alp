#[cfg(test)]
use crate::test_tools::test_item;
use crate::{
    codec::{Codec, WithOffset, WithSize},
    spec::v1_2 as spec,
    wizzilab::v5_3::dash7::stack_error::InterfaceFinalStatusCode,
};
#[cfg(test)]
use hex_literal::hex;
use std::convert::TryInto;

/// Result of an action in a previously sent request
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InterfaceFinalStatus {
    /// Index of the ALP action associated with this status, in the original request as seen from
    /// the receiver side.
    // ALP_SPEC This is complicated to process because we have to known/possibly infer the position
    // of the action on the receiver side, and that we have to do that while also interpreting who
    // responded (the local modem won't have the same index as the distant device.).
    pub interface: spec::operand::InterfaceId,
    /// Length
    // TODO What is the encoding of this field? Is is a varint?
    pub len: u8,
    /// Result code
    pub status: InterfaceFinalStatusCode,
}
impl std::fmt::Display for InterfaceFinalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "f_itf[{}][{}]=>{}",
            self.interface, self.len, self.status
        )
    }
}
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum InterfaceFinalStatusDecodingError {
    MissingBytes(usize),
    UnknownStatusCode(u8),
    UnknownInterface(u8),
}
impl Codec for InterfaceFinalStatus {
    type Error = InterfaceFinalStatusDecodingError;
    fn encoded_size(&self) -> usize {
        1 + 1 + 1
    }
    unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
        out[0] = self.interface as u8;
        out[1] = self.len;
        out[2] = self.status as u8;
        3
    }
    fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
        if out.len() < 3 {
            return Err(WithOffset::new_head(Self::Error::MissingBytes(
                3 - out.len(),
            )));
        }
        Ok(WithSize {
            value: Self {
                interface: out[0]
                    .try_into()
                    .map_err(|e| WithOffset::new(0, Self::Error::UnknownInterface(e)))?,
                len: out[1],
                status: out[2]
                    .try_into()
                    .map_err(|e| WithOffset::new(2, Self::Error::UnknownStatusCode(e)))?,
            },
            size: 3,
        })
    }
}
#[test]
fn test_interface_final_status_operand() {
    test_item(
        InterfaceFinalStatus {
            interface: spec::operand::InterfaceId::Host,
            len: 2,
            status: InterfaceFinalStatusCode::Busy,
        },
        &hex!("00 02 FF"),
    )
}

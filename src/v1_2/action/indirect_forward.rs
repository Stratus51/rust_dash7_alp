#[cfg(test)]
use crate::{test_tools::test_item, v1_2::dash7};
#[cfg(test)]
use hex_literal::hex;

use crate::{
    codec::{Codec, StdError, WithOffset, WithSize},
    v1_2::operand,
};

#[derive(Clone, Debug, PartialEq)]
pub struct IndirectForward {
    // ALP_SPEC Ask for response ?
    pub resp: bool,
    pub interface: operand::indirect_interface::IndirectInterface,
}
impl std::fmt::Display for IndirectForward {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "[{}]{}",
            if self.resp { "R" } else { "-" },
            self.interface
        )
    }
}
impl Codec for IndirectForward {
    type Error = StdError;
    fn encoded_size(&self) -> usize {
        1 + self.interface.encoded_size()
    }
    unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
        let overload = match self.interface {
            operand::indirect_interface::IndirectInterface::Overloaded(_) => true,
            operand::indirect_interface::IndirectInterface::NonOverloaded(_) => false,
        };
        out[0] = super::control_byte!(overload, self.resp, super::OpCode::IndirectForward);
        1 + super::serialize_all!(&mut out[1..], &self.interface)
    }
    fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
        if out.is_empty() {
            Err(WithOffset::new_head(Self::Error::MissingBytes(1)))
        } else {
            let mut offset = 0;
            let WithSize {
                value: op1,
                size: op1_size,
            } = operand::indirect_interface::IndirectInterface::decode(out)?;
            offset += op1_size;
            Ok(WithSize {
                value: Self {
                    resp: out[0] & 0x40 != 0,
                    interface: op1,
                },
                size: offset,
            })
        }
    }
}
#[test]
fn test_indirect_forward() {
    test_item(
        IndirectForward {
            resp: true,
            interface: operand::indirect_interface::IndirectInterface::Overloaded(
                operand::indirect_interface::OverloadedIndirectInterface {
                    interface_file_id: 4,
                    nls_method: dash7::NlsMethod::AesCcm32,
                    access_class: 0xFF,
                    address: dash7::Address::Vid([0xAB, 0xCD]),
                },
            ),
        },
        &hex!("F3   04   37 FF ABCD"),
    )
}

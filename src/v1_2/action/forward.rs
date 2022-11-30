#[cfg(test)]
use crate::test_tools::test_item;
#[cfg(test)]
use hex_literal::hex;

use crate::{
    codec::{Codec, WithOffset, WithSize},
    v1_2::operand,
};

#[derive(Clone, Debug, PartialEq)]
pub struct Forward {
    // ALP_SPEC Ask for response ?
    pub resp: bool,
    pub conf: operand::interface_configuration::InterfaceConfiguration,
}
impl std::fmt::Display for Forward {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}{}", if self.resp { "[R]" } else { "-" }, self.conf)
    }
}
impl Codec for Forward {
    type Error = operand::interface_configuration::InterfaceConfigurationDecodingError;
    fn encoded_size(&self) -> usize {
        1 + self.conf.encoded_size()
    }
    unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
        out[0] = super::control_byte!(false, self.resp, crate::v1_2::action::OpCode::Forward);
        1 + self.conf.encode_in(&mut out[1..])
    }
    fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
        let min_size = 1 + 1;
        if out.len() < min_size {
            return Err(WithOffset::new(
                0,
                Self::Error::MissingBytes(min_size - out.len()),
            ));
        }
        let WithSize {
            value: conf,
            size: conf_size,
        } = operand::interface_configuration::InterfaceConfiguration::decode(&out[1..])
            .map_err(|e| e.shift(1))?;
        Ok(WithSize {
            value: Self {
                resp: out[0] & 0x40 != 0,
                conf,
            },
            size: 1 + conf_size,
        })
    }
}
#[test]
fn test_forward() {
    test_item(
        Forward {
            resp: true,
            conf: operand::interface_configuration::InterfaceConfiguration::Host,
        },
        &hex!("72 00"),
    )
}

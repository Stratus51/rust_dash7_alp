pub use crate::spec::v1_2::operand::interface_configuration::InterfaceConfigurationDecodingError;
#[cfg(test)]
use crate::test_tools::test_item;
use crate::{
    codec::{Codec, WithOffset, WithSize},
    spec::v1_2 as spec,
    wizzilab::v5_3::dash7,
};
#[cfg(test)]
use hex_literal::hex;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InterfaceId {
    Host = 0,
    D7asp = 0xD7,
}

/// Meta data required to send a packet depending on the sending interface type
#[derive(Clone, Debug, PartialEq)]
pub enum InterfaceConfiguration {
    Host,
    D7asp(dash7::InterfaceConfiguration),
}
impl std::fmt::Display for InterfaceConfiguration {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Host => write!(f, "HOST"),
            Self::D7asp(conf) => write!(f, "D7:{}", conf),
        }
    }
}
impl Codec for InterfaceConfiguration {
    type Error = InterfaceConfigurationDecodingError;
    fn encoded_size(&self) -> usize {
        1 + match self {
            InterfaceConfiguration::Host => 0,
            InterfaceConfiguration::D7asp(v) => v.encoded_size(),
        }
    }
    unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
        match self {
            InterfaceConfiguration::Host => {
                out[0] = InterfaceId::Host as u8;
                1
            }
            InterfaceConfiguration::D7asp(v) => {
                out[0] = InterfaceId::D7asp as u8;
                1 + v.encode_in(&mut out[1..])
            }
        }
    }
    fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
        if out.is_empty() {
            return Err(WithOffset::new_head(Self::Error::MissingBytes(1)));
        }
        const HOST: u8 = InterfaceId::Host as u8;
        const D7ASP: u8 = InterfaceId::D7asp as u8;
        Ok(match out[0] {
            HOST => WithSize {
                value: InterfaceConfiguration::Host,
                size: 1,
            },
            D7ASP => {
                let WithSize { value, size } = dash7::InterfaceConfiguration::decode(&out[1..])
                    .map_err(|e| e.map_value(InterfaceConfigurationDecodingError::D7asp))?;
                WithSize {
                    value: InterfaceConfiguration::D7asp(value),
                    size: size + 1,
                }
            }
            id => {
                return Err(WithOffset {
                    value: Self::Error::BadInterfaceId(id),
                    offset: 0,
                })
            }
        })
    }
}
#[test]
fn test_interface_configuration_d7asp() {
    test_item(
        InterfaceConfiguration::D7asp(dash7::InterfaceConfiguration {
            qos: dash7::Qos {
                retry: dash7::RetryMode::Oneshot,
                resp: dash7::RespMode::Any,
            },
            to: 0x23,
            te: 0x34,
            nls_method: dash7::NlsMethod::AesCcm32,
            access_class: 0xFF,
            address: dash7::Address::Vid([0xAB, 0xCD]),
            use_vid: false,
        }),
        &hex!("D7   02 23 34   37 FF ABCD"),
    )
}
#[test]
fn test_interface_configuration_host() {
    test_item(InterfaceConfiguration::Host, &hex!("00"))
}
impl From<spec::operand::InterfaceConfiguration> for InterfaceConfiguration {
    fn from(o: spec::operand::InterfaceConfiguration) -> Self {
        match o {
            spec::operand::InterfaceConfiguration::Host => Self::Host,
            spec::operand::InterfaceConfiguration::D7asp(conf) => Self::D7asp(conf.into()),
        }
    }
}
impl From<InterfaceConfiguration> for spec::operand::InterfaceConfiguration {
    fn from(o: InterfaceConfiguration) -> Self {
        match o {
            InterfaceConfiguration::Host => Self::Host,
            InterfaceConfiguration::D7asp(conf) => Self::D7asp(conf.into()),
        }
    }
}

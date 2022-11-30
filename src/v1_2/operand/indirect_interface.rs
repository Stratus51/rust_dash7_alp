#[cfg(test)]
use crate::test_tools::test_item;
use crate::{
    codec::{Codec, StdError, WithOffset, WithSize},
    v1_2::dash7,
};
#[cfg(test)]
use hex_literal::hex;

/// Dash7 interface
#[derive(Clone, Debug, PartialEq)]
pub struct OverloadedIndirectInterface {
    /// File containing the `QoS`, `to` and `te` to use for the transmission (see
    /// dash7::InterfaceConfiguration
    pub interface_file_id: u8,
    pub nls_method: dash7::NlsMethod,
    pub access_class: u8,
    pub address: dash7::Address,
}
impl std::fmt::Display for OverloadedIndirectInterface {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{},{},{},{}",
            self.interface_file_id, self.nls_method, self.access_class, self.address
        )
    }
}

impl Codec for OverloadedIndirectInterface {
    type Error = StdError;
    fn encoded_size(&self) -> usize {
        1 + 2 + self.address.encoded_size()
    }
    unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
        out[0] = self.interface_file_id;
        out[1] = ((self.address.id_type() as u8) << 4) | (self.nls_method as u8);
        out[2] = self.access_class;
        1 + 2 + self.address.encode_in(&mut out[3..])
    }
    fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
        if out.len() < 1 + 2 {
            return Err(WithOffset::new_head(Self::Error::MissingBytes(
                1 + 2 - out.len(),
            )));
        }
        let interface_file_id = out[0];
        let address_type = dash7::AddressType::from((out[1] & 0x30) >> 4);
        let nls_method = unsafe { dash7::NlsMethod::from(out[1] & 0x0F) };
        let access_class = out[2];
        let WithSize {
            value: address,
            size: address_size,
        } = dash7::Address::parse(address_type, &out[3..]).map_err(|e| e.shift(3))?;
        Ok(WithSize {
            value: Self {
                interface_file_id,
                nls_method,
                access_class,
                address,
            },
            size: 1 + 2 + address_size,
        })
    }
}
#[test]
fn test_overloaded_indirect_interface() {
    test_item(
        OverloadedIndirectInterface {
            interface_file_id: 4,
            nls_method: dash7::NlsMethod::AesCcm32,
            access_class: 0xFF,
            address: dash7::Address::Vid([0xAB, 0xCD]),
        },
        &hex!("04   37 FF ABCD"),
    )
}

/// Non Dash7 interface
#[derive(Clone, Debug, PartialEq)]
// ALP SPEC: This seems undoable if we do not know the interface (per protocol specific support)
//  which is still a pretty legitimate policy on a low power protocol.
pub struct NonOverloadedIndirectInterface {
    pub interface_file_id: u8,
    // ALP SPEC: Where is this defined? Is this ID specific?
    pub data: Box<[u8]>,
}

impl Codec for NonOverloadedIndirectInterface {
    type Error = StdError;
    fn encoded_size(&self) -> usize {
        1 + self.data.len()
    }
    unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
        out[0] = self.interface_file_id;
        let mut offset = 1;
        out[offset..offset + self.data.len()].clone_from_slice(&self.data);
        offset += self.data.len();
        offset
    }
    fn decode(_out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
        todo!("TODO")
    }
}
impl std::fmt::Display for NonOverloadedIndirectInterface {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{},0x{}",
            self.interface_file_id,
            hex::encode_upper(&self.data)
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum IndirectInterface {
    Overloaded(OverloadedIndirectInterface),
    NonOverloaded(NonOverloadedIndirectInterface),
}
impl std::fmt::Display for IndirectInterface {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Overloaded(v) => write!(f, "O:{}", v),
            Self::NonOverloaded(v) => write!(f, "N:{}", v),
        }
    }
}

impl Codec for IndirectInterface {
    type Error = StdError;
    fn encoded_size(&self) -> usize {
        match self {
            IndirectInterface::Overloaded(v) => v.encoded_size(),
            IndirectInterface::NonOverloaded(v) => v.encoded_size(),
        }
    }
    unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
        match self {
            IndirectInterface::Overloaded(v) => v.encode_in(out),
            IndirectInterface::NonOverloaded(v) => v.encode_in(out),
        }
    }
    fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
        if out.is_empty() {
            return Err(WithOffset::new_head(Self::Error::MissingBytes(1)));
        }
        Ok(if out[0] & 0x80 != 0 {
            let WithSize { size, value } =
                OverloadedIndirectInterface::decode(&out[1..]).map_err(|e| e.shift(1))?;
            WithSize {
                size: size + 1,
                value: Self::Overloaded(value),
            }
        } else {
            let WithSize { size, value } =
                NonOverloadedIndirectInterface::decode(&out[1..]).map_err(|e| e.shift(1))?;
            WithSize {
                size: size + 1,
                value: Self::NonOverloaded(value),
            }
        })
    }
}

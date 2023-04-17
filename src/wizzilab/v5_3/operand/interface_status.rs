#[cfg(test)]
use crate::test_tools::test_item;
use crate::{
    codec::{Codec, StdError, WithOffset, WithSize},
    spec::v1_2 as spec,
    wizzilab::v5_3::{dash7, operand::InterfaceId, varint},
};
#[cfg(test)]
use hex_literal::hex;
pub use spec::operand::InterfaceStatusUnknown;

// TODO Allow padding at the end
// We should support the parsing and the encoding of this padding
/// Meta data from a received packet depending on the receiving interface type
#[derive(Clone, Debug, PartialEq)]
pub enum InterfaceStatus {
    Host,
    D7asp(dash7::InterfaceStatus),
    Unknown(InterfaceStatusUnknown),
}
impl std::fmt::Display for InterfaceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Host => write!(f, "HOST"),
            Self::D7asp(status) => write!(f, "D7={}", status),
            Self::Unknown(status) => write!(f, "?={}", status),
        }
    }
}
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum InterfaceStatusDecodingError {
    MissingBytes(usize),
    BadInterfaceId(u8),
}
impl From<StdError> for InterfaceStatusDecodingError {
    fn from(e: StdError) -> Self {
        match e {
            StdError::MissingBytes(n) => Self::MissingBytes(n),
        }
    }
}
impl Codec for InterfaceStatus {
    type Error = InterfaceStatusDecodingError;
    fn encoded_size(&self) -> usize {
        let data_size = match self {
            InterfaceStatus::Host => 0,
            InterfaceStatus::D7asp(itf) => itf.encoded_size(),
            InterfaceStatus::Unknown(InterfaceStatusUnknown { data, .. }) => data.len(),
        };
        1 + unsafe { varint::size(data_size as u32) } as usize + data_size
    }
    unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
        let mut offset = 1;
        match self {
            InterfaceStatus::Host => {
                out[0] = InterfaceId::Host as u8;
                out[1] = 0;
                offset += 1;
            }
            InterfaceStatus::D7asp(v) => {
                out[0] = InterfaceId::D7asp as u8;
                let size = v.encoded_size() as u32;
                let size_size = varint::encode_in(size, &mut out[offset..]);
                offset += size_size as usize;
                offset += v.encode_in(&mut out[offset..]);
            }
            InterfaceStatus::Unknown(InterfaceStatusUnknown { id, data, .. }) => {
                out[0] = *id;
                let size = data.len() as u32;
                let size_size = varint::encode_in(size, &mut out[offset..]);
                offset += size_size as usize;
                out[offset..offset + data.len()].clone_from_slice(data);
                offset += data.len();
            }
        };
        offset
    }
    fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
        if out.is_empty() {
            return Err(WithOffset::new_head(Self::Error::MissingBytes(1)));
        }
        const HOST: u8 = InterfaceId::Host as u8;
        const D7ASP: u8 = InterfaceId::D7asp as u8;
        let mut offset = 1;
        let value = match out[0] {
            HOST => {
                offset += 1;
                InterfaceStatus::Host
            }
            D7ASP => {
                let WithSize {
                    value: size,
                    size: size_size,
                } = varint::decode(&out[offset..]).map_err(|e| {
                    let WithOffset { offset: off, value } = e;
                    WithOffset {
                        offset: offset + off,
                        value: value.into(),
                    }
                })?;
                let announced_size = size as usize;
                offset += size_size;
                let WithSize { value, size } = dash7::InterfaceStatus::decode(
                    &out[offset..offset + announced_size],
                )
                .map_err(|e| {
                    let WithOffset { offset: off, value } = e;
                    WithOffset {
                        offset: offset + off,
                        value: value.into(),
                    }
                })?;
                offset += size.max(announced_size);
                InterfaceStatus::D7asp(value)
            }
            id => {
                let WithSize {
                    value: size,
                    size: size_size,
                } = varint::decode(&out[offset..]).map_err(|e| {
                    let WithOffset { offset: off, value } = e;
                    WithOffset {
                        offset: offset + off,
                        value: value.into(),
                    }
                })?;
                let size = size as usize;
                offset += size_size;
                if out.len() < offset + size {
                    return Err(WithOffset::new(
                        offset,
                        Self::Error::MissingBytes(offset + size - out.len()),
                    ));
                }
                let mut data = vec![0u8; size].into_boxed_slice();
                data.clone_from_slice(&out[offset..size]);
                offset += size;
                InterfaceStatus::Unknown(InterfaceStatusUnknown { id, data })
            }
        };
        Ok(WithSize {
            value,
            size: offset,
        })
    }
}
#[test]
fn test_interface_status_d7asp() {
    test_item(
        InterfaceStatus::D7asp(dash7::InterfaceStatus {
            ch_header: 1,
            ch_idx: 0x0123,
            rxlev: 2,
            lb: 3,
            snr: 4,
            status: 0xB0,
            token: 6,
            seq: 7,
            resp_to: 8,
            fof: 9,
            access_class: 0xFF,
            address: dash7::Address::Vid([0xAB, 0xCD]),
            nls_state: dash7::NlsState::AesCcm32(hex!("00 11 22 33 44")),
        }),
        &hex!("D7 1C    01 0123 02 03 04 B0 06 07 0800 0900   37 FF ABCD 000000000000  0011223344"),
    )
}
#[test]
fn test_interface_status_host() {
    test_item(InterfaceStatus::Host, &hex!("00 00"))
}

impl From<spec::operand::InterfaceStatus> for InterfaceStatus {
    fn from(itf: spec::operand::InterfaceStatus) -> Self {
        match itf {
            spec::operand::InterfaceStatus::Host => Self::Host,
            spec::operand::InterfaceStatus::D7asp(itf) => Self::D7asp(itf.into()),
            spec::operand::InterfaceStatus::Unknown(itf) => Self::Unknown(itf),
        }
    }
}

impl From<InterfaceStatus> for spec::operand::InterfaceStatus {
    fn from(itf: InterfaceStatus) -> Self {
        match itf {
            InterfaceStatus::Host => Self::Host,
            InterfaceStatus::D7asp(itf) => Self::D7asp(itf.into()),
            InterfaceStatus::Unknown(itf) => Self::Unknown(itf),
        }
    }
}

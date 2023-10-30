#[cfg(test)]
use crate::test_tools::test_item;
#[cfg(test)]
use crate::wizzilab::v5_3::dash7::{stack_error::InterfaceFinalStatusCode, Address, NlsMethod};
use crate::{
    codec::{Codec, StdError, WithOffset, WithSize},
    spec::v1_2 as spec,
    wizzilab::v5_3::{
        dash7::{self},
        operand::InterfaceId,
        varint,
    },
};
#[cfg(test)]
use hex_literal::hex;

// TODO Allow padding at the end
// We should support the parsing and the encoding of this padding
/// Meta data from a received packet depending on the receiving interface type
#[derive(Clone, Debug, PartialEq)]
pub enum InterfaceTxStatus {
    Host,
    D7asp(dash7::interface_tx_status::InterfaceTxStatus),
    Unknown(spec::operand::InterfaceStatusUnknown),
}
impl std::fmt::Display for InterfaceTxStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Host => write!(f, "HOST"),
            Self::D7asp(status) => write!(f, "D7={}", status),
            Self::Unknown(status) => write!(f, "?={}", status),
        }
    }
}
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum InterfaceTxStatusDecodingError {
    MissingBytes(usize),
    BadInterfaceId(u8),
    UnknownStatusCode(u8),
}
impl From<StdError> for InterfaceTxStatusDecodingError {
    fn from(e: StdError) -> Self {
        match e {
            StdError::MissingBytes(n) => Self::MissingBytes(n),
        }
    }
}
impl From<dash7::interface_tx_status::InterfaceTxStatusDecodingError>
    for InterfaceTxStatusDecodingError
{
    fn from(e: dash7::interface_tx_status::InterfaceTxStatusDecodingError) -> Self {
        match e {
            dash7::interface_tx_status::InterfaceTxStatusDecodingError::MissingBytes(n) => {
                Self::MissingBytes(n)
            }
            dash7::interface_tx_status::InterfaceTxStatusDecodingError::UnknownStatusCode(n) => {
                Self::UnknownStatusCode(n)
            }
        }
    }
}
impl Codec for InterfaceTxStatus {
    type Error = InterfaceTxStatusDecodingError;
    fn encoded_size(&self) -> usize {
        let data_size = match self {
            InterfaceTxStatus::Host => 0,
            InterfaceTxStatus::D7asp(itf) => itf.encoded_size(),
            InterfaceTxStatus::Unknown(spec::operand::InterfaceStatusUnknown { data, .. }) => {
                data.len()
            }
        };
        1 + unsafe { varint::size(data_size as u32) } as usize + data_size
    }
    unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
        let mut offset = 1;
        match self {
            InterfaceTxStatus::Host => {
                out[0] = InterfaceId::Host as u8;
                out[1] = 0;
                offset += 1;
            }
            InterfaceTxStatus::D7asp(v) => {
                out[0] = InterfaceId::D7asp as u8;
                let size = v.encoded_size() as u32;
                let size_size = varint::encode_in(size, &mut out[offset..]);
                offset += size_size as usize;
                offset += v.encode_in(&mut out[offset..]);
            }
            InterfaceTxStatus::Unknown(spec::operand::InterfaceStatusUnknown {
                id, data, ..
            }) => {
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
                InterfaceTxStatus::Host
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
                let WithSize { value, size } =
                    dash7::interface_tx_status::InterfaceTxStatus::decode(
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
                InterfaceTxStatus::D7asp(value)
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
                data.clone_from_slice(&out[offset..offset + size]);
                offset += size;
                InterfaceTxStatus::Unknown(spec::operand::InterfaceStatusUnknown { id, data })
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
        InterfaceTxStatus::D7asp(dash7::interface_tx_status::InterfaceTxStatus {
            ch_header: 1,
            ch_idx: 0x0123,
            eirp: 2,
            err: InterfaceFinalStatusCode::Busy,
            rfu_0: 4,
            rfu_1: 5,
            rfu_2: 6,
            lts: 0x0708_0000,
            access_class: 0xFF,
            nls_method: NlsMethod::AesCcm64,
            address: Address::Vid([0x00, 0x11]),
        }),
        &hex!("D7 16    01 0123 02 FF 04 05 06 0000 0807  36 FF 0011 000000000000"),
    )
}
#[test]
fn test_interface_status_host() {
    test_item(InterfaceTxStatus::Host, &hex!("00 00"))
}

#[test]
fn test_interface_status_unknown() {
    test_item(
        InterfaceTxStatus::Unknown(spec::operand::InterfaceStatusUnknown {
            id: 0x12,
            data: vec![0x34, 0x56, 0x78].into_boxed_slice(),
        }),
        &hex!("12 03 345678"),
    )
}

pub mod action;
pub mod define;
pub mod interface;

use crate::decodable::{FailableDecodable, FailableEncodedData, WithByteSize};
use crate::encodable::Encodable;
use crate::v1_2::define::op_code;
use crate::v1_2::error::{StatusDecodeError, StatusSizeError, UnsupportedExtension};
pub use define::StatusExtension;
pub use interface::{EncodedStatusInterface, StatusInterface, StatusInterfaceRef};

// TODO Add feature based sub types support (also in status_interface)

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum StatusRef<'item> {
    // Action(),
    Interface(StatusInterfaceRef<'item>),
}

impl<'data> Encodable for StatusRef<'data> {
    unsafe fn encode_in_ptr(&self, out: *mut u8) -> usize {
        *out.add(0) = op_code::STATUS | ((self.extension() as u8) << 6);
        1 + match self {
            Self::Interface(status) => status.encode_in_ptr(out.add(1)),
        }
    }

    fn encoded_size(&self) -> usize {
        1 + match self {
            Self::Interface(status) => status.encoded_size(),
        }
    }
}

impl<'item> StatusRef<'item> {
    pub fn extension(&self) -> StatusExtension {
        match self {
            Self::Interface(_) => StatusExtension::Interface,
        }
    }

    pub fn to_owned(&self) -> Status {
        match self {
            Self::Interface(status) => Status::Interface(status.to_owned()),
        }
    }
}

pub enum ValidEncodedStatus<'data> {
    Interface(EncodedStatusInterface<'data>),
}

impl<'data> EncodedStatus<'data> {
    /// # Safety
    /// This reads data without checking boundaries.
    /// If self.data.len() < self.encoded_size() then this is safe.
    ///
    /// # Errors
    /// Fails if the status extension is unsupported.
    pub unsafe fn extension(&self) -> Result<StatusExtension, UnsupportedExtension<'data>> {
        let byte = self.data.get_unchecked(0);
        let code = byte >> 6;
        StatusExtension::from(code).map_err(|_| UnsupportedExtension {
            extension: code,
            remaining_data: self.data.get_unchecked(1..),
        })
    }

    /// # Safety
    /// This reads data without checking boundaries.
    /// If self.data.len() < self.encoded_size() then this is safe.
    ///
    /// # Errors
    /// Fails if the status extension is unsupported.
    pub unsafe fn status(&self) -> Result<ValidEncodedStatus<'data>, UnsupportedExtension<'data>> {
        Ok(match self.extension()? {
            StatusExtension::Interface => ValidEncodedStatus::Interface(
                StatusInterfaceRef::start_decoding_unchecked(&self.data[1..]),
            ),
        })
    }
}

pub struct EncodedStatus<'data> {
    data: &'data [u8],
}

impl<'data> FailableEncodedData<'data> for EncodedStatus<'data> {
    type SizeError = StatusSizeError<'data>;
    type DecodeError = StatusDecodeError<'data>;
    type DecodedData = StatusRef<'data>;

    fn new(data: &'data [u8]) -> Self {
        Self { data }
    }

    fn encoded_size(&self) -> Result<usize, Self::SizeError> {
        if self.data.is_empty() {
            return Err(StatusSizeError::MissingBytes);
        }
        match unsafe { self.status()? } {
            ValidEncodedStatus::Interface(status) => status.encoded_size(),
        }
        .map(|v| v + 1)
        .map_err(|e| e.into())
    }

    unsafe fn complete_decoding(
        &self,
    ) -> Result<WithByteSize<StatusRef<'data>>, Self::DecodeError> {
        let mut ret = match &self.status()? {
            ValidEncodedStatus::Interface(interface) => {
                let WithByteSize {
                    item: status,
                    byte_size: size,
                } = interface.complete_decoding()?;
                WithByteSize {
                    item: StatusRef::Interface(status),
                    byte_size: size,
                }
            }
        };
        ret.byte_size += 1;
        Ok(ret)
    }
}

impl<'data> FailableDecodable<'data> for StatusRef<'data> {
    type Data = EncodedStatus<'data>;
    type FullDecodeError = StatusSizeError<'data>;
}

/// Details from the interface the command is coming from
#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Status {
    // Action(),
    Interface(StatusInterface),
}

impl Status {
    pub fn as_ref(&self) -> StatusRef {
        match self {
            Self::Interface(status) => StatusRef::Interface(status.as_ref()),
        }
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::unwrap_in_result, clippy::panic, clippy::expect_used)]
    use super::*;
    use crate::decodable::{FailableDecodable, WithByteSize};
    use crate::v1_2::dash7::{
        addressee::{AccessClass, AddresseeIdentifierRef, AddresseeRef, NlsMethod},
        interface_status::{AddresseeWithNlsStateRef, Dash7InterfaceStatusRef},
    };

    #[test]
    fn known() {
        fn test(op: StatusRef, data: &[u8]) {
            // Test op.encode_in() == data
            let mut encoded = [0_u8; 64];
            let size = op.encode_in(&mut encoded).unwrap();
            assert_eq!(size, data.len());
            assert_eq!(&encoded[..size], data);

            // Test decode(data) == op
            let WithByteSize {
                item: ret,
                byte_size: size,
            } = StatusRef::decode(data).unwrap();
            assert_eq!(size, data.len());
            assert_eq!(ret, op);
        }
        test(
            StatusRef::Interface(StatusInterfaceRef::Dash7(Dash7InterfaceStatusRef {
                ch_header: 0x1,
                ch_idx: 0x2,
                rxlev: 0x3,
                lb: 0x4,
                snr: 0x5,
                status: 0x6,
                token: 0x7,
                seq: 0x8,
                resp_to: 0x9,
                addressee_with_nls_state: AddresseeWithNlsStateRef::new(
                    AddresseeRef {
                        nls_method: NlsMethod::None,
                        access_class: AccessClass(0xE1),
                        identifier: AddresseeIdentifierRef::Uid(&[
                            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
                        ]),
                    },
                    None,
                )
                .unwrap(),
            })),
            &[
                34 | 0x40,
                0xD7,
                0x14,
                0x01,
                0x02,
                0x00,
                0x03,
                0x04,
                0x05,
                0x06,
                0x07,
                0x08,
                0x09,
                0x20,
                0xE1,
                0x00,
                0x11,
                0x22,
                0x33,
                0x44,
                0x55,
                0x66,
                0x77,
            ],
        );
    }

    #[test]
    fn consistence() {
        const TOT_SIZE: usize = 23;
        let op = StatusRef::Interface(StatusInterfaceRef::Dash7(Dash7InterfaceStatusRef {
            ch_header: 0x1,
            ch_idx: 0x2,
            rxlev: 0x3,
            lb: 0x4,
            snr: 0x5,
            status: 0x6,
            token: 0x7,
            seq: 0x8,
            resp_to: 0x9,
            addressee_with_nls_state: AddresseeWithNlsStateRef::new(
                AddresseeRef {
                    nls_method: NlsMethod::None,
                    access_class: AccessClass(0xE1),
                    identifier: AddresseeIdentifierRef::Uid(&[
                        0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
                    ]),
                },
                None,
            )
            .unwrap(),
        }));

        // Test decode(op.encode_in()) == op
        let mut encoded = [0_u8; TOT_SIZE];
        let size_encoded = op.encode_in(&mut encoded).unwrap();
        let WithByteSize {
            item: ret,
            byte_size: size_decoded,
        } = StatusRef::decode(&encoded).unwrap();
        assert_eq!(size_encoded, size_decoded);
        assert_eq!(ret, op);

        // Test decode(data).encode_in() == data
        let mut encoded2 = [0_u8; TOT_SIZE];
        let size_encoded2 = op.encode_in(&mut encoded2).unwrap();
        assert_eq!(size_encoded, size_encoded2);
        assert_eq!(encoded2[..size_encoded2], encoded[..size_encoded]);
    }
}

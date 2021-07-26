pub mod action;
pub mod define;
pub mod interface;

use crate::decodable::{FailableDecodable, FailableEncodedData, WithByteSize};
use crate::encodable::Encodable;
use crate::v1_2::define::op_code;
use crate::v1_2::error::action::status::{
    StatusDecodeError, StatusSizeError, UnsupportedExtension,
};
use define::extension::{self, StatusExtension};
use interface::{
    EncodedInterfaceStatus, EncodedInterfaceStatusMut, InterfaceStatus, InterfaceStatusRef,
};

// TODO Add feature based sub types support

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum StatusRef<'data> {
    // Action(),
    Interface(InterfaceStatusRef<'data>),
}

impl<'data> Encodable for StatusRef<'data> {
    unsafe fn encode_in_ptr(&self, out: *mut u8) -> usize {
        let extension = match self {
            Self::Interface(_) => extension::INTERFACE,
        };
        *out.add(0) = op_code::STATUS | (extension << 6);
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

impl<'data> StatusRef<'data> {
    pub fn extension(&self) -> StatusExtension {
        match self {
            Self::Interface(_) => StatusExtension::Interface,
        }
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_owned(&self) -> Status {
        match self {
            Self::Interface(status) => Status::Interface(status.to_owned()),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct EncodedStatus<'data> {
    data: &'data [u8],
}

pub enum ValidEncodedStatus<'data> {
    Interface(EncodedInterfaceStatus<'data>),
}

impl<'data> EncodedStatus<'data> {
    /// # Errors
    /// Fails if the status extension is unsupported.
    pub fn extension(&self) -> Result<StatusExtension, UnsupportedExtension<'data>> {
        unsafe {
            let byte = self.data.get_unchecked(0);
            let code = byte >> 6;
            StatusExtension::from(code).map_err(|_| UnsupportedExtension {
                extension: code,
                remaining_data: self.data.get_unchecked(1..),
            })
        }
    }

    /// # Errors
    /// Fails if the status extension is unsupported.
    pub fn status(&self) -> Result<ValidEncodedStatus<'data>, UnsupportedExtension<'data>> {
        unsafe {
            Ok(match self.extension()? {
                StatusExtension::Interface => ValidEncodedStatus::Interface(
                    InterfaceStatusRef::start_decoding_unchecked(self.data.get_unchecked(1..)),
                ),
            })
        }
    }
}

impl<'data> FailableEncodedData<'data> for EncodedStatus<'data> {
    type SourceData = &'data [u8];
    type SizeError = StatusSizeError<'data>;
    type DecodeError = StatusDecodeError<'data>;
    type DecodedData = StatusRef<'data>;

    unsafe fn new(data: Self::SourceData) -> Self {
        Self { data }
    }

    fn encoded_size(&self) -> Result<usize, Self::SizeError> {
        match self.status()? {
            ValidEncodedStatus::Interface(status) => status.encoded_size(),
        }
        .map(|v| v + 1)
        .map_err(|e| e.into())
    }

    fn complete_decoding(&self) -> Result<WithByteSize<Self::DecodedData>, Self::DecodeError> {
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

#[derive(Eq, PartialEq, Debug)]
pub struct EncodedStatusMut<'data> {
    data: &'data mut [u8],
}

pub enum ValidEncodedStatusMut<'data> {
    Interface(EncodedInterfaceStatusMut<'data>),
}

crate::make_downcastable!(EncodedStatusMut, EncodedStatus);

impl<'data> EncodedStatusMut<'data> {
    /// # Errors
    /// Fails if the status extension is unsupported.
    pub fn extension(&self) -> Result<StatusExtension, UnsupportedExtension<'data>> {
        self.borrow().extension()
    }

    /// # Errors
    /// Fails if the status extension is unsupported.
    pub fn status(&self) -> Result<ValidEncodedStatus, UnsupportedExtension<'data>> {
        self.borrow().status()
    }

    /// Changes the status extension.
    ///
    /// # Safety
    /// This will break:
    /// - the whole status structure.
    ///
    /// It also breaks the payload after this action.
    ///
    /// Only use it if you are sure about what you are doing.
    pub unsafe fn set_extension(&mut self, extension: StatusExtension) {
        *self.data.get_unchecked_mut(0) =
            (*self.data.get_unchecked(0) & 0x3F) | ((extension as u8) << 6);
    }

    /// # Errors
    /// Fails if the status extension is unsupported.
    pub fn status_mut(&mut self) -> Result<ValidEncodedStatusMut, UnsupportedExtension<'data>> {
        unsafe {
            Ok(match self.extension()? {
                StatusExtension::Interface => ValidEncodedStatusMut::Interface(
                    InterfaceStatusRef::start_decoding_unchecked_mut(
                        self.data.get_unchecked_mut(1..),
                    ),
                ),
            })
        }
    }
}

impl<'data> FailableEncodedData<'data> for EncodedStatusMut<'data> {
    type SourceData = &'data mut [u8];
    type SizeError = StatusSizeError<'data>;
    type DecodeError = StatusDecodeError<'data>;
    type DecodedData = StatusRef<'data>;

    unsafe fn new(data: Self::SourceData) -> Self {
        Self { data }
    }

    fn encoded_size(&self) -> Result<usize, Self::SizeError> {
        self.borrow().encoded_size()
    }

    fn complete_decoding(&self) -> Result<WithByteSize<Self::DecodedData>, Self::DecodeError> {
        self.borrow().complete_decoding()
    }
}

impl<'data> FailableDecodable<'data> for StatusRef<'data> {
    type Data = EncodedStatus<'data>;
    type DataMut = EncodedStatusMut<'data>;
    type FullDecodeError = StatusSizeError<'data>;
}

/// Details from the interface the command is coming from
#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Status {
    // Action(),
    Interface(InterfaceStatus),
}

impl Status {
    pub fn borrow(&self) -> StatusRef {
        match self {
            Self::Interface(status) => StatusRef::Interface(status.borrow()),
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
    use interface::ValidEncodedInterfaceStatusMut;

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

            // Test partial mutability
            let WithByteSize {
                item: mut decoder_mut,
                byte_size: expected_size,
            } = StatusRef::start_decoding_mut(&mut encoded).unwrap();
            assert_eq!(expected_size, size);

            match decoder_mut.status_mut().unwrap() {
                ValidEncodedStatusMut::Interface(mut decoder_mut) => {
                    match decoder_mut.status_mut().unwrap() {
                        ValidEncodedInterfaceStatusMut::Host => (),
                        ValidEncodedInterfaceStatusMut::Dash7(mut decoder_mut) => {
                            let original = decoder_mut.ch_header();
                            let new_ch_header = !original;
                            assert!(new_ch_header != original);
                            decoder_mut.set_ch_header(new_ch_header);
                            assert_eq!(decoder_mut.ch_header(), new_ch_header);
                        }
                    }
                }
            }

            // Unsafe mutations
            // TODO When multiple extension exist
            // let original = decoder_mut.extension().unwrap();
            // let target = if let StatusExtension::Interface = original {
            //     StatusExtension::Action
            // } else {
            //     StatusExtension::Interface
            // };
            // assert!(target != original);
            // unsafe { decoder_mut.set_extension(target) };
            // assert_eq!(decoder_mut.extension().unwrap(), target);

            // Check undecodability of shorter payload
            for i in 1..data.len() {
                assert_eq!(
                    StatusRef::start_decoding(&data[..i]),
                    Err(StatusSizeError::MissingBytes)
                );
            }

            // Check unencodability in shorter arrays
            for i in 0..data.len() {
                let mut array = vec![0; i];
                let ret = op.encode_in(&mut array);
                let missing = ret.unwrap_err();
                assert_eq!(missing, data.len());
            }
        }
        test(
            StatusRef::Interface(InterfaceStatusRef::Dash7(Dash7InterfaceStatusRef {
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
    fn errors() {
        let data = [34 | 0xC0, 1, 2];
        assert_eq!(
            unsafe { StatusRef::start_decoding_unchecked(&data).extension() },
            Err(UnsupportedExtension {
                extension: 3,
                remaining_data: &[1, 2],
            })
        );
    }

    #[test]
    fn consistence() {
        const TOT_SIZE: usize = 23;
        let op = StatusRef::Interface(InterfaceStatusRef::Dash7(Dash7InterfaceStatusRef {
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

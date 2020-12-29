pub mod action;
pub mod define;
pub mod interface;

use super::super::define::op_code::OpCode;
use crate::v1_2::error::{StatusDecodeError, UncheckedStatusDecodeError};
pub use define::StatusExtension;
pub use interface::{DecodableStatusInterface, StatusInterface};

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Status {
    // Action(),
    Interface(StatusInterface),
}

impl Status {
    pub fn extension(&self) -> StatusExtension {
        match self {
            Self::Interface(_) => StatusExtension::Interface,
        }
    }

    /// Encodes the Item into a data pointer without checking the size of the
    /// receiving byte array.
    ///
    /// This method is meant to allow unchecked cross language wrapper libraries
    /// to implement an unchecked call without having to build a fake slice with
    /// a fake size.
    ///
    /// It is not meant to be used inside a Rust library/binary.
    ///
    /// # Safety
    /// You are responsible for checking that `out.len()` >= [`self.size()`](#method.size).
    ///
    /// Failing that will result in the program writing out of bound in
    /// random parts of your memory.
    pub unsafe fn encode_in_ptr(&self, out: *mut u8) -> usize {
        *out.add(0) = OpCode::Status as u8 | ((self.extension() as u8) << 6);
        1 + match self {
            Self::Interface(status) => status.encode_in_ptr(out.add(1)),
        }
    }

    /// Encodes the Item without checking the size of the receiving
    /// byte array.
    ///
    /// # Safety
    /// You are responsible for checking that `out.len()` >= [`self.size()`](#method.size).
    ///
    /// Failing that will result in the program writing out of bound in
    /// random parts of your memory.
    pub unsafe fn encode_in_unchecked(&self, out: &mut [u8]) -> usize {
        self.encode_in_ptr(out.as_mut_ptr())
    }

    /// Encodes the value into pre allocated array.
    ///
    /// # Errors
    /// Fails if the pre allocated array is smaller than [self.size()](#method.size)
    /// returning the number of input bytes required.
    pub fn encode_in(&self, out: &mut [u8]) -> Result<usize, usize> {
        let size = self.size();
        if out.len() >= size {
            Ok(unsafe { self.encode_in_ptr(out.as_mut_ptr()) })
        } else {
            Err(size)
        }
    }

    /// Size in bytes of the encoded equivalent of the item.
    pub fn size(&self) -> usize {
        1 + match self {
            Self::Interface(status) => status.size(),
        }
    }

    /// Creates a decodable item from a data pointer without checking the data size.
    ///
    /// This method is meant to allow unchecked cross language wrapper libraries
    /// to implement an unchecked call without having to build a fake slice with
    /// a fake size.
    ///
    /// It is not meant to be used inside a Rust library/binary.
    ///
    /// # Errors
    /// Fails if the status extension is unknown. Returns the status extension.
    ///
    /// # Safety
    /// You are to check that:
    /// - The decoded data is bigger than the expected size of the `decodable` object.
    /// Meaning that given the resulting decodable object `decodable`:
    /// `data.len()` >= [`decodable.size()`](struct.DecodableStatus.html#method.size).
    ///
    /// Failing that might result in reading and interpreting data outside the given
    /// array (depending on what is done with the resulting object).
    pub unsafe fn start_decoding_ptr<'data>(data: *const u8) -> Result<DecodableStatus<'data>, u8> {
        DecodableStatus::from_ptr(data)
    }

    /// Creates a decodable item without checking the data size.
    ///
    /// # Errors
    /// Fails if the status extension is unknown. Returns the status extension.
    ///
    /// # Safety
    /// You are to check that:
    /// - The decoded data is bigger than the expected size of the `decodable` object.
    /// Meaning that given the resulting decodable object `decodable`:
    /// `data.len()` >= [`decodable.size()`](struct.DecodableStatus.html#method.size).
    ///
    /// Failing that might result in reading and interpreting data outside the given
    /// array (depending on what is done with the resulting object).
    pub unsafe fn start_decoding_unchecked(data: &[u8]) -> Result<DecodableStatus, u8> {
        DecodableStatus::new(data)
    }

    /// Creates a decodable item.
    ///
    /// This decodable item allows each parts of the item independently.
    ///
    /// # Errors
    /// - Fails if first byte of the data contains an invalid querycode.
    /// - Fails if the status extension is unknown.
    /// - Fails if data is empty.
    /// - Fails if data is smaller then the decoded expected size.
    pub fn start_decoding(data: &[u8]) -> Result<DecodableStatus, StatusDecodeError> {
        match data.get(0) {
            None => return Err(StatusDecodeError::MissingBytes(1)),
            Some(byte) => {
                if *byte & 0x3F != OpCode::Status as u8 {
                    return Err(StatusDecodeError::UnknownOpCode);
                }
            }
        }
        let ret = unsafe {
            Self::start_decoding_unchecked(data).map_err(|extension| {
                StatusDecodeError::UnknownExtension {
                    extension,
                    offset: 0,
                }
            })?
        };
        let ret_size = ret
            .size()
            .map_err(|id| StatusDecodeError::UnknownInterfaceId { id, offset: 1 })?;
        if data.len() < ret_size {
            return Err(StatusDecodeError::MissingBytes(ret_size));
        }
        Ok(ret)
    }

    /// Decodes the Item from a data pointer.
    ///
    /// Returns the decoded data and the number of bytes consumed to produce it.
    ///
    /// This method is meant to allow unchecked cross language wrapper libraries
    /// to implement an unchecked call without having to build a fake slice with
    /// a fake size.
    ///
    /// It is not meant to be used inside a Rust library/binary.
    ///
    /// # Errors
    /// Fails if the status extension is unknown. Returns the status extension.
    ///
    /// # Safety
    /// You are to check that:
    /// - The first byte contains this action's querycode.
    /// - The data is not empty.
    /// - The resulting size of the data consumed is smaller than the size of the
    /// decoded data.
    ///
    /// Failing that might result in reading and interpreting data outside the given
    /// array (depending on what is done with the resulting object).
    pub unsafe fn decode_ptr(data: *const u8) -> Result<(Self, usize), UncheckedStatusDecodeError> {
        Self::start_decoding_ptr(data)
            .map_err(|extension| UncheckedStatusDecodeError::UnknownExtension {
                extension,
                offset: 0,
            })?
            .complete_decoding()
            .map_err(|id| UncheckedStatusDecodeError::UnknownInterfaceId { id, offset: 1 })
    }

    /// Decodes the Item from bytes.
    ///
    /// Returns the decoded data and the number of bytes consumed to produce it.
    ///
    /// # Errors
    /// Fails if the status extension is unknown. Returns the status extension.
    ///
    /// # Safety
    /// You are to check that:
    /// - The first byte contains this action's querycode.
    /// - The data is not empty.
    /// - The resulting size of the data consumed is smaller than the size of the
    /// decoded data.
    ///
    /// Failing that might result in reading and interpreting data outside the given
    /// array (depending on what is done with the resulting object).
    pub unsafe fn decode_unchecked(
        data: &[u8],
    ) -> Result<(Self, usize), UncheckedStatusDecodeError> {
        Self::start_decoding_unchecked(data)
            .map_err(|extension| UncheckedStatusDecodeError::UnknownExtension {
                extension,
                offset: 0,
            })?
            .complete_decoding()
            .map_err(|id| UncheckedStatusDecodeError::UnknownInterfaceId { id, offset: 1 })
    }

    /// Decodes the item from bytes.
    ///
    /// # Errors
    /// - Fails if the status extension is unknown.
    /// - Fails if data is smaller then the decoded expected size.
    pub fn decode(data: &[u8]) -> Result<(Self, usize), StatusDecodeError> {
        Self::start_decoding(data)?
            .complete_decoding()
            .map_err(|id| StatusDecodeError::UnknownInterfaceId { id, offset: 1 })
    }
}

pub enum DecodableStatus<'data> {
    Interface(DecodableStatusInterface<'data>),
}

impl<'data> DecodableStatus<'data> {
    /// # Errors
    /// Fails if the status extension is unknown. Return the status extension.
    ///
    /// # Safety
    /// The data has to contain at least one byte.
    pub unsafe fn new(data: &'data [u8]) -> Result<Self, u8> {
        Self::from_ptr(data.as_ptr())
    }

    /// # Errors
    /// Fails if the querycode is invalid. Returning the querycode.
    ///
    /// # Safety
    /// The data has to contain at least one byte.
    unsafe fn from_ptr(data: *const u8) -> Result<Self, u8> {
        let byte = *data.add(0);
        let code = byte >> 6;
        let extension = match StatusExtension::from(code) {
            Ok(ext) => ext,
            Err(_) => return Err(code),
        };
        Ok(match extension {
            StatusExtension::Interface => {
                DecodableStatus::Interface(StatusInterface::start_decoding_ptr(data.add(1)))
            }
        })
    }

    /// Decodes the size of the Item in bytes
    ///
    /// # Errors
    /// Fails if this is an interface status with an unknown interface ID.
    pub fn size(&self) -> Result<usize, u8> {
        Ok(1 + match self {
            Self::Interface(status) => status.size()?,
        })
    }

    /// Fully decode the Item
    ///
    /// Returns the decoded data and the number of bytes consumed to produce it.
    ///
    /// # Errors
    /// Fails if the decoded status is an interface status and if the decoded
    /// interface_id is unknown.
    pub fn complete_decoding(&self) -> Result<(Status, usize), u8> {
        let (status, size) = match &self {
            DecodableStatus::Interface(interface) => {
                let (status, size) = interface.complete_decoding()?;
                (Status::Interface(status), size)
            }
        };
        Ok((status, 1 + size))
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::unwrap_in_result, clippy::panic, clippy::expect_used)]
    use super::*;
    use crate::v1_2::dash7::{
        addressee::{AccessClass, Addressee, AddresseeIdentifier, NlsMethod},
        interface_status::{AddresseeWithNlsState, Dash7InterfaceStatus},
    };

    #[test]
    fn known() {
        fn test(op: Status, data: &[u8]) {
            // Test op.encode_in() == data
            let mut encoded = [0_u8; 64];
            let size = op.encode_in(&mut encoded).unwrap();
            assert_eq!(size, data.len());
            assert_eq!(&encoded[..size], data);

            // Test decode(data) == op
            let (ret, size) = Status::decode(data).unwrap();
            assert_eq!(size, data.len());
            assert_eq!(ret, op);
        }
        test(
            Status::Interface(StatusInterface::Dash7(Dash7InterfaceStatus {
                ch_header: 0x1,
                ch_idx: 0x2,
                rxlev: 0x3,
                lb: 0x4,
                snr: 0x5,
                status: 0x6,
                token: 0x7,
                seq: 0x8,
                resp_to: 0x9,
                addressee_with_nls_state: AddresseeWithNlsState::new(
                    Addressee {
                        nls_method: NlsMethod::None,
                        access_class: AccessClass(0xE1),
                        identifier: AddresseeIdentifier::Uid([
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
        let op = Status::Interface(StatusInterface::Dash7(Dash7InterfaceStatus {
            ch_header: 0x1,
            ch_idx: 0x2,
            rxlev: 0x3,
            lb: 0x4,
            snr: 0x5,
            status: 0x6,
            token: 0x7,
            seq: 0x8,
            resp_to: 0x9,
            addressee_with_nls_state: AddresseeWithNlsState::new(
                Addressee {
                    nls_method: NlsMethod::None,
                    access_class: AccessClass(0xE1),
                    identifier: AddresseeIdentifier::Uid([
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
        let (ret, size_decoded) = Status::decode(&encoded).unwrap();
        assert_eq!(size_encoded, size_decoded);
        assert_eq!(ret, op);

        // Test decode(data).encode_in() == data
        let mut encoded2 = [0_u8; TOT_SIZE];
        let size_encoded2 = op.encode_in(&mut encoded2).unwrap();
        assert_eq!(size_encoded, size_encoded2);
        assert_eq!(encoded2[..size_encoded2], encoded[..size_encoded]);
    }
}

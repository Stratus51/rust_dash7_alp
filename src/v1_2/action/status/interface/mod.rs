use super::super::super::error::StatusInterfaceDecodeError;
use crate::v1_2::dash7::interface_status::{Dash7InterfaceStatus, DecodableDash7InterfaceStatus};
use crate::varint::{DecodableVarint, Varint};

pub mod define;
use define::InterfaceId;

/// Writes data to a file.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum StatusInterface {
    Host,
    Dash7(Dash7InterfaceStatus),
}

impl StatusInterface {
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
        let mut offset = 1;
        match self {
            Self::Host => {
                *out.add(0) = InterfaceId::Host as u8;
                offset += 1;
            }
            Self::Dash7(status) => {
                *out.add(0) = InterfaceId::Dash7 as u8;
                let status_length = Varint::new_unchecked(status.size() as u32);
                status_length.encode_in_ptr(out.add(offset));
                offset += status_length.size();
                offset += status.encode_in_ptr(out.add(offset));
            }
        }
        offset
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
    /// Fails if the pre allocated array is smaller than [`self.size()`](#method.size)
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
            Self::Host => 1,
            Self::Dash7(status) => {
                let status_len = unsafe { Varint::new_unchecked(status.size() as u32) };
                status_len.size() + status_len.usize()
            }
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
    /// # Safety
    /// You are to check that:
    /// - The first byte contains this action's querycode.
    /// - The decoded data is bigger than the expected size of the `decodable` object.
    /// Meaning that given the resulting decodable object `decodable`:
    /// `data.len()` >= [`decodable.size()`](struct.DecodableStatusInterface.html#method.size).
    ///
    /// Failing that might result in reading and interpreting data outside the given
    /// array (depending on what is done with the resulting object).
    pub unsafe fn start_decoding_ptr<'data>(data: *const u8) -> DecodableStatusInterface<'data> {
        DecodableStatusInterface::from_ptr(data)
    }

    /// Creates a decodable item without checking the data size.
    ///
    /// # Safety
    /// You are to check that:
    /// - The first byte contains this action's querycode.
    /// - The decoded data is bigger than the expected size of the `decodable` object.
    /// Meaning that given the resulting decodable object `decodable`:
    /// `data.len()` >= [`decodable.size()`](struct.DecodableStatusInterface.html#method.size).
    ///
    /// Failing that might result in reading and interpreting data outside the given
    /// array (depending on what is done with the resulting object).
    pub unsafe fn start_decoding_unchecked(data: &[u8]) -> DecodableStatusInterface {
        DecodableStatusInterface::new(data)
    }

    /// Creates a decodable item.
    ///
    /// This decodable item allows each parts of the item independently.
    ///
    /// # Errors
    /// - Fails if first byte of the data contains an unknown interface ID.
    /// - Fails if data is empty.
    /// - Fails if data is smaller then the decoded expected size.
    pub fn start_decoding(
        data: &[u8],
    ) -> Result<DecodableStatusInterface, StatusInterfaceDecodeError> {
        let ret = unsafe { Self::start_decoding_unchecked(data) };
        let ret_size = ret
            .size()
            .map_err(StatusInterfaceDecodeError::UnknownInterfaceId)?;
        if data.len() < ret_size {
            return Err(StatusInterfaceDecodeError::MissingBytes(ret_size));
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
    /// - Fails if first byte of the data contains an unknown interface ID.
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
    pub unsafe fn decode_ptr(data: *const u8) -> Result<(Self, usize), u8> {
        Self::start_decoding_ptr(data).complete_decoding()
    }

    /// Decodes the Item from bytes.
    ///
    /// Returns the decoded data and the number of bytes consumed to produce it.
    ///
    /// # Errors
    /// - Fails if first byte of the data contains an unknown interface ID.
    ///
    /// # Safety
    /// You are to check that:
    /// - The data is not empty.
    /// - The resulting size of the data consumed is smaller than the size of the
    /// decoded data.
    ///
    /// Failing that might result in reading and interpreting data outside the given
    /// array (depending on what is done with the resulting object).
    pub unsafe fn decode_unchecked(data: &[u8]) -> Result<(Self, usize), u8> {
        Self::start_decoding_unchecked(data).complete_decoding()
    }

    /// Decodes the item from bytes.
    ///
    /// On success, returns the decoded data and the number of bytes consumed
    /// to produce it.
    ///
    /// # Errors
    /// - Fails if first byte of the data contains an unknown interface ID.
    /// - Fails if data is smaller then the decoded expected size.
    // TODO This could be faster if instead of relying on start_decoding, we run a
    // start_decoding_unchecked and verify the size of the decoded data after parsing it.
    // But that implies potentially reading out of accessible memory which may trigger
    // some OS level panic, if the memory accesses are monitored.
    pub fn decode(data: &[u8]) -> Result<(Self, usize), StatusInterfaceDecodeError> {
        match Self::start_decoding(data) {
            Ok(v) => Ok(v
                .complete_decoding()
                .map_err(StatusInterfaceDecodeError::UnknownInterfaceId)?),
            Err(e) => Err(e),
        }
    }
}

pub enum DecodableStatusInterfaceKind<'data> {
    Host,
    Dash7(DecodableDash7InterfaceStatus<'data>),
}

pub struct DecodableStatusInterface<'data> {
    data: *const u8,
    data_life: core::marker::PhantomData<&'data ()>,
}

impl<'data> DecodableStatusInterface<'data> {
    unsafe fn new(data: &'data [u8]) -> Self {
        Self::from_ptr(data.as_ptr())
    }

    unsafe fn from_ptr(data: *const u8) -> Self {
        Self {
            data,
            data_life: core::marker::PhantomData,
        }
    }

    /// Decodes the size of the Item in bytes
    ///
    /// # Errors
    /// Fails if the decoded interface_id is unknown.
    pub fn size(&self) -> Result<usize, u8> {
        Ok(1 + self.len_field().size()
            + match &self.status()? {
                DecodableStatusInterfaceKind::Host => 0,
                DecodableStatusInterfaceKind::Dash7(status) => status.size(),
            })
    }

    /// # Errors
    /// Fails if the decoded interface_id is unknown.
    pub fn interface_id(&self) -> Result<InterfaceId, u8> {
        let byte = unsafe { *self.data.add(0) };
        Ok(InterfaceId::from(byte).map_err(|_| byte)?)
    }

    /// # Errors
    /// Fails if the decoded interface_id is unknown.
    pub fn len_field(&self) -> DecodableVarint<'data> {
        unsafe { Varint::start_decoding_ptr(self.data.add(1)) }
    }

    /// # Errors
    /// Fails if the decoded interface_id is unknown.
    pub fn status(&self) -> Result<DecodableStatusInterfaceKind<'data>, u8> {
        let offset = 1 + self.len_field().size();
        unsafe {
            Ok(match self.interface_id()? {
                InterfaceId::Host => DecodableStatusInterfaceKind::Host,
                InterfaceId::Dash7 => DecodableStatusInterfaceKind::Dash7(
                    Dash7InterfaceStatus::start_decoding_ptr(self.data.add(offset)),
                ),
            })
        }
    }

    /// Fully decode the Item
    ///
    /// Returns the decoded data and the number of bytes consumed to produce it.
    ///
    /// # Errors
    /// Fails if the decoded interface_id is unknown.
    pub fn complete_decoding(&self) -> Result<(StatusInterface, usize), u8> {
        let offset = 1 + self.len_field().size();
        unsafe {
            Ok(match self.interface_id()? {
                InterfaceId::Host => (StatusInterface::Host, offset),
                InterfaceId::Dash7 => {
                    let (status, size) = Dash7InterfaceStatus::decode_ptr(self.data.add(offset));
                    (StatusInterface::Dash7(status), offset + size)
                }
            })
        }
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::unwrap_in_result, clippy::panic, clippy::expect_used)]
    use super::*;
    use crate::v1_2::dash7::{
        addressee::{AccessClass, Addressee, AddresseeIdentifier, NlsMethod},
        interface_status::AddresseeWithNlsState,
    };

    #[test]
    fn known() {
        fn test(op: StatusInterface, data: &[u8]) {
            // Test op.encode_in() == data
            let mut encoded = [0_u8; 64];
            let size = op.encode_in(&mut encoded).unwrap();
            assert_eq!(size, data.len());
            assert_eq!(&encoded[..size], data);

            // Test decode(data) == op
            let (ret, size) = StatusInterface::decode(data).unwrap();
            assert_eq!(size, data.len());
            assert_eq!(ret, op);

            // Test partial_decode == op
            let decoder = StatusInterface::start_decoding(data).unwrap();
            assert_eq!(size, decoder.size().unwrap());
            assert_eq!(
                op,
                match decoder.status().unwrap() {
                    DecodableStatusInterfaceKind::Host => StatusInterface::Host,
                    DecodableStatusInterfaceKind::Dash7(status) =>
                        StatusInterface::Dash7(status.complete_decoding().0),
                },
            );
        }
        test(StatusInterface::Host, &[0x00, 0x00]);
        test(
            StatusInterface::Dash7(Dash7InterfaceStatus {
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
            }),
            &[
                0xD7, 0x14, 0x01, 0x02, 0x00, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x20, 0xE1,
                0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
            ],
        );
    }

    #[test]
    fn consistence() {
        const TOT_SIZE: usize = 22;
        let op = StatusInterface::Dash7(Dash7InterfaceStatus {
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
        });

        // Test decode(op.encode_in()) == op
        let mut encoded = [0_u8; TOT_SIZE];
        let size_encoded = op.encode_in(&mut encoded).unwrap();
        let (ret, size_decoded) = StatusInterface::decode(&encoded).unwrap();
        assert_eq!(size_encoded, size_decoded);
        assert_eq!(ret, op);

        // Test decode(data).encode_in() == data
        let mut encoded2 = [0_u8; TOT_SIZE];
        let size_encoded2 = op.encode_in(&mut encoded2).unwrap();
        assert_eq!(size_encoded, size_encoded2);
        assert_eq!(encoded2[..size_encoded2], encoded[..size_encoded]);
    }
}

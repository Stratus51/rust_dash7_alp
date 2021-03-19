use crate::decodable::{
    Decodable, EncodedData, FailableDecodable, FailableEncodedData, WithByteSize,
};
use crate::encodable::Encodable;
use crate::v1_2::dash7::interface_status::{
    Dash7InterfaceStatus, Dash7InterfaceStatusRef, EncodedDash7InterfaceStatus,
    EncodedDash7InterfaceStatusMut,
};
use crate::v1_2::error::action::status::interface::{
    StatusInterfaceSizeError, UnsupportedInterfaceId,
};
use crate::varint::{EncodedVarint, EncodedVarintMut, Varint};

pub mod define;
use define::InterfaceId;

// TODO ALP SPEC: The length field of this operand seems superfluous.
//
// The reason it exists is because it allows a generic device (IoT, server, gateway)
// receiving the ALP command to parse what is after the interface status.
//
// But in any real life situation:
// The interface status is generated by the interface of the Dash7 receiving device.
// Thus the device itself knows necessary full well how to decode the interface status.
// The issue only arises if this full payload is then forwarded to an entity which doesn't
// know about that interface status format.
//
// To me, that could be 2 types of entities:
// - Another IoT device, like if the communication architecture relies on a bridge which
// forwards the message from one communication mean to another.
//     But in that case, either:
//     - The bridging is done only to forward the messages from side A to side B where a gateway
//      B will be able to forward both traffic A and B to a global server. A global server is
//      supposed to know its population of devices and thus should have all the parsers required
//      to parse the interface status, and will need them to keep track of who sent what.
//     - The 2 IoT sides needs to communicate with each other and thus needs to known how to
//      parse those interface status, or they won't be able to communicate with any device on the
//      other side.
//
// I fail to see any use case where any entity would be happy to just skip this interface status
// and process anonymous ALP commands.
//
// I used to support the inclusion of this feature at the time, because of my background in server
// protocol where it is better to always be able to parse any generic payload that comes at you.
// But having worked with IoT for a while now,
//
// I think that this problematic does not apply to this field at all. We do not need to be overly
// generic and always parseable. We need instead to be specific and make payloads as small as possible,
// exactly like what is done in the rest of the ALP specification:
// For example, if we wanted any one to always be able to parse any ALP action it is capable of
// parsing, amidst of ALP actions we don't know, we could put a byte length in front of each action
// so that we could skip the unknown actions. But:
// - That would cost a lot bytes over the air.
// - This would not make any sense as the ALP commands are built as a sequence of instructions
// with control flow. This means that if you were, for example, to be able to ignore a query
// operand just because you don't know how to parse it, you would be executing a different ALP
// command than the other devices that would receive the exact same command.
// This would be terrible because it would make the effect of an ALP command almost random
// at the filesystem level (before even talking about any semantics attached to the files),
// depending on the listening population.
//
// While yes, skipping the interface status is non lethal, because it is not a flow control
// operand, I hardly think it is useful to keep this feature, which might just encourage
// using it for the wrong reasons.

// TODO Rename InterfaceStatus
/// Details from the interface the command is coming from
#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum StatusInterfaceRef<'data> {
    Host,
    Dash7(Dash7InterfaceStatusRef<'data>),
}

impl<'data> Encodable for StatusInterfaceRef<'data> {
    unsafe fn encode_in_ptr(&self, out: *mut u8) -> usize {
        let mut offset = 1;
        match self {
            Self::Host => {
                *out.add(0) = InterfaceId::Host as u8;
                offset += 1;
            }
            Self::Dash7(status) => {
                *out.add(0) = InterfaceId::Dash7 as u8;
                let status_length = Varint::new_unchecked(status.encoded_size() as u32);
                status_length.encode_in_ptr(out.add(offset));
                offset += status_length.encoded_size();
                offset += status.encode_in_ptr(out.add(offset));
            }
        }
        offset
    }

    fn encoded_size(&self) -> usize {
        1 + match self {
            Self::Host => 1,
            Self::Dash7(status) => {
                let status_len = unsafe { Varint::new_unchecked(status.encoded_size() as u32) };
                status_len.encoded_size() + unsafe { status_len.usize() }
            }
        }
    }
}

impl<'data> StatusInterfaceRef<'data> {
    pub fn to_owned(&self) -> StatusInterface {
        match self {
            Self::Host => StatusInterface::Host,
            Self::Dash7(status) => StatusInterface::Dash7(status.to_owned()),
        }
    }
}

pub enum ValidEncodedStatusInterface<'data> {
    Host,
    Dash7(EncodedDash7InterfaceStatus<'data>),
}

pub struct EncodedStatusInterface<'data> {
    data: &'data [u8],
}

impl<'data> EncodedStatusInterface<'data> {
    /// # Errors
    /// Fails if the interface status id is unsupported.
    pub fn interface_id(&self) -> Result<InterfaceId, UnsupportedInterfaceId<'data>> {
        unsafe {
            let byte = self.data.get_unchecked(0);
            Ok(
                InterfaceId::from(*byte).map_err(|_| UnsupportedInterfaceId {
                    id: *byte,
                    remaining_data: self.data.get_unchecked(1..),
                })?,
            )
        }
    }

    pub fn len_field(&self) -> EncodedVarint<'data> {
        unsafe { Varint::start_decoding_unchecked(self.data.get_unchecked(1..)) }
    }

    /// # Errors
    /// Fails if the interface status id is unsupported.
    pub fn status(
        &self,
    ) -> Result<ValidEncodedStatusInterface<'data>, UnsupportedInterfaceId<'data>> {
        unsafe {
            let offset = 1 + self.len_field().encoded_size_unchecked();
            Ok(match self.interface_id()? {
                InterfaceId::Host => ValidEncodedStatusInterface::Host,
                InterfaceId::Dash7 => ValidEncodedStatusInterface::Dash7(
                    Dash7InterfaceStatusRef::start_decoding_unchecked(
                        self.data.get_unchecked(offset..),
                    ),
                ),
            })
        }
    }
}

impl<'data> FailableEncodedData<'data> for EncodedStatusInterface<'data> {
    type SourceData = &'data [u8];
    type SizeError = StatusInterfaceSizeError<'data>;
    type DecodeError = UnsupportedInterfaceId<'data>;
    type DecodedData = StatusInterfaceRef<'data>;

    unsafe fn new(data: Self::SourceData) -> Self {
        Self { data }
    }

    fn encoded_size(&self) -> Result<usize, Self::SizeError> {
        let mut size = 2;
        let data_size = self.data.len();
        if data_size < size {
            return Err(StatusInterfaceSizeError::MissingBytes);
        }
        size = 1 + unsafe { self.len_field().encoded_size_unchecked() };
        if data_size < size {
            return Err(StatusInterfaceSizeError::MissingBytes);
        }
        size += match &self
            .status()
            .map_err(StatusInterfaceSizeError::UnsupportedInterfaceId)?
        {
            ValidEncodedStatusInterface::Host => 0,
            ValidEncodedStatusInterface::Dash7(status) => match status.encoded_size() {
                Ok(size) => size,
                Err(_) => return Err(StatusInterfaceSizeError::MissingBytes),
            },
        };
        if data_size < size {
            return Err(StatusInterfaceSizeError::MissingBytes);
        }
        Ok(size)
    }

    fn complete_decoding(&self) -> Result<WithByteSize<Self::DecodedData>, Self::DecodeError> {
        let offset = 1 + unsafe { self.len_field().encoded_size_unchecked() };
        unsafe {
            Ok(match self.interface_id()? {
                InterfaceId::Host => WithByteSize {
                    item: StatusInterfaceRef::Host,
                    byte_size: offset,
                },
                InterfaceId::Dash7 => {
                    let WithByteSize {
                        item: status,
                        byte_size: size,
                    } = Dash7InterfaceStatusRef::decode_unchecked(
                        self.data.get_unchecked(offset..),
                    );
                    WithByteSize {
                        item: StatusInterfaceRef::Dash7(status),
                        byte_size: offset + size,
                    }
                }
            })
        }
    }
}

pub struct EncodedStatusInterfaceMut<'data> {
    data: &'data mut [u8],
}

pub enum ValidEncodedStatusInterfaceMut<'data> {
    Host,
    Dash7(EncodedDash7InterfaceStatusMut<'data>),
}

crate::make_downcastable!(EncodedStatusInterfaceMut, EncodedStatusInterface);

impl<'data> EncodedStatusInterfaceMut<'data> {
    /// # Errors
    /// Fails if the interface status id is unsupported.
    pub fn interface_id(&self) -> Result<InterfaceId, UnsupportedInterfaceId<'data>> {
        self.as_ref().interface_id()
    }

    pub fn len_field(&self) -> EncodedVarint<'data> {
        self.as_ref().len_field()
    }

    /// # Errors
    /// Fails if the interface status id is unsupported.
    pub fn status(
        &self,
    ) -> Result<ValidEncodedStatusInterface<'data>, UnsupportedInterfaceId<'data>> {
        self.as_ref().status()
    }

    /// # Safety
    /// This method method changes the interface id of the status.
    /// This will probably make the payload incoherent unless you are sure that the payloads for
    /// both those interface statuses are identical.
    pub unsafe fn set_interface_id(&mut self, interface_id: u8) {
        *self.data.get_unchecked_mut(0) = interface_id;
    }

    /// # Safety
    /// Even though it is technically possible, changing the length field of the interface status
    /// has a high chance of making it incoherent.
    /// Be sure that you have adapted the payload somehow to match it.
    pub unsafe fn len_field_mut(&mut self) -> EncodedVarintMut {
        Varint::start_decoding_unchecked_mut(self.data.get_unchecked_mut(1..))
    }

    /// # Errors
    /// Fails if the interface status id is unsupported.
    pub fn status_mut(
        &mut self,
    ) -> Result<ValidEncodedStatusInterfaceMut, UnsupportedInterfaceId<'data>> {
        unsafe {
            let offset = 1 + self.len_field().encoded_size_unchecked();
            Ok(match self.interface_id()? {
                InterfaceId::Host => ValidEncodedStatusInterfaceMut::Host,
                InterfaceId::Dash7 => ValidEncodedStatusInterfaceMut::Dash7(
                    Dash7InterfaceStatusRef::start_decoding_unchecked_mut(
                        self.data.get_unchecked_mut(offset..),
                    ),
                ),
            })
        }
    }
}

impl<'data> FailableEncodedData<'data> for EncodedStatusInterfaceMut<'data> {
    type SourceData = &'data mut [u8];
    type SizeError = StatusInterfaceSizeError<'data>;
    type DecodeError = UnsupportedInterfaceId<'data>;
    type DecodedData = StatusInterfaceRef<'data>;

    unsafe fn new(data: Self::SourceData) -> Self {
        Self { data }
    }

    fn encoded_size(&self) -> Result<usize, Self::SizeError> {
        self.as_ref().encoded_size()
    }

    fn complete_decoding(&self) -> Result<WithByteSize<Self::DecodedData>, Self::DecodeError> {
        self.as_ref().complete_decoding()
    }
}

impl<'data> FailableDecodable<'data> for StatusInterfaceRef<'data> {
    type Data = EncodedStatusInterface<'data>;
    type DataMut = EncodedStatusInterfaceMut<'data>;
    type FullDecodeError = StatusInterfaceSizeError<'data>;
}

/// Details from the interface the command is coming from
#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum StatusInterface {
    Host,
    Dash7(Dash7InterfaceStatus),
}

impl StatusInterface {
    pub fn as_ref(&self) -> StatusInterfaceRef {
        match self {
            Self::Host => StatusInterfaceRef::Host,
            Self::Dash7(status) => StatusInterfaceRef::Dash7(status.as_ref()),
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
        interface_status::AddresseeWithNlsStateRef,
    };

    #[test]
    fn known() {
        fn test(op: StatusInterfaceRef, data: &[u8]) {
            // Test op.encode_in() == data
            let mut encoded = [0_u8; 64];
            let size = op.encode_in(&mut encoded).unwrap();
            assert_eq!(size, data.len());
            assert_eq!(&encoded[..size], data);

            // Test decode(data) == op
            let WithByteSize {
                item: ret,
                byte_size: size,
            } = StatusInterfaceRef::decode(data).unwrap();
            assert_eq!(size, data.len());
            assert_eq!(ret, op);

            // Test partial_decode == op
            let WithByteSize {
                item: decoder,
                byte_size: expected_size,
            } = StatusInterfaceRef::start_decoding(data).unwrap();
            assert_eq!(expected_size, size);
            assert_eq!(decoder.encoded_size().unwrap(), size);
            assert_eq!(
                op,
                match decoder.status().unwrap() {
                    ValidEncodedStatusInterface::Host => StatusInterfaceRef::Host,
                    ValidEncodedStatusInterface::Dash7(status) =>
                        StatusInterfaceRef::Dash7(status.complete_decoding().item),
                },
            );
        }
        test(StatusInterfaceRef::Host, &[0x00, 0x00]);
        test(
            StatusInterfaceRef::Dash7(Dash7InterfaceStatusRef {
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
        let op = StatusInterfaceRef::Dash7(Dash7InterfaceStatusRef {
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
        });

        // Test decode(op.encode_in()) == op
        let mut encoded = [0_u8; TOT_SIZE];
        let size_encoded = op.encode_in(&mut encoded).unwrap();
        let WithByteSize {
            item: ret,
            byte_size: size_decoded,
        } = StatusInterfaceRef::decode(&encoded).unwrap();
        assert_eq!(size_encoded, size_decoded);
        assert_eq!(ret, op);

        // Test decode(data).encode_in() == data
        let mut encoded2 = [0_u8; TOT_SIZE];
        let size_encoded2 = op.encode_in(&mut encoded2).unwrap();
        assert_eq!(size_encoded, size_encoded2);
        assert_eq!(encoded2[..size_encoded2], encoded[..size_encoded]);
    }
}

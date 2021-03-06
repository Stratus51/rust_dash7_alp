use super::addressee::{self, Addressee, AddresseeRef, EncodedAddressee, NlsMethod};
use crate::decodable::{Decodable, EncodedData, SizeError, WithByteSize};
use crate::encodable::Encodable;

/// Maximum byte size of an encoded `ReadFileData`
pub const MAX_SIZE: usize = 10 + addressee::MAX_SIZE + 5;

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct AddresseeWithNlsStateRef<'item> {
    addressee: AddresseeRef<'item>,
    nls_state: Option<&'item [u8; 5]>,
}

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum AddresseeWithNlsStateError {
    NlsMethodMismatchNlsStatePresence,
}

impl<'item> AddresseeWithNlsStateRef<'item> {
    /// # Safety
    /// You are to make sure the nls_state exists if and only if the addressee nls_method is None.
    pub unsafe fn new_unchecked(
        addressee: AddresseeRef<'item>,
        nls_state: Option<&'item [u8; 5]>,
    ) -> Self {
        Self {
            addressee,
            nls_state,
        }
    }

    /// # Errors
    /// Fails if the nls_method is None and the nls_state is defined or if the nls_method is
    /// not None and the nls_state is None.
    pub fn new(
        addressee: AddresseeRef<'item>,
        nls_state: Option<&'item [u8; 5]>,
    ) -> Result<Self, AddresseeWithNlsStateError> {
        let security = addressee.nls_method != NlsMethod::None;
        if security == nls_state.is_some() {
            Ok(unsafe { Self::new_unchecked(addressee, nls_state) })
        } else {
            Err(AddresseeWithNlsStateError::NlsMethodMismatchNlsStatePresence)
        }
    }

    pub fn addressee(&self) -> &AddresseeRef {
        &self.addressee
    }

    pub fn nls_state(&self) -> &Option<&'item [u8; 5]> {
        &self.nls_state
    }

    pub fn encoded_size(&self) -> usize {
        let addressee_size = self.addressee.encoded_size();
        if self.nls_state.is_some() {
            addressee_size + 5
        } else {
            addressee_size
        }
    }

    pub fn to_owned(&self) -> AddresseeWithNlsState {
        AddresseeWithNlsState {
            addressee: self.addressee.to_owned(),
            nls_state: self.nls_state.copied(),
        }
    }
}

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct AddresseeWithNlsState {
    addressee: Addressee,
    nls_state: Option<[u8; 5]>,
}

impl AddresseeWithNlsState {
    pub fn as_ref(&self) -> AddresseeWithNlsStateRef {
        AddresseeWithNlsStateRef {
            addressee: self.addressee.as_ref(),
            nls_state: self.nls_state.as_ref(),
        }
    }
}

pub struct EncodedAddresseeWithNlsState<'data> {
    data: &'data [u8],
    addressee: EncodedAddressee<'data>,
}

impl<'data> EncodedAddresseeWithNlsState<'data> {
    /// # Safety
    /// This reads data without checking boundaries.
    /// If self.data.len() < self.encoded_size() then this is safe.
    pub unsafe fn has_auth(&self) -> bool {
        self.addressee.nls_method() != NlsMethod::None
    }

    /// # Safety
    /// This reads data without checking boundaries.
    /// If self.data.len() < self.encoded_size() then this is safe.
    pub unsafe fn addressee(&self) -> &EncodedAddressee<'data> {
        &self.addressee
    }

    /// # Safety
    /// This reads data without checking boundaries.
    /// If self.data.len() < self.encoded_size() then this is safe.
    pub unsafe fn nls_state(&self) -> Option<&[u8]> {
        if self.addressee.nls_method() == NlsMethod::None {
            None
        } else {
            let size = self.addressee.size_unchecked();
            let data = &*(self.data.get_unchecked(size..size + 5).as_ptr() as *const [u8; 5]);
            Some(&*data)
        }
    }

    /// # Safety
    /// You are to warrant, somehow, that the input byte array contains a complete item.
    /// Else this might result in out of bound reads, and absurd results.
    pub unsafe fn size_unchecked(&self) -> usize {
        let nls_state_size = if self.has_auth() { 5 } else { 0 };
        self.addressee.size_unchecked() + nls_state_size
    }
}

impl<'data> EncodedData<'data> for EncodedAddresseeWithNlsState<'data> {
    type DecodedData = AddresseeWithNlsStateRef<'data>;
    fn new(data: &'data [u8]) -> Self {
        Self {
            data,
            addressee: AddresseeRef::start_decoding_unchecked(data),
        }
    }

    fn encoded_size(&self) -> Result<usize, SizeError> {
        let mut size = 3;
        let data_size = self.data.len();
        if data_size < size {
            return Err(SizeError::MissingBytes);
        }
        size = unsafe { self.addressee.size_unchecked() };
        if data_size < size {
            return Err(SizeError::MissingBytes);
        }
        Ok(size)
    }

    unsafe fn complete_decoding(&self) -> WithByteSize<AddresseeWithNlsStateRef<'data>> {
        let WithByteSize {
            item: addressee,
            byte_size: addressee_size,
        } = self.addressee.complete_decoding();
        let (nls_state, nls_state_size) = if addressee.nls_method == NlsMethod::None {
            (None, 0)
        } else {
            let data = &*(self
                .data
                .get_unchecked(addressee_size..addressee_size + 5)
                .as_ptr() as *const [u8; 5]);
            (Some(&*data), 5)
        };
        WithByteSize {
            item: AddresseeWithNlsStateRef::new_unchecked(addressee, nls_state),
            byte_size: addressee_size + nls_state_size,
        }
    }
}

impl<'data> Decodable<'data> for AddresseeWithNlsStateRef<'data> {
    type Data = EncodedAddresseeWithNlsState<'data>;
}

/// Writes data to a file.
#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Dash7InterfaceStatusRef<'item> {
    pub ch_header: u8,
    pub ch_idx: u16,
    pub rxlev: u8,
    pub lb: u8,
    pub snr: u8,
    pub status: u8,
    pub token: u8,
    pub seq: u8,
    pub resp_to: u8,
    pub addressee_with_nls_state: AddresseeWithNlsStateRef<'item>,
}

impl<'data> Encodable for Dash7InterfaceStatusRef<'data> {
    unsafe fn encode_in_ptr(&self, out: *mut u8) -> usize {
        let mut size = 10;
        *out.add(0) = self.ch_header;
        // TODO: SPEC: Endianness ?
        out.add(1).copy_from(self.ch_idx.to_le_bytes().as_ptr(), 2);
        *out.add(3) = self.rxlev;
        *out.add(4) = self.lb;
        *out.add(5) = self.snr;
        *out.add(6) = self.status;
        *out.add(7) = self.token;
        *out.add(8) = self.seq;
        *out.add(9) = self.resp_to;
        size += self
            .addressee_with_nls_state
            .addressee()
            .encode_in_ptr(out.add(10));
        match self.addressee_with_nls_state.nls_state() {
            Some(nls_state) => {
                out.add(size).copy_from(nls_state.as_ptr(), 5);
                size += 5
            }
            None => (),
        }

        size
    }

    fn encoded_size(&self) -> usize {
        10 + self.addressee_with_nls_state.encoded_size()
    }
}

impl<'item> Dash7InterfaceStatusRef<'item> {
    pub fn to_owned(&self) -> Dash7InterfaceStatus {
        Dash7InterfaceStatus {
            ch_header: self.ch_header,
            ch_idx: self.ch_idx,
            rxlev: self.rxlev,
            lb: self.lb,
            snr: self.snr,
            status: self.status,
            token: self.token,
            seq: self.seq,
            resp_to: self.resp_to,
            addressee_with_nls_state: self.addressee_with_nls_state.to_owned(),
        }
    }
}

pub struct EncodedDash7InterfaceStatus<'data> {
    data: &'data [u8],
}

impl<'data> EncodedDash7InterfaceStatus<'data> {
    /// # Safety
    /// This reads data without checking boundaries.
    /// If self.data.len() < self.encoded_size() then this is safe.
    pub unsafe fn ch_header(&self) -> u8 {
        *self.data.get_unchecked(0)
    }

    /// # Safety
    /// This reads data without checking boundaries.
    /// If self.data.len() < self.encoded_size() then this is safe.
    pub unsafe fn ch_idx(&self) -> u16 {
        let mut data: [u8; 2] = [core::mem::MaybeUninit::uninit().assume_init(); 2];
        data.as_mut_ptr().copy_from(self.data.get_unchecked(1), 2);
        // TODO SPEC endianess
        u16::from_le_bytes(data)
    }
    /// # Safety
    /// This reads data without checking boundaries.
    /// If self.data.len() < self.encoded_size() then this is safe.
    pub unsafe fn rxlev(&self) -> u8 {
        *self.data.get_unchecked(3)
    }
    /// # Safety
    /// This reads data without checking boundaries.
    /// If self.data.len() < self.encoded_size() then this is safe.
    pub unsafe fn lb(&self) -> u8 {
        *self.data.get_unchecked(4)
    }
    /// # Safety
    /// This reads data without checking boundaries.
    /// If self.data.len() < self.encoded_size() then this is safe.
    pub unsafe fn snr(&self) -> u8 {
        *self.data.get_unchecked(5)
    }
    /// # Safety
    /// This reads data without checking boundaries.
    /// If self.data.len() < self.encoded_size() then this is safe.
    pub unsafe fn status(&self) -> u8 {
        *self.data.get_unchecked(6)
    }
    /// # Safety
    /// This reads data without checking boundaries.
    /// If self.data.len() < self.encoded_size() then this is safe.
    pub unsafe fn token(&self) -> u8 {
        *self.data.get_unchecked(7)
    }
    /// # Safety
    /// This reads data without checking boundaries.
    /// If self.data.len() < self.encoded_size() then this is safe.
    pub unsafe fn seq(&self) -> u8 {
        *self.data.get_unchecked(8)
    }
    /// # Safety
    /// This reads data without checking boundaries.
    /// If self.data.len() < self.encoded_size() then this is safe.
    pub unsafe fn resp_to(&self) -> u8 {
        *self.data.get_unchecked(9)
    }
    /// # Safety
    /// This reads data without checking boundaries.
    /// If self.data.len() < self.encoded_size() then this is safe.
    pub unsafe fn addressee(&self) -> EncodedAddressee<'data> {
        AddresseeRef::start_decoding_unchecked(self.data.get_unchecked(10..))
    }

    /// # Safety
    /// This reads data without checking boundaries.
    /// If self.data.len() < self.encoded_size() then this is safe.
    pub unsafe fn addressee_with_nls_state(&self) -> EncodedAddresseeWithNlsState<'data> {
        EncodedAddresseeWithNlsState::new(self.data.get_unchecked(10..))
    }

    /// # Safety
    /// You have to be sure your byte array is at least 11 bytes long for this
    /// function call to be safe.
    pub unsafe fn size_unchecked(&self) -> usize {
        10 + self.addressee_with_nls_state().size_unchecked()
    }
}

impl<'data> EncodedData<'data> for EncodedDash7InterfaceStatus<'data> {
    type DecodedData = Dash7InterfaceStatusRef<'data>;

    fn new(data: &'data [u8]) -> Self {
        Self { data }
    }

    fn encoded_size(&self) -> Result<usize, SizeError> {
        let mut size = 11;
        let data_size = self.data.len();
        if data_size < size {
            return Err(SizeError::MissingBytes);
        }
        size += unsafe { self.addressee_with_nls_state().size_unchecked() };
        size -= 1;
        if data_size < size {
            return Err(SizeError::MissingBytes);
        }
        Ok(size)
    }

    unsafe fn complete_decoding(&self) -> WithByteSize<Dash7InterfaceStatusRef<'data>> {
        let WithByteSize {
            item: addressee_with_nls_state,
            byte_size: end_size,
        } = self.addressee_with_nls_state().complete_decoding();
        WithByteSize {
            item: Dash7InterfaceStatusRef {
                ch_header: self.ch_header(),
                ch_idx: self.ch_idx(),
                rxlev: self.rxlev(),
                lb: self.lb(),
                snr: self.snr(),
                status: self.status(),
                token: self.token(),
                seq: self.seq(),
                resp_to: self.resp_to(),
                addressee_with_nls_state,
            },
            byte_size: 10 + end_size,
        }
    }
}

impl<'data> Decodable<'data> for Dash7InterfaceStatusRef<'data> {
    type Data = EncodedDash7InterfaceStatus<'data>;
}

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Dash7InterfaceStatus {
    ch_header: u8,
    ch_idx: u16,
    rxlev: u8,
    lb: u8,
    snr: u8,
    status: u8,
    token: u8,
    seq: u8,
    resp_to: u8,
    addressee_with_nls_state: AddresseeWithNlsState,
}

impl Dash7InterfaceStatus {
    pub fn as_ref(&self) -> Dash7InterfaceStatusRef {
        Dash7InterfaceStatusRef {
            ch_header: self.ch_header,
            ch_idx: self.ch_idx,
            rxlev: self.rxlev,
            lb: self.lb,
            snr: self.snr,
            status: self.status,
            token: self.token,
            seq: self.seq,
            resp_to: self.resp_to,
            addressee_with_nls_state: self.addressee_with_nls_state.as_ref(),
        }
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::unwrap_in_result, clippy::panic, clippy::expect_used)]
    use super::addressee::{AccessClass, AddresseeIdentifierRef};
    use super::*;

    #[test]
    fn known() {
        fn test(op: Dash7InterfaceStatusRef, data: &[u8]) {
            // Test op.encode_in() == data
            let mut encoded = [0_u8; 64];
            let size = op.encode_in(&mut encoded).unwrap();
            assert_eq!(size, data.len());
            assert_eq!(&encoded[..size], data);

            // Test decode(data) == op
            let WithByteSize {
                item: ret,
                byte_size: size,
            } = Dash7InterfaceStatusRef::decode(data).unwrap();
            assert_eq!(size, data.len());
            assert_eq!(ret, op);

            // Test partial_decode == op
            let WithByteSize {
                item: decoder,
                byte_size: expected_size,
            } = Dash7InterfaceStatusRef::start_decoding(data).unwrap();
            unsafe {
                assert_eq!(
                    ret.addressee_with_nls_state.addressee(),
                    &decoder.addressee().complete_decoding().item
                );
                assert_eq!(expected_size, size);
                assert_eq!(decoder.size_unchecked(), size);
                assert_eq!(decoder.encoded_size().unwrap(), size);
                assert_eq!(
                    op,
                    Dash7InterfaceStatusRef {
                        ch_header: decoder.ch_header(),
                        ch_idx: decoder.ch_idx(),
                        rxlev: decoder.rxlev(),
                        lb: decoder.lb(),
                        snr: decoder.snr(),
                        status: decoder.status(),
                        token: decoder.token(),
                        seq: decoder.seq(),
                        resp_to: decoder.resp_to(),
                        addressee_with_nls_state: decoder
                            .addressee_with_nls_state()
                            .complete_decoding()
                            .item,
                    }
                );
            }
        }
        test(
            Dash7InterfaceStatusRef {
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
                        nls_method: NlsMethod::AesCcm64,
                        access_class: AccessClass(0xE1),
                        identifier: AddresseeIdentifierRef::Uid(&[
                            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
                        ]),
                    },
                    Some(&[0xA, 0xB, 0xC, 0xD, 0xE]),
                )
                .unwrap(),
            },
            &[
                0x01, 0x02, 0x00, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x26, 0xE1, 0x00, 0x11,
                0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0xA, 0xB, 0xC, 0xD, 0xE,
            ],
        );
        test(
            Dash7InterfaceStatusRef {
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
            },
            &[
                0x01, 0x02, 0x00, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x20, 0xE1, 0x00, 0x11,
                0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
            ],
        );
    }

    #[test]
    fn consistence() {
        const TOT_SIZE: usize = 10 + 10 + 5;
        let op = Dash7InterfaceStatusRef {
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
                    nls_method: NlsMethod::AesCcm64,
                    access_class: AccessClass(0xE1),
                    identifier: AddresseeIdentifierRef::Uid(&[
                        0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
                    ]),
                },
                Some(&[0xA, 0xB, 0xC, 0xD, 0xE]),
            )
            .unwrap(),
        };

        // Test decode(op.encode_in()) == op
        let mut encoded = [0_u8; TOT_SIZE];
        let size_encoded = op.encode_in(&mut encoded).unwrap();
        let WithByteSize {
            item: ret,
            byte_size: size_decoded,
        } = Dash7InterfaceStatusRef::decode(&encoded).unwrap();
        assert_eq!(size_encoded, size_decoded);
        assert_eq!(ret, op);

        // Test decode(data).encode_in() == data
        let mut encoded2 = [0_u8; TOT_SIZE];
        let size_encoded2 = op.encode_in(&mut encoded2).unwrap();
        assert_eq!(size_encoded, size_encoded2);
        assert_eq!(encoded2[..size_encoded2], encoded[..size_encoded]);
    }
}

use super::addressee::{self, Addressee, AddresseeRef, EncodedAddressee, NlsMethod};
use crate::decodable::{Decodable, EncodedData, WithByteSize};

/// Maximum byte size of an encoded `ReadFileData`
pub const MAX_SIZE: usize = 10 + addressee::MAX_SIZE + 5;

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct AddresseeWithNlsStateRef<'item> {
    addressee: AddresseeRef<'item>,
    nls_state: Option<&'item [u8; 5]>,
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
    ) -> Result<Self, ()> {
        let security = addressee.nls_method != NlsMethod::None;
        if security == nls_state.is_some() {
            Ok(unsafe { Self::new_unchecked(addressee, nls_state) })
        } else {
            Err(())
        }
    }

    pub fn addressee(&self) -> &AddresseeRef {
        &self.addressee
    }

    pub fn nls_state(&self) -> &Option<&'item [u8; 5]> {
        &self.nls_state
    }

    pub fn size(&self) -> usize {
        let addressee_size = self.addressee.size();
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
    pub fn has_auth(&self) -> bool {
        self.addressee.nls_method() != NlsMethod::None
    }

    pub fn addressee(&self) -> &EncodedAddressee<'data> {
        &self.addressee
    }

    pub fn nls_state(&self) -> Option<&[u8]> {
        if self.addressee.nls_method() == NlsMethod::None {
            None
        } else {
            unsafe {
                let size = self.addressee.expected_size();
                let data = &*(self.data.get_unchecked(size..size + 5).as_ptr() as *const [u8; 5]);
                Some(&*data)
            }
        }
    }
}

impl<'data> EncodedData<'data> for EncodedAddresseeWithNlsState<'data> {
    type DecodedData = AddresseeWithNlsStateRef<'data>;
    unsafe fn new(data: &'data [u8]) -> Self {
        Self {
            data,
            addressee: AddresseeRef::start_decoding_unchecked(data),
        }
    }

    unsafe fn expected_size(&self) -> usize {
        let nls_state_size = if self.has_auth() { 5 } else { 0 };
        self.addressee.expected_size() + nls_state_size
    }

    fn smaller_than(&self, data_size: usize) -> Result<usize, usize> {
        let mut size = 1;
        if data_size < size {
            return Err(size);
        }
        size = unsafe { self.addressee.expected_size() };
        if data_size < size {
            return Err(size);
        }
        Ok(size)
    }

    fn complete_decoding(&self) -> WithByteSize<AddresseeWithNlsStateRef<'data>> {
        let WithByteSize {
            item: addressee,
            byte_size: addressee_size,
        } = self.addressee.complete_decoding();
        let (nls_state, nls_state_size) = unsafe {
            if addressee.nls_method == NlsMethod::None {
                (None, 0)
            } else {
                let data = &*(self
                    .data
                    .get_unchecked(addressee_size..addressee_size + 5)
                    .as_ptr() as *const [u8; 5]);
                (Some(&*data), 5)
            }
        };
        WithByteSize {
            item: unsafe { AddresseeWithNlsStateRef::new_unchecked(addressee, nls_state) },
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

impl<'item> Dash7InterfaceStatusRef<'item> {
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
        10 + self.addressee_with_nls_state.size()
    }

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
    pub fn ch_header(&self) -> u8 {
        unsafe { *self.data.get_unchecked(0) }
    }

    pub fn ch_idx(&self) -> u16 {
        unsafe {
            let mut data: [u8; 2] = [core::mem::MaybeUninit::uninit().assume_init(); 2];
            data.as_mut_ptr().copy_from(self.data.get_unchecked(1), 2);
            // TODO SPEC endianess
            u16::from_le_bytes(data)
        }
    }
    pub fn rxlev(&self) -> u8 {
        unsafe { *self.data.get_unchecked(3) }
    }
    pub fn lb(&self) -> u8 {
        unsafe { *self.data.get_unchecked(4) }
    }
    pub fn snr(&self) -> u8 {
        unsafe { *self.data.get_unchecked(5) }
    }
    pub fn status(&self) -> u8 {
        unsafe { *self.data.get_unchecked(6) }
    }
    pub fn token(&self) -> u8 {
        unsafe { *self.data.get_unchecked(7) }
    }
    pub fn seq(&self) -> u8 {
        unsafe { *self.data.get_unchecked(8) }
    }
    pub fn resp_to(&self) -> u8 {
        unsafe { *self.data.get_unchecked(9) }
    }
    pub fn addressee(&self) -> EncodedAddressee<'data> {
        unsafe { AddresseeRef::start_decoding_unchecked(self.data.get_unchecked(10..)) }
    }

    pub fn addressee_with_nls_state(&self) -> EncodedAddresseeWithNlsState<'data> {
        unsafe { EncodedAddresseeWithNlsState::new(self.data.get_unchecked(10..)) }
    }
}

impl<'data> EncodedData<'data> for EncodedDash7InterfaceStatus<'data> {
    type DecodedData = Dash7InterfaceStatusRef<'data>;

    unsafe fn new(data: &'data [u8]) -> Self {
        Self { data }
    }

    unsafe fn expected_size(&self) -> usize {
        10 + self.addressee_with_nls_state().expected_size()
    }

    fn smaller_than(&self, data_size: usize) -> Result<usize, usize> {
        let mut size = 11;
        if data_size < size {
            return Err(size);
        }
        size += unsafe { self.addressee_with_nls_state().expected_size() };
        size -= 1;
        if data_size < size {
            return Err(size);
        }
        Ok(size)
    }

    fn complete_decoding(&self) -> WithByteSize<Dash7InterfaceStatusRef<'data>> {
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
            assert_eq!(
                ret.addressee_with_nls_state.addressee(),
                &decoder.addressee().complete_decoding().item
            );
            assert_eq!(expected_size, size);
            assert_eq!(unsafe { decoder.expected_size() }, size);
            assert_eq!(decoder.smaller_than(data.len()).unwrap(), size);
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

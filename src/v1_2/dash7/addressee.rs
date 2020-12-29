/// Maximum byte size of an encoded `ReadFileData`
pub const MAX_SIZE: usize = 2 + 8;

/// Required size of a data buffer to determine the size of a resulting
/// decoded object
pub const HEADER_SIZE: usize = 2;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct AccessClass(pub u8);

impl AccessClass {
    pub fn u8(self) -> u8 {
        let Self(n) = self;
        n
    }

    pub fn specifier(self) -> u8 {
        self.u8() >> 4
    }

    pub fn mask(self) -> u8 {
        self.u8() & 0x0F
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum NlsMethod {
    None = 0,
    AesCtr = 1,
    AesCbcMac128 = 2,
    AesCbcMac64 = 3,
    AesCbcMac32 = 4,
    AesCcm128 = 5,
    AesCcm64 = 6,
    AesCcm32 = 7,
    Rfu8 = 8,
    Rfu9 = 9,
    Rfu10 = 10,
    Rfu11 = 11,
    Rfu12 = 12,
    Rfu13 = 13,
    Rfu14 = 14,
    Rfu15 = 15,
}

// TODO These enum constant stuff very surely has an impact on the final
// binary size because it enforces a branch construct where it could just
// be a simple cast or even no operation at all.
impl NlsMethod {
    /// # Safety
    /// You are responsible for checking that `n` < 16.
    pub unsafe fn from_unchecked(n: u8) -> Self {
        match n {
            0 => Self::None,
            1 => Self::AesCtr,
            2 => Self::AesCbcMac128,
            3 => Self::AesCbcMac64,
            4 => Self::AesCbcMac32,
            5 => Self::AesCcm128,
            6 => Self::AesCcm64,
            7 => Self::AesCcm32,
            8 => Self::Rfu8,
            9 => Self::Rfu9,
            10 => Self::Rfu10,
            11 => Self::Rfu11,
            12 => Self::Rfu12,
            13 => Self::Rfu13,
            14 => Self::Rfu14,
            15 => Self::Rfu15,
            _ => Self::None,
        }
    }
}

// TODO All those intermediary types probably an impact on binary size and performance.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum AddresseeIdentifierType {
    Nbid = 0,
    Noid = 1,
    Uid = 2,
    Vid = 3,
}

impl AddresseeIdentifierType {
    pub fn size(&self) -> usize {
        match self {
            Self::Nbid => 1,
            Self::Noid => 0,
            Self::Uid => 8,
            Self::Vid => 2,
        }
    }

    /// # Safety
    /// You are responsible for checking that `n` < 4.
    pub unsafe fn from_unchecked(n: u8) -> Self {
        match n {
            0 => Self::Nbid,
            1 => Self::Noid,
            2 => Self::Uid,
            3 => Self::Vid,
            _ => Self::Nbid,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum AddresseeIdentifier {
    Nbid(u8),
    Noid,
    Uid([u8; 8]),
    Vid([u8; 2]),
}

impl AddresseeIdentifier {
    pub fn id_type(&self) -> AddresseeIdentifierType {
        match self {
            Self::Nbid(_) => AddresseeIdentifierType::Nbid,
            Self::Noid => AddresseeIdentifierType::Noid,
            Self::Uid(_) => AddresseeIdentifierType::Uid,
            Self::Vid(_) => AddresseeIdentifierType::Vid,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Addressee {
    pub nls_method: NlsMethod,
    pub access_class: AccessClass,
    pub identifier: AddresseeIdentifier,
}

impl Addressee {
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
        let id_type = self.identifier.id_type();
        *out.add(0) = (id_type as u8) << 4 | (self.nls_method as u8);
        *out.add(1) = self.access_class.u8();
        match &self.identifier {
            AddresseeIdentifier::Nbid(n) => *out.add(2) = *n,
            AddresseeIdentifier::Noid => (),
            AddresseeIdentifier::Uid(uid) => out.add(2).copy_from(uid.as_ptr(), uid.len()),
            AddresseeIdentifier::Vid(vid) => out.add(2).copy_from(vid.as_ptr(), vid.len()),
        }

        2 + id_type.size()
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
        2 + self.identifier.id_type().size()
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
    /// `data.len()` >= [`decodable.size()`](struct.DecodableAddressee.html#method.size).
    ///
    /// Failing that might result in reading and interpreting data outside the given
    /// array (depending on what is done with the resulting object).
    pub const unsafe fn start_decoding_ptr<'data>(data: *const u8) -> DecodableAddressee<'data> {
        DecodableAddressee::from_ptr(data)
    }

    /// Creates a decodable item without checking the data size.
    ///
    /// # Safety
    /// You are to check that:
    /// - The first byte contains this action's querycode.
    /// - The decoded data is bigger than the expected size of the `decodable` object.
    /// Meaning that given the resulting decodable object `decodable`:
    /// `data.len()` >= [`decodable.size()`](struct.DecodableAddressee.html#method.size).
    ///
    /// Failing that might result in reading and interpreting data outside the given
    /// array (depending on what is done with the resulting object).
    pub const unsafe fn start_decoding_unchecked(data: &[u8]) -> DecodableAddressee {
        DecodableAddressee::new(data)
    }

    /// Creates a decodable item.
    ///
    /// This decodable item allows each parts of the item independently.
    ///
    /// # Errors
    /// - Fails if data is less than 2 bytes.
    /// - Fails if data is smaller then the decoded expected size.
    ///
    /// Returns the number of bytes required to continue decoding.
    pub fn start_decoding(data: &[u8]) -> Result<DecodableAddressee, usize> {
        if data.len() < HEADER_SIZE {
            return Err(HEADER_SIZE);
        }
        let ret = unsafe { Self::start_decoding_unchecked(data) };
        let ret_size = ret.size();
        if data.len() < ret_size {
            return Err(ret_size);
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
    /// # Safety
    /// You are to check that:
    /// - The first byte contains this action's querycode.
    /// - The data is not empty.
    /// - The resulting size of the data consumed is smaller than the size of the
    /// decoded data.
    ///
    /// Failing that might result in reading and interpreting data outside the given
    /// array (depending on what is done with the resulting object).
    pub unsafe fn decode_ptr(data: *const u8) -> (Self, usize) {
        Self::start_decoding_ptr(data).complete_decoding()
    }

    /// Decodes the Item from bytes.
    ///
    /// Returns the decoded data and the number of bytes consumed to produce it.
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
    pub unsafe fn decode_unchecked(data: &[u8]) -> (Self, usize) {
        Self::start_decoding_unchecked(data).complete_decoding()
    }

    /// Decodes the item from bytes.
    ///
    /// On success, returns the decoded data and the number of bytes consumed
    /// to produce it.
    ///
    /// # Errors
    /// - Fails if data is less than 2 bytes.
    /// - Fails if data is smaller then the decoded expected size.
    ///
    /// Returns the number of bytes required to continue decoding.
    pub fn decode(data: &[u8]) -> Result<(Self, usize), usize> {
        match Self::start_decoding(data) {
            Ok(v) => Ok(v.complete_decoding()),
            Err(e) => Err(e),
        }
    }
}

pub struct DecodableAddressee<'data> {
    data: *const u8,
    data_life: core::marker::PhantomData<&'data ()>,
}

impl<'data> DecodableAddressee<'data> {
    const fn new(data: &'data [u8]) -> Self {
        Self {
            data: data.as_ptr(),
            data_life: core::marker::PhantomData,
        }
    }

    const fn from_ptr(data: *const u8) -> Self {
        Self {
            data,
            data_life: core::marker::PhantomData,
        }
    }

    /// Decodes the size of the Item in bytes
    pub fn size(&self) -> usize {
        2 + self.id_type().size()
    }

    pub fn id_type(&self) -> AddresseeIdentifierType {
        unsafe { AddresseeIdentifierType::from_unchecked(*self.data.add(0) >> 4 & 0x07) }
    }

    pub fn nls_method(&self) -> NlsMethod {
        unsafe { NlsMethod::from_unchecked(*self.data.add(0) & 0x0F) }
    }

    pub fn access_class(&self) -> AccessClass {
        unsafe { AccessClass(*self.data.add(1)) }
    }

    pub fn identifier(&self) -> AddresseeIdentifier {
        unsafe {
            match self.id_type() {
                AddresseeIdentifierType::Nbid => AddresseeIdentifier::Nbid(*self.data.add(2)),
                AddresseeIdentifierType::Noid => AddresseeIdentifier::Noid,
                AddresseeIdentifierType::Uid => {
                    let mut data: [u8; 8] = [core::mem::MaybeUninit::uninit().assume_init(); 8];
                    data.as_mut_ptr().copy_from(self.data.add(2), 8);
                    AddresseeIdentifier::Uid(data)
                }
                AddresseeIdentifierType::Vid => {
                    let mut data: [u8; 2] = [core::mem::MaybeUninit::uninit().assume_init(); 2];
                    data.as_mut_ptr().copy_from(self.data.add(2), 2);
                    AddresseeIdentifier::Vid(data)
                }
            }
        }
    }

    /// Fully decode the Item
    ///
    /// Returns the decoded data and the number of bytes consumed to produce it.
    pub fn complete_decoding(&self) -> (Addressee, usize) {
        let id_type = self.id_type();
        let identifier = unsafe {
            match id_type {
                AddresseeIdentifierType::Nbid => AddresseeIdentifier::Nbid(*self.data.add(2)),
                AddresseeIdentifierType::Noid => AddresseeIdentifier::Noid,
                AddresseeIdentifierType::Uid => {
                    let mut data: [u8; 8] = [core::mem::MaybeUninit::uninit().assume_init(); 8];
                    data.as_mut_ptr().copy_from(self.data.add(2), 8);
                    AddresseeIdentifier::Uid(data)
                }
                AddresseeIdentifierType::Vid => {
                    let mut data: [u8; 2] = [core::mem::MaybeUninit::uninit().assume_init(); 2];
                    data.as_mut_ptr().copy_from(self.data.add(2), 2);
                    AddresseeIdentifier::Vid(data)
                }
            }
        };
        (
            Addressee {
                nls_method: self.nls_method(),
                access_class: self.access_class(),
                identifier,
            },
            2 + id_type.size(),
        )
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::unwrap_in_result, clippy::panic, clippy::expect_used)]
    use super::*;

    #[test]
    fn known() {
        fn test(op: Addressee, data: &[u8]) {
            // Test op.encode_in() == data
            let mut encoded = [0_u8; 64];
            let size = op.encode_in(&mut encoded).unwrap();
            assert_eq!(size, data.len());
            assert_eq!(&encoded[..size], data);

            // Test decode(data) == op
            let (ret, size) = Addressee::decode(data).unwrap();
            assert_eq!(size, data.len());
            assert_eq!(ret, op);

            // Test partial_decode == op
            let decoder = Addressee::start_decoding(data).unwrap();
            assert_eq!(ret.identifier.id_type(), decoder.id_type());
            assert_eq!(size, decoder.size());
            assert_eq!(
                op,
                Addressee {
                    nls_method: decoder.nls_method(),
                    access_class: decoder.access_class(),
                    identifier: decoder.identifier(),
                }
            );
        }
        test(
            Addressee {
                nls_method: NlsMethod::None,
                access_class: AccessClass(0x01),
                identifier: AddresseeIdentifier::Nbid(4),
            },
            &[0x00, 0x01, 0x04],
        );
        test(
            Addressee {
                nls_method: NlsMethod::AesCtr,
                access_class: AccessClass(0x21),
                identifier: AddresseeIdentifier::Noid,
            },
            &[0x11, 0x21],
        );
        test(
            Addressee {
                nls_method: NlsMethod::AesCcm64,
                access_class: AccessClass(0xE1),
                identifier: AddresseeIdentifier::Uid([
                    0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
                ]),
            },
            &[0x26, 0xE1, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77],
        );
        test(
            Addressee {
                nls_method: NlsMethod::AesCbcMac64,
                access_class: AccessClass(0x71),
                identifier: AddresseeIdentifier::Vid([0xCA, 0xFE]),
            },
            &[0x33, 0x71, 0xCA, 0xFE],
        );
    }

    #[test]
    fn consistence() {
        const TOT_SIZE: usize = 2 + 8;
        let op = Addressee {
            nls_method: NlsMethod::AesCcm64,
            access_class: AccessClass(0xE1),
            identifier: AddresseeIdentifier::Uid([0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77]),
        };

        // Test decode(op.encode_in()) == op
        let mut encoded = [0_u8; TOT_SIZE];
        let size_encoded = op.encode_in(&mut encoded).unwrap();
        let (ret, size_decoded) = Addressee::decode(&encoded).unwrap();
        assert_eq!(size_encoded, size_decoded);
        assert_eq!(ret, op);

        // Test decode(data).encode_in() == data
        let mut encoded2 = [0_u8; TOT_SIZE];
        let size_encoded2 = op.encode_in(&mut encoded2).unwrap();
        assert_eq!(size_encoded, size_encoded2);
        assert_eq!(encoded2[..size_encoded2], encoded[..size_encoded]);
    }
}

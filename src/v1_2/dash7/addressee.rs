use crate::decodable::{Decodable, EncodedData, SizeError, WithByteSize};
use crate::encodable::Encodable;

/// Maximum byte size of an encoded `an Addressee`
pub const MAX_SIZE: usize = 2 + 8;

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

#[cfg_attr(feature = "repr_c", repr(C))]
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

#[cfg_attr(feature = "repr_c", repr(C))]
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

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum AddresseeIdentifierRef<'item> {
    Nbid(u8),
    Noid,
    Uid(&'item [u8; 8]),
    Vid(&'item [u8; 2]),
}

impl<'item> AddresseeIdentifierRef<'item> {
    pub fn id_type(&self) -> AddresseeIdentifierType {
        match self {
            Self::Nbid(_) => AddresseeIdentifierType::Nbid,
            Self::Noid => AddresseeIdentifierType::Noid,
            Self::Uid(_) => AddresseeIdentifierType::Uid,
            Self::Vid(_) => AddresseeIdentifierType::Vid,
        }
    }

    pub fn size(&self) -> usize {
        match self {
            Self::Nbid(_) => 1,
            Self::Noid => 0,
            Self::Uid(_) => 8,
            Self::Vid(_) => 2,
        }
    }

    pub fn to_owned(&self) -> AddresseeIdentifier {
        match self {
            Self::Nbid(n) => AddresseeIdentifier::Nbid(*n),
            Self::Noid => AddresseeIdentifier::Noid,
            Self::Uid(uid) => AddresseeIdentifier::Uid(**uid),
            Self::Vid(vid) => AddresseeIdentifier::Vid(**vid),
        }
    }
}

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum AddresseeIdentifier {
    Nbid(u8),
    Noid,
    Uid([u8; 8]),
    Vid([u8; 2]),
}

impl AddresseeIdentifier {
    pub fn as_ref(&self) -> AddresseeIdentifierRef {
        match self {
            Self::Nbid(n) => AddresseeIdentifierRef::Nbid(*n),
            Self::Noid => AddresseeIdentifierRef::Noid,
            Self::Uid(uid) => AddresseeIdentifierRef::Uid(uid),
            Self::Vid(vid) => AddresseeIdentifierRef::Vid(vid),
        }
    }
}

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct AddresseeRef<'item> {
    pub nls_method: NlsMethod,
    pub access_class: AccessClass,
    pub identifier: AddresseeIdentifierRef<'item>,
}

impl<'data> Encodable for AddresseeRef<'data> {
    unsafe fn encode_in_ptr(&self, out: *mut u8) -> usize {
        let id_type = self.identifier.id_type();
        *out.add(0) = (id_type as u8) << 4 | (self.nls_method as u8);
        *out.add(1) = self.access_class.u8();
        match &self.identifier {
            AddresseeIdentifierRef::Nbid(n) => *out.add(2) = *n,
            AddresseeIdentifierRef::Noid => (),
            AddresseeIdentifierRef::Uid(uid) => out.add(2).copy_from(uid.as_ptr(), uid.len()),
            AddresseeIdentifierRef::Vid(vid) => out.add(2).copy_from(vid.as_ptr(), vid.len()),
        }

        2 + id_type.size()
    }

    fn size(&self) -> usize {
        2 + self.identifier.id_type().size()
    }
}

impl<'item> AddresseeRef<'item> {
    pub fn to_owned(&self) -> Addressee {
        Addressee {
            nls_method: self.nls_method,
            access_class: self.access_class,
            identifier: self.identifier.to_owned(),
        }
    }
}

pub struct EncodedAddressee<'data> {
    data: &'data [u8],
}

impl<'data> EncodedAddressee<'data> {
    pub fn id_type(&self) -> AddresseeIdentifierType {
        unsafe { AddresseeIdentifierType::from_unchecked(*self.data.get_unchecked(0) >> 4 & 0x07) }
    }

    pub fn nls_method(&self) -> NlsMethod {
        unsafe { NlsMethod::from_unchecked(*self.data.get_unchecked(0) & 0x0F) }
    }

    pub fn access_class(&self) -> AccessClass {
        unsafe { AccessClass(*self.data.get_unchecked(1)) }
    }

    pub fn identifier(&self) -> AddresseeIdentifierRef<'data> {
        unsafe {
            match self.id_type() {
                AddresseeIdentifierType::Nbid => {
                    AddresseeIdentifierRef::Nbid(*self.data.get_unchecked(2))
                }
                AddresseeIdentifierType::Noid => AddresseeIdentifierRef::Noid,
                AddresseeIdentifierType::Uid => {
                    let data = &*(self.data.get_unchecked(2..).as_ptr() as *const [u8; 8]);
                    AddresseeIdentifierRef::Uid(data)
                }
                AddresseeIdentifierType::Vid => {
                    let data = &*(self.data.get_unchecked(2..).as_ptr() as *const [u8; 2]);
                    AddresseeIdentifierRef::Vid(data)
                }
            }
        }
    }

    /// # Safety
    /// You are to warrant, somehow, that the input byte array contains a complete item.
    /// Else this might result in out of bound reads, and absurd results.
    pub unsafe fn size_unchecked(&self) -> usize {
        2 + self.id_type().size()
    }
}

impl<'data> EncodedData<'data> for EncodedAddressee<'data> {
    type DecodedData = AddresseeRef<'data>;
    unsafe fn new(data: &'data [u8]) -> Self {
        Self { data }
    }

    fn size(&self) -> Result<usize, SizeError> {
        let mut size = 1;
        let data_size = self.data.len();
        if data_size < size {
            return Err(SizeError::MissingBytes);
        }
        size = unsafe { self.size_unchecked() };
        if data_size < size {
            return Err(SizeError::MissingBytes);
        }
        Ok(size)
    }

    fn complete_decoding(&self) -> WithByteSize<AddresseeRef<'data>> {
        let identifier = self.identifier();
        WithByteSize {
            item: AddresseeRef {
                nls_method: self.nls_method(),
                access_class: self.access_class(),
                identifier,
            },
            byte_size: 2 + identifier.size(),
        }
    }
}

impl<'data> Decodable<'data> for AddresseeRef<'data> {
    type Data = EncodedAddressee<'data>;
}

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Addressee {
    pub nls_method: NlsMethod,
    pub access_class: AccessClass,
    pub identifier: AddresseeIdentifier,
}

impl Addressee {
    pub fn as_ref(&self) -> AddresseeRef {
        AddresseeRef {
            nls_method: self.nls_method,
            access_class: self.access_class,
            identifier: self.identifier.as_ref(),
        }
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::unwrap_in_result, clippy::panic, clippy::expect_used)]
    use super::*;

    #[test]
    fn known() {
        fn test(op: AddresseeRef, data: &[u8]) {
            // Test op.encode_in() == data
            let mut encoded = [0_u8; 64];
            let size = op.encode_in(&mut encoded).unwrap();
            assert_eq!(size, data.len());
            assert_eq!(&encoded[..size], data);

            // Test decode(data) == op
            let WithByteSize {
                item: ret,
                byte_size: size,
            } = AddresseeRef::decode(data).unwrap();
            assert_eq!(size, data.len());
            assert_eq!(ret, op);

            // Test partial_decode == op
            let WithByteSize {
                item: decoder,
                byte_size: expected_size,
            } = AddresseeRef::start_decoding(data).unwrap();
            assert_eq!(ret.identifier.id_type(), decoder.id_type());
            assert_eq!(expected_size, size);
            assert_eq!(unsafe { decoder.size_unchecked() }, size);
            assert_eq!(decoder.size().unwrap(), size);
            assert_eq!(
                op,
                AddresseeRef {
                    nls_method: decoder.nls_method(),
                    access_class: decoder.access_class(),
                    identifier: decoder.identifier(),
                }
            );
        }
        test(
            AddresseeRef {
                nls_method: NlsMethod::None,
                access_class: AccessClass(0x01),
                identifier: AddresseeIdentifierRef::Nbid(4),
            },
            &[0x00, 0x01, 0x04],
        );
        test(
            AddresseeRef {
                nls_method: NlsMethod::AesCtr,
                access_class: AccessClass(0x21),
                identifier: AddresseeIdentifierRef::Noid,
            },
            &[0x11, 0x21],
        );
        test(
            AddresseeRef {
                nls_method: NlsMethod::AesCcm64,
                access_class: AccessClass(0xE1),
                identifier: AddresseeIdentifierRef::Uid(&[
                    0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
                ]),
            },
            &[0x26, 0xE1, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77],
        );
        test(
            AddresseeRef {
                nls_method: NlsMethod::AesCbcMac64,
                access_class: AccessClass(0x71),
                identifier: AddresseeIdentifierRef::Vid(&[0xCA, 0xFE]),
            },
            &[0x33, 0x71, 0xCA, 0xFE],
        );
    }

    #[test]
    fn consistence() {
        const TOT_SIZE: usize = 2 + 8;
        let op = AddresseeRef {
            nls_method: NlsMethod::AesCcm64,
            access_class: AccessClass(0xE1),
            identifier: AddresseeIdentifierRef::Uid(&[
                0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
            ]),
        };

        // Test decode(op.encode_in()) == op
        let mut encoded = [0_u8; TOT_SIZE];
        let size_encoded = op.encode_in(&mut encoded).unwrap();
        let WithByteSize {
            item: ret,
            byte_size: size_decoded,
        } = AddresseeRef::decode(&encoded).unwrap();
        assert_eq!(size_encoded, size_decoded);
        assert_eq!(ret, op);

        // Test decode(data).encode_in() == data
        let mut encoded2 = [0_u8; TOT_SIZE];
        let size_encoded2 = op.encode_in(&mut encoded2).unwrap();
        assert_eq!(size_encoded, size_encoded2);
        assert_eq!(encoded2[..size_encoded2], encoded[..size_encoded]);
    }
}

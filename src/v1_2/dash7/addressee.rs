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
    pub fn encoded_size(&self) -> usize {
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
pub enum AddresseeIdentifierRef<'data> {
    Nbid(u8),
    Noid,
    Uid(&'data [u8; 8]),
    Vid(&'data [u8; 2]),
}

impl<'data> AddresseeIdentifierRef<'data> {
    pub fn id_type(&self) -> AddresseeIdentifierType {
        match self {
            Self::Nbid(_) => AddresseeIdentifierType::Nbid,
            Self::Noid => AddresseeIdentifierType::Noid,
            Self::Uid(_) => AddresseeIdentifierType::Uid,
            Self::Vid(_) => AddresseeIdentifierType::Vid,
        }
    }

    pub fn encoded_size(&self) -> usize {
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
pub struct AddresseeRef<'data> {
    pub nls_method: NlsMethod,
    pub access_class: AccessClass,
    pub identifier: AddresseeIdentifierRef<'data>,
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

        2 + id_type.encoded_size()
    }

    fn encoded_size(&self) -> usize {
        2 + self.identifier.id_type().encoded_size()
    }
}

impl<'data> AddresseeRef<'data> {
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
    /// You have to warrant that somehow that there is enough byte to decode the encoded size.
    /// If you fail to do so, out of bound bytes will be read, and an absurd value will be
    /// returned.
    pub unsafe fn encoded_size_unchecked(&self) -> usize {
        2 + self.id_type().encoded_size()
    }
}

impl<'data> EncodedData<'data> for EncodedAddressee<'data> {
    type SourceData = &'data [u8];
    type DecodedData = AddresseeRef<'data>;

    unsafe fn new(data: Self::SourceData) -> Self {
        Self { data }
    }

    fn encoded_size(&self) -> Result<usize, SizeError> {
        let mut size = 1;
        let data_size = self.data.len();
        if data_size < size {
            return Err(SizeError::MissingBytes);
        }
        size = unsafe { self.encoded_size_unchecked() };
        if data_size < size {
            return Err(SizeError::MissingBytes);
        }
        Ok(size)
    }

    fn complete_decoding(&self) -> WithByteSize<Self::DecodedData> {
        let identifier = self.identifier();
        WithByteSize {
            item: AddresseeRef {
                nls_method: self.nls_method(),
                access_class: self.access_class(),
                identifier,
            },
            byte_size: 2 + identifier.encoded_size(),
        }
    }
}

pub struct EncodedAddresseeMut<'data> {
    data: &'data mut [u8],
}

crate::make_downcastable!(EncodedAddresseeMut, EncodedAddressee);

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum AddresseeSetNlsMethodError {
    /// The requested nls method change implies an nls_state field size change. Thus is impossible.
    NlsStateMismatch,
}

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum AddresseeSetIdentifierError {
    /// The given identifier byte size does not match the encoded identifier byte size.
    IdMismatch,
}

impl<'data> EncodedAddresseeMut<'data> {
    pub fn id_type(&self) -> AddresseeIdentifierType {
        self.as_ref().id_type()
    }

    pub fn nls_method(&self) -> NlsMethod {
        self.as_ref().nls_method()
    }

    pub fn access_class(&self) -> AccessClass {
        self.as_ref().access_class()
    }

    pub fn identifier(&self) -> AddresseeIdentifierRef<'data> {
        self.as_ref().identifier()
    }

    /// # Safety
    /// You have to warrant that somehow that there is enough byte to decode the encoded size.
    /// If you fail to do so, out of bound bytes will be read, and an absurd value will be
    /// returned.
    pub unsafe fn encoded_size_unchecked(&self) -> usize {
        self.as_ref().encoded_size_unchecked()
    }

    /// # Safety
    /// Changing this breaks the coherence of the data.
    /// Make sure you change the identifier part accordingly.
    pub unsafe fn set_id_type(&mut self, ty: AddresseeIdentifierType) {
        *self.data.get_unchecked_mut(0) &= 0x0F;
        *self.data.get_unchecked_mut(0) |= (ty as u8) << 4;
    }

    /// # Safety
    /// Changing this can break the coherence of the data.
    /// Make sure that the nls_state size stays the same whatever value you replace the nls method
    /// with.
    pub unsafe fn set_nls_method_unchecked(&mut self, nls_method: NlsMethod) {
        *self.data.get_unchecked_mut(0) &= 0xF0;
        *self.data.get_unchecked_mut(0) |= nls_method as u8;
    }

    /// # Errors
    /// Fails if the nls method change implies a nls_state field byte size change.
    pub fn set_nls_method(
        &mut self,
        nls_method: NlsMethod,
    ) -> Result<(), AddresseeSetNlsMethodError> {
        match self.nls_method() {
            NlsMethod::None => match nls_method {
                NlsMethod::None => (),
                _ => return Err(AddresseeSetNlsMethodError::NlsStateMismatch),
            },
            _ => {
                if let NlsMethod::None = &nls_method {
                    return Err(AddresseeSetNlsMethodError::NlsStateMismatch);
                }
            }
        }
        unsafe { self.set_nls_method_unchecked(nls_method) };
        Ok(())
    }

    pub fn set_access_class(&mut self, access_class: AccessClass) {
        unsafe {
            *self.data.get_unchecked_mut(1) = access_class.u8();
        }
    }

    /// # Safety
    pub unsafe fn set_identifier_unchecked(&mut self, identifier: AddresseeIdentifierRef<'data>) {
        match identifier {
            AddresseeIdentifierRef::Nbid(n) => {
                *self.data.get_unchecked_mut(2) = n;
            }
            AddresseeIdentifierRef::Noid => (),
            AddresseeIdentifierRef::Uid(id) => {
                self.data.get_unchecked_mut(2..).copy_from_slice(id);
            }
            AddresseeIdentifierRef::Vid(id) => {
                self.data.get_unchecked_mut(2..).copy_from_slice(id);
            }
        }
    }

    /// # Errors
    /// Returns an error if the given identifier is not of the same type as the encoded one,
    /// because it would imply an encoded size mismatch.
    pub fn set_identifier(
        &mut self,
        identifier: AddresseeIdentifierRef<'data>,
    ) -> Result<(), AddresseeSetIdentifierError> {
        if self.id_type() != identifier.id_type() {
            return Err(AddresseeSetIdentifierError::IdMismatch);
        }
        unsafe { self.set_identifier_unchecked(identifier) };
        Ok(())
    }
}

impl<'data> EncodedData<'data> for EncodedAddresseeMut<'data> {
    type SourceData = &'data mut [u8];
    type DecodedData = AddresseeRef<'data>;

    unsafe fn new(data: Self::SourceData) -> Self {
        Self { data }
    }

    fn encoded_size(&self) -> Result<usize, SizeError> {
        self.as_ref().encoded_size()
    }

    fn complete_decoding(&self) -> WithByteSize<Self::DecodedData> {
        self.as_ref().complete_decoding()
    }
}

impl<'data> Decodable<'data> for AddresseeRef<'data> {
    type Data = EncodedAddressee<'data>;
    type DataMut = EncodedAddresseeMut<'data>;
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
            assert_eq!(unsafe { decoder.encoded_size_unchecked() }, size);
            assert_eq!(decoder.encoded_size().unwrap(), size);
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

// TODO ALP_SPEC: The encoding of the value is not specified!
// Big endian at bit and byte level probably, but it has to be specified!
use crate::decodable::{Decodable, EncodedData, SizeError, WithByteSize};
use crate::encodable::Encodable;

/// Maximum value writable in a Varint encodable on 1 byte
pub const U8_MAX: u8 = 0x3F;
/// Maximum value writable in a Varint encodable on 2 byte
pub const U16_MAX: u16 = 0x3F_FF;
/// Maximum value writable in a Varint encodable on 3 byte
pub const U24_MAX: u32 = 0x3F_FF_FF;
/// Maximum value writable in a Varint encodable on 4 byte
pub const U32_MAX: u32 = 0x3F_FF_FF_FF;

/// Maximum byte size of an encoded Varint
pub const MAX_SIZE: usize = 4;

/// Represents a variable integer as described by the Dash7 ALP specification.
#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Varint {
    value: u32,
}

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum VarintError {
    ValueTooBig,
}

impl Varint {
    /// Create a struct representing a Varint
    ///
    /// # Safety
    /// Only call this on u32 that are less than [`MAX`](constant.MAX.html).
    ///
    /// Calling this on a large integer will result in a structure
    /// containing an impossible value. Plus, trying to encode the
    /// wrong value will result in the encoding of another lower
    /// value (the value will be masked upon encoding).
    pub const unsafe fn new_unchecked(value: u32) -> Self {
        Self { value }
    }

    /// Create a struct representing a Varint
    ///
    /// # Errors
    /// Fails if the value is bigger than [`MAX`](constant.MAX.html)
    pub const fn new(value: u32) -> Result<Self, VarintError> {
        if value > U32_MAX {
            Err(VarintError::ValueTooBig)
        } else {
            unsafe { Ok(Self::new_unchecked(value)) }
        }
    }

    /// Returns the contained value.
    ///
    /// The internal value is inaccessible to prevent unchecked modifications
    /// that would result in an invalid value.
    pub const fn u32(&self) -> u32 {
        self.value
    }

    /// # Safety
    /// Casting a u32 to a usize might overflow.
    /// Check your target architecture.
    pub const unsafe fn usize(&self) -> usize {
        self.value as usize
    }

    /// Encode the value into a single byte array.
    ///
    /// # Safety
    /// You are to warrant that the value does not exceed [`U8_MAX`](constant.U8_MAX.html)
    pub const unsafe fn encode_as_u8(value: u8) -> [u8; 1] {
        [value & 0x3F]
    }

    /// Encode the value into a two bytes array.
    ///
    /// Event though overkill, you technically can encode a small integer (such as 0) in
    /// this fixed sized array.
    ///
    /// # Safety
    /// You are to warrant that the value does not exceed [`U16_MAX`](constant.U16_MAX.html)
    pub const unsafe fn encode_as_u16(value: u16) -> [u8; 2] {
        [((value >> 8) & 0x3F) as u8, (value & 0xFF) as u8]
    }

    /// Encode the value into a three bytes array.
    ///
    /// Event though overkill, you technically can encode a small integer (such as 0) in
    /// this fixed sized array.
    ///
    /// # Safety
    /// You are to warrant that the value does not exceed [`U24_MAX`](constant.U24_MAX.html)
    pub const unsafe fn encode_as_u24(value: u32) -> [u8; 3] {
        [
            ((value >> 16) & 0x3F) as u8,
            ((value >> 8) & 0xFF) as u8,
            (value & 0xFF) as u8,
        ]
    }

    /// Encode the value into a four bytes array.
    ///
    /// Event though overkill, you technically can encode a small integer (such as 0) in
    /// this fixed sized array.
    ///
    /// # Safety
    /// You are to warrant that the value does not exceed [`U32_MAX`](constant.U32_MAX.html)
    pub const unsafe fn encode_as_u32(value: u32) -> [u8; 4] {
        [
            ((value >> 24) & 0x3F) as u8,
            ((value >> 8) & 0xFF) as u8,
            ((value >> 8) & 0xFF) as u8,
            (value & 0xFF) as u8,
        ]
    }
}

impl Encodable for Varint {
    unsafe fn encode_in_ptr(&self, out: *mut u8) -> usize {
        let size = self.encoded_size();
        match size {
            1 => *out.add(0) = (self.value & 0x3F) as u8,
            2 => {
                *out.add(0) = ((self.value >> 8) & 0x3F) as u8;
                *out.add(1) = (self.value & 0xFF) as u8;
            }
            3 => {
                *out.add(0) = ((self.value >> 16) & 0x3F) as u8;
                *out.add(1) = ((self.value >> 8) & 0xFF) as u8;
                *out.add(2) = (self.value & 0xFF) as u8;
            }
            4 => {
                *out.add(0) = ((self.value >> 24) & 0x3F) as u8;
                *out.add(1) = ((self.value >> 16) & 0xFF) as u8;
                *out.add(2) = ((self.value >> 8) & 0xFF) as u8;
                *out.add(3) = (self.value & 0xFF) as u8;
            }
            _ => (),
        }
        *out.add(0) |= (size as u8 - 1) << 6;
        size
    }

    fn encoded_size(&self) -> usize {
        let n = self.value;
        if n <= 0x3F {
            1
        } else if n <= 0x3F_FF {
            2
        } else if n <= 0x3F_FF_FF {
            3
        } else {
            4
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct EncodedVarint<'data> {
    data: &'data [u8],
}

impl<'data> EncodedVarint<'data> {
    /// # Safety
    /// You have to warrant that somehow that there is enough byte to decode the encoded size.
    /// If you fail to do so, out of bound bytes will be read, and an absurd value will be
    /// returned.
    pub unsafe fn encoded_size_unchecked(&self) -> usize {
        ((self.data.get_unchecked(0) & 0xC0) >> 6) as usize + 1
    }
}

impl<'data> EncodedData<'data> for EncodedVarint<'data> {
    type SourceData = &'data [u8];
    type DecodedData = Varint;
    unsafe fn new(data: Self::SourceData) -> Self {
        Self { data }
    }

    fn encoded_size(&self) -> Result<usize, SizeError> {
        let mut size = 1;
        if self.data.len() < size {
            return Err(SizeError::MissingBytes);
        }
        size = unsafe { self.encoded_size_unchecked() };
        if self.data.len() < size {
            return Err(SizeError::MissingBytes);
        }
        Ok(size)
    }

    fn complete_decoding(&self) -> WithByteSize<Varint> {
        unsafe {
            let size = self.encoded_size_unchecked();
            let data = &self.data;
            let ret = Varint::new_unchecked(match size {
                1 => (*data.get_unchecked(0) & 0x3F) as u32,
                2 => {
                    (((*data.get_unchecked(0) & 0x3F) as u32) << 8) + *data.get_unchecked(1) as u32
                }
                3 => {
                    (((*data.get_unchecked(0) & 0x3F) as u32) << 16)
                        + ((*data.get_unchecked(1) as u32) << 8)
                        + *data.get_unchecked(2) as u32
                }
                4 => {
                    (((*data.get_unchecked(0) & 0x3F) as u32) << 24)
                        + ((*data.get_unchecked(1) as u32) << 16)
                        + ((*data.get_unchecked(2) as u32) << 8)
                        + *data.get_unchecked(3) as u32
                }
                // This is bad and incorrect. But size should mathematically never evaluate to this
                // case. Let's just hope the size method is not broken.
                _ => 0,
            });
            WithByteSize {
                item: ret,
                byte_size: size,
            }
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
pub struct EncodedVarintMut<'data> {
    data: &'data mut [u8],
}

crate::make_downcastable!(EncodedVarintMut, EncodedVarint);

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum VarintSetError {
    /// The encoded size of the given varint does not match the size of the currently encoded
    /// varint.
    SizeMismatch,
}

impl<'data> EncodedVarintMut<'data> {
    /// # Safety
    /// You have to warrant that somehow that there is enough byte to decode the encoded size.
    /// If you fail to do so, out of bound bytes will be read, and an absurd value will be
    /// returned.
    pub unsafe fn encoded_size_unchecked(&self) -> usize {
        self.as_ref().encoded_size_unchecked()
    }

    /// Changes the number of bytes the varint is encoded on.
    /// The number of byte is n + 1.
    /// You are to ensure that n <= 3.
    ///
    /// # Safety
    /// This will break:
    /// - the value: add or substract bytes at/from its end.
    ///
    /// It also breaks the payload after this varint.
    ///
    /// Only use it if you are sure about what you are doing.
    pub unsafe fn set_encoded_size(&mut self, n: u8) {
        *self.data.get_unchecked_mut(0) = (*self.data.get_unchecked(0) & 0x3F) | (n << 6);
    }

    /// Modify the value of the Varint in place.
    ///
    /// # Errors
    /// Fails if the new value is encoded on a different size of array (if it requires more or less
    /// bytes than the current value).
    pub fn set_value(&mut self, value: &Varint) -> Result<(), VarintSetError> {
        unsafe {
            if self.encoded_size_unchecked() != value.encoded_size() {
                return Err(VarintSetError::SizeMismatch);
            }
            value.encode_in_unchecked(self.data);
            Ok(())
        }
    }
}

impl<'data> EncodedData<'data> for EncodedVarintMut<'data> {
    type SourceData = &'data mut [u8];
    type DecodedData = Varint;
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

impl<'data> Decodable<'data> for Varint {
    type Data = EncodedVarint<'data>;
    type DataMut = EncodedVarintMut<'data>;
}

#[cfg(test)]
mod test {
    #![allow(clippy::unwrap_in_result, clippy::panic, clippy::expect_used)]

    use super::*;
    use crate::decodable::{Decodable, EncodedData};
    use hex_literal::hex;

    #[test]
    fn test_is_valid() {
        assert!(Varint::new(0x3F_FF_FF_FF).is_ok());
        assert!(Varint::new(0x40_00_00_00).is_err());
    }

    #[test]
    fn test_size() {
        assert_eq!(Varint::new(0x00).unwrap().encoded_size(), 1);
        assert_eq!(Varint::new(0x3F).unwrap().encoded_size(), 1);
        assert_eq!(Varint::new(0x3F_FF).unwrap().encoded_size(), 2);
        assert_eq!(Varint::new(0x3F_FF_FF).unwrap().encoded_size(), 3);
        assert_eq!(Varint::new(0x3F_FF_FF_FF).unwrap().encoded_size(), 4);
    }

    #[test]
    fn test_encode() {
        fn test(value: u32, data: &[u8]) {
            let mut encoded = [0_u8; MAX_SIZE];
            let size = Varint::new(value)
                .unwrap()
                .encode_in(&mut encoded[..data.len()])
                .unwrap();
            assert_eq!(data.len(), size);
            assert_eq!(*data, encoded[..data.len()]);

            let WithByteSize {
                item: mut decoder_mut,
                byte_size: expected_size,
            } = Varint::start_decoding_mut(&mut encoded).unwrap();
            assert_eq!(expected_size, size);

            assert_eq!(decoder_mut.complete_decoding().item.u32(), value);
            let new_value = Varint::new(if data.len() == 1 {
                (value == 0) as u32
            } else {
                value ^ 0x3F
            })
            .unwrap();
            assert!(new_value.u32() != value);
            decoder_mut.set_value(&new_value).unwrap();
            assert_eq!(decoder_mut.complete_decoding().item, new_value);

            // Unsafe mutations
            let original = decoder_mut.encoded_size().unwrap() - 1;
            let target = !original & 0x3;
            assert!(target != original);
            unsafe { decoder_mut.set_encoded_size(target as u8) };
            assert_eq!(decoder_mut.encoded_size().unwrap() - 1, target);
        }
        test(0x00, &[0]);
        test(0x3F, &hex!("3F"));
        test(0xFF, &hex!("40 FF"));
        test(0x3F_FF, &hex!("7F FF"));
        test(0x3F_FF_FF, &hex!("BF FF FF"));
        test(0x3F_FF_FF_FF, &hex!("FF FF FF FF"));
    }

    #[test]
    fn test_decode() {
        fn test_ok(data: &[u8], value: u32, size: usize) {
            // Check full decode
            let WithByteSize {
                item: ret,
                byte_size: decode_size,
            } = Varint::decode(data).unwrap();
            assert_eq!(decode_size, size);
            assert_eq!(ret, Varint::new(value).unwrap());

            // Check partial decoding
            let WithByteSize {
                item: decoder,
                byte_size: expected_size,
            } = Varint::start_decoding(data).unwrap();
            let part_size = decoder.encoded_size().unwrap();
            assert_eq!(expected_size, size);
            assert_eq!(part_size, size);
            assert_eq!(unsafe { decoder.encoded_size_unchecked() }, size);

            // Test partial mutability
            let mut encoded = [0_u8; MAX_SIZE];
            Varint::new(value)
                .unwrap()
                .encode_in(&mut encoded[..])
                .unwrap();

            // Check undecodability of shorter payload
            for i in 1..data.len() {
                assert_eq!(
                    Varint::start_decoding(&data[..i]),
                    Err(SizeError::MissingBytes)
                );
            }

            // Check unencodability in shorter arrays
            let value = Varint::new(value).unwrap();
            for i in 0..value.encoded_size() {
                let mut array = vec![0; i];
                assert_eq!(value.encode_in(&mut array), Err(value.encoded_size()));
            }
        }
        test_ok(&[0], 0x00, 1);
        test_ok(&hex!("3F"), 0x3F, 1);
        test_ok(&hex!("7F FF"), 0x3F_FF, 2);
        test_ok(&hex!("BF FF FF"), 0x3F_FF_FF, 3);
        test_ok(&hex!("FF FF FF FF"), 0x3F_FF_FF_FF, 4);

        test_ok(&hex!("00"), 0, 1);
        test_ok(&hex!("40 00"), 0, 2);
        test_ok(&hex!("80 00 00"), 0, 3);
        test_ok(&hex!("C0 00 00 00"), 0, 4);
    }

    #[test]
    fn test_errors() {
        // Varint new errors
        assert_eq!(Varint::new(0xFF_FF_FF_FF), Err(VarintError::ValueTooBig));
        assert_eq!(Varint::new(0x40_00_00_00), Err(VarintError::ValueTooBig));
        assert!(Varint::new(0x3F_FF_FF_FF).is_ok());

        // Varint decoding errors
        assert!(Varint::start_decoding(&[0x00]).is_ok());
        fn should_be_missing_bytes(data: &[u8]) {
            for i in 0..=data.len() {
                assert_eq!(
                    Varint::start_decoding(&data[..i]),
                    Err(SizeError::MissingBytes)
                );
            }
        }
        should_be_missing_bytes(&[0x40]);
        should_be_missing_bytes(&[0x80, 0x00]);
        should_be_missing_bytes(&[0xC0, 0x00, 0x00]);

        let varints: Vec<_> = vec![0x00, 0x40, 0x40_00, 0x40_00_00]
            .into_iter()
            .map(|n| Varint::new(n).unwrap())
            .collect();

        for i in 0..varints.len() {
            for j in 0..varints.len() {
                let mut encoded = vec![0; varints[i].encoded_size()];
                varints[i].encode_in(&mut encoded).unwrap();

                let WithByteSize {
                    item: mut decoder_mut,
                    byte_size: _,
                } = Varint::start_decoding_mut(&mut encoded).unwrap();
                if i == j {
                    assert!(decoder_mut.set_value(&varints[j]).is_ok());
                } else {
                    assert_eq!(
                        decoder_mut.set_value(&varints[j]),
                        Err(VarintSetError::SizeMismatch)
                    );
                }
            }
        }
    }
}

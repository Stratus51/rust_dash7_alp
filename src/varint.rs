// TODO ALP_SPEC: The encoding of the value is not specified!
// Big endian at bit and byte level probably, but it has to be specified!
use crate::decodable::{Decodable, EncodedData, WithByteSize};

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
    pub const fn new(value: u32) -> Result<Self, ()> {
        if value > U32_MAX {
            Err(())
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

    pub const fn usize(&self) -> usize {
        self.value as usize
    }

    /// Size in bytes of the encoded equivalent of the item.
    pub const fn size(&self) -> usize {
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

    /// # Safety
    /// You are responsible for checking that:
    /// - `size` == [`self.size()`](#method.size) (this determines the integer encoding)
    /// - `out.len()` >= `size`.
    ///
    /// Failing that will result in the program writing out of bound in
    /// random parts of your memory.
    pub unsafe fn __encode_in_ptr(&self, out: *mut u8, size: usize) {
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
        let size = self.size();
        self.__encode_in_ptr(out, size);
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
        let size = self.size();
        self.__encode_in_ptr(out.as_mut_ptr(), size);
        size
    }

    /// Encodes the value into pre allocated array.
    ///
    /// # Errors
    /// Fails if the pre allocated array is smaller than [`self.size()`](#method.size)
    /// returning the number of input bytes required.
    pub fn encode_in(&self, out: &mut [u8]) -> Result<usize, usize> {
        let size = self.size();
        if out.len() >= size {
            unsafe { self.__encode_in_ptr(out.as_mut_ptr(), size) };
            Ok(size)
        } else {
            Err(size)
        }
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

pub struct EncodedVarint<'data> {
    data: &'data [u8],
}

impl<'data> EncodedVarint<'data> {
    /// # Safety
    /// You are to warrant, somehow, that the input byte array contains a complete item.
    /// Else this might result in out of bound reads, and absurd results.
    pub unsafe fn size_unchecked(&self) -> usize {
        ((self.data.get_unchecked(0) & 0xC0) >> 6) as usize + 1
    }
}

impl<'data> EncodedData<'data> for EncodedVarint<'data> {
    type DecodedData = Varint;
    unsafe fn new(data: &'data [u8]) -> Self {
        Self { data }
    }

    fn size(&self) -> Result<usize, ()> {
        let mut size = 1;
        if self.data.len() < size {
            return Err(());
        }
        size = unsafe { self.size_unchecked() };
        if self.data.len() < size {
            return Err(());
        }
        Ok(size)
    }

    fn complete_decoding(&self) -> WithByteSize<Varint> {
        unsafe {
            let size = self.size_unchecked();
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

impl<'data> Decodable<'data> for Varint {
    type Data = EncodedVarint<'data>;
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
        assert_eq!(Varint::new(0x00).unwrap().size(), 1);
        assert_eq!(Varint::new(0x3F).unwrap().size(), 1);
        assert_eq!(Varint::new(0x3F_FF).unwrap().size(), 2);
        assert_eq!(Varint::new(0x3F_FF_FF).unwrap().size(), 3);
        assert_eq!(Varint::new(0x3F_FF_FF_FF).unwrap().size(), 4);
    }

    #[test]
    fn test_encode() {
        fn test(n: u32, truth: &[u8]) {
            let mut encoded = [0_u8; MAX_SIZE];
            let size = Varint::new(n).unwrap().encode_in(&mut encoded[..]).unwrap();
            assert_eq!(truth.len(), size);
            assert_eq!(*truth, encoded[..truth.len()]);
        }
        test(0x00, &[0]);
        test(0x3F, &hex!("3F"));
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
            let part_size = decoder.size().unwrap();
            assert_eq!(expected_size, size);
            assert_eq!(part_size, size);
            assert_eq!(unsafe { decoder.size_unchecked() }, size);
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
}

/// TODO This should be in outside v1.2. But it is here during the refactoring.
///
// TODO ALP_SPEC: The encoding of the value is not specified!
// Big endian at bit and byte level probably, but it has to be specified!

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

/// Variable integer
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Varint {
    value: u32,
}

impl Varint {
    /// Create a struct representing a Varint
    ///
    /// # Safety
    /// Only call this on u32 that are less than [MAX](constant.MAX.html)
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
    /// May fail if the value is bigger than [MAX](constant.MAX.html)
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
    pub const fn value(&self) -> u32 {
        self.value
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

    unsafe fn __encode_in_unchecked(&self, out: *mut u8, size: usize) {
        match size {
            1 => *out.offset(0) = (self.value & 0x3F) as u8,
            2 => {
                *out.offset(0) = ((self.value >> 8) & 0x3F) as u8;
                *out.offset(1) = (self.value & 0xFF) as u8;
            }
            3 => {
                *out.offset(0) = ((self.value >> 16) & 0x3F) as u8;
                *out.offset(1) = ((self.value >> 8) & 0xFF) as u8;
                *out.offset(2) = (self.value & 0xFF) as u8;
            }
            4 => {
                *out.offset(0) = ((self.value >> 24) & 0x3F) as u8;
                *out.offset(1) = ((self.value >> 16) & 0xFF) as u8;
                *out.offset(2) = ((self.value >> 8) & 0xFF) as u8;
                *out.offset(3) = (self.value & 0xFF) as u8;
            }
            _ => (),
        }
        *out.offset(0) |= (size as u8 - 1) << 6;
    }

    /// Encodes the Item without checking the size of the receiving
    /// byte array.
    ///
    /// # Safety
    /// You are responsible for checking that `out.len() >= size`. Failing that
    /// will result in the program writing out of bound. In the current
    /// implementation, it will trigger a panic.
    pub unsafe fn encode_in_unchecked(&self, out: &mut [u8]) -> usize {
        let size = self.size();
        self.__encode_in_unchecked(out.as_mut_ptr(), size);
        size
    }

    /// Encodes the value into pre allocated array.
    ///
    /// May fail if the pre allocated array is smaller than [self.size()](#method.size).
    pub fn encode_in(&self, out: &mut [u8]) -> Result<usize, ()> {
        let size = self.size();
        if out.len() >= size {
            unsafe { self.__encode_in_unchecked(out.as_mut_ptr(), size) };
            Ok(size)
        } else {
            Err(())
        }
    }

    /// Creates a decodable item without checking the data size.
    ///
    /// # Safety
    /// You are to check that data is not empty and that data.len() >=
    /// [DecodableVarint.size()](struct.DecodableVarint.html#method.size)
    /// (the expected byte size of the returned DecodableItem).
    pub const unsafe fn start_decoding_unchecked(data: &[u8]) -> DecodableVarint {
        DecodableVarint::new(data)
    }

    /// Creates a decodable item.
    ///
    /// This decodable item allows each parts of the item independently.
    pub const fn start_decoding(data: &[u8]) -> Result<DecodableVarint, ()> {
        if data.is_empty() {
            return Err(());
        }
        let ret = unsafe { Self::start_decoding_unchecked(data) };
        if data.len() < ret.size() {
            return Err(());
        }
        Ok(ret)
    }

    /// Decodes the Item from bytes.
    ///
    /// # Safety
    /// You are to check that data is not empty and that data.len() >=
    /// [DecodableVarint.size()](struct.DecodableVarint.html#method.size)
    /// (the expected byte size of the returned DecodableItem).
    pub const fn decode_unchecked(data: &[u8]) -> (Self, usize) {
        unsafe { Self::start_decoding_unchecked(data).complete_decoding() }
    }

    /// Decodes the item from bytes.
    pub const fn decode(data: &[u8]) -> Result<(Self, usize), ()> {
        match Self::start_decoding(data) {
            Ok(v) => Ok(v.complete_decoding()),
            Err(_) => Err(()),
        }
    }

    /// Encode the value into a single byte array.
    ///
    /// # Safety
    /// You are to warrant that the value does not exceed [U8_MAX](constant.U8_MAX.html)
    pub const unsafe fn encode_as_u8(value: u8) -> [u8; 1] {
        [value & 0x3F]
    }

    /// Encode the value into a two bytes array.
    ///
    /// Event though overkill, you technically can encode a small integer (such as 0) in
    /// this fixed sized array.
    ///
    /// # Safety
    /// You are to warrant that the value does not exceed [U16_MAX](constant.U16_MAX.html)
    pub const unsafe fn encode_as_u16(value: u16) -> [u8; 2] {
        [((value >> 8) & 0x3F) as u8, (value & 0xFF) as u8]
    }

    /// Encode the value into a three bytes array.
    ///
    /// Event though overkill, you technically can encode a small integer (such as 0) in
    /// this fixed sized array.
    ///
    /// # Safety
    /// You are to warrant that the value does not exceed [U24_MAX](constant.U24_MAX.html)
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
    /// You are to warrant that the value does not exceed [U32_MAX](constant.U32_MAX.html)
    pub const unsafe fn encode_as_u32(value: u32) -> [u8; 4] {
        [
            ((value >> 24) & 0x3F) as u8,
            ((value >> 8) & 0xFF) as u8,
            ((value >> 8) & 0xFF) as u8,
            (value & 0xFF) as u8,
        ]
    }
}

pub struct DecodableVarint<'a> {
    pub data_slice: &'a [u8],
    pub data: *const u8,
}

impl<'a> DecodableVarint<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            data_slice: data,
            data: data.as_ptr(),
        }
    }

    /// Decodes the size of the Item in bytes
    pub const fn size(&self) -> usize {
        ((*self.data.offset(0) & 0xC0) >> 6) as usize
    }

    /// Fully decode the Item
    pub const fn complete_decoding(&self) -> (Varint, usize) {
        let size = self.size();
        let data = &self.data;
        let ret = unsafe {
            Varint::new_unchecked(match size {
                0 => (*data.offset(0) & 0x3F) as u32,
                1 => (((*data.offset(0) & 0x3F) as u32) << 8) + *data.offset(1) as u32,
                2 => {
                    (((*data.offset(0) & 0x3F) as u32) << 16)
                        + ((*data.offset(1) as u32) << 8)
                        + *data.offset(2) as u32
                }
                3 => {
                    (((*data.offset(0) & 0x3F) as u32) << 24)
                        + ((*data.offset(1) as u32) << 16)
                        + ((*data.offset(2) as u32) << 8)
                        + *data.offset(3) as u32
                }
                // This is bad and incorrect. But size should mathematically never evaluate to this
                // case. Let's just hope the size method is not broken.
                _ => 0,
            })
        };
        (ret, size)
    }
}

#[cfg(test)]
mod test {
    use super::*;
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
            let mut encoded = [0u8; MAX_SIZE];
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
            let (ret, decode_size) = Varint::decode(data).unwrap();
            assert_eq!(decode_size, size);
            assert_eq!(ret, Varint::new(value).unwrap());
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

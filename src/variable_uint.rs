use crate::Serializable;
use crate::{ParseError, ParseResult, ParseValue};
use std::convert::TryFrom;

#[cfg(test)]
use hex_literal::hex;

#[derive(Debug, PartialEq)]
pub struct VariableUint {
    pub value: u32,
}
const MAX_VARIABLE_UINT: u32 = 0x3F_FF_FF_FF;

impl VariableUint {
    pub fn new(value: u32) -> Result<Self, ()> {
        if value > MAX_VARIABLE_UINT {
            Err(())
        } else {
            Ok(Self { value })
        }
    }

    pub fn set(&mut self, value: u32) -> Result<(), ()> {
        if value > MAX_VARIABLE_UINT {
            Err(())
        } else {
            self.value = value;
            Ok(())
        }
    }

    pub fn is_valid(n: u32) -> Result<(), ()> {
        if n > MAX_VARIABLE_UINT {
            Err(())
        } else {
            Ok(())
        }
    }

    pub fn usize_is_valid(n: usize) -> Result<(), ()> {
        u32::try_from(n).map_err(|_| ()).and_then(Self::is_valid)
    }

    /// # Safety
    /// Only call this on u32 that are less than 0x3F_FF_FF_FF.
    ///
    /// Calling this on a large integer will return a size of 4 which
    /// is technically incorrect because the integer is non-encodable.
    pub unsafe fn unsafe_size(n: u32) -> u8 {
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

    pub fn size(n: u32) -> Result<u8, ()> {
        if n > MAX_VARIABLE_UINT {
            Err(())
        } else {
            Ok(unsafe { Self::unsafe_size(n) })
        }
    }

    // TODO Is this serialization correct? Check the SPEC!
    /// # Safety
    /// Only call this on u32 that are less than 0x3F_FF_FF_FF.
    ///
    /// Calling this on a large integer will return an unpredictable
    /// result (it won't crash).
    pub unsafe fn u32_serialize(n: u32, out: &mut [u8]) -> u8 {
        let u8_size = Self::unsafe_size(n);
        let size = u8_size as usize;
        for (i, byte) in out.iter_mut().enumerate().take(size) {
            *byte = ((n >> ((size - 1 - i) * 8)) & 0xFF) as u8;
        }
        out[0] |= ((size - 1) as u8) << 6;
        u8_size
    }

    pub fn u32_deserialize(out: &[u8]) -> ParseResult<u32> {
        if out.is_empty() {
            return Err(ParseError::MissingBytes(Some(1)));
        }
        let size = ((out[0] >> 6) + 1) as usize;
        if out.len() < size as usize {
            return Err(ParseError::MissingBytes(Some(size as usize - out.len())));
        }
        let mut ret = (out[0] & 0x3F) as u32;
        for byte in out.iter().take(size).skip(1) {
            ret = (ret << 8) + *byte as u32;
        }
        Ok(ParseValue {
            value: ret,
            data_read: size,
        })
    }
}

impl Serializable for VariableUint {
    fn serialized_size(&self) -> usize {
        unsafe { Self::unsafe_size(self.value) as usize }
    }

    fn serialize(&self, out: &mut [u8]) -> usize {
        unsafe { Self::u32_serialize(self.value, out) as usize }
    }

    fn deserialize(out: &[u8]) -> ParseResult<Self> {
        Self::u32_deserialize(out).map(|ParseValue { value, data_read }| ParseValue {
            value: Self { value },
            data_read,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_new() {
        assert_eq!(VariableUint::new(0xFF_FF_FF_FF), Err(()));
        assert_eq!(VariableUint::new(0x01), Ok(VariableUint { value: 0x01 }));
    }

    #[test]
    fn test_set() {
        let mut uint = VariableUint::new(0x01).unwrap();
        assert_eq!(uint.set(0x3F_FF_FF_FF), Ok(()));
        assert_eq!(uint.set(0x40_00_00_00), Err(()));
    }

    #[test]
    fn test_is_valid() {
        assert_eq!(VariableUint::is_valid(0x3F_FF_FF_FF), Ok(()));
        assert_eq!(VariableUint::is_valid(0x40_00_00_00), Err(()));
    }

    #[test]
    fn test_unsafe_size() {
        unsafe {
            assert_eq!(VariableUint::unsafe_size(0x00), 1);
            assert_eq!(VariableUint::unsafe_size(0x3F), 1);
            assert_eq!(VariableUint::unsafe_size(0x3F_FF), 2);
            assert_eq!(VariableUint::unsafe_size(0x3F_FF_FF), 3);
            assert_eq!(VariableUint::unsafe_size(0x3F_FF_FF_FF), 4);
        }
    }

    #[test]
    fn test_size() {
        assert_eq!(VariableUint::size(0x00), Ok(1));
        assert_eq!(VariableUint::size(0x40_00_00_00), Err(()));
    }

    #[test]
    fn test_u32_serialize() {
        fn test(n: u32, truth: &[u8]) {
            let mut encoded = vec![0u8; truth.len()];
            println!("{}", encoded.len());
            assert_eq!(
                unsafe { VariableUint::u32_serialize(n, &mut encoded[..]) },
                truth.len() as u8
            );
            assert_eq!(*truth, encoded[..]);
        }
        test(0x00, &[0]);
        test(0x3F, &hex!("3F"));
        test(0x3F_FF, &hex!("7F FF"));
        test(0x3F_FF_FF, &hex!("BF FF FF"));
        test(0x3F_FF_FF_FF, &hex!("FF FF FF FF"));
    }

    #[test]
    fn test_serialize() {
        assert_eq!(
            *VariableUint::new(0x3F_FF_FF).unwrap().serialize_to_box(),
            hex!("BF FF FF")
        );
    }
}

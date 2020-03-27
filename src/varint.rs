use crate::codec::{ParseFail, ParseResult, ParseValue};
pub const MAX: u32 = 0x3F_FF_FF_FF;
/// Returns whether the value is encodable into a varint or not.
pub fn is_valid(n: u32) -> Result<(), ()> {
    if n > MAX {
        Err(())
    } else {
        Ok(())
    }
}

/// Calculate the size in bytes of the value encoded as a varint.
///
/// # Safety
/// Only call this on u32 that are less than 0x3F_FF_FF_FF.
///
/// Calling this on a large integer will return a size of 4 which
/// is technically incorrect because the integer is non-encodable.
pub unsafe fn size(n: u32) -> u8 {
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

/// Encode the value into a varint.
///
/// # Safety
/// Only call this on u32 that are less than 0x3F_FF_FF_FF.
///
/// Calling this on a large integer will return an unpredictable
/// result (it won't crash).
pub unsafe fn encode(n: u32, out: &mut [u8]) -> u8 {
    let u8_size = size(n);
    let size = u8_size as usize;
    for (i, byte) in out.iter_mut().enumerate().take(size) {
        *byte = ((n >> ((size - 1 - i) * 8)) & 0xFF) as u8;
    }
    out[0] |= ((size - 1) as u8) << 6;
    u8_size
}

/// Decode a byte array as a varint.
pub fn decode(out: &[u8]) -> ParseResult<u32> {
    if out.is_empty() {
        return Err(ParseFail::MissingBytes(1));
    }
    let size = ((out[0] >> 6) + 1) as usize;
    if out.len() < size as usize {
        return Err(ParseFail::MissingBytes(size as usize - out.len()));
    }
    let mut ret = (out[0] & 0x3F) as u32;
    for byte in out.iter().take(size).skip(1) {
        ret = (ret << 8) + *byte as u32;
    }
    Ok(ParseValue { value: ret, size })
}

#[cfg(test)]
mod test {
    use super::*;
    use hex_literal::hex;

    #[test]
    fn test_is_valid() {
        assert_eq!(is_valid(0x3F_FF_FF_FF), Ok(()));
        assert_eq!(is_valid(0x40_00_00_00), Err(()));
    }

    #[test]
    fn test_unsafe_size() {
        unsafe {
            assert_eq!(size(0x00), 1);
            assert_eq!(size(0x3F), 1);
            assert_eq!(size(0x3F_FF), 2);
            assert_eq!(size(0x3F_FF_FF), 3);
            assert_eq!(size(0x3F_FF_FF_FF), 4);
        }
    }

    #[test]
    fn test_encode() {
        fn test(n: u32, truth: &[u8]) {
            let mut encoded = vec![0u8; truth.len()];
            assert_eq!(unsafe { encode(n, &mut encoded[..]) }, truth.len() as u8);
            assert_eq!(*truth, encoded[..]);
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
            assert_eq!(decode(data), Ok(ParseValue { value, size: size }),);
        }
        test_ok(&[0], 0x00, 1);
        test_ok(&hex!("3F"), 0x3F, 1);
        test_ok(&hex!("7F FF"), 0x3F_FF, 2);
        test_ok(&hex!("BF FF FF"), 0x3F_FF_FF, 3);
        test_ok(&hex!("FF FF FF FF"), 0x3F_FF_FF_FF, 4);
    }
}

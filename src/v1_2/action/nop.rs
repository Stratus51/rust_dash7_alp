use super::error::BasicDecodeError;
use super::flag;
use super::op_code::OpCode;

/// Maximum byte size of an encoded Nop
pub const MAX_SIZE: usize = 1;

/// Does nothing.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Nop {
    /// Group with next action
    pub group: bool,
    /// Ask for a response (status)
    pub response: bool,
}

impl Default for Nop {
    /// Default Nop with group = false and response = true.
    ///
    /// Because that would be the most common use case: a ping command.
    fn default() -> Self {
        Self {
            group: false,
            response: true,
        }
    }
}

impl Nop {
    /// Encodes the Item into a fixed size array
    pub const fn encode_to_array(&self) -> [u8; 1] {
        [OpCode::Nop as u8
            + if self.group { flag::GROUP } else { 0 }
            + if self.response { flag::RESPONSE } else { 0 }]
    }

    /// Encodes the Item without checking the size of the receiving
    /// byte array.
    ///
    /// # Safety
    /// You are responsible for checking that `size` == [self.size()](#method.size) and
    /// to insure `out.len() >= size`. Failing that will result in the
    /// program writing out of bound. In the current implementation, it
    /// will trigger a panic.
    pub unsafe fn encode_in_unchecked(&self, buf: &mut [u8]) -> usize {
        *buf.get_unchecked_mut(0) = OpCode::Nop as u8
            + if self.group { flag::GROUP } else { 0 }
            + if self.response { flag::RESPONSE } else { 0 };
        1
    }

    /// Encodes the value into pre allocated array.
    ///
    /// Fails if the pre allocated array is smaller than [self.size()](#method.size)
    /// returning the number of input bytes required.
    pub fn encode_in(&self, out: &mut [u8]) -> Result<usize, usize> {
        let size = self.size();
        if out.len() >= size {
            Ok(unsafe { self.encode_in_unchecked(out) })
        } else {
            Err(size)
        }
    }

    /// Size in bytes of the encoded equivalent of the item.
    pub const fn size(&self) -> usize {
        1
    }

    /// Creates a decodable item without checking the data size.
    ///
    /// # Safety
    /// You are to check that data is not empty and that data.len() >=
    /// [DecodableNop.size()](struct.DecodableNop.html#method.size)
    /// (the expected byte size of the returned DecodableItem).
    ///
    /// You are also expected to warrant that the opcode contained in the
    /// first byte corresponds to this action.
    pub const unsafe fn start_decoding_unchecked(data: &[u8]) -> DecodableNop {
        DecodableNop::new(data)
    }

    /// Creates a decodable item.
    ///
    /// This decodable item allows each parts of the item independently.
    pub const fn start_decoding(data: &[u8]) -> Result<DecodableNop, BasicDecodeError> {
        if data.is_empty() {
            return Err(BasicDecodeError::MissingBytes(1));
        }
        if data[0] & 0x3F != OpCode::Nop as u8 {
            return Err(BasicDecodeError::BadOpCode);
        }
        let ret = unsafe { Self::start_decoding_unchecked(data) };
        Ok(ret)
    }

    /// Decodes the Item from bytes.
    ///
    /// Returns the decoded data and the number of bytes consumed to produce it.
    ///
    /// # Safety
    /// You are to check that data is not empty and that data.len() >=
    /// [DecodableNop.size()](struct.DecodableNop.html#method.size)
    /// (the expected byte size of the returned DecodableItem).
    pub unsafe fn decode_unchecked(data: &[u8]) -> (Self, usize) {
        Self::start_decoding_unchecked(data).complete_decoding()
    }

    /// Decodes the item from bytes.
    ///
    /// On success, returns the decoded data and the number of bytes consumed
    /// to produce it.
    pub fn decode(data: &[u8]) -> Result<(Self, usize), BasicDecodeError> {
        match Self::start_decoding(data) {
            Ok(v) => Ok(v.complete_decoding()),
            Err(e) => Err(e),
        }
    }
}

pub struct DecodableNop<'a> {
    data: &'a [u8],
}

impl<'a> DecodableNop<'a> {
    const fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    /// Decodes the size of the Item in bytes
    pub const fn size(&self) -> usize {
        1
    }

    pub fn group(&self) -> bool {
        unsafe { *self.data.get_unchecked(0) & flag::GROUP != 0 }
    }

    pub fn response(&self) -> bool {
        unsafe { *self.data.get_unchecked(0) & flag::RESPONSE != 0 }
    }

    /// Fully decode the Item
    ///
    /// Returns the decoded data and the number of bytes consumed to produce it.
    pub fn complete_decoding(&self) -> (Nop, usize) {
        (
            Nop {
                group: self.group(),
                response: self.response(),
            },
            1,
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn known() {
        fn test(op: Nop, data: &[u8]) {
            // Test op.encode_in() == data
            let mut encoded = [0u8; 1];
            let size = op.encode_in(&mut encoded).unwrap();
            assert_eq!(size, data.len());
            assert_eq!(&encoded, data);

            // Test op.encode_to_array() == data
            assert_eq!(&op.encode_to_array(), data);

            // Test decode(data) == op
            let (ret, size) = Nop::decode(&data).unwrap();
            assert_eq!(size, data.len());
            assert_eq!(ret, op);
        }
        test(
            Nop {
                group: false,
                response: true,
            },
            &[0x40],
        );
        test(
            Nop {
                group: true,
                response: false,
            },
            &[0x80],
        );
        test(
            Nop {
                group: true,
                response: true,
            },
            &[0xC0],
        );
        test(
            Nop {
                group: false,
                response: false,
            },
            &[0x00],
        );
    }

    #[test]
    fn consistence() {
        let op = Nop {
            group: true,
            response: false,
        };

        // Test decode(op.encode_to_array()) == op
        let data = op.encode_to_array();
        let (ret, size) = Nop::decode(&op.encode_to_array()).unwrap();
        assert_eq!(size, data.len());
        assert_eq!(ret, op);

        // Test decode(data).encode_to_array() == data
        let (ret, size) = Nop::decode(&data).unwrap();
        assert_eq!(size, data.len());
        assert_eq!(ret.encode_to_array(), data);
    }
}

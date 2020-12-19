use super::error::BasicDecodeError;
use super::flag;
use super::op_code::OpCode;

/// Does nothing.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Nop {
    /// Group with next action
    pub group: bool,
    /// Ask for a response (status)
    pub response: bool,
}

impl Nop {
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
        buf[0] = OpCode::Nop as u8
            + if self.group { flag::GROUP } else { 0 }
            + if self.response { flag::RESPONSE } else { 0 };
        1
    }

    /// Encodes the value into pre allocated array.
    ///
    /// May fail if the pre allocated array is smaller than [self.size()](#method.size).
    pub fn encode_in(&self, out: &mut [u8]) -> Result<usize, ()> {
        let size = self.size();
        if out.len() >= size {
            Ok(unsafe { self.encode_in_unchecked(out) })
        } else {
            Err(())
        }
    }

    pub const fn size(&self) -> usize {
        1
    }

    /// Creates a decodable item without checking the data size.
    ///
    /// # Safety
    /// You are to check that data is not empty and that data.len() >=
    /// [DecodableVarint.size()](struct.DecodableNop.html#method.size)
    /// (the expected byte size of the returned DecodableItem).
    ///
    /// You are also expected to warrant that the opcode contained in the
    /// first byte corresponds to this action.
    pub const unsafe fn start_decoding_unchecked(data: &[u8]) -> DecodableNop {
        DecodableNop { data }
    }

    /// Creates a decodable item.
    ///
    /// This decodable item allows each parts of the item independently.
    pub const fn start_decoding(data: &[u8]) -> Result<DecodableNop, BasicDecodeError> {
        if data.is_empty() {
            return Err(BasicDecodeError::MissingBytes);
        }
        if data[0] & 0x3F != OpCode::Nop as u8 {
            return Err(BasicDecodeError::BadOpCode);
        }
        let ret = unsafe { Self::start_decoding_unchecked(data) };
        // TODO XXX
        if data.len() < ret.size() {
            return Err(BasicDecodeError::MissingBytes);
        }
        Ok(ret)
    }

    /// Decodes the Item from bytes.
    ///
    /// # Safety
    /// You are to check that data is not empty and that data.len() >=
    /// [DecodableVarint.size()](struct.DecodableVarint.html#method.size)
    /// (the expected byte size of the returned DecodableItem).
    pub const fn decode_unchecked(data: &[u8]) -> Self {
        unsafe { Self::start_decoding_unchecked(data).complete_decoding() }
    }

    /// Decodes the item from bytes.
    pub const fn decode(data: &[u8]) -> Result<Self, BasicDecodeError> {
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
    /// Decodes the size of the Item in bytes
    pub const fn size(&self) -> usize {
        1
    }

    pub const fn group(&self) -> bool {
        self.data[0] & flag::GROUP != 0
    }

    pub const fn response(&self) -> bool {
        self.data[0] & flag::RESPONSE != 0
    }

    /// Fully decode the Item
    pub const fn complete_decoding(&self) -> Nop {
        Nop {
            group: self.group(),
            response: self.response(),
        }
    }
}

#[test]
fn known() {
    assert_eq!(
        Nop {
            group: false,
            response: true
        }
        .encode_to_array(),
        [0x40]
    );
    assert_eq!(
        Nop::decode(&[0x40]).unwrap(),
        Nop {
            group: false,
            response: true
        }
    );
}

#[test]
fn consistence() {
    let op = Nop {
        group: true,
        response: false,
    };
    let data = op.encode_to_array();
    assert_eq!(Nop::decode(&op.encode_to_array()).unwrap(), op);
    assert_eq!(Nop::decode(&data).unwrap().encode_to_array(), data);
}

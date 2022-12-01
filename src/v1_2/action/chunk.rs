#[cfg(test)]
use crate::test_tools::test_item;
#[cfg(test)]
use hex_literal::hex;

use crate::{
    codec::{Codec, StdError, WithOffset, WithSize},
    v1_2::action::OpCode,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Chunk {
    Continue = 0,
    Start = 1,
    End = 2,
    StartEnd = 3,
}
impl std::fmt::Display for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self {
            Chunk::Continue => write!(f, "[C]"),
            Chunk::Start => write!(f, "[S]"),
            Chunk::End => write!(f, "[E]"),
            Chunk::StartEnd => write!(f, "[R]"),
        }
    }
}
impl Chunk {
    fn from(n: u8) -> Self {
        match n {
            0 => Chunk::Continue,
            1 => Chunk::Start,
            2 => Chunk::End,
            3 => Chunk::StartEnd,
            x => panic!("Impossible chunk step {}", x),
        }
    }
}
impl Codec for Chunk {
    type Error = StdError;
    fn encoded_size(&self) -> usize {
        1
    }
    unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
        out[0] = OpCode::Chunk as u8 + ((*self as u8) << 6);
        1
    }
    fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
        if out.is_empty() {
            return Err(WithOffset::new_head(Self::Error::MissingBytes(1)));
        }
        Ok(WithSize {
            value: Self::from(out[0] >> 6),
            size: 1,
        })
    }
}
#[test]
fn test_chunk() {
    test_item(Chunk::End, &hex!("B0"))
}

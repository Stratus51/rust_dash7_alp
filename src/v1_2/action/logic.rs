#[cfg(test)]
use crate::test_tools::test_item;
#[cfg(test)]
use hex_literal::hex;

use crate::codec::{Codec, StdError, WithOffset, WithSize};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Logic {
    Or = 0,
    Xor = 1,
    Nor = 2,
    Nand = 3,
}
impl Logic {
    fn from(n: u8) -> Self {
        match n {
            0 => Logic::Or,
            1 => Logic::Xor,
            2 => Logic::Nor,
            3 => Logic::Nand,
            x => panic!("Impossible logic op {}", x),
        }
    }
}
impl std::fmt::Display for Logic {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Logic::Or => write!(f, "[OR]"),
            Logic::Xor => write!(f, "[XOR]"),
            Logic::Nor => write!(f, "[NOR]"),
            Logic::Nand => write!(f, "[NAND]"),
        }
    }
}
impl Codec for Logic {
    type Error = StdError;
    fn encoded_size(&self) -> usize {
        1
    }
    unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
        out[0] = crate::v1_2::action::OpCode::Logic as u8 + ((*self as u8) << 6);
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
fn test_logic() {
    test_item(Logic::Nand, &hex!("F1"))
}

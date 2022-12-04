use crate::{
    codec::{Codec, StdError, WithOffset, WithSize},
    v1_2::operand,
};

#[derive(Clone, Debug, PartialEq)]
pub struct IndirectForward {
    // ALP_SPEC Ask for response ?
    pub resp: bool,
    pub interface: operand::IndirectInterface,
}
impl std::fmt::Display for IndirectForward {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "[{}]{}",
            if self.resp { "R" } else { "-" },
            self.interface
        )
    }
}
impl Codec for IndirectForward {
    type Error = StdError;
    fn encoded_size(&self) -> usize {
        1 + self.interface.encoded_size()
    }
    unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
        let overload = match self.interface {
            operand::IndirectInterface::Overloaded(_) => true,
            operand::IndirectInterface::NonOverloaded(_) => false,
        };
        out[0] |= ((overload as u8) << 7) | ((self.resp as u8) << 6);
        1 + super::serialize_all!(&mut out[1..], &self.interface)
    }
    fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
        if out.is_empty() {
            Err(WithOffset::new_head(Self::Error::MissingBytes(1)))
        } else {
            let mut offset = 0;
            let WithSize {
                value: op1,
                size: op1_size,
            } = operand::IndirectInterface::decode(out)?;
            offset += op1_size;
            Ok(WithSize {
                value: Self {
                    resp: out[0] & 0x40 != 0,
                    interface: op1,
                },
                size: offset,
            })
        }
    }
}

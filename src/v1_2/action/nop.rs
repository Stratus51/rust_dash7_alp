use crate::codec::{Codec, StdError, WithOffset, WithSize};

/// Does nothing
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Nop {
    /// Group with next action
    pub group: bool,
    /// Ask for a response (status)
    pub resp: bool,
}
super::impl_display_simple_op!(Nop);
impl Codec for Nop {
    type Error = StdError;

    fn encoded_size(&self) -> usize {
        1
    }
    unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
        out[0] |= ((self.group as u8) << 7) | ((self.resp as u8) << 6);
        1
    }
    fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
        if out.is_empty() {
            Err(WithOffset::new_head(Self::Error::MissingBytes(1)))
        } else {
            Ok(WithSize {
                size: 1,
                value: Self {
                    resp: out[0] & 0x40 != 0,
                    group: out[0] & 0x80 != 0,
                },
            })
        }
    }
}

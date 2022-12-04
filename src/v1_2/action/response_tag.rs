use crate::codec::{Codec, StdError, WithOffset, WithSize};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResponseTag {
    /// End of packet
    ///
    /// Signal the last response packet for the request `id`
    /// (E)
    pub eop: bool,
    /// An error occured
    /// (R)
    pub err: bool,
    pub id: u8,
}
impl std::fmt::Display for ResponseTag {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "[{}{}]({})",
            if self.eop { "E" } else { "-" },
            if self.err { "R" } else { "-" },
            self.id,
        )
    }
}
super::impl_simple_op!(ResponseTag, eop, err, id);

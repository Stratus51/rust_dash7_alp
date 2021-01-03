#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum StatusExtension {
    // Action = 0,
    Interface = 1,
}

impl StatusExtension {
    /// # Errors
    /// Returns an error if the query code is unknown
    pub const fn from(n: u8) -> Result<Self, u8> {
        Ok(match n {
            // 0 => Self::Action,
            1 => Self::Interface,
            x => return Err(x),
        })
    }
}

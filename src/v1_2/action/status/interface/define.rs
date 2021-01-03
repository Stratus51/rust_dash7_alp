#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum InterfaceId {
    Host = 0,
    Dash7 = 0xD7,
}
impl InterfaceId {
    /// # Safety
    /// You have to ensure that the n belongs to the set defined to
    /// [`InterfaceId`](enum.InterfaceId.html)
    pub const unsafe fn from_unchecked(n: u8) -> Self {
        match n {
            0 => Self::Host,
            0xD7 => Self::Dash7,
            // Should never occured if used safely
            _ => Self::Host,
        }
    }

    /// # Errors
    /// Returns an error if n > 7
    pub const fn from(n: u8) -> Result<Self, ()> {
        Ok(match n {
            0 => Self::Host,
            0xD7 => Self::Dash7,
            _ => return Err(()),
        })
    }
}

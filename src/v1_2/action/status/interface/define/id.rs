use crate::v1_2::error::define::InterfaceIdError;

const HOST: u8 = 0;
const DASH7: u8 = 0xD7;

#[repr(u8)]
#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum InterfaceId {
    Host = HOST,
    Dash7 = DASH7,
}

impl InterfaceId {
    /// # Safety
    /// You have to ensure that the n belongs to the set defined to
    /// [`InterfaceId`](enum.InterfaceId.html)
    pub unsafe fn from_unchecked(n: u8) -> Self {
        core::mem::transmute(n)
    }

    /// # Errors
    /// Returns an error if n > 7
    pub const fn from(n: u8) -> Result<Self, InterfaceIdError> {
        Ok(match n {
            HOST => Self::Host,
            DASH7 => Self::Dash7,
            id => return Err(InterfaceIdError::Unsupported { id }),
        })
    }
}

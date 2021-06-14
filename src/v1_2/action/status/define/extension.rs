use crate::v1_2::error::define::StatusExtensionError;

pub const ACTION: u8 = 0;
pub const INTERFACE: u8 = 1;

#[repr(u8)]
#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum StatusExtension {
    // Action = ACTION,
    Interface = INTERFACE,
}

impl StatusExtension {
    /// # Errors
    /// Returns an error if the query code is unknown
    pub const fn from(n: u8) -> Result<Self, StatusExtensionError> {
        Ok(match n {
            // ACTION => Self::Action,
            INTERFACE => Self::Interface,
            ext if ext <= 3 => return Err(StatusExtensionError::Unsupported { ext }),
            _ => return Err(StatusExtensionError::Invalid),
        })
    }
}

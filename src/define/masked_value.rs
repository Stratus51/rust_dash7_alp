#[cfg(feature = "alloc")]
use alloc::prelude::v1::Box;

#[cfg(feature = "alloc")]
use super::EncodableData;
use super::EncodableDataRef;

/// Represents some encodable data that can be masked.
///
/// To be valid, the mask, if present must be the same size as the data.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct MaskedValueRef<'a> {
    value: EncodableDataRef<'a>,
    mask: Option<&'a [u8]>,
}

impl<'a> MaskedValueRef<'a> {
    /// # Safety
    /// If mask is defined you are to warrant that value.len() == mask.len().
    pub const unsafe fn new_unchecked(value: EncodableDataRef<'a>, mask: Option<&'a [u8]>) -> Self {
        Self { value, mask }
    }

    /// # Errors
    /// Fails if the mask is defined and the mask and the value do not have the same size.
    pub fn new(value: EncodableDataRef<'a>, mask: Option<&'a [u8]>) -> Result<Self, ()> {
        if let Some(mask) = &mask {
            if mask.len() != value.len() {
                return Err(());
            }
        }
        Ok(unsafe { Self::new_unchecked(value, mask) })
    }

    pub const fn value(&self) -> &[u8] {
        self.value.data()
    }

    pub const fn mask(&self) -> Option<&[u8]> {
        self.mask
    }

    pub fn len(&self) -> usize {
        self.value.len()
    }

    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    #[cfg(feature = "alloc")]
    pub fn to_owned(&self) -> MaskedValue {
        MaskedValue {
            value: self.value.to_owned(),
            mask: self.mask.map(|mask| mask.into()),
        }
    }
}

#[cfg(feature = "alloc")]
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct MaskedValue {
    value: EncodableData,
    mask: Option<Box<[u8]>>,
}

#[cfg(feature = "alloc")]
impl MaskedValue {
    pub fn as_ref(&self) -> MaskedValueRef {
        MaskedValueRef {
            value: self.value.as_ref(),
            mask: self.mask.as_ref().map(|mask| mask.as_ref()),
        }
    }
}

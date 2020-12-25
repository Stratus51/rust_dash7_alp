use super::EncodableData;

/// Represents some encodable data that can be masked.
///
/// To be valid, the mask, if present must be the same size as the data.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct MaskedValue<'a> {
    value: EncodableData<'a>,
    mask: Option<&'a [u8]>,
}

impl<'a> MaskedValue<'a> {
    /// # Safety
    /// If mask is defined you are to warrant that value.len() == mask.len().
    pub const unsafe fn new_unchecked(value: EncodableData<'a>, mask: Option<&'a [u8]>) -> Self {
        Self { value, mask }
    }

    /// # Errors
    /// Fails if the mask is defined and the mask and the value do not have the same size.
    pub fn new(value: EncodableData<'a>, mask: Option<&'a [u8]>) -> Result<Self, ()> {
        if let Some(mask) = &mask {
            if mask.len() != value.len() {
                return Err(());
            }
        }
        Ok(unsafe { Self::new_unchecked(value, mask) })
    }

    pub const fn value(&self) -> &[u8] {
        self.value.get()
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
}

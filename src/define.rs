use crate::varint;

/// Wrapper struct representing a dash7 file id.
///
/// It is exactly homogeneous to a byte, and the wrapping is only done
/// to help insure semantic correctness of the code.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct FileId(pub u8);

impl FileId {
    pub const fn new(n: u8) -> Self {
        Self(n)
    }

    pub const fn u8(self) -> u8 {
        let FileId(fid) = self;
        fid
    }
}

/// Wrapper of a raw data byte array to be used in the dash7 ALP
/// actions.
///
/// To be valid, it needs to have a size encodable using a [Varint](varint::Varint),
/// and thus must have a length <= [varint::MAX_SIZE](varint::MAX_SIZE)
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct EncodableData<'a>(&'a [u8]);

impl<'a> EncodableData<'a> {
    /// # Safety
    /// You are to warrant that data.len() <= [varint::MAX_SIZE](varint::MAX_SIZE)
    pub const unsafe fn new_unchecked(data: &'a [u8]) -> Self {
        Self(data)
    }

    /// Fails if the length of the data is bigger than [varint::MAX_SIZE](varint::MAX_SIZE).
    pub const fn new(data: &'a [u8]) -> Result<Self, ()> {
        if data.len() > varint::MAX_SIZE {
            Err(())
        } else {
            Ok(unsafe { Self::new_unchecked(data) })
        }
    }

    pub const fn get(&self) -> &[u8] {
        let Self(data) = self;
        data
    }

    pub fn len(&self) -> usize {
        self.get().len()
    }

    pub fn is_empty(&self) -> bool {
        self.get().is_empty()
    }
}

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

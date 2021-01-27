#[cfg(feature = "alloc")]
use alloc::prelude::v1::Box;

use crate::varint;

/// Wrapper of a raw data byte array to be used in the dash7 ALP
/// actions.
///
/// To be valid, it needs to have a size encodable using a [Varint](varint::Varint),
/// and thus must have a length <= [varint::MAX_SIZE](varint::MAX_SIZE)
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct EncodableDataRef<'a>(&'a [u8]);

impl<'a> EncodableDataRef<'a> {
    /// # Safety
    /// You are to warrant that data.len() <= [varint::MAX_SIZE](varint::MAX_SIZE)
    pub const unsafe fn new_unchecked(data: &'a [u8]) -> Self {
        Self(data)
    }

    /// # Errors
    /// Fails if the length of the data is bigger than [varint::MAX_SIZE](varint::MAX_SIZE).
    pub const fn new(data: &'a [u8]) -> Result<Self, ()> {
        if data.len() > varint::MAX_SIZE {
            Err(())
        } else {
            Ok(unsafe { Self::new_unchecked(data) })
        }
    }

    pub const fn data(&self) -> &[u8] {
        let Self(data) = self;
        data
    }

    pub fn len(&self) -> usize {
        self.data().len()
    }

    pub fn is_empty(&self) -> bool {
        self.data().is_empty()
    }

    #[cfg(feature = "alloc")]
    pub fn to_owned(&self) -> EncodableData {
        EncodableData(self.data().into())
    }
}

#[cfg(feature = "alloc")]
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct EncodableData(Box<[u8]>);

#[cfg(feature = "alloc")]
impl EncodableData {
    pub const fn data(&self) -> &[u8] {
        let Self(data) = self;
        data
    }

    pub fn as_ref(&self) -> EncodableDataRef {
        EncodableDataRef(self.data())
    }
}

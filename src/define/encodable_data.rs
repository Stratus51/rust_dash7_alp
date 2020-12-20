use crate::varint;

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

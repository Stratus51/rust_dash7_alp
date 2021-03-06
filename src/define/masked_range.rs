#[cfg(feature = "alloc")]
use alloc::prelude::v1::Box;

/// Represents a bitmap range.
///
/// `start` and `end` both represent a bit offset in a virtually infinite bitmap.
/// Every value in the bitmap between `start` and `end` are considered selected
/// by this range.
///
/// The bitmap, if present, selects which of the values in the range are truly selected.
/// Thus the first bit of the bitmap corresponds to the start bit of the range.
// TODO SPEC: The endianess of the bitmap is not clearly stated in the spec.
///
/// If a bitmap is defined, its size must match `floor((end - start + 6)/8)`.
///
/// NB: In theory, because the start and end values are encoded on integers of
/// any size specifiable by a varint, they can have huge values (`256^(0x3F_FF_FF_FF)`).
///
/// For ergonomy purpose, this library does not take those possibilities into consideration
/// and the start and end field are encoded on a `usize` which corresponds to the size of a
/// pointer on your architecture, and should be more than enough for IoT purpose. Indeed
/// if your goal is to transmit this payload over the air in an IoT context, chances are,
/// you will have trouble transmitting anything bigger than 256 bytes.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct MaskedRangeRef<'a> {
    start: usize,
    end: usize,
    bitmap: Option<&'a [u8]>,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum MaskedRangeNewError {
    BadBitmapLength { expected: usize },
}

impl<'a> MaskedRangeRef<'a> {
    /// # Safety
    /// If bitmap is defined you are to warrant that bitmap.len() == `floor((end - start + 6)/8)`.
    pub const unsafe fn new_unchecked(start: usize, end: usize, bitmap: Option<&'a [u8]>) -> Self {
        Self { start, end, bitmap }
    }

    pub const fn bitmap_size(start: usize, end: usize) -> usize {
        (end + 6 - start) / 8
    }

    /// # Errors
    /// Fails if the bitmap is defined and bitmap.len() != `floor((end - start + 6)/8)`.
    pub fn new(
        start: usize,
        end: usize,
        bitmap: Option<&'a [u8]>,
    ) -> Result<Self, MaskedRangeNewError> {
        if let Some(bitmap) = &bitmap {
            let bitmap_size = Self::bitmap_size(start, end);
            if bitmap.len() != bitmap_size {
                return Err(MaskedRangeNewError::BadBitmapLength {
                    expected: bitmap_size,
                });
            }
        }
        Ok(unsafe { Self::new_unchecked(start, end, bitmap) })
    }

    pub const fn start(&self) -> usize {
        self.start
    }

    pub const fn end(&self) -> usize {
        self.end
    }

    pub const fn bitmap(&self) -> Option<&'a [u8]> {
        self.bitmap
    }

    pub const fn boundaries_size(&self) -> u32 {
        if self.start > 0xFF_FF_FF || self.end > 0xFF_FF_FF {
            4
        } else if self.start > 0xFF_FF || self.end > 0xFF_FF {
            3
        } else if self.start > 0xFF || self.end > 0xFF {
            2
        } else {
            1
        }
    }

    #[cfg(feature = "alloc")]
    pub fn to_owned(&self) -> MaskedRange {
        MaskedRange {
            start: self.start,
            end: self.end,
            bitmap: self.bitmap.map(|bitmap| bitmap.into()),
        }
    }
}

#[cfg(feature = "alloc")]
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct MaskedRange {
    start: usize,
    end: usize,
    bitmap: Option<Box<[u8]>>,
}

#[cfg(feature = "alloc")]
impl MaskedRange {
    pub fn as_ref(&self) -> MaskedRangeRef {
        MaskedRangeRef {
            start: self.start,
            end: self.end,
            bitmap: self.bitmap.as_ref().map(|bitmap| bitmap.as_ref()),
        }
    }
}

use super::define::{QueryCode, QueryRangeComparisonType};
use crate::decodable::{Decodable, EncodedData, SizeError, WithByteSize};
use crate::define::{FileId, MaskedRangeRef};
use crate::v1_2::define::flag;
use crate::varint::{EncodedVarint, Varint};

#[cfg(feature = "alloc")]
use crate::define::MaskedRange;

/// Compares data to a range of data.
#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct ComparisonWithRangeRef<'item> {
    pub signed_data: bool,
    pub comparison_type: QueryRangeComparisonType,
    pub range: MaskedRangeRef<'item>,
    pub file_id: FileId,
    pub offset: Varint,
}

impl<'item> ComparisonWithRangeRef<'item> {
    /// Encodes the Item into a data pointer without checking the size of the
    /// receiving byte array.
    ///
    /// This method is meant to allow unchecked cross language wrapper libraries
    /// to implement an unchecked call without having to build a fake slice with
    /// a fake size.
    ///
    /// It is not meant to be used inside a Rust library/binary.
    ///
    /// # Safety
    /// You are responsible for checking that `out.len()` >= [`self.size()`](#method.size).
    ///
    /// Failing that will result in the program writing out of bound in
    /// random parts of your memory.
    pub unsafe fn encode_in_ptr(&self, out: *mut u8) -> usize {
        let mut size = 0;

        // Write query flags
        let mask_flag = match self.range.bitmap() {
            Some(_) => flag::QUERY_MASK,
            None => 0,
        };
        let signed_flag = if self.signed_data {
            flag::QUERY_SIGNED_DATA
        } else {
            0
        };
        *out.offset(0) = ((QueryCode::ComparisonWithRange as u8) << 5)
            | mask_flag
            | signed_flag
            | self.comparison_type as u8;
        size += 1;

        // Write compare_length
        let boundaries_size = self.range.boundaries_size();
        size += Varint::new_unchecked(boundaries_size).encode_in_ptr(out.add(size));
        let boundaries_size = boundaries_size as usize;

        // Write range boundaries
        // TODO SPEC What endianess???
        out.add(size)
            .copy_from(self.range.start().to_le_bytes().as_ptr(), boundaries_size);
        size += boundaries_size;
        out.add(size)
            .copy_from(self.range.end().to_le_bytes().as_ptr(), boundaries_size);
        size += boundaries_size;

        // Write bitmap
        if let Some(bitmap) = &self.range.bitmap() {
            out.add(size).copy_from(bitmap.as_ptr(), bitmap.len());
            size += bitmap.len();
        }

        *out.add(size) = self.file_id.u8();
        size += 1;
        size += self.offset.encode_in_ptr(out.add(size));

        size
    }

    /// Encodes the Item without checking the size of the receiving
    /// byte array.
    ///
    /// # Safety
    /// You are responsible for checking that `out.len()` >= [`self.size()`](#method.size).
    ///
    /// Failing that will result in the program writing out of bound in
    /// random parts of your memory.
    pub unsafe fn encode_in_unchecked(&self, out: &mut [u8]) -> usize {
        self.encode_in_ptr(out.as_mut_ptr())
    }

    /// Encodes the value into pre allocated array.
    ///
    /// # Errors
    /// Fails if the pre allocated array is smaller than [self.size()](#method.size)
    /// returning the number of input bytes required.
    pub fn encode_in(&self, out: &mut [u8]) -> Result<usize, usize> {
        let size = self.size();
        if out.len() >= size {
            Ok(unsafe { self.encode_in_ptr(out.as_mut_ptr()) })
        } else {
            Err(size)
        }
    }

    /// Size in bytes of the encoded equivalent of the item.
    pub fn size(&self) -> usize {
        unsafe {
            1 + Varint::new_unchecked(self.range.boundaries_size()).size()
                + match &self.range.bitmap() {
                    Some(bitmap) => bitmap.len(),
                    None => 0,
                }
                + 1
                + self.offset.size()
        }
    }

    #[cfg(feature = "alloc")]
    pub fn to_owned(&self) -> ComparisonWithRange {
        ComparisonWithRange {
            signed_data: self.signed_data,
            comparison_type: self.comparison_type,
            range: self.range.to_owned(),
            file_id: self.file_id,
            offset: self.offset,
        }
    }
}

pub struct EncodedComparisonWithRange<'data> {
    data: &'data [u8],
}

impl<'data> EncodedComparisonWithRange<'data> {
    pub fn mask_flag(&self) -> bool {
        unsafe { *self.data.get_unchecked(0) & flag::QUERY_MASK == flag::QUERY_MASK }
    }

    pub fn signed_data(&self) -> bool {
        unsafe { *self.data.get_unchecked(0) & flag::QUERY_SIGNED_DATA == flag::QUERY_SIGNED_DATA }
    }

    pub fn comparison_type(&self) -> QueryRangeComparisonType {
        unsafe {
            QueryRangeComparisonType::from_unchecked(
                *self.data.get_unchecked(0) & flag::QUERY_COMPARISON_TYPE,
            )
        }
    }

    pub fn compare_length(&self) -> EncodedVarint {
        unsafe { Varint::start_decoding_unchecked(self.data.get_unchecked(1..)) }
    }

    pub fn range_boundaries(&self) -> (usize, usize) {
        let WithByteSize {
            item: compare_length,
            byte_size: compare_length_size,
        } = self.compare_length().complete_decoding();
        let mut start_slice = 0_usize.to_le_bytes();
        let mut end_slice = 0_usize.to_le_bytes();
        let mut size = 1 + compare_length_size;
        unsafe {
            start_slice
                .as_mut_ptr()
                .copy_from(self.data.get_unchecked(size), compare_length.usize());
            size += compare_length.usize();
            end_slice
                .as_mut_ptr()
                .copy_from(self.data.get_unchecked(size), compare_length.usize());
        }
        (
            usize::from_le_bytes(start_slice),
            usize::from_le_bytes(end_slice),
        )
    }

    pub fn range(&self) -> MaskedRangeRef<'data> {
        unsafe {
            let WithByteSize {
                item: compare_length,
                byte_size: compare_length_size,
            } = self.compare_length().complete_decoding();
            let mut start_slice = 0_usize.to_le_bytes();
            let mut end_slice = 0_usize.to_le_bytes();
            let mut size = 1 + compare_length_size;
            start_slice
                .as_mut_ptr()
                .copy_from(self.data.get_unchecked(size), compare_length.usize());
            size += compare_length.usize();
            end_slice
                .as_mut_ptr()
                .copy_from(self.data.get_unchecked(size), compare_length.usize());
            size += compare_length.usize();
            let start = usize::from_le_bytes(start_slice);
            let end = usize::from_le_bytes(end_slice);

            let bitmap = if self.mask_flag() {
                let bitmap_size = MaskedRangeRef::bitmap_size(start, end);
                let bitmap =
                    core::slice::from_raw_parts(self.data.get_unchecked(size), bitmap_size);
                Some(bitmap)
            } else {
                None
            };
            MaskedRangeRef::new_unchecked(start, end, bitmap)
        }
    }

    pub fn file_id(&self) -> FileId {
        let WithByteSize {
            item: compare_length,
            byte_size: compare_length_size,
        } = self.compare_length().complete_decoding();
        let mut start_slice = 0_usize.to_le_bytes();
        let mut end_slice = 0_usize.to_le_bytes();
        let mut size = 1 + compare_length_size;
        unsafe {
            start_slice
                .as_mut_ptr()
                .copy_from(self.data.get_unchecked(size), compare_length.usize());
            size += compare_length.usize();
            end_slice
                .as_mut_ptr()
                .copy_from(self.data.get_unchecked(size), compare_length.usize());
            size += compare_length.usize();
            let start = usize::from_le_bytes(start_slice);
            let end = usize::from_le_bytes(end_slice);
            let bitmap_size = MaskedRangeRef::bitmap_size(start, end);
            size += bitmap_size;

            FileId(*self.data.get_unchecked(size))
        }
    }

    pub fn offset(&self) -> EncodedVarint {
        let WithByteSize {
            item: compare_length,
            byte_size: compare_length_size,
        } = self.compare_length().complete_decoding();
        let mut start_slice = 0_usize.to_le_bytes();
        let mut end_slice = 0_usize.to_le_bytes();
        let mut size = 1 + compare_length_size;
        unsafe {
            start_slice
                .as_mut_ptr()
                .copy_from(self.data.get_unchecked(size), compare_length.usize());
            size += compare_length.usize();
            end_slice
                .as_mut_ptr()
                .copy_from(self.data.get_unchecked(size), compare_length.usize());
            size += compare_length.usize();
            let start = usize::from_le_bytes(start_slice);
            let end = usize::from_le_bytes(end_slice);
            let bitmap_size = MaskedRangeRef::bitmap_size(start, end);
            size += bitmap_size;
            size += 1;

            Varint::start_decoding_unchecked(self.data.get_unchecked(size..))
        }
    }

    /// # Safety
    /// You are to warrant, somehow, that the input byte array contains a complete item.
    /// Else this might result in out of bound reads, and absurd results.
    pub unsafe fn size_unchecked(&self) -> usize {
        let WithByteSize {
            item: compare_length,
            byte_size: compare_length_size,
        } = self.compare_length().complete_decoding();
        let mut size = 1 + compare_length_size;
        if self.mask_flag() {
            let mut start_slice = 0_usize.to_le_bytes();
            let mut end_slice = 0_usize.to_le_bytes();
            start_slice
                .as_mut_ptr()
                .copy_from(self.data.get_unchecked(size), compare_length.usize());
            size += compare_length.usize();
            end_slice
                .as_mut_ptr()
                .copy_from(self.data.get_unchecked(size), compare_length.usize());
            size += compare_length.usize();
            let start = usize::from_le_bytes(start_slice);
            let end = usize::from_le_bytes(end_slice);
            let bitmap_size = MaskedRangeRef::bitmap_size(start, end);
            size += bitmap_size;
        } else {
            size += 2 * compare_length.usize();
        }
        size += 1;
        let decodable_offset = Varint::start_decoding_unchecked(self.data.get_unchecked(size..));
        size += decodable_offset.size_unchecked();
        size
    }
}

impl<'data> EncodedData<'data> for EncodedComparisonWithRange<'data> {
    type DecodedData = ComparisonWithRangeRef<'data>;
    unsafe fn new(data: &'data [u8]) -> Self {
        Self { data }
    }

    fn size(&self) -> Result<usize, SizeError> {
        unsafe {
            let mut size = 2;
            let data_size = self.data.len();
            if data_size < size {
                return Err(SizeError::MissingBytes);
            }
            let compare_length = self.compare_length();
            size = 1 + compare_length.size_unchecked();
            if data_size < size {
                return Err(SizeError::MissingBytes);
            }
            let compare_length = compare_length.complete_decoding().item.usize();

            size += 2 * compare_length;
            if data_size < size {
                return Err(SizeError::MissingBytes);
            }

            if self.mask_flag() {
                let mut start_slice = 0_usize.to_le_bytes();
                let mut end_slice = 0_usize.to_le_bytes();
                start_slice.as_mut_ptr().copy_from(
                    self.data.get_unchecked(size - 2 * compare_length),
                    compare_length,
                );
                end_slice.as_mut_ptr().copy_from(
                    self.data.get_unchecked(size - compare_length),
                    compare_length,
                );
                let start = usize::from_le_bytes(start_slice);
                let end = usize::from_le_bytes(end_slice);
                let bitmap_size = MaskedRangeRef::bitmap_size(start, end);
                size += bitmap_size;
            } else {
                size += compare_length;
            }
            size += 2;
            if data_size < size {
                return Err(SizeError::MissingBytes);
            }
            let decodable_offset =
                Varint::start_decoding_unchecked(self.data.get_unchecked(size - 1..));
            size += decodable_offset.size_unchecked();
            size -= 1;
            if data_size < size {
                return Err(SizeError::MissingBytes);
            }
            Ok(size)
        }
    }

    fn complete_decoding(&self) -> WithByteSize<ComparisonWithRangeRef<'data>> {
        unsafe {
            let WithByteSize {
                item: compare_length,
                byte_size: compare_length_size,
            } = self.compare_length().complete_decoding();
            let mut start_slice = 0_usize.to_le_bytes();
            let mut end_slice = 0_usize.to_le_bytes();
            let mut size = 1 + compare_length_size;
            start_slice
                .as_mut_ptr()
                .copy_from(self.data.get_unchecked(size), compare_length.usize());
            size += compare_length.usize();
            end_slice
                .as_mut_ptr()
                .copy_from(self.data.get_unchecked(size), compare_length.usize());
            size += compare_length.usize();
            let start = usize::from_le_bytes(start_slice);
            let end = usize::from_le_bytes(end_slice);

            let bitmap = if self.mask_flag() {
                let bitmap_size = MaskedRangeRef::bitmap_size(start, end);
                let bitmap =
                    core::slice::from_raw_parts(self.data.get_unchecked(size), bitmap_size);
                size += bitmap_size;
                Some(bitmap)
            } else {
                None
            };
            let range = MaskedRangeRef::new_unchecked(start, end, bitmap);

            let file_id = FileId(*self.data.get_unchecked(size));
            size += 1;
            let WithByteSize {
                item: offset,
                byte_size: offset_size,
            } = Varint::decode_unchecked(self.data.get_unchecked(size..));
            size += offset_size;

            WithByteSize {
                item: ComparisonWithRangeRef {
                    signed_data: self.signed_data(),
                    comparison_type: self.comparison_type(),
                    range,
                    file_id,
                    offset,
                },
                byte_size: size,
            }
        }
    }
}

impl<'data> Decodable<'data> for ComparisonWithRangeRef<'data> {
    type Data = EncodedComparisonWithRange<'data>;
}

/// Compares data to a range of data.
#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[cfg(feature = "alloc")]
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct ComparisonWithRange {
    pub signed_data: bool,
    pub comparison_type: QueryRangeComparisonType,
    pub range: MaskedRange,
    pub file_id: FileId,
    pub offset: Varint,
}

#[cfg(feature = "alloc")]
impl ComparisonWithRange {
    pub fn as_ref(&self) -> ComparisonWithRangeRef {
        ComparisonWithRangeRef {
            signed_data: self.signed_data,
            comparison_type: self.comparison_type,
            range: self.range.as_ref(),
            file_id: self.file_id,
            offset: self.offset,
        }
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::unwrap_in_result, clippy::panic, clippy::expect_used)]
    use super::*;

    #[test]
    fn known() {
        fn test(op: ComparisonWithRangeRef, data: &[u8]) {
            // Test op.encode_in() == data
            let mut encoded = [0_u8; 64];
            let size = op.encode_in(&mut encoded).unwrap();
            assert_eq!(size, data.len());
            assert_eq!(&encoded[..size], data);

            // Test decode(data) == op
            let WithByteSize {
                item: ret,
                byte_size: size,
            } = ComparisonWithRangeRef::decode(data).unwrap();
            assert_eq!(size, data.len());
            assert_eq!(ret, op);

            // Test partial_decode == op
            let WithByteSize {
                item: decoder,
                byte_size: expected_size,
            } = ComparisonWithRangeRef::start_decoding(data).unwrap();
            assert_eq!(ret.range.bitmap().is_some(), decoder.mask_flag());
            assert_eq!(
                ret.range.boundaries_size(),
                decoder.compare_length().complete_decoding().item.u32()
            );
            assert_eq!(
                (ret.range.start(), ret.range.end()),
                decoder.range_boundaries()
            );
            assert_eq!(ret.range, decoder.range());
            assert_eq!(expected_size, size);
            assert_eq!(unsafe { decoder.size_unchecked() }, size);
            assert_eq!(decoder.size().unwrap(), size);
            assert_eq!(
                op,
                ComparisonWithRangeRef {
                    signed_data: decoder.signed_data(),
                    comparison_type: decoder.comparison_type(),
                    range: decoder.range(),
                    file_id: decoder.file_id(),
                    offset: decoder.offset().complete_decoding().item,
                }
            );
        }
        test(
            ComparisonWithRangeRef {
                signed_data: true,
                comparison_type: QueryRangeComparisonType::InRange,
                range: MaskedRangeRef::new(0, 5, Some(&[0x11])).unwrap(),
                file_id: FileId::new(0x42),
                offset: Varint::new(0x40_00).unwrap(),
            },
            &[
                0x80 | 0x18 | 0x01,
                0x01,
                0x00,
                0x05,
                0x11,
                0x42,
                0x80,
                0x40,
                0x00,
            ],
        );
        test(
            ComparisonWithRangeRef {
                signed_data: false,
                comparison_type: QueryRangeComparisonType::NotInRange,
                range: MaskedRangeRef::new(50, 66, Some(&[0x33, 0x22])).unwrap(),
                file_id: FileId::new(0x88),
                offset: Varint::new(0xFF).unwrap(),
            },
            &[0x80 | 0x10, 0x01, 50, 66, 0x33, 0x22, 0x88, 0x40, 0xFF],
        );
    }

    #[test]
    fn consistence() {
        const TOT_SIZE: usize = 1 + 1 + 1 + 1 + 4 + 1 + 3;
        let op = ComparisonWithRangeRef {
            signed_data: true,
            comparison_type: QueryRangeComparisonType::InRange,
            range: MaskedRangeRef::new(0, 32, Some(&[0x33, 0x22, 0x33, 0x44])).unwrap(),
            file_id: FileId::new(0xFF),
            offset: Varint::new(0x3F_FF_00).unwrap(),
        };

        // Test decode(op.encode_in()) == op
        let mut encoded = [0_u8; TOT_SIZE];
        let size_encoded = op.encode_in(&mut encoded).unwrap();
        let WithByteSize {
            item: ret,
            byte_size: size_decoded,
        } = ComparisonWithRangeRef::decode(&encoded).unwrap();
        assert_eq!(size_encoded, size_decoded);
        assert_eq!(ret, op);

        // Test decode(data).encode_in() == data
        let mut encoded2 = [0_u8; TOT_SIZE];
        let size_encoded2 = op.encode_in(&mut encoded2).unwrap();
        assert_eq!(size_encoded, size_encoded2);
        assert_eq!(encoded2[..size_encoded2], encoded[..size_encoded]);
    }
}

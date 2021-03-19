use super::define::{QueryCode, QueryRangeComparisonType};
use crate::decodable::{
    Decodable, EncodedData, FailableDecodable, FailableEncodedData, WithByteSize,
};
use crate::define::{
    file_offset_operand::{EncodedFileOffsetOperand, EncodedFileOffsetOperandMut},
    FileId, MaskedRangeRef,
};
use crate::encodable::Encodable;
use crate::v1_2::define::flag;
use crate::v1_2::error::{
    QueryRangeError, QueryRangeSetError, QueryRangeSetLooselyError, QueryRangeSizeError,
};
use crate::varint::{EncodedVarint, EncodedVarintMut, Varint};

#[cfg(feature = "alloc")]
use crate::define::MaskedRange;

/// Compares data to a range of data.
#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct ComparisonWithRangeRef<'data> {
    pub signed_data: bool,
    pub comparison_type: QueryRangeComparisonType,
    pub range: MaskedRangeRef<'data>,
    pub file_id: FileId,
    pub offset: Varint,
}

impl<'data> Encodable for ComparisonWithRangeRef<'data> {
    unsafe fn encode_in_ptr(&self, out: *mut u8) -> usize {
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

    fn encoded_size(&self) -> usize {
        unsafe {
            1 + Varint::new_unchecked(self.range.boundaries_size()).encoded_size()
                + match &self.range.bitmap() {
                    Some(bitmap) => bitmap.len(),
                    None => 0,
                }
                + 1
                + self.offset.encoded_size()
        }
    }
}

impl<'data> ComparisonWithRangeRef<'data> {
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

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Range {
    pub compare_length: usize,
    pub start: usize,
    pub end: usize,
}

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
pub struct RangeWithFileOffset<'data> {
    pub masked_range: MaskedRangeRef<'data>,
    pub file_offset: EncodedFileOffsetOperand<'data>,
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

    // TODO Should this return an error if the Varint is bigger that the usize encoding on this
    // architecture?
    pub fn compare_length(&self) -> EncodedVarint<'data> {
        unsafe { Varint::start_decoding_unchecked(self.data.get_unchecked(1..)) }
    }

    pub fn range_boundaries_with_offset(&self) -> (Range, usize) {
        let WithByteSize {
            item: compare_length,
            byte_size: compare_length_size,
        } = self.compare_length().complete_decoding();
        // TODO SPEC: What is the endianness of these boundaries?
        let mut start_slice = 0_usize.to_le_bytes();
        let mut end_slice = 0_usize.to_le_bytes();
        let mut offset = 1 + compare_length_size;
        unsafe {
            start_slice
                .as_mut()
                .get_unchecked_mut(..compare_length.usize())
                .copy_from_slice(
                    self.data
                        .get_unchecked(offset..)
                        .get_unchecked(..compare_length.usize()),
                );
            offset += compare_length.usize();
            end_slice
                .as_mut()
                .get_unchecked_mut(..compare_length.usize())
                .copy_from_slice(
                    self.data
                        .get_unchecked(offset..)
                        .get_unchecked(..compare_length.usize()),
                );
            offset += compare_length.usize();
        }
        (
            Range {
                compare_length: unsafe { compare_length.usize() },
                start: usize::from_le_bytes(start_slice),
                end: usize::from_le_bytes(end_slice),
            },
            offset,
        )
    }

    pub fn range_boundaries(&self) -> Range {
        self.range_boundaries_with_offset().0
    }

    /// # Errors
    /// Fails if the encoded bitmap start > end.
    /// In that case, the bitmap size is negative, thus the bitmap cannot be decoded.
    fn range_with_post_offset(&self) -> Result<(MaskedRangeRef<'data>, usize), QueryRangeError> {
        unsafe {
            let (
                Range {
                    compare_length,
                    start,
                    end,
                },
                mut offset,
            ) = self.range_boundaries_with_offset();

            if start > end {
                return Err(QueryRangeError::BadEncodedRange);
            }

            let bitmap = if self.mask_flag() {
                let bitmap_size = MaskedRangeRef::bitmap_size(start, end);
                let bitmap = self
                    .data
                    .get_unchecked(offset..)
                    .get_unchecked(..bitmap_size);
                offset += bitmap_size;
                Some(bitmap)
            } else {
                None
            };
            Ok((
                MaskedRangeRef::new_unchecked(compare_length, start, end, bitmap),
                offset,
            ))
        }
    }

    /// # Errors
    /// Fails if the encoded bitmap start > end.
    /// In that case, the bitmap size is negative, thus the bitmap cannot be decoded.
    pub fn range_with_file_offset(&self) -> Result<RangeWithFileOffset<'data>, QueryRangeError> {
        self.range_with_post_offset()
            .map(|(masked_range, offset)| RangeWithFileOffset {
                masked_range,
                file_offset: EncodedFileOffsetOperand::new(unsafe {
                    self.data.get_unchecked(offset..)
                }),
            })
    }

    /// # Errors
    /// Fails if the encoded bitmap start > end.
    /// In that case, the bitmap size is negative, thus the bitmap cannot be decoded.
    pub fn range(&self) -> Result<MaskedRangeRef<'data>, QueryRangeError> {
        self.range_with_file_offset()
            .map(|RangeWithFileOffset { masked_range, .. }| masked_range)
    }
}

impl<'data> FailableEncodedData<'data> for EncodedComparisonWithRange<'data> {
    type SourceData = &'data [u8];
    type SizeError = QueryRangeSizeError;
    type DecodeError = QueryRangeError;
    type DecodedData = ComparisonWithRangeRef<'data>;

    unsafe fn new(data: Self::SourceData) -> Self {
        Self { data }
    }

    fn encoded_size(&self) -> Result<usize, Self::SizeError> {
        unsafe {
            let mut size = 2;
            let data_size = self.data.len();
            if data_size < size {
                return Err(Self::SizeError::MissingBytes);
            }
            let compare_length = self.compare_length();
            size = 1 + compare_length.encoded_size_unchecked();
            if data_size < size {
                return Err(Self::SizeError::MissingBytes);
            }
            let compare_length = compare_length.complete_decoding().item.usize();

            size += 2 * compare_length;
            if data_size < size {
                return Err(Self::SizeError::MissingBytes);
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
                if start > end {
                    return Err(Self::SizeError::BadEncodedRange);
                }
                let bitmap_size = MaskedRangeRef::bitmap_size(start, end);
                size += bitmap_size;
            } else {
                size += compare_length;
            }
            size += 2;
            if data_size < size {
                return Err(Self::SizeError::MissingBytes);
            }
            let decodable_offset =
                Varint::start_decoding_unchecked(self.data.get_unchecked(size - 1..));
            size += decodable_offset.encoded_size_unchecked();
            size -= 1;
            if data_size < size {
                return Err(Self::SizeError::MissingBytes);
            }
            Ok(size)
        }
    }

    fn complete_decoding(&self) -> Result<WithByteSize<Self::DecodedData>, Self::DecodeError> {
        unsafe {
            let (range, mut size) = self.range_with_post_offset()?;
            let file_offset = EncodedFileOffsetOperand::new(self.data.get_unchecked(size..));

            let file_id = file_offset.file_id();
            size += 1;
            let WithByteSize {
                item: offset,
                byte_size: offset_size,
            } = file_offset.offset().complete_decoding();
            size += offset_size;

            Ok(WithByteSize {
                item: ComparisonWithRangeRef {
                    signed_data: self.signed_data(),
                    comparison_type: self.comparison_type(),
                    range,
                    file_id,
                    offset,
                },
                byte_size: size,
            })
        }
    }
}

pub struct EncodedComparisonWithRangeMut<'data> {
    data: &'data mut [u8],
}

crate::make_downcastable!(EncodedComparisonWithRangeMut, EncodedComparisonWithRange);

impl<'data> EncodedComparisonWithRangeMut<'data> {
    pub fn mask_flag(&self) -> bool {
        self.as_ref().mask_flag()
    }

    pub fn signed_data(&self) -> bool {
        self.as_ref().signed_data()
    }

    pub fn comparison_type(&self) -> QueryRangeComparisonType {
        self.as_ref().comparison_type()
    }

    pub fn compare_length(&self) -> EncodedVarint {
        self.as_ref().compare_length()
    }

    pub fn range_boundaries(&self) -> Range {
        self.as_ref().range_boundaries()
    }

    /// # Errors
    /// Fails if the encoded bitmap start > end.
    /// In that case, the bitmap size is negative, thus the bitmap cannot be decoded.
    pub fn range_with_file_offset(&self) -> Result<RangeWithFileOffset<'data>, QueryRangeError> {
        self.as_ref().range_with_file_offset()
    }

    /// # Errors
    /// Fails if the encoded bitmap start > end.
    /// In that case, the bitmap size is negative, thus the bitmap cannot be decoded.
    pub fn range(&self) -> Result<MaskedRangeRef<'data>, QueryRangeError> {
        self.as_ref().range()
    }

    pub fn set_mask_flag(&mut self, mask_flag: bool) {
        if mask_flag {
            unsafe { *self.data.get_unchecked_mut(0) |= flag::QUERY_MASK }
        } else {
            unsafe { *self.data.get_unchecked_mut(0) &= !flag::QUERY_MASK }
        }
    }

    pub fn set_signed_data(&mut self, signed_data: bool) {
        if signed_data {
            unsafe { *self.data.get_unchecked_mut(0) |= flag::QUERY_SIGNED_DATA }
        } else {
            unsafe { *self.data.get_unchecked_mut(0) &= !flag::QUERY_SIGNED_DATA }
        }
    }

    pub fn set_comparison_type(&mut self, ty: QueryRangeComparisonType) {
        unsafe {
            *self.data.get_unchecked_mut(0) &= !flag::QUERY_COMPARISON_TYPE;
            *self.data.get_unchecked_mut(0) |= ty as u8;
        }
    }

    pub fn compare_length_mut(&mut self) -> EncodedVarintMut {
        unsafe { Varint::start_decoding_unchecked_mut(self.data.get_unchecked_mut(1..)) }
    }

    /// # Safety
    /// You have to guarantee:
    /// - That range.start <= range.end.
    /// - That the bitmap size in bytes remains the same before and after the boundary change.
    ///
    /// If you change the bitmap bit size (while keeping the same byte size) then make sure you
    /// know the state of the additional bits interpreted.
    pub unsafe fn set_range_boundaries_unchecked(&mut self, range: &Range) {
        let WithByteSize {
            item: compare_length,
            byte_size: compare_length_size,
        } = self.compare_length().complete_decoding();
        let mut offset = 1 + compare_length_size;

        let start_slice = range.start.to_le_bytes();
        let end_slice = range.end.to_le_bytes();

        self.data
            .get_unchecked_mut(offset..)
            .copy_from_slice(start_slice.get_unchecked(..compare_length.usize()));
        offset += compare_length.usize();
        self.data.get_unchecked_mut(offset..).copy_from_slice(
            end_slice
                .get_unchecked(offset..)
                .get_unchecked(..compare_length.usize()),
        );
    }

    /// # Errors
    /// Fails if the operand does not have the exact same size as the previous one.
    pub fn set_range_boundaries_loosely(
        &mut self,
        range: &Range,
    ) -> Result<(), QueryRangeSetLooselyError> {
        if range.start > range.end {
            return Err(QueryRangeSetLooselyError::BadGivenRange);
        }

        let Range {
            compare_length,
            start: current_start,
            end: current_end,
        } = self.range_boundaries();
        if current_start > current_end {
            return Err(QueryRangeSetLooselyError::BadEncodedRange);
        }
        let old_size = 2 * compare_length + MaskedRangeRef::bitmap_size(current_start, current_end);
        let new_size =
            2 * range.compare_length + MaskedRangeRef::bitmap_size(range.start, range.end);
        if new_size != old_size {
            return Err(QueryRangeSetLooselyError::ByteSizeMismatch);
        }
        unsafe { self.set_range_boundaries_unchecked(range) };
        Ok(())
    }

    /// # Errors
    /// Fails if the new bitmap size in bit differs from its original size.
    pub fn set_range_boundaries(&mut self, range: &Range) -> Result<(), QueryRangeSetError> {
        if range.start > range.end {
            return Err(QueryRangeSetError::BadGivenRange);
        }

        let Range {
            compare_length,
            start: current_start,
            end: current_end,
        } = self.range_boundaries();
        if current_start > current_end {
            return Err(QueryRangeSetError::BadEncodedRange);
        }
        if range.compare_length != compare_length {
            return Err(QueryRangeSetError::CompareLengthMismatch);
        }
        if range.end - range.start != current_end - current_start {
            return Err(QueryRangeSetError::BitmapBitSizeMismatch);
        }
        unsafe { self.set_range_boundaries_unchecked(range) };
        Ok(())
    }

    pub fn range_mask_mut(&mut self) -> Option<&mut [u8]> {
        if self.mask_flag() {
            let (Range { start, end, .. }, offset) = self.as_ref().range_boundaries_with_offset();
            let bitmap_size = MaskedRangeRef::bitmap_size(start, end);
            let bitmap = unsafe {
                self.data
                    .get_unchecked_mut(offset..)
                    .get_unchecked_mut(..bitmap_size)
            };
            Some(bitmap)
        } else {
            None
        }
    }

    /// # Errors
    /// Fails if the encoded bitmap start > end.
    /// In that case, the bitmap size is negative, thus the bitmap cannot be decoded.
    pub fn file_offset_mut(&mut self) -> Result<EncodedFileOffsetOperandMut, QueryRangeError> {
        let offset = self.as_ref().range_with_post_offset()?.1;
        Ok(EncodedFileOffsetOperandMut::new(unsafe {
            self.data.get_unchecked_mut(offset..)
        }))
    }
}

impl<'data> FailableEncodedData<'data> for EncodedComparisonWithRangeMut<'data> {
    type SourceData = &'data mut [u8];
    type SizeError = QueryRangeSizeError;
    type DecodeError = QueryRangeError;
    type DecodedData = ComparisonWithRangeRef<'data>;

    unsafe fn new(data: Self::SourceData) -> Self {
        Self { data }
    }

    fn encoded_size(&self) -> Result<usize, Self::SizeError> {
        self.as_ref().encoded_size()
    }

    fn complete_decoding(&self) -> Result<WithByteSize<Self::DecodedData>, Self::DecodeError> {
        self.as_ref().complete_decoding()
    }
}

impl<'data> FailableDecodable<'data> for ComparisonWithRangeRef<'data> {
    type Data = EncodedComparisonWithRange<'data>;
    type DataMut = EncodedComparisonWithRangeMut<'data>;
    type FullDecodeError = QueryRangeSizeError;
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
                Range {
                    compare_length: ret.range.compare_length(),
                    start: ret.range.start(),
                    end: ret.range.end()
                },
                decoder.range_boundaries()
            );
            assert_eq!(ret.range, decoder.range().unwrap());
            assert_eq!(expected_size, size);
            assert_eq!(decoder.encoded_size().unwrap(), size);
            assert_eq!(
                op,
                ComparisonWithRangeRef {
                    signed_data: decoder.signed_data(),
                    comparison_type: decoder.comparison_type(),
                    range: decoder.range().unwrap(),
                    file_id: decoder
                        .range_with_file_offset()
                        .unwrap()
                        .file_offset
                        .file_id(),
                    offset: decoder
                        .range_with_file_offset()
                        .unwrap()
                        .file_offset
                        .offset()
                        .complete_decoding()
                        .item,
                }
            );
        }
        test(
            ComparisonWithRangeRef {
                signed_data: true,
                comparison_type: QueryRangeComparisonType::InRange,
                range: MaskedRangeRef::new(1, 0, 5, Some(&[0x11])).unwrap(),
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
                range: MaskedRangeRef::new(1, 50, 66, Some(&[0x33, 0x22])).unwrap(),
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
            range: MaskedRangeRef::new(1, 0, 32, Some(&[0x33, 0x22, 0x33, 0x44])).unwrap(),
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

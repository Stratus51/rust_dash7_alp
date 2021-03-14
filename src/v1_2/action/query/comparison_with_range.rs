use super::define::{QueryCode, QueryRangeComparisonType};
use crate::decodable::{Decodable, EncodedData, SizeError, WithByteSize};
use crate::define::{FileId, MaskedRangeRef};
use crate::encodable::Encodable;
use crate::v1_2::define::flag;
use crate::varint::{EncodedVarint, EncodedVarintMut, Varint};

#[cfg(feature = "alloc")]
use crate::define::MaskedRange;

/// Compares data to a range of data.
#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct ComparisonWithRangeRef<'item, 'data> {
    pub signed_data: bool,
    pub comparison_type: QueryRangeComparisonType,
    pub range: MaskedRangeRef<'item, 'data>,
    pub file_id: FileId,
    pub offset: Varint,
}

impl<'item, 'data> Encodable for ComparisonWithRangeRef<'item, 'data> {
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

impl<'item, 'data> ComparisonWithRangeRef<'item, 'data> {
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

pub struct EncodedComparisonWithRange<'item, 'data> {
    data: &'item &'data [u8],
}

#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Range {
    pub start: usize,
    pub end: usize,
}

impl<'item, 'data> EncodedComparisonWithRange<'item, 'data> {
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

    pub fn compare_length<'result>(&self) -> EncodedVarint<'result, 'data> {
        unsafe { Varint::start_decoding_unchecked(self.data.get_unchecked(1..)) }
    }

    pub fn range_boundaries_with_offset(&self) -> (Range, usize) {
        let WithByteSize {
            item: compare_length,
            byte_size: compare_length_size,
        } = self.compare_length().complete_decoding();
        let mut start_slice = 0_usize.to_le_bytes();
        let mut end_slice = 0_usize.to_le_bytes();
        let mut offset = 1 + compare_length_size;
        unsafe {
            start_slice.as_mut().copy_from_slice(
                self.data
                    .get_unchecked(offset..)
                    .get_unchecked(..compare_length.usize()),
            );
            offset += compare_length.usize();
            end_slice.as_mut().copy_from_slice(
                self.data
                    .get_unchecked(offset..)
                    .get_unchecked(..compare_length.usize()),
            );
            offset += compare_length.usize();
        }
        (
            Range {
                start: usize::from_le_bytes(start_slice),
                end: usize::from_le_bytes(end_slice),
            },
            offset,
        )
    }

    pub fn range_boundaries(&self) -> Range {
        self.range_boundaries_with_offset().0
    }

    // TODO This method should fail if start > end. Thus the global decoding method should be failable.
    pub fn range<'result>(&self) -> MaskedRangeRef<'result, 'data> {
        unsafe {
            let (Range { start, end }, offset) = self.range_boundaries_with_offset();

            let bitmap = if self.mask_flag() {
                let bitmap_size = MaskedRangeRef::bitmap_size(start, end);
                let bitmap = self
                    .data
                    .get_unchecked(offset..)
                    .get_unchecked(..bitmap_size);
                Some(bitmap)
            } else {
                None
            };
            MaskedRangeRef::new_unchecked(start, end, bitmap)
        }
    }

    pub fn file_id_offset(&self) -> usize {
        if self.mask_flag() {
            let (Range { start, end }, mut offset) = self.range_boundaries_with_offset();
            let bitmap_size = MaskedRangeRef::bitmap_size(start, end);
            offset += bitmap_size;
            offset
        } else {
            let WithByteSize {
                item: compare_length,
                byte_size: compare_length_size,
            } = self.compare_length().complete_decoding();
            1 + compare_length_size + 2 * compare_length.usize()
        }
    }

    pub fn file_id(&self) -> FileId {
        let offset = self.file_id_offset();
        unsafe { FileId(*self.data.get_unchecked(offset)) }
    }

    pub fn offset<'result>(&self) -> EncodedVarint<'result, 'data> {
        let offset = self.file_id_offset() + 1;
        unsafe { Varint::start_decoding_unchecked(self.data.get_unchecked(offset..)) }
    }

    /// # Safety
    /// You have to warrant that somehow that there is enough byte to decode the encoded size.
    /// If you fail to do so, out of bound bytes will be read, and an absurd value will be
    /// returned.
    pub unsafe fn encoded_size_unchecked(&self) -> usize {
        let offset = self.file_id_offset();
        let mut size = offset + 1;
        let decodable_offset = Varint::start_decoding_unchecked(self.data.get_unchecked(size..));
        size += decodable_offset.encoded_size_unchecked();
        size
    }
}

impl<'item, 'data, 'result> EncodedData<'data, 'result>
    for EncodedComparisonWithRange<'item, 'data>
{
    type SourceData = &'data [u8];
    type DecodedData = ComparisonWithRangeRef<'result, 'data>;

    unsafe fn new(data: Self::SourceData) -> Self {
        Self { data }
    }

    fn encoded_size(&self) -> Result<usize, SizeError> {
        unsafe {
            let mut size = 2;
            let data_size = self.data.len();
            if data_size < size {
                return Err(SizeError::MissingBytes);
            }
            let compare_length = self.compare_length();
            size = 1 + compare_length.encoded_size_unchecked();
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
            size += decodable_offset.encoded_size_unchecked();
            size -= 1;
            if data_size < size {
                return Err(SizeError::MissingBytes);
            }
            Ok(size)
        }
    }

    fn complete_decoding(&self) -> WithByteSize<Self::DecodedData> {
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

pub struct EncodedComparisonWithRangeMut<'item, 'data> {
    data: &'item &'data mut [u8],
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum QueryRangeSetError {
    /// The bitmap bit size calculated with the given range does not match the size of the encoded
    /// bitmap.
    BitmapBitSizeMismatch,
    /// The given range is invalid because, range.start > range.end
    BadGivenRange,
    /// The encoded range is invalid because, range.start > range.end
    BadEncodedRange,
}

impl<'item, 'data> EncodedComparisonWithRangeMut<'item, 'data> {
    pub fn as_ref<'result>(&self) -> EncodedComparisonWithRange<'result, 'data> {
        unsafe { EncodedComparisonWithRange::new(self.data) }
    }

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

    pub fn range<'result>(&self) -> MaskedRangeRef<'result, 'data> {
        self.as_ref().range()
    }

    pub fn file_id(&self) -> FileId {
        self.as_ref().file_id()
    }

    pub fn offset(&self) -> EncodedVarint {
        self.as_ref().offset()
    }

    /// # Safety
    /// You have to warrant that somehow that there is enough byte to decode the encoded size.
    /// If you fail to do so, out of bound bytes will be read, and an absurd value will be
    /// returned.
    pub unsafe fn encoded_size_unchecked(&self) -> usize {
        self.as_ref().encoded_size_unchecked()
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

    pub fn compare_length_mut<'result>(&mut self) -> EncodedVarintMut<'result, 'data> {
        unsafe { Varint::start_decoding_unchecked_mut(self.data.get_unchecked_mut(1..)) }
    }

    /// # Safety
    /// You have to guarantee:
    /// - That range.start <= range.end.
    /// - That the bitmap size in bytes remains the same before and after the boundary change.
    ///
    /// If you change the bitmap bit size (while keeping the same byte size) then make sure you
    /// know the state of the additional bits interpreted.
    pub unsafe fn set_range_boundaries_unchecked(&self, range: &Range) {
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
    /// Fails if the new bitmap size in bit differs from its original size.
    pub fn set_range_boundaries(&self, range: &Range) -> Result<(), QueryRangeSetError> {
        if range.start > range.end {
            return Err(QueryRangeSetError::BadGivenRange);
        }
        let bitmap_size = MaskedRangeRef::bitmap_size(range.start, range.end);

        let Range {
            start: current_start,
            end: current_end,
        } = self.range_boundaries();
        if current_start > current_end {
            return Err(QueryRangeSetError::BadGivenRange);
        }
        if range.end - range.start != current_end - current_start {
            return Err(QueryRangeSetError::BitmapBitSizeMismatch);
        }
        unsafe { self.set_range_boundaries_unchecked(range) };
        Ok(())
    }

    pub fn range_mask_mut(&mut self) -> Option<&'data mut [u8]> {
        if self.mask_flag() {
            let (Range { start, end }, offset) = self.as_ref().range_boundaries_with_offset();
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

    pub fn set_file_id(&mut self, file_id: FileId) {
        let offset = self.as_ref().file_id_offset();
        *self.data.get_unchecked_mut(offset) = file_id.u8();
    }

    pub fn offset_mut<'result>(&mut self) -> EncodedVarintMut<'result, 'data> {
        let offset = self.as_ref().file_id_offset() + 1;
        unsafe { Varint::start_decoding_unchecked_mut(self.data.get_unchecked_mut(offset..)) }
    }
}

impl<'item, 'data, 'result> EncodedData<'data, 'result>
    for EncodedComparisonWithRangeMut<'item, 'data>
{
    type SourceData = &'data mut [u8];
    type DecodedData = ComparisonWithRangeRef<'result, 'data>;

    unsafe fn new(data: Self::SourceData) -> Self {
        Self { data }
    }

    fn encoded_size(&self) -> Result<usize, SizeError> {
        self.as_ref().encoded_size()
    }

    fn complete_decoding(&self) -> WithByteSize<Self::DecodedData> {
        self.as_ref().complete_decoding()
    }
}

impl<'item, 'data, 'result> Decodable<'data, 'result> for ComparisonWithRangeRef<'item, 'data> {
    type Data = EncodedComparisonWithRange<'item, 'data>;
    type DataMut = EncodedComparisonWithRangeMut<'item, 'data>;
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
                    start: ret.range.start(),
                    end: ret.range.end()
                },
                decoder.range_boundaries()
            );
            assert_eq!(ret.range, decoder.range());
            assert_eq!(expected_size, size);
            assert_eq!(unsafe { decoder.encoded_size_unchecked() }, size);
            assert_eq!(decoder.encoded_size().unwrap(), size);
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

use super::super::super::define::flag;
use super::super::super::error::QueryOperandDecodeError;
use super::define::{QueryCode, QueryRangeComparisonType};
use crate::define::{FileId, MaskedRange};
use crate::varint::{DecodableVarint, Varint};

/// Writes data to a file.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct ComparisonWithRange<'item> {
    pub signed_data: bool,
    pub comparison_type: QueryRangeComparisonType,
    pub range: MaskedRange<'item>,
    pub file_id: FileId,
    pub offset: Varint,
}

impl<'item> ComparisonWithRange<'item> {
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

    /// Creates a decodable item from a data pointer without checking the data size.
    ///
    /// This method is meant to allow unchecked cross language wrapper libraries
    /// to implement an unchecked call without having to build a fake slice with
    /// a fake size.
    ///
    /// It is not meant to be used inside a Rust library/binary.
    ///
    /// # Safety
    /// You are to check that:
    /// - The first byte contains this action's querycode.
    /// - The decodable object fits in the given data:
    /// [`decodable.smaller_than(data.len())`](struct.DecodableComparisonWithRange.html#method.smaller_than)
    ///
    /// Failing that might result in reading and interpreting data outside the given
    /// array (depending on what is done with the resulting object).
    pub const unsafe fn start_decoding_ptr<'data>(
        data: *const u8,
    ) -> DecodableComparisonWithRange<'data> {
        DecodableComparisonWithRange::from_ptr(data)
    }

    /// Creates a decodable item without checking the data size.
    ///
    /// # Safety
    /// You are to check that:
    /// - The first byte contains this action's querycode.
    /// - The decodable object fits in the given data:
    /// [`decodable.smaller_than(data.len())`](struct.DecodableComparisonWithRange.html#method.smaller_than)
    ///
    /// Failing that might result in reading and interpreting data outside the given
    /// array (depending on what is done with the resulting object).
    pub const unsafe fn start_decoding_unchecked(data: &[u8]) -> DecodableComparisonWithRange {
        DecodableComparisonWithRange::new(data)
    }

    /// Returns a Decodable object and its expected byte size.
    ///
    /// This decodable item allows each parts of the item to be decoded independently.
    ///
    /// # Errors
    /// - Fails if first byte of the data contains the wrong querycode.
    /// - Fails if data is smaller then the decoded expected size.
    pub fn start_decoding(
        data: &[u8],
    ) -> Result<(DecodableComparisonWithRange, usize), QueryOperandDecodeError> {
        match data.get(0) {
            None => return Err(QueryOperandDecodeError::MissingBytes(1)),
            Some(byte) => {
                let code = *byte >> 5;
                if code != QueryCode::ComparisonWithRange as u8 {
                    return Err(QueryOperandDecodeError::UnknownQueryCode(code));
                }
            }
        }
        let ret = unsafe { Self::start_decoding_unchecked(data) };
        let size = ret
            .smaller_than(data.len())
            .map_err(QueryOperandDecodeError::MissingBytes)?;
        Ok((ret, size))
    }

    /// Decodes the Item from a data pointer.
    ///
    /// Returns the decoded data and the number of bytes consumed to produce it.
    ///
    /// This method is meant to allow unchecked cross language wrapper libraries
    /// to implement an unchecked call without having to build a fake slice with
    /// a fake size.
    ///
    /// It is not meant to be used inside a Rust library/binary.
    ///
    /// # Safety
    /// You are to check that:
    /// - The first byte contains this action's querycode.
    /// - The resulting size of the data consumed is smaller than the size of the
    /// decoded data.
    ///
    /// Failing that might result in reading and interpreting data outside the given
    /// array (depending on what is done with the resulting object).
    pub unsafe fn decode_ptr(data: *const u8) -> (Self, usize) {
        Self::start_decoding_ptr(data).complete_decoding()
    }

    /// Decodes the Item from bytes.
    ///
    /// Returns the decoded data and the number of bytes consumed to produce it.
    ///
    /// # Safety
    /// You are to check that:
    /// - The first byte contains this action's querycode.
    /// - The resulting size of the data consumed is smaller than the size of the
    /// decoded data.
    ///
    /// Failing that might result in reading and interpreting data outside the given
    /// array (depending on what is done with the resulting object).
    pub unsafe fn decode_unchecked(data: &'item [u8]) -> (Self, usize) {
        Self::start_decoding_unchecked(data).complete_decoding()
    }

    /// Decodes the item from bytes.
    ///
    /// On success, returns the decoded data and the number of bytes consumed
    /// to produce it.
    ///
    /// # Errors
    /// - Fails if first byte of the data contains the wrong querycode.
    /// - Fails if data is smaller then the decoded expected size.
    pub fn decode(data: &'item [u8]) -> Result<(Self, usize), QueryOperandDecodeError> {
        match Self::start_decoding(data) {
            Ok(v) => Ok(v.0.complete_decoding()),
            Err(e) => Err(e),
        }
    }
}

pub struct DecodableComparisonWithRange<'data> {
    data: *const u8,
    data_life: core::marker::PhantomData<&'data ()>,
}

impl<'data> DecodableComparisonWithRange<'data> {
    const fn new(data: &'data [u8]) -> Self {
        Self::from_ptr(data.as_ptr())
    }

    const fn from_ptr(data: *const u8) -> Self {
        Self {
            data,
            data_life: core::marker::PhantomData,
        }
    }

    /// Decodes the size of the Item in bytes
    ///
    /// # Safety
    /// This requires reading the data bytes that may be out of bound to be calculate.
    pub unsafe fn expected_size(&self) -> usize {
        let (compare_length, compare_length_size) = self.compare_length().complete_decoding();
        let mut size = 1 + compare_length_size;
        if self.mask_flag() {
            let mut start_slice = 0_usize.to_le_bytes();
            let mut end_slice = 0_usize.to_le_bytes();
            start_slice
                .as_mut_ptr()
                .copy_from(self.data.add(size), compare_length.usize());
            size += compare_length.usize();
            end_slice
                .as_mut_ptr()
                .copy_from(self.data.add(size), compare_length.usize());
            size += compare_length.usize();
            let start = usize::from_le_bytes(start_slice);
            let end = usize::from_le_bytes(end_slice);
            let bitmap_size = MaskedRange::bitmap_size(start, end);
            size += bitmap_size;
        } else {
            size += 2 * compare_length.usize();
        }
        size += 1;
        let decodable_offset = Varint::start_decoding_ptr(self.data.add(size));
        size += decodable_offset.expected_size();
        size
    }

    /// Checks whether the given data_size is bigger than the decoded object expected size.
    ///
    /// On success, returns the size of the decoded object.
    ///
    /// # Errors
    /// Fails if the data_size is smaller than the required data size to decode the object.
    pub fn smaller_than(&self, data_size: usize) -> Result<usize, usize> {
        unsafe {
            let mut size = 2;
            if data_size < size {
                return Err(size);
            }
            let compare_length = self.compare_length();
            size = 1 + compare_length.expected_size();
            if data_size < size {
                return Err(size);
            }
            let compare_length = compare_length.complete_decoding().0.usize();

            size += 2 * compare_length;
            if data_size < size {
                return Err(size);
            }

            if self.mask_flag() {
                let mut start_slice = 0_usize.to_le_bytes();
                let mut end_slice = 0_usize.to_le_bytes();
                start_slice
                    .as_mut_ptr()
                    .copy_from(self.data.add(size - 2 * compare_length), compare_length);
                end_slice
                    .as_mut_ptr()
                    .copy_from(self.data.add(size - compare_length), compare_length);
                let start = usize::from_le_bytes(start_slice);
                let end = usize::from_le_bytes(end_slice);
                let bitmap_size = MaskedRange::bitmap_size(start, end);
                size += bitmap_size;
            } else {
                size += compare_length;
            }
            size += 2;
            if data_size < size {
                return Err(size);
            }
            let decodable_offset = Varint::start_decoding_ptr(self.data.add(size - 1));
            size += decodable_offset.expected_size();
            size -= 1;
            if data_size < size {
                return Err(size);
            }
            Ok(size)
        }
    }

    pub fn mask_flag(&self) -> bool {
        unsafe { *self.data.add(0) & flag::QUERY_MASK == flag::QUERY_MASK }
    }

    pub fn signed_data(&self) -> bool {
        unsafe { *self.data.add(0) & flag::QUERY_SIGNED_DATA == flag::QUERY_SIGNED_DATA }
    }

    pub fn comparison_type(&self) -> QueryRangeComparisonType {
        unsafe {
            QueryRangeComparisonType::from_unchecked(
                *self.data.add(0) & flag::QUERY_COMPARISON_TYPE,
            )
        }
    }

    pub fn compare_length(&self) -> DecodableVarint {
        unsafe { Varint::start_decoding_ptr(self.data.add(1)) }
    }

    pub fn range_boundaries(&self) -> (usize, usize) {
        let (compare_length, compare_length_size) = self.compare_length().complete_decoding();
        let mut start_slice = 0_usize.to_le_bytes();
        let mut end_slice = 0_usize.to_le_bytes();
        let mut size = 1 + compare_length_size;
        unsafe {
            start_slice
                .as_mut_ptr()
                .copy_from(self.data.add(size), compare_length.usize());
            size += compare_length.usize();
            end_slice
                .as_mut_ptr()
                .copy_from(self.data.add(size), compare_length.usize());
        }
        (
            usize::from_le_bytes(start_slice),
            usize::from_le_bytes(end_slice),
        )
    }

    pub fn range(&self) -> MaskedRange<'data> {
        unsafe {
            let (compare_length, compare_length_size) = self.compare_length().complete_decoding();
            let mut start_slice = 0_usize.to_le_bytes();
            let mut end_slice = 0_usize.to_le_bytes();
            let mut size = 1 + compare_length_size;
            start_slice
                .as_mut_ptr()
                .copy_from(self.data.add(size), compare_length.usize());
            size += compare_length.usize();
            end_slice
                .as_mut_ptr()
                .copy_from(self.data.add(size), compare_length.usize());
            size += compare_length.usize();
            let start = usize::from_le_bytes(start_slice);
            let end = usize::from_le_bytes(end_slice);

            let bitmap = if self.mask_flag() {
                let bitmap_size = MaskedRange::bitmap_size(start, end);
                let bitmap = core::slice::from_raw_parts(self.data.add(size), bitmap_size);
                Some(bitmap)
            } else {
                None
            };
            MaskedRange::new_unchecked(start, end, bitmap)
        }
    }

    pub fn file_id(&self) -> FileId {
        let (compare_length, compare_length_size) = self.compare_length().complete_decoding();
        let mut start_slice = 0_usize.to_le_bytes();
        let mut end_slice = 0_usize.to_le_bytes();
        let mut size = 1 + compare_length_size;
        unsafe {
            start_slice
                .as_mut_ptr()
                .copy_from(self.data.add(size), compare_length.usize());
            size += compare_length.usize();
            end_slice
                .as_mut_ptr()
                .copy_from(self.data.add(size), compare_length.usize());
            size += compare_length.usize();
            let start = usize::from_le_bytes(start_slice);
            let end = usize::from_le_bytes(end_slice);
            let bitmap_size = MaskedRange::bitmap_size(start, end);
            size += bitmap_size;

            FileId(*self.data.add(size))
        }
    }

    pub fn offset(&self) -> DecodableVarint {
        let (compare_length, compare_length_size) = self.compare_length().complete_decoding();
        let mut start_slice = 0_usize.to_le_bytes();
        let mut end_slice = 0_usize.to_le_bytes();
        let mut size = 1 + compare_length_size;
        unsafe {
            start_slice
                .as_mut_ptr()
                .copy_from(self.data.add(size), compare_length.usize());
            size += compare_length.usize();
            end_slice
                .as_mut_ptr()
                .copy_from(self.data.add(size), compare_length.usize());
            size += compare_length.usize();
            let start = usize::from_le_bytes(start_slice);
            let end = usize::from_le_bytes(end_slice);
            let bitmap_size = MaskedRange::bitmap_size(start, end);
            size += bitmap_size;
            size += 1;

            Varint::start_decoding_ptr(self.data.add(size))
        }
    }

    /// Fully decode the Item
    ///
    /// Returns the decoded data and the number of bytes consumed to produce it.
    pub fn complete_decoding(&self) -> (ComparisonWithRange<'data>, usize) {
        unsafe {
            let (compare_length, compare_length_size) = self.compare_length().complete_decoding();
            let mut start_slice = 0_usize.to_le_bytes();
            let mut end_slice = 0_usize.to_le_bytes();
            let mut size = 1 + compare_length_size;
            start_slice
                .as_mut_ptr()
                .copy_from(self.data.add(size), compare_length.usize());
            size += compare_length.usize();
            end_slice
                .as_mut_ptr()
                .copy_from(self.data.add(size), compare_length.usize());
            size += compare_length.usize();
            let start = usize::from_le_bytes(start_slice);
            let end = usize::from_le_bytes(end_slice);

            let bitmap = if self.mask_flag() {
                let bitmap_size = MaskedRange::bitmap_size(start, end);
                let bitmap = core::slice::from_raw_parts(self.data.add(size), bitmap_size);
                size += bitmap_size;
                Some(bitmap)
            } else {
                None
            };
            let range = MaskedRange::new_unchecked(start, end, bitmap);

            let file_id = FileId(*self.data.add(size));
            size += 1;
            let (offset, offset_size) = Varint::decode_ptr(self.data.add(size));
            size += offset_size;

            (
                ComparisonWithRange {
                    signed_data: self.signed_data(),
                    comparison_type: self.comparison_type(),
                    range,
                    file_id,
                    offset,
                },
                size,
            )
        }
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::unwrap_in_result, clippy::panic, clippy::expect_used)]
    use super::*;

    #[test]
    fn known() {
        fn test(op: ComparisonWithRange, data: &[u8]) {
            // Test op.encode_in() == data
            let mut encoded = [0_u8; 64];
            let size = op.encode_in(&mut encoded).unwrap();
            assert_eq!(size, data.len());
            assert_eq!(&encoded[..size], data);

            // Test decode(data) == op
            let (ret, size) = ComparisonWithRange::decode(data).unwrap();
            assert_eq!(size, data.len());
            assert_eq!(ret, op);

            // Test partial_decode == op
            let (decoder, expected_size) = ComparisonWithRange::start_decoding(data).unwrap();
            assert_eq!(ret.range.bitmap().is_some(), decoder.mask_flag());
            assert_eq!(
                ret.range.boundaries_size(),
                decoder.compare_length().complete_decoding().0.u32()
            );
            assert_eq!(
                (ret.range.start(), ret.range.end()),
                decoder.range_boundaries()
            );
            assert_eq!(ret.range, decoder.range());
            assert_eq!(expected_size, size);
            assert_eq!(unsafe { decoder.expected_size() }, size);
            assert_eq!(decoder.smaller_than(data.len()).unwrap(), size);
            assert_eq!(
                op,
                ComparisonWithRange {
                    signed_data: decoder.signed_data(),
                    comparison_type: decoder.comparison_type(),
                    range: decoder.range(),
                    file_id: decoder.file_id(),
                    offset: decoder.offset().complete_decoding().0,
                }
            );
        }
        test(
            ComparisonWithRange {
                signed_data: true,
                comparison_type: QueryRangeComparisonType::InRange,
                range: MaskedRange::new(0, 5, Some(&[0x11])).unwrap(),
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
            ComparisonWithRange {
                signed_data: false,
                comparison_type: QueryRangeComparisonType::NotInRange,
                range: MaskedRange::new(50, 66, Some(&[0x33, 0x22])).unwrap(),
                file_id: FileId::new(0x88),
                offset: Varint::new(0xFF).unwrap(),
            },
            &[0x80 | 0x10, 0x01, 50, 66, 0x33, 0x22, 0x88, 0x40, 0xFF],
        );
    }

    #[test]
    fn consistence() {
        const TOT_SIZE: usize = 1 + 1 + 1 + 1 + 4 + 1 + 3;
        let op = ComparisonWithRange {
            signed_data: true,
            comparison_type: QueryRangeComparisonType::InRange,
            range: MaskedRange::new(0, 32, Some(&[0x33, 0x22, 0x33, 0x44])).unwrap(),
            file_id: FileId::new(0xFF),
            offset: Varint::new(0x3F_FF_00).unwrap(),
        };

        // Test decode(op.encode_in()) == op
        let mut encoded = [0_u8; TOT_SIZE];
        let size_encoded = op.encode_in(&mut encoded).unwrap();
        let (ret, size_decoded) = ComparisonWithRange::decode(&encoded).unwrap();
        assert_eq!(size_encoded, size_decoded);
        assert_eq!(ret, op);

        // Test decode(data).encode_in() == data
        let mut encoded2 = [0_u8; TOT_SIZE];
        let size_encoded2 = op.encode_in(&mut encoded2).unwrap();
        assert_eq!(size_encoded, size_encoded2);
        assert_eq!(encoded2[..size_encoded2], encoded[..size_encoded]);
    }
}

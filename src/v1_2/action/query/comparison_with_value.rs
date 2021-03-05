use super::super::super::define::flag;
use super::define::{QueryCode, QueryComparisonType};
use crate::decodable::{Decodable, EncodedData, WithByteSize};
use crate::define::{EncodableDataRef, FileId, MaskedValueRef};
use crate::varint::{EncodedVarint, Varint};

#[cfg(feature = "alloc")]
use crate::define::MaskedValue;

/// Compares data to a value.
#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct ComparisonWithValueRef<'item> {
    pub signed_data: bool,
    pub comparison_type: QueryComparisonType,
    pub compare_value: MaskedValueRef<'item>,
    pub file_id: FileId,
    pub offset: Varint,
}

impl<'item> ComparisonWithValueRef<'item> {
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
        let mask_flag = match self.compare_value.mask() {
            Some(_) => flag::QUERY_MASK,
            None => 0,
        };
        let signed_flag = if self.signed_data {
            flag::QUERY_SIGNED_DATA
        } else {
            0
        };
        *out.offset(0) = ((QueryCode::ComparisonWithValue as u8) << 5)
            | mask_flag
            | signed_flag
            | self.comparison_type as u8;
        size += 1;

        // Write compare_length
        size += Varint::new_unchecked(self.compare_value.len() as u32).encode_in_ptr(out.add(size));

        // Write value mask
        if let Some(mask) = &self.compare_value.mask() {
            out.add(size).copy_from(mask.as_ptr(), mask.len());
            size += mask.len();
        }

        // Write value
        out.add(size).copy_from(
            self.compare_value.value().as_ptr(),
            self.compare_value.len(),
        );
        size += self.compare_value.len();

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
    /// Fails if the pre allocated array is smaller than [`self.size()`](#method.size)
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
            1 + Varint::new_unchecked(self.compare_value.len() as u32).size()
                + match &self.compare_value.mask() {
                    Some(mask) => mask.len(),
                    None => 0,
                }
                + self.compare_value.len()
                + 1
                + self.offset.size()
        }
    }

    #[cfg(feature = "alloc")]
    pub fn to_owned(&self) -> ComparisonWithValue {
        ComparisonWithValue {
            signed_data: self.signed_data,
            comparison_type: self.comparison_type,
            compare_value: self.compare_value.to_owned(),
            file_id: self.file_id,
            offset: self.offset,
        }
    }
}

pub struct EncodedComparisonWithValue<'data> {
    data: *const u8,
    data_life: core::marker::PhantomData<&'data ()>,
}

impl<'data> EncodedComparisonWithValue<'data> {
    pub fn mask_flag(&self) -> bool {
        unsafe { *self.data.add(0) & flag::QUERY_MASK == flag::QUERY_MASK }
    }

    pub fn signed_data(&self) -> bool {
        unsafe { *self.data.add(0) & flag::QUERY_SIGNED_DATA == flag::QUERY_SIGNED_DATA }
    }

    pub fn comparison_type(&self) -> QueryComparisonType {
        unsafe {
            QueryComparisonType::from_unchecked(*self.data.add(0) & flag::QUERY_COMPARISON_TYPE)
        }
    }

    pub fn compare_length(&self) -> EncodedVarint {
        unsafe { Varint::start_decoding_ptr(self.data.add(1)) }
    }

    pub fn mask(&self) -> Option<&'data [u8]> {
        if self.mask_flag() {
            let WithByteSize {
                item: compare_length,
                byte_size: compare_length_size,
            } = self.compare_length().complete_decoding();
            let mask = unsafe {
                core::slice::from_raw_parts(
                    self.data.add(1 + compare_length_size),
                    compare_length.u32() as usize,
                )
            };
            Some(mask)
        } else {
            None
        }
    }

    pub fn value(&self) -> EncodableDataRef<'data> {
        let WithByteSize {
            item: compare_length,
            byte_size: compare_length_size,
        } = self.compare_length().complete_decoding();
        let mut offset = 1 + compare_length_size;
        if self.mask_flag() {
            offset += compare_length.u32() as usize;
        }
        unsafe {
            EncodableDataRef::new_unchecked(core::slice::from_raw_parts(
                self.data.add(offset),
                compare_length.u32() as usize,
            ))
        }
    }

    pub fn compare_value(&self) -> MaskedValueRef<'data> {
        let WithByteSize {
            item: compare_length,
            byte_size: compare_length_size,
        } = self.compare_length().complete_decoding();
        let compare_length = compare_length.u32() as usize;
        let mut offset = 1 + compare_length_size;
        unsafe {
            let mask = if self.mask_flag() {
                let mask = core::slice::from_raw_parts(self.data.add(offset), compare_length);
                offset += compare_length;
                Some(mask)
            } else {
                None
            };
            let value = EncodableDataRef::new_unchecked(core::slice::from_raw_parts(
                self.data.add(offset),
                compare_length,
            ));
            MaskedValueRef::new_unchecked(value, mask)
        }
    }

    pub fn file_id(&self) -> FileId {
        let WithByteSize {
            item: compare_length,
            byte_size: compare_length_size,
        } = self.compare_length().complete_decoding();
        let compare_length = compare_length.u32() as usize;
        let value_offset = if self.mask_flag() {
            1 + compare_length_size + 2 * compare_length
        } else {
            1 + compare_length_size + compare_length
        };
        unsafe { FileId(*self.data.add(value_offset)) }
    }

    pub fn offset(&self) -> EncodedVarint {
        let WithByteSize {
            item: compare_length,
            byte_size: compare_length_size,
        } = self.compare_length().complete_decoding();
        let compare_length = compare_length.u32() as usize;
        let value_offset = if self.mask_flag() {
            1 + compare_length_size + 2 * compare_length
        } else {
            1 + compare_length_size + compare_length
        };
        unsafe { Varint::start_decoding_ptr(self.data.add(value_offset + 1)) }
    }
}

impl<'data> EncodedData<'data> for EncodedComparisonWithValue<'data> {
    type DecodedData = ComparisonWithValueRef<'data>;
    unsafe fn from_data_ref(data: &'data [u8]) -> Self {
        Self::from_data_ptr(data.as_ptr())
    }

    unsafe fn from_data_ptr(data: *const u8) -> Self {
        Self {
            data,
            data_life: core::marker::PhantomData,
        }
    }

    unsafe fn expected_size(&self) -> usize {
        let WithByteSize {
            item: compare_length,
            byte_size: compare_length_size,
        } = self.compare_length().complete_decoding();
        let compare_length = compare_length.u32() as usize;
        let value_offset = if self.mask_flag() {
            1 + compare_length_size + 2 * compare_length
        } else {
            1 + compare_length_size + compare_length
        };
        let decodable_offset = Varint::start_decoding_ptr(self.data.add(value_offset + 1));
        value_offset + 1 + decodable_offset.expected_size()
    }

    fn smaller_than(&self, data_size: usize) -> Result<usize, usize> {
        unsafe {
            let mut size = 2;
            if data_size < size {
                return Err(size);
            }
            let compare_length = self.compare_length();
            size += compare_length.expected_size();
            if data_size < size {
                return Err(size);
            }
            let compare_length = compare_length.complete_decoding().item.usize();
            if self.mask_flag() {
                size += compare_length;
            }
            size += compare_length;
            size += 1;
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

    fn complete_decoding(&self) -> WithByteSize<ComparisonWithValueRef<'data>> {
        unsafe {
            let WithByteSize {
                item: compare_length,
                byte_size: compare_length_size,
            } = self.compare_length().complete_decoding();
            let compare_length = compare_length.u32() as usize;
            let mut size = 1 + compare_length_size;
            let mask = if self.mask_flag() {
                let mask = core::slice::from_raw_parts(self.data.add(size), compare_length);
                size += compare_length;
                Some(mask)
            } else {
                None
            };
            let value = EncodableDataRef::new_unchecked(core::slice::from_raw_parts(
                self.data.add(size),
                compare_length,
            ));
            size += compare_length;
            let file_id = FileId(*self.data.add(size));
            size += 1;
            let WithByteSize {
                item: offset,
                byte_size: offset_size,
            } = Varint::decode_ptr(self.data.add(size));
            size += offset_size;

            WithByteSize {
                item: ComparisonWithValueRef {
                    signed_data: self.signed_data(),
                    comparison_type: self.comparison_type(),
                    compare_value: MaskedValueRef::new_unchecked(value, mask),
                    file_id,
                    offset,
                },
                byte_size: size,
            }
        }
    }
}

impl<'data> Decodable<'data> for ComparisonWithValueRef<'data> {
    type Data = EncodedComparisonWithValue<'data>;
}

/// Compares data to a value.
#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[cfg(feature = "alloc")]
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct ComparisonWithValue {
    pub signed_data: bool,
    pub comparison_type: QueryComparisonType,
    pub compare_value: MaskedValue,
    pub file_id: FileId,
    pub offset: Varint,
}

#[cfg(feature = "alloc")]
impl ComparisonWithValue {
    pub fn as_ref(&self) -> ComparisonWithValueRef {
        ComparisonWithValueRef {
            signed_data: self.signed_data,
            comparison_type: self.comparison_type,
            compare_value: self.compare_value.as_ref(),
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
        fn test(op: ComparisonWithValueRef, data: &[u8]) {
            // Test op.encode_in() == data
            let mut encoded = [0_u8; 64];
            let size = op.encode_in(&mut encoded).unwrap();
            assert_eq!(size, data.len());
            assert_eq!(&encoded[..size], data);

            // Test decode(data) == op
            let WithByteSize {
                item: ret,
                byte_size: size,
            } = ComparisonWithValueRef::decode(data).unwrap();
            assert_eq!(size, data.len());
            assert_eq!(ret, op);

            // Test partial_decode == op
            let WithByteSize {
                item: decoder,
                byte_size: expected_size,
            } = ComparisonWithValueRef::start_decoding(data).unwrap();
            assert_eq!(ret.compare_value.mask().is_some(), decoder.mask_flag());
            assert_eq!(
                ret.compare_value.len(),
                decoder.compare_length().complete_decoding().item.u32() as usize
            );
            assert_eq!(ret.compare_value.mask(), decoder.mask());
            assert_eq!(ret.compare_value.value(), decoder.value().data());
            assert_eq!(expected_size, size);
            assert_eq!(unsafe { decoder.expected_size() }, size);
            assert_eq!(decoder.smaller_than(data.len()).unwrap(), size);
            assert_eq!(
                op,
                ComparisonWithValueRef {
                    signed_data: decoder.signed_data(),
                    comparison_type: decoder.comparison_type(),
                    compare_value: decoder.compare_value(),
                    file_id: decoder.file_id(),
                    offset: decoder.offset().complete_decoding().item,
                }
            );
        }
        test(
            ComparisonWithValueRef {
                signed_data: true,
                comparison_type: QueryComparisonType::Equal,
                compare_value: MaskedValueRef::new(
                    EncodableDataRef::new(&[0x00, 0x01, 0x02]).unwrap(),
                    None,
                )
                .unwrap(),
                file_id: FileId::new(0x42),
                offset: Varint::new(0x40_00).unwrap(),
            },
            &[
                0x40 | 0x08 | 0x01,
                0x03,
                0x00,
                0x01,
                0x02,
                0x42,
                0x80,
                0x40,
                0x00,
            ],
        );
        test(
            ComparisonWithValueRef {
                signed_data: false,
                comparison_type: QueryComparisonType::GreaterThan,
                compare_value: MaskedValueRef::new(
                    EncodableDataRef::new(&[0x0A, 0x0B, 0x0C, 0x0D]).unwrap(),
                    Some(&[0x00, 0xFF, 0x0F, 0xFF]),
                )
                .unwrap(),
                file_id: FileId::new(0x88),
                offset: Varint::new(0xFF).unwrap(),
            },
            &[
                0x40 | 0x10 | 0x04,
                0x04,
                0x00,
                0xFF,
                0x0F,
                0xFF,
                0x0A,
                0x0B,
                0x0C,
                0x0D,
                0x88,
                0x40,
                0xFF,
            ],
        );
    }

    #[test]
    fn consistence() {
        const TOT_SIZE: usize = 1 + 1 + 3 + 3 + 1 + 3;
        let op = ComparisonWithValueRef {
            signed_data: true,
            comparison_type: QueryComparisonType::GreaterThanOrEqual,
            compare_value: MaskedValueRef::new(
                EncodableDataRef::new(&[0x00, 0x43, 0x02]).unwrap(),
                Some(&[0x44, 0x88, 0x11]),
            )
            .unwrap(),
            file_id: FileId::new(0xFF),
            offset: Varint::new(0x3F_FF_00).unwrap(),
        };

        // Test decode(op.encode_in()) == op
        let mut encoded = [0_u8; TOT_SIZE];
        let size_encoded = op.encode_in(&mut encoded).unwrap();
        let WithByteSize {
            item: ret,
            byte_size: size_decoded,
        } = ComparisonWithValueRef::decode(&encoded).unwrap();
        assert_eq!(size_encoded, size_decoded);
        assert_eq!(ret, op);

        // Test decode(data).encode_in() == data
        let mut encoded2 = [0_u8; TOT_SIZE];
        let size_encoded2 = op.encode_in(&mut encoded2).unwrap();
        assert_eq!(size_encoded, size_encoded2);
        assert_eq!(encoded2[..size_encoded2], encoded[..size_encoded]);
    }
}

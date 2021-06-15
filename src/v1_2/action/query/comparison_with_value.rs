use super::define::{code, comparison_type::QueryComparisonType};
use crate::decodable::{Decodable, EncodedData, SizeError, WithByteSize};
use crate::define::{EncodableDataRef, FileId, MaskedValueRef};
use crate::encodable::Encodable;
use crate::v1_2::define::flag;
use crate::varint::{EncodedVarint, EncodedVarintMut, Varint};

#[cfg(feature = "alloc")]
use crate::define::MaskedValue;

/// Compares data to a value.
#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct ComparisonWithValueRef<'data> {
    pub signed_data: bool,
    pub comparison_type: QueryComparisonType,
    pub compare_value: MaskedValueRef<'data>,
    pub file_id: FileId,
    pub offset: Varint,
}

impl<'data> Encodable for ComparisonWithValueRef<'data> {
    unsafe fn encode_in_ptr(&self, out: *mut u8) -> usize {
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
        *out.offset(0) = (code::COMPARISON_WITH_VALUE << 5)
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

    fn encoded_size(&self) -> usize {
        unsafe {
            1 + Varint::new_unchecked(self.compare_value.len() as u32).encoded_size()
                + match &self.compare_value.mask() {
                    Some(mask) => mask.len(),
                    None => 0,
                }
                + self.compare_value.len()
                + 1
                + self.offset.encoded_size()
        }
    }
}

impl<'data> ComparisonWithValueRef<'data> {
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
    data: &'data [u8],
}

impl<'data> EncodedComparisonWithValue<'data> {
    pub fn mask_flag(&self) -> bool {
        unsafe { *self.data.get_unchecked(0) & flag::QUERY_MASK == flag::QUERY_MASK }
    }

    pub fn signed_data(&self) -> bool {
        unsafe { *self.data.get_unchecked(0) & flag::QUERY_SIGNED_DATA == flag::QUERY_SIGNED_DATA }
    }

    pub fn comparison_type(&self) -> QueryComparisonType {
        unsafe {
            QueryComparisonType::from_unchecked(
                *self.data.get_unchecked(0) & flag::QUERY_COMPARISON_TYPE,
            )
        }
    }

    pub fn compare_length(&self) -> EncodedVarint<'data> {
        unsafe { Varint::start_decoding_unchecked(self.data.get_unchecked(1..)) }
    }

    pub fn mask(&self) -> Option<&'data [u8]> {
        if self.mask_flag() {
            let WithByteSize {
                item: compare_length,
                byte_size: compare_length_size,
            } = self.compare_length().complete_decoding();
            let mask = unsafe {
                self.data
                    .get_unchecked(1 + compare_length_size..)
                    .get_unchecked(..compare_length.usize())
            };
            Some(mask)
        } else {
            None
        }
    }

    pub fn value(&self) -> &'data [u8] {
        let WithByteSize {
            item: compare_length,
            byte_size: compare_length_size,
        } = self.compare_length().complete_decoding();
        let mut offset = 1 + compare_length_size;
        if self.mask_flag() {
            offset += unsafe { compare_length.usize() };
        }
        unsafe {
            self.data
                .get_unchecked(offset..)
                .get_unchecked(..compare_length.usize())
        }
    }

    pub fn compare_value(&self) -> MaskedValueRef<'data> {
        let WithByteSize {
            item: compare_length,
            byte_size: compare_length_size,
        } = self.compare_length().complete_decoding();
        let compare_length = unsafe { compare_length.usize() };
        let mut offset = 1 + compare_length_size;
        unsafe {
            let mask = if self.mask_flag() {
                let mask = self
                    .data
                    .get_unchecked(offset..)
                    .get_unchecked(..compare_length);
                offset += compare_length;
                Some(mask)
            } else {
                None
            };
            let value = EncodableDataRef::new_unchecked(
                self.data
                    .get_unchecked(offset..)
                    .get_unchecked(..compare_length),
            );
            MaskedValueRef::new_unchecked(value, mask)
        }
    }

    pub fn file_id(&self) -> FileId {
        let WithByteSize {
            item: compare_length,
            byte_size: compare_length_size,
        } = self.compare_length().complete_decoding();
        let compare_length = unsafe { compare_length.usize() };
        let value_offset = if self.mask_flag() {
            1 + compare_length_size + 2 * compare_length
        } else {
            1 + compare_length_size + compare_length
        };
        unsafe { FileId(*self.data.get_unchecked(value_offset)) }
    }

    pub fn offset(&self) -> EncodedVarint<'data> {
        let WithByteSize {
            item: compare_length,
            byte_size: compare_length_size,
        } = self.compare_length().complete_decoding();
        let compare_length = unsafe { compare_length.usize() };
        let value_offset = if self.mask_flag() {
            1 + compare_length_size + 2 * compare_length
        } else {
            1 + compare_length_size + compare_length
        };
        unsafe { Varint::start_decoding_unchecked(self.data.get_unchecked(value_offset + 1..)) }
    }

    /// # Safety
    /// You have to warrant that somehow that there is enough byte to decode the encoded size.
    /// If you fail to do so, out of bound bytes will be read, and an absurd value will be
    /// returned.
    pub unsafe fn encoded_size_unchecked(&self) -> usize {
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
        let decodable_offset =
            Varint::start_decoding_unchecked(self.data.get_unchecked(value_offset + 1..));
        value_offset + 1 + decodable_offset.encoded_size_unchecked()
    }
}

impl<'data> EncodedData<'data> for EncodedComparisonWithValue<'data> {
    type SourceData = &'data [u8];
    type DecodedData = ComparisonWithValueRef<'data>;

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
            size += compare_length.encoded_size_unchecked();
            if data_size < size {
                return Err(SizeError::MissingBytes);
            }
            let compare_length = compare_length.complete_decoding().item.usize();
            if self.mask_flag() {
                size += compare_length;
            }
            size += compare_length;
            size += 1;
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
            let compare_length = compare_length.u32() as usize;
            let mut size = 1 + compare_length_size;
            let mask = if self.mask_flag() {
                let mask =
                    core::slice::from_raw_parts(self.data.get_unchecked(size), compare_length);
                size += compare_length;
                Some(mask)
            } else {
                None
            };
            let value = EncodableDataRef::new_unchecked(core::slice::from_raw_parts(
                self.data.get_unchecked(size),
                compare_length,
            ));
            size += compare_length;
            let file_id = FileId(*self.data.get_unchecked(size));
            size += 1;
            let WithByteSize {
                item: offset,
                byte_size: offset_size,
            } = Varint::decode_unchecked(self.data.get_unchecked(size..));
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

pub struct EncodedComparisonWithValueMut<'data> {
    data: &'data mut [u8],
}

crate::make_downcastable!(EncodedComparisonWithValueMut, EncodedComparisonWithValue);

impl<'data> EncodedComparisonWithValueMut<'data> {
    pub fn mask_flag(&self) -> bool {
        self.as_ref().mask_flag()
    }

    pub fn signed_data(&self) -> bool {
        self.as_ref().signed_data()
    }

    pub fn comparison_type(&self) -> QueryComparisonType {
        self.as_ref().comparison_type()
    }

    pub fn compare_length(&self) -> EncodedVarint<'data> {
        self.as_ref().compare_length()
    }

    pub fn mask(&self) -> Option<&'data [u8]> {
        self.as_ref().mask()
    }

    pub fn value(&self) -> &'data [u8] {
        self.as_ref().value()
    }

    pub fn compare_value(&self) -> MaskedValueRef<'data> {
        self.as_ref().compare_value()
    }

    pub fn file_id(&self) -> FileId {
        self.as_ref().file_id()
    }

    pub fn offset(&self) -> EncodedVarint<'data> {
        self.as_ref().offset()
    }

    /// # Safety
    /// You have to warrant that somehow that there is enough byte to decode the encoded size.
    /// If you fail to do so, out of bound bytes will be read, and an absurd value will be
    /// returned.
    pub unsafe fn encoded_size_unchecked(&self) -> usize {
        self.as_ref().encoded_size_unchecked()
    }

    /// # Safety
    /// This changes the structure of the packet and you have to make sure that the rest of the
    /// payload is coherent.
    pub unsafe fn set_mask_flag(&mut self, mask_flag: bool) {
        if mask_flag {
            *self.data.get_unchecked_mut(0) |= flag::QUERY_MASK
        } else {
            *self.data.get_unchecked_mut(0) &= !flag::QUERY_MASK
        }
    }

    pub fn set_signed_data(&mut self, signed_data: bool) {
        unsafe {
            if signed_data {
                *self.data.get_unchecked_mut(0) |= flag::QUERY_SIGNED_DATA
            } else {
                *self.data.get_unchecked_mut(0) &= !flag::QUERY_SIGNED_DATA
            }
        }
    }

    pub fn set_comparison_type(&mut self, ty: QueryComparisonType) {
        unsafe {
            *self.data.get_unchecked_mut(0) &= !flag::QUERY_COMPARISON_TYPE;
            *self.data.get_unchecked_mut(0) |= ty as u8;
        }
    }

    /// # Safety
    /// This value should never be modified in place because it details the length in bytes of the
    /// value and mask items.
    /// If you are to modify this, make sure you know what you are doing.
    pub unsafe fn compare_length_mut(&'data mut self) -> EncodedVarintMut<'data> {
        Varint::start_decoding_unchecked_mut(self.data.get_unchecked_mut(1..))
    }

    pub fn mask_mut(&mut self) -> Option<&mut [u8]> {
        if self.mask_flag() {
            let WithByteSize {
                item: compare_length,
                byte_size: compare_length_size,
            } = self.compare_length().complete_decoding();
            let mask = unsafe {
                self.data
                    .get_unchecked_mut(1 + compare_length_size..)
                    .get_unchecked_mut(..compare_length.usize())
            };
            Some(mask)
        } else {
            None
        }
    }

    pub fn value_mut(&mut self) -> &mut [u8] {
        let WithByteSize {
            item: compare_length,
            byte_size: compare_length_size,
        } = self.compare_length().complete_decoding();
        let mut offset = 1 + compare_length_size;
        if self.mask_flag() {
            offset += unsafe { compare_length.usize() };
        }
        unsafe {
            self.data
                .get_unchecked_mut(offset..)
                .get_unchecked_mut(..compare_length.usize())
        }
    }

    pub fn set_file_id(&mut self, file_id: FileId) {
        let WithByteSize {
            item: compare_length,
            byte_size: compare_length_size,
        } = self.compare_length().complete_decoding();
        let compare_length = unsafe { compare_length.usize() };
        let value_offset = if self.mask_flag() {
            1 + compare_length_size + 2 * compare_length
        } else {
            1 + compare_length_size + compare_length
        };
        unsafe { *self.data.get_unchecked_mut(value_offset) = file_id.u8() }
    }

    pub fn offset_mut(&mut self) -> EncodedVarintMut {
        let WithByteSize {
            item: compare_length,
            byte_size: compare_length_size,
        } = self.compare_length().complete_decoding();
        let compare_length = unsafe { compare_length.usize() };
        let value_offset = if self.mask_flag() {
            1 + compare_length_size + 2 * compare_length
        } else {
            1 + compare_length_size + compare_length
        };
        unsafe {
            Varint::start_decoding_unchecked_mut(self.data.get_unchecked_mut(value_offset + 1..))
        }
    }
}

impl<'data> EncodedData<'data> for EncodedComparisonWithValueMut<'data> {
    type SourceData = &'data mut [u8];
    type DecodedData = ComparisonWithValueRef<'data>;

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

impl<'data> Decodable<'data> for ComparisonWithValueRef<'data> {
    type Data = EncodedComparisonWithValue<'data>;
    type DataMut = EncodedComparisonWithValueMut<'data>;
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
            assert_eq!(ret.compare_value.value(), decoder.value());
            assert_eq!(expected_size, size);
            assert_eq!(unsafe { decoder.encoded_size_unchecked() }, size);
            assert_eq!(decoder.encoded_size().unwrap(), size);
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

            // Test partial mutability
            let WithByteSize {
                item: mut decoder_mut,
                byte_size: expected_size,
            } = ComparisonWithValueRef::start_decoding_mut(&mut encoded).unwrap();
            assert_eq!(expected_size, size);

            assert_eq!(decoder_mut.signed_data(), op.signed_data);
            let new_signed_data = !op.signed_data;
            assert!(new_signed_data != op.signed_data);
            decoder_mut.set_signed_data(new_signed_data);
            assert_eq!(decoder_mut.signed_data(), new_signed_data);

            assert_eq!(decoder_mut.comparison_type(), op.comparison_type);
            let new_comparison_type =
                QueryComparisonType::from((op.comparison_type as u8 + 1) % 8).unwrap();
            assert!(new_comparison_type != op.comparison_type);
            decoder_mut.set_comparison_type(new_comparison_type);
            assert_eq!(decoder_mut.comparison_type(), new_comparison_type);

            if decoder_mut.mask_mut().is_some() {
                let original = op.compare_value.mask().unwrap();
                let mut new_data = vec![0_u8; original.len()];
                {
                    let data_mut = decoder_mut.mask_mut().unwrap();
                    for (i, b) in original.iter().enumerate() {
                        new_data[i] = !b;
                        data_mut[i] = new_data[i];
                    }
                    assert!(&new_data[..] != original);
                }
                assert_eq!(decoder_mut.mask().unwrap(), &new_data);
            }

            {
                let original = op.compare_value.value();
                let mut new_data = vec![0_u8; original.len()];
                let data_mut = decoder_mut.value_mut();
                for (i, b) in original.iter().enumerate() {
                    new_data[i] = !b;
                    data_mut[i] = new_data[i];
                }
                assert!(&new_data[..] != original);
                assert_eq!(decoder_mut.value(), &new_data);
            }

            assert_eq!(decoder_mut.file_id(), op.file_id);
            let new_file_id = FileId(!op.file_id.u8());
            assert!(new_file_id != op.file_id);
            decoder_mut.set_file_id(new_file_id);
            assert_eq!(decoder_mut.file_id(), new_file_id);

            {
                let original = op.offset;
                let mut decoder_mut = decoder_mut.offset_mut();
                assert_eq!(decoder_mut.complete_decoding().item.u32(), original.u32());
                let new_value = Varint::new(if original.encoded_size() == 1 {
                    (original.u32() == 0) as u32
                } else {
                    original.u32() ^ 0x3F
                })
                .unwrap();
                assert!(new_value != original);
                decoder_mut.set_value(&new_value).unwrap();
                assert_eq!(decoder_mut.complete_decoding().item, new_value);
            }
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

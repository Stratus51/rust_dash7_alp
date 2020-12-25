use super::super::super::define::flag;
use super::super::super::error::QueryOperandDecodeError;
use super::define::{QueryCode, QueryComparisonType};
use crate::define::{EncodableData, FileId, MaskedValue};
use crate::varint::{DecodableVarint, Varint};

/// Writes data to a file.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct ComparisonWithValue<'item> {
    pub signed_data: bool,
    pub comparison_type: QueryComparisonType,
    pub compare_value: MaskedValue<'item>,
    pub file_id: FileId,
    pub offset: Varint,
}

impl<'item> ComparisonWithValue<'item> {
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
    /// - The decoded data is bigger than the expected size of the `decodable` object.
    /// Meaning that given the resulting decodable object `decodable`:
    /// `data.len()` >= [`decodable.size()`](struct.DecodableComparisonWithValue.html#method.size).
    ///
    /// Failing that might result in reading and interpreting data outside the given
    /// array (depending on what is done with the resulting object).
    pub const unsafe fn start_decoding_ptr<'data>(
        data: *const u8,
    ) -> DecodableComparisonWithValue<'data> {
        DecodableComparisonWithValue::from_ptr(data)
    }

    /// Creates a decodable item without checking the data size.
    ///
    /// # Safety
    /// You are to check that:
    /// - The first byte contains this action's querycode.
    /// - The decoded data is bigger than the expected size of the `decodable` object.
    /// Meaning that given the resulting decodable object `decodable`:
    /// `data.len()` >= [`decodable.size()`](struct.DecodableComparisonWithValue.html#method.size).
    ///
    /// Failing that might result in reading and interpreting data outside the given
    /// array (depending on what is done with the resulting object).
    pub const unsafe fn start_decoding_unchecked(data: &[u8]) -> DecodableComparisonWithValue {
        DecodableComparisonWithValue::new(data)
    }

    /// Creates a decodable item.
    ///
    /// This decodable item allows each parts of the item independently.
    ///
    /// # Errors
    /// - Fails if first byte of the data contains the wrong querycode.
    /// - Fails if data is empty.
    /// - Fails if data is smaller then the decoded expected size.
    pub fn start_decoding(
        data: &[u8],
    ) -> Result<DecodableComparisonWithValue, QueryOperandDecodeError> {
        match data.get(0) {
            None => return Err(QueryOperandDecodeError::MissingBytes(1)),
            Some(byte) => {
                let code = *byte >> 5;
                if code != QueryCode::ComparisonWithValue as u8 {
                    return Err(QueryOperandDecodeError::BadQueryCode(code));
                }
            }
        }
        let ret = unsafe { Self::start_decoding_unchecked(data) };
        let ret_size = ret.size();
        if data.len() < ret_size {
            return Err(QueryOperandDecodeError::MissingBytes(ret_size));
        }
        Ok(ret)
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
    /// - The data is not empty.
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
    /// - The data is not empty.
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
            Ok(v) => Ok(v.complete_decoding()),
            Err(e) => Err(e),
        }
    }
}

pub struct DecodableComparisonWithValue<'data> {
    data: *const u8,
    data_life: core::marker::PhantomData<&'data ()>,
}

impl<'data> DecodableComparisonWithValue<'data> {
    const fn new(data: &'data [u8]) -> Self {
        Self {
            data: data.as_ptr(),
            data_life: core::marker::PhantomData,
        }
    }

    const fn from_ptr(data: *const u8) -> Self {
        Self {
            data,
            data_life: core::marker::PhantomData,
        }
    }

    /// Decodes the size of the Item in bytes
    pub fn size(&self) -> usize {
        let (compare_length, compare_length_size) = self.compare_length().complete_decoding();
        let compare_length = compare_length.u32() as usize;
        let value_offset = if self.mask_flag() {
            1 + compare_length_size + 2 * compare_length
        } else {
            1 + compare_length_size + compare_length
        };
        let decodable_offset =
            unsafe { Varint::start_decoding_ptr(self.data.add(value_offset + 1)) };
        value_offset + 1 + decodable_offset.size()
    }

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

    pub fn compare_length(&self) -> DecodableVarint {
        unsafe { Varint::start_decoding_ptr(self.data.add(1)) }
    }

    pub fn mask(&self) -> Option<&'data [u8]> {
        if self.mask_flag() {
            let (compare_length, compare_length_size) = self.compare_length().complete_decoding();
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

    pub fn value(&self) -> EncodableData<'data> {
        let (compare_length, compare_length_size) = self.compare_length().complete_decoding();
        let mut offset = 1 + compare_length_size;
        if self.mask_flag() {
            offset += compare_length.u32() as usize;
        }
        unsafe {
            EncodableData::new_unchecked(core::slice::from_raw_parts(
                self.data.add(offset),
                compare_length.u32() as usize,
            ))
        }
    }

    pub fn compare_value(&self) -> MaskedValue<'data> {
        let (compare_length, compare_length_size) = self.compare_length().complete_decoding();
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
            let value = EncodableData::new_unchecked(core::slice::from_raw_parts(
                self.data.add(offset),
                compare_length,
            ));
            MaskedValue::new_unchecked(value, mask)
        }
    }

    pub fn file_id(&self) -> FileId {
        let (compare_length, compare_length_size) = self.compare_length().complete_decoding();
        let compare_length = compare_length.u32() as usize;
        let value_offset = if self.mask_flag() {
            1 + compare_length_size + 2 * compare_length
        } else {
            1 + compare_length_size + compare_length
        };
        unsafe { FileId(*self.data.add(value_offset)) }
    }

    pub fn offset(&self) -> DecodableVarint {
        let (compare_length, compare_length_size) = self.compare_length().complete_decoding();
        let compare_length = compare_length.u32() as usize;
        let value_offset = if self.mask_flag() {
            1 + compare_length_size + 2 * compare_length
        } else {
            1 + compare_length_size + compare_length
        };
        unsafe { Varint::start_decoding_ptr(self.data.add(value_offset + 1)) }
    }

    /// Fully decode the Item
    ///
    /// Returns the decoded data and the number of bytes consumed to produce it.
    pub fn complete_decoding(&self) -> (ComparisonWithValue<'data>, usize) {
        unsafe {
            let (compare_length, compare_length_size) = self.compare_length().complete_decoding();
            let compare_length = compare_length.u32() as usize;
            let mut size = 1 + compare_length_size;
            let mask = if self.mask_flag() {
                let mask = core::slice::from_raw_parts(self.data.add(size), compare_length);
                size += compare_length;
                Some(mask)
            } else {
                None
            };
            let value = EncodableData::new_unchecked(core::slice::from_raw_parts(
                self.data.add(size),
                compare_length,
            ));
            size += compare_length;
            let file_id = FileId(*self.data.add(size));
            size += 1;
            let (offset, offset_size) = Varint::decode_ptr(self.data.add(size));
            size += offset_size;

            (
                ComparisonWithValue {
                    signed_data: self.signed_data(),
                    comparison_type: self.comparison_type(),
                    compare_value: MaskedValue::new_unchecked(value, mask),
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
    use super::*;

    #[test]
    fn known() {
        fn test(op: ComparisonWithValue, data: &[u8]) {
            // Test op.encode_in() == data
            let mut encoded = [0_u8; 64];
            let size = op.encode_in(&mut encoded).unwrap();
            assert_eq!(size, data.len());
            assert_eq!(&encoded[..size], data);

            // Test decode(data) == op
            let (ret, size) = ComparisonWithValue::decode(data).unwrap();
            assert_eq!(size, data.len());
            assert_eq!(ret, op);

            // Test partial_decode == op
            let decoder = ComparisonWithValue::start_decoding(&data).unwrap();
            assert_eq!(ret.compare_value.mask().is_some(), decoder.mask_flag());
            assert_eq!(
                ret.compare_value.len(),
                decoder.compare_length().complete_decoding().0.u32() as usize
            );
            assert_eq!(ret.compare_value.mask(), decoder.mask());
            assert_eq!(ret.compare_value.value(), decoder.value().get());
            assert_eq!(size, decoder.size());
            assert_eq!(
                op,
                ComparisonWithValue {
                    signed_data: decoder.signed_data(),
                    comparison_type: decoder.comparison_type(),
                    compare_value: decoder.compare_value(),
                    file_id: decoder.file_id(),
                    offset: decoder.offset().complete_decoding().0,
                }
            );
        }
        test(
            ComparisonWithValue {
                signed_data: true,
                comparison_type: QueryComparisonType::Equal,
                compare_value: MaskedValue::new(
                    EncodableData::new(&[0x00, 0x01, 0x02]).unwrap(),
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
            ComparisonWithValue {
                signed_data: false,
                comparison_type: QueryComparisonType::GreaterThan,
                compare_value: MaskedValue::new(
                    EncodableData::new(&[0x0A, 0x0B, 0x0C, 0x0D]).unwrap(),
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
        let op = ComparisonWithValue {
            signed_data: true,
            comparison_type: QueryComparisonType::GreaterThanOrEqual,
            compare_value: MaskedValue::new(
                EncodableData::new(&[0x00, 0x43, 0x02]).unwrap(),
                Some(&[0x44, 0x88, 0x11]),
            )
            .unwrap(),
            file_id: FileId::new(0xFF),
            offset: Varint::new(0x3F_FF_00).unwrap(),
        };

        // Test decode(op.encode_in()) == op
        let mut encoded = [0_u8; TOT_SIZE];
        let size_encoded = op.encode_in(&mut encoded).unwrap();
        let (ret, size_decoded) = ComparisonWithValue::decode(&encoded).unwrap();
        assert_eq!(size_encoded, size_decoded);
        assert_eq!(ret, op);

        // Test decode(data).encode_in() == data
        let mut encoded2 = [0_u8; TOT_SIZE];
        let size_encoded2 = op.encode_in(&mut encoded2).unwrap();
        assert_eq!(size_encoded, size_encoded2);
        assert_eq!(encoded2[..size_encoded2], encoded[..size_encoded]);
    }
}

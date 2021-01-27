use super::super::define::flag;
use super::super::define::op_code::OpCode;
use super::super::error::BasicDecodeError;
#[cfg(feature = "alloc")]
use crate::define::EncodableData;
use crate::define::{EncodableDataRef, FileId};
use crate::varint::{DecodableVarint, Varint};

// TODO Is it the role of this library to teach the semantics of the protocol or should it just
// focus on documenting its usage, based on the assumption that that semantic is already known?
/// Writes data to a file.
#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct WriteFileDataRef<'item> {
    /// Group with next action
    pub group: bool,
    /// Ask for a response (a status)
    pub response: bool,
    /// File ID of the file to read
    pub file_id: FileId,
    /// Offset at which to start the reading
    pub offset: Varint,
    /// Data to write
    pub data: EncodableDataRef<'item>,
}

impl<'item> WriteFileDataRef<'item> {
    /// Most common builder `WriteFileData` builder.
    ///
    /// group = false
    /// response = true
    pub fn new(file_id: FileId, offset: Varint, data: EncodableDataRef<'item>) -> Self {
        Self {
            group: false,
            response: true,
            file_id,
            offset,
            data,
        }
    }

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
        *out.add(0) = OpCode::WriteFileData as u8
            | if self.group { flag::GROUP } else { 0 }
            | if self.response { flag::RESPONSE } else { 0 };
        *out.add(1) = self.file_id.u8();
        size += 2;
        size += self.offset.encode_in_ptr(out.add(size));
        let length = Varint::new_unchecked(self.data.len() as u32);
        size += length.encode_in_ptr(out.add(size));
        out.add(size)
            .copy_from(self.data.data().as_ptr(), length.u32() as usize);
        size += length.u32() as usize;
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
        let length = unsafe { Varint::new_unchecked(self.data.len() as u32) };
        1 + 1 + self.offset.size() + length.size()
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
    /// - The decodable object fits in the given data:
    /// [`decodable.smaller_than(data.len())`](struct.DecodableWriteFileData.html#method.smaller_than)
    ///
    /// Failing that might result in reading and interpreting data outside the given
    /// array (depending on what is done with the resulting object).
    pub const unsafe fn start_decoding_ptr<'data>(
        data: *const u8,
    ) -> DecodableWriteFileData<'data> {
        DecodableWriteFileData::from_ptr(data)
    }

    /// Creates a decodable item without checking the data size.
    ///
    /// # Safety
    /// You are to check that:
    /// - The decodable object fits in the given data:
    /// [`decodable.smaller_than(data.len())`](struct.DecodableWriteFileData.html#method.smaller_than)
    ///
    /// Failing that might result in reading and interpreting data outside the given
    /// array (depending on what is done with the resulting object).
    pub const unsafe fn start_decoding_unchecked(data: &[u8]) -> DecodableWriteFileData {
        DecodableWriteFileData::new(data)
    }

    /// Returns a Decodable object and its expected byte size.
    ///
    /// This decodable item allows each parts of the item to be decoded independently.
    ///
    /// # Errors
    /// - Fails if data is smaller then the decoded expected size.
    pub fn start_decoding(
        data: &[u8],
    ) -> Result<(DecodableWriteFileData, usize), BasicDecodeError> {
        if data.is_empty() {
            return Err(BasicDecodeError::MissingBytes(1));
        }
        let ret = unsafe { Self::start_decoding_unchecked(data) };
        let size = ret
            .smaller_than(data.len())
            .map_err(BasicDecodeError::MissingBytes)?;
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
    /// May attempt to read bytes after the end of the array.
    ///
    /// You are to check that:
    /// - The resulting size of the data consumed is smaller than the size of the
    /// decoded data.
    ///
    /// Failing that will result in reading and interpreting data outside the given
    /// array.
    pub unsafe fn decode_ptr(data: *const u8) -> (Self, usize) {
        Self::start_decoding_ptr(data).complete_decoding()
    }

    /// Decodes the Item from bytes.
    ///
    /// Returns the decoded data and the number of bytes consumed to produce it.
    ///
    /// # Safety
    /// May attempt to read bytes after the end of the array.
    ///
    /// You are to check that:
    /// - The resulting size of the data consumed is smaller than the size of the
    /// decoded data.
    ///
    /// Failing that will result in reading and interpreting data outside the given
    /// array.
    pub unsafe fn decode_unchecked(data: &'item [u8]) -> (Self, usize) {
        Self::start_decoding_unchecked(data).complete_decoding()
    }

    /// Decodes the item from bytes.
    ///
    /// On success, returns the decoded data and the number of bytes consumed
    /// to produce it.
    ///
    /// # Errors
    /// - Fails if data is smaller then the decoded expected size.
    pub fn decode(data: &'item [u8]) -> Result<(Self, usize), BasicDecodeError> {
        match Self::start_decoding(data) {
            Ok(v) => Ok(v.0.complete_decoding()),
            Err(e) => Err(e),
        }
    }

    #[cfg(feature = "alloc")]
    pub fn to_owned(&self) -> WriteFileData {
        WriteFileData {
            group: self.group,
            response: self.response,
            file_id: self.file_id,
            offset: self.offset,
            data: self.data.to_owned(),
        }
    }
}

pub struct DecodableWriteFileData<'data> {
    data: *const u8,
    data_life: core::marker::PhantomData<&'data ()>,
}

impl<'data> DecodableWriteFileData<'data> {
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
        let offset_size = self.offset().expected_size();
        let (length, length_size) = Varint::decode_ptr(self.data.add(2 + offset_size));
        2 + offset_size + length_size + length.usize()
    }

    /// Checks whether the given data_size is bigger than the decoded object expected size.
    ///
    /// On success, returns the size of the decoded object.
    ///
    /// # Errors
    /// Fails if the data_size is smaller than the required data size to decode the object.
    pub fn smaller_than(&self, data_size: usize) -> Result<usize, usize> {
        unsafe {
            let mut size = 3;
            if data_size < size {
                return Err(size);
            }
            size += self.offset().expected_size();
            if data_size < size {
                return Err(size);
            }
            let (length, length_size) = Varint::decode_ptr(self.data.add(size - 1));
            size += length.usize() + length_size - 1;
            if data_size < size {
                return Err(size);
            }
            Ok(size)
        }
    }

    pub fn group(&self) -> bool {
        unsafe { *self.data.add(0) & flag::GROUP != 0 }
    }

    pub fn response(&self) -> bool {
        unsafe { *self.data.add(0) & flag::RESPONSE != 0 }
    }

    pub fn file_id(&self) -> FileId {
        unsafe { FileId(*self.data.add(1)) }
    }

    pub fn offset(&self) -> DecodableVarint {
        unsafe { Varint::start_decoding_ptr(self.data.add(2)) }
    }

    pub fn length(&self) -> DecodableVarint {
        unsafe {
            let offset_size = (((*self.data.add(2) & 0xC0) >> 6) + 1) as usize;
            Varint::start_decoding_ptr(self.data.add(2 + offset_size))
        }
    }

    pub fn data(&self) -> (EncodableDataRef<'data>, usize) {
        unsafe {
            let offset_size = (((*self.data.add(2) & 0xC0) >> 6) + 1) as usize;
            let (length, length_size) = Varint::decode_ptr(self.data.add(2 + offset_size));
            let data_offset = 2 + offset_size + length_size;
            let data =
                core::slice::from_raw_parts(self.data.add(data_offset), length.u32() as usize);
            (
                EncodableDataRef::new_unchecked(data),
                length_size + length.u32() as usize,
            )
        }
    }

    /// Fully decode the Item
    ///
    /// Returns the decoded data and the number of bytes consumed to produce it.
    pub fn complete_decoding(&self) -> (WriteFileDataRef<'data>, usize) {
        let (offset, offset_size) = self.offset().complete_decoding();
        let (data, length_size, length) = unsafe {
            let (length, length_size) = Varint::decode_ptr(self.data.add(2 + offset_size));
            let data_offset = 2 + offset_size + length_size;
            let data =
                core::slice::from_raw_parts(self.data.add(data_offset), length.u32() as usize);
            (
                EncodableDataRef::new_unchecked(data),
                length_size,
                length.u32(),
            )
        };
        (
            WriteFileDataRef {
                group: self.group(),
                response: self.response(),
                file_id: self.file_id(),
                offset,
                data,
            },
            2 + offset_size + length_size + length as usize,
        )
    }
}

/// Writes data to a file.
#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[cfg(feature = "alloc")]
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct WriteFileData {
    /// Group with next action
    pub group: bool,
    /// Ask for a response (a status)
    pub response: bool,
    /// File ID of the file to read
    pub file_id: FileId,
    /// Offset at which to start the reading
    pub offset: Varint,
    /// Data to write
    pub data: EncodableData,
}

#[cfg(feature = "alloc")]
impl WriteFileData {
    pub fn as_ref(&self) -> WriteFileDataRef {
        WriteFileDataRef {
            group: self.group,
            response: self.response,
            file_id: self.file_id,
            offset: self.offset,
            data: self.data.as_ref(),
        }
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::unwrap_in_result, clippy::panic, clippy::expect_used)]
    use super::*;

    #[test]
    fn known() {
        fn test(op: WriteFileDataRef, data: &[u8]) {
            // Test op.encode_in() == data
            let mut encoded = [0_u8; 2 + 8];
            let size = op.encode_in(&mut encoded).unwrap();
            assert_eq!(size, data.len());
            assert_eq!(&encoded[..size], data);

            // Test decode(data) == op
            let (ret, size) = WriteFileDataRef::decode(data).unwrap();
            assert_eq!(size, data.len());
            assert_eq!(ret, op);

            // Test partial_decode == op
            let (decoder, expected_size) = WriteFileDataRef::start_decoding(data).unwrap();
            assert_eq!(expected_size, size);
            assert_eq!(unsafe { decoder.expected_size() }, size);
            assert_eq!(decoder.smaller_than(data.len()).unwrap(), size);
            assert_eq!(
                op.data.len(),
                decoder.length().complete_decoding().0.u32() as usize
            );
            assert_eq!(
                op,
                WriteFileDataRef {
                    group: decoder.group(),
                    response: decoder.response(),
                    file_id: decoder.file_id(),
                    offset: decoder.offset().complete_decoding().0,
                    data: decoder.data().0,
                }
            );
        }
        test(
            WriteFileDataRef {
                group: false,
                response: true,
                file_id: FileId::new(0),
                offset: Varint::new(0).unwrap(),
                data: EncodableDataRef::new(&[0, 1, 2]).unwrap(),
            },
            &[0x44, 0x00, 0x00, 0x03, 0x00, 0x01, 0x02],
        );
        test(
            WriteFileDataRef {
                group: true,
                response: false,
                file_id: FileId::new(1),
                offset: Varint::new(0x3F_FF).unwrap(),
                data: EncodableDataRef::new(&[]).unwrap(),
            },
            &[0x84, 0x01, 0x7F, 0xFF, 0x00],
        );
        test(
            WriteFileDataRef {
                group: true,
                response: true,
                file_id: FileId::new(0x80),
                offset: Varint::new(0).unwrap(),
                data: EncodableDataRef::new(&[0x44]).unwrap(),
            },
            &[0xC4, 0x80, 0x00, 0x01, 0x44],
        );
        test(
            WriteFileDataRef {
                group: false,
                response: false,
                file_id: FileId::new(0xFF),
                offset: Varint::new(0x3F_FF_FF_FF).unwrap(),
                data: EncodableDataRef::new(&[0xFF, 0xFE]).unwrap(),
            },
            &[0x04, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x02, 0xFF, 0xFE],
        );
    }

    #[test]
    fn consistence() {
        const TOT_SIZE: usize = 2 + 2 + 3;
        let op = WriteFileDataRef {
            group: true,
            response: false,
            file_id: FileId::new(0x80),
            offset: Varint::new(89).unwrap(),
            data: EncodableDataRef::new(&[0xFF, 0xFE]).unwrap(),
        };

        // Test decode(op.encode_in()) == op
        let mut encoded = [0_u8; TOT_SIZE];
        let size_encoded = op.encode_in(&mut encoded).unwrap();
        let (ret, size_decoded) = WriteFileDataRef::decode(&encoded).unwrap();
        assert_eq!(size_encoded, size_decoded);
        assert_eq!(ret, op);

        // Test decode(data).encode_in() == data
        let mut encoded2 = [0_u8; TOT_SIZE];
        let size_encoded2 = op.encode_in(&mut encoded2).unwrap();
        assert_eq!(size_encoded, size_encoded2);
        assert_eq!(encoded2[..size_encoded2], encoded[..size_encoded]);
    }
}

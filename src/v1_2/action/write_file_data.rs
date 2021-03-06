use super::super::define::flag;
use super::super::define::op_code::OpCode;
use crate::decodable::{Decodable, EncodedData, WithByteSize};
#[cfg(feature = "alloc")]
use crate::define::EncodableData;
use crate::define::{EncodableDataRef, FileId};
use crate::varint::{EncodedVarint, Varint};

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

pub struct EncodedWriteFileData<'data> {
    data: &'data [u8],
}

impl<'data> EncodedWriteFileData<'data> {
    pub fn group(&self) -> bool {
        unsafe { *self.data.get_unchecked(0) & flag::GROUP != 0 }
    }

    pub fn response(&self) -> bool {
        unsafe { *self.data.get_unchecked(0) & flag::RESPONSE != 0 }
    }

    pub fn file_id(&self) -> FileId {
        unsafe { FileId(*self.data.get_unchecked(1)) }
    }

    pub fn offset(&self) -> EncodedVarint {
        unsafe { Varint::start_decoding_unchecked(self.data.get_unchecked(2..)) }
    }

    pub fn length(&self) -> EncodedVarint {
        unsafe {
            let offset_size = (((*self.data.get_unchecked(2) & 0xC0) >> 6) + 1) as usize;
            Varint::start_decoding_unchecked(self.data.get_unchecked(2 + offset_size..))
        }
    }

    pub fn data(&self) -> WithByteSize<EncodableDataRef<'data>> {
        unsafe {
            let offset_size = (((*self.data.get_unchecked(2) & 0xC0) >> 6) + 1) as usize;
            let WithByteSize {
                item: length,
                byte_size: length_size,
            } = Varint::decode_unchecked(self.data.get_unchecked(2 + offset_size..));
            let data_offset = 2 + offset_size + length_size;
            let data = core::slice::from_raw_parts(
                self.data.get_unchecked(data_offset),
                length.u32() as usize,
            );
            WithByteSize {
                item: EncodableDataRef::new_unchecked(data),
                byte_size: length_size + length.u32() as usize,
            }
        }
    }

    /// # Safety
    /// You are to warrant, somehow, that the input byte array contains a complete item.
    /// Else this might result in out of bound reads, and absurd results.
    pub unsafe fn size_unchecked(&self) -> usize {
        let offset_size = self.offset().size_unchecked();
        let WithByteSize {
            item: length,
            byte_size: length_size,
        } = Varint::decode_unchecked(self.data.get_unchecked(2 + offset_size..));
        2 + offset_size + length_size + length.usize()
    }
}

impl<'data> EncodedData<'data> for EncodedWriteFileData<'data> {
    type DecodedData = WriteFileDataRef<'data>;
    unsafe fn new(data: &'data [u8]) -> Self {
        Self { data }
    }

    fn size(&self) -> Result<usize, ()> {
        unsafe {
            let mut size = 3;
            let data_size = self.data.len();
            if data_size < size {
                return Err(());
            }
            size += self.offset().size_unchecked();
            if data_size < size {
                return Err(());
            }
            let WithByteSize {
                item: length,
                byte_size: length_size,
            } = Varint::decode_unchecked(self.data.get_unchecked(size - 1..));
            size += length.usize() + length_size - 1;
            if data_size < size {
                return Err(());
            }
            Ok(size)
        }
    }

    fn complete_decoding(&self) -> WithByteSize<WriteFileDataRef<'data>> {
        let WithByteSize {
            item: offset,
            byte_size: offset_size,
        } = self.offset().complete_decoding();
        let (data, length_size, length) = unsafe {
            let WithByteSize {
                item: length,
                byte_size: length_size,
            } = Varint::decode_unchecked(self.data.get_unchecked(2 + offset_size..));
            let data_offset = 2 + offset_size + length_size;
            let data = core::slice::from_raw_parts(
                self.data.get_unchecked(data_offset),
                length.u32() as usize,
            );
            (
                EncodableDataRef::new_unchecked(data),
                length_size,
                length.u32(),
            )
        };
        WithByteSize {
            item: WriteFileDataRef {
                group: self.group(),
                response: self.response(),
                file_id: self.file_id(),
                offset,
                data,
            },
            byte_size: 2 + offset_size + length_size + length as usize,
        }
    }
}

impl<'data> Decodable<'data> for WriteFileDataRef<'data> {
    type Data = EncodedWriteFileData<'data>;
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
    use crate::decodable::{Decodable, EncodedData};

    #[test]
    fn known() {
        fn test(op: WriteFileDataRef, data: &[u8]) {
            // Test op.encode_in() == data
            let mut encoded = [0_u8; 2 + 8];
            let size = op.encode_in(&mut encoded).unwrap();
            assert_eq!(size, data.len());
            assert_eq!(&encoded[..size], data);

            // Test decode(data) == op
            let WithByteSize {
                item: ret,
                byte_size: size,
            } = WriteFileDataRef::decode(data).unwrap();
            assert_eq!(size, data.len());
            assert_eq!(ret, op);

            // Test partial_decode == op
            let WithByteSize {
                item: decoder,
                byte_size: expected_size,
            } = WriteFileDataRef::start_decoding(data).unwrap();
            assert_eq!(expected_size, size);
            assert_eq!(unsafe { decoder.size_unchecked() }, size);
            assert_eq!(decoder.size().unwrap(), size);
            assert_eq!(
                op.data.len(),
                decoder.length().complete_decoding().item.u32() as usize
            );
            assert_eq!(
                op,
                WriteFileDataRef {
                    group: decoder.group(),
                    response: decoder.response(),
                    file_id: decoder.file_id(),
                    offset: decoder.offset().complete_decoding().item,
                    data: decoder.data().item,
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
        let WithByteSize {
            item: ret,
            byte_size: size_decoded,
        } = WriteFileDataRef::decode(&encoded).unwrap();
        assert_eq!(size_encoded, size_decoded);
        assert_eq!(ret, op);

        // Test decode(data).encode_in() == data
        let mut encoded2 = [0_u8; TOT_SIZE];
        let size_encoded2 = op.encode_in(&mut encoded2).unwrap();
        assert_eq!(size_encoded, size_encoded2);
        assert_eq!(encoded2[..size_encoded2], encoded[..size_encoded]);
    }
}

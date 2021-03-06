use super::super::define::flag;
use super::super::define::op_code::OpCode;
use crate::decodable::{Decodable, EncodedData, SizeError, WithByteSize};
use crate::define::FileId;
use crate::varint::{self, EncodedVarint, Varint};

// TODO SPEC: Verify if the new ReadFileData successfull length overflow
// is described in the specification, because it is not intuitive.
//
// (RF(offset 0, length 4) of file(length = 1) return Response(length = 1)

/// Maximum byte size of an encoded `ReadFileData`
pub const MAX_SIZE: usize = 2 + 2 * varint::MAX_SIZE;

/// Read data from a file.
#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct ReadFileDataRef<'item> {
    /// Group with next action
    pub group: bool,
    /// Ask for a response (read data via ReturnFileData)
    ///
    /// Generally true unless you just want to trigger a read on the filesystem
    pub response: bool,
    /// File ID of the file to read
    pub file_id: FileId,
    /// Offset at which to start the reading
    pub offset: Varint,
    /// Number of bytes to read after offset
    pub length: Varint,
    /// Empty data required for lifetime compilation.
    pub phantom: core::marker::PhantomData<&'item ()>,
}

impl<'item> ReadFileDataRef<'item> {
    /// Most common builder `ReadFileData` builder.
    ///
    /// group = false
    /// response = true
    pub const fn new(file_id: FileId, offset: Varint, length: Varint) -> Self {
        Self {
            group: false,
            response: true,
            file_id,
            offset,
            length,
            phantom: core::marker::PhantomData,
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
        *out.add(0) = OpCode::ReadFileData as u8
            | if self.group { flag::GROUP } else { 0 }
            | if self.response { flag::RESPONSE } else { 0 };
        *out.add(1) = self.file_id.u8();
        size += 2;
        size += self.offset.encode_in_ptr(out.add(size));
        size += self.length.encode_in_ptr(out.add(size));
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
    pub const fn size(&self) -> usize {
        1 + 1 + self.offset.size() + self.length.size()
    }

    pub fn to_owned(&self) -> ReadFileData {
        ReadFileData {
            group: self.group,
            response: self.response,
            file_id: self.file_id,
            offset: self.offset,
            length: self.length,
        }
    }
}

pub struct EncodedReadFileData<'data> {
    data: &'data [u8],
}

impl<'data> EncodedReadFileData<'data> {
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

    /// # Safety
    /// You are to warrant, somehow, that the input byte array contains a complete item.
    /// Else this might result in out of bound reads, and absurd results.
    pub unsafe fn size_unchecked(&self) -> usize {
        let offset_size = self.offset().size_unchecked();
        let length_size =
            Varint::start_decoding_unchecked(self.data.get_unchecked(2 + offset_size..))
                .size_unchecked();
        2 + offset_size + length_size
    }
}

impl<'data> EncodedData<'data> for EncodedReadFileData<'data> {
    type DecodedData = ReadFileDataRef<'data>;
    unsafe fn new(data: &'data [u8]) -> Self {
        Self { data }
    }

    fn size(&self) -> Result<usize, SizeError> {
        unsafe {
            let mut size = 3;
            let data_size = self.data.len();
            if data_size < size {
                return Err(SizeError::MissingBytes);
            }
            size += self.offset().size_unchecked();
            if data_size < size {
                return Err(SizeError::MissingBytes);
            }
            size += Varint::start_decoding_unchecked(self.data.get_unchecked(size - 1..))
                .size_unchecked();
            size -= 1;
            if data_size < size {
                return Err(SizeError::MissingBytes);
            }
            Ok(size)
        }
    }

    fn complete_decoding(&self) -> WithByteSize<ReadFileDataRef<'data>> {
        let WithByteSize {
            item: offset,
            byte_size: offset_size,
        } = self.offset().complete_decoding();
        let WithByteSize {
            item: length,
            byte_size: length_size,
        } = unsafe { Varint::decode_unchecked(self.data.get_unchecked(2 + offset_size..)) };
        WithByteSize {
            item: ReadFileDataRef {
                group: self.group(),
                response: self.response(),
                file_id: self.file_id(),
                offset,
                length,
                phantom: core::marker::PhantomData,
            },
            byte_size: 2 + offset_size + length_size,
        }
    }
}

impl<'data> Decodable<'data> for ReadFileDataRef<'data> {
    type Data = EncodedReadFileData<'data>;
}

/// Read data from a file.
#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct ReadFileData {
    /// Group with next action
    pub group: bool,
    /// Ask for a response (read data via ReturnFileData)
    ///
    /// Generally true unless you just want to trigger a read on the filesystem
    pub response: bool,
    /// File ID of the file to read
    pub file_id: FileId,
    /// Offset at which to start the reading
    pub offset: Varint,
    /// Number of bytes to read after offset
    pub length: Varint,
}

impl ReadFileData {
    pub fn as_ref(&self) -> ReadFileDataRef {
        ReadFileDataRef {
            group: self.group,
            response: self.response,
            file_id: self.file_id,
            offset: self.offset,
            length: self.length,
            phantom: core::marker::PhantomData,
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
        fn test(op: ReadFileDataRef, data: &[u8]) {
            // Test op.encode_in() == data
            let mut encoded = [0_u8; 2 + 8];
            let size = op.encode_in(&mut encoded).unwrap();
            assert_eq!(size, data.len());
            assert_eq!(&encoded[..size], data);

            // Test decode(data) == op
            let WithByteSize {
                item: ret,
                byte_size: size,
            } = ReadFileDataRef::decode(data).unwrap();
            assert_eq!(size, data.len());
            assert_eq!(ret, op);

            // Test partial_decode == op
            let WithByteSize {
                item: decoder,
                byte_size: expected_size,
            } = ReadFileDataRef::start_decoding(data).unwrap();
            assert_eq!(expected_size, size);
            assert_eq!(unsafe { decoder.size_unchecked() }, size);
            assert_eq!(decoder.size().unwrap(), size);
            assert_eq!(
                op,
                ReadFileDataRef {
                    group: decoder.group(),
                    response: decoder.response(),
                    file_id: decoder.file_id(),
                    offset: decoder.offset().complete_decoding().item,
                    length: decoder.length().complete_decoding().item,
                    phantom: core::marker::PhantomData,
                }
            );
        }
        test(
            ReadFileDataRef {
                group: false,
                response: true,
                file_id: FileId::new(0),
                offset: Varint::new(0).unwrap(),
                length: Varint::new(0x3F_FF_FF_FF).unwrap(),
                phantom: core::marker::PhantomData,
            },
            &[0x41, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF],
        );
        test(
            ReadFileDataRef {
                group: true,
                response: false,
                file_id: FileId::new(1),
                offset: Varint::new(0x3F_FF).unwrap(),
                length: Varint::new(0x3F_FF_FF).unwrap(),
                phantom: core::marker::PhantomData,
            },
            &[0x81, 0x01, 0x7F, 0xFF, 0xBF, 0xFF, 0xFF],
        );
        test(
            ReadFileDataRef {
                group: true,
                response: true,
                file_id: FileId::new(0x80),
                offset: Varint::new(0).unwrap(),
                length: Varint::new(0).unwrap(),
                phantom: core::marker::PhantomData,
            },
            &[0xC1, 0x80, 0x00, 0x00],
        );
        test(
            ReadFileDataRef {
                group: false,
                response: false,
                file_id: FileId::new(0xFF),
                offset: Varint::new(0x3F_FF_FF_FF).unwrap(),
                length: Varint::new(0x3F_FF_FF_FF).unwrap(),
                phantom: core::marker::PhantomData,
            },
            &[0x01, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
        );
    }

    #[test]
    fn consistence() {
        let op = ReadFileDataRef {
            group: true,
            response: false,
            file_id: FileId::new(0x80),
            offset: Varint::new(89).unwrap(),
            length: Varint::new(0xFF_FF_FF).unwrap(),
            phantom: core::marker::PhantomData,
        };

        // Test decode(op.encode_in()) == op
        let mut encoded = [0_u8; MAX_SIZE];
        let size_encoded = op.encode_in(&mut encoded).unwrap();
        let WithByteSize {
            item: ret,
            byte_size: size_decoded,
        } = ReadFileDataRef::decode(&encoded).unwrap();
        assert_eq!(size_encoded, size_decoded);
        assert_eq!(ret, op);

        // Test decode(data).encode_in() == data
        let mut encoded2 = [0_u8; MAX_SIZE];
        let size_encoded2 = op.encode_in(&mut encoded2).unwrap();
        assert_eq!(size_encoded, size_encoded2);
        assert_eq!(encoded2[..size_encoded2], encoded[..size_encoded]);
    }
}

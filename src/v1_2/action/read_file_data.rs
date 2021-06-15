use crate::decodable::{Decodable, EncodedData, SizeError, WithByteSize};
use crate::define::FileId;
use crate::encodable::Encodable;
use crate::v1_2::define::flag;
use crate::v1_2::define::op_code;
use crate::varint::{self, EncodedVarint, EncodedVarintMut, Varint};

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
pub struct ReadFileDataRef<'data> {
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
    pub phantom: core::marker::PhantomData<&'data ()>,
}

impl<'data> ReadFileDataRef<'data> {
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

impl<'data> Encodable for ReadFileDataRef<'data> {
    unsafe fn encode_in_ptr(&self, out: *mut u8) -> usize {
        let mut size = 0;
        *out.add(0) = op_code::READ_FILE_DATA as u8
            | if self.group { flag::GROUP } else { 0 }
            | if self.response { flag::RESPONSE } else { 0 };
        *out.add(1) = self.file_id.u8();
        size += 2;
        size += self.offset.encode_in_ptr(out.add(size));
        size += self.length.encode_in_ptr(out.add(size));
        size
    }

    fn encoded_size(&self) -> usize {
        1 + 1 + self.offset.encoded_size() + self.length.encoded_size()
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

    pub fn offset(&self) -> EncodedVarint<'data> {
        unsafe { Varint::start_decoding_unchecked(self.data.get_unchecked(2..)) }
    }

    pub fn length(&self) -> EncodedVarint<'data> {
        unsafe {
            let offset_size = (((*self.data.get_unchecked(2) & 0xC0) >> 6) + 1) as usize;
            Varint::start_decoding_unchecked(self.data.get_unchecked(2 + offset_size..))
        }
    }

    /// # Safety
    /// You have to warrant that somehow that there is enough byte to decode the encoded size.
    /// If you fail to do so, out of bound bytes will be read, and an absurd value will be
    /// returned.
    pub unsafe fn encoded_size_unchecked(&self) -> usize {
        let offset_size = self.offset().encoded_size_unchecked();
        let length_size =
            Varint::start_decoding_unchecked(self.data.get_unchecked(2 + offset_size..))
                .encoded_size_unchecked();
        2 + offset_size + length_size
    }
}

impl<'data> EncodedData<'data> for EncodedReadFileData<'data> {
    type SourceData = &'data [u8];
    type DecodedData = ReadFileDataRef<'data>;

    unsafe fn new(data: Self::SourceData) -> Self {
        Self { data }
    }

    fn encoded_size(&self) -> Result<usize, SizeError> {
        unsafe {
            let mut size = 3;
            let data_size = self.data.len();
            if data_size < size {
                return Err(SizeError::MissingBytes);
            }
            size += self.offset().encoded_size_unchecked();
            if data_size < size {
                return Err(SizeError::MissingBytes);
            }
            size += Varint::start_decoding_unchecked(self.data.get_unchecked(size - 1..))
                .encoded_size_unchecked();
            size -= 1;
            if data_size < size {
                return Err(SizeError::MissingBytes);
            }
            Ok(size)
        }
    }

    fn complete_decoding(&self) -> WithByteSize<Self::DecodedData> {
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

pub struct EncodedReadFileDataMut<'data> {
    data: &'data mut [u8],
}

crate::make_downcastable!(EncodedReadFileDataMut, EncodedReadFileData);

impl<'data> EncodedReadFileDataMut<'data> {
    pub fn group(&self) -> bool {
        self.as_ref().group()
    }

    pub fn response(&self) -> bool {
        self.as_ref().response()
    }

    pub fn file_id(&self) -> FileId {
        self.as_ref().file_id()
    }

    pub fn offset(&self) -> EncodedVarint<'data> {
        self.as_ref().offset()
    }

    pub fn length(&self) -> EncodedVarint<'data> {
        self.as_ref().length()
    }

    /// # Safety
    /// You have to warrant that somehow that there is enough byte to decode the encoded size.
    /// If you fail to do so, out of bound bytes will be read, and an absurd value will be
    /// returned.
    pub unsafe fn encoded_size_unchecked(&self) -> usize {
        self.as_ref().encoded_size_unchecked()
    }

    pub fn set_group(&mut self, group: bool) {
        unsafe {
            if group {
                *self.data.get_unchecked_mut(0) |= flag::GROUP
            } else {
                *self.data.get_unchecked_mut(0) &= !flag::GROUP
            }
        }
    }

    pub fn set_response(&mut self, response: bool) {
        unsafe {
            if response {
                *self.data.get_unchecked_mut(0) |= flag::RESPONSE
            } else {
                *self.data.get_unchecked_mut(0) &= !flag::RESPONSE
            }
        }
    }

    pub fn set_file_id(&mut self, file_id: FileId) {
        unsafe { *self.data.get_unchecked_mut(1) = file_id.u8() }
    }

    pub fn offset_mut(&mut self) -> EncodedVarintMut {
        unsafe { Varint::start_decoding_unchecked_mut(self.data.get_unchecked_mut(2..)) }
    }

    pub fn length_mut(&mut self) -> EncodedVarintMut {
        unsafe {
            let offset_size = self.offset().encoded_size_unchecked() as usize;
            Varint::start_decoding_unchecked_mut(self.data.get_unchecked_mut(2 + offset_size..))
        }
    }
}

impl<'data> EncodedData<'data> for EncodedReadFileDataMut<'data> {
    type SourceData = &'data mut [u8];
    type DecodedData = ReadFileDataRef<'data>;

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

impl<'data> Decodable<'data> for ReadFileDataRef<'data> {
    type Data = EncodedReadFileData<'data>;
    type DataMut = EncodedReadFileDataMut<'data>;
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
            assert_eq!(unsafe { decoder.encoded_size_unchecked() }, size);
            assert_eq!(decoder.encoded_size().unwrap(), size);
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

            // Test partial mutability
            let WithByteSize {
                item: mut decoder_mut,
                byte_size: expected_size,
            } = ReadFileDataRef::start_decoding_mut(&mut encoded).unwrap();
            assert_eq!(expected_size, size);

            assert_eq!(decoder_mut.group(), op.group);
            let new_group = !op.group;
            assert!(new_group != op.group);
            decoder_mut.set_group(new_group);
            assert_eq!(decoder_mut.group(), new_group);

            assert_eq!(decoder_mut.response(), op.response);
            let new_response = !op.response;
            assert!(new_response != op.response);
            decoder_mut.set_response(new_response);
            assert_eq!(decoder_mut.response(), new_response);

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

            {
                let original = op.length;
                let mut decoder_mut = decoder_mut.length_mut();
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

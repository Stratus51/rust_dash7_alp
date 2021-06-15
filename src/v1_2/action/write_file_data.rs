use crate::decodable::{Decodable, EncodedData, SizeError, WithByteSize};
#[cfg(feature = "alloc")]
use crate::define::EncodableData;
use crate::define::{EncodableDataRef, FileId};
use crate::encodable::Encodable;
use crate::v1_2::define::{flag, op_code};
use crate::varint::{EncodedVarint, EncodedVarintMut, Varint};

// TODO Is it the role of this library to teach the semantics of the protocol or should it just
// focus on documenting its usage, based on the assumption that that semantic is already known?
/// Writes data to a file.
#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct WriteFileDataRef<'data> {
    /// Group with next action
    pub group: bool,
    /// Ask for a response (a status)
    pub response: bool,
    /// File ID of the file to read
    pub file_id: FileId,
    /// Offset at which to start the reading
    pub offset: Varint,
    /// Data to write
    pub data: EncodableDataRef<'data>,
}

impl<'data> WriteFileDataRef<'data> {
    /// Most common builder `WriteFileData` builder.
    ///
    /// group = false
    /// response = true
    pub fn new(file_id: FileId, offset: Varint, data: EncodableDataRef<'data>) -> Self {
        Self {
            group: false,
            response: true,
            file_id,
            offset,
            data,
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

impl<'data> Encodable for WriteFileDataRef<'data> {
    unsafe fn encode_in_ptr(&self, out: *mut u8) -> usize {
        let mut size = 0;
        *out.add(0) = op_code::WRITE_FILE_DATA
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

    fn encoded_size(&self) -> usize {
        let length = unsafe { Varint::new_unchecked(self.data.len() as u32) };
        1 + 1 + self.offset.encoded_size() + length.encoded_size()
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

    pub fn offset(&self) -> EncodedVarint<'data> {
        unsafe { Varint::start_decoding_unchecked(self.data.get_unchecked(2..)) }
    }

    pub fn length(&self) -> EncodedVarint<'data> {
        unsafe {
            let offset_size = self.offset().encoded_size_unchecked() as usize;
            Varint::start_decoding_unchecked(self.data.get_unchecked(2 + offset_size..))
        }
    }

    /// Return the payload
    pub fn data(&self) -> &'data [u8] {
        unsafe {
            let offset_size = self.offset().encoded_size_unchecked() as usize;
            let WithByteSize {
                item: length,
                byte_size: length_size,
            } = Varint::decode_unchecked(self.data.get_unchecked(2 + offset_size..));
            let data_offset = 2 + offset_size + length_size;
            self.data
                .get_unchecked(data_offset..)
                .get_unchecked(..length.usize())
        }
    }

    /// # Safety
    /// You have to warrant that somehow that there is enough byte to decode the encoded size.
    /// If you fail to do so, out of bound bytes will be read, and an absurd value will be
    /// returned.
    pub unsafe fn encoded_size_unchecked(&self) -> usize {
        let offset_size = self.offset().encoded_size_unchecked();
        let WithByteSize {
            item: length,
            byte_size: length_size,
        } = Varint::decode_unchecked(self.data.get_unchecked(2 + offset_size..));
        2 + offset_size + length_size + length.usize()
    }
}

impl<'data> EncodedData<'data> for EncodedWriteFileData<'data> {
    type SourceData = &'data [u8];
    type DecodedData = WriteFileDataRef<'data>;

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
            let WithByteSize {
                item: length,
                byte_size: length_size,
            } = Varint::decode_unchecked(self.data.get_unchecked(size - 1..));
            size += length.usize() + length_size - 1;
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

pub struct EncodedWriteFileDataMut<'data> {
    data: &'data mut [u8],
}

crate::make_downcastable!(EncodedWriteFileDataMut, EncodedWriteFileData);

impl<'data> EncodedWriteFileDataMut<'data> {
    pub fn group(&self) -> bool {
        self.as_ref().group()
    }

    pub fn response(&self) -> bool {
        self.as_ref().response()
    }

    pub fn file_id(&self) -> FileId {
        self.as_ref().file_id()
    }

    pub fn offset(&self) -> EncodedVarint {
        self.as_ref().offset()
    }

    pub fn length(&self) -> EncodedVarint {
        self.as_ref().length()
    }

    pub fn data(&self) -> &'data [u8] {
        self.as_ref().data()
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

    /// # Safety
    /// You are not supposed to modify the length without changing the following data because it
    /// indicates its size. Unless you know very well what you are doing.
    pub unsafe fn length_mut(&mut self) -> EncodedVarintMut {
        let offset_size = self.offset().encoded_size_unchecked() as usize;
        Varint::start_decoding_unchecked_mut(self.data.get_unchecked_mut(2 + offset_size..))
    }

    pub fn data_mut(&mut self) -> &mut [u8] {
        unsafe {
            let offset_size = self.offset().encoded_size_unchecked() as usize;
            let WithByteSize {
                item: length,
                byte_size: length_size,
            } = Varint::decode_unchecked(self.data.get_unchecked(2 + offset_size..));
            let data_offset = 2 + offset_size + length_size;
            self.data
                .get_unchecked_mut(data_offset..)
                .get_unchecked_mut(..length.usize())
        }
    }
}

impl<'data> EncodedData<'data> for EncodedWriteFileDataMut<'data> {
    type SourceData = &'data mut [u8];
    type DecodedData = WriteFileDataRef<'data>;

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

impl<'data> Decodable<'data> for WriteFileDataRef<'data> {
    type Data = EncodedWriteFileData<'data>;
    type DataMut = EncodedWriteFileDataMut<'data>;
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
            assert_eq!(unsafe { decoder.encoded_size_unchecked() }, size);
            assert_eq!(decoder.encoded_size().unwrap(), size);
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
                    data: unsafe { EncodableDataRef::new_unchecked(decoder.data()) },
                }
            );

            // Test partial mutability
            let WithByteSize {
                item: mut decoder_mut,
                byte_size: expected_size,
            } = WriteFileDataRef::start_decoding_mut(&mut encoded).unwrap();
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
                let mut offset = decoder_mut.offset_mut();
                assert_eq!(offset.complete_decoding().item.u32(), original.u32());
                let new_value = Varint::new(if original.encoded_size() == 1 {
                    (original.u32() == 0) as u32
                } else {
                    original.u32() ^ 0x3F
                })
                .unwrap();
                assert!(new_value != original);
                offset.set_value(&new_value).unwrap();
                assert_eq!(offset.complete_decoding().item, new_value);
            }

            if !op.data.is_empty() {
                let original = &op.data;
                let mut new_data = vec![0_u8; original.len()];
                let data_mut = decoder_mut.data_mut();
                for (i, b) in original.data().iter().enumerate() {
                    new_data[i] = !b;
                    data_mut[i] = new_data[i];
                }
                assert!(&new_data[..] != original.data());
                assert_eq!(decoder_mut.data(), &new_data);
            }
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

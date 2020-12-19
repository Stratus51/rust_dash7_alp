use super::error::BasicDecodeError;
use super::flag;
use super::op_code::OpCode;
use crate::defines::{EncodableData, FileId};
use crate::varint::Varint;

/// WriteFileData builder

/// Write data to a file
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct WriteFileData<'a> {
    /// Group with next action
    pub group: bool,
    /// Ask for a response (a status)
    pub response: bool,
    /// File ID of the file to read
    pub file_id: FileId,
    /// Offset at which to start the reading
    pub offset: Varint,
    /// Data to write
    pub data: EncodableData<'a>,
}

impl<'a> WriteFileData<'a> {
    /// Most common builder WriteFileData builder.
    ///
    /// group = false
    /// response = true
    pub fn new(file_id: FileId, offset: Varint, data: EncodableData<'a>) -> Self {
        Self {
            group: false,
            response: true,
            file_id,
            offset,
            data,
        }
    }

    /// Encodes the Item without checking the size of the receiving
    /// byte array.
    ///
    /// # Safety
    /// You are responsible for checking that `size` == [self.size()](#method.size) and
    /// to insure `out.len() >= size`. Failing that will result in the
    /// program writing out of bound. In the current implementation, it
    /// will trigger a panic.
    pub unsafe fn encode_in_unchecked(&self, buf: &mut [u8]) -> usize {
        let mut size = 0;
        *buf.get_unchecked_mut(0) = OpCode::WriteFileData as u8
            + if self.group { flag::GROUP } else { 0 }
            + if self.response { flag::RESPONSE } else { 0 };
        *buf.get_unchecked_mut(1) = self.file_id.u8();
        size += 2;
        size += self
            .offset
            .encode_in_unchecked(buf.get_unchecked_mut(size..));
        let length = Varint::new_unchecked(self.data.len() as u32);
        size += length.encode_in_unchecked(buf.get_unchecked_mut(size..));
        buf.get_unchecked_mut(size..(size + length.get() as usize))
            .copy_from_slice(self.data.get());
        size += length.get() as usize;
        size
    }

    /// Encodes the value into pre allocated array.
    ///
    /// Fails if the pre allocated array is smaller than [self.size()](#method.size)
    /// returning the number of input bytes required.
    pub fn encode_in(&self, out: &mut [u8]) -> Result<usize, usize> {
        let size = self.size();
        if out.len() >= size {
            Ok(unsafe { self.encode_in_unchecked(out) })
        } else {
            Err(size)
        }
    }

    /// Size in bytes of the encoded equivalent of the item.
    pub fn size(&self) -> usize {
        let length = unsafe { Varint::new_unchecked(self.data.len() as u32) };
        1 + 1 + self.offset.size() + length.size()
    }

    /// Creates a decodable item without checking the data size.
    ///
    /// # Safety
    /// You are to check that data is not empty and that data.len() >=
    /// [DecodableVarint.size()](struct.DecodableWriteFileData.html#method.size)
    /// (the expected byte size of the returned DecodableItem).
    ///
    /// You are also expected to warrant that the opcode contained in the
    /// first byte corresponds to this action.
    pub const unsafe fn start_decoding_unchecked(data: &[u8]) -> DecodableWriteFileData {
        DecodableWriteFileData::new(data)
    }

    /// Creates a decodable item.
    ///
    /// This decodable item allows each parts of the item independently.
    pub const fn start_decoding(data: &[u8]) -> Result<DecodableWriteFileData, BasicDecodeError> {
        if data.is_empty() {
            return Err(BasicDecodeError::MissingBytes(1));
        }
        if data[0] & 0x3F != OpCode::WriteFileData as u8 {
            return Err(BasicDecodeError::BadOpCode);
        }
        let ret = unsafe { Self::start_decoding_unchecked(data) };
        let ret_size = ret.size();
        if data.len() < ret_size {
            return Err(BasicDecodeError::MissingBytes(ret_size));
        }
        Ok(ret)
    }

    /// Decodes the Item from bytes.
    ///
    /// Returns the decoded data and the number of bytes consumed to produce it.
    ///
    /// # Safety
    /// You are to check that data is not empty and that data.len() >=
    /// [DecodableWriteFileData.size()](struct.DecodableWriteFileData.html#method.size)
    /// (the expected byte size of the returned DecodableItem).
    pub fn decode_unchecked(data: &'a [u8]) -> (Self, usize) {
        unsafe { Self::start_decoding_unchecked(data).complete_decoding() }
    }

    /// Decodes the item from bytes.
    ///
    /// On success, returns the decoded data and the number of bytes consumed
    /// to produce it.
    pub fn decode(data: &'a [u8]) -> Result<(Self, usize), BasicDecodeError> {
        match Self::start_decoding(data) {
            Ok(v) => Ok(v.complete_decoding()),
            Err(e) => Err(e),
        }
    }
}

pub struct DecodableWriteFileData<'a, 'b: 'a> {
    data: &'b [u8],
    phantom: core::marker::PhantomData<&'a ()>,
}

impl<'a, 'b: 'a> DecodableWriteFileData<'a, 'b> {
    const fn new(data: &'b [u8]) -> Self {
        Self {
            data,
            phantom: core::marker::PhantomData,
        }
    }

    /// Decodes the size of the Item in bytes
    pub const fn size(&self) -> usize {
        1
    }

    pub fn group(&self) -> bool {
        unsafe { *self.data.get_unchecked(0) & flag::GROUP != 0 }
    }

    pub fn response(&self) -> bool {
        unsafe { *self.data.get_unchecked(0) & flag::RESPONSE != 0 }
    }

    pub fn file_id(&self) -> FileId {
        unsafe { FileId(*self.data.get_unchecked(1)) }
    }

    pub fn offset(&self) -> (Varint, usize) {
        unsafe { Varint::decode_unchecked(self.data.get_unchecked(2..)) }
    }

    pub fn length(&self) -> (Varint, usize) {
        unsafe {
            let offset_size = (((*self.data.get_unchecked(2) & 0xC0) >> 6) + 1) as usize;
            Varint::decode_unchecked(self.data.get_unchecked(2 + offset_size..))
        }
    }

    pub fn data(&self) -> (&'a [u8], usize) {
        unsafe {
            let offset_size = (((*self.data.get_unchecked(2) & 0xC0) >> 6) + 1) as usize;
            let (length, length_size) =
                Varint::decode_unchecked(self.data.get_unchecked(2 + offset_size..));
            let data_offset = 2 + offset_size + length_size;
            let data = self
                .data
                .get_unchecked(data_offset..data_offset + length.get() as usize);
            (data, length_size + length.get() as usize)
        }
    }

    /// Fully decode the Item
    ///
    /// Returns the decoded data and the number of bytes consumed to produce it.
    pub fn complete_decoding(&self) -> (WriteFileData<'b>, usize) {
        let (offset, offset_size) = self.offset();
        let (data, length_size, length) = unsafe {
            let (length, length_size) =
                Varint::decode_unchecked(self.data.get_unchecked(2 + offset_size..));
            let data_offset = 2 + offset_size + length_size;
            let data = self
                .data
                .get_unchecked(data_offset..data_offset + length.get() as usize);
            (
                EncodableData::new_unchecked(data),
                length_size,
                length.get(),
            )
        };
        (
            WriteFileData {
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn known() {
        fn test(op: WriteFileData, data: &[u8]) {
            // Test op.encode_in() == data
            let mut encoded = [0u8; 2 + 8];
            let size = op.encode_in(&mut encoded).unwrap();
            assert_eq!(size, data.len());
            assert_eq!(&encoded[..size], data);

            // Test decode(data) == op
            let (ret, size) = WriteFileData::decode(&data).unwrap();
            assert_eq!(size, data.len());
            assert_eq!(ret, op);
        }
        test(
            WriteFileData {
                group: false,
                response: true,
                file_id: FileId::new(0),
                offset: Varint::new(0).unwrap(),
                data: EncodableData::new(&[0, 1, 2]).unwrap(),
            },
            &[0x44, 0x00, 0x00, 0x03, 0x00, 0x01, 0x02],
        );
        test(
            WriteFileData {
                group: true,
                response: false,
                file_id: FileId::new(1),
                offset: Varint::new(0x3F_FF).unwrap(),
                data: EncodableData::new(&[]).unwrap(),
            },
            &[0x84, 0x01, 0x7F, 0xFF, 0x00],
        );
        test(
            WriteFileData {
                group: true,
                response: true,
                file_id: FileId::new(0x80),
                offset: Varint::new(0).unwrap(),
                data: EncodableData::new(&[0x44]).unwrap(),
            },
            &[0xC4, 0x80, 0x00, 0x01, 0x44],
        );
        test(
            WriteFileData {
                group: false,
                response: false,
                file_id: FileId::new(0xFF),
                offset: Varint::new(0x3F_FF_FF_FF).unwrap(),
                data: EncodableData::new(&[0xFF, 0xFE]).unwrap(),
            },
            &[0x04, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x02, 0xFF, 0xFE],
        );
    }

    #[test]
    fn consistence() {
        let op = WriteFileData {
            group: true,
            response: false,
            file_id: FileId::new(0x80),
            offset: Varint::new(89).unwrap(),
            data: EncodableData::new(&[0xFF, 0xFE]).unwrap(),
        };

        // Test decode(op.encode_in()) == op
        const TOT_SIZE: usize = 2 + 2 + 3;
        let mut encoded = [0u8; TOT_SIZE];
        let size_encoded = op.encode_in(&mut encoded).unwrap();
        let (ret, size_decoded) = WriteFileData::decode(&encoded).unwrap();
        assert_eq!(size_encoded, size_decoded);
        assert_eq!(ret, op);

        // Test decode(data).encode_in() == data
        let mut encoded2 = [0u8; TOT_SIZE];
        let size_encoded2 = op.encode_in(&mut encoded2).unwrap();
        assert_eq!(size_encoded, size_encoded2);
        assert_eq!(encoded2[..size_encoded2], encoded[..size_encoded]);
    }
}

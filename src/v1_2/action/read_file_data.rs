use super::defines::FileId;
use super::error::BasicDecodeError;
use super::flag;
use super::op_code::OpCode;
use crate::varint::{self, Varint};

// TODO SPEC: Verify if the new ReadFileData successfull length overflow
// is described in the specification, because it is not intuitive.
//
// (RF(offset 0, length 4) of file(length = 1) return Response(length = 1)

/// Maximum byte size of an encoded ReadFileData
pub const MAX_SIZE: usize = 2 + 2 * varint::MAX_SIZE;

/// Read data from a file
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
    /// Most common builder ReadFileData builder.
    ///
    /// group = false
    /// response = true
    pub fn new(file_id: FileId, offset: Varint, length: Varint) -> Self {
        Self {
            group: false,
            response: true,
            file_id,
            offset,
            length,
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
        *buf.get_unchecked_mut(0) = OpCode::ReadFileData as u8
            + if self.group { flag::GROUP } else { 0 }
            + if self.response { flag::RESPONSE } else { 0 };
        *buf.get_unchecked_mut(1) = self.file_id.u8();
        size += 2;
        size += self
            .offset
            .encode_in_unchecked(buf.get_unchecked_mut(size..));
        size += self
            .length
            .encode_in_unchecked(buf.get_unchecked_mut(size..));
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
    pub const fn size(&self) -> usize {
        1 + 1 + self.offset.size() + self.length.size()
    }

    /// Creates a decodable item without checking the data size.
    ///
    /// # Safety
    /// You are to check that data is not empty and that data.len() >=
    /// [DecodableVarint.size()](struct.DecodableReadFileData.html#method.size)
    /// (the expected byte size of the returned DecodableItem).
    ///
    /// You are also expected to warrant that the opcode contained in the
    /// first byte corresponds to this action.
    pub const unsafe fn start_decoding_unchecked(data: &[u8]) -> DecodableReadFileData {
        DecodableReadFileData::new(data)
    }

    /// Creates a decodable item.
    ///
    /// This decodable item allows each parts of the item independently.
    pub const fn start_decoding(data: &[u8]) -> Result<DecodableReadFileData, BasicDecodeError> {
        if data.is_empty() {
            return Err(BasicDecodeError::MissingBytes(1));
        }
        if data[0] & 0x3F != OpCode::ReadFileData as u8 {
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
    /// [DecodableReadFileData.size()](struct.DecodableReadFileData.html#method.size)
    /// (the expected byte size of the returned DecodableItem).
    pub fn decode_unchecked(data: &[u8]) -> (Self, usize) {
        unsafe { Self::start_decoding_unchecked(data).complete_decoding() }
    }

    /// Decodes the item from bytes.
    ///
    /// On success, returns the decoded data and the number of bytes consumed
    /// to produce it.
    pub fn decode(data: &[u8]) -> Result<(Self, usize), BasicDecodeError> {
        match Self::start_decoding(data) {
            Ok(v) => Ok(v.complete_decoding()),
            Err(e) => Err(e),
        }
    }
}

pub struct DecodableReadFileData<'a> {
    data: &'a [u8],
}

impl<'a> DecodableReadFileData<'a> {
    const fn new(data: &'a [u8]) -> Self {
        Self { data }
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

    /// Fully decode the Item
    ///
    /// Returns the decoded data and the number of bytes consumed to produce it.
    pub fn complete_decoding(&self) -> (ReadFileData, usize) {
        let (offset, offset_size) = self.offset();
        let (length, length_size) =
            unsafe { Varint::decode_unchecked(self.data.get_unchecked(2 + offset_size..)) };
        (
            ReadFileData {
                group: self.group(),
                response: self.response(),
                file_id: self.file_id(),
                offset,
                length,
            },
            2 + offset_size + length_size,
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn known() {
        fn test(op: ReadFileData, data: &[u8]) {
            let mut encoded = [0u8; 2 + 8];
            let size = op.encode_in(&mut encoded).unwrap();
            assert_eq!(size, data.len());
            assert_eq!(&encoded, data);
            let (ret, size) = ReadFileData::decode(&data).unwrap();
            assert_eq!(size, data.len());
            assert_eq!(ret, op);
        }
        test(
            ReadFileData {
                group: false,
                response: true,
                file_id: FileId::new(0),
                offset: Varint::new(0).unwrap(),
                length: Varint::new(0x3F_FF_FF_FF).unwrap(),
            },
            &[0x40, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF],
        );
        test(
            ReadFileData {
                group: true,
                response: false,
                file_id: FileId::new(1),
                offset: Varint::new(0x3F_FF).unwrap(),
                length: Varint::new(0x3F_FF_FF).unwrap(),
            },
            &[0x80, 0x01, 0x7F, 0xFF, 0xBF, 0xFF, 0xFF],
        );
        test(
            ReadFileData {
                group: true,
                response: true,
                file_id: FileId::new(0x80),
                offset: Varint::new(0).unwrap(),
                length: Varint::new(0).unwrap(),
            },
            &[0xC0, 0x80, 0x00, 0x00],
        );
        test(
            ReadFileData {
                group: false,
                response: false,
                file_id: FileId::new(0xFF),
                offset: Varint::new(0x3F_FF_FF_FF).unwrap(),
                length: Varint::new(0x3F_FF_FF_FF).unwrap(),
            },
            &[0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
        );
    }

    #[test]
    fn consistence() {
        let op = ReadFileData {
            group: true,
            response: false,
            file_id: FileId::new(0x80),
            offset: Varint::new(89).unwrap(),
            length: Varint::new(0xFF_FF_FF).unwrap(),
        };
        let mut encoded = [0u8; MAX_SIZE];
        let size_encoded = op.encode_in(&mut encoded).unwrap();
        let (ret, size_decoded) = ReadFileData::decode(&encoded).unwrap();
        assert_eq!(size_encoded, size_decoded);
        assert_eq!(ret, op);

        let mut encoded2 = [0u8; MAX_SIZE];
        let size_decoded2 = op.encode_in(&mut encoded2).unwrap();
        assert_eq!(size_encoded, size_decoded2);
        assert_eq!(encoded2, encoded);
    }
}

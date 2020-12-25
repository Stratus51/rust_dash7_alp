use super::super::define::flag;
use super::super::define::op_code::OpCode;
use super::super::error::BasicDecodeError;
use crate::define::FileId;
use crate::varint::{self, DecodableVarint, Varint};

// TODO SPEC: Verify if the new ReadFileData successfull length overflow
// is described in the specification, because it is not intuitive.
//
// (RF(offset 0, length 4) of file(length = 1) return Response(length = 1)

/// Maximum byte size of an encoded `ReadFileData`
pub const MAX_SIZE: usize = 2 + 2 * varint::MAX_SIZE;

/// Required size of a data buffer to determine the size of a resulting
/// decoded object
pub const HEADER_SIZE: usize = 1;

/// Read data from a file.
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
    /// - The first byte contains this action's opcode.
    /// - The data is bigger than `HEADER_SIZE`.
    /// - The decoded data is bigger than the expected size of the `decodable` object.
    /// Meaning that given the resulting decodable object `decodable`:
    /// `data.len()` >= [`decodable.size()`](struct.DecodableReadFileData.html#method.size).
    ///
    /// Failing that might result in reading and interpreting data outside the given
    /// array (depending on what is done with the resulting object).
    pub const unsafe fn start_decoding_ptr<'data>(data: *const u8) -> DecodableReadFileData<'data> {
        DecodableReadFileData::from_ptr(data)
    }

    /// Creates a decodable item without checking the data size.
    ///
    /// # Safety
    /// You are to check that:
    /// - The first byte contains this action's opcode.
    /// - The data is bigger than `HEADER_SIZE`.
    /// - The decoded data is bigger than the expected size of the `decodable` object.
    /// Meaning that given the resulting decodable object `decodable`:
    /// `data.len()` >= [`decodable.size()`](struct.DecodableReadFileData.html#method.size).
    ///
    /// Failing that might result in reading and interpreting data outside the given
    /// array (depending on what is done with the resulting object).
    pub const unsafe fn start_decoding_unchecked(data: &[u8]) -> DecodableReadFileData {
        DecodableReadFileData::new(data)
    }

    /// Creates a decodable item.
    ///
    /// This decodable item allows each parts of the item independently.
    ///
    /// # Errors
    /// - Fails if first byte of the data contains the wrong opcode.
    /// - Fails if `data.len()` < `HEADER_SIZE`.
    /// - Fails if data is smaller then the decoded expected size.
    pub fn start_decoding(data: &[u8]) -> Result<DecodableReadFileData, BasicDecodeError> {
        match data.get(0) {
            None => return Err(BasicDecodeError::MissingBytes(HEADER_SIZE)),
            Some(byte) => {
                if *byte & 0x3F != OpCode::ReadFileData as u8 {
                    return Err(BasicDecodeError::BadOpCode);
                }
            }
        }
        let ret = unsafe { Self::start_decoding_unchecked(data) };
        let ret_size = ret.size();
        if data.len() < ret_size {
            return Err(BasicDecodeError::MissingBytes(ret_size));
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
    /// May attempt to read bytes after the end of the array.
    ///
    /// You are to check that:
    /// - The first byte contains this action's opcode.
    /// - The data is bigger than `HEADER_SIZE` (to be sure the Item size will be
    /// decoded correctly).
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
    /// - The first byte contains this action's opcode.
    /// - The data is bigger than `HEADER_SIZE` (to be sure the Item size will be
    /// decoded correctly).
    /// - The resulting size of the data consumed is smaller than the size of the
    /// decoded data.
    ///
    /// Failing that will result in reading and interpreting data outside the given
    /// array.
    pub unsafe fn decode_unchecked(data: &[u8]) -> (Self, usize) {
        Self::start_decoding_unchecked(data).complete_decoding()
    }

    /// Decodes the item from bytes.
    ///
    /// On success, returns the decoded data and the number of bytes consumed
    /// to produce it.
    ///
    /// # Errors
    /// - Fails if first byte of the data contains the wrong opcode.
    /// - Fails if `data.len()` < `HEADER_SIZE`.
    /// - Fails if data is smaller then the decoded expected size.
    pub fn decode(data: &[u8]) -> Result<(Self, usize), BasicDecodeError> {
        match Self::start_decoding(data) {
            Ok(v) => Ok(v.complete_decoding()),
            Err(e) => Err(e),
        }
    }
}

pub struct DecodableReadFileData<'data> {
    data: *const u8,
    data_life: core::marker::PhantomData<&'data ()>,
}

impl<'data> DecodableReadFileData<'data> {
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
        let offset_size = self.offset().size();
        let length_size =
            unsafe { Varint::start_decoding_ptr(self.data.add(2 + offset_size)).size() };
        2 + offset_size + length_size
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

    /// Fully decode the Item
    ///
    /// Returns the decoded data and the number of bytes consumed to produce it.
    pub fn complete_decoding(&self) -> (ReadFileData, usize) {
        let (offset, offset_size) = self.offset().complete_decoding();
        let (length, length_size) = unsafe { Varint::decode_ptr(self.data.add(2 + offset_size)) };
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
            // Test op.encode_in() == data
            let mut encoded = [0_u8; 2 + 8];
            let size = op.encode_in(&mut encoded).unwrap();
            assert_eq!(size, data.len());
            assert_eq!(&encoded[..size], data);

            // Test decode(data) == op
            let (ret, size) = ReadFileData::decode(data).unwrap();
            assert_eq!(size, data.len());
            assert_eq!(ret, op);

            // Test partial_decode == op
            let decoder = ReadFileData::start_decoding(data).unwrap();
            assert_eq!(size, decoder.size());
            assert_eq!(
                op,
                ReadFileData {
                    group: decoder.group(),
                    response: decoder.response(),
                    file_id: decoder.file_id(),
                    offset: decoder.offset().complete_decoding().0,
                    length: decoder.length().complete_decoding().0,
                }
            );
        }
        test(
            ReadFileData {
                group: false,
                response: true,
                file_id: FileId::new(0),
                offset: Varint::new(0).unwrap(),
                length: Varint::new(0x3F_FF_FF_FF).unwrap(),
            },
            &[0x41, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF],
        );
        test(
            ReadFileData {
                group: true,
                response: false,
                file_id: FileId::new(1),
                offset: Varint::new(0x3F_FF).unwrap(),
                length: Varint::new(0x3F_FF_FF).unwrap(),
            },
            &[0x81, 0x01, 0x7F, 0xFF, 0xBF, 0xFF, 0xFF],
        );
        test(
            ReadFileData {
                group: true,
                response: true,
                file_id: FileId::new(0x80),
                offset: Varint::new(0).unwrap(),
                length: Varint::new(0).unwrap(),
            },
            &[0xC1, 0x80, 0x00, 0x00],
        );
        test(
            ReadFileData {
                group: false,
                response: false,
                file_id: FileId::new(0xFF),
                offset: Varint::new(0x3F_FF_FF_FF).unwrap(),
                length: Varint::new(0x3F_FF_FF_FF).unwrap(),
            },
            &[0x01, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
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

        // Test decode(op.encode_in()) == op
        let mut encoded = [0_u8; MAX_SIZE];
        let size_encoded = op.encode_in(&mut encoded).unwrap();
        let (ret, size_decoded) = ReadFileData::decode(&encoded).unwrap();
        assert_eq!(size_encoded, size_decoded);
        assert_eq!(ret, op);

        // Test decode(data).encode_in() == data
        let mut encoded2 = [0_u8; MAX_SIZE];
        let size_encoded2 = op.encode_in(&mut encoded2).unwrap();
        assert_eq!(size_encoded, size_encoded2);
        assert_eq!(encoded2[..size_encoded2], encoded[..size_encoded]);
    }
}

use super::super::define::flag;
use super::super::define::op_code::OpCode;
use super::super::error::BasicDecodeError;
use crate::define::FileId;

/// Maximum byte size of an encoded ReadFileProperties
pub const MAX_SIZE: usize = 2;

/// This action has a fixed size
pub const SIZE: usize = 2;

/// Reads the properties of a file
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct ReadFileProperties {
    /// Group with next action
    pub group: bool,
    /// Ask for a response (status)
    pub response: bool,
    /// File ID of the file to read
    pub file_id: FileId,
}

impl ReadFileProperties {
    /// Most common builder ReadFileProperties builder.
    ///
    /// group = false
    /// response = true
    pub fn new(file_id: FileId) -> Self {
        Self {
            group: false,
            response: true,
            file_id,
        }
    }

    /// Encodes the Item into a fixed size array
    pub const fn encode_to_array(&self) -> [u8; 2] {
        [
            OpCode::ReadFileProperties as u8
                + if self.group { flag::GROUP } else { 0 }
                + if self.response { flag::RESPONSE } else { 0 },
            self.file_id.u8(),
        ]
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
    /// You are responsible for checking that `out.len() >= size`. Failing that
    /// will result in the program writing out of bound. In the current
    /// implementation, it will silently attempt to write out of bounds.
    pub unsafe fn encode_in_ptr(&self, out: *mut u8) -> usize {
        *out.add(0) = OpCode::ReadFileProperties as u8
            + if self.group { flag::GROUP } else { 0 }
            + if self.response { flag::RESPONSE } else { 0 };
        *out.add(1) = self.file_id.u8();
        2
    }

    /// Encodes the Item without checking the size of the receiving
    /// byte array.
    ///
    /// # Safety
    /// You are responsible for checking that `size` == [self.size()](#method.size) and
    /// to insure `out.len() >= size`. Failing that will result in the
    /// program writing out of bound. In the current implementation, it
    /// implementation, it will silently attempt to write out of bounds.
    pub unsafe fn encode_in_unchecked(&self, out: &mut [u8]) -> usize {
        self.encode_in_ptr(out.as_mut_ptr())
    }

    /// Encodes the value into pre allocated array.
    ///
    /// Fails if the pre allocated array is smaller than [self.size()](#method.size)
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
        2
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
    /// You are to check that data is not empty and that data.len() >=
    /// [DecodableVarint.size()](struct.DecodableVarint.html#method.size)
    /// (the expected byte size of the returned DecodableItem).
    ///
    /// You are also expected to warrant that the opcode contained in the
    /// first byte corresponds to this action.
    pub const unsafe fn start_decoding_ptr<'data>(
        data: *const u8,
    ) -> DecodableReadFileProperties<'data> {
        DecodableReadFileProperties::from_ptr(data)
    }

    /// Creates a decodable item without checking the data size.
    ///
    /// # Safety
    /// You are to check that data is not empty and that data.len() >=
    /// [DecodableReadFileProperties.size()](struct.DecodableReadFileProperties.html#method.size)
    /// (the expected byte size of the returned DecodableItem).
    ///
    /// You are also expected to warrant that the opcode contained in the
    /// first byte corresponds to this action.
    pub const unsafe fn start_decoding_unchecked(data: &[u8]) -> DecodableReadFileProperties {
        DecodableReadFileProperties::new(data)
    }

    /// Creates a decodable item.
    ///
    /// This decodable item allows each parts of the item independently.
    pub const fn start_decoding(
        data: &[u8],
    ) -> Result<DecodableReadFileProperties, BasicDecodeError> {
        if data.len() < 2 {
            return Err(BasicDecodeError::MissingBytes(2));
        }
        if data[0] & 0x3F != OpCode::ReadFileProperties as u8 {
            return Err(BasicDecodeError::BadOpCode);
        }
        let ret = unsafe { Self::start_decoding_unchecked(data) };
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
    /// You are to check that data is not empty and that data.len() >=
    /// [DecodableVarint.size()](struct.DecodableVarint.html#method.size)
    /// (the expected byte size of the returned DecodableItem).
    ///
    /// Failing that will result in reading and interpreting data outside the given
    /// array.
    ///
    /// You are also expected to warrant that the opcode contained in the
    /// first byte corresponds to this action.
    pub unsafe fn decode_ptr(data: *const u8) -> (Self, usize) {
        Self::start_decoding_ptr(data).complete_decoding()
    }

    /// Decodes the Item from bytes.
    ///
    /// Returns the decoded data and the number of bytes consumed to produce it.
    ///
    /// # Safety
    /// You are to check that data is not empty and that data.len() >=
    /// [DecodableReadFileProperties.size()](struct.DecodableReadFileProperties.html#method.size)
    /// (the expected byte size of the returned DecodableItem).
    ///
    /// You are also expected to warrant that the opcode contained in the
    /// first byte corresponds to this action.
    pub unsafe fn decode_unchecked(data: &[u8]) -> (Self, usize) {
        Self::start_decoding_unchecked(data).complete_decoding()
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

pub struct DecodableReadFileProperties<'data> {
    data: *const u8,
    data_life: core::marker::PhantomData<&'data ()>,
}

impl<'data> DecodableReadFileProperties<'data> {
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
    pub const fn size(&self) -> usize {
        2
    }

    pub fn group(&self) -> bool {
        unsafe { *self.data.add(0) & flag::GROUP != 0 }
    }

    pub fn response(&self) -> bool {
        unsafe { *self.data.add(0) & flag::RESPONSE != 0 }
    }

    pub fn file_id(&self) -> FileId {
        unsafe { FileId::new(*self.data.add(1)) }
    }

    /// Fully decode the Item
    ///
    /// Returns the decoded data and the number of bytes consumed to produce it.
    pub fn complete_decoding(&self) -> (ReadFileProperties, usize) {
        (
            ReadFileProperties {
                group: self.group(),
                response: self.response(),
                file_id: self.file_id(),
            },
            2,
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn known() {
        fn test(op: ReadFileProperties, data: &[u8]) {
            // Test op.encode_in() == data
            let mut encoded = [0u8; 2];
            let size = op.encode_in(&mut encoded).unwrap();
            assert_eq!(size, data.len());
            assert_eq!(&encoded, data);

            // Test op.encode_to_array() == data
            assert_eq!(&op.encode_to_array(), data);

            // Test decode(data) == op
            let (ret, size) = ReadFileProperties::decode(&data).unwrap();
            assert_eq!(size, data.len());
            assert_eq!(ret, op);

            // Test partial_decode == op
            let decoder = ReadFileProperties::start_decoding(&data).unwrap();
            assert_eq!(size, decoder.size());
            assert_eq!(
                op,
                ReadFileProperties {
                    group: decoder.group(),
                    response: decoder.response(),
                    file_id: decoder.file_id(),
                }
            );
        }
        test(
            ReadFileProperties {
                group: false,
                response: true,
                file_id: FileId::new(0),
            },
            &[0x42, 0x00],
        );
        test(
            ReadFileProperties {
                group: true,
                response: false,
                file_id: FileId::new(1),
            },
            &[0x82, 0x01],
        );
        test(
            ReadFileProperties {
                group: true,
                response: true,
                file_id: FileId::new(2),
            },
            &[0xC2, 0x02],
        );
        test(
            ReadFileProperties {
                group: false,
                response: false,
                file_id: FileId::new(0xFF),
            },
            &[0x02, 0xFF],
        );
    }

    #[test]
    fn consistence() {
        let op = ReadFileProperties {
            group: true,
            response: false,
            file_id: FileId::new(42),
        };

        // Test decode(op.encode_to_array()) == op
        let data = op.encode_to_array();
        let (ret, size) = ReadFileProperties::decode(&op.encode_to_array()).unwrap();
        assert_eq!(size, data.len());
        assert_eq!(ret, op);

        // Test decode(data).encode_to_array() == data
        let (ret, size) = ReadFileProperties::decode(&data).unwrap();
        assert_eq!(size, data.len());
        assert_eq!(ret.encode_to_array(), data);
    }
}

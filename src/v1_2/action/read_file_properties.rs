use crate::decodable::{Decodable, EncodedData, SizeError, WithByteSize};
use crate::define::FileId;
use crate::encodable::Encodable;
use crate::v1_2::define::{flag, op_code};

/// Maximum byte size of an encoded `ReadFileProperties`
pub const MAX_SIZE: usize = 2;

/// This action has a fixed size
pub const SIZE: usize = 2;

/// Reads the properties of a file
#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct ReadFilePropertiesRef<'item> {
    /// Group with next action
    pub group: bool,
    /// Ask for a response (status)
    pub response: bool,
    /// File ID of the file to read
    pub file_id: FileId,
    /// Empty data required for lifetime compilation.
    phantom: core::marker::PhantomData<&'item ()>,
}

impl<'item> ReadFilePropertiesRef<'item> {
    pub const fn new(group: bool, response: bool, file_id: FileId) -> Self {
        Self {
            group,
            response,
            file_id,
            phantom: core::marker::PhantomData,
        }
    }

    /// Encodes the Item into a fixed size array
    pub const fn encode_to_array(&self) -> [u8; 2] {
        [
            op_code::READ_FILE_PROPERTIES
                | if self.group { flag::GROUP } else { 0 }
                | if self.response { flag::RESPONSE } else { 0 },
            self.file_id.u8(),
        ]
    }

    pub fn to_owned(&self) -> ReadFileProperties {
        ReadFileProperties {
            group: self.group,
            response: self.response,
            file_id: self.file_id,
        }
    }
}

impl<'data> Encodable for ReadFilePropertiesRef<'data> {
    unsafe fn encode_in_ptr(&self, out: *mut u8) -> usize {
        *out.add(0) = op_code::READ_FILE_PROPERTIES
            | if self.group { flag::GROUP } else { 0 }
            | if self.response { flag::RESPONSE } else { 0 };
        *out.add(1) = self.file_id.u8();
        2
    }

    /// Size in bytes of the encoded equivalent of the item.
    fn size(&self) -> usize {
        SIZE
    }
}

pub struct EncodedReadFileProperties<'data> {
    data: &'data [u8],
}

impl<'data> EncodedReadFileProperties<'data> {
    pub fn group(&self) -> bool {
        unsafe { *self.data.get_unchecked(0) & flag::GROUP != 0 }
    }

    pub fn response(&self) -> bool {
        unsafe { *self.data.get_unchecked(0) & flag::RESPONSE != 0 }
    }

    pub fn file_id(&self) -> FileId {
        unsafe { FileId::new(*self.data.get_unchecked(1)) }
    }

    pub fn size_unchecked(&self) -> usize {
        SIZE
    }
}

impl<'data> EncodedData<'data> for EncodedReadFileProperties<'data> {
    type DecodedData = ReadFilePropertiesRef<'data>;

    unsafe fn new(data: &'data [u8]) -> Self {
        Self { data }
    }

    fn size(&self) -> Result<usize, SizeError> {
        Ok(SIZE)
    }

    fn complete_decoding(&self) -> WithByteSize<ReadFilePropertiesRef<'data>> {
        WithByteSize {
            item: ReadFilePropertiesRef {
                group: self.group(),
                response: self.response(),
                file_id: self.file_id(),
                phantom: core::marker::PhantomData,
            },
            byte_size: 2,
        }
    }
}

impl<'data> Decodable<'data> for ReadFilePropertiesRef<'data> {
    type Data = EncodedReadFileProperties<'data>;
}

/// Reads the properties of a file
#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
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
    pub fn as_ref(&self) -> ReadFilePropertiesRef {
        ReadFilePropertiesRef {
            group: self.group,
            response: self.response,
            file_id: self.file_id,
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
        fn test(op: ReadFilePropertiesRef, data: &[u8]) {
            // Test op.encode_in() == data
            let mut encoded = [0_u8; 2];
            let size = op.encode_in(&mut encoded).unwrap();
            assert_eq!(size, data.len());
            assert_eq!(&encoded, data);

            // Test op.encode_to_array() == data
            assert_eq!(&op.encode_to_array(), data);

            // Test decode(data) == op
            let WithByteSize {
                item: ret,
                byte_size: size,
            } = ReadFilePropertiesRef::decode(data).unwrap();
            assert_eq!(size, data.len());
            assert_eq!(ret, op);

            // Test partial_decode == op
            let decoder = ReadFilePropertiesRef::start_decoding(data).unwrap().item;
            assert_eq!(size, decoder.size_unchecked());
            assert_eq!(size, decoder.size().unwrap());
            assert_eq!(
                op,
                ReadFilePropertiesRef {
                    group: decoder.group(),
                    response: decoder.response(),
                    file_id: decoder.file_id(),
                    phantom: core::marker::PhantomData,
                }
            );
        }
        test(
            ReadFilePropertiesRef {
                group: false,
                response: true,
                file_id: FileId::new(0),
                phantom: core::marker::PhantomData,
            },
            &[0x42, 0x00],
        );
        test(
            ReadFilePropertiesRef {
                group: true,
                response: false,
                file_id: FileId::new(1),
                phantom: core::marker::PhantomData,
            },
            &[0x82, 0x01],
        );
        test(
            ReadFilePropertiesRef {
                group: true,
                response: true,
                file_id: FileId::new(2),
                phantom: core::marker::PhantomData,
            },
            &[0xC2, 0x02],
        );
        test(
            ReadFilePropertiesRef {
                group: false,
                response: false,
                file_id: FileId::new(0xFF),
                phantom: core::marker::PhantomData,
            },
            &[0x02, 0xFF],
        );
    }

    #[test]
    fn consistence() {
        let op = ReadFilePropertiesRef {
            group: true,
            response: false,
            file_id: FileId::new(42),
            phantom: core::marker::PhantomData,
        };

        // Test decode(op.encode_to_array()) == op
        let data = op.encode_to_array();
        let WithByteSize {
            item: ret,
            byte_size: size,
        } = ReadFilePropertiesRef::decode(&data).unwrap();
        assert_eq!(size, data.len());
        assert_eq!(ret, op);

        // Test decode(data).encode_to_array() == data
        let WithByteSize {
            item: ret,
            byte_size: size,
        } = ReadFilePropertiesRef::decode(&data).unwrap();
        assert_eq!(size, data.len());
        assert_eq!(ret.encode_to_array(), data);
    }
}

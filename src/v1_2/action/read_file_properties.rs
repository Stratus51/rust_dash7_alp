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
pub struct ReadFilePropertiesRef<'item, 'data> {
    /// Group with next action
    pub group: bool,
    /// Ask for a response (status)
    pub response: bool,
    /// File ID of the file to read
    pub file_id: FileId,
    /// Empty data required for lifetime compilation.
    item_phantom: core::marker::PhantomData<&'item ()>,
    data_phantom: core::marker::PhantomData<&'data ()>,
}

impl<'item, 'data> ReadFilePropertiesRef<'item, 'data> {
    pub const fn new(group: bool, response: bool, file_id: FileId) -> Self {
        Self {
            group,
            response,
            file_id,
            item_phantom: core::marker::PhantomData,
            data_phantom: core::marker::PhantomData,
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

impl<'item, 'data> Encodable for ReadFilePropertiesRef<'item, 'data> {
    unsafe fn encode_in_ptr(&self, out: *mut u8) -> usize {
        *out.add(0) = op_code::READ_FILE_PROPERTIES
            | if self.group { flag::GROUP } else { 0 }
            | if self.response { flag::RESPONSE } else { 0 };
        *out.add(1) = self.file_id.u8();
        2
    }

    /// Size in bytes of the encoded equivalent of the item.
    fn encoded_size(&self) -> usize {
        SIZE
    }
}

pub struct EncodedReadFileProperties<'item, 'data> {
    data: &'item &'data [u8],
}

impl<'item, 'data> EncodedReadFileProperties<'item, 'data> {
    pub fn group(&self) -> bool {
        unsafe { *self.data.get_unchecked(0) & flag::GROUP != 0 }
    }

    pub fn response(&self) -> bool {
        unsafe { *self.data.get_unchecked(0) & flag::RESPONSE != 0 }
    }

    pub fn file_id(&self) -> FileId {
        unsafe { FileId::new(*self.data.get_unchecked(1)) }
    }

    pub fn encoded_size_unchecked(&self) -> usize {
        SIZE
    }
}

impl<'item, 'data> EncodedData<'item, 'data> for EncodedReadFileProperties<'item, 'data> {
    type SourceData = &'data [u8];
    type DecodedData = ReadFilePropertiesRef<'item, 'data>;

    unsafe fn new(data: Self::SourceData) -> Self {
        Self { data }
    }

    fn encoded_size(&self) -> Result<usize, SizeError> {
        Ok(SIZE)
    }

    fn complete_decoding(&self) -> WithByteSize<Self::DecodedData> {
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

pub struct EncodedReadFilePropertiesMut<'item, 'data> {
    data: &'item mut &'data mut [u8],
}

impl<'item, 'data> EncodedReadFilePropertiesMut<'item, 'data> {
    pub fn as_ref<'result>(&'data self) -> EncodedReadFileProperties<'result, 'data> {
        unsafe { EncodedReadFileProperties::new(self.data) }
    }

    pub fn group(&self) -> bool {
        self.as_ref().group()
    }

    pub fn response(&self) -> bool {
        self.as_ref().response()
    }

    pub fn encoded_size_unchecked(&self) -> usize {
        self.as_ref().encoded_size_unchecked()
    }

    pub fn set_group(&mut self, group: bool) {
        if group {
            unsafe { *self.data.get_unchecked_mut(0) |= flag::GROUP }
        } else {
            unsafe { *self.data.get_unchecked_mut(0) &= !flag::GROUP }
        }
    }

    pub fn set_response(&mut self, response: bool) {
        if response {
            unsafe { *self.data.get_unchecked_mut(0) |= flag::RESPONSE }
        } else {
            unsafe { *self.data.get_unchecked_mut(0) &= !flag::RESPONSE }
        }
    }
}

impl<'item, 'data, 'result> EncodedData<'data, 'result>
    for EncodedReadFilePropertiesMut<'item, 'data>
{
    type SourceData = &'data mut [u8];
    type DecodedData = ReadFilePropertiesRef<'result, 'data>;

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

impl<'item, 'data, 'result> Decodable<'data, 'result> for ReadFilePropertiesRef<'item, 'data> {
    type Data = EncodedReadFileProperties<'item, 'data>;
    type DataMut = EncodedReadFilePropertiesMut<'item, 'data>;
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
            assert_eq!(size, decoder.encoded_size_unchecked());
            assert_eq!(size, decoder.encoded_size().unwrap());
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

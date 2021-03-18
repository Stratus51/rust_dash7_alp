use crate::decodable::{Decodable, EncodedData, SizeError, WithByteSize};
use crate::encodable::Encodable;
use crate::v1_2::define::{flag, op_code};

/// Maximum byte size of an encoded Nop
pub const MAX_SIZE: usize = 1;

/// This action has a fixed size
pub const SIZE: usize = 1;

/// Does nothing.
#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct NopRef<'data> {
    /// Group with next action
    pub group: bool,
    /// Ask for a response (status)
    pub response: bool,
    /// Empty data required for lifetime compilation.
    phantom: core::marker::PhantomData<&'data ()>,
}

impl<'data> NopRef<'data> {
    pub const fn new(group: bool, response: bool) -> Self {
        Self {
            group,
            response,
            phantom: core::marker::PhantomData,
        }
    }

    /// Encodes the Item into a fixed size array
    pub const fn encode_to_array(&self) -> [u8; 1] {
        [op_code::NOP
            | if self.group { flag::GROUP } else { 0 }
            | if self.response { flag::RESPONSE } else { 0 }]
    }

    pub fn to_owned(&self) -> Nop {
        Nop {
            group: self.group,
            response: self.response,
        }
    }
}

impl<'data> Encodable for NopRef<'data> {
    unsafe fn encode_in_ptr(&self, out: *mut u8) -> usize {
        *out.add(0) = op_code::NOP
            | if self.group { flag::GROUP } else { 0 }
            | if self.response { flag::RESPONSE } else { 0 };
        1
    }

    fn encoded_size(&self) -> usize {
        SIZE
    }
}

pub struct EncodedNop<'data> {
    data: &'data [u8],
}

impl<'data> EncodedNop<'data> {
    pub fn group(&self) -> bool {
        unsafe { *self.data.get_unchecked(0) & flag::GROUP != 0 }
    }

    pub fn response(&self) -> bool {
        unsafe { *self.data.get_unchecked(0) & flag::RESPONSE != 0 }
    }

    pub const fn encoded_size_unchecked(&self) -> usize {
        SIZE
    }
}

impl<'data, 'result> EncodedData<'data> for EncodedNop<'data> {
    type SourceData = &'data [u8];
    type DecodedData = NopRef<'data>;

    unsafe fn new(data: Self::SourceData) -> Self {
        Self { data }
    }

    fn encoded_size(&self) -> Result<usize, SizeError> {
        Ok(SIZE)
    }

    fn complete_decoding(&self) -> WithByteSize<Self::DecodedData> {
        WithByteSize {
            item: NopRef {
                group: self.group(),
                response: self.response(),
                phantom: core::marker::PhantomData,
            },
            byte_size: 1,
        }
    }
}

pub struct EncodedNopMut<'data> {
    data: &'data mut [u8],
}

crate::make_downcastable!(EncodedNopMut, EncodedNop);

impl<'data> EncodedNopMut<'data> {
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

impl<'data> EncodedData<'data> for EncodedNopMut<'data> {
    type SourceData = &'data mut [u8];
    type DecodedData = NopRef<'data>;

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

impl<'data> Decodable<'data> for NopRef<'data> {
    type Data = EncodedNop<'data>;
    type DataMut = EncodedNopMut<'data>;
}

/// Does nothing.
#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Nop {
    /// Group with next action
    pub group: bool,
    /// Ask for a response (status)
    pub response: bool,
}

impl Nop {
    pub fn as_ref(&self) -> NopRef {
        NopRef {
            group: self.group,
            response: self.response,
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
        fn test(op: NopRef, data: &[u8]) {
            // Test op.encode_in() == data
            let mut encoded = [0_u8; 1];
            let size = op.encode_in(&mut encoded).unwrap();
            assert_eq!(size, data.len());
            assert_eq!(&encoded, data);

            // Test op.encode_to_array() == data
            assert_eq!(&op.encode_to_array(), data);

            // Test decode(data) == op
            let WithByteSize {
                item: ret,
                byte_size,
            } = NopRef::decode(data).unwrap();
            assert_eq!(byte_size, data.len());
            assert_eq!(ret, op);

            // Test partial_decode == op
            let decoder = NopRef::start_decoding(data).unwrap().item;
            assert_eq!(decoder.encoded_size_unchecked(), size);
            assert_eq!(decoder.encoded_size().unwrap(), size);
            assert_eq!(
                op,
                NopRef {
                    group: decoder.group(),
                    response: decoder.response(),
                    phantom: core::marker::PhantomData,
                }
            );
        }
        test(
            NopRef {
                group: false,
                response: true,
                phantom: core::marker::PhantomData,
            },
            &[0x40],
        );
        test(
            NopRef {
                group: true,
                response: false,
                phantom: core::marker::PhantomData,
            },
            &[0x80],
        );
        test(
            NopRef {
                group: true,
                response: true,
                phantom: core::marker::PhantomData,
            },
            &[0xC0],
        );
        test(
            NopRef {
                group: false,
                response: false,
                phantom: core::marker::PhantomData,
            },
            &[0x00],
        );
    }

    #[test]
    fn consistence() {
        let op = NopRef {
            group: true,
            response: false,
            phantom: core::marker::PhantomData,
        };

        // Test decode(op.encode_to_array()) == op
        let data = op.encode_to_array();
        let WithByteSize {
            item: ret,
            byte_size,
        } = NopRef::decode(&data).unwrap();
        assert_eq!(byte_size, data.len());
        assert_eq!(ret, op);

        // Test decode(data).encode_to_array() == data
        let WithByteSize {
            item: ret,
            byte_size,
        } = NopRef::decode(&data).unwrap();
        assert_eq!(byte_size, data.len());
        assert_eq!(ret.encode_to_array(), data);
    }
}

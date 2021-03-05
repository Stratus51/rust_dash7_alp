use super::super::define::flag;
use super::super::define::op_code::OpCode;
use crate::decodable::{Decodable, EncodedData, WithByteSize};

/// Maximum byte size of an encoded Nop
pub const MAX_SIZE: usize = 1;

/// This action has a fixed size
pub const SIZE: usize = 1;

/// Does nothing.
#[cfg_attr(feature = "repr_c", repr(C))]
#[cfg_attr(feature = "packed", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct NopRef<'item> {
    /// Group with next action
    pub group: bool,
    /// Ask for a response (status)
    pub response: bool,
    /// Empty data required for lifetime compilation.
    phantom: core::marker::PhantomData<&'item ()>,
}

impl<'item> Default for NopRef<'item> {
    /// Default Nop with group = false and response = true.
    ///
    /// Because that would be the most common use case: a ping command.
    fn default() -> Self {
        Self {
            group: false,
            response: true,
            phantom: core::marker::PhantomData,
        }
    }
}

impl<'item> NopRef<'item> {
    pub const fn new(group: bool, response: bool) -> Self {
        Self {
            group,
            response,
            phantom: core::marker::PhantomData,
        }
    }

    /// Encodes the Item into a fixed size array
    pub const fn encode_to_array(&self) -> [u8; 1] {
        [OpCode::Nop as u8
            | if self.group { flag::GROUP } else { 0 }
            | if self.response { flag::RESPONSE } else { 0 }]
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
        *out.add(0) = OpCode::Nop as u8
            | if self.group { flag::GROUP } else { 0 }
            | if self.response { flag::RESPONSE } else { 0 };
        1
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
        SIZE
    }

    pub fn to_owned(&self) -> Nop {
        Nop {
            group: self.group,
            response: self.response,
        }
    }
}

pub struct EncodedNop<'data> {
    data: *const u8,
    data_life: core::marker::PhantomData<&'data ()>,
}

impl<'data> EncodedNop<'data> {
    pub fn group(&self) -> bool {
        unsafe { *self.data.add(0) & flag::GROUP != 0 }
    }

    pub fn response(&self) -> bool {
        unsafe { *self.data.add(0) & flag::RESPONSE != 0 }
    }
}

impl<'data> EncodedData<'data> for EncodedNop<'data> {
    type DecodedData = NopRef<'data>;

    unsafe fn from_data_ref(data: &'data [u8]) -> Self {
        Self::from_data_ptr(data.as_ptr())
    }

    unsafe fn from_data_ptr(data: *const u8) -> Self {
        Self {
            data,
            data_life: core::marker::PhantomData,
        }
    }

    unsafe fn expected_size(&self) -> usize {
        SIZE
    }

    fn smaller_than(&self, data_size: usize) -> Result<usize, usize> {
        if data_size < SIZE {
            return Err(SIZE);
        }
        Ok(SIZE)
    }

    fn complete_decoding(&self) -> WithByteSize<NopRef<'data>> {
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

impl<'data> Decodable<'data> for NopRef<'data> {
    type Data = EncodedNop<'data>;
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
            assert_eq!(unsafe { decoder.expected_size() }, size);
            assert_eq!(decoder.smaller_than(data.len()).unwrap(), size);
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

use super::super::define::flag;
use super::super::define::op_code::OpCode;
use super::super::error::BasicDecodeError;

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
    /// - The data is bigger than `SIZE`.
    ///
    /// Failing that might result in reading and interpreting data outside the given
    /// array (depending on what is done with the resulting object).
    pub const unsafe fn start_decoding_ptr<'data>(data: *const u8) -> DecodableNop<'data> {
        DecodableNop::from_ptr(data)
    }

    /// Creates a decodable item without checking the data size.
    ///
    /// # Safety
    /// You are to check that:
    /// - The data is bigger than `SIZE`.
    ///
    /// Failing that might result in reading and interpreting data outside the given
    /// array (depending on what is done with the resulting object).
    pub const unsafe fn start_decoding_unchecked(data: &[u8]) -> DecodableNop {
        DecodableNop::new(data)
    }

    /// Returns a Decodable object.
    ///
    /// This decodable item allows each parts of the item to be decoded independently.
    ///
    /// # Errors
    /// - Fails if `data.len()` < `SIZE`.
    pub fn start_decoding(data: &[u8]) -> Result<DecodableNop, BasicDecodeError> {
        if data.is_empty() {
            return Err(BasicDecodeError::MissingBytes(SIZE));
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
    /// May attempt to read bytes after the end of the array.
    ///
    /// You are to check that:
    /// - The data is bigger than `SIZE`.
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
    /// - The data is bigger than `SIZE`.
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
    /// - Fails if `data.len()` < `SIZE`.
    pub fn decode(data: &[u8]) -> Result<(Self, usize), BasicDecodeError> {
        match Self::start_decoding(data) {
            Ok(v) => Ok(v.complete_decoding()),
            Err(e) => Err(e),
        }
    }

    pub fn to_owned(&self) -> Nop {
        Nop {
            group: self.group,
            response: self.response,
        }
    }
}

pub struct DecodableNop<'data> {
    data: *const u8,
    data_life: core::marker::PhantomData<&'data ()>,
}

impl<'data> DecodableNop<'data> {
    const fn new(data: &'data [u8]) -> Self {
        Self::from_ptr(data.as_ptr())
    }

    const fn from_ptr(data: *const u8) -> Self {
        Self {
            data,
            data_life: core::marker::PhantomData,
        }
    }

    /// Decodes the size of the Item in bytes
    pub const fn expected_size(&self) -> usize {
        SIZE
    }

    /// Checks whether the given data_size is bigger than the decoded object expected size.
    ///
    /// On success, returns the size of the decoded object.
    ///
    /// # Errors
    /// Fails if the data_size is smaller than the required data size to decode the object.
    pub const fn smaller_than(&self, data_size: usize) -> Result<usize, usize> {
        if data_size < SIZE {
            return Err(SIZE);
        }
        Ok(SIZE)
    }

    pub fn group(&self) -> bool {
        unsafe { *self.data.add(0) & flag::GROUP != 0 }
    }

    pub fn response(&self) -> bool {
        unsafe { *self.data.add(0) & flag::RESPONSE != 0 }
    }

    /// Fully decode the Item
    ///
    /// Returns the decoded data and the number of bytes consumed to produce it.
    pub fn complete_decoding<'item>(&self) -> (NopRef<'item>, usize) {
        (
            NopRef {
                group: self.group(),
                response: self.response(),
                phantom: core::marker::PhantomData,
            },
            1,
        )
    }
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
            let (ret, size) = NopRef::decode(data).unwrap();
            assert_eq!(size, data.len());
            assert_eq!(ret, op);

            // Test partial_decode == op
            let decoder = NopRef::start_decoding(data).unwrap();
            assert_eq!(decoder.expected_size(), size);
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
        let (ret, size) = NopRef::decode(&op.encode_to_array()).unwrap();
        assert_eq!(size, data.len());
        assert_eq!(ret, op);

        // Test decode(data).encode_to_array() == data
        let (ret, size) = NopRef::decode(&data).unwrap();
        assert_eq!(size, data.len());
        assert_eq!(ret.encode_to_array(), data);
    }
}

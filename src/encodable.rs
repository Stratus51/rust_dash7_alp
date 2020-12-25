pub trait Encodable {
    type Error;

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
    unsafe fn encode_in_ptr(&self, out: *mut u8) -> usize;

    /// Encodes the Item without checking the size of the receiving
    /// byte array.
    ///
    /// # Safety
    /// You are responsible for checking that `size` == [self.size()](#method.size) and
    /// to insure `out.len() >= size`. Failing that will result in the
    /// program writing out of bound. In the current implementation, it
    /// implementation, it will silently attempt to write out of bounds.
    unsafe fn encode_in_unchecked(&self, out: &mut [u8]) -> usize {
        self.encode_in_ptr(out.as_mut_ptr())
    }

    /// Encodes the value into pre allocated array.
    ///
    /// Fails if the pre allocated array is smaller than [self.size()](#method.size)
    /// returning the number of input bytes required.
    fn encode_in(&self, out: &mut [u8]) -> Result<usize, Self::Error>;

    /// Size in bytes of the encoded equivalent of the item.
    fn size(&self) -> usize;
}

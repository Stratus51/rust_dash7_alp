use crate::decodable::Decodable;
use crate::define::FileId;
use crate::varint::{EncodedVarint, EncodedVarintMut, Varint};

// TODO Make all encoded data raw data accessible
pub struct EncodedFileOffsetOperand<'data> {
    data: &'data [u8],
}

impl<'data> EncodedFileOffsetOperand<'data> {
    pub fn new(data: &'data [u8]) -> Self {
        Self { data }
    }

    pub fn file_id(&self) -> FileId {
        unsafe { FileId(*self.data.get_unchecked(0)) }
    }

    pub fn offset(&self) -> EncodedVarint<'data> {
        unsafe { Varint::start_decoding_unchecked(self.data.get_unchecked(1..)) }
    }
}

pub struct EncodedFileOffsetOperandMut<'data> {
    data: &'data mut [u8],
}

crate::make_downcastable!(EncodedFileOffsetOperandMut, EncodedFileOffsetOperand);

impl<'data> EncodedFileOffsetOperandMut<'data> {
    pub fn new(data: &'data mut [u8]) -> Self {
        Self { data }
    }

    pub fn file_id(&self) -> FileId {
        self.as_ref().file_id()
    }

    pub fn offset(&self) -> EncodedVarint<'data> {
        self.as_ref().offset()
    }

    pub fn set_file_id(&mut self, file_id: FileId) {
        unsafe {
            *self.data.get_unchecked_mut(0) = file_id.u8();
        }
    }

    pub fn offset_mut(&mut self) -> EncodedVarintMut {
        unsafe { Varint::start_decoding_unchecked_mut(self.data.get_unchecked_mut(1..)) }
    }
}

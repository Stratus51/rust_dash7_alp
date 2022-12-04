use crate::{
    codec::{Codec, StdError, WithOffset, WithSize},
    spec::v1_2::varint,
};

use super::OperandValidationError;

/// Read data from a file
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReadFileData {
    /// Group with next action
    pub group: bool,
    /// Ask for a response (read data via ReturnFileData)
    ///
    /// Generally true unless you just want to trigger a read on the filesystem
    pub resp: bool,
    pub file_id: u8,
    pub offset: u32,
    pub size: u32,
}
super::impl_display_simple_file_op!(ReadFileData, file_id, offset, size);
impl ReadFileData {
    pub fn validate(self) -> Result<(), OperandValidationError> {
        if self.offset > varint::MAX {
            return Err(OperandValidationError::OffsetTooBig);
        }
        if self.size > varint::MAX {
            return Err(OperandValidationError::SizeTooBig);
        }
        Ok(())
    }
}

impl Codec for ReadFileData {
    type Error = StdError;
    fn encoded_size(&self) -> usize {
        1 + 1 + super::unsafe_varint_serialize_sizes!(self.offset, self.size) as usize
    }
    unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
        out[0] |= ((self.group as u8) << 7) | ((self.resp as u8) << 6);
        out[1] = self.file_id;
        1 + 1 + super::unsafe_varint_serialize!(out[2..], self.offset, self.size)
    }
    fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
        let min_size = 1 + 1 + 1 + 1;
        if out.len() < min_size {
            return Err(WithOffset::new(
                0,
                Self::Error::MissingBytes(min_size - out.len()),
            ));
        }
        let group = out[0] & 0x80 != 0;
        let resp = out[0] & 0x40 != 0;
        let file_id = out[1];
        let mut off = 2;
        let WithSize {
            value: offset,
            size: offset_size,
        } = varint::decode(&out[off..]).map_err(|e| {
            e.shift(off);
            e
        })?;
        off += offset_size;
        let WithSize {
            value: size,
            size: size_size,
        } = varint::decode(&out[off..]).map_err(|e| {
            e.shift(off);
            e
        })?;
        off += size_size;
        Ok(WithSize {
            value: Self {
                group,
                resp,
                file_id,
                offset,
                size,
            },
            size: off,
        })
    }
}

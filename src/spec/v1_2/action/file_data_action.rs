use crate::{
    codec::{Codec, StdError, WithOffset, WithSize},
    spec::v1_2::varint,
};

/// Write data to a file
#[derive(Clone, Debug, PartialEq)]
pub struct FileDataAction {
    /// Group with next action
    pub group: bool,
    /// Ask for a response (a status)
    pub resp: bool,
    pub file_id: u8,
    pub offset: u32,
    pub data: Box<[u8]>,
}
super::impl_display_data_file_op!(FileDataAction);
impl FileDataAction {
    pub fn validate(&self) -> Result<(), super::OperandValidationError> {
        if self.offset > varint::MAX {
            return Err(super::OperandValidationError::OffsetTooBig);
        }
        let size = self.data.len() as u32;
        if size > varint::MAX {
            return Err(super::OperandValidationError::SizeTooBig);
        }
        Ok(())
    }
}
impl Codec for FileDataAction {
    type Error = StdError;
    fn encoded_size(&self) -> usize {
        1 + 1
            + super::unsafe_varint_serialize_sizes!(self.offset, self.data.len() as u32) as usize
            + self.data.len()
    }
    unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
        out[0] |= ((self.group as u8) << 7) | ((self.resp as u8) << 6);
        out[1] = self.file_id;
        let mut offset = 2;
        offset +=
            super::unsafe_varint_serialize!(out[2..], self.offset, self.data.len() as u32) as usize;
        out[offset..offset + self.data.len()].clone_from_slice(&self.data[..]);
        offset += self.data.len();
        offset
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
        } = varint::decode(&out[off..])?;
        off += offset_size;
        let WithSize {
            value: size,
            size: size_size,
        } = varint::decode(&out[off..])?;
        off += size_size;
        let size = size as usize;
        let mut data = vec![0u8; size].into_boxed_slice();
        data.clone_from_slice(&out[off..off + size]);
        off += size;
        Ok(WithSize {
            value: Self {
                group,
                resp,
                file_id,
                offset,
                data,
            },
            size: off,
        })
    }
}

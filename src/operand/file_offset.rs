#[cfg(test)]
use crate::test_tools::test_item;
use crate::{
    codec::{Codec, StdError, WithOffset, WithSize},
    varint,
};
#[cfg(test)]
use hex_literal::hex;

/// Describe the location of some data on the filesystem (file + data offset).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FileOffset {
    pub id: u8,
    pub offset: u32,
}
impl std::fmt::Display for FileOffset {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{},{}", self.id, self.offset)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum FileOffsetDecodingError {
    MissingBytes(usize),
    Offset(StdError),
}
impl Codec for FileOffset {
    type Error = FileOffsetDecodingError;
    fn encoded_size(&self) -> usize {
        1 + unsafe { varint::size(self.offset) } as usize
    }
    unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
        out[0] = self.id;
        1 + varint::encode_in(self.offset, &mut out[1..]) as usize
    }
    fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
        if out.len() < 2 {
            return Err(WithOffset::new_head(Self::Error::MissingBytes(
                2 - out.len(),
            )));
        }
        let WithSize {
            value: offset,
            size,
        } = varint::decode(&out[1..]).map_err(|e| {
            let WithOffset { offset, value } = e;
            WithOffset {
                offset: offset + 1,
                value: FileOffsetDecodingError::Offset(value),
            }
        })?;
        Ok(WithSize {
            value: Self { id: out[0], offset },
            size: 1 + size,
        })
    }
}
#[test]
fn test_file_offset_operand() {
    test_item(
        FileOffset {
            id: 2,
            offset: 0x3F_FF,
        },
        &hex!("02 7F FF"),
    )
}

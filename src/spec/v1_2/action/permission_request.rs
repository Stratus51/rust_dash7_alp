use crate::{
    codec::{Codec, WithOffset, WithSize},
    spec::v1_2::operand::{Permission, PermissionDecodingError},
};

/// Request a level of permission using some permission type
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PermissionRequest {
    /// Group with next action
    pub group: bool,
    /// Ask for a response (a status)
    pub resp: bool,
    /// See operand::permission_level
    pub level: u8,
    pub permission: Permission,
}
super::impl_display_simple_op!(PermissionRequest, level, permission);
impl Codec for PermissionRequest {
    type Error = PermissionDecodingError;
    fn encoded_size(&self) -> usize {
        1 + 1 + super::encoded_size!(self.permission)
    }
    unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
        out[0] |= ((self.group as u8) << 7) | ((self.resp as u8) << 6);
        out[1] = self.level;
        1 + super::serialize_all!(&mut out[2..], self.permission)
    }
    fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
        if out.is_empty() {
            Err(WithOffset::new_head(Self::Error::MissingBytes(1)))
        } else {
            let mut offset = 1;
            let level = out[offset];
            offset += 1;
            let WithSize {
                value: permission,
                size,
            } = Permission::decode(&out[offset..]).map_err(|e| e.shift(offset))?;
            offset += size;
            Ok(WithSize {
                value: Self {
                    group: out[0] & 0x80 != 0,
                    resp: out[0] & 0x40 != 0,
                    level,
                    permission,
                },
                size: offset,
            })
        }
    }
}

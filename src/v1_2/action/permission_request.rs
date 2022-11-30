#[cfg(test)]
use crate::test_tools::test_item;
#[cfg(test)]
use hex_literal::hex;

use crate::{
    codec::{Codec, WithOffset, WithSize},
    v1_2::{
        action::OpCode,
        operand::{Permission, PermissionDecodingError},
    },
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
        out[0] = super::control_byte!(self.group, self.resp, OpCode::PermissionRequest);
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
#[test]
fn test_permission_request() {
    test_item(
        PermissionRequest {
            group: false,
            resp: false,
            level: crate::v1_2::operand::permission_level::ROOT,
            permission: Permission::Dash7(hex!("0102030405060708")),
        },
        &hex!("0A   01 42 0102030405060708"),
    )
}

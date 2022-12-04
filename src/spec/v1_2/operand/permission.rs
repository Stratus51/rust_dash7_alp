use crate::codec::{Codec, WithOffset, WithSize};

// ALP SPEC: where is this defined? Link? Not found in either specs !
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Permission {
    Dash7([u8; 8]),
}

impl Permission {
    fn id(self) -> u8 {
        match self {
            Permission::Dash7(_) => 0x42, // ALP_SPEC Undefined
        }
    }
}
impl std::fmt::Display for Permission {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Dash7(data) => write!(f, "D7:0x{}", hex::encode_upper(data)),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PermissionDecodingError {
    MissingBytes(usize),
    UnknownId(u8),
}

impl Codec for Permission {
    type Error = PermissionDecodingError;
    fn encoded_size(&self) -> usize {
        1 + match self {
            Permission::Dash7(_) => 8,
        }
    }
    unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
        out[0] = self.id();
        1 + match self {
            Permission::Dash7(token) => {
                out[1..1 + token.len()].clone_from_slice(&token[..]);
                8
            }
        }
    }
    fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
        if out.is_empty() {
            return Err(WithOffset::new_head(Self::Error::MissingBytes(1)));
        }
        let mut offset = 1;
        match out[0] {
            0x42 => {
                let mut token = [0; 8];
                token.clone_from_slice(&out[offset..offset + 8]);
                offset += 8;
                Ok(WithSize {
                    value: Permission::Dash7(token),
                    size: offset,
                })
            }
            x => Err(WithOffset::new_head(Self::Error::UnknownId(x))),
        }
    }
}

pub mod permission_level {
    pub const USER: u8 = 0;
    pub const ROOT: u8 = 1;
    // ALP SPEC: Does something else exist?
}

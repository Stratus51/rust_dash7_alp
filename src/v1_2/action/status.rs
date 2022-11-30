#[cfg(test)]
use crate::test_tools::test_item;
#[cfg(test)]
use hex_literal::hex;

use crate::{
    codec::{Codec, StdError, WithOffset, WithSize},
    v1_2::{action::OpCode, operand},
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StatusType {
    Action = 0,
    Interface = 1,
}
impl StatusType {
    fn from(n: u8) -> Result<Self, u8> {
        Ok(match n {
            0 => StatusType::Action,
            1 => StatusType::Interface,
            x => return Err(x),
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Status {
    // ALP SPEC: This is named status, but it should be named action status compared to the '2'
    // other statuses.
    Action(operand::action_status::Status),
    Interface(operand::interface_status::InterfaceStatus),
    // ALP SPEC: Where are the stack errors?
}
impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Action(v) => write!(f, "[ACT]:{}", v),
            Self::Interface(v) => write!(f, "[ITF]:{}", v),
        }
    }
}
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum StatusDecodingError {
    MissingBytes(usize),
    UnknownType(u8),
    Action(StdError),
    Interface(operand::interface_status::InterfaceStatusDecodingError),
}
impl Codec for Status {
    type Error = StatusDecodingError;
    fn encoded_size(&self) -> usize {
        1 + match self {
            Status::Action(op) => op.encoded_size(),
            Status::Interface(op) => op.encoded_size(),
        }
    }
    unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
        out[0] = OpCode::Status as u8
            + ((match self {
                Status::Action(_) => StatusType::Action,
                Status::Interface(_) => StatusType::Interface,
            } as u8)
                << 6);
        let out = &mut out[1..];
        1 + match self {
            Status::Action(op) => op.encode_in(out),
            Status::Interface(op) => op.encode_in(out),
        }
    }
    fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
        if out.is_empty() {
            return Err(WithOffset::new_head(Self::Error::MissingBytes(1)));
        }
        let status_type = out[0] >> 6;
        Ok(
            match StatusType::from(status_type)
                .map_err(|e| WithOffset::new_head(Self::Error::UnknownType(e)))?
            {
                StatusType::Action => {
                    let WithSize { size, value } =
                        operand::action_status::Status::decode(&out[1..])
                            .map_err(|e| e.shift(1).map_value(Self::Error::Action))?;
                    WithSize {
                        size: size + 1,
                        value: Self::Action(value),
                    }
                }
                StatusType::Interface => {
                    let WithSize { size, value } =
                        operand::interface_status::InterfaceStatus::decode(&out[1..])
                            .map_err(|e| e.shift(1).map_value(Self::Error::Interface))?;
                    WithSize {
                        size: size + 1,
                        value: Self::Interface(value),
                    }
                }
            },
        )
    }
}
#[test]
fn test_status() {
    test_item(
        Status::Action(operand::action_status::Status {
            action_id: 2,
            status: operand::action_status::status::UNKNOWN_OPERATION,
        }),
        &hex!("22 02 F6"),
    )
}

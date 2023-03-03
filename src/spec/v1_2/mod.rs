#[cfg(test)]
use hex_literal::hex;

/// ALP basic Actions used to build Commands
pub mod action;
/// Dash7 specific items (most of the ALP protocol could be in theory be used over any
/// communication link)
pub mod dash7;
/// Filesystem related items
pub mod data;
/// Operands used to build the ALP Actions
pub mod operand;
/// ALP variable int codec implementation
pub mod varint;
pub use crate::codec::{Codec, WithOffset, WithSize};
pub use action::Action;

// TODO Verify each item's name against the SPEC

// TODO Look into const function to replace some macros?
// TODO usize is target dependent. In other words, on a 16 bit processor, we will run into
// troubles if we were to convert u32 to usize (even if a 64Ko payload seems a bit big).
// Maybe we should just embrace this limitation? (Not to be lazy or anything...)
// The bad thing is that u32 to u16 will compile and panic at runtime if the value is too big.
// TODO Slice copies still check length consistency dynamically. Is there a way to get rid of that
// at runtime while still testing it at compile/test time?
//      - For simple index access, get_unchecked_mut can do the trick. But It makes the code hard to
//      read...
// TODO Build a consistent validation API that encourages the user to check
// the validity of its structures

// ===============================================================================
// Command
// ===============================================================================
/// ALP request that can be sent to an ALP compatible device.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Command {
    // Does that impact application that don't use the structure?
    pub actions: Vec<Action>,
}

impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "[")?;
        let end = self.actions.len() - 1;
        for (i, action) in self.actions.iter().enumerate() {
            write!(f, "{}", action)?;
            if i != end {
                write!(f, "; ")?;
            }
        }
        write!(f, "]")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommandParseFail {
    pub actions: Vec<Action>,
    pub error: action::ActionDecodingError,
}

impl Command {
    pub fn encoded_size(&self) -> usize {
        self.actions.iter().map(|act| act.encoded_size()).sum()
    }
    /// Encode the item into a given byte array.
    /// # Safety
    /// You have to ensure there is enough space in the given array (compared to what
    /// [encoded_size](#encoded_size) returns) or this method will panic.
    /// # Panics
    /// Panics if the given `out` array is too small.
    pub unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
        let mut offset = 0;
        for action in self.actions.iter() {
            offset += action.encode_in(&mut out[offset..]);
        }
        offset
    }
    pub fn encode(&self) -> Box<[u8]> {
        let mut data = vec![0; self.encoded_size()].into_boxed_slice();
        unsafe { self.encode_in(&mut data) };
        data
    }
    pub fn decode(out: &[u8]) -> Result<Self, WithOffset<CommandParseFail>> {
        let mut actions = vec![];
        let mut offset = 0;
        loop {
            if offset == out.len() {
                break;
            }
            match Action::decode(&out[offset..]) {
                Ok(WithSize { value, size }) => {
                    actions.push(value);
                    offset += size;
                }
                Err(error) => {
                    let WithOffset { offset: off, value } = error;
                    return Err(WithOffset {
                        offset: offset + off,
                        value: CommandParseFail {
                            actions,
                            error: value,
                        },
                    });
                }
            }
        }
        Ok(Self { actions })
    }

    pub fn request_id(&self) -> Option<u8> {
        for action in self.actions.iter() {
            if let Action::RequestTag(action::RequestTag { id, .. }) = action {
                return Some(*id);
            }
        }
        None
    }

    pub fn response_id(&self) -> Option<u8> {
        for action in self.actions.iter() {
            if let Action::ResponseTag(action::ResponseTag { id, .. }) = action {
                return Some(*id);
            }
        }
        None
    }
}
#[test]
fn test_command() {
    let cmd = Command {
        actions: vec![
            Action::RequestTag(action::RequestTag { id: 66, eop: true }),
            Action::ReadFileData(action::ReadFileData {
                resp: true,
                group: false,
                file_id: 0,
                offset: 0,
                size: 8,
            }),
            Action::ReadFileData(action::ReadFileData {
                resp: false,
                group: true,
                file_id: 4,
                offset: 2,
                size: 3,
            }),
            Action::Nop(action::Nop {
                resp: true,
                group: true,
            }),
        ],
    };
    let data = &hex!("B4 42   41 00 00 08   81 04 02 03  C0") as &[u8];

    assert_eq!(&cmd.encode()[..], data);
    assert_eq!(
        Command::decode(data).expect("should be parsed without error"),
        cmd,
    );
}
#[test]
fn test_command_display() {
    assert_eq!(
        Command {
            actions: vec![
                Action::RequestTag(action::RequestTag { id: 66, eop: true }),
                Action::Nop(action::Nop {
                    resp: true,
                    group: true,
                }),
            ]
        }
        .to_string(),
        "[RTAG[E](66); NOP[GR]]"
    );
}

#[test]
fn test_command_request_id() {
    assert_eq!(
        Command {
            actions: vec![Action::request_tag(true, 66), Action::nop(true, true)]
        }
        .request_id(),
        Some(66)
    );
    assert_eq!(
        Command {
            actions: vec![Action::nop(true, false), Action::request_tag(true, 44)]
        }
        .request_id(),
        Some(44)
    );
    assert_eq!(
        Command {
            actions: vec![Action::nop(true, false), Action::nop(true, false)]
        }
        .request_id(),
        None
    );
}

#[test]
fn test_comman_response_id() {
    assert_eq!(
        Command {
            actions: vec![
                Action::response_tag(true, true, 66),
                Action::nop(true, true)
            ]
        }
        .response_id(),
        Some(66)
    );
    assert_eq!(
        Command {
            actions: vec![
                Action::nop(true, false),
                Action::response_tag(true, true, 44)
            ]
        }
        .response_id(),
        Some(44)
    );
    assert_eq!(
        Command {
            actions: vec![Action::nop(true, false), Action::nop(true, false)]
        }
        .response_id(),
        None
    );
}

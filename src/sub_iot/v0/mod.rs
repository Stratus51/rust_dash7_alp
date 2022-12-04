#[cfg(test)]
use hex_literal::hex;

/// ALP basic Actions used to build Commands
pub mod action;
/// Dash7 specific items (most of the ALP protocol could be in theory be used over any
/// communication link)
pub mod dash7;
pub mod operand;
/// ALP variable int codec implementation
pub use crate::codec::{Codec, WithOffset, WithSize};
pub use action::Action;

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
        "[RTG[E](66); NOP[GR]]"
    );
}

//! Implementation of a [Dash7](https://dash7-alliance.org/) ALP protocol parser from its
//! public specification.
//!
//! The protocol
//! ==============================================================================
//! The protocol specifies ALP Commands that can be sent to another system to communicate.
//! Each command is an aggregation of ALP Actions.
//!
//! The protocol is based on the fact that each communicating party hold a Dash7 filesystem.
//! Each request toward an other device is then composed as an array of simple filesystem operation
//! (ALP actions).
//!
//! About this library
//! ==============================================================================
//! The goal of this library is to implement a specification with an emphasis on correctness, then
//! on usability. Performance and memory usage are currently considered a secondary objective.
//!
//! Quickstart
//! ==============================================================================
//!
//! ```
//! use hex_literal::hex;
//! use dash7_alp::{Action, Command, Codec, action};
//!
//! fn main() {
//!     let cmd = Command {
//!         actions: vec![
//!             Action::RequestTag(action::RequestTag { id: 66, eop: true }),
//!             Action::ReadFileData(action::ReadFileData {
//!                 resp: true,
//!                 group: false,
//!                 file_id: 0,
//!                 offset: 0,
//!                 size: 8,
//!             }),
//!             Action::ReadFileData(action::ReadFileData {
//!                 resp: false,
//!                 group: true,
//!                 file_id: 4,
//!                 offset: 2,
//!                 size: 3,
//!             }),
//!             Action::Nop(action::Nop {
//!                 resp: true,
//!                 group: true,
//!             }),
//!         ],
//!     };
//!     let data = &hex!("B4 42   41 00 00 08   81 04 02 03  C0") as &[u8];
//!
//!     assert_eq!(&cmd.encode()[..], data);
//!     let parsed_cmd = Command::decode(data).expect("should be parsed without error");
//!     assert_eq!(parsed_cmd, cmd);
//! }
//! ```
//!
//! Notes
//! ==============================================================================
//! Group
//! ------------------------------------------------------------------------------
//! Many ALP action have a group flag. This allows those to be grouped.
//!
//! This means that:
//! - If any action of this group fails, the next actions are skipped.
//! - A query before the group will apply to the whole group (to defined
//! whether it will be executed).
//! - If the group contains queries, a prior Logical action will determine how they
//! are composed between them (OR, XOR, NOR, NAND). Without any Logical action, the
//! queries are AND'ed.
//!
//! Codec trait
//! ------------------------------------------------------------------------------
//! This trait implements the encode/decode methods. You very probably want to import
//! it into scope.

#[cfg(test)]
mod test_tools;
#[cfg(test)]
use hex_literal::hex;

/// ALP basic Actions used to build Commands
pub mod action;
/// A Codec module specifying how to encode/decode each encodable items
pub mod codec;
/// Dash7 specific items (most of the ALP protocol could be in theory be used over any
/// communication link)
pub mod dash7;
/// Filesystem related items
pub mod data;
/// Module managing the creation of protected items
pub mod new;
/// Operands used to build the ALP Actions
pub mod operand;
/// ALP variable int codec implementation
pub mod varint;
pub use action::Action;
pub use codec::{Codec, WithOffset, WithSize};

// TODO Verify each item's name against the SPEC

// TODO Look into const function to replace some macros?
// TODO Use uninitialized memory where possible
// TODO Int enums: fn from(): find a way to avoid double value definition
// TODO Int enums: optim: find a way to cast from int to enum instead of calling a matching
// function (much more resource intensive). Only do that for enums that match all possible
// values that result from the parsing.
// TODO Optimize min size calculation (fold it into the upper OP when possible)
// TODO usize is target dependent. In other words, on a 16 bit processor, we will run into
// troubles if we were to convert u32 to usize (even if a 64Ko payload seems a bit big).
// Maybe we should just embrace this limitation? (Not to be lazy or anything...)
// The bad thing is that u32 to u16 will compile and panic at runtime if the value is too big.
// TODO Slice copies still check length consistency dynamically. Is there a way to get rid of that
// at runtime while still testing it at compile/test time?
//      - For simple index access, get_unchecked_mut can do the trick. But It makes the code hard to
//      read...
// TODO is {out = &out[offset..]; out[..size]} more efficient than {out[offset..offset+size]} ?
// TODO Add function to encode without having to define a temporary structure
// TODO Build a consistent validation API that encourages the user to check
// the validity of its structures

// ===============================================================================
// Command
// ===============================================================================
/// ALP request that can be sent to an ALP compatible device.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Command {
    // TODO This Vec makes us ::collection dependent.
    // Does that impact application that don't use the structure?
    pub actions: Vec<Action>,
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

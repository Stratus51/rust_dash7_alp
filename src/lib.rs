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

#[cfg(test)]
use test_tools::test_item;

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
pub use codec::Codec;

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

// ===============================================================================
// Definitions
// ===============================================================================
#[derive(Clone, Debug, PartialEq)]
pub enum Enum {
    OpCode,
    NlsMethod,
    RetryMode,
    RespMode,
    InterfaceId,
    PermissionId,
    PermissionLevel,
    QueryComparisonType,
    QueryRangeComparisonType,
    QueryCode,
    StatusType,
    ActionCondition,
}

// ===============================================================================
// Command
// ===============================================================================
/// ALP request that can be sent to an ALP compatible device.
#[derive(Clone, Debug, PartialEq)]
pub struct Command {
    // TODO This Vec makes us ::collection dependent.
    // Does that impact application that don't use the structure?
    pub actions: Vec<Action>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct CommandParseFail {
    pub actions: Vec<Action>,
    pub error: codec::ParseFail,
}

impl Default for Command {
    fn default() -> Self {
        Self { actions: vec![] }
    }
}
impl Command {
    fn partial_decode(out: &[u8]) -> Result<codec::ParseValue<Command>, CommandParseFail> {
        let mut actions = vec![];
        let mut offset = 0;
        loop {
            if offset == out.len() {
                break;
            }
            match Action::decode(&out[offset..]) {
                Ok(codec::ParseValue { value, size }) => {
                    actions.push(value);
                    offset += size;
                }
                Err(error) => {
                    return Err(CommandParseFail {
                        actions,
                        error: error.inc_offset(offset),
                    })
                }
            }
        }
        Ok(codec::ParseValue {
            value: Self { actions },
            size: offset,
        })
    }
}
impl Codec for Command {
    fn encoded_size(&self) -> usize {
        self.actions.iter().map(|act| act.encoded_size()).sum()
    }
    unsafe fn encode(&self, out: &mut [u8]) -> usize {
        let mut offset = 0;
        for action in self.actions.iter() {
            offset += action.encode(&mut out[offset..]);
        }
        offset
    }
    fn decode(out: &[u8]) -> codec::ParseResult<Self> {
        Self::partial_decode(out).map_err(|v| v.error)
    }
}
#[test]
fn test_command() {
    test_item(
        Command {
            actions: vec![
                Action::RequestTag(action::RequestTag { id: 66, eop: true }),
                Action::ReadFileData(
                    action::new::ReadFileData {
                        resp: true,
                        group: false,
                        file_id: 0,
                        offset: 0,
                        size: 8,
                    }
                    .build()
                    .unwrap(),
                ),
                Action::ReadFileData(
                    action::new::ReadFileData {
                        resp: false,
                        group: true,
                        file_id: 4,
                        offset: 2,
                        size: 3,
                    }
                    .build()
                    .unwrap(),
                ),
                Action::Nop(action::Nop {
                    resp: true,
                    group: true,
                }),
            ],
        },
        &hex!("B4 42   41 00 00 08   81 04 02 03  C0"),
    )
}

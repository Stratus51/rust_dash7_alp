#[cfg(test)]
mod test_tools;

pub mod action;
pub mod codec;
pub mod dash7;
pub mod data;
pub mod operand;
pub mod varint;
use action::Action;
use codec::Codec;

// TODO Document int Enum values meanings (Error & Spec enums)
// TODO Split this file into more pertinent submodules and choose a better naming convention
//      (if possible, making use of the module names).
//      Also organise modules by internal section because it is a labyrinth.
// TODO Verify each item's name against the SPEC
// TODO Document each item with its specification

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
#[derive(Clone, Debug, PartialEq)]
pub struct Command {
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
            if out.is_empty() {
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
    fn encode(&self, out: &mut [u8]) -> usize {
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

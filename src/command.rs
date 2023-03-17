use crate::codec::{Codec, WithOffset, WithSize};
use crate::spec::v1_2::{
    action::{RequestTag, ResponseTag, Status},
    data::FileProperties,
};

pub trait CommandAction: Codec {
    fn nop(group: bool, resp: bool) -> Self;

    fn read_file_data(group: bool, resp: bool, file_id: u8, offset: u32, size: u32) -> Self;

    fn write_file_data(group: bool, resp: bool, file_id: u8, offset: u32, data: Box<[u8]>) -> Self;

    fn return_file_data(group: bool, resp: bool, file_id: u8, offset: u32, size: u32) -> Self;

    fn write_file_properties(
        group: bool,
        resp: bool,
        file_id: u8,
        file_properties: FileProperties,
    ) -> Self;

    fn create_new_file(
        group: bool,
        resp: bool,
        file_id: u8,
        file_properties: FileProperties,
    ) -> Self;

    fn delete_file(group: bool, resp: bool, file_id: u8) -> Self;

    fn return_file_properties(
        group: bool,
        resp: bool,
        file_id: u8,
        file_properties: FileProperties,
    ) -> Self;

    fn action_query(group: bool, resp: bool, query: crate::spec::v1_2::operand::Query) -> Self;
    fn break_query(group: bool, resp: bool, query: crate::spec::v1_2::operand::Query) -> Self;
    fn verify_checksum(
        group: bool,
        resp: bool,
        file_id: u8,
        offset: u32,
        size: u32,
        checksum: u32,
    ) -> Self;

    fn read_file_properties(
        group: bool,
        resp: bool,
        file_id: u8,
        file_properties: FileProperties,
    ) -> Self;

    crate::spec::v1_2::action::impl_action_builder_file_id!(test_exist_file, ExistFile);
    crate::spec::v1_2::action::impl_action_builder_file_id!(test_delete_file, DeleteFile);
    crate::spec::v1_2::action::impl_action_builder_file_id!(test_restore_file, RestoreFile);
    crate::spec::v1_2::action::impl_action_builder_file_id!(test_flush_file, FlushFile);
    crate::spec::v1_2::action::impl_action_builder_file_id!(test_execute_file, ExecuteFile);

    pub fn copy_file(group: bool, resp: bool, src_file_id: u8, dst_file_id: u8) -> Self {
        Self::CopyFile(CopyFile {
            group,
            resp,
            src_file_id,
            dst_file_id,
        })
    }

    pub fn status(status: Status) -> Self {
        Self::Status(status)
    }

    pub fn response_tag(eop: bool, err: bool, id: u8) -> Self {
        Self::ResponseTag(ResponseTag { eop, err, id })
    }

    pub fn chunk(chunk: Chunk) -> Self {
        Self::Chunk(chunk)
    }

    pub fn logic(logic: Logic) -> Self {
        Self::Logic(logic)
    }

    pub fn forward(forward: Forward) -> Self {
        Self::Forward(forward)
    }

    pub fn indirect_forward(indirect_forward: IndirectForward) -> Self {
        Self::IndirectForward(indirect_forward)
    }

    pub fn request_tag(eop: bool, id: u8) -> Self {
        Self::RequestTag(RequestTag { eop, id })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommandParseFail<Action: CommandAction> {
    pub actions: Vec<Action>,
    pub error: Action::Error,
}

pub struct Command<Action: CommandAction> {
    pub actions: Vec<Action>,
}

impl<Action: CommandAction> Command<Action> {
    pub fn new() -> Command<Action> {
        Command {
            actions: Vec::new(),
        }
    }

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
    pub fn decode(out: &[u8]) -> Result<Self, WithOffset<CommandParseFail<Action>>> {
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

    pub fn is_last_response(&self) -> bool {
        for action in self.actions.iter() {
            if let Action::ResponseTag(action::ResponseTag { eop, .. }) = action {
                return *eop;
            }
        }
        false
    }
}

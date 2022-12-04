use crate::{
    codec::{Codec, WithOffset, WithSize},
    v1_2::data,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FilePropertiesAction {
    /// Group with next action
    pub group: bool,
    /// Ask for a response (a status)
    pub resp: bool,
    pub file_id: u8,
    pub header: data::FileHeader,
}
crate::v1_2::action::impl_header_op!(FilePropertiesAction, group, resp, file_id, header);
impl std::fmt::Display for FilePropertiesAction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "[{}{}]f({}){}",
            if self.group { "G" } else { "-" },
            if self.resp { "R" } else { "-" },
            self.file_id,
            self.header,
        )
    }
}

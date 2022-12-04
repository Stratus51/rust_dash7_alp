use crate::codec::{Codec, StdError, WithOffset, WithSize};

/// Checks whether a file exists
// ALP_SPEC: How is the result of this command different from a read file of size 0?
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FileIdAction {
    /// Group with next action
    pub group: bool,
    /// Ask for a response (status?)
    pub resp: bool,
    pub file_id: u8,
}
super::impl_display_simple_file_op!(FileIdAction, file_id);
super::impl_simple_op!(FileIdAction, group, resp, file_id);

#[cfg(test)]
use crate::test_tools::test_item;
#[cfg(test)]
use hex_literal::hex;

use crate::codec::{Codec, StdError, WithOffset, WithSize};

// ALP_SPEC: What does that mean? Is it a complete file copy including the file properties or just
// the data? If not then if the destination file is bigger than the source, does the copy only
// overwrite the first part of the destination file?
//
// Wouldn't it be more appropriate to have 1 size and 2 file offsets?
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CopyFile {
    /// Group with next action
    pub group: bool,
    /// Ask for a response (status)
    pub resp: bool,
    pub src_file_id: u8,
    pub dst_file_id: u8,
}
impl std::fmt::Display for CopyFile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "[{}{}]f({})f({})",
            if self.group { "G" } else { "-" },
            if self.resp { "R" } else { "-" },
            self.src_file_id,
            self.dst_file_id,
        )
    }
}
super::impl_simple_op!(CopyFile, group, resp, src_file_id, dst_file_id);
#[test]
fn test_copy_file() {
    test_item(
        CopyFile {
            group: false,
            resp: false,
            src_file_id: 0x42,
            dst_file_id: 0x24,
        },
        &hex!("17 42 24"),
    )
}

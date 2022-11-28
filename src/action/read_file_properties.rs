#[cfg(test)]
use crate::test_tools::test_item;
#[cfg(test)]
use hex_literal::hex;

use crate::codec::{Codec, StdError, WithOffset, WithSize};

/// Read properties of a file
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReadFileProperties {
    /// Group with next action
    pub group: bool,
    /// Ask for a response (ReturnFileProperties)
    pub resp: bool,
    pub file_id: u8,
}
super::impl_simple_op!(ReadFileProperties, group, resp, file_id);
super::impl_display_simple_file_op!(ReadFileProperties, file_id);
#[test]
fn test_read_file_properties() {
    test_item(
        ReadFileProperties {
            group: false,
            resp: false,
            file_id: 9,
        },
        &hex!("02 09"),
    )
}

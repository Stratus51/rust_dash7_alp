use super::flag;
use super::op_code::OpCode;

/// Read data from a file
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct ReadFileData {
    /// Group with next action
    pub group: bool,
    /// Ask for a response (read data via ReturnFileData)
    ///
    /// Generally true unless you just want to trigger a read on the filesystem
    pub resp: bool,
    pub file_id: u8,
    pub offset: u32,
    pub size: u32,
    _private: (),
}

pub mod encodable_data;
pub mod masked_range;
pub mod masked_value;

pub use encodable_data::EncodableData;
pub use masked_range::MaskedRange;
pub use masked_value::MaskedValue;

/// Wrapper struct representing a dash7 file id.
///
/// It is exactly homogeneous to a byte, and the wrapping is only done
/// to help insure semantic correctness of the code.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct FileId(pub u8);

impl FileId {
    pub const fn new(n: u8) -> Self {
        Self(n)
    }

    pub const fn u8(self) -> u8 {
        let FileId(fid) = self;
        fid
    }
}

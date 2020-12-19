#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct FileId(pub u8);

impl FileId {
    pub fn new(n: u8) -> Self {
        Self(n)
    }

    pub fn u8(self) -> u8 {
        let FileId(fid) = self;
        fid
    }
}

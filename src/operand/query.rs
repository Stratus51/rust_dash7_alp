use crate::operand::file_offset::{FileOffset, FileOffsetDecodingError};
#[cfg(test)]
use crate::test_tools::test_item;
use crate::{
    codec::{Codec, StdError, WithOffset, WithSize},
    varint,
};
#[cfg(test)]
use hex_literal::hex;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum QueryComparisonType {
    Inequal = 0,
    Equal = 1,
    LessThan = 2,
    LessThanOrEqual = 3,
    GreaterThan = 4,
    GreaterThanOrEqual = 5,
}
impl QueryComparisonType {
    fn from(n: u8) -> Result<Self, u8> {
        Ok(match n {
            0 => QueryComparisonType::Inequal,
            1 => QueryComparisonType::Equal,
            2 => QueryComparisonType::LessThan,
            3 => QueryComparisonType::LessThanOrEqual,
            4 => QueryComparisonType::GreaterThan,
            5 => QueryComparisonType::GreaterThanOrEqual,
            x => return Err(x),
        })
    }
}
impl std::fmt::Display for QueryComparisonType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Inequal => "NEQ",
                Self::Equal => "EQU",
                Self::LessThan => "LTH",
                Self::LessThanOrEqual => "LTE",
                Self::GreaterThan => "GTH",
                Self::GreaterThanOrEqual => "GTE",
            }
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum QueryRangeComparisonType {
    NotInRange = 0,
    InRange = 1,
}
impl std::fmt::Display for QueryRangeComparisonType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", *self as u8)
    }
}
impl QueryRangeComparisonType {
    fn from(n: u8) -> Result<Self, u8> {
        Ok(match n {
            0 => QueryRangeComparisonType::NotInRange,
            1 => QueryRangeComparisonType::InRange,
            x => return Err(x),
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum QueryCode {
    NonVoid = 0,
    ComparisonWithZero = 1,
    ComparisonWithValue = 2,
    ComparisonWithOtherFile = 3,
    BitmapRangeComparison = 4,
    StringTokenSearch = 7,
}
impl QueryCode {
    fn from(n: u8) -> Result<Self, u8> {
        Ok(match n {
            0 => QueryCode::NonVoid,
            1 => QueryCode::ComparisonWithZero,
            2 => QueryCode::ComparisonWithValue,
            3 => QueryCode::ComparisonWithOtherFile,
            4 => QueryCode::BitmapRangeComparison,
            7 => QueryCode::StringTokenSearch,
            x => return Err(x),
        })
    }
}
impl std::fmt::Display for QueryCode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", *self as u8)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum QueryOperandDecodingError {
    MissingBytes(usize),
    Size(StdError),
    FileOffset1(FileOffsetDecodingError),
    FileOffset2(FileOffsetDecodingError),
    UnknownComparisonType(u8),
}

// ALP_SPEC Does this fail if the content overflows the file?
/// Checks if the file content exists.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NonVoid {
    pub size: u32,
    pub file: FileOffset,
}
impl std::fmt::Display for NonVoid {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{},f({})", self.size, self.file)
    }
}
impl Codec for NonVoid {
    type Error = QueryOperandDecodingError;
    fn encoded_size(&self) -> usize {
        1 + unsafe { varint::size(self.size) } as usize + self.file.encoded_size()
    }
    unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
        out[0] = QueryCode::NonVoid as u8;
        let mut offset = 1;
        offset += varint::encode_in(self.size, &mut out[offset..]) as usize;
        offset += self.file.encode_in(&mut out[offset..]);
        offset
    }
    fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
        if out.len() < 3 {
            return Err(WithOffset::new_head(Self::Error::MissingBytes(
                3 - out.len(),
            )));
        }
        let mut offset = 1;
        let WithSize {
            value: size,
            size: size_size,
        } = varint::decode(&out[offset..]).map_err(|e| {
            let WithOffset { offset: off, value } = e;
            WithOffset {
                offset: offset + off,
                value: Self::Error::Size(value),
            }
        })?;
        offset += size_size;
        let WithSize {
            value: file,
            size: file_size,
        } = FileOffset::decode(&out[offset..]).map_err(|e| {
            let WithOffset { offset: off, value } = e;
            WithOffset {
                offset: offset + off,
                value: Self::Error::FileOffset1(value),
            }
        })?;
        offset += file_size;
        Ok(WithSize {
            value: Self { size, file },
            size: offset,
        })
    }
}
#[test]
fn test_non_void_query_operand() {
    test_item(
        NonVoid {
            size: 4,
            file: FileOffset { id: 5, offset: 6 },
        },
        &hex!("00 04  05 06"),
    )
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum QueryValidationError {
    /// Query data size can't fit in a varint
    SizeTooBig,
    /// Given mask size does not match described value size
    BadMaskSize,
    /// BitmapRangeComparison: "start offset" should always be smaller than "stop offset"
    StartGreaterThanStop,
}

/// Compare file content, optionally masked, with 0.
#[derive(Clone, Debug, PartialEq)]
pub struct ComparisonWithZero {
    pub signed_data: bool,
    pub comparison_type: QueryComparisonType,
    pub size: u32,
    pub mask: Option<Box<[u8]>>,
    pub file: FileOffset,
}
impl std::fmt::Display for ComparisonWithZero {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}|{},{},",
            if self.signed_data { "S" } else { "U" },
            self.comparison_type,
            self.size
        )?;
        if let Some(mask) = &self.mask {
            write!(f, "msk=0x{},", hex::encode_upper(mask))?;
        }
        write!(f, "f({})", self.file)
    }
}
impl ComparisonWithZero {
    pub fn validate(&self) -> Result<(), QueryValidationError> {
        if self.size > varint::MAX {
            return Err(QueryValidationError::SizeTooBig);
        }
        if let Some(mask) = &self.mask {
            if mask.len() as u32 != self.size {
                return Err(QueryValidationError::BadMaskSize);
            }
        }
        Ok(())
    }
}
impl Codec for ComparisonWithZero {
    type Error = QueryOperandDecodingError;
    fn encoded_size(&self) -> usize {
        let mask_size = match self.mask {
            Some(_) => self.size as usize,
            None => 0,
        };
        1 + unsafe { varint::size(self.size) } as usize + mask_size + self.file.encoded_size()
    }
    unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
        let mask_flag = match self.mask {
            Some(_) => 1,
            None => 0,
        };
        let signed_flag = if self.signed_data { 1 } else { 0 };
        let mut offset = 0;
        out[0] = ((QueryCode::ComparisonWithZero as u8) << 5)
            | (mask_flag << 4)
            | (signed_flag << 3)
            | self.comparison_type as u8;
        offset += 1;
        offset += varint::encode_in(self.size, &mut out[offset..]) as usize;
        if let Some(mask) = &self.mask {
            out[offset..offset + (self.size as usize)].clone_from_slice(mask);
            offset += mask.len();
        }
        offset += self.file.encode_in(&mut out[offset..]);
        offset
    }
    fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
        if out.len() < 1 + 1 + 2 {
            return Err(WithOffset::new_head(Self::Error::MissingBytes(
                1 + 1 + 2 - out.len(),
            )));
        }
        let mask_flag = out[0] & (1 << 4) != 0;
        let signed_data = out[0] & (1 << 3) != 0;
        let comparison_type = QueryComparisonType::from(out[0] & 0x07)
            .map_err(|e| WithOffset::new_head(Self::Error::UnknownComparisonType(e)))?;
        let WithSize {
            value: size,
            size: size_size,
        } = varint::decode(&out[1..]).map_err(|e| {
            let WithOffset { offset, value } = e;
            WithOffset {
                offset: offset + 1,
                value: Self::Error::Size(value),
            }
        })?;
        let mut offset = 1 + size_size;
        let mask = if mask_flag {
            let mut data = vec![0u8; size as usize].into_boxed_slice();
            data.clone_from_slice(&out[offset..offset + size as usize]);
            offset += size as usize;
            Some(data)
        } else {
            None
        };
        let WithSize {
            value: file,
            size: offset_size,
        } = FileOffset::decode(&out[offset..]).map_err(|e| {
            let WithOffset { offset: off, value } = e;
            WithOffset {
                offset: offset + off,
                value: Self::Error::FileOffset1(value),
            }
        })?;
        offset += offset_size;
        Ok(WithSize {
            value: Self {
                signed_data,
                comparison_type,
                size,
                mask,
                file,
            },
            size: offset,
        })
    }
}
#[test]
fn test_comparison_with_zero_operand() {
    test_item(
        ComparisonWithZero {
            signed_data: true,
            comparison_type: QueryComparisonType::Inequal,
            size: 3,
            mask: Some(vec![0, 1, 2].into_boxed_slice()),
            file: FileOffset { id: 4, offset: 5 },
        },
        &hex!("38 03  000102  04 05"),
    )
}

/// Compare some file content optionally masked, with a value
#[derive(Clone, Debug, PartialEq)]
pub struct ComparisonWithValue {
    pub signed_data: bool,
    pub comparison_type: QueryComparisonType,
    pub size: u32,
    pub mask: Option<Box<[u8]>>,
    pub value: Box<[u8]>,
    pub file: FileOffset,
}
impl std::fmt::Display for ComparisonWithValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}|{},{},",
            if self.signed_data { "S" } else { "U" },
            self.comparison_type,
            self.size
        )?;
        if let Some(mask) = &self.mask {
            write!(f, "msk=0x{},", hex::encode_upper(mask))?;
        }
        write!(f, "v=0x{},f({})", hex::encode_upper(&self.value), self.file)
    }
}
impl ComparisonWithValue {
    pub fn validate(&self) -> Result<(), QueryValidationError> {
        let size = self.value.len();
        if size as u32 > varint::MAX {
            return Err(QueryValidationError::SizeTooBig);
        }
        if let Some(mask) = &self.mask {
            if mask.len() != size {
                return Err(QueryValidationError::BadMaskSize);
            }
        }
        Ok(())
    }
}
impl Codec for ComparisonWithValue {
    type Error = QueryOperandDecodingError;
    fn encoded_size(&self) -> usize {
        let mask_size = match self.mask {
            Some(_) => self.size as usize,
            None => 0,
        };
        1 + unsafe { varint::size(self.size) } as usize
            + mask_size
            + self.value.len()
            + self.file.encoded_size()
    }
    unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
        let mask_flag = match self.mask {
            Some(_) => 1,
            None => 0,
        };
        let signed_flag = if self.signed_data { 1 } else { 0 };
        let mut offset = 0;
        out[0] = ((QueryCode::ComparisonWithValue as u8) << 5)
            | (mask_flag << 4)
            | (signed_flag << 3)
            | self.comparison_type as u8;
        offset += 1;
        offset += varint::encode_in(self.size, &mut out[offset..]) as usize;
        if let Some(mask) = &self.mask {
            out[offset..offset + self.size as usize].clone_from_slice(mask);
            offset += mask.len();
        }
        out[offset..offset + self.size as usize].clone_from_slice(&self.value[..]);
        offset += self.value.len();
        offset += self.file.encode_in(&mut out[offset..]);
        offset
    }
    fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
        if out.len() < 1 + 1 + 2 {
            return Err(WithOffset::new_head(Self::Error::MissingBytes(
                1 + 1 + 2 - out.len(),
            )));
        }
        let mask_flag = out[0] & (1 << 4) != 0;
        let signed_data = out[0] & (1 << 3) != 0;
        let comparison_type = QueryComparisonType::from(out[0] & 0x07)
            .map_err(|e| WithOffset::new_head(Self::Error::UnknownComparisonType(e)))?;
        let WithSize {
            value: size,
            size: size_size,
        } = varint::decode(&out[1..]).map_err(|e| {
            let WithOffset { offset, value } = e;
            WithOffset {
                offset: offset + 1,
                value: Self::Error::Size(value),
            }
        })?;
        let mut offset = 1 + size_size;
        let mask = if mask_flag {
            let mut data = vec![0u8; size as usize].into_boxed_slice();
            data.clone_from_slice(&out[offset..offset + size as usize]);
            offset += size as usize;
            Some(data)
        } else {
            None
        };
        let mut value = vec![0u8; size as usize].into_boxed_slice();
        value.clone_from_slice(&out[offset..offset + size as usize]);
        offset += size as usize;
        let WithSize {
            value: file,
            size: offset_size,
        } = FileOffset::decode(&out[offset..]).map_err(|e| {
            let WithOffset { offset: off, value } = e;
            WithOffset {
                offset: offset + off,
                value: Self::Error::FileOffset1(value),
            }
        })?;
        offset += offset_size;
        Ok(WithSize {
            value: Self {
                signed_data,
                comparison_type,
                size,
                mask,
                value,
                file,
            },
            size: offset,
        })
    }
}
#[test]
fn test_comparison_with_value_operand() {
    test_item(
        ComparisonWithValue {
            signed_data: false,
            comparison_type: QueryComparisonType::Equal,
            size: 3,
            mask: None,
            value: vec![9, 9, 9].into_boxed_slice(),
            file: FileOffset { id: 4, offset: 5 },
        },
        &hex!("41 03   090909  04 05"),
    )
}

/// Compare content of 2 files optionally masked
#[derive(Clone, Debug, PartialEq)]
pub struct ComparisonWithOtherFile {
    pub signed_data: bool,
    pub comparison_type: QueryComparisonType,
    pub size: u32,
    pub mask: Option<Box<[u8]>>,
    pub file1: FileOffset,
    pub file2: FileOffset,
}
impl std::fmt::Display for ComparisonWithOtherFile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}|{},{},",
            if self.signed_data { "S" } else { "U" },
            self.comparison_type,
            self.size
        )?;
        if let Some(mask) = &self.mask {
            write!(f, "msk=0x{},", hex::encode_upper(mask))?;
        }
        write!(f, "f({})~f({})", self.file1, self.file2)
    }
}
impl ComparisonWithOtherFile {
    pub fn validate(&self) -> Result<(), QueryValidationError> {
        if self.size > varint::MAX {
            return Err(QueryValidationError::SizeTooBig);
        }
        if let Some(mask) = &self.mask {
            if mask.len() as u32 != self.size {
                return Err(QueryValidationError::BadMaskSize);
            }
        }
        Ok(())
    }
}
impl Codec for ComparisonWithOtherFile {
    type Error = QueryOperandDecodingError;
    fn encoded_size(&self) -> usize {
        let mask_size = match self.mask {
            Some(_) => self.size as usize,
            None => 0,
        };
        1 + unsafe { varint::size(self.size) } as usize
            + mask_size
            + self.file1.encoded_size()
            + self.file2.encoded_size()
    }
    unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
        let mask_flag = match self.mask {
            Some(_) => 1,
            None => 0,
        };
        let signed_flag = if self.signed_data { 1 } else { 0 };
        let mut offset = 0;
        out[0] = ((QueryCode::ComparisonWithOtherFile as u8) << 5)
            | (mask_flag << 4)
            | (signed_flag << 3)
            | self.comparison_type as u8;
        offset += 1;
        offset += varint::encode_in(self.size, &mut out[offset..]) as usize;
        if let Some(mask) = &self.mask {
            out[offset..offset + self.size as usize].clone_from_slice(mask);
            offset += mask.len();
        }
        offset += self.file1.encode_in(&mut out[offset..]);
        offset += self.file2.encode_in(&mut out[offset..]);
        offset
    }
    fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
        if out.len() < 1 + 1 + 2 + 2 {
            return Err(WithOffset::new_head(Self::Error::MissingBytes(
                1 + 1 + 2 + 2 - out.len(),
            )));
        }
        let mask_flag = out[0] & (1 << 4) != 0;
        let signed_data = out[0] & (1 << 3) != 0;
        let comparison_type = QueryComparisonType::from(out[0] & 0x07)
            .map_err(|e| WithOffset::new_head(Self::Error::UnknownComparisonType(e)))?;
        let WithSize {
            value: size,
            size: size_size,
        } = varint::decode(&out[1..]).map_err(|e| {
            let WithOffset { offset: _, value } = e;
            WithOffset {
                offset: 1,
                value: Self::Error::Size(value),
            }
        })?;
        let mut offset = 1 + size_size;
        let mask = if mask_flag {
            let mut data = vec![0u8; size as usize].into_boxed_slice();
            data.clone_from_slice(&out[offset..offset + size as usize]);
            offset += size as usize;
            Some(data)
        } else {
            None
        };
        let WithSize {
            value: file1,
            size: file1_size,
        } = FileOffset::decode(&out[offset..]).map_err(|e| {
            let WithOffset { offset: off, value } = e;
            WithOffset {
                offset: offset + off,
                value: Self::Error::FileOffset1(value),
            }
        })?;
        offset += file1_size;
        let WithSize {
            value: file2,
            size: file2_size,
        } = FileOffset::decode(&out[offset..]).map_err(|e| {
            let WithOffset { offset: off, value } = e;
            WithOffset {
                offset: offset + off,
                value: Self::Error::FileOffset2(value),
            }
        })?;
        offset += file2_size;
        Ok(WithSize {
            value: Self {
                signed_data,
                comparison_type,
                size,
                mask,
                file1,
                file2,
            },
            size: offset,
        })
    }
}
#[test]
fn test_comparison_with_other_file_operand() {
    test_item(
        ComparisonWithOtherFile {
            signed_data: false,
            comparison_type: QueryComparisonType::GreaterThan,
            size: 2,
            mask: Some(vec![0xFF, 0xFF].into_boxed_slice()),
            file1: FileOffset { id: 4, offset: 5 },
            file2: FileOffset { id: 8, offset: 9 },
        },
        &hex!("74 02 FFFF   04 05    08 09"),
    )
}

/// Check if the content of a file is (not) contained in the sent bitmap values
#[derive(Clone, Debug, PartialEq)]
pub struct BitmapRangeComparison {
    pub signed_data: bool,
    pub comparison_type: QueryRangeComparisonType,
    pub size: u32,
    /// ALP SPEC: In theory, start and stop can be huge array thus impossible to cast into any trivial
    /// number. For simplicity's sake, this library encodes them in a u32.
    pub start: u32,
    pub stop: u32,
    pub mask: Option<Box<[u8]>>,
    pub file: FileOffset,
}
impl std::fmt::Display for BitmapRangeComparison {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}|{},{},{}-{},",
            if self.signed_data { "S" } else { "U" },
            self.comparison_type,
            self.size,
            self.start,
            self.stop
        )?;
        if let Some(mask) = &self.mask {
            write!(f, "msk=0x{},", hex::encode_upper(mask))?;
        }
        write!(f, "f({})", self.file)
    }
}
impl BitmapRangeComparison {
    pub fn validate(&self) -> Result<(), QueryValidationError> {
        if self.start > self.stop {
            return Err(QueryValidationError::StartGreaterThanStop);
        }

        let bitmap_size = (self.stop - self.start + 6) / 8; // ALP SPEC: Thanks for the calculation
        if let Some(mask) = &self.mask {
            if mask.len() != bitmap_size as usize {
                return Err(QueryValidationError::BadMaskSize);
            }
        }
        Ok(())
    }
}
impl Codec for BitmapRangeComparison {
    type Error = QueryOperandDecodingError;
    fn encoded_size(&self) -> usize {
        1 + unsafe { varint::size(self.size) } as usize
            + 2 * self.size as usize
            + self.mask.as_ref().map(|b| b.len()).unwrap_or(0)
            + self.file.encoded_size()
    }
    unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
        let mut offset = 0;
        let signed_flag = if self.signed_data { 1 } else { 0 };
        out[0] = ((QueryCode::BitmapRangeComparison as u8) << 5)
            | ((self.mask.is_none() as u8) << 4)
            | (signed_flag << 3)
            | self.comparison_type as u8;
        offset += 1;
        offset += varint::encode_in(self.size, &mut out[offset..]) as usize;
        let size = self.size as usize;
        out[offset..offset + size].clone_from_slice(&self.start.to_be_bytes()[4 - size..]);
        offset += size;
        out[offset..offset + size].clone_from_slice(&self.stop.to_be_bytes()[4 - size..]);
        offset += size;
        if let Some(mask) = &self.mask {
            out[offset..offset + mask.len()].clone_from_slice(&mask[..]);
            offset += mask.len();
        }
        offset += self.file.encode_in(&mut out[offset..]);
        offset
    }
    fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
        if out.len() < 1 + 1 + 2 {
            return Err(WithOffset::new_head(Self::Error::MissingBytes(
                1 + 1 + 2 - out.len(),
            )));
        }
        let mask_flag = out[0] & (1 << 4) == 0;
        let signed_data = out[0] & (1 << 3) != 0;
        let comparison_type = QueryRangeComparisonType::from(out[0] & 0x07)
            .map_err(|e| WithOffset::new_head(Self::Error::UnknownComparisonType(e)))?;
        let WithSize {
            value: size32,
            size: size_size,
        } = varint::decode(&out[1..]).map_err(|e| {
            let WithOffset { offset, value } = e;
            WithOffset {
                offset: offset + 1,
                value: Self::Error::Size(value),
            }
        })?;
        let size = size32 as usize;
        let mut offset = 1 + size_size;
        let mut raw_start = vec![0u8; size].into_boxed_slice();
        raw_start.clone_from_slice(&out[offset..offset + size]);
        offset += size;
        let mut raw_stop = vec![0u8; size].into_boxed_slice();
        raw_stop.clone_from_slice(&out[offset..offset + size]);
        offset += size;
        let mut start = 0u32;
        let mut stop = 0u32;
        for i in 0..size {
            start = (start << 8) + raw_start[i] as u32;
            stop = (stop << 8) + raw_stop[i] as u32;
        }
        let mask = if mask_flag {
            let bitmap_size = (stop - start + 6) / 8; // ALP SPEC: Thanks for the calculation
            let mut bitmap = vec![0u8; bitmap_size as usize].into_boxed_slice();
            bitmap.clone_from_slice(&out[offset..offset + bitmap_size as usize]);
            offset += bitmap_size as usize;
            Some(bitmap)
        } else {
            None
        };
        let WithSize {
            value: file,
            size: file_size,
        } = FileOffset::decode(&out[offset..]).map_err(|e| {
            let WithOffset { offset: off, value } = e;
            WithOffset {
                offset: offset + off,
                value: Self::Error::FileOffset1(value),
            }
        })?;
        offset += file_size;
        Ok(WithSize {
            value: Self {
                signed_data,
                comparison_type,
                size: size32,
                start,
                stop,
                mask,
                file,
            },
            size: offset,
        })
    }
}
#[test]
fn test_bitmap_range_comparison_operand() {
    test_item(
        BitmapRangeComparison {
            signed_data: false,
            comparison_type: QueryRangeComparisonType::InRange,
            size: 2,

            start: 3,
            stop: 32,
            mask: Some(Box::new(hex!("01020304"))),

            file: FileOffset { id: 0, offset: 4 },
        },
        &hex!("81 02 0003  0020  01020304  00 04"),
    )
}

/// Compare some file content, optional masked, with an array of bytes and up to a certain number
/// of errors.
#[derive(Clone, Debug, PartialEq)]
pub struct StringTokenSearch {
    pub max_errors: u8,
    pub size: u32,
    pub mask: Option<Box<[u8]>>,
    pub value: Box<[u8]>,
    pub file: FileOffset,
}
impl std::fmt::Display for StringTokenSearch {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{},{},", self.max_errors, self.size)?;
        if let Some(mask) = &self.mask {
            write!(f, "msk=0x{},", hex::encode_upper(mask))?;
        }
        write!(f, "v=0x{},f({})", hex::encode_upper(&self.value), self.file)
    }
}
impl StringTokenSearch {
    pub fn validate(&self) -> Result<(), QueryValidationError> {
        if self.size > varint::MAX {
            return Err(QueryValidationError::SizeTooBig);
        }
        if let Some(mask) = &self.mask {
            if mask.len() as u32 != self.size {
                return Err(QueryValidationError::BadMaskSize);
            }
        }
        Ok(())
    }
}
impl Codec for StringTokenSearch {
    type Error = QueryOperandDecodingError;
    fn encoded_size(&self) -> usize {
        let mask_size = match self.mask {
            Some(_) => self.size as usize,
            None => 0,
        };
        1 + unsafe { varint::size(self.size) } as usize
            + mask_size
            + self.value.len()
            + self.file.encoded_size()
    }
    unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
        let mask_flag = match self.mask {
            Some(_) => 1,
            None => 0,
        };
        let mut offset = 0;
        out[0] = ((QueryCode::StringTokenSearch as u8) << 5)
            | (mask_flag << 4)
            // | (0 << 3)
            | self.max_errors;
        offset += 1;
        offset += varint::encode_in(self.size, &mut out[offset..]) as usize;
        if let Some(mask) = &self.mask {
            out[offset..offset + self.size as usize].clone_from_slice(mask);
            offset += mask.len();
        }
        out[offset..offset + self.size as usize].clone_from_slice(&self.value[..]);
        offset += self.value.len();
        offset += self.file.encode_in(&mut out[offset..]);
        offset
    }
    fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
        if out.len() < 1 + 1 + 2 {
            return Err(WithOffset::new_head(Self::Error::MissingBytes(
                1 + 1 + 2 - out.len(),
            )));
        }
        let mask_flag = out[0] & (1 << 4) != 0;
        let max_errors = out[0] & 0x07;
        let WithSize {
            value: size32,
            size: size_size,
        } = varint::decode(&out[1..]).map_err(|e| {
            let WithOffset { offset, value } = e;
            WithOffset {
                offset: offset + 1,
                value: Self::Error::Size(value),
            }
        })?;
        let size = size32 as usize;
        let mut offset = 1 + size_size;
        let mask = if mask_flag {
            let mut data = vec![0u8; size].into_boxed_slice();
            data.clone_from_slice(&out[offset..offset + size]);
            offset += size;
            Some(data)
        } else {
            None
        };
        let mut value = vec![0u8; size].into_boxed_slice();
        value.clone_from_slice(&out[offset..offset + size]);
        offset += size;
        let WithSize {
            value: file,
            size: offset_size,
        } = FileOffset::decode(&out[offset..]).map_err(|e| {
            let WithOffset { offset: off, value } = e;
            WithOffset {
                offset: offset + off,
                value: Self::Error::FileOffset1(value),
            }
        })?;
        offset += offset_size;
        Ok(WithSize {
            value: Self {
                max_errors,
                size: size32,
                mask,
                value,
                file,
            },
            size: offset,
        })
    }
}
#[test]
fn test_string_token_search_operand() {
    test_item(
        StringTokenSearch {
            max_errors: 2,
            size: 4,
            mask: Some(Box::new(hex!("FF00FF00"))),
            value: Box::new(hex!("01020304")),
            file: FileOffset { id: 0, offset: 4 },
        },
        &hex!("F2 04 FF00FF00  01020304  00 04"),
    )
}

/// The query operand provides a way to do optional actions. It represents a condition.
#[derive(Clone, Debug, PartialEq)]
pub enum Query {
    NonVoid(NonVoid),
    ComparisonWithZero(ComparisonWithZero),
    ComparisonWithValue(ComparisonWithValue),
    ComparisonWithOtherFile(ComparisonWithOtherFile),
    BitmapRangeComparison(BitmapRangeComparison),
    StringTokenSearch(StringTokenSearch),
}
impl std::fmt::Display for Query {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::NonVoid(v) => write!(f, "NV:[{}]", v),
            Self::ComparisonWithZero(v) => write!(f, "WZ:[{}]", v),
            Self::ComparisonWithValue(v) => write!(f, "WV:[{}]", v),
            Self::ComparisonWithOtherFile(v) => write!(f, "WF:[{}]", v),
            Self::BitmapRangeComparison(v) => write!(f, "BM:[{}]", v),
            Self::StringTokenSearch(v) => write!(f, "ST:[{}]", v),
        }
    }
}
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum QueryDecodingError {
    MissingBytes(usize),
    UnknownQueryCode(u8),
    NonVoid(QueryOperandDecodingError),
    ComparisonWithZero(QueryOperandDecodingError),
    ComparisonWithValue(QueryOperandDecodingError),
    ComparisonWithOtherFile(QueryOperandDecodingError),
    BitmapRangeComparison(QueryOperandDecodingError),
    StringTokenSearch(QueryOperandDecodingError),
}
impl Codec for Query {
    type Error = QueryDecodingError;
    fn encoded_size(&self) -> usize {
        match self {
            Query::NonVoid(v) => v.encoded_size(),
            Query::ComparisonWithZero(v) => v.encoded_size(),
            Query::ComparisonWithValue(v) => v.encoded_size(),
            Query::ComparisonWithOtherFile(v) => v.encoded_size(),
            Query::BitmapRangeComparison(v) => v.encoded_size(),
            Query::StringTokenSearch(v) => v.encoded_size(),
        }
    }
    unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
        match self {
            Query::NonVoid(v) => v.encode_in(out),
            Query::ComparisonWithZero(v) => v.encode_in(out),
            Query::ComparisonWithValue(v) => v.encode_in(out),
            Query::ComparisonWithOtherFile(v) => v.encode_in(out),
            Query::BitmapRangeComparison(v) => v.encode_in(out),
            Query::StringTokenSearch(v) => v.encode_in(out),
        }
    }
    fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
        match QueryCode::from(out[0] >> 5)
            .map_err(|e| WithOffset::new_head(Self::Error::UnknownQueryCode(e)))?
        {
            QueryCode::NonVoid => NonVoid::decode(out)
                .map(|ok| ok.map_value(Query::NonVoid))
                .map_err(|e| e.map_value(Self::Error::NonVoid)),
            QueryCode::ComparisonWithZero => ComparisonWithZero::decode(out)
                .map(|ok| ok.map_value(Query::ComparisonWithZero))
                .map_err(|e| e.map_value(Self::Error::ComparisonWithZero)),
            QueryCode::ComparisonWithValue => ComparisonWithValue::decode(out)
                .map(|ok| ok.map_value(Query::ComparisonWithValue))
                .map_err(|e| e.map_value(Self::Error::ComparisonWithValue)),
            QueryCode::ComparisonWithOtherFile => ComparisonWithOtherFile::decode(out)
                .map(|ok| ok.map_value(Query::ComparisonWithOtherFile))
                .map_err(|e| e.map_value(Self::Error::ComparisonWithOtherFile)),
            QueryCode::BitmapRangeComparison => BitmapRangeComparison::decode(out)
                .map(|ok| ok.map_value(Query::BitmapRangeComparison))
                .map_err(|e| e.map_value(Self::Error::BitmapRangeComparison)),
            QueryCode::StringTokenSearch => StringTokenSearch::decode(out)
                .map(|ok| ok.map_value(Query::StringTokenSearch))
                .map_err(|e| e.map_value(Self::Error::StringTokenSearch)),
        }
    }
}

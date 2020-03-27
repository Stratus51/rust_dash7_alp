#[cfg(test)]
use crate::test_tools::test_item;
use crate::{
    codec::{Codec, ParseError, ParseFail, ParseResult, ParseResultExtension, ParseValue},
    dash7, varint, Enum,
};
#[cfg(test)]
use hex_literal::hex;

// ===============================================================================
// Alp Interfaces
// ===============================================================================
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InterfaceId {
    Host = 0,
    D7asp = 0xD7,
}

#[derive(Clone, Debug, PartialEq)]
pub enum InterfaceConfiguration {
    Host,
    D7asp(dash7::InterfaceConfiguration),
}
impl Codec for InterfaceConfiguration {
    fn encoded_size(&self) -> usize {
        1 + match self {
            InterfaceConfiguration::Host => 0,
            InterfaceConfiguration::D7asp(v) => v.encoded_size(),
        }
    }
    fn encode(&self, out: &mut [u8]) -> usize {
        match self {
            InterfaceConfiguration::Host => {
                out[0] = InterfaceId::Host as u8;
                1
            }
            InterfaceConfiguration::D7asp(v) => {
                out[0] = InterfaceId::D7asp as u8;
                1 + v.encode(&mut out[1..])
            }
        }
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
        if out.is_empty() {
            return Err(ParseFail::MissingBytes(Some(1)));
        }
        const HOST: u8 = InterfaceId::Host as u8;
        const D7ASP: u8 = InterfaceId::D7asp as u8;
        Ok(match out[0] {
            HOST => ParseValue {
                value: InterfaceConfiguration::Host,
                size: 1,
            },
            D7ASP => {
                let ParseValue { value, size } =
                    dash7::InterfaceConfiguration::decode(&out[1..]).inc_offset(1)?;
                ParseValue {
                    value: InterfaceConfiguration::D7asp(value),
                    size: size + 1,
                }
            }
            id => {
                return Err(ParseFail::Error {
                    error: ParseError::UnknownEnumVariant {
                        en: Enum::InterfaceId,
                        value: id,
                    },
                    offset: 0,
                })
            }
        })
    }
}
#[test]
fn test_interface_configuration_d7asp() {
    test_item(
        InterfaceConfiguration::D7asp(dash7::InterfaceConfiguration {
            qos: dash7::Qos {
                retry: dash7::RetryMode::No,
                resp: dash7::RespMode::Any,
            },
            to: 0x23,
            te: 0x34,
            addressee: dash7::Addressee {
                nls_method: dash7::NlsMethod::AesCcm32,
                access_class: 0xFF,
                address: dash7::Address::Vid(Box::new([0xAB, 0xCD])),
            },
        }),
        &hex!("D7   02 23 34   37 FF ABCD"),
    )
}
#[test]
fn test_interface_configuration_host() {
    test_item(InterfaceConfiguration::Host, &hex!("00"))
}

pub struct NewInterfaceStatusUnknown {
    pub id: u8,
    pub data: Box<[u8]>,
}
impl NewInterfaceStatusUnknown {
    pub fn build(self) -> Result<InterfaceStatusUnknown, InterfaceStatusUnknownError> {
        InterfaceStatusUnknown::new(self)
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct InterfaceStatusUnknown {
    pub id: u8,
    pub data: Box<[u8]>,
    _private: (),
}
#[derive(Clone, Debug, PartialEq)]
pub enum InterfaceStatusUnknownError {
    DataTooBig,
}
impl InterfaceStatusUnknown {
    pub fn new(new: NewInterfaceStatusUnknown) -> Result<Self, InterfaceStatusUnknownError> {
        if new.data.len() > varint::MAX as usize {
            return Err(InterfaceStatusUnknownError::DataTooBig);
        }
        Ok(Self {
            id: new.id,
            data: new.data,
            _private: (),
        })
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum InterfaceStatus {
    Host,
    D7asp(dash7::InterfaceStatus),
    Unknown(InterfaceStatusUnknown),
}
impl Codec for InterfaceStatus {
    fn encoded_size(&self) -> usize {
        let data_size = match self {
            InterfaceStatus::Host => 0,
            InterfaceStatus::D7asp(itf) => itf.encoded_size(),
            InterfaceStatus::Unknown(InterfaceStatusUnknown { data, .. }) => data.len(),
        };
        1 + unsafe { varint::size(data_size as u32) as usize } + data_size
    }
    fn encode(&self, out: &mut [u8]) -> usize {
        let mut offset = 1;
        match self {
            InterfaceStatus::Host => {
                out[0] = InterfaceId::Host as u8;
                out[1] = 0;
                offset += 1;
            }
            InterfaceStatus::D7asp(v) => {
                out[0] = InterfaceId::D7asp as u8;
                let size = v.encoded_size() as u32;
                let size_size = unsafe { varint::encode(size, &mut out[offset..]) };
                offset += size_size as usize;
                offset += v.encode(&mut out[offset..]);
            }
            InterfaceStatus::Unknown(InterfaceStatusUnknown { id, data, .. }) => {
                out[0] = *id;
                let size = data.len() as u32;
                let size_size = unsafe { varint::encode(size, &mut out[offset..]) };
                offset += size_size as usize;
                out[offset..offset + data.len()].clone_from_slice(data);
                offset += data.len();
            }
        };
        offset
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
        if out.is_empty() {
            return Err(ParseFail::MissingBytes(Some(1)));
        }
        const HOST: u8 = InterfaceId::Host as u8;
        const D7ASP: u8 = InterfaceId::D7asp as u8;
        let mut offset = 1;
        let value = match out[0] {
            HOST => {
                offset += 1;
                InterfaceStatus::Host
            }
            D7ASP => {
                let ParseValue {
                    value: size,
                    size: size_size,
                } = varint::decode(&out[offset..])?;
                let size = size as usize;
                offset += size_size;
                let ParseValue { value, size } =
                    dash7::InterfaceStatus::decode(&out[offset..offset + size])
                        .inc_offset(offset)?;
                offset += size;
                InterfaceStatus::D7asp(value)
            }
            id => {
                let ParseValue {
                    value: size,
                    size: size_size,
                } = varint::decode(&out[offset..])?;
                let size = size as usize;
                offset += size_size;
                if out.len() < offset + size {
                    return Err(ParseFail::MissingBytes(Some(offset + size - out.len())));
                }
                let mut data = vec![0u8; size].into_boxed_slice();
                data.clone_from_slice(&out[offset..size]);
                offset += size;
                InterfaceStatus::Unknown(InterfaceStatusUnknown {
                    id,
                    data,
                    _private: (),
                })
            }
        };
        Ok(ParseValue {
            value,
            size: offset,
        })
    }
}
#[test]
fn test_interface_status_d7asp() {
    test_item(
        InterfaceStatus::D7asp(
            dash7::NewInterfaceStatus {
                ch_header: 1,
                ch_idx: 0x0123,
                rxlev: 2,
                lb: 3,
                snr: 4,
                status: 5,
                token: 6,
                seq: 7,
                resp_to: 8,
                addressee: dash7::Addressee {
                    nls_method: dash7::NlsMethod::AesCcm32,
                    access_class: 0xFF,
                    address: dash7::Address::Vid(Box::new([0xAB, 0xCD])),
                },
                nls_state: Some(hex!("00 11 22 33 44")),
            }
            .build()
            .unwrap(),
        ),
        &hex!("D7 13    01 0123 02 03 04 05 06 07 08   37 FF ABCD  0011223344"),
    )
}
#[test]
fn test_interface_status_host() {
    test_item(InterfaceStatus::Host, &hex!("00 00"))
}

// ===============================================================================
// Operands
// ===============================================================================
pub struct NewFileOffset {
    pub id: u8,
    pub offset: u32,
}
impl NewFileOffset {
    pub fn build(self) -> Result<FileOffset, FileOffsetError> {
        FileOffset::new(self)
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FileOffset {
    pub id: u8,
    pub offset: u32,
    _private: (),
}
#[derive(Clone, Debug, PartialEq)]
pub enum FileOffsetError {
    OffsetTooBig,
}

impl FileOffset {
    pub fn new(new: NewFileOffset) -> Result<Self, FileOffsetError> {
        if new.offset > varint::MAX {
            return Err(FileOffsetError::OffsetTooBig);
        }
        Ok(Self {
            id: new.id,
            offset: new.offset,
            _private: (),
        })
    }
}

impl Codec for FileOffset {
    fn encoded_size(&self) -> usize {
        1 + unsafe { varint::size(self.offset) } as usize
    }
    fn encode(&self, out: &mut [u8]) -> usize {
        out[0] = self.id;
        1 + unsafe { varint::encode(self.offset, &mut out[1..]) } as usize
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
        if out.len() < 2 {
            return Err(ParseFail::MissingBytes(Some(2 - out.len())));
        }
        let ParseValue {
            value: offset,
            size,
        } = varint::decode(&out[1..])?;
        Ok(ParseValue {
            value: Self {
                id: out[0],
                offset,
                _private: (),
            },
            size: 1 + size,
        })
    }
}
#[test]
fn test_file_offset_operand() {
    test_item(
        FileOffset {
            id: 2,
            offset: 0x3F_FF,
            _private: (),
        },
        &hex!("02 7F FF"),
    )
}

pub mod status_code {
    pub const RECEIVED: u8 = 1;
    pub const OK: u8 = 0;
    pub const FILE_ID_MISSING: u8 = 0xFF;
    pub const CREATE_FILE_ID_ALREADY_EXIST: u8 = 0xFE;
    pub const FILE_IS_NOT_RESTORABLE: u8 = 0xFD;
    pub const INSUFFICIENT_PERMISSION: u8 = 0xFC;
    pub const CREATE_FILE_LENGTH_OVERFLOW: u8 = 0xFB;
    pub const CREATE_FILE_ALLOCATION_OVERFLOW: u8 = 0xFA; // ALP_SPEC: ??? Difference with the previous one?;
    pub const WRITE_OFFSET_OVERFLOW: u8 = 0xF9;
    pub const WRITE_DATA_OVERFLOW: u8 = 0xF8;
    pub const WRITE_STORAGE_UNAVAILABLE: u8 = 0xF7;
    pub const UNKNOWN_OPERATION: u8 = 0xF6;
    pub const OPERAND_INCOMPLETE: u8 = 0xF5;
    pub const OPERAND_WRONG_FORMAT: u8 = 0xF4;
    pub const UNKNOWN_ERROR: u8 = 0x80;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Status {
    pub action_id: u8,
    pub status: u8,
}
impl Codec for Status {
    fn encoded_size(&self) -> usize {
        1 + 1
    }
    fn encode(&self, out: &mut [u8]) -> usize {
        out[0] = self.action_id;
        out[1] = self.status as u8;
        2
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
        if out.len() < 2 {
            return Err(ParseFail::MissingBytes(Some(2 - out.len())));
        }
        Ok(ParseValue {
            value: Self {
                action_id: out[0],
                status: out[1],
            },
            size: 2,
        })
    }
}
#[test]
fn test_status_operand() {
    test_item(
        Status {
            action_id: 2,
            status: status_code::UNKNOWN_OPERATION,
        },
        &hex!("02 F6"),
    )
}

// ALP SPEC: where is this defined? Link?
//  Not found in either specs !
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Permission {
    Dash7([u8; 8]), // TODO ALP SPEC Check + what is its name?
}

impl Permission {
    fn id(self) -> u8 {
        match self {
            Permission::Dash7(_) => 0x42, // TODO Check
        }
    }
}

impl Codec for Permission {
    fn encoded_size(&self) -> usize {
        1 + match self {
            Permission::Dash7(_) => 8,
        }
    }
    fn encode(&self, out: &mut [u8]) -> usize {
        out[0] = self.id();
        1 + match self {
            Permission::Dash7(token) => {
                out[1..1 + token.len()].clone_from_slice(&token[..]);
                8
            }
        }
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
        if out.is_empty() {
            return Err(ParseFail::MissingBytes(Some(1)));
        }
        let mut offset = 1;
        match out[0] {
            0x42 => {
                let mut token = [0; 8];
                token.clone_from_slice(&out[offset..offset + 8]);
                offset += 8;
                Ok(ParseValue {
                    value: Permission::Dash7(token),
                    size: offset,
                })
            }
            x => Err(ParseFail::Error {
                error: ParseError::UnknownEnumVariant {
                    en: Enum::PermissionId,
                    value: x,
                },
                offset: 0,
            }),
        }
    }
}

pub mod permission_level {
    pub const USER: u8 = 0;
    pub const ROOT: u8 = 1;
    // ALP SPEC: Does something else exist?
}

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
    fn from(n: u8) -> Result<Self, ParseFail> {
        Ok(match n {
            0 => QueryComparisonType::Inequal,
            1 => QueryComparisonType::Equal,
            2 => QueryComparisonType::LessThan,
            3 => QueryComparisonType::LessThanOrEqual,
            4 => QueryComparisonType::GreaterThan,
            5 => QueryComparisonType::GreaterThanOrEqual,
            x => {
                return Err(ParseFail::Error {
                    error: ParseError::UnknownEnumVariant {
                        en: Enum::QueryComparisonType,
                        value: x,
                    },
                    offset: 0,
                })
            }
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum QueryRangeComparisonType {
    NotInRange = 0,
    InRange = 1,
}
impl QueryRangeComparisonType {
    fn from(n: u8) -> Result<Self, ParseFail> {
        Ok(match n {
            0 => QueryRangeComparisonType::NotInRange,
            1 => QueryRangeComparisonType::InRange,
            x => {
                return Err(ParseFail::Error {
                    error: ParseError::UnknownEnumVariant {
                        en: Enum::QueryRangeComparisonType,
                        value: x,
                    },
                    offset: 0,
                })
            }
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
    fn from(n: u8) -> Result<Self, ParseFail> {
        Ok(match n {
            0 => QueryCode::NonVoid,
            1 => QueryCode::ComparisonWithZero,
            2 => QueryCode::ComparisonWithValue,
            3 => QueryCode::ComparisonWithOtherFile,
            4 => QueryCode::BitmapRangeComparison,
            7 => QueryCode::StringTokenSearch,
            x => {
                return Err(ParseFail::Error {
                    error: ParseError::UnknownEnumVariant {
                        en: Enum::QueryCode,
                        value: x,
                    },
                    offset: 0,
                })
            }
        })
    }
}

pub struct NewNonVoid {
    pub size: u32,
    pub file: FileOffset,
}
impl NewNonVoid {
    pub fn build(self) -> Result<NonVoid, NonVoidError> {
        NonVoid::new(self)
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NonVoid {
    pub size: u32,
    pub file: FileOffset,
    _private: (),
}
#[derive(Clone, Debug, PartialEq)]
pub enum NonVoidError {
    SizeTooBig,
}
impl NonVoid {
    pub fn new(new: NewNonVoid) -> Result<Self, NonVoidError> {
        if new.size > varint::MAX {
            return Err(NonVoidError::SizeTooBig);
        }
        Ok(Self {
            size: new.size,
            file: new.file,
            _private: (),
        })
    }
}
impl Codec for NonVoid {
    fn encoded_size(&self) -> usize {
        1 + unsafe { varint::size(self.size) } as usize + self.file.encoded_size()
    }
    fn encode(&self, out: &mut [u8]) -> usize {
        out[0] = QueryCode::NonVoid as u8;
        let mut offset = 1;
        offset += unsafe { varint::encode(self.size, &mut out[offset..]) } as usize;
        offset += self.file.encode(&mut out[offset..]);
        offset
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
        if out.len() < 3 {
            return Err(ParseFail::MissingBytes(Some(3 - out.len())));
        }
        let mut offset = 1;
        let ParseValue {
            value: size,
            size: size_size,
        } = varint::decode(&out[offset..])?;
        offset += size_size;
        let ParseValue {
            value: file,
            size: file_size,
        } = FileOffset::decode(&out[offset..])?;
        offset += file_size;
        Ok(ParseValue {
            value: Self {
                size,
                file,
                _private: (),
            },
            size: offset,
        })
    }
}
#[test]
fn test_non_void_query_operand() {
    test_item(
        NonVoid {
            size: 4,
            file: FileOffset {
                id: 5,
                offset: 6,
                _private: (),
            },
            _private: (),
        },
        &hex!("00 04  05 06"),
    )
}

pub struct NewComparisonWithZero {
    pub signed_data: bool,
    pub comparison_type: QueryComparisonType,
    pub size: u32,
    pub mask: Option<Box<[u8]>>,
    pub file: FileOffset,
}
impl NewComparisonWithZero {
    pub fn build(self) -> Result<ComparisonWithZero, ComparisonWithZeroError> {
        ComparisonWithZero::new(self)
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct ComparisonWithZero {
    pub signed_data: bool,
    pub comparison_type: QueryComparisonType,
    pub size: u32,
    pub mask: Option<Box<[u8]>>,
    pub file: FileOffset,
    _private: (),
}
#[derive(Clone, Debug, PartialEq)]
pub enum ComparisonWithZeroError {
    SizeTooBig,
    MaskBadSize,
}
impl ComparisonWithZero {
    pub fn new(new: NewComparisonWithZero) -> Result<Self, ComparisonWithZeroError> {
        if new.size > varint::MAX {
            return Err(ComparisonWithZeroError::SizeTooBig);
        }
        if let Some(mask) = &new.mask {
            if mask.len() as u32 != new.size {
                return Err(ComparisonWithZeroError::MaskBadSize);
            }
        }
        Ok(Self {
            signed_data: new.signed_data,
            comparison_type: new.comparison_type,
            size: new.size,
            mask: new.mask,
            file: new.file,
            _private: (),
        })
    }
}
impl Codec for ComparisonWithZero {
    fn encoded_size(&self) -> usize {
        let mask_size = match self.mask {
            Some(_) => self.size as usize,
            None => 0,
        };
        1 + unsafe { varint::size(self.size) } as usize + mask_size + self.file.encoded_size()
    }
    fn encode(&self, out: &mut [u8]) -> usize {
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
        offset += unsafe { varint::encode(self.size, &mut out[offset..]) } as usize;
        if let Some(mask) = &self.mask {
            out[offset..offset + (self.size as usize)].clone_from_slice(&mask);
            offset += mask.len();
        }
        offset += self.file.encode(&mut out[offset..]);
        offset
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
        if out.len() < 1 + 1 + 2 {
            return Err(ParseFail::MissingBytes(Some(1 + 1 + 2 - out.len())));
        }
        let mask_flag = out[0] & (1 << 4) != 0;
        let signed_data = out[0] & (1 << 3) != 0;
        let comparison_type = QueryComparisonType::from(out[0] & 0x07)?;
        let ParseValue {
            value: size,
            size: size_size,
        } = varint::decode(&out[1..])?;
        let mut offset = 1 + size_size;
        let mask = if mask_flag {
            let mut data = vec![0u8; size as usize].into_boxed_slice();
            data.clone_from_slice(&out[offset..offset + size as usize]);
            offset += size as usize;
            Some(data)
        } else {
            None
        };
        let ParseValue {
            value: file,
            size: offset_size,
        } = FileOffset::decode(&out[offset..])?;
        offset += offset_size;
        Ok(ParseValue {
            value: Self {
                signed_data,
                comparison_type,
                size,
                mask,
                file,
                _private: (),
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
            file: FileOffset {
                id: 4,
                offset: 5,
                _private: (),
            },
            _private: (),
        },
        &hex!("38 03  000102  04 05"),
    )
}

pub struct NewComparisonWithValue {
    pub signed_data: bool,
    pub comparison_type: QueryComparisonType,
    pub mask: Option<Box<[u8]>>,
    pub value: Box<[u8]>,
    pub file: FileOffset,
}
impl NewComparisonWithValue {
    pub fn build(self) -> Result<ComparisonWithValue, ComparisonWithValueError> {
        ComparisonWithValue::new(self)
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct ComparisonWithValue {
    pub signed_data: bool,
    pub comparison_type: QueryComparisonType,
    pub size: u32,
    pub mask: Option<Box<[u8]>>,
    pub value: Box<[u8]>,
    pub file: FileOffset,
    _private: (),
}
#[derive(Clone, Debug, PartialEq)]
pub enum ComparisonWithValueError {
    SizeTooBig,
    MaskBadSize,
}
impl ComparisonWithValue {
    pub fn new(new: NewComparisonWithValue) -> Result<Self, ComparisonWithValueError> {
        let size = new.value.len() as u32;
        if size > varint::MAX {
            return Err(ComparisonWithValueError::SizeTooBig);
        }
        if let Some(mask) = &new.mask {
            if mask.len() as u32 != size {
                return Err(ComparisonWithValueError::MaskBadSize);
            }
        }
        Ok(Self {
            signed_data: new.signed_data,
            comparison_type: new.comparison_type,
            size,
            mask: new.mask,
            value: new.value,
            file: new.file,
            _private: (),
        })
    }
}
impl Codec for ComparisonWithValue {
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
    fn encode(&self, out: &mut [u8]) -> usize {
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
        offset += unsafe { varint::encode(self.size, &mut out[offset..]) } as usize;
        if let Some(mask) = &self.mask {
            out[offset..offset + self.size as usize].clone_from_slice(&mask);
            offset += mask.len();
        }
        out[offset..offset + self.size as usize].clone_from_slice(&self.value[..]);
        offset += self.value.len();
        offset += self.file.encode(&mut out[offset..]);
        offset
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
        if out.len() < 1 + 1 + 2 {
            return Err(ParseFail::MissingBytes(Some(1 + 1 + 2 - out.len())));
        }
        let mask_flag = out[0] & (1 << 4) != 0;
        let signed_data = out[0] & (1 << 3) != 0;
        let comparison_type = QueryComparisonType::from(out[0] & 0x07)?;
        let ParseValue {
            value: size,
            size: size_size,
        } = varint::decode(&out[1..])?;
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
        let ParseValue {
            value: file,
            size: offset_size,
        } = FileOffset::decode(&out[offset..])?;
        offset += offset_size;
        Ok(ParseValue {
            value: Self {
                signed_data,
                comparison_type,
                size,
                mask,
                value,
                file,
                _private: (),
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
            file: FileOffset {
                id: 4,
                offset: 5,
                _private: (),
            },
            _private: (),
        },
        &hex!("41 03   090909  04 05"),
    )
}

pub struct NewComparisonWithOtherFile {
    pub signed_data: bool,
    pub comparison_type: QueryComparisonType,
    pub size: u32,
    pub mask: Option<Box<[u8]>>,
    pub file1: FileOffset,
    pub file2: FileOffset,
}
impl NewComparisonWithOtherFile {
    pub fn build(self) -> Result<ComparisonWithOtherFile, ComparisonWithOtherFileError> {
        ComparisonWithOtherFile::new(self)
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct ComparisonWithOtherFile {
    pub signed_data: bool,
    pub comparison_type: QueryComparisonType,
    pub size: u32,
    pub mask: Option<Box<[u8]>>,
    pub file1: FileOffset,
    pub file2: FileOffset,
    _private: (),
}
#[derive(Clone, Debug, PartialEq)]
pub enum ComparisonWithOtherFileError {
    SizeTooBig,
    MaskBadSize,
}
impl ComparisonWithOtherFile {
    pub fn new(new: NewComparisonWithOtherFile) -> Result<Self, ComparisonWithOtherFileError> {
        if new.size > varint::MAX {
            return Err(ComparisonWithOtherFileError::SizeTooBig);
        }
        if let Some(mask) = &new.mask {
            if mask.len() as u32 != new.size {
                return Err(ComparisonWithOtherFileError::MaskBadSize);
            }
        }
        Ok(Self {
            signed_data: new.signed_data,
            comparison_type: new.comparison_type,
            size: new.size,
            mask: new.mask,
            file1: new.file1,
            file2: new.file2,
            _private: (),
        })
    }
}
impl Codec for ComparisonWithOtherFile {
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
    fn encode(&self, out: &mut [u8]) -> usize {
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
        offset += unsafe { varint::encode(self.size, &mut out[offset..]) } as usize;
        if let Some(mask) = &self.mask {
            out[offset..offset + self.size as usize].clone_from_slice(&mask);
            offset += mask.len();
        }
        offset += self.file1.encode(&mut out[offset..]);
        offset += self.file2.encode(&mut out[offset..]);
        offset
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
        if out.len() < 1 + 1 + 2 + 2 {
            return Err(ParseFail::MissingBytes(Some(1 + 1 + 2 + 2 - out.len())));
        }
        let mask_flag = out[0] & (1 << 4) != 0;
        let signed_data = out[0] & (1 << 3) != 0;
        let comparison_type = QueryComparisonType::from(out[0] & 0x07)?;
        let ParseValue {
            value: size,
            size: size_size,
        } = varint::decode(&out[1..])?;
        let mut offset = 1 + size_size;
        let mask = if mask_flag {
            let mut data = vec![0u8; size as usize].into_boxed_slice();
            data.clone_from_slice(&out[offset..offset + size as usize]);
            offset += size as usize;
            Some(data)
        } else {
            None
        };
        let ParseValue {
            value: file1,
            size: file1_size,
        } = FileOffset::decode(&out[offset..])?;
        offset += file1_size;
        let ParseValue {
            value: file2,
            size: file2_size,
        } = FileOffset::decode(&out[offset..])?;
        offset += file2_size;
        Ok(ParseValue {
            value: Self {
                signed_data,
                comparison_type,
                size,
                mask,
                file1,
                file2,
                _private: (),
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
            file1: FileOffset {
                id: 4,
                offset: 5,
                _private: (),
            },
            file2: FileOffset {
                id: 8,
                offset: 9,
                _private: (),
            },
            _private: (),
        },
        &hex!("74 02 FFFF   04 05    08 09"),
    )
}

pub struct NewBitmapRangeComparison {
    pub signed_data: bool,
    pub comparison_type: QueryRangeComparisonType,
    // TODO Is u32 a pertinent size?
    pub start: u32,
    pub stop: u32,
    pub bitmap: Box<[u8]>,
    pub file: FileOffset,
}
impl NewBitmapRangeComparison {
    pub fn build(self) -> Result<BitmapRangeComparison, BitmapRangeComparisonError> {
        BitmapRangeComparison::new(self)
    }
}
// TODO Check size coherence upon creation (start, stop and bitmap)
#[derive(Clone, Debug, PartialEq)]
pub struct BitmapRangeComparison {
    pub signed_data: bool,
    pub comparison_type: QueryRangeComparisonType,
    pub size: u32,
    // ALP SPEC: In theory, start and stop can be huge array thus impossible to cast into any trivial
    // number. How do we deal with this.
    // If the max size is ever settled by the spec, replace the buffer by the max size. This may take up more
    // memory, but would be way easier to use. Also it would avoid having to specify the ".size"
    // field.
    pub start: Box<[u8]>,
    pub stop: Box<[u8]>,
    pub bitmap: Box<[u8]>,
    pub file: FileOffset,
    _private: (),
}
#[derive(Clone, Debug, PartialEq)]
pub enum BitmapRangeComparisonError {
    StartGreaterThanStop,
    SizeTooBig,
    BitmapBadSize,
}
impl BitmapRangeComparison {
    pub fn new(new: NewBitmapRangeComparison) -> Result<Self, BitmapRangeComparisonError> {
        if new.start > new.stop {
            return Err(BitmapRangeComparisonError::StartGreaterThanStop);
        }
        let max = new.stop;
        let size: u32 = if max <= 0xFF {
            1
        } else if max <= 0xFF_FF {
            2
        } else if max <= 0xFF_FF_FF {
            3
        } else {
            4
        };
        let mut start = vec![0u8; size as usize].into_boxed_slice();
        start.clone_from_slice(&new.start.to_be_bytes());
        let mut stop = vec![0u8; size as usize].into_boxed_slice();
        stop.clone_from_slice(&new.stop.to_be_bytes());

        let bitmap_size = (new.stop - new.start + 6) / 8; // ALP SPEC: Thanks for the calculation
        if new.bitmap.len() != bitmap_size as usize {
            return Err(BitmapRangeComparisonError::BitmapBadSize);
        }
        Ok(Self {
            signed_data: new.signed_data,
            comparison_type: new.comparison_type,
            size,
            start,
            stop,
            bitmap: new.bitmap,
            file: new.file,
            _private: (),
        })
    }
}
impl Codec for BitmapRangeComparison {
    fn encoded_size(&self) -> usize {
        1 + unsafe { varint::size(self.size) } as usize
            + 2 * self.size as usize
            + self.bitmap.len()
            + self.file.encoded_size()
    }
    fn encode(&self, out: &mut [u8]) -> usize {
        let mut offset = 0;
        let signed_flag = if self.signed_data { 1 } else { 0 };
        out[0] = ((QueryCode::BitmapRangeComparison as u8) << 5)
            // | (0 << 4)
            | (signed_flag << 3)
            | self.comparison_type as u8;
        offset += 1;
        offset += unsafe { varint::encode(self.size, &mut out[offset..]) } as usize;
        out[offset..offset + self.size as usize].clone_from_slice(&self.start[..]);
        offset += self.start.len();
        out[offset..offset + self.size as usize].clone_from_slice(&self.stop[..]);
        offset += self.stop.len();
        out[offset..offset + self.bitmap.len()].clone_from_slice(&self.bitmap[..]);
        offset += self.bitmap.len();
        offset += self.file.encode(&mut out[offset..]);
        offset
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
        if out.len() < 1 + 1 + 2 {
            return Err(ParseFail::MissingBytes(Some(1 + 1 + 2 - out.len())));
        }
        let signed_data = out[0] & (1 << 3) != 0;
        let comparison_type = QueryRangeComparisonType::from(out[0] & 0x07)?;
        let ParseValue {
            value: size32,
            size: size_size,
        } = varint::decode(&out[1..])?;
        let size = size32 as usize;
        let mut offset = 1 + size_size;
        let mut start = vec![0u8; size].into_boxed_slice();
        start.clone_from_slice(&out[offset..offset + size]);
        offset += size;
        let mut stop = vec![0u8; size].into_boxed_slice();
        stop.clone_from_slice(&out[offset..offset + size]);
        offset += size;
        // TODO Current max start/stop size chosen is u32 because that is the file size limit.
        // But in theory there is no requirement for the bitmap to have any relation with the
        // file sizes. So this might panic if you download your amazon bluerays over ALP.
        let mut start_n = 0u32;
        let mut stop_n = 0u32;
        for i in 0..size {
            start_n = (start_n << 8) + start[i] as u32;
            stop_n = (stop_n << 8) + stop[i] as u32;
        }
        let bitmap_size = (stop_n - start_n + 6) / 8; // ALP SPEC: Thanks for the calculation
        let mut bitmap = vec![0u8; bitmap_size as usize].into_boxed_slice();
        bitmap.clone_from_slice(&out[offset..offset + bitmap_size as usize]);
        offset += bitmap_size as usize;
        let ParseValue {
            value: file,
            size: file_size,
        } = FileOffset::decode(&out[offset..])?;
        offset += file_size;
        Ok(ParseValue {
            value: Self {
                signed_data,
                comparison_type,
                size: size32,
                start,
                stop,
                bitmap,
                file,
                _private: (),
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

            start: Box::new(hex!("00 03")),
            stop: Box::new(hex!("00 20")),
            bitmap: Box::new(hex!("01020304")),

            file: FileOffset {
                id: 0,
                offset: 4,
                _private: (),
            },
            _private: (),
        },
        &hex!("81 02 0003  0020  01020304  00 04"),
    )
}

pub struct NewStringTokenSearch {
    pub max_errors: u8,
    pub mask: Option<Box<[u8]>>,
    pub value: Box<[u8]>,
    pub file: FileOffset,
}
impl NewStringTokenSearch {
    pub fn build(self) -> Result<StringTokenSearch, StringTokenSearchError> {
        StringTokenSearch::new(self)
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct StringTokenSearch {
    pub max_errors: u8,
    pub size: u32,
    pub mask: Option<Box<[u8]>>,
    pub value: Box<[u8]>,
    pub file: FileOffset,
    _private: (),
}
#[derive(Clone, Debug, PartialEq)]
pub enum StringTokenSearchError {
    SizeTooBig,
    MaskBadSize,
}
impl StringTokenSearch {
    pub fn new(new: NewStringTokenSearch) -> Result<Self, StringTokenSearchError> {
        let size = new.value.len() as u32;
        if size > varint::MAX {
            return Err(StringTokenSearchError::SizeTooBig);
        }
        if let Some(mask) = &new.mask {
            if mask.len() as u32 != size {
                return Err(StringTokenSearchError::MaskBadSize);
            }
        }
        Ok(Self {
            max_errors: new.max_errors,
            size,
            mask: new.mask,
            value: new.value,
            file: new.file,
            _private: (),
        })
    }
}
impl Codec for StringTokenSearch {
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
    fn encode(&self, out: &mut [u8]) -> usize {
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
        offset += unsafe { varint::encode(self.size, &mut out[offset..]) } as usize;
        if let Some(mask) = &self.mask {
            out[offset..offset + self.size as usize].clone_from_slice(&mask);
            offset += mask.len();
        }
        out[offset..offset + self.size as usize].clone_from_slice(&self.value[..]);
        offset += self.value.len();
        offset += self.file.encode(&mut out[offset..]);
        offset
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
        if out.len() < 1 + 1 + 2 {
            return Err(ParseFail::MissingBytes(Some(1 + 1 + 2 - out.len())));
        }
        let mask_flag = out[0] & (1 << 4) != 0;
        let max_errors = out[0] & 0x07;
        let ParseValue {
            value: size32,
            size: size_size,
        } = varint::decode(&out[1..])?;
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
        let ParseValue {
            value: file,
            size: offset_size,
        } = FileOffset::decode(&out[offset..])?;
        offset += offset_size;
        Ok(ParseValue {
            value: Self {
                max_errors,
                size: size32,
                mask,
                value,
                file,
                _private: (),
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
            file: FileOffset {
                id: 0,
                offset: 4,
                _private: (),
            },
            _private: (),
        },
        &hex!("F2 04 FF00FF00  01020304  00 04"),
    )
}

#[derive(Clone, Debug, PartialEq)]
pub enum Query {
    NonVoid(NonVoid),
    ComparisonWithZero(ComparisonWithZero),
    ComparisonWithValue(ComparisonWithValue),
    ComparisonWithOtherFile(ComparisonWithOtherFile),
    BitmapRangeComparison(BitmapRangeComparison),
    StringTokenSearch(StringTokenSearch),
}
impl Codec for Query {
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
    fn encode(&self, out: &mut [u8]) -> usize {
        match self {
            Query::NonVoid(v) => v.encode(out),
            Query::ComparisonWithZero(v) => v.encode(out),
            Query::ComparisonWithValue(v) => v.encode(out),
            Query::ComparisonWithOtherFile(v) => v.encode(out),
            Query::BitmapRangeComparison(v) => v.encode(out),
            Query::StringTokenSearch(v) => v.encode(out),
        }
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
        Ok(match QueryCode::from(out[0] >> 5)? {
            QueryCode::NonVoid => NonVoid::decode(out).map(|ok| ok.map_value(Query::NonVoid)),
            QueryCode::ComparisonWithZero => {
                ComparisonWithZero::decode(out).map(|ok| ok.map_value(Query::ComparisonWithZero))
            }
            QueryCode::ComparisonWithValue => {
                ComparisonWithValue::decode(out).map(|ok| ok.map_value(Query::ComparisonWithValue))
            }
            QueryCode::ComparisonWithOtherFile => ComparisonWithOtherFile::decode(out)
                .map(|ok| ok.map_value(Query::ComparisonWithOtherFile)),
            QueryCode::BitmapRangeComparison => BitmapRangeComparison::decode(out)
                .map(|ok| ok.map_value(Query::BitmapRangeComparison)),
            QueryCode::StringTokenSearch => {
                StringTokenSearch::decode(out).map(|ok| ok.map_value(Query::StringTokenSearch))
            }
        }?)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct OverloadedIndirectInterface {
    pub interface_file_id: u8,
    pub addressee: dash7::Addressee,
}

impl Codec for OverloadedIndirectInterface {
    fn encoded_size(&self) -> usize {
        1 + self.addressee.encoded_size()
    }
    fn encode(&self, out: &mut [u8]) -> usize {
        out[0] = self.interface_file_id;
        1 + self.addressee.encode(&mut out[1..])
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
        if out.len() < 1 + 2 {
            return Err(ParseFail::MissingBytes(Some(1 + 2 - out.len())));
        }
        let interface_file_id = out[0];
        let ParseValue {
            value: addressee,
            size: addressee_size,
        } = dash7::Addressee::decode(&out[1..]).inc_offset(1)?;
        Ok(ParseValue {
            value: Self {
                interface_file_id,
                addressee,
            },
            size: 1 + addressee_size,
        })
    }
}
#[test]
fn test_overloaded_indirect_interface() {
    test_item(
        OverloadedIndirectInterface {
            interface_file_id: 4,
            addressee: dash7::Addressee {
                nls_method: dash7::NlsMethod::AesCcm32,
                access_class: 0xFF,
                address: dash7::Address::Vid(Box::new([0xAB, 0xCD])),
            },
        },
        &hex!("04   37 FF ABCD"),
    )
}

#[derive(Clone, Debug, PartialEq)]
// ALP SPEC: This seems undoable if we do not know the interface (per protocol specific support)
//  which is still a pretty legitimate policy on a low power protocol.
pub struct NonOverloadedIndirectInterface {
    pub interface_file_id: u8,
    // ALP SPEC: Where is this defined? Is this ID specific?
    pub data: Box<[u8]>,
}

impl Codec for NonOverloadedIndirectInterface {
    fn encoded_size(&self) -> usize {
        1 + self.data.len()
    }
    fn encode(&self, out: &mut [u8]) -> usize {
        out[0] = self.interface_file_id;
        let mut offset = 1;
        out[offset..offset + self.data.len()].clone_from_slice(&self.data);
        offset += self.data.len();
        offset
    }
    fn decode(_out: &[u8]) -> ParseResult<Self> {
        todo!("TODO")
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum IndirectInterface {
    Overloaded(OverloadedIndirectInterface),
    NonOverloaded(NonOverloadedIndirectInterface),
}

impl Codec for IndirectInterface {
    fn encoded_size(&self) -> usize {
        match self {
            IndirectInterface::Overloaded(v) => v.encoded_size(),
            IndirectInterface::NonOverloaded(v) => v.encoded_size(),
        }
    }
    fn encode(&self, out: &mut [u8]) -> usize {
        match self {
            IndirectInterface::Overloaded(v) => v.encode(out),
            IndirectInterface::NonOverloaded(v) => v.encode(out),
        }
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
        if out.is_empty() {
            return Err(ParseFail::MissingBytes(Some(1)));
        }
        Ok(if out[0] & 0x80 != 0 {
            OverloadedIndirectInterface::decode(&out[1..])?
                .map(|v, i| (IndirectInterface::Overloaded(v), i + 1))
        } else {
            NonOverloadedIndirectInterface::decode(&out[1..])?
                .map(|v, i| (IndirectInterface::NonOverloaded(v), i + 1))
        })
    }
}

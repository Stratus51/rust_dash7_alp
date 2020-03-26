#[cfg(test)]
use hex_literal::hex;

mod codec;
pub use codec::Codec;
pub use codec::{ParseError, ParseFail, ParseResult, ParseResultExtension, ParseValue};

mod test_tools;
#[cfg(test)]
use test_tools::test_item;

pub mod varint;

mod data_element;
pub use data_element::*;

mod dash7;
pub use dash7::*;

mod operand;
pub use operand::*;

// TODO Document int Enum values meanings (Error & Spec enums)
// TODO Split this file into more pertinent submodules and choose a better naming convention
//      (if possible, making use of the module names).
//      Also organise modules by internal section because it is a labyrinth.
// TODO Document each item with its specification

// TODO Look into const function to replace some macros?
// TODO Use uninitialized memory where possible
// TODO Int enums: fn from(): find a way to avoid double value definition
// TODO Int enums: optim: find a way to cast from int to enum instead of calling a matching
// function (much more resource intensive). Only do that for enums that match all possible
// values that result from the parsing.
// TODO Optimize min size calculation (fold it into the upper OP when possible)
// TODO usize is target dependent. In other words, on a 16 bit processor, we will run into
// troubles if we were to convert u32 to usize (even if a 64Ko payload seems a bit big).
// Maybe we should just embrace this limitation? (Not to be lazy or anything...)
// The bad thing is that u32 to u16 will compile and panic at runtime if the value is too big.
// TODO Slice copies still check length consistency dynamically. Is there a way to get rid of that
// at runtime while still testing it at compile/test time?
//      - For simple index access, get_unchecked_mut can do the trick. But It makes the code hard to
//      read...
// TODO is {out = &out[offset..]; out[..size]} more efficient than {out[offset..offset+size]} ?
// TODO Add function to encode without having to define a temporary structure

// ===============================================================================
// Macros
// ===============================================================================
macro_rules! serialize_all {
    ($out: expr, $($x: expr),*) => {
        {
            let mut offset = 0;
            $({
                offset += $x.encode(&mut $out[offset..]);
            })*
            offset
        }
    }
}

macro_rules! encoded_size {
    ( $($x: expr),* ) => {
        {
            let mut total = 0;
            $({
                total += $x.encoded_size();
            })*
            total
        }
    }
}

macro_rules! control_byte {
    ($flag7: expr, $flag6: expr, $op_code: expr) => {{
        let mut ctrl = $op_code as u8;
        if $flag7 {
            ctrl |= 0x80;
        }
        if $flag6 {
            ctrl |= 0x40;
        }
        ctrl
    }};
}

macro_rules! impl_op_serialized {
    ($name: ident, $flag7: ident, $flag6: ident, $op1: ident, $op1_type: ident) => {
        impl Codec for $name {
            fn encoded_size(&self) -> usize {
                1 + encoded_size!(self.$op1)
            }
            fn encode(&self, out: &mut [u8]) -> usize {
                out[0] = control_byte!(self.$flag7, self.$flag6, OpCode::$name);
                1 + serialize_all!(&mut out[1..], &self.$op1)
            }
            fn decode(out: &[u8]) -> ParseResult<Self> {
                if (out.is_empty()) {
                    Err(ParseFail::MissingBytes(Some(1)))
                } else {
                    let mut offset = 1;
                    let ParseValue {
                        value: op1,
                        size: op1_size,
                    } = $op1_type::decode(&out[offset..]).inc_offset(offset)?;
                    offset += op1_size;
                    Ok(ParseValue {
                        value: Self {
                            $flag6: out[0] & 0x40 != 0,
                            $flag7: out[0] & 0x80 != 0,
                            $op1: op1,
                        },
                        size: offset,
                    })
                }
            }
        }
    };
}

macro_rules! unsafe_varint_serialize_sizes {
    ( $($x: expr),* ) => {{
        let mut ret = 0;
            $(unsafe {
                ret += varint::size($x);
            })*
        ret
    }}
}

macro_rules! unsafe_varint_serialize {
    ($out: expr, $($x: expr),*) => {
        {
            let mut offset: usize = 0;
            $(unsafe {
                offset += varint::encode($x, &mut $out[offset..]) as usize;
            })*
            offset
        }
    }
}

macro_rules! count {
    () => (0usize);
    ( $x:tt $($xs:tt)* ) => (1usize + count!($($xs)*));
}

macro_rules! build_simple_op {
    ($name: ident, $out: expr, $flag7: ident, $flag6: ident, $x1: ident, $x2: ident) => {
        $name {
            $flag6: $out[0] & 0x40 != 0,
            $flag7: $out[0] & 0x80 != 0,
            $x1: $out[1],
            $x2: $out[2],
        }
    };
    ($name: ident, $out: expr, $flag7: ident, $flag6: ident, $x: ident) => {
        $name {
            $flag6: $out[0] & 0x40 != 0,
            $flag7: $out[0] & 0x80 != 0,
            $x: $out[1],
        }
    };
}

macro_rules! impl_simple_op {
    ($name: ident, $flag7: ident, $flag6: ident, $($x: ident),* ) => {
        impl Codec for $name {
            fn encoded_size(&self) -> usize {
                1 + count!($( $x )*)
            }
            fn encode(&self, out: &mut [u8]) -> usize {
                out[0] = control_byte!(self.$flag7, self.$flag6, OpCode::$name);
                let mut offset = 1;
                $({
                    out[offset] = self.$x;
                    offset += 1;
                })*
                1 + offset
            }
            fn decode(out: &[u8]) -> ParseResult<Self> {
                const SIZE: usize = 1 + count!($( $x )*);
                if(out.len() < SIZE) {
                    Err(ParseFail::MissingBytes(Some(SIZE - out.len())))
                } else {
                    Ok(ParseValue {
                        value: build_simple_op!($name, out, $flag7, $flag6, $($x),*),
                        size: SIZE,
                    })
                }
            }
        }
    };
}

macro_rules! impl_header_op {
    ($name: ident, $flag7: ident, $flag6: ident, $file_id: ident, $file_header: ident) => {
        impl Codec for $name {
            fn encoded_size(&self) -> usize {
                1 + 1 + 12
            }
            fn encode(&self, out: &mut [u8]) -> usize {
                out[0] = control_byte!(self.group, self.resp, OpCode::$name);
                out[1] = self.file_id;
                let mut offset = 2;
                offset += self.$file_header.encode(&mut out[offset..]);
                offset
            }
            fn decode(out: &[u8]) -> ParseResult<Self> {
                const SIZE: usize = 1 + 1 + 12;
                if (out.len() < SIZE) {
                    Err(ParseFail::MissingBytes(Some(SIZE - out.len())))
                } else {
                    let ParseValue { value: header, .. } =
                        FileHeader::decode(&out[2..]).inc_offset(2)?;
                    Ok(ParseValue {
                        value: Self {
                            $flag6: out[0] & 0x40 != 0,
                            $flag7: out[0] & 0x80 != 0,
                            $file_id: out[1],
                            $file_header: header,
                        },
                        size: SIZE,
                    })
                }
            }
        }
    };
}

// ===============================================================================
// Definitions
// ===============================================================================
#[derive(Clone, Debug, PartialEq)]
pub enum Enum {
    OpCode,
    NlsMethod,
    RetryMode,
    RespMode,
    InterfaceId,
    PermissionId,
    PermissionLevel,
    QueryComparisonType,
    QueryRangeComparisonType,
    QueryCode,
    StatusType,
    ActionCondition,
}

// ===============================================================================
// Opcodes
// ===============================================================================
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OpCode {
    // Nop
    Nop = 0,

    // Read
    ReadFileData = 1,
    ReadFileProperties = 2,

    // Write
    WriteFileData = 4,
    // ALP SPEC: This is out of spec. Can't write + flush already do that job. Is it worth
    //  saving 2 bytes by taking an opcode?
    // WriteFileDataFlush = 5,
    WriteFileProperties = 6,
    ActionQuery = 8,
    BreakQuery = 9,
    PermissionRequest = 10,
    VerifyChecksum = 11,

    // Management
    ExistFile = 16,
    CreateNewFile = 17,
    DeleteFile = 18,
    RestoreFile = 19,
    FlushFile = 20,
    CopyFile = 23,
    ExecuteFile = 31,

    // Response
    ReturnFileData = 32,
    ReturnFileProperties = 33,
    Status = 34,
    ResponseTag = 35,

    // Special
    Chunk = 48,
    Logic = 49,
    Forward = 50,
    IndirectForward = 51,
    RequestTag = 52,
    Extension = 63,
}
impl OpCode {
    fn from(n: u8) -> Result<Self, ParseFail> {
        Ok(match n {
            // Nop
            0 => OpCode::Nop,

            // Read
            1 => OpCode::ReadFileData,
            2 => OpCode::ReadFileProperties,

            // Write
            4 => OpCode::WriteFileData,
            6 => OpCode::WriteFileProperties,
            8 => OpCode::ActionQuery,
            9 => OpCode::BreakQuery,
            10 => OpCode::PermissionRequest,
            11 => OpCode::VerifyChecksum,

            // Management
            16 => OpCode::ExistFile,
            17 => OpCode::CreateNewFile,
            18 => OpCode::DeleteFile,
            19 => OpCode::RestoreFile,
            20 => OpCode::FlushFile,
            23 => OpCode::CopyFile,
            31 => OpCode::ExecuteFile,

            // Response
            32 => OpCode::ReturnFileData,
            33 => OpCode::ReturnFileProperties,
            34 => OpCode::Status,
            35 => OpCode::ResponseTag,

            // Special
            48 => OpCode::Chunk,
            49 => OpCode::Logic,
            50 => OpCode::Forward,
            51 => OpCode::IndirectForward,
            52 => OpCode::RequestTag,
            63 => OpCode::Extension,

            // On unknown OpCode return an error
            x => {
                return Err(ParseFail::Error {
                    error: ParseError::UnknownEnumVariant {
                        en: Enum::OpCode,
                        value: x,
                    },
                    offset: 0,
                })
            }
        })
    }
}

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
    D7asp(D7aspInterfaceConfiguration),
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
                    D7aspInterfaceConfiguration::decode(&out[1..]).inc_offset(1)?;
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
        InterfaceConfiguration::D7asp(D7aspInterfaceConfiguration {
            qos: Qos {
                retry: RetryMode::No,
                resp: RespMode::Any,
            },
            to: 0x23,
            te: 0x34,
            addressee: Addressee {
                nls_method: NlsMethod::AesCcm32,
                access_class: 0xFF,
                address: Address::Vid(Box::new([0xAB, 0xCD])),
            },
        }),
        &hex!("D7   02 23 34   37 FF ABCD"),
    )
}
#[test]
fn test_interface_configuration_host() {
    test_item(InterfaceConfiguration::Host, &hex!("00"))
}

pub struct InterfaceStatusNew {
    pub id: u8,
    pub data: Box<[u8]>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct InterfaceStatusUnknown {
    pub id: u8,
    pub data: Box<[u8]>,
    _private: (),
}
pub enum InterfaceStatusUnknownError {
    DataTooBig,
}
impl InterfaceStatusUnknown {
    pub fn new(new: InterfaceStatusNew) -> Result<Self, InterfaceStatusUnknownError> {
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
    D7asp(D7aspInterfaceStatus),
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
                    D7aspInterfaceStatus::decode(&out[offset..offset + size]).inc_offset(offset)?;
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
            D7aspInterfaceStatusNew {
                ch_header: 1,
                ch_idx: 0x0123,
                rxlev: 2,
                lb: 3,
                snr: 4,
                status: 5,
                token: 6,
                seq: 7,
                resp_to: 8,
                addressee: Addressee {
                    nls_method: NlsMethod::AesCcm32,
                    access_class: 0xFF,
                    address: Address::Vid(Box::new([0xAB, 0xCD])),
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
// Actions
// ===============================================================================
// Nop
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Nop {
    pub group: bool,
    pub resp: bool,
}
impl Codec for Nop {
    fn encoded_size(&self) -> usize {
        1
    }
    fn encode(&self, out: &mut [u8]) -> usize {
        out[0] = control_byte!(self.group, self.resp, OpCode::Nop);
        1
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
        if out.is_empty() {
            Err(ParseFail::MissingBytes(Some(1)))
        } else {
            Ok(ParseValue {
                value: Self {
                    resp: out[0] & 0x40 != 0,
                    group: out[0] & 0x80 != 0,
                },
                size: 1,
            })
        }
    }
}
#[test]
fn test_nop() {
    test_item(
        Nop {
            group: true,
            resp: false,
        },
        &hex!("80"),
    )
}

// Read
pub struct ReadFileDataNew {
    pub group: bool,
    pub resp: bool,
    pub file_id: u8,
    pub offset: u32,
    pub size: u32,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReadFileData {
    pub group: bool,
    pub resp: bool,
    pub file_id: u8,
    pub offset: u32,
    pub size: u32,
    _private: (),
}
pub enum ReadFileDataError {
    OffsetTooBig,
    SizeTooBig,
}
impl ReadFileData {
    pub fn new(new: ReadFileDataNew) -> Result<Self, ReadFileDataError> {
        if new.offset > varint::MAX {
            return Err(ReadFileDataError::OffsetTooBig);
        }
        if new.size > varint::MAX {
            return Err(ReadFileDataError::SizeTooBig);
        }
        Ok(Self {
            group: new.group,
            resp: new.resp,
            file_id: new.file_id,
            offset: new.offset,
            size: new.size,
            _private: (),
        })
    }
}

impl Codec for ReadFileData {
    fn encoded_size(&self) -> usize {
        1 + 1 + unsafe_varint_serialize_sizes!(self.offset, self.size) as usize
    }
    fn encode(&self, out: &mut [u8]) -> usize {
        out[0] = control_byte!(self.group, self.resp, OpCode::ReadFileData);
        out[1] = self.file_id;
        1 + 1 + unsafe_varint_serialize!(out[2..], self.offset, self.size)
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
        let min_size = 1 + 1 + 1 + 1;
        if out.len() < min_size {
            return Err(ParseFail::MissingBytes(Some(min_size - out.len())));
        }
        let group = out[0] & 0x80 != 0;
        let resp = out[0] & 0x40 != 0;
        let file_id = out[1];
        let mut off = 2;
        let ParseValue {
            value: offset,
            size: offset_size,
        } = varint::decode(&out[off..])?;
        off += offset_size;
        let ParseValue {
            value: size,
            size: size_size,
        } = varint::decode(&out[off..])?;
        off += size_size;
        Ok(ParseValue {
            value: Self {
                group,
                resp,
                file_id,
                offset,
                size,
                _private: (),
            },
            size: off,
        })
    }
}
#[test]
fn test_read_file_data() {
    test_item(
        ReadFileData {
            group: false,
            resp: true,
            file_id: 1,
            offset: 2,
            size: 3,
            _private: (),
        },
        &hex!("41 01 02 03"),
    )
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReadFileProperties {
    pub group: bool,
    pub resp: bool,
    pub file_id: u8,
}
impl_simple_op!(ReadFileProperties, group, resp, file_id);
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

// Write
pub struct WriteFileDataNew {
    pub group: bool,
    pub resp: bool,
    pub file_id: u8,
    pub offset: u32,
    pub data: Box<[u8]>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct WriteFileData {
    pub group: bool,
    pub resp: bool,
    pub file_id: u8,
    pub offset: u32,
    pub data: Box<[u8]>,
    _private: (),
}
pub enum WriteFileDataError {
    OffsetTooBig,
    SizeTooBig,
}
impl WriteFileData {
    pub fn new(new: WriteFileDataNew) -> Result<Self, WriteFileDataError> {
        if new.offset > varint::MAX {
            return Err(WriteFileDataError::OffsetTooBig);
        }
        let size = new.data.len() as u32;
        if size > varint::MAX {
            return Err(WriteFileDataError::SizeTooBig);
        }
        Ok(Self {
            group: new.group,
            resp: new.resp,
            file_id: new.file_id,
            offset: new.offset,
            data: new.data,
            _private: (),
        })
    }
}
impl Codec for WriteFileData {
    fn encoded_size(&self) -> usize {
        1 + 1
            + unsafe_varint_serialize_sizes!(self.offset, self.data.len() as u32) as usize
            + self.data.len()
    }
    fn encode(&self, out: &mut [u8]) -> usize {
        out[0] = control_byte!(self.group, self.resp, OpCode::WriteFileData);
        out[1] = self.file_id;
        let mut offset = 2;
        offset += unsafe_varint_serialize!(out[2..], self.offset, self.data.len() as u32) as usize;
        out[offset..offset + self.data.len()].clone_from_slice(&self.data[..]);
        offset += self.data.len();
        offset
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
        let min_size = 1 + 1 + 1 + 1;
        if out.len() < min_size {
            return Err(ParseFail::MissingBytes(Some(min_size - out.len())));
        }
        let group = out[0] & 0x80 != 0;
        let resp = out[0] & 0x40 != 0;
        let file_id = out[1];
        let mut off = 2;
        let ParseValue {
            value: offset,
            size: offset_size,
        } = varint::decode(&out[off..])?;
        off += offset_size;
        let ParseValue {
            value: size,
            size: size_size,
        } = varint::decode(&out[off..])?;
        off += size_size;
        let size = size as usize;
        let mut data = vec![0u8; size].into_boxed_slice();
        data.clone_from_slice(&out[off..off + size]);
        off += size;
        Ok(ParseValue {
            value: Self {
                group,
                resp,
                file_id,
                offset,
                data,
                _private: (),
            },
            size: off,
        })
    }
}
#[test]
fn test_write_file_data() {
    test_item(
        WriteFileData {
            group: true,
            resp: false,
            file_id: 9,
            offset: 5,
            data: Box::new(hex!("01 02 03")),
            _private: (),
        },
        &hex!("84   09 05 03  010203"),
    )
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WriteFileProperties {
    pub group: bool,
    pub resp: bool,
    pub file_id: u8,
    pub header: FileHeader,
}
impl_header_op!(WriteFileProperties, group, resp, file_id, header);
#[test]
fn test_write_file_properties() {
    test_item(
        WriteFileProperties {
            group: true,
            resp: false,
            file_id: 9,
            header: FileHeader {
                permissions: Permissions {
                    encrypted: true,
                    executable: false,
                    user: UserPermissions {
                        read: true,
                        write: true,
                        run: true,
                    },
                    guest: UserPermissions {
                        read: false,
                        write: false,
                        run: false,
                    },
                },
                properties: FileProperties {
                    act_en: false,
                    act_cond: ActionCondition::Read,
                    storage_class: StorageClass::Permanent,
                },
                alp_cmd_fid: 1,
                interface_file_id: 2,
                file_size: 0xDEAD_BEEF,
                allocated_size: 0xBAAD_FACE,
            },
        },
        &hex!("86   09   B8 13 01 02 DEADBEEF BAADFACE"),
    )
}

#[derive(Clone, Debug, PartialEq)]
pub struct ActionQuery {
    pub group: bool,
    pub resp: bool,
    pub query: QueryOperand,
}
impl_op_serialized!(ActionQuery, group, resp, query, QueryOperand);
#[test]
fn test_action_query() {
    test_item(
        ActionQuery {
            group: true,
            resp: true,
            query: QueryOperand::NonVoid(
                NonVoidNew {
                    size: 4,
                    file: FileOffsetOperandNew { id: 5, offset: 6 }.build().unwrap(),
                }
                .build()
                .unwrap(),
            ),
        },
        &hex!("C8   00 04  05 06"),
    )
}

#[derive(Clone, Debug, PartialEq)]
pub struct BreakQuery {
    pub group: bool,
    pub resp: bool,
    pub query: QueryOperand,
}
impl_op_serialized!(BreakQuery, group, resp, query, QueryOperand);
#[test]
fn test_break_query() {
    test_item(
        BreakQuery {
            group: true,
            resp: true,
            query: QueryOperand::NonVoid(
                NonVoidNew {
                    size: 4,
                    file: FileOffsetOperandNew { id: 5, offset: 6 }.build().unwrap(),
                }
                .build()
                .unwrap(),
            ),
        },
        &hex!("C9   00 04  05 06"),
    )
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PermissionRequest {
    pub group: bool,
    pub resp: bool,
    pub level: u8,
    pub permission: Permission,
}
impl Codec for PermissionRequest {
    fn encoded_size(&self) -> usize {
        1 + 1 + encoded_size!(self.permission)
    }
    fn encode(&self, out: &mut [u8]) -> usize {
        out[0] = control_byte!(self.group, self.resp, OpCode::PermissionRequest);
        out[1] = self.level;
        1 + serialize_all!(&mut out[2..], self.permission)
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
        if out.is_empty() {
            Err(ParseFail::MissingBytes(Some(1)))
        } else {
            let mut offset = 1;
            let level = out[offset];
            offset += 1;
            let ParseValue {
                value: permission,
                size,
            } = Permission::decode(&out[offset..]).inc_offset(offset)?;
            offset += size;
            Ok(ParseValue {
                value: Self {
                    group: out[0] & 0x80 != 0,
                    resp: out[0] & 0x40 != 0,
                    level,
                    permission,
                },
                size: offset,
            })
        }
    }
}
#[test]
fn test_permission_request() {
    test_item(
        PermissionRequest {
            group: false,
            resp: false,
            level: permission_level::ROOT,
            permission: Permission::Dash7(hex!("0102030405060708")),
        },
        &hex!("0A   01 42 0102030405060708"),
    )
}

#[derive(Clone, Debug, PartialEq)]
pub struct VerifyChecksum {
    pub group: bool,
    pub resp: bool,
    pub query: QueryOperand,
}
impl_op_serialized!(VerifyChecksum, group, resp, query, QueryOperand);
#[test]
fn test_verify_checksum() {
    test_item(
        VerifyChecksum {
            group: false,
            resp: false,
            query: QueryOperand::NonVoid(
                NonVoidNew {
                    size: 4,
                    file: FileOffsetOperandNew { id: 5, offset: 6 }.build().unwrap(),
                }
                .build()
                .unwrap(),
            ),
        },
        &hex!("0B   00 04  05 06"),
    )
}

// Management
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExistFile {
    pub group: bool,
    pub resp: bool,
    pub file_id: u8,
}
impl_simple_op!(ExistFile, group, resp, file_id);
#[test]
fn test_exist_file() {
    test_item(
        ExistFile {
            group: false,
            resp: false,
            file_id: 9,
        },
        &hex!("10 09"),
    )
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CreateNewFile {
    pub group: bool,
    pub resp: bool,
    pub file_id: u8,
    pub header: FileHeader,
}
impl_header_op!(CreateNewFile, group, resp, file_id, header);
#[test]
fn test_create_new_file() {
    test_item(
        CreateNewFile {
            group: true,
            resp: false,
            file_id: 3,
            header: FileHeader {
                permissions: Permissions {
                    encrypted: true,
                    executable: false,
                    user: UserPermissions {
                        read: true,
                        write: true,
                        run: true,
                    },
                    guest: UserPermissions {
                        read: false,
                        write: false,
                        run: false,
                    },
                },
                properties: FileProperties {
                    act_en: false,
                    act_cond: ActionCondition::Read,
                    storage_class: StorageClass::Permanent,
                },
                alp_cmd_fid: 1,
                interface_file_id: 2,
                file_size: 0xDEAD_BEEF,
                allocated_size: 0xBAAD_FACE,
            },
        },
        &hex!("91   03   B8 13 01 02 DEADBEEF BAADFACE"),
    )
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DeleteFile {
    pub group: bool,
    pub resp: bool,
    pub file_id: u8,
}
impl_simple_op!(DeleteFile, group, resp, file_id);
#[test]
fn test_delete_file() {
    test_item(
        DeleteFile {
            group: false,
            resp: true,
            file_id: 9,
        },
        &hex!("52 09"),
    )
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RestoreFile {
    pub group: bool,
    pub resp: bool,
    pub file_id: u8,
}
impl_simple_op!(RestoreFile, group, resp, file_id);
#[test]
fn test_restore_file() {
    test_item(
        RestoreFile {
            group: true,
            resp: true,
            file_id: 9,
        },
        &hex!("D3 09"),
    )
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlushFile {
    pub group: bool,
    pub resp: bool,
    pub file_id: u8,
}
impl_simple_op!(FlushFile, group, resp, file_id);
#[test]
fn test_flush_file() {
    test_item(
        FlushFile {
            group: false,
            resp: false,
            file_id: 9,
        },
        &hex!("14 09"),
    )
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CopyFile {
    pub group: bool,
    pub resp: bool,
    pub src_file_id: u8,
    pub dst_file_id: u8,
}
impl_simple_op!(CopyFile, group, resp, src_file_id, dst_file_id);
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExecuteFile {
    pub group: bool,
    pub resp: bool,
    pub file_id: u8,
}
impl_simple_op!(ExecuteFile, group, resp, file_id);
#[test]
fn test_execute_file() {
    test_item(
        ExecuteFile {
            group: false,
            resp: false,
            file_id: 9,
        },
        &hex!("1F 09"),
    )
}

// Response
pub struct ReturnFileDataNew {
    pub group: bool,
    pub resp: bool,
    pub file_id: u8,
    pub offset: u32,
    pub data: Box<[u8]>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct ReturnFileData {
    pub group: bool,
    pub resp: bool,
    pub file_id: u8,
    pub offset: u32,
    pub data: Box<[u8]>,
    _private: (),
}
pub enum ReturnFileDataError {
    OffsetTooBig,
    SizeTooBig,
}
impl ReturnFileData {
    pub fn new(new: ReturnFileDataNew) -> Result<Self, ReturnFileDataError> {
        if new.offset > varint::MAX {
            return Err(ReturnFileDataError::OffsetTooBig);
        }
        let size = new.data.len() as u32;
        if size > varint::MAX {
            return Err(ReturnFileDataError::SizeTooBig);
        }
        Ok(Self {
            group: new.group,
            resp: new.resp,
            file_id: new.file_id,
            offset: new.offset,
            data: new.data,
            _private: (),
        })
    }
}
impl Codec for ReturnFileData {
    fn encoded_size(&self) -> usize {
        1 + 1
            + unsafe_varint_serialize_sizes!(self.offset, self.data.len() as u32) as usize
            + self.data.len()
    }
    fn encode(&self, out: &mut [u8]) -> usize {
        out[0] = control_byte!(self.group, self.resp, OpCode::ReturnFileData);
        out[1] = self.file_id;
        let mut offset = 2;
        offset += unsafe_varint_serialize!(out[2..], self.offset, self.data.len() as u32) as usize;
        out[offset..offset + self.data.len()].clone_from_slice(&self.data[..]);
        offset += self.data.len();
        offset
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
        let min_size = 1 + 1 + 1 + 1;
        if out.len() < min_size {
            return Err(ParseFail::MissingBytes(Some(min_size - out.len())));
        }
        let group = out[0] & 0x80 != 0;
        let resp = out[0] & 0x40 != 0;
        let file_id = out[1];
        let mut off = 2;
        let ParseValue {
            value: offset,
            size: offset_size,
        } = varint::decode(&out[off..])?;
        off += offset_size;
        let ParseValue {
            value: size,
            size: size_size,
        } = varint::decode(&out[off..])?;
        off += size_size;
        let size = size as usize;
        let mut data = vec![0u8; size].into_boxed_slice();
        data.clone_from_slice(&out[off..off + size]);
        off += size;
        Ok(ParseValue {
            value: Self {
                group,
                resp,
                file_id,
                offset,
                data,
                _private: (),
            },
            size: off,
        })
    }
}
#[test]
fn test_return_file_data() {
    test_item(
        ReturnFileData {
            group: false,
            resp: false,
            file_id: 9,
            offset: 5,
            data: Box::new(hex!("01 02 03")),
            _private: (),
        },
        &hex!("20   09 05 03  010203"),
    )
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReturnFileProperties {
    pub group: bool,
    pub resp: bool,
    pub file_id: u8,
    pub header: FileHeader,
}
impl_header_op!(ReturnFileProperties, group, resp, file_id, header);
#[test]
fn test_return_file_properties() {
    test_item(
        ReturnFileProperties {
            group: false,
            resp: false,
            file_id: 9,
            header: FileHeader {
                permissions: Permissions {
                    encrypted: true,
                    executable: false,
                    user: UserPermissions {
                        read: true,
                        write: true,
                        run: true,
                    },
                    guest: UserPermissions {
                        read: false,
                        write: false,
                        run: false,
                    },
                },
                properties: FileProperties {
                    act_en: false,
                    act_cond: ActionCondition::Read,
                    storage_class: StorageClass::Permanent,
                },
                alp_cmd_fid: 1,
                interface_file_id: 2,
                file_size: 0xDEAD_BEEF,
                allocated_size: 0xBAAD_FACE,
            },
        },
        &hex!("21   09   B8 13 01 02 DEADBEEF BAADFACE"),
    )
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StatusType {
    Action = 0,
    Interface = 1,
}
impl StatusType {
    fn from(n: u8) -> Result<Self, ParseFail> {
        Ok(match n {
            0 => StatusType::Action,
            1 => StatusType::Interface,
            x => {
                return Err(ParseFail::Error {
                    error: ParseError::UnknownEnumVariant {
                        en: Enum::StatusType,
                        value: x,
                    },
                    offset: 0,
                })
            }
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Status {
    // ALP SPEC: This is named status, but it should be named action status compared to the '2'
    // other statuses.
    Action(StatusOperand),
    Interface(InterfaceStatus),
    // ALP SPEC: Where are the stack errors?
}
impl Codec for Status {
    fn encoded_size(&self) -> usize {
        1 + match self {
            Status::Action(op) => op.encoded_size(),
            Status::Interface(op) => op.encoded_size(),
        }
    }
    fn encode(&self, out: &mut [u8]) -> usize {
        out[0] = OpCode::Status as u8
            + ((match self {
                Status::Action(_) => StatusType::Action,
                Status::Interface(_) => StatusType::Interface,
            } as u8)
                << 6);
        let out = &mut out[1..];
        1 + match self {
            Status::Action(op) => op.encode(out),
            Status::Interface(op) => op.encode(out),
        }
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
        if out.is_empty() {
            return Err(ParseFail::MissingBytes(Some(1)));
        }
        let status_type = out[0] >> 6;
        Ok(match StatusType::from(status_type)? {
            StatusType::Action => {
                StatusOperand::decode(&out[1..])?.map(|v, size| (Status::Action(v), size + 1))
            }
            StatusType::Interface => InterfaceStatus::decode(&out[1..])
                .inc_offset(1)?
                .map(|v, size| (Status::Interface(v), size + 1)),
        })
    }
}
#[test]
fn test_status() {
    test_item(
        Status::Action(StatusOperand {
            action_id: 2,
            status: status_code::UNKNOWN_OPERATION,
        }),
        &hex!("22 02 F6"),
    )
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResponseTag {
    pub eop: bool, // End of packet
    pub err: bool,
    pub id: u8,
}
impl_simple_op!(ResponseTag, eop, err, id);
#[test]
fn test_response_tag() {
    test_item(
        ResponseTag {
            eop: true,
            err: false,
            id: 8,
        },
        &hex!("A3 08"),
    )
}

// Special
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ChunkStep {
    Continue = 0,
    Start = 1,
    End = 2,
    StartEnd = 3,
}
impl ChunkStep {
    // TODO Optimize, that can never be wrong
    fn from(n: u8) -> Self {
        match n {
            0 => ChunkStep::Continue,
            1 => ChunkStep::Start,
            2 => ChunkStep::End,
            3 => ChunkStep::StartEnd,
            x => panic!("Impossible chunk step {}", x),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Chunk {
    pub step: ChunkStep,
}
impl Codec for Chunk {
    fn encoded_size(&self) -> usize {
        1
    }
    fn encode(&self, out: &mut [u8]) -> usize {
        out[0] = OpCode::Chunk as u8 + ((self.step as u8) << 6);
        1
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
        if out.is_empty() {
            return Err(ParseFail::MissingBytes(Some(1)));
        }
        Ok(ParseValue {
            value: Self {
                step: ChunkStep::from(out[0] >> 6),
            },
            size: 1,
        })
    }
}
#[test]
fn test_chunk() {
    test_item(
        Chunk {
            step: ChunkStep::End,
        },
        &hex!("B0"),
    )
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LogicOp {
    Or = 0,
    Xor = 1,
    Nor = 2,
    Nand = 3,
}
impl LogicOp {
    // TODO Optimize, that can never be wrong
    fn from(n: u8) -> Self {
        match n {
            0 => LogicOp::Or,
            1 => LogicOp::Xor,
            2 => LogicOp::Nor,
            3 => LogicOp::Nand,
            x => panic!("Impossible logic op {}", x),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Logic {
    pub logic: LogicOp,
}
impl Codec for Logic {
    fn encoded_size(&self) -> usize {
        1
    }
    fn encode(&self, out: &mut [u8]) -> usize {
        out[0] = OpCode::Logic as u8 + ((self.logic as u8) << 6);
        1
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
        if out.is_empty() {
            return Err(ParseFail::MissingBytes(Some(1)));
        }
        Ok(ParseValue {
            value: Self {
                logic: LogicOp::from(out[0] >> 6),
            },
            size: 1,
        })
    }
}
#[test]
fn test_logic() {
    test_item(
        Logic {
            logic: LogicOp::Nand,
        },
        &hex!("F1"),
    )
}

#[derive(Clone, Debug, PartialEq)]
pub struct Forward {
    pub resp: bool,
    pub conf: InterfaceConfiguration,
}
impl Codec for Forward {
    fn encoded_size(&self) -> usize {
        1 + self.conf.encoded_size()
    }
    fn encode(&self, out: &mut [u8]) -> usize {
        out[0] = control_byte!(false, self.resp, OpCode::Forward);
        1 + self.conf.encode(&mut out[1..])
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
        let min_size = 1 + 1;
        if out.len() < min_size {
            return Err(ParseFail::MissingBytes(Some(min_size - out.len())));
        }
        let ParseValue {
            value: conf,
            size: conf_size,
        } = InterfaceConfiguration::decode(&out[1..]).inc_offset(1)?;
        Ok(ParseValue {
            value: Self {
                resp: out[0] & 0x40 != 0,
                conf,
            },
            size: 1 + conf_size,
        })
    }
}
#[test]
fn test_forward() {
    test_item(
        Forward {
            resp: true,
            conf: InterfaceConfiguration::Host,
        },
        &hex!("72 00"),
    )
}

#[derive(Clone, Debug, PartialEq)]
pub struct IndirectForward {
    pub resp: bool,
    pub interface: IndirectInterface,
}
impl Codec for IndirectForward {
    fn encoded_size(&self) -> usize {
        1 + self.interface.encoded_size()
    }
    fn encode(&self, out: &mut [u8]) -> usize {
        let overload = match self.interface {
            IndirectInterface::Overloaded(_) => true,
            IndirectInterface::NonOverloaded(_) => false,
        };
        out[0] = control_byte!(overload, self.resp, OpCode::IndirectForward);
        1 + serialize_all!(&mut out[1..], &self.interface)
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
        if out.is_empty() {
            Err(ParseFail::MissingBytes(Some(1)))
        } else {
            let mut offset = 0;
            let ParseValue {
                value: op1,
                size: op1_size,
            } = IndirectInterface::decode(out)?;
            offset += op1_size;
            Ok(ParseValue {
                value: Self {
                    resp: out[0] & 0x40 != 0,
                    interface: op1,
                },
                size: offset,
            })
        }
    }
}
#[test]
fn test_indirect_forward() {
    test_item(
        IndirectForward {
            resp: true,
            interface: IndirectInterface::Overloaded(OverloadedIndirectInterface {
                interface_file_id: 4,
                addressee: Addressee {
                    nls_method: NlsMethod::AesCcm32,
                    access_class: 0xFF,
                    address: Address::Vid(Box::new([0xAB, 0xCD])),
                },
            }),
        },
        &hex!("F3   04   37 FF ABCD"),
    )
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RequestTag {
    pub eop: bool, // End of packet
    pub id: u8,
}
impl Codec for RequestTag {
    fn encoded_size(&self) -> usize {
        1 + 1
    }
    fn encode(&self, out: &mut [u8]) -> usize {
        out[0] = control_byte!(self.eop, false, OpCode::RequestTag);
        out[1] = self.id;
        1 + 1
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
        let min_size = 1 + 1;
        if out.len() < min_size {
            return Err(ParseFail::MissingBytes(Some(min_size - out.len())));
        }
        Ok(ParseValue {
            value: Self {
                eop: out[0] & 0x80 != 0,
                id: out[1],
            },
            size: 2,
        })
    }
}
#[test]
fn test_request_tag() {
    test_item(RequestTag { eop: true, id: 8 }, &hex!("B4 08"))
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Extension {
    pub group: bool,
    pub resp: bool,
}
impl Codec for Extension {
    fn encoded_size(&self) -> usize {
        todo!()
    }
    fn encode(&self, _out: &mut [u8]) -> usize {
        todo!()
    }
    fn decode(_out: &[u8]) -> ParseResult<Self> {
        todo!()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    // Nop
    Nop(Nop),

    // Read
    ReadFileData(ReadFileData),
    ReadFileProperties(ReadFileProperties),

    // Write
    WriteFileData(WriteFileData),
    // ALP SPEC: This is not specified even though it is implemented
    // WriteFileDataFlush(WriteFileDataFlush),
    WriteFileProperties(WriteFileProperties),
    ActionQuery(ActionQuery),
    BreakQuery(BreakQuery),
    PermissionRequest(PermissionRequest),
    VerifyChecksum(VerifyChecksum),

    // Management
    ExistFile(ExistFile),
    CreateNewFile(CreateNewFile),
    DeleteFile(DeleteFile),
    RestoreFile(RestoreFile),
    FlushFile(FlushFile),
    CopyFile(CopyFile),
    ExecuteFile(ExecuteFile),

    // Response
    ReturnFileData(ReturnFileData),
    ReturnFileProperties(ReturnFileProperties),
    Status(Status),
    ResponseTag(ResponseTag),

    // Special
    Chunk(Chunk),
    Logic(Logic),
    Forward(Forward),
    IndirectForward(IndirectForward),
    RequestTag(RequestTag),
    Extension(Extension),
}

impl Codec for Action {
    fn encoded_size(&self) -> usize {
        match self {
            Action::Nop(x) => x.encoded_size(),
            Action::ReadFileData(x) => x.encoded_size(),
            Action::ReadFileProperties(x) => x.encoded_size(),
            Action::WriteFileData(x) => x.encoded_size(),
            // Action::WriteFileDataFlush(x) => x.encoded_size(),
            Action::WriteFileProperties(x) => x.encoded_size(),
            Action::ActionQuery(x) => x.encoded_size(),
            Action::BreakQuery(x) => x.encoded_size(),
            Action::PermissionRequest(x) => x.encoded_size(),
            Action::VerifyChecksum(x) => x.encoded_size(),
            Action::ExistFile(x) => x.encoded_size(),
            Action::CreateNewFile(x) => x.encoded_size(),
            Action::DeleteFile(x) => x.encoded_size(),
            Action::RestoreFile(x) => x.encoded_size(),
            Action::FlushFile(x) => x.encoded_size(),
            Action::CopyFile(x) => x.encoded_size(),
            Action::ExecuteFile(x) => x.encoded_size(),
            Action::ReturnFileData(x) => x.encoded_size(),
            Action::ReturnFileProperties(x) => x.encoded_size(),
            Action::Status(x) => x.encoded_size(),
            Action::ResponseTag(x) => x.encoded_size(),
            Action::Chunk(x) => x.encoded_size(),
            Action::Logic(x) => x.encoded_size(),
            Action::Forward(x) => x.encoded_size(),
            Action::IndirectForward(x) => x.encoded_size(),
            Action::RequestTag(x) => x.encoded_size(),
            Action::Extension(x) => x.encoded_size(),
        }
    }
    fn encode(&self, out: &mut [u8]) -> usize {
        match self {
            Action::Nop(x) => x.encode(out),
            Action::ReadFileData(x) => x.encode(out),
            Action::ReadFileProperties(x) => x.encode(out),
            Action::WriteFileData(x) => x.encode(out),
            // Action::WriteFileDataFlush(x) => x.encode(out),
            Action::WriteFileProperties(x) => x.encode(out),
            Action::ActionQuery(x) => x.encode(out),
            Action::BreakQuery(x) => x.encode(out),
            Action::PermissionRequest(x) => x.encode(out),
            Action::VerifyChecksum(x) => x.encode(out),
            Action::ExistFile(x) => x.encode(out),
            Action::CreateNewFile(x) => x.encode(out),
            Action::DeleteFile(x) => x.encode(out),
            Action::RestoreFile(x) => x.encode(out),
            Action::FlushFile(x) => x.encode(out),
            Action::CopyFile(x) => x.encode(out),
            Action::ExecuteFile(x) => x.encode(out),
            Action::ReturnFileData(x) => x.encode(out),
            Action::ReturnFileProperties(x) => x.encode(out),
            Action::Status(x) => x.encode(out),
            Action::ResponseTag(x) => x.encode(out),
            Action::Chunk(x) => x.encode(out),
            Action::Logic(x) => x.encode(out),
            Action::Forward(x) => x.encode(out),
            Action::IndirectForward(x) => x.encode(out),
            Action::RequestTag(x) => x.encode(out),
            Action::Extension(x) => x.encode(out),
        }
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
        if out.is_empty() {
            return Err(ParseFail::MissingBytes(Some(1)));
        }
        let opcode = OpCode::from(out[0] & 0x3F)?;
        Ok(match opcode {
            OpCode::Nop => Nop::decode(&out)?.map_value(Action::Nop),
            OpCode::ReadFileData => ReadFileData::decode(&out)?.map_value(Action::ReadFileData),
            OpCode::ReadFileProperties => {
                ReadFileProperties::decode(&out)?.map_value(Action::ReadFileProperties)
            }
            OpCode::WriteFileData => WriteFileData::decode(&out)?.map_value(Action::WriteFileData),
            // OpCode::WriteFileDataFlush => {
            //     WriteFileDataFlush::decode(&out)?.map_value( Action::WriteFileDataFlush)
            // }
            OpCode::WriteFileProperties => {
                WriteFileProperties::decode(&out)?.map_value(Action::WriteFileProperties)
            }
            OpCode::ActionQuery => ActionQuery::decode(&out)?.map_value(Action::ActionQuery),
            OpCode::BreakQuery => BreakQuery::decode(&out)?.map_value(Action::BreakQuery),
            OpCode::PermissionRequest => {
                PermissionRequest::decode(&out)?.map_value(Action::PermissionRequest)
            }
            OpCode::VerifyChecksum => {
                VerifyChecksum::decode(&out)?.map_value(Action::VerifyChecksum)
            }
            OpCode::ExistFile => ExistFile::decode(&out)?.map_value(Action::ExistFile),
            OpCode::CreateNewFile => CreateNewFile::decode(&out)?.map_value(Action::CreateNewFile),
            OpCode::DeleteFile => DeleteFile::decode(&out)?.map_value(Action::DeleteFile),
            OpCode::RestoreFile => RestoreFile::decode(&out)?.map_value(Action::RestoreFile),
            OpCode::FlushFile => FlushFile::decode(&out)?.map_value(Action::FlushFile),
            OpCode::CopyFile => CopyFile::decode(&out)?.map_value(Action::CopyFile),
            OpCode::ExecuteFile => ExecuteFile::decode(&out)?.map_value(Action::ExecuteFile),
            OpCode::ReturnFileData => {
                ReturnFileData::decode(&out)?.map_value(Action::ReturnFileData)
            }
            OpCode::ReturnFileProperties => {
                ReturnFileProperties::decode(&out)?.map_value(Action::ReturnFileProperties)
            }
            OpCode::Status => Status::decode(&out)?.map_value(Action::Status),
            OpCode::ResponseTag => ResponseTag::decode(&out)?.map_value(Action::ResponseTag),
            OpCode::Chunk => Chunk::decode(&out)?.map_value(Action::Chunk),
            OpCode::Logic => Logic::decode(&out)?.map_value(Action::Logic),
            OpCode::Forward => Forward::decode(&out)?.map_value(Action::Forward),
            OpCode::IndirectForward => {
                IndirectForward::decode(&out)?.map_value(Action::IndirectForward)
            }
            OpCode::RequestTag => RequestTag::decode(&out)?.map_value(Action::RequestTag),
            OpCode::Extension => Extension::decode(&out)?.map_value(Action::Extension),
        })
    }
}

// ===============================================================================
// Command
// ===============================================================================
#[derive(Clone, Debug, PartialEq)]
pub struct Command {
    pub actions: Vec<Action>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct CommandParseFail {
    pub actions: Vec<Action>,
    pub error: ParseFail,
}

impl Default for Command {
    fn default() -> Self {
        Self { actions: vec![] }
    }
}
impl Command {
    fn partial_decode(out: &[u8]) -> Result<ParseValue<Command>, CommandParseFail> {
        let mut actions = vec![];
        let mut offset = 0;
        loop {
            if out.is_empty() {
                break;
            }
            match Action::decode(&out[offset..]) {
                Ok(ParseValue { value, size }) => {
                    actions.push(value);
                    offset += size;
                }
                Err(error) => {
                    return Err(CommandParseFail {
                        actions,
                        error: error.inc_offset(offset),
                    })
                }
            }
        }
        Ok(ParseValue {
            value: Self { actions },
            size: offset,
        })
    }
}
impl Codec for Command {
    fn encoded_size(&self) -> usize {
        self.actions.iter().map(|act| act.encoded_size()).sum()
    }
    fn encode(&self, out: &mut [u8]) -> usize {
        let mut offset = 0;
        for action in self.actions.iter() {
            offset += action.encode(&mut out[offset..]);
        }
        offset
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
        Self::partial_decode(out).map_err(|v| v.error)
    }
}

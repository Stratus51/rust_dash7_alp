#[cfg(test)]
use crate::{dash7, test_tools::test_item};
#[cfg(test)]
use hex_literal::hex;

use crate::{
    codec::{Codec, StdError, WithOffset, WithSize},
    data, operand, varint,
};

// ===============================================================================
// Macros
// ===============================================================================
macro_rules! serialize_all {
    ($out: expr, $($x: expr),*) => {
        {
            let mut offset = 0;
            $({
                offset += $x.encode_in(&mut $out[offset..]);
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
    ($name: ident, $flag7: ident, $flag6: ident, $op1: ident, $op1_type: ident, $error: ty) => {
        impl Codec for $name {
            type Error = $error;
            fn encoded_size(&self) -> usize {
                1 + encoded_size!(self.$op1)
            }
            unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
                out[0] = control_byte!(self.$flag7, self.$flag6, OpCode::$name);
                1 + serialize_all!(&mut out[1..], &self.$op1)
            }
            fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
                if (out.is_empty()) {
                    Err(WithOffset::new_head(Self::Error::MissingBytes(1)))
                } else {
                    let mut offset = 1;
                    let WithSize {
                        size: op1_size,
                        value: op1,
                    } = operand::$op1_type::decode(&out[offset..]).map_err(|e| e.shift(offset))?;
                    offset += op1_size;
                    Ok(WithSize {
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
            $({
                offset += varint::encode_in($x, &mut $out[offset..]) as usize;
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
            type Error = StdError;
            fn encoded_size(&self) -> usize {
                1 + count!($( $x )*)
            }
            unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
                out[0] = control_byte!(self.$flag7, self.$flag6, OpCode::$name);
                let mut offset = 1;
                $({
                    out[offset] = self.$x;
                    offset += 1;
                })*
                1 + offset
            }
            fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
                const SIZE: usize = 1 + count!($( $x )*);
                if(out.len() < SIZE) {
                    Err(WithOffset::new_head( Self::Error::MissingBytes(SIZE - out.len())))
                } else {
                    Ok(WithSize {
                        size: SIZE,
                        value: build_simple_op!($name, out, $flag7, $flag6, $($x),*),
                    })
                }
            }
        }
    };
}

macro_rules! impl_display_simple_op {
    ($name: ident) => {
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(
                    f,
                    "[{}{}]",
                    if self.group { "G" } else { "-" },
                    if self.resp { "R" } else { "-" },
                )
            }
        }
    };
    ($name: ident, $field1: ident) => {
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(
                    f,
                    "[{}{}]{}",
                    if self.group { "G" } else { "-" },
                    if self.resp { "R" } else { "-" },
                    self.$field1
                )
            }
        }
    };
    ($name: ident, $field1: ident, $field2: ident) => {
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(
                    f,
                    "[{}{}]{},{}",
                    if self.group { "G" } else { "-" },
                    if self.resp { "R" } else { "-" },
                    self.$field1,
                    self.$field2
                )
            }
        }
    };
}

macro_rules! impl_display_simple_file_op {
    ($name: ident, $field1: ident) => {
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(
                    f,
                    "[{}{}]f({})",
                    if self.group { "G" } else { "-" },
                    if self.resp { "R" } else { "-" },
                    self.$field1,
                )
            }
        }
    };
    ($name: ident, $field1: ident, $field2: ident) => {
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(
                    f,
                    "[{}{}]f({},{})",
                    if self.group { "G" } else { "-" },
                    if self.resp { "R" } else { "-" },
                    self.$field1,
                    self.$field2,
                )
            }
        }
    };
    ($name: ident, $field1: ident, $field2: ident, $field3: ident) => {
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(
                    f,
                    "[{}{}]f({},{},{})",
                    if self.group { "G" } else { "-" },
                    if self.resp { "R" } else { "-" },
                    self.$field1,
                    self.$field2,
                    self.$field3,
                )
            }
        }
    };
}

macro_rules! impl_display_data_file_op {
    ($name: ident) => {
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(
                    f,
                    "[{}{}]f({},{},0x{})",
                    if self.group { "G" } else { "-" },
                    if self.resp { "R" } else { "-" },
                    self.file_id,
                    self.offset,
                    hex::encode_upper(&self.data),
                )
            }
        }
    };
}

macro_rules! impl_display_prop_file_op {
    ($name: ident, $field: ident) => {
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(
                    f,
                    "[{}{}]f({}){}",
                    if self.group { "G" } else { "-" },
                    if self.resp { "R" } else { "-" },
                    self.file_id,
                    self.$field,
                )
            }
        }
    };
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum HeaderActionDecodingError {
    MissingBytes(usize),
    FileHeader(StdError),
}

macro_rules! impl_header_op {
    ($name: ident, $flag7: ident, $flag6: ident, $file_id: ident, $file_header: ident) => {
        impl Codec for $name {
            type Error = HeaderActionDecodingError;
            fn encoded_size(&self) -> usize {
                1 + 1 + 12
            }
            unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
                out[0] = control_byte!(self.group, self.resp, OpCode::$name);
                out[1] = self.file_id;
                let mut offset = 2;
                offset += self.$file_header.encode_in(&mut out[offset..]);
                offset
            }
            fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
                const SIZE: usize = 1 + 1 + 12;
                if (out.len() < SIZE) {
                    Err(WithOffset::new(
                        0,
                        Self::Error::MissingBytes(SIZE - out.len()),
                    ))
                } else {
                    let WithSize { value: header, .. } = data::FileHeader::decode(&out[2..])
                        .map_err(|e| {
                            let WithOffset { offset, value } = e;
                            WithOffset {
                                offset: offset + 2,
                                value: Self::Error::FileHeader(value),
                            }
                        })?;
                    Ok(WithSize {
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
    fn from(n: u8) -> Result<Self, u8> {
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
            x => return Err(x),
        })
    }
}
impl std::fmt::Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            // Nop
            OpCode::Nop => write!(f, "NOP"),

            // Read
            OpCode::ReadFileData => write!(f, "RD"),
            OpCode::ReadFileProperties => write!(f, "RDP"),

            // Write
            OpCode::WriteFileData => write!(f, "WR"),
            OpCode::WriteFileProperties => write!(f, "WRP"),
            OpCode::ActionQuery => write!(f, "AQ"),
            OpCode::BreakQuery => write!(f, "BQ"),
            OpCode::PermissionRequest => write!(f, "PR"),
            OpCode::VerifyChecksum => write!(f, "VCS"),

            // Management
            OpCode::ExistFile => write!(f, "HAS"),
            OpCode::CreateNewFile => write!(f, "NEW"),
            OpCode::DeleteFile => write!(f, "DEL"),
            OpCode::RestoreFile => write!(f, "RST"),
            OpCode::FlushFile => write!(f, "FLUSH"),
            OpCode::CopyFile => write!(f, "CP"),
            OpCode::ExecuteFile => write!(f, "RUN"),

            // Response
            OpCode::ReturnFileData => write!(f, "DATA"),
            OpCode::ReturnFileProperties => write!(f, "PROP"),
            OpCode::Status => write!(f, "S"),
            OpCode::ResponseTag => write!(f, "TG"),

            // Special
            OpCode::Chunk => write!(f, "CHK"),
            OpCode::Logic => write!(f, "LOG"),
            OpCode::Forward => write!(f, "FWD"),
            OpCode::IndirectForward => write!(f, "IFWD"),
            OpCode::RequestTag => write!(f, "RTG"),
            OpCode::Extension => write!(f, "EXT"),
        }
    }
}

// ===============================================================================
// Actions
// ===============================================================================
// Nop
/// Does nothing
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Nop {
    /// Group with next action
    pub group: bool,
    /// Ask for a response (status)
    pub resp: bool,
}
impl_display_simple_op!(Nop);
impl Codec for Nop {
    type Error = StdError;

    fn encoded_size(&self) -> usize {
        1
    }
    unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
        out[0] = control_byte!(self.group, self.resp, OpCode::Nop);
        1
    }
    fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
        if out.is_empty() {
            Err(WithOffset::new_head(Self::Error::MissingBytes(1)))
        } else {
            Ok(WithSize {
                size: 1,
                value: Self {
                    resp: out[0] & 0x40 != 0,
                    group: out[0] & 0x80 != 0,
                },
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

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum OperandValidationError {
    /// Offset is too big to be encoded in a varint
    OffsetTooBig,
    /// Size is too big to be encoded in a varint
    SizeTooBig,
}

// Read
/// Read data from a file
#[derive(Clone, Copy, Debug, PartialEq)]
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
}
impl_display_simple_file_op!(ReadFileData, file_id, offset, size);
impl ReadFileData {
    pub fn validate(self) -> Result<(), OperandValidationError> {
        if self.offset > varint::MAX {
            return Err(OperandValidationError::OffsetTooBig);
        }
        if self.size > varint::MAX {
            return Err(OperandValidationError::SizeTooBig);
        }
        Ok(())
    }
}

impl Codec for ReadFileData {
    type Error = StdError;
    fn encoded_size(&self) -> usize {
        1 + 1 + unsafe_varint_serialize_sizes!(self.offset, self.size) as usize
    }
    unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
        out[0] = control_byte!(self.group, self.resp, OpCode::ReadFileData);
        out[1] = self.file_id;
        1 + 1 + unsafe_varint_serialize!(out[2..], self.offset, self.size)
    }
    fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
        let min_size = 1 + 1 + 1 + 1;
        if out.len() < min_size {
            return Err(WithOffset::new(
                0,
                Self::Error::MissingBytes(min_size - out.len()),
            ));
        }
        let group = out[0] & 0x80 != 0;
        let resp = out[0] & 0x40 != 0;
        let file_id = out[1];
        let mut off = 2;
        let WithSize {
            value: offset,
            size: offset_size,
        } = varint::decode(&out[off..]).map_err(|e| {
            e.shift(off);
            e
        })?;
        off += offset_size;
        let WithSize {
            value: size,
            size: size_size,
        } = varint::decode(&out[off..]).map_err(|e| {
            e.shift(off);
            e
        })?;
        off += size_size;
        Ok(WithSize {
            value: Self {
                group,
                resp,
                file_id,
                offset,
                size,
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
        },
        &hex!("41 01 02 03"),
    )
}

/// Read properties of a file
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReadFileProperties {
    /// Group with next action
    pub group: bool,
    /// Ask for a response (ReturnFileProperties)
    pub resp: bool,
    pub file_id: u8,
}
impl_simple_op!(ReadFileProperties, group, resp, file_id);
impl_display_simple_file_op!(ReadFileProperties, file_id);
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
/// Write data to a file
#[derive(Clone, Debug, PartialEq)]
pub struct WriteFileData {
    /// Group with next action
    pub group: bool,
    /// Ask for a response (a status)
    pub resp: bool,
    pub file_id: u8,
    pub offset: u32,
    pub data: Box<[u8]>,
}
impl_display_data_file_op!(WriteFileData);
impl WriteFileData {
    pub fn validate(&self) -> Result<(), OperandValidationError> {
        if self.offset > varint::MAX {
            return Err(OperandValidationError::OffsetTooBig);
        }
        let size = self.data.len() as u32;
        if size > varint::MAX {
            return Err(OperandValidationError::SizeTooBig);
        }
        Ok(())
    }
}
impl Codec for WriteFileData {
    type Error = StdError;
    fn encoded_size(&self) -> usize {
        1 + 1
            + unsafe_varint_serialize_sizes!(self.offset, self.data.len() as u32) as usize
            + self.data.len()
    }
    unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
        out[0] = control_byte!(self.group, self.resp, OpCode::WriteFileData);
        out[1] = self.file_id;
        let mut offset = 2;
        offset += unsafe_varint_serialize!(out[2..], self.offset, self.data.len() as u32) as usize;
        out[offset..offset + self.data.len()].clone_from_slice(&self.data[..]);
        offset += self.data.len();
        offset
    }
    fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
        let min_size = 1 + 1 + 1 + 1;
        if out.len() < min_size {
            return Err(WithOffset::new(
                0,
                Self::Error::MissingBytes(min_size - out.len()),
            ));
        }
        let group = out[0] & 0x80 != 0;
        let resp = out[0] & 0x40 != 0;
        let file_id = out[1];
        let mut off = 2;
        let WithSize {
            value: offset,
            size: offset_size,
        } = varint::decode(&out[off..])?;
        off += offset_size;
        let WithSize {
            value: size,
            size: size_size,
        } = varint::decode(&out[off..])?;
        off += size_size;
        let size = size as usize;
        let mut data = vec![0u8; size].into_boxed_slice();
        data.clone_from_slice(&out[off..off + size]);
        off += size;
        Ok(WithSize {
            value: Self {
                group,
                resp,
                file_id,
                offset,
                data,
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
        },
        &hex!("84   09 05 03  010203"),
    )
}

/// Write the properties of a file
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WriteFileProperties {
    /// Group with next action
    pub group: bool,
    /// Ask for a response (a status)
    pub resp: bool,
    pub file_id: u8,
    pub header: data::FileHeader,
}
impl_header_op!(WriteFileProperties, group, resp, file_id, header);
impl std::fmt::Display for WriteFileProperties {
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
#[test]
fn test_write_file_properties() {
    test_item(
        WriteFileProperties {
            group: true,
            resp: false,
            file_id: 9,
            header: data::FileHeader {
                permissions: data::Permissions {
                    encrypted: true,
                    executable: false,
                    user: data::UserPermissions {
                        read: true,
                        write: true,
                        run: true,
                    },
                    guest: data::UserPermissions {
                        read: false,
                        write: false,
                        run: false,
                    },
                },
                properties: data::FileProperties {
                    act_en: false,
                    act_cond: data::ActionCondition::Read,
                    storage_class: data::StorageClass::Permanent,
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

/// Add a condition on the execution of the next group of action.
///
/// If the condition is not met, the next group of action should be skipped.
#[derive(Clone, Debug, PartialEq)]
pub struct ActionQuery {
    /// Group with next action
    pub group: bool,
    /// Does not make sense.
    pub resp: bool,
    pub query: operand::Query,
}
impl_display_simple_op!(ActionQuery, query);
impl_op_serialized!(
    ActionQuery,
    group,
    resp,
    query,
    Query,
    operand::QueryDecodingError
);
#[test]
fn test_action_query() {
    test_item(
        ActionQuery {
            group: true,
            resp: true,
            query: operand::Query::NonVoid(operand::NonVoid {
                size: 4,
                file: operand::FileOffset { id: 5, offset: 6 },
            }),
        },
        &hex!("C8   00 04  05 06"),
    )
}

/// Add a condition to continue the processing of this ALP command.
///
/// If the condition is not met the all the next ALP action of this command should be ignored.
#[derive(Clone, Debug, PartialEq)]
pub struct BreakQuery {
    /// Group with next action
    pub group: bool,
    /// Does not make sense.
    pub resp: bool,
    pub query: operand::Query,
}
impl std::fmt::Display for BreakQuery {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "[{}{}]{}",
            if self.group { "G" } else { "-" },
            if self.resp { "R" } else { "-" },
            self.query,
        )
    }
}
impl_op_serialized!(
    BreakQuery,
    group,
    resp,
    query,
    Query,
    operand::QueryDecodingError
);
#[test]
fn test_break_query() {
    test_item(
        BreakQuery {
            group: true,
            resp: true,
            query: operand::Query::NonVoid(operand::NonVoid {
                size: 4,
                file: operand::FileOffset { id: 5, offset: 6 },
            }),
        },
        &hex!("C9   00 04  05 06"),
    )
}

/// Request a level of permission using some permission type
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PermissionRequest {
    /// Group with next action
    pub group: bool,
    /// Ask for a response (a status)
    pub resp: bool,
    /// See operand::permission_level
    pub level: u8,
    pub permission: operand::Permission,
}
impl_display_simple_op!(PermissionRequest, level, permission);
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PermissionRequestDecodingError {
    MissingBytes(usize),
    Permission(operand::PermissionDecodingError),
}
impl Codec for PermissionRequest {
    type Error = PermissionRequestDecodingError;
    fn encoded_size(&self) -> usize {
        1 + 1 + encoded_size!(self.permission)
    }
    unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
        out[0] = control_byte!(self.group, self.resp, OpCode::PermissionRequest);
        out[1] = self.level;
        1 + serialize_all!(&mut out[2..], self.permission)
    }
    fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
        if out.is_empty() {
            Err(WithOffset::new_head(Self::Error::MissingBytes(1)))
        } else {
            let mut offset = 1;
            let level = out[offset];
            offset += 1;
            let WithSize {
                value: permission,
                size,
            } = operand::Permission::decode(&out[offset..])
                .map_err(|e| e.shift(offset).map_value(Self::Error::Permission))?;
            offset += size;
            Ok(WithSize {
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
            level: operand::permission_level::ROOT,
            permission: operand::Permission::Dash7(hex!("0102030405060708")),
        },
        &hex!("0A   01 42 0102030405060708"),
    )
}

/// Calculate checksum of file and compare with checksum in query
// ALP_SPEC: Is the checksum calculated on the targeted data (offset, size) or the whole file?
#[derive(Clone, Debug, PartialEq)]
pub struct VerifyChecksum {
    /// Group with next action
    pub group: bool,
    /// Ask for a response (status?)
    pub resp: bool,
    pub query: operand::Query,
}
impl_display_simple_op!(VerifyChecksum, query);
impl_op_serialized!(
    VerifyChecksum,
    group,
    resp,
    query,
    Query,
    operand::QueryDecodingError
);
#[test]
fn test_verify_checksum() {
    test_item(
        VerifyChecksum {
            group: false,
            resp: false,
            query: operand::Query::NonVoid(operand::NonVoid {
                size: 4,
                file: operand::FileOffset { id: 5, offset: 6 },
            }),
        },
        &hex!("0B   00 04  05 06"),
    )
}

// Management
/// Checks whether a file exists
// ALP_SPEC: How is the result of this command different from a read file of size 0?
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExistFile {
    /// Group with next action
    pub group: bool,
    /// Ask for a response (status?)
    pub resp: bool,
    pub file_id: u8,
}
impl_display_simple_file_op!(ExistFile, file_id);
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

/// Create a new file
// ALP_SPEC: How do you create a remote file? Is this Wizzilab specific.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CreateNewFile {
    /// Group with next action
    pub group: bool,
    /// Ask for a response (status)
    pub resp: bool,
    pub file_id: u8,
    pub header: data::FileHeader,
}
impl_display_prop_file_op!(CreateNewFile, header);
impl_header_op!(CreateNewFile, group, resp, file_id, header);
#[test]
fn test_create_new_file() {
    test_item(
        CreateNewFile {
            group: true,
            resp: false,
            file_id: 3,
            header: data::FileHeader {
                permissions: data::Permissions {
                    encrypted: true,
                    executable: false,
                    user: data::UserPermissions {
                        read: true,
                        write: true,
                        run: true,
                    },
                    guest: data::UserPermissions {
                        read: false,
                        write: false,
                        run: false,
                    },
                },
                properties: data::FileProperties {
                    act_en: false,
                    act_cond: data::ActionCondition::Read,
                    storage_class: data::StorageClass::Permanent,
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

/// Deletes a file.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DeleteFile {
    /// Group with next action
    pub group: bool,
    /// Ask for a response (status)
    pub resp: bool,
    pub file_id: u8,
}
impl_display_simple_file_op!(DeleteFile, file_id);
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

/// Restores a restorable file
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RestoreFile {
    /// Group with next action
    pub group: bool,
    /// Ask for a response (status)
    pub resp: bool,
    pub file_id: u8,
}
impl_display_simple_file_op!(RestoreFile, file_id);
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

/// Flush a file
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlushFile {
    /// Group with next action
    pub group: bool,
    /// Ask for a response (status)
    pub resp: bool,
    pub file_id: u8,
}
impl_display_simple_file_op!(FlushFile, file_id);
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

/// Copy a file to another file
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

/// Execute a file if executable
// ALP_SPEC: Is that an "ALP executable" or a binary executable?
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExecuteFile {
    /// Group with next action
    pub group: bool,
    /// Ask for a response (status)
    pub resp: bool,
    pub file_id: u8,
}
impl_display_simple_file_op!(ExecuteFile, file_id);
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
/// Responds to a ReadFileData request.
///
/// This can also be used to report unsollicited data.
#[derive(Clone, Debug, PartialEq)]
pub struct ReturnFileData {
    /// Group with next action
    pub group: bool,
    /// Ask for a response (status)
    pub resp: bool,
    pub file_id: u8,
    pub offset: u32,
    pub data: Box<[u8]>,
}
impl_display_data_file_op!(ReturnFileData);
impl ReturnFileData {
    pub fn validate(&self) -> Result<(), OperandValidationError> {
        if self.offset > varint::MAX {
            return Err(OperandValidationError::OffsetTooBig);
        }
        let size = self.data.len() as u32;
        if size > varint::MAX {
            return Err(OperandValidationError::SizeTooBig);
        }
        Ok(())
    }
}
impl Codec for ReturnFileData {
    type Error = StdError;
    fn encoded_size(&self) -> usize {
        1 + 1
            + unsafe_varint_serialize_sizes!(self.offset, self.data.len() as u32) as usize
            + self.data.len()
    }
    unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
        out[0] = control_byte!(self.group, self.resp, OpCode::ReturnFileData);
        out[1] = self.file_id;
        let mut offset = 2;
        offset += unsafe_varint_serialize!(out[2..], self.offset, self.data.len() as u32) as usize;
        out[offset..offset + self.data.len()].clone_from_slice(&self.data[..]);
        offset += self.data.len();
        offset
    }
    fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
        let min_size = 1 + 1 + 1 + 1;
        if out.len() < min_size {
            return Err(WithOffset::new(
                0,
                Self::Error::MissingBytes(min_size - out.len()),
            ));
        }
        let group = out[0] & 0x80 != 0;
        let resp = out[0] & 0x40 != 0;
        let file_id = out[1];
        let mut off = 2;
        let WithSize {
            value: offset,
            size: offset_size,
        } = varint::decode(&out[off..])?;
        off += offset_size;
        let WithSize {
            value: size,
            size: size_size,
        } = varint::decode(&out[off..])?;
        off += size_size;
        let size = size as usize;
        let mut data = vec![0u8; size].into_boxed_slice();
        data.clone_from_slice(&out[off..off + size]);
        off += size;
        Ok(WithSize {
            value: Self {
                group,
                resp,
                file_id,
                offset,
                data,
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
        },
        &hex!("20   09 05 03  010203"),
    )
}

/// Result of a ReadFileProperties request
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReturnFileProperties {
    /// Group with next action
    pub group: bool,
    /// Ask for a response (status)
    pub resp: bool,
    pub file_id: u8,
    pub header: data::FileHeader,
}
impl_display_prop_file_op!(ReturnFileProperties, header);
impl_header_op!(ReturnFileProperties, group, resp, file_id, header);
#[test]
fn test_return_file_properties() {
    test_item(
        ReturnFileProperties {
            group: false,
            resp: false,
            file_id: 9,
            header: data::FileHeader {
                permissions: data::Permissions {
                    encrypted: true,
                    executable: false,
                    user: data::UserPermissions {
                        read: true,
                        write: true,
                        run: true,
                    },
                    guest: data::UserPermissions {
                        read: false,
                        write: false,
                        run: false,
                    },
                },
                properties: data::FileProperties {
                    act_en: false,
                    act_cond: data::ActionCondition::Read,
                    storage_class: data::StorageClass::Permanent,
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
    fn from(n: u8) -> Result<Self, u8> {
        Ok(match n {
            0 => StatusType::Action,
            1 => StatusType::Interface,
            x => return Err(x),
        })
    }
}

/// Statuses regarding actions sent in a request
#[derive(Clone, Debug, PartialEq)]
pub enum Status {
    // ALP SPEC: This is named status, but it should be named action status compared to the '2'
    // other statuses.
    Action(operand::Status),
    Interface(operand::InterfaceStatus),
    // ALP SPEC: Where are the stack errors?
}
impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Action(v) => write!(f, "[ACT]:{}", v),
            Self::Interface(v) => write!(f, "[ITF]:{}", v),
        }
    }
}
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum StatusDecodingError {
    MissingBytes(usize),
    UnknownType(u8),
    Action(StdError),
    Interface(operand::InterfaceStatusDecodingError),
}
impl Codec for Status {
    type Error = StatusDecodingError;
    fn encoded_size(&self) -> usize {
        1 + match self {
            Status::Action(op) => op.encoded_size(),
            Status::Interface(op) => op.encoded_size(),
        }
    }
    unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
        out[0] = OpCode::Status as u8
            + ((match self {
                Status::Action(_) => StatusType::Action,
                Status::Interface(_) => StatusType::Interface,
            } as u8)
                << 6);
        let out = &mut out[1..];
        1 + match self {
            Status::Action(op) => op.encode_in(out),
            Status::Interface(op) => op.encode_in(out),
        }
    }
    fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
        if out.is_empty() {
            return Err(WithOffset::new_head(Self::Error::MissingBytes(1)));
        }
        let status_type = out[0] >> 6;
        Ok(
            match StatusType::from(status_type)
                .map_err(|e| WithOffset::new_head(Self::Error::UnknownType(e)))?
            {
                StatusType::Action => {
                    let WithSize { size, value } = operand::Status::decode(&out[1..])
                        .map_err(|e| e.shift(1).map_value(Self::Error::Action))?;
                    WithSize {
                        size: size + 1,
                        value: Self::Action(value),
                    }
                }
                StatusType::Interface => {
                    let WithSize { size, value } = operand::InterfaceStatus::decode(&out[1..])
                        .map_err(|e| e.shift(1).map_value(Self::Error::Interface))?;
                    WithSize {
                        size: size + 1,
                        value: Self::Interface(value),
                    }
                }
            },
        )
    }
}
#[test]
fn test_status() {
    test_item(
        Status::Action(operand::Status {
            action_id: 2,
            status: operand::status_code::UNKNOWN_OPERATION,
        }),
        &hex!("22 02 F6"),
    )
}

/// Action received before any responses to a request that contained a RequestTag
///
/// This allows matching responses to requests when doing multiple requests in parallel.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResponseTag {
    /// End of packet
    ///
    /// Signal the last response packet for the request `id`
    /// (E)
    pub eop: bool,
    /// An error occured
    /// (R)
    pub err: bool,
    pub id: u8,
}
impl std::fmt::Display for ResponseTag {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "[{}{}]({})",
            if self.eop { "E" } else { "-" },
            if self.err { "R" } else { "-" },
            self.id,
        )
    }
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
/// Provide chunk information and therefore allows to send an ALP command by chunks.
///
/// Specification:
/// An ALP Command may be chunked into multiple Chunks. A special Chunk Action is inserted at the beginning of each
/// ALP Command Chunk to define its chunk state – START, CONTINUE or END (see 6.2.2.1). If the Chunk Action is not
/// present, the ALP Command is not chunked (implicit START/END). The Group (11.5.3) and Break Query conditions are
/// extended over all chunks of the ALP Command.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Chunk {
    pub step: ChunkStep,
}
impl std::fmt::Display for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.step {
            ChunkStep::Continue => write!(f, "[C]"),
            ChunkStep::Start => write!(f, "[S]"),
            ChunkStep::End => write!(f, "[E]"),
            ChunkStep::StartEnd => write!(f, "[R]"),
        }
    }
}
impl Codec for Chunk {
    type Error = StdError;
    fn encoded_size(&self) -> usize {
        1
    }
    unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
        out[0] = OpCode::Chunk as u8 + ((self.step as u8) << 6);
        1
    }
    fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
        if out.is_empty() {
            return Err(WithOffset::new_head(Self::Error::MissingBytes(1)));
        }
        Ok(WithSize {
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

/// Provide logical link of a group of queries
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LogicOp {
    Or = 0,
    Xor = 1,
    Nor = 2,
    Nand = 3,
}
impl LogicOp {
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
impl std::fmt::Display for Logic {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.logic {
            LogicOp::Or => write!(f, "[OR]"),
            LogicOp::Xor => write!(f, "[XOR]"),
            LogicOp::Nor => write!(f, "[NOR]"),
            LogicOp::Nand => write!(f, "[NAND]"),
        }
    }
}
impl Codec for Logic {
    type Error = StdError;
    fn encoded_size(&self) -> usize {
        1
    }
    unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
        out[0] = OpCode::Logic as u8 + ((self.logic as u8) << 6);
        1
    }
    fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
        if out.is_empty() {
            return Err(WithOffset::new_head(Self::Error::MissingBytes(1)));
        }
        Ok(WithSize {
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

/// Forward rest of the command over the interface
#[derive(Clone, Debug, PartialEq)]
pub struct Forward {
    // ALP_SPEC Ask for response ?
    pub resp: bool,
    pub conf: operand::InterfaceConfiguration,
}
impl std::fmt::Display for Forward {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}{}", if self.resp { "[R]" } else { "-" }, self.conf)
    }
}
impl Codec for Forward {
    type Error = operand::InterfaceConfigurationDecodingError;
    fn encoded_size(&self) -> usize {
        1 + self.conf.encoded_size()
    }
    unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
        out[0] = control_byte!(false, self.resp, OpCode::Forward);
        1 + self.conf.encode_in(&mut out[1..])
    }
    fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
        let min_size = 1 + 1;
        if out.len() < min_size {
            return Err(WithOffset::new(
                0,
                Self::Error::MissingBytes(min_size - out.len()),
            ));
        }
        let WithSize {
            value: conf,
            size: conf_size,
        } = operand::InterfaceConfiguration::decode(&out[1..]).map_err(|e| e.shift(1))?;
        Ok(WithSize {
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
            conf: operand::InterfaceConfiguration::Host,
        },
        &hex!("72 00"),
    )
}

/// Forward rest of the command over the interface
#[derive(Clone, Debug, PartialEq)]
pub struct IndirectForward {
    // ALP_SPEC Ask for response ?
    pub resp: bool,
    pub interface: operand::IndirectInterface,
}
impl std::fmt::Display for IndirectForward {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "[{}]{}",
            if self.resp { "R" } else { "-" },
            self.interface
        )
    }
}
impl Codec for IndirectForward {
    type Error = StdError;
    fn encoded_size(&self) -> usize {
        1 + self.interface.encoded_size()
    }
    unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
        let overload = match self.interface {
            operand::IndirectInterface::Overloaded(_) => true,
            operand::IndirectInterface::NonOverloaded(_) => false,
        };
        out[0] = control_byte!(overload, self.resp, OpCode::IndirectForward);
        1 + serialize_all!(&mut out[1..], &self.interface)
    }
    fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
        if out.is_empty() {
            Err(WithOffset::new_head(Self::Error::MissingBytes(1)))
        } else {
            let mut offset = 0;
            let WithSize {
                value: op1,
                size: op1_size,
            } = operand::IndirectInterface::decode(out)?;
            offset += op1_size;
            Ok(WithSize {
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
            interface: operand::IndirectInterface::Overloaded(
                operand::OverloadedIndirectInterface {
                    interface_file_id: 4,
                    nls_method: dash7::NlsMethod::AesCcm32,
                    access_class: 0xFF,
                    address: dash7::Address::Vid([0xAB, 0xCD]),
                },
            ),
        },
        &hex!("F3   04   37 FF ABCD"),
    )
}

/// Provide command payload identifier
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RequestTag {
    /// Ask for end of packet
    ///
    /// Signal the last response packet for the request `id`
    /// (E)
    pub eop: bool,
    pub id: u8,
}
impl std::fmt::Display for RequestTag {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "[{}]({})", if self.eop { "E" } else { "-" }, self.id)
    }
}
impl Codec for RequestTag {
    type Error = StdError;
    fn encoded_size(&self) -> usize {
        1 + 1
    }
    unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
        out[0] = control_byte!(self.eop, false, OpCode::RequestTag);
        out[1] = self.id;
        1 + 1
    }
    fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
        let min_size = 1 + 1;
        if out.len() < min_size {
            return Err(WithOffset::new(
                0,
                Self::Error::MissingBytes(min_size - out.len()),
            ));
        }
        Ok(WithSize {
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

/// TODO Panics
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Extension {
    /// Group with next action
    pub group: bool,
    /// Ask for a response
    pub resp: bool,
}
impl_display_simple_op!(Extension);
impl Codec for Extension {
    type Error = ();
    fn encoded_size(&self) -> usize {
        todo!()
    }
    unsafe fn encode_in(&self, _out: &mut [u8]) -> usize {
        todo!()
    }
    fn decode(_out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
        todo!()
    }
}

/// An ALP Action
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

impl Action {
    pub fn op_code(&self) -> OpCode {
        match self {
            // Nop
            Self::Nop(_) => OpCode::Nop,

            // Read
            Self::ReadFileData(_) => OpCode::ReadFileData,
            Self::ReadFileProperties(_) => OpCode::ReadFileProperties,

            // Write
            Self::WriteFileData(_) => OpCode::WriteFileData,
            // ALP SPEC: This is not specified even though it is implemented
            // Self::WriteFileDataFlush(_) => OpCode::WriteFileDataFlush,
            Self::WriteFileProperties(_) => OpCode::WriteFileProperties,
            Self::ActionQuery(_) => OpCode::ActionQuery,
            Self::BreakQuery(_) => OpCode::BreakQuery,
            Self::PermissionRequest(_) => OpCode::PermissionRequest,
            Self::VerifyChecksum(_) => OpCode::VerifyChecksum,

            // Management
            Self::ExistFile(_) => OpCode::ExistFile,
            Self::CreateNewFile(_) => OpCode::CreateNewFile,
            Self::DeleteFile(_) => OpCode::DeleteFile,
            Self::RestoreFile(_) => OpCode::RestoreFile,
            Self::FlushFile(_) => OpCode::FlushFile,
            Self::CopyFile(_) => OpCode::CopyFile,
            Self::ExecuteFile(_) => OpCode::ExecuteFile,

            // Response
            Self::ReturnFileData(_) => OpCode::ReturnFileData,
            Self::ReturnFileProperties(_) => OpCode::ReturnFileProperties,
            Self::Status(_) => OpCode::Status,
            Self::ResponseTag(_) => OpCode::ResponseTag,

            // Special
            Self::Chunk(_) => OpCode::Chunk,
            Self::Logic(_) => OpCode::Logic,
            Self::Forward(_) => OpCode::Forward,
            Self::IndirectForward(_) => OpCode::IndirectForward,
            Self::RequestTag(_) => OpCode::RequestTag,
            Self::Extension(_) => OpCode::Extension,
        }
    }
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let op_code = self.op_code();
        match self {
            // Nop
            Self::Nop(op) => write!(f, "{}{}", op_code, op),

            // Read
            Self::ReadFileData(op) => write!(f, "{}{}", op_code, op),
            Self::ReadFileProperties(op) => write!(f, "{}{}", op_code, op),

            // Write
            Self::WriteFileData(op) => write!(f, "{}{}", op_code, op),
            // ALP SPEC: This is not specified even though it is implemented
            // Self::WriteFileDataFlush(op) => write!(f, "{}{}", op_code, op),
            Self::WriteFileProperties(op) => write!(f, "{}{}", op_code, op),
            Self::ActionQuery(op) => write!(f, "{}{}", op_code, op),
            Self::BreakQuery(op) => write!(f, "{}{}", op_code, op),
            Self::PermissionRequest(op) => write!(f, "{}{}", op_code, op),
            Self::VerifyChecksum(op) => write!(f, "{}{}", op_code, op),

            // Management
            Self::ExistFile(op) => write!(f, "{}{}", op_code, op),
            Self::CreateNewFile(op) => write!(f, "{}{}", op_code, op),
            Self::DeleteFile(op) => write!(f, "{}{}", op_code, op),
            Self::RestoreFile(op) => write!(f, "{}{}", op_code, op),
            Self::FlushFile(op) => write!(f, "{}{}", op_code, op),
            Self::CopyFile(op) => write!(f, "{}{}", op_code, op),
            Self::ExecuteFile(op) => write!(f, "{}{}", op_code, op),

            // Response
            Self::ReturnFileData(op) => write!(f, "{}{}", op_code, op),
            Self::ReturnFileProperties(op) => write!(f, "{}{}", op_code, op),
            Self::Status(op) => write!(f, "{}{}", op_code, op),
            Self::ResponseTag(op) => write!(f, "{}{}", op_code, op),

            // Special
            Self::Chunk(op) => write!(f, "{}{}", op_code, op),
            Self::Logic(op) => write!(f, "{}{}", op_code, op),
            Self::Forward(op) => write!(f, "{}{}", op_code, op),
            Self::IndirectForward(op) => write!(f, "{}{}", op_code, op),
            Self::RequestTag(op) => write!(f, "{}{}", op_code, op),
            Self::Extension(op) => write!(f, "{}{}", op_code, op),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ActionDecodingError {
    NoData,
    UnknownOpCode(u8),
    Nop(StdError),
    ReadFileData(StdError),
    ReadFileProperties(StdError),
    WriteFileData(StdError),
    WriteFileProperties(HeaderActionDecodingError),
    ActionQuery(operand::QueryDecodingError),
    BreakQuery(operand::QueryDecodingError),
    PermissionRequest(PermissionRequestDecodingError),
    VerifyChecksum(operand::QueryDecodingError),
    ExistFile(StdError),
    CreateNewFile(HeaderActionDecodingError),
    DeleteFile(StdError),
    RestoreFile(StdError),
    FlushFile(StdError),
    CopyFile(StdError),
    ExecuteFile(StdError),
    ReturnFileData(StdError),
    ReturnFileProperties(HeaderActionDecodingError),
    Status(StatusDecodingError),
    ResponseTag(StdError),
    Chunk(StdError),
    Logic(StdError),
    Forward(operand::InterfaceConfigurationDecodingError),
    IndirectForward(StdError),
    RequestTag(StdError),
    Extension(()),
}

macro_rules! impl_std_error_map {
    ($name: ident, $variant: ident, $error: ty) => {
        fn $name(o: WithOffset<$error>) -> WithOffset<ActionDecodingError> {
            let WithOffset { offset, value } = o;
            WithOffset {
                offset,
                value: Self::$variant(value),
            }
        }
    };
}

impl ActionDecodingError {
    impl_std_error_map!(map_nop, Nop, StdError);
    impl_std_error_map!(map_read_file_data, ReadFileData, StdError);
    impl_std_error_map!(map_read_file_properties, ReadFileProperties, StdError);
    impl_std_error_map!(map_write_file_data, WriteFileData, StdError);
    impl_std_error_map!(
        map_write_file_properties,
        WriteFileProperties,
        HeaderActionDecodingError
    );
    impl_std_error_map!(map_action_query, ActionQuery, operand::QueryDecodingError);
    impl_std_error_map!(map_break_query, BreakQuery, operand::QueryDecodingError);
    impl_std_error_map!(
        map_permission_request,
        PermissionRequest,
        PermissionRequestDecodingError
    );
    impl_std_error_map!(
        map_verify_checksum,
        VerifyChecksum,
        operand::QueryDecodingError
    );
    impl_std_error_map!(map_exist_file, ExistFile, StdError);
    impl_std_error_map!(
        map_create_new_file,
        CreateNewFile,
        HeaderActionDecodingError
    );
    impl_std_error_map!(map_delete_file, DeleteFile, StdError);
    impl_std_error_map!(map_restore_file, RestoreFile, StdError);
    impl_std_error_map!(map_flush_file, FlushFile, StdError);
    impl_std_error_map!(map_copy_file, CopyFile, StdError);
    impl_std_error_map!(map_execute_file, ExecuteFile, StdError);
    impl_std_error_map!(map_return_file_data, ReturnFileData, StdError);
    impl_std_error_map!(
        map_return_file_properties,
        ReturnFileProperties,
        HeaderActionDecodingError
    );
    impl_std_error_map!(map_status, Status, StatusDecodingError);
    impl_std_error_map!(map_response_tag, ResponseTag, StdError);
    impl_std_error_map!(map_chunk, Chunk, StdError);
    impl_std_error_map!(map_logic, Logic, StdError);
    impl_std_error_map!(
        map_forward,
        Forward,
        operand::InterfaceConfigurationDecodingError
    );
    impl_std_error_map!(map_indirect_forward, IndirectForward, StdError);
    impl_std_error_map!(map_request_tag, RequestTag, StdError);
    impl_std_error_map!(map_extension, Extension, ());
}

impl Codec for Action {
    type Error = ActionDecodingError;
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
    unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
        match self {
            Action::Nop(x) => x.encode_in(out),
            Action::ReadFileData(x) => x.encode_in(out),
            Action::ReadFileProperties(x) => x.encode_in(out),
            Action::WriteFileData(x) => x.encode_in(out),
            // Action::WriteFileDataFlush(x) => x.encode_in(out),
            Action::WriteFileProperties(x) => x.encode_in(out),
            Action::ActionQuery(x) => x.encode_in(out),
            Action::BreakQuery(x) => x.encode_in(out),
            Action::PermissionRequest(x) => x.encode_in(out),
            Action::VerifyChecksum(x) => x.encode_in(out),
            Action::ExistFile(x) => x.encode_in(out),
            Action::CreateNewFile(x) => x.encode_in(out),
            Action::DeleteFile(x) => x.encode_in(out),
            Action::RestoreFile(x) => x.encode_in(out),
            Action::FlushFile(x) => x.encode_in(out),
            Action::CopyFile(x) => x.encode_in(out),
            Action::ExecuteFile(x) => x.encode_in(out),
            Action::ReturnFileData(x) => x.encode_in(out),
            Action::ReturnFileProperties(x) => x.encode_in(out),
            Action::Status(x) => x.encode_in(out),
            Action::ResponseTag(x) => x.encode_in(out),
            Action::Chunk(x) => x.encode_in(out),
            Action::Logic(x) => x.encode_in(out),
            Action::Forward(x) => x.encode_in(out),
            Action::IndirectForward(x) => x.encode_in(out),
            Action::RequestTag(x) => x.encode_in(out),
            Action::Extension(x) => x.encode_in(out),
        }
    }
    fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
        if out.is_empty() {
            return Err(WithOffset::new_head(Self::Error::NoData));
        }
        let opcode = OpCode::from(out[0] & 0x3F)
            .map_err(Self::Error::UnknownOpCode)
            .map_err(WithOffset::new_head)?;
        Ok(match opcode {
            OpCode::Nop => Nop::decode(out)
                .map_err(ActionDecodingError::map_nop)?
                .map_value(Action::Nop),
            OpCode::ReadFileData => ReadFileData::decode(out)
                .map_err(ActionDecodingError::map_read_file_data)?
                .map_value(Action::ReadFileData),
            OpCode::ReadFileProperties => ReadFileProperties::decode(out)
                .map_err(ActionDecodingError::map_read_file_properties)?
                .map_value(Action::ReadFileProperties),
            OpCode::WriteFileData => WriteFileData::decode(out)
                .map_err(ActionDecodingError::map_write_file_data)?
                .map_value(Action::WriteFileData),
            // OpCode::WriteFileDataFlush => {
            //     WriteFileDataFlush::decode(&out)?.map_value( Action::WriteFileDataFlush)
            // }
            OpCode::WriteFileProperties => WriteFileProperties::decode(out)
                .map_err(ActionDecodingError::map_write_file_properties)?
                .map_value(Action::WriteFileProperties),
            OpCode::ActionQuery => ActionQuery::decode(out)
                .map_err(ActionDecodingError::map_action_query)?
                .map_value(Action::ActionQuery),
            OpCode::BreakQuery => BreakQuery::decode(out)
                .map_err(ActionDecodingError::map_break_query)?
                .map_value(Action::BreakQuery),
            OpCode::PermissionRequest => PermissionRequest::decode(out)
                .map_err(ActionDecodingError::map_permission_request)?
                .map_value(Action::PermissionRequest),
            OpCode::VerifyChecksum => VerifyChecksum::decode(out)
                .map_err(ActionDecodingError::map_verify_checksum)?
                .map_value(Action::VerifyChecksum),
            OpCode::ExistFile => ExistFile::decode(out)
                .map_err(ActionDecodingError::map_exist_file)?
                .map_value(Action::ExistFile),
            OpCode::CreateNewFile => CreateNewFile::decode(out)
                .map_err(ActionDecodingError::map_create_new_file)?
                .map_value(Action::CreateNewFile),
            OpCode::DeleteFile => DeleteFile::decode(out)
                .map_err(ActionDecodingError::map_delete_file)?
                .map_value(Action::DeleteFile),
            OpCode::RestoreFile => RestoreFile::decode(out)
                .map_err(ActionDecodingError::map_restore_file)?
                .map_value(Action::RestoreFile),
            OpCode::FlushFile => FlushFile::decode(out)
                .map_err(ActionDecodingError::map_flush_file)?
                .map_value(Action::FlushFile),
            OpCode::CopyFile => CopyFile::decode(out)
                .map_err(ActionDecodingError::map_copy_file)?
                .map_value(Action::CopyFile),
            OpCode::ExecuteFile => ExecuteFile::decode(out)
                .map_err(ActionDecodingError::map_execute_file)?
                .map_value(Action::ExecuteFile),
            OpCode::ReturnFileData => ReturnFileData::decode(out)
                .map_err(ActionDecodingError::map_return_file_data)?
                .map_value(Action::ReturnFileData),
            OpCode::ReturnFileProperties => ReturnFileProperties::decode(out)
                .map_err(ActionDecodingError::map_return_file_properties)?
                .map_value(Action::ReturnFileProperties),
            OpCode::Status => Status::decode(out)
                .map_err(ActionDecodingError::map_status)?
                .map_value(Action::Status),
            OpCode::ResponseTag => ResponseTag::decode(out)
                .map_err(ActionDecodingError::map_response_tag)?
                .map_value(Action::ResponseTag),
            OpCode::Chunk => Chunk::decode(out)
                .map_err(ActionDecodingError::map_chunk)?
                .map_value(Action::Chunk),
            OpCode::Logic => Logic::decode(out)
                .map_err(ActionDecodingError::map_logic)?
                .map_value(Action::Logic),
            OpCode::Forward => Forward::decode(out)
                .map_err(ActionDecodingError::map_forward)?
                .map_value(Action::Forward),
            OpCode::IndirectForward => IndirectForward::decode(out)
                .map_err(ActionDecodingError::map_indirect_forward)?
                .map_value(Action::IndirectForward),
            OpCode::RequestTag => RequestTag::decode(out)
                .map_err(ActionDecodingError::map_request_tag)?
                .map_value(Action::RequestTag),
            OpCode::Extension => Extension::decode(out)
                .map_err(ActionDecodingError::map_extension)?
                .map_value(Action::Extension),
        })
    }
}

#[test]
fn test_nop_display() {
    assert_eq!(
        Action::Nop(Nop {
            resp: false,
            group: true
        })
        .to_string(),
        "NOP[G-]"
    );
}

#[test]
fn test_read_file_data_display() {
    assert_eq!(
        Action::ReadFileData(ReadFileData {
            resp: false,
            group: true,
            file_id: 1,
            offset: 2,
            size: 3,
        })
        .to_string(),
        "RD[G-]f(1,2,3)"
    );
}

#[test]
fn test_read_file_properties_display() {
    assert_eq!(
        Action::ReadFileProperties(ReadFileProperties {
            resp: true,
            group: false,
            file_id: 1,
        })
        .to_string(),
        "RDP[-R]f(1)"
    );
}

#[test]
fn test_write_file_data_display() {
    assert_eq!(
        Action::WriteFileData(WriteFileData {
            resp: true,
            group: false,
            file_id: 1,
            offset: 2,
            data: Box::new([3, 4, 5]),
        })
        .to_string(),
        "WR[-R]f(1,2,0x030405)"
    );
}

#[test]
fn test_write_file_properties_display() {
    assert_eq!(
        Action::WriteFileProperties(WriteFileProperties {
            resp: false,
            group: true,
            file_id: 1,
            header: data::FileHeader {
                permissions: data::Permissions {
                    encrypted: true,
                    executable: false,
                    user: data::UserPermissions {
                        read: true,
                        write: true,
                        run: true,
                    },
                    guest: data::UserPermissions {
                        read: false,
                        write: false,
                        run: false,
                    },
                },
                properties: data::FileProperties {
                    act_en: false,
                    act_cond: data::ActionCondition::Read,
                    storage_class: data::StorageClass::Permanent,
                },
                alp_cmd_fid: 1,
                interface_file_id: 2,
                file_size: 3,
                allocated_size: 4,
            }
        })
        .to_string(),
        "WRP[G-]f(1)[E-|user=RWX|guest=---|0RP|f(1),2,3,4]"
    );
}

#[test]
fn test_action_query_display() {
    assert_eq!(
        Action::ActionQuery(ActionQuery {
            group: true,
            resp: true,
            query: operand::Query::BitmapRangeComparison(operand::BitmapRangeComparison {
                signed_data: false,
                comparison_type: operand::QueryRangeComparisonType::InRange,
                size: 2,

                start: 3,
                stop: 32,
                mask: Some(Box::new(hex!("01020304"))),

                file: operand::FileOffset { id: 0, offset: 4 },
            },),
        })
        .to_string(),
        "AQ[GR]BM:[U|1,2,3-32,msk=0x01020304,f(0,4)]"
    );
    assert_eq!(
        Action::ActionQuery(ActionQuery {
            group: true,
            resp: true,
            query: operand::Query::ComparisonWithZero(operand::ComparisonWithZero {
                signed_data: true,
                comparison_type: operand::QueryComparisonType::Inequal,
                size: 3,
                mask: Some(vec![0, 1, 2].into_boxed_slice()),
                file: operand::FileOffset { id: 4, offset: 5 },
            }),
        })
        .to_string(),
        "AQ[GR]WZ:[S|NEQ,3,msk=0x000102,f(4,5)]"
    );
}

#[test]
fn test_break_query_display() {
    assert_eq!(
        Action::BreakQuery(BreakQuery {
            group: true,
            resp: true,
            query: operand::Query::NonVoid(operand::NonVoid {
                size: 4,
                file: operand::FileOffset { id: 5, offset: 6 },
            }),
        })
        .to_string(),
        "BQ[GR]NV:[4,f(5,6)]"
    );
    assert_eq!(
        Action::BreakQuery(BreakQuery {
            group: true,
            resp: true,
            query: operand::Query::ComparisonWithOtherFile(operand::ComparisonWithOtherFile {
                signed_data: false,
                comparison_type: operand::QueryComparisonType::GreaterThan,
                size: 2,
                mask: Some(vec![0xF1, 0xF2].into_boxed_slice()),
                file1: operand::FileOffset { id: 4, offset: 5 },
                file2: operand::FileOffset { id: 8, offset: 9 },
            }),
        })
        .to_string(),
        "BQ[GR]WF:[U|GTH,2,msk=0xF1F2,f(4,5)~f(8,9)]"
    );
}

#[test]
fn test_permission_request_display() {
    assert_eq!(
        Action::PermissionRequest(PermissionRequest {
            group: false,
            resp: true,
            level: 1,
            permission: operand::Permission::Dash7([2, 3, 4, 5, 6, 7, 8, 9]),
        })
        .to_string(),
        "PR[-R]1,D7:0x0203040506070809"
    );
}

#[test]
fn test_verify_checksum_display() {
    assert_eq!(
        Action::VerifyChecksum(VerifyChecksum {
            group: false,
            resp: false,
            query: operand::Query::ComparisonWithValue(operand::ComparisonWithValue {
                signed_data: false,
                comparison_type: operand::QueryComparisonType::GreaterThan,
                size: 2,
                mask: Some(vec![0xF1, 0xF2].into_boxed_slice()),
                value: Box::new([0xA9, 0xA8]),
                file: operand::FileOffset { id: 4, offset: 5 },
            }),
        })
        .to_string(),
        "VCS[--]WV:[U|GTH,2,msk=0xF1F2,v=0xA9A8,f(4,5)]"
    );
    assert_eq!(
        Action::VerifyChecksum(VerifyChecksum {
            group: true,
            resp: false,
            query: operand::Query::StringTokenSearch(operand::StringTokenSearch {
                max_errors: 2,
                size: 4,
                mask: Some(Box::new(hex!("FF00FF00"))),
                value: Box::new(hex!("01020304")),
                file: operand::FileOffset { id: 0, offset: 4 },
            }),
        })
        .to_string(),
        "VCS[G-]ST:[2,4,msk=0xFF00FF00,v=0x01020304,f(0,4)]"
    );
}

#[test]
fn test_exist_file_display() {
    assert_eq!(
        Action::ExistFile(ExistFile {
            group: false,
            resp: true,
            file_id: 9,
        })
        .to_string(),
        "HAS[-R]f(9)"
    )
}

#[test]
fn test_create_new_file_display() {
    assert_eq!(
        Action::CreateNewFile(CreateNewFile {
            group: true,
            resp: false,
            file_id: 6,
            header: data::FileHeader {
                permissions: data::Permissions {
                    encrypted: true,
                    executable: false,
                    user: data::UserPermissions {
                        read: true,
                        write: true,
                        run: true,
                    },
                    guest: data::UserPermissions {
                        read: false,
                        write: false,
                        run: false,
                    },
                },
                properties: data::FileProperties {
                    act_en: false,
                    act_cond: data::ActionCondition::Read,
                    storage_class: data::StorageClass::Permanent,
                },
                alp_cmd_fid: 1,
                interface_file_id: 2,
                file_size: 3,
                allocated_size: 4,
            }
        })
        .to_string(),
        "NEW[G-]f(6)[E-|user=RWX|guest=---|0RP|f(1),2,3,4]"
    )
}

#[test]
fn test_delete_file_display() {
    assert_eq!(
        Action::DeleteFile(DeleteFile {
            group: false,
            resp: true,
            file_id: 7,
        })
        .to_string(),
        "DEL[-R]f(7)"
    )
}

#[test]
fn test_restore_file_display() {
    assert_eq!(
        Action::RestoreFile(RestoreFile {
            group: false,
            resp: true,
            file_id: 5,
        })
        .to_string(),
        "RST[-R]f(5)"
    )
}

#[test]
fn test_flush_file_display() {
    assert_eq!(
        Action::FlushFile(FlushFile {
            group: false,
            resp: true,
            file_id: 4,
        })
        .to_string(),
        "FLUSH[-R]f(4)"
    )
}

#[test]
fn test_copy_file_display() {
    assert_eq!(
        Action::CopyFile(CopyFile {
            group: false,
            resp: true,
            src_file_id: 2,
            dst_file_id: 8,
        })
        .to_string(),
        "CP[-R]f(2)f(8)"
    )
}

#[test]
fn test_execute_file_display() {
    assert_eq!(
        Action::ExecuteFile(ExecuteFile {
            group: false,
            resp: true,
            file_id: 4,
        })
        .to_string(),
        "RUN[-R]f(4)"
    )
}

#[test]
fn test_return_file_data_display() {
    assert_eq!(
        Action::ReturnFileData(ReturnFileData {
            resp: true,
            group: false,
            file_id: 1,
            offset: 2,
            data: Box::new([3, 4, 5]),
        })
        .to_string(),
        "DATA[-R]f(1,2,0x030405)"
    );
}

#[test]
fn test_return_file_properties_display() {
    assert_eq!(
        Action::ReturnFileProperties(ReturnFileProperties {
            resp: false,
            group: true,
            file_id: 1,
            header: data::FileHeader {
                permissions: data::Permissions {
                    encrypted: true,
                    executable: false,
                    user: data::UserPermissions {
                        read: true,
                        write: true,
                        run: true,
                    },
                    guest: data::UserPermissions {
                        read: false,
                        write: false,
                        run: false,
                    },
                },
                properties: data::FileProperties {
                    act_en: false,
                    act_cond: data::ActionCondition::Read,
                    storage_class: data::StorageClass::Permanent,
                },
                alp_cmd_fid: 1,
                interface_file_id: 2,
                file_size: 3,
                allocated_size: 4,
            }
        })
        .to_string(),
        "PROP[G-]f(1)[E-|user=RWX|guest=---|0RP|f(1),2,3,4]"
    );
}

#[test]
fn test_status_display() {
    assert_eq!(
        Action::Status(Status::Action(operand::Status {
            action_id: 2,
            status: 4
        }))
        .to_string(),
        "S[ACT]:a[2]=>4"
    );
    assert_eq!(
        Action::Status(Status::Interface(operand::InterfaceStatus::Host)).to_string(),
        "S[ITF]:HOST"
    );
    assert_eq!(
        Action::Status(Status::Interface(operand::InterfaceStatus::D7asp(
            dash7::InterfaceStatus {
                ch_header: 1,
                ch_idx: 0x0123,
                rxlev: 2,
                lb: 3,
                snr: 4,
                status: 5,
                token: 6,
                seq: 7,
                resp_to: 8,
                access_class: 0xFF,
                address: dash7::Address::Vid([0xAB, 0xCD]),
                nls_state: dash7::NlsState::AesCcm32(hex!("00 11 22 33 44")),
            }
        )))
        .to_string(),
        "S[ITF]:D7=ch(1;291),sig(2,3,4),s=5,tok=6,sq=7,rto=8,xclass=0xFF,VID[ABCD],NLS[7|0011223344]"
    );
}

#[test]
fn test_response_tag_display() {
    assert_eq!(
        Action::ResponseTag(ResponseTag {
            eop: true,
            err: false,
            id: 8,
        })
        .to_string(),
        "TG[E-](8)"
    );
}

#[test]
fn test_chunk_display() {
    assert_eq!(
        Action::Chunk(Chunk {
            step: ChunkStep::Start
        })
        .to_string(),
        "CHK[S]"
    );
}

#[test]
fn test_logic_display() {
    assert_eq!(
        Action::Logic(Logic {
            logic: LogicOp::Xor
        })
        .to_string(),
        "LOG[XOR]"
    );
}

#[test]
fn test_forward_display() {
    assert_eq!(
        Action::Forward(Forward {
            resp: true,
            conf: operand::InterfaceConfiguration::Host,
        })
        .to_string(),
        "FWD[R]HOST"
    );
    assert_eq!(
        Action::Forward(Forward {
            resp: true,
            conf: operand::InterfaceConfiguration::D7asp(dash7::InterfaceConfiguration {
                qos: dash7::Qos {
                    retry: dash7::RetryMode::No,
                    resp: dash7::RespMode::Any,
                },
                to: 0x23,
                te: 0x34,
                nls_method: dash7::NlsMethod::AesCcm32,
                access_class: 0xFF,
                address: dash7::Address::Vid([0xAB, 0xCD]),
            }),
        })
        .to_string(),
        "FWD[R]D7:0X,35,52|0xFF,NLS[7],VID[ABCD]"
    );
}

#[test]
fn test_indirect_forward_display() {
    assert_eq!(
        Action::IndirectForward(IndirectForward {
            resp: true,
            interface: operand::IndirectInterface::Overloaded(
                operand::OverloadedIndirectInterface {
                    interface_file_id: 4,
                    nls_method: dash7::NlsMethod::AesCcm32,
                    access_class: 0xFF,
                    address: dash7::Address::Vid([0xAB, 0xCD]),
                }
            ),
        })
        .to_string(),
        "IFWD[R]O:4,NLS[7],255,VID[ABCD]"
    );
}

#[test]
fn test_request_tag_display() {
    assert_eq!(
        Action::RequestTag(RequestTag { eop: true, id: 9 }).to_string(),
        "RTG[E](9)"
    );
}

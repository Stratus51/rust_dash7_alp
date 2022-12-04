#[cfg(test)]
use crate::{test_tools::test_item, v1_2::dash7};
#[cfg(test)]
use hex_literal::hex;

use crate::{
    codec::{Codec, StdError, WithOffset, WithSize},
    v1_2::operand,
};

pub mod chunk;
pub mod copy_file;
pub mod file_data_action;
pub mod file_id_action;
pub mod file_properties_action;
pub mod forward;
pub mod indirect_forward;
pub mod logic;
pub mod nop;
pub mod permission_request;
pub mod query_action;
pub mod read_file_data;
pub mod request_tag;
pub mod response_tag;
pub mod status;

pub use chunk::Chunk;
pub use copy_file::CopyFile;
pub use file_data_action::FileDataAction;
pub use file_id_action::FileIdAction;
pub use file_properties_action::FilePropertiesAction;
pub use forward::Forward;
pub use indirect_forward::IndirectForward;
pub use logic::Logic;
pub use nop::Nop;
pub use permission_request::PermissionRequest;
pub use query_action::QueryAction;
pub use read_file_data::ReadFileData;
pub use request_tag::RequestTag;
pub use response_tag::ResponseTag;
pub use status::Status;

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
pub(crate) use serialize_all;

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
pub(crate) use encoded_size;

macro_rules! impl_op_serialized {
    ($name: ident, $flag7: ident, $flag6: ident, $op1: ident, $op1_type: ty, $error: ty) => {
        impl crate::codec::Codec for $name {
            type Error = $error;
            fn encoded_size(&self) -> usize {
                1 + crate::v1_2::action::encoded_size!(self.$op1)
            }
            unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
                out[0] |= ((self.$flag7 as u8) << 7) | ((self.$flag6 as u8) << 6);
                1 + crate::v1_2::action::serialize_all!(&mut out[1..], &self.$op1)
            }
            fn decode(
                out: &[u8],
            ) -> Result<crate::codec::WithSize<Self>, crate::codec::WithOffset<Self::Error>> {
                if (out.is_empty()) {
                    Err(crate::v1_2::WithOffset::new_head(
                        Self::Error::MissingBytes(1),
                    ))
                } else {
                    let mut offset = 1;
                    let crate::v1_2::WithSize {
                        size: op1_size,
                        value: op1,
                    } = <$op1_type>::decode(&out[offset..]).map_err(|e| e.shift(offset))?;
                    offset += op1_size;
                    Ok(crate::v1_2::WithSize {
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
pub(crate) use impl_op_serialized;

macro_rules! unsafe_varint_serialize_sizes {
    ( $($x: expr),* ) => {{
        let mut ret = 0;
            $(unsafe {
                ret += varint::size($x);
            })*
        ret
    }}
}
pub(crate) use unsafe_varint_serialize_sizes;

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
pub(crate) use unsafe_varint_serialize;

macro_rules! count {
    () => (0usize);
    ( $x:tt $($xs:tt)* ) => (1usize + crate::v1_2::action::count!($($xs)*));
}
pub(crate) use count;

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
pub(crate) use build_simple_op;

macro_rules! impl_simple_op {
    ($name: ident, $flag7: ident, $flag6: ident, $($x: ident),* ) => {
        impl Codec for $name {
            type Error = StdError;
            fn encoded_size(&self) -> usize {
                1 + crate::v1_2::action::count!($( $x )*)
            }
            unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
                out[0] |= ((self.$flag7 as u8) << 7) | ((self.$flag6 as u8) << 6);
                let mut offset = 1;
                $({
                    out[offset] = self.$x;
                    offset += 1;
                })*
                1 + offset
            }
            fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
                const SIZE: usize = 1 + crate::v1_2::action::count!($( $x )*);
                if(out.len() < SIZE) {
                    Err(WithOffset::new_head( Self::Error::MissingBytes(SIZE - out.len())))
                } else {
                    Ok(WithSize {
                        size: SIZE,
                        value: crate::v1_2::action::build_simple_op!($name, out, $flag7, $flag6, $($x),*),
                    })
                }
            }
        }
    };
}
pub(crate) use impl_simple_op;

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
pub(crate) use impl_display_simple_op;

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
pub(crate) use impl_display_simple_file_op;

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
pub(crate) use impl_display_data_file_op;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum HeaderActionDecodingError {
    MissingBytes(usize),
    FileHeader(StdError),
}

macro_rules! impl_header_op {
    ($name: ident, $flag7: ident, $flag6: ident, $file_id: ident, $file_header: ident) => {
        impl Codec for $name {
            type Error = crate::v1_2::action::HeaderActionDecodingError;
            fn encoded_size(&self) -> usize {
                1 + 1 + 12
            }
            unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
                out[0] |= ((self.$flag7 as u8) << 7) | ((self.$flag6 as u8) << 6);
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
pub(crate) use impl_header_op;

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

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum OperandValidationError {
    /// Offset is too big to be encoded in a varint
    OffsetTooBig,
    /// Size is too big to be encoded in a varint
    SizeTooBig,
}

/// An ALP Action
#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    // Nop
    Nop(Nop),

    // Read
    ReadFileData(ReadFileData),
    ReadFileProperties(FileIdAction),

    // Write
    WriteFileData(FileDataAction),
    // ALP SPEC: This is not specified even though it is implemented
    // WriteFileDataActionFlush(WriteFileDataActionFlush),
    WriteFileProperties(FilePropertiesAction),
    ActionQuery(QueryAction),
    BreakQuery(QueryAction),
    PermissionRequest(PermissionRequest),
    VerifyChecksum(QueryAction),

    // Management
    ExistFile(FileIdAction),
    CreateNewFile(FilePropertiesAction),
    DeleteFile(FileIdAction),
    RestoreFile(FileIdAction),
    FlushFile(FileIdAction),
    CopyFile(CopyFile),
    ExecuteFile(FileIdAction),

    // Response
    ReturnFileData(FileDataAction),
    ReturnFileProperties(FilePropertiesAction),
    Status(Status),
    ResponseTag(ResponseTag),

    // Special
    Chunk(Chunk),
    Logic(Logic),
    Forward(Forward),
    IndirectForward(IndirectForward),
    RequestTag(RequestTag),
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
    PermissionRequest(operand::PermissionDecodingError),
    VerifyChecksum(operand::QueryDecodingError),
    ExistFile(StdError),
    CreateNewFile(HeaderActionDecodingError),
    DeleteFile(StdError),
    RestoreFile(StdError),
    FlushFile(StdError),
    CopyFile(StdError),
    ExecuteFile(StdError),
    ReturnFileDataAction(StdError),
    ReturnFilePropertiesAction(HeaderActionDecodingError),
    Status(status::StatusDecodingError),
    ResponseTag(StdError),
    Chunk(StdError),
    Logic(StdError),
    Forward(operand::InterfaceConfigurationDecodingError),
    IndirectForward(StdError),
    RequestTag(StdError),
    Extension,
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
        operand::PermissionDecodingError
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
    impl_std_error_map!(map_return_file_data, ReturnFileDataAction, StdError);
    impl_std_error_map!(
        map_return_file_properties,
        ReturnFilePropertiesAction,
        HeaderActionDecodingError
    );
    impl_std_error_map!(map_status, Status, status::StatusDecodingError);
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
        }
    }
    unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
        out[0] = self.op_code() as u8;
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
            OpCode::ReadFileProperties => FileIdAction::decode(out)
                .map_err(ActionDecodingError::map_read_file_properties)?
                .map_value(Action::ReadFileProperties),
            OpCode::WriteFileData => FileDataAction::decode(out)
                .map_err(ActionDecodingError::map_write_file_data)?
                .map_value(Action::WriteFileData),
            // OpCode::WriteFileDataActionFlush => {
            //     WriteFileDataActionFlush::decode(&out)?.map_value( Action::WriteFileDataActionFlush)
            // }
            OpCode::WriteFileProperties => FilePropertiesAction::decode(out)
                .map_err(ActionDecodingError::map_write_file_properties)?
                .map_value(Action::WriteFileProperties),
            OpCode::ActionQuery => QueryAction::decode(out)
                .map_err(ActionDecodingError::map_action_query)?
                .map_value(Action::ActionQuery),
            OpCode::BreakQuery => QueryAction::decode(out)
                .map_err(ActionDecodingError::map_break_query)?
                .map_value(Action::BreakQuery),
            OpCode::PermissionRequest => PermissionRequest::decode(out)
                .map_err(ActionDecodingError::map_permission_request)?
                .map_value(Action::PermissionRequest),
            OpCode::VerifyChecksum => QueryAction::decode(out)
                .map_err(ActionDecodingError::map_verify_checksum)?
                .map_value(Action::VerifyChecksum),
            OpCode::ExistFile => FileIdAction::decode(out)
                .map_err(ActionDecodingError::map_exist_file)?
                .map_value(Action::ExistFile),
            OpCode::CreateNewFile => FilePropertiesAction::decode(out)
                .map_err(ActionDecodingError::map_create_new_file)?
                .map_value(Action::CreateNewFile),
            OpCode::DeleteFile => FileIdAction::decode(out)
                .map_err(ActionDecodingError::map_delete_file)?
                .map_value(Action::DeleteFile),
            OpCode::RestoreFile => FileIdAction::decode(out)
                .map_err(ActionDecodingError::map_restore_file)?
                .map_value(Action::RestoreFile),
            OpCode::FlushFile => FileIdAction::decode(out)
                .map_err(ActionDecodingError::map_flush_file)?
                .map_value(Action::FlushFile),
            OpCode::CopyFile => CopyFile::decode(out)
                .map_err(ActionDecodingError::map_copy_file)?
                .map_value(Action::CopyFile),
            OpCode::ExecuteFile => FileIdAction::decode(out)
                .map_err(ActionDecodingError::map_execute_file)?
                .map_value(Action::ExecuteFile),
            OpCode::ReturnFileData => FileDataAction::decode(out)
                .map_err(ActionDecodingError::map_return_file_data)?
                .map_value(Action::ReturnFileData),
            OpCode::ReturnFileProperties => FilePropertiesAction::decode(out)
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
            OpCode::Extension => return Err(WithOffset::new_head(ActionDecodingError::Extension)),
        })
    }
}

#[cfg(test)]
mod test_codec {
    use super::*;
    use crate::v1_2::data;

    #[test]
    fn nop() {
        test_item(
            Action::Nop(Nop {
                group: true,
                resp: false,
            }),
            &hex!("80"),
        )
    }
    #[test]
    fn read_file_data() {
        test_item(
            Action::ReadFileData(ReadFileData {
                group: false,
                resp: true,
                file_id: 1,
                offset: 2,
                size: 3,
            }),
            &hex!("41 01 02 03"),
        )
    }

    macro_rules! impl_file_data_test {
        ($name: ident, $test_name: ident) => {
            #[test]
            fn $test_name() {
                test_item(
                    Action::$name(FileDataAction {
                        group: false,
                        resp: true,
                        file_id: 9,
                        offset: 5,
                        data: Box::new(hex!("01 02 03")),
                    }),
                    &vec![
                        [crate::v1_2::action::OpCode::$name as u8 | (1 << 6)].as_slice(),
                        &hex!("09 05 03  010203"),
                    ]
                    .concat()[..],
                )
            }
        };
    }
    impl_file_data_test!(WriteFileData, write_file_data);
    impl_file_data_test!(ReturnFileData, return_file_data);

    macro_rules! impl_file_properties_test {
        ($name: ident, $test_name: ident) => {
            #[test]
            fn $test_name() {
                test_item(
                    Action::$name(FilePropertiesAction {
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
                    }),
                    &vec![
                        [crate::v1_2::action::OpCode::$name as u8 | (1 << 7)].as_slice(),
                        &hex!("09   B8 13 01 02 DEADBEEF BAADFACE"),
                    ]
                    .concat()[..],
                )
            }
        };
    }
    impl_file_properties_test!(WriteFileProperties, write_file_properties);
    impl_file_properties_test!(CreateNewFile, create_new_file);
    impl_file_properties_test!(ReturnFileProperties, return_file_properties);

    macro_rules! impl_query_test {
        ($name: ident, $test_name: ident) => {
            #[test]
            fn $test_name() {
                crate::test_tools::test_item(
                    Action::$name(QueryAction {
                        group: true,
                        resp: true,
                        query: crate::v1_2::operand::Query::NonVoid(
                            crate::v1_2::operand::NonVoid {
                                size: 4,
                                file: crate::v1_2::operand::FileOffset { id: 5, offset: 6 },
                            },
                        ),
                    }),
                    &vec![
                        [crate::v1_2::action::OpCode::$name as u8 | (3 << 6)].as_slice(),
                        &hex_literal::hex!("00 04  05 06"),
                    ]
                    .concat()[..],
                )
            }
        };
    }
    impl_query_test!(ActionQuery, action_query);
    impl_query_test!(BreakQuery, break_query);
    impl_query_test!(VerifyChecksum, verify_checksum);

    #[test]
    fn permission_request() {
        test_item(
            Action::PermissionRequest(PermissionRequest {
                group: false,
                resp: false,
                level: crate::v1_2::operand::permission_level::ROOT,
                permission: operand::Permission::Dash7(hex!("0102030405060708")),
            }),
            &hex!("0A   01 42 0102030405060708"),
        )
    }

    macro_rules! impl_file_id {
        ($name: ident, $test_name: ident) => {
            #[test]
            fn $test_name() {
                test_item(
                    Action::$name(FileIdAction {
                        group: false,
                        resp: false,
                        file_id: 9,
                    }),
                    &[crate::v1_2::action::OpCode::$name as u8, 0x09],
                )
            }
        };
    }
    impl_file_id!(ReadFileProperties, test_read_file_properties);
    impl_file_id!(ExistFile, test_exist_file);
    impl_file_id!(DeleteFile, test_delete_file);
    impl_file_id!(RestoreFile, test_restore_file);
    impl_file_id!(FlushFile, test_flush_file);
    impl_file_id!(ExecuteFile, test_execute_file);

    #[test]
    fn copy_file() {
        test_item(
            Action::CopyFile(CopyFile {
                group: false,
                resp: false,
                src_file_id: 0x42,
                dst_file_id: 0x24,
            }),
            &hex!("17 42 24"),
        )
    }

    #[test]
    fn status() {
        test_item(
            Action::Status(Status::Action(operand::ActionStatus {
                action_id: 2,
                status: operand::status_code::UNKNOWN_OPERATION,
            })),
            &hex!("22 02 F6"),
        )
    }

    #[test]
    fn response_tag() {
        test_item(
            Action::ResponseTag(ResponseTag {
                eop: true,
                err: false,
                id: 8,
            }),
            &hex!("A3 08"),
        )
    }

    #[test]
    fn chunk() {
        test_item(Action::Chunk(Chunk::End), &hex!("B0"))
    }

    #[test]
    fn logic() {
        test_item(Action::Logic(Logic::Nand), &hex!("F1"))
    }

    #[test]
    fn forward() {
        test_item(
            Action::Forward(Forward {
                resp: true,
                conf: operand::InterfaceConfiguration::Host,
            }),
            &hex!("72 00"),
        )
    }

    #[test]
    fn indirect_forward() {
        test_item(
            Action::IndirectForward(IndirectForward {
                resp: true,
                interface: operand::IndirectInterface::Overloaded(
                    operand::OverloadedIndirectInterface {
                        interface_file_id: 4,
                        nls_method: dash7::NlsMethod::AesCcm32,
                        access_class: 0xFF,
                        address: dash7::Address::Vid([0xAB, 0xCD]),
                    },
                ),
            }),
            &hex!("F3   04   37 FF ABCD"),
        )
    }

    #[test]
    fn request_tag() {
        test_item(
            Action::RequestTag(RequestTag { eop: true, id: 8 }),
            &hex!("B4 08"),
        )
    }
}

#[cfg(test)]
mod test_display {
    use super::*;
    use crate::v1_2::data;

    #[test]
    fn nop() {
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
    fn read_file_data() {
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
    fn read_file_properties() {
        assert_eq!(
            Action::ReadFileProperties(FileIdAction {
                resp: true,
                group: false,
                file_id: 1,
            })
            .to_string(),
            "RDP[-R]f(1)"
        );
    }

    #[test]
    fn write_file_data() {
        assert_eq!(
            Action::WriteFileData(FileDataAction {
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
    fn write_file_properties() {
        assert_eq!(
            Action::WriteFileProperties(FilePropertiesAction {
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
    fn action_query() {
        assert_eq!(
            Action::ActionQuery(QueryAction {
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
            Action::ActionQuery(QueryAction {
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
    fn break_query() {
        assert_eq!(
            Action::BreakQuery(QueryAction {
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
            Action::BreakQuery(QueryAction {
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
    fn permission_request() {
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
    fn verify_checksum() {
        assert_eq!(
            Action::VerifyChecksum(QueryAction {
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
            Action::VerifyChecksum(QueryAction {
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
    fn exist_file() {
        assert_eq!(
            Action::ExistFile(FileIdAction {
                group: false,
                resp: true,
                file_id: 9,
            })
            .to_string(),
            "HAS[-R]f(9)"
        )
    }

    #[test]
    fn create_new_file() {
        assert_eq!(
            Action::CreateNewFile(FilePropertiesAction {
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
    fn delete_file() {
        assert_eq!(
            Action::DeleteFile(FileIdAction {
                group: false,
                resp: true,
                file_id: 7,
            })
            .to_string(),
            "DEL[-R]f(7)"
        )
    }

    #[test]
    fn restore_file() {
        assert_eq!(
            Action::RestoreFile(FileIdAction {
                group: false,
                resp: true,
                file_id: 5,
            })
            .to_string(),
            "RST[-R]f(5)"
        )
    }

    #[test]
    fn flush_file() {
        assert_eq!(
            Action::FlushFile(FileIdAction {
                group: false,
                resp: true,
                file_id: 4,
            })
            .to_string(),
            "FLUSH[-R]f(4)"
        )
    }

    #[test]
    fn copy_file() {
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
    fn execute_file() {
        assert_eq!(
            Action::ExecuteFile(FileIdAction {
                group: false,
                resp: true,
                file_id: 4,
            })
            .to_string(),
            "RUN[-R]f(4)"
        )
    }

    #[test]
    fn return_file_data() {
        assert_eq!(
            Action::ReturnFileData(FileDataAction {
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
    fn return_file_properties() {
        assert_eq!(
            Action::ReturnFileProperties(FilePropertiesAction {
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
    fn status() {
        assert_eq!(
            Action::Status(Status::Action(operand::ActionStatus {
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
    fn response_tag() {
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
    fn chunk() {
        assert_eq!(Action::Chunk(Chunk::Start).to_string(), "CHK[S]");
    }

    #[test]
    fn logic() {
        assert_eq!(Action::Logic(Logic::Xor).to_string(), "LOG[XOR]");
    }

    #[test]
    fn forward() {
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
    fn indirect_forward() {
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
    fn request_tag() {
        assert_eq!(
            Action::RequestTag(RequestTag { eop: true, id: 9 }).to_string(),
            "RTG[E](9)"
        );
    }
}

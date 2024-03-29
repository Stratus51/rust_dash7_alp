#[cfg(test)]
use crate::{test_tools::test_item, wizzilab::v5_3::dash7};
#[cfg(test)]
use hex_literal::hex;

use super::operand;
use crate::codec::{Codec, StdError, WithOffset, WithSize};
pub use crate::spec::v1_2::action::{
    Chunk, CopyFile, FileDataAction, FileIdAction, FilePropertiesAction, HeaderActionDecodingError,
    Logic, Nop, OpCode as SpecOpCode, PermissionRequest, QueryAction, ReadFileData, RequestTag,
    ResponseTag,
};
pub use status::Status;

pub mod flow;
pub mod forward;
pub mod indirect_forward;
pub mod status;
pub mod tx_status;

pub use flow::{Flow, FlowSeqnum};
pub use forward::Forward;
pub use indirect_forward::IndirectForward;
pub use tx_status::TxStatus;

// ===============================================================================
// Opcodes
// ===============================================================================
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OpCode {
    // Nop
    Nop = SpecOpCode::Nop as isize,

    // Read
    ReadFileData = SpecOpCode::ReadFileData as isize,
    ReadFileProperties = SpecOpCode::ReadFileProperties as isize,

    // Write
    WriteFileData = SpecOpCode::WriteFileData as isize,
    WriteFileDataFlush = 5,
    WriteFileProperties = SpecOpCode::WriteFileProperties as isize,
    ActionQuery = SpecOpCode::ActionQuery as isize,
    BreakQuery = SpecOpCode::BreakQuery as isize,
    PermissionRequest = SpecOpCode::PermissionRequest as isize,
    VerifyChecksum = SpecOpCode::VerifyChecksum as isize,

    // Management
    ExistFile = SpecOpCode::ExistFile as isize,
    CreateNewFile = SpecOpCode::CreateNewFile as isize,
    DeleteFile = SpecOpCode::DeleteFile as isize,
    RestoreFile = SpecOpCode::RestoreFile as isize,
    FlushFile = SpecOpCode::FlushFile as isize,
    CopyFile = SpecOpCode::CopyFile as isize,
    ExecuteFile = SpecOpCode::ExecuteFile as isize,

    // Response
    ReturnFileData = SpecOpCode::ReturnFileData as isize,
    ReturnFileProperties = SpecOpCode::ReturnFileProperties as isize,
    Status = SpecOpCode::Status as isize,
    ResponseTag = SpecOpCode::ResponseTag as isize,
    TxStatus = 38,

    // Special
    Chunk = SpecOpCode::Chunk as isize,
    Logic = SpecOpCode::Logic as isize,
    Forward = SpecOpCode::Forward as isize,
    IndirectForward = SpecOpCode::IndirectForward as isize,
    RequestTag = SpecOpCode::RequestTag as isize,
    Flow = 54,
    Extension = SpecOpCode::Extension as isize,
}
impl OpCode {
    pub(crate) fn from(n: u8) -> Result<Self, u8> {
        Ok(match n {
            // Nop
            0 => OpCode::Nop,

            // Read
            1 => OpCode::ReadFileData,
            2 => OpCode::ReadFileProperties,

            // Write
            4 => OpCode::WriteFileData,
            5 => OpCode::WriteFileDataFlush,
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
            38 => OpCode::TxStatus,

            // Special
            48 => OpCode::Chunk,
            49 => OpCode::Logic,
            50 => OpCode::Forward,
            51 => OpCode::IndirectForward,
            52 => OpCode::RequestTag,
            54 => OpCode::Flow,
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
            OpCode::ReadFileData => write!(f, "R"),
            OpCode::ReadFileProperties => write!(f, "RP"),

            // Write
            OpCode::WriteFileData => write!(f, "W"),
            OpCode::WriteFileDataFlush => write!(f, "WF"),
            OpCode::WriteFileProperties => write!(f, "WP"),
            OpCode::ActionQuery => write!(f, "AQ"),
            OpCode::BreakQuery => write!(f, "BQ"),
            OpCode::PermissionRequest => write!(f, "PRM"),
            OpCode::VerifyChecksum => write!(f, "VCS"),

            // Management
            OpCode::ExistFile => write!(f, "HAS"),
            OpCode::CreateNewFile => write!(f, "NEW"),
            OpCode::DeleteFile => write!(f, "DEL"),
            OpCode::RestoreFile => write!(f, "RST"),
            OpCode::FlushFile => write!(f, "FLSH"),
            OpCode::CopyFile => write!(f, "CP"),
            OpCode::ExecuteFile => write!(f, "RUN"),

            // Response
            OpCode::ReturnFileData => write!(f, "DATA"),
            OpCode::ReturnFileProperties => write!(f, "PROP"),
            OpCode::Status => write!(f, "S"),
            OpCode::ResponseTag => write!(f, "TAG"),
            OpCode::TxStatus => write!(f, "TXS"),

            // Special
            OpCode::Chunk => write!(f, "CHK"),
            OpCode::Logic => write!(f, "LOG"),
            OpCode::Forward => write!(f, "FWD"),
            OpCode::IndirectForward => write!(f, "IFWD"),
            OpCode::RequestTag => write!(f, "RTAG"),
            OpCode::Flow => write!(f, "FLOW"),
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
    WriteFileDataFlush(FileDataAction),
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
    TxStatus(TxStatus),

    // Special
    Chunk(Chunk),
    Logic(Logic),
    Forward(Forward),
    IndirectForward(IndirectForward),
    RequestTag(RequestTag),
    Flow(Flow),
}
crate::spec::v1_2::action::impl_action_builders!(Action);

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
            Self::WriteFileDataFlush(_) => OpCode::WriteFileDataFlush,
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
            Self::TxStatus(_) => OpCode::TxStatus,

            // Special
            Self::Chunk(_) => OpCode::Chunk,
            Self::Logic(_) => OpCode::Logic,
            Self::Forward(_) => OpCode::Forward,
            Self::IndirectForward(_) => OpCode::IndirectForward,
            Self::RequestTag(_) => OpCode::RequestTag,
            Self::Flow(_) => OpCode::Flow,
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
            Self::WriteFileDataFlush(op) => write!(f, "{}{}", op_code, op),
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
            Self::TxStatus(op) => write!(f, "{}{}", op_code, op),

            // Special
            Self::Chunk(op) => write!(f, "{}{}", op_code, op),
            Self::Logic(op) => write!(f, "{}{}", op_code, op),
            Self::Forward(op) => write!(f, "{}{}", op_code, op),
            Self::IndirectForward(op) => write!(f, "{}{}", op_code, op),
            Self::RequestTag(op) => write!(f, "{}{}", op_code, op),
            Self::Flow(op) => write!(f, "{}{}", op_code, op),
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
    TxStatus(tx_status::TxStatusDecodingError),
    Chunk(StdError),
    Logic(StdError),
    Forward(operand::InterfaceConfigurationDecodingError),
    IndirectForward(StdError),
    RequestTag(StdError),
    Flow(StdError),
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
    impl_std_error_map!(map_tx_status, TxStatus, tx_status::TxStatusDecodingError);
    impl_std_error_map!(map_chunk, Chunk, StdError);
    impl_std_error_map!(map_logic, Logic, StdError);
    impl_std_error_map!(
        map_forward,
        Forward,
        operand::InterfaceConfigurationDecodingError
    );
    impl_std_error_map!(map_indirect_forward, IndirectForward, StdError);
    impl_std_error_map!(map_request_tag, RequestTag, StdError);
    impl_std_error_map!(map_flow, Flow, StdError);
}

impl Codec for Action {
    type Error = ActionDecodingError;
    fn encoded_size(&self) -> usize {
        match self {
            Action::Nop(x) => x.encoded_size(),
            Action::ReadFileData(x) => x.encoded_size(),
            Action::ReadFileProperties(x) => x.encoded_size(),
            Action::WriteFileData(x) => x.encoded_size(),
            Action::WriteFileDataFlush(x) => x.encoded_size(),
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
            Action::TxStatus(x) => x.encoded_size(),
            Action::Chunk(x) => x.encoded_size(),
            Action::Logic(x) => x.encoded_size(),
            Action::Forward(x) => x.encoded_size(),
            Action::IndirectForward(x) => x.encoded_size(),
            Action::RequestTag(x) => x.encoded_size(),
            Action::Flow(x) => x.encoded_size(),
        }
    }
    unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
        out[0] = self.op_code() as u8;
        match self {
            Action::Nop(x) => x.encode_in(out),
            Action::ReadFileData(x) => x.encode_in(out),
            Action::ReadFileProperties(x) => x.encode_in(out),
            Action::WriteFileData(x) => x.encode_in(out),
            Action::WriteFileDataFlush(x) => x.encode_in(out),
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
            Action::TxStatus(x) => x.encode_in(out),
            Action::Chunk(x) => x.encode_in(out),
            Action::Logic(x) => x.encode_in(out),
            Action::Forward(x) => x.encode_in(out),
            Action::IndirectForward(x) => x.encode_in(out),
            Action::RequestTag(x) => x.encode_in(out),
            Action::Flow(x) => x.encode_in(out),
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
            OpCode::WriteFileDataFlush => FileDataAction::decode(out)
                .map_err(ActionDecodingError::map_write_file_data)?
                .map_value(Action::WriteFileDataFlush),
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
            OpCode::TxStatus => TxStatus::decode(out)
                .map_err(ActionDecodingError::map_tx_status)?
                .map_value(Action::TxStatus),
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
            OpCode::Flow => Flow::decode(out)
                .map_err(ActionDecodingError::map_flow)?
                .map_value(Action::Flow),
            OpCode::Extension => return Err(WithOffset::new_head(ActionDecodingError::Extension)),
        })
    }
}

#[cfg(test)]
mod test_codec {
    use super::*;
    use crate::spec::v1_2::data;

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
                        [crate::spec::v1_2::action::OpCode::$name as u8 | (1 << 6)].as_slice(),
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
                        [crate::spec::v1_2::action::OpCode::$name as u8 | (1 << 7)].as_slice(),
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
                        query: crate::spec::v1_2::operand::Query::NonVoid(
                            crate::spec::v1_2::operand::NonVoid {
                                size: 4,
                                file: crate::spec::v1_2::operand::FileOffset { id: 5, offset: 6 },
                            },
                        ),
                    }),
                    &vec![
                        [crate::spec::v1_2::action::OpCode::$name as u8 | (3 << 6)].as_slice(),
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
                level: crate::spec::v1_2::operand::permission_level::ROOT,
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
                    &[crate::spec::v1_2::action::OpCode::$name as u8, 0x09],
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
                status: operand::StatusCode::UnknownOperation,
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
    fn tx_status() {
        test_item(
            Action::TxStatus(TxStatus::Interface(operand::InterfaceTxStatus::D7asp(
                dash7::interface_tx_status::InterfaceTxStatus {
                    ch_header: 1,
                    ch_idx: 0x0123,
                    eirp: 2,
                    err: dash7::stack_error::InterfaceFinalStatusCode::Busy,
                    rfu_0: 4,
                    rfu_1: 5,
                    rfu_2: 6,
                    lts: 0x0708_0000,
                    access_class: 0xFF,
                    nls_method: dash7::NlsMethod::AesCcm64,
                    address: dash7::Address::Vid([0x00, 0x11]),
                },
            ))),
            &hex!("66 D7 16    01 0123 02 FF 04 05 06 0000 0807  36 FF 0011 000000000000"),
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
            &hex!("F3   04   37 FF ABCD  000000000000"),
        )
    }

    #[test]
    fn request_tag() {
        test_item(
            Action::RequestTag(RequestTag { eop: true, id: 8 }),
            &hex!("B4 08"),
        )
    }

    #[test]
    fn flow() {
        let raw = "36 FD 0004";
        let raw = hex::decode(raw.replace(' ', "")).unwrap();
        test_item(
            Action::Flow(Flow {
                flow: 0xFD,
                seqnum: FlowSeqnum::U16(0x0004),
            }),
            &raw,
        )
    }
}

#[cfg(test)]
mod test_display {
    use super::*;
    use crate::spec::v1_2::data;

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
            "R[G-]f(1,2,3)"
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
            "RP[-R]f(1)"
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
            "W[-R]f(1,2,0x030405)"
        );
    }

    #[test]
    fn write_file_data_flush() {
        assert_eq!(
            Action::WriteFileDataFlush(FileDataAction {
                resp: true,
                group: false,
                file_id: 1,
                offset: 2,
                data: Box::new([3, 4, 5]),
            })
            .to_string(),
            "WF[-R]f(1,2,0x030405)"
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
            "WP[G-]f(1)[E-|user=RWX|guest=---|0RP|f(1),2,3,4]"
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
            "PRM[-R]1,D7:0x0203040506070809"
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
            "FLSH[-R]f(4)"
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
                status: operand::StatusCode::UnknownError,
            }))
            .to_string(),
            "S[ACT]:a[2]=>E_?"
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
                    fof: 9,
                    access_class: 0xFF,
                    address: dash7::Address::Vid([0xAB, 0xCD]),
                    nls_state: dash7::NlsState::AesCcm32(hex!("00 11 22 33 44")),
                }
            )))
            .to_string(),
            "S[ITF]:D7=ch(1;291),sig(2,3,4),s=5,tok=6,sq=7,rto=8,fof=9,xcl=0xFF,VID[ABCD],NLS[7|0011223344]"
        );
        assert_eq!(
            Action::Status(Status::InterfaceFinal(operand::InterfaceFinalStatus {
                interface: dash7::spec::operand::InterfaceId::D7asp,
                len: 1,
                status: dash7::stack_error::InterfaceFinalStatusCode::Busy
            }))
            .to_string(),
            "S[ITF_END]:f_itf[D7][1]=>BUSY"
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
            "TAG[E-](8)"
        );
    }

    #[test]
    fn tx_status() {
        assert_eq!(
            Action::TxStatus(TxStatus::Interface(operand::InterfaceTxStatus::Host)).to_string(),
            "TXS[ITF]:HOST"
        );
        assert_eq!(
            Action::TxStatus(TxStatus::Interface(operand::InterfaceTxStatus::D7asp(
                dash7::interface_tx_status::InterfaceTxStatus {
                    ch_header: 1,
                    ch_idx: 0x0123,
                    eirp: 2,
                    err: dash7::stack_error::InterfaceFinalStatusCode::Busy,
                    rfu_0: 4,
                    rfu_1: 5,
                    rfu_2: 6,
                    lts: 0x0708_0000,
                    access_class: 0xFF,
                    nls_method: dash7::NlsMethod::AesCcm128,
                    address: dash7::Address::Vid([0x00, 0x11]),
                }
            )))
            .to_string(),
            "TXS[ITF]:D7=ch(1;291),eirp=2,err=BUSY,lts=117964800,address=VID[0011]"
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
                        retry: dash7::RetryMode::Oneshot,
                        resp: dash7::RespMode::Any,
                    },
                    to: 0x23,
                    te: 0x34,
                    nls_method: dash7::NlsMethod::AesCcm32,
                    access_class: 0xFF,
                    address: dash7::Address::Vid([0xAB, 0xCD]),
                    use_vid: false,
                    group_condition: dash7::GroupCondition::Any,
                }),
            })
            .to_string(),
            "FWD[R]D7:0X,35,52|0xFF,use_vid=false,NLS[7],GCD=X,VID[ABCD]"
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
            "RTAG[E](9)"
        );
    }

    #[test]
    fn consistency() {
        use crate::spec::v1_2 as spec;
        macro_rules! cmp_str {
            ($name: ident, $op: expr) => {
                assert_eq!(
                    Action::$name($op.clone().into()).to_string(),
                    spec::Action::$name($op.clone().into()).to_string()
                );
            };
        }

        let op = Nop {
            resp: false,
            group: true,
        };
        cmp_str!(Nop, op);
        let op = ReadFileData {
            resp: false,
            group: true,
            file_id: 1,
            offset: 2,
            size: 3,
        };
        cmp_str!(ReadFileData, op);

        let op = FileDataAction {
            group: false,
            resp: true,
            file_id: 9,
            offset: 5,
            data: Box::new(hex!("01 02 03")),
        };
        cmp_str!(WriteFileData, op);
        cmp_str!(ReturnFileData, op);

        let op = FilePropertiesAction {
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
        };
        cmp_str!(WriteFileProperties, op);
        cmp_str!(CreateNewFile, op);
        cmp_str!(ReturnFileProperties, op);

        let op = QueryAction {
            group: true,
            resp: true,
            query: crate::spec::v1_2::operand::Query::NonVoid(
                crate::spec::v1_2::operand::NonVoid {
                    size: 4,
                    file: crate::spec::v1_2::operand::FileOffset { id: 5, offset: 6 },
                },
            ),
        };
        cmp_str!(ActionQuery, op);
        cmp_str!(BreakQuery, op);
        cmp_str!(VerifyChecksum, op);

        let op = PermissionRequest {
            group: false,
            resp: false,
            level: crate::spec::v1_2::operand::permission_level::ROOT,
            permission: operand::Permission::Dash7(hex!("0102030405060708")),
        };
        cmp_str!(PermissionRequest, op);

        let op = FileIdAction {
            resp: true,
            group: false,
            file_id: 1,
        };
        cmp_str!(ReadFileProperties, op);
        cmp_str!(ExistFile, op);
        cmp_str!(DeleteFile, op);
        cmp_str!(RestoreFile, op);
        cmp_str!(FlushFile, op);
        cmp_str!(ExecuteFile, op);

        let op = CopyFile {
            group: false,
            resp: false,
            src_file_id: 0x42,
            dst_file_id: 0x24,
        };
        cmp_str!(CopyFile, op);

        let op = spec::action::Status::Action(spec::operand::ActionStatus {
            action_id: 2,
            status: spec::operand::StatusCode::UnknownOperation,
        });
        cmp_str!(Status, op);

        let op = ResponseTag {
            eop: true,
            err: false,
            id: 8,
        };
        cmp_str!(ResponseTag, op);

        // let op = TxStatus::Interface(operand::InterfaceTxStatus::Host);
        // cmp_str!(TxStatus, op);

        let op = Chunk::End;
        cmp_str!(Chunk, op);

        let op = Logic::Nand;
        cmp_str!(Logic, op);

        let op = Forward {
            resp: true,
            conf: operand::InterfaceConfiguration::Host,
        };
        cmp_str!(Forward, op);

        let op = IndirectForward {
            resp: true,
            interface: operand::IndirectInterface::Overloaded(
                operand::OverloadedIndirectInterface {
                    interface_file_id: 4,
                    nls_method: dash7::NlsMethod::AesCcm32,
                    access_class: 0xFF,
                    address: dash7::Address::Vid([0xAB, 0xCD]),
                },
            ),
        };
        cmp_str!(IndirectForward, op);

        let op = RequestTag { eop: true, id: 8 };
        cmp_str!(RequestTag, op);
    }
}

pub use crate::spec::v1_2::operand::{
    BitmapRangeComparison, ComparisonWithOtherFile, ComparisonWithValue, ComparisonWithZero,
    FileOffset, IndirectInterface, InterfaceStatus, InterfaceStatusDecodingError, NonVoid,
    OverloadedIndirectInterface, Permission, PermissionDecodingError, Query, QueryCode,
    QueryComparisonType, QueryDecodingError, QueryRangeComparisonType, StringTokenSearch,
};

pub mod interface_configuration;
pub use interface_configuration::{
    InterfaceConfiguration, InterfaceConfigurationDecodingError, InterfaceId,
};

pub mod action_status;
pub use action_status::{ActionStatus, ActionStatusDecodingError, StatusCode};

pub mod interface_final_status;
pub use interface_final_status::{InterfaceFinalStatus, InterfaceFinalStatusDecodingError};

pub use crate::spec::v1_2::operand::{
    status_code, ActionStatus, BitmapRangeComparison, ComparisonWithOtherFile, ComparisonWithValue,
    ComparisonWithZero, FileOffset, IndirectInterface, InterfaceStatus, NonVoid,
    OverloadedIndirectInterface, Permission, PermissionDecodingError, Query, QueryCode,
    QueryComparisonType, QueryDecodingError, QueryRangeComparisonType, StringTokenSearch,
};

pub mod interface_configuration;
pub use interface_configuration::{
    InterfaceConfiguration, InterfaceConfigurationDecodingError, InterfaceId,
};

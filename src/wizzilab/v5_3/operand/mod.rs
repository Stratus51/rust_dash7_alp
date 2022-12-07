pub use crate::spec::v1_2::operand::{
    ActionStatus, BitmapRangeComparison, ComparisonWithOtherFile, ComparisonWithValue,
    ComparisonWithZero, FileOffset, IndirectInterface, InterfaceStatus, NonVoid,
    OverloadedIndirectInterface, Permission, PermissionDecodingError, Query, QueryCode,
    QueryComparisonType, QueryDecodingError, QueryRangeComparisonType, StatusCode,
    StringTokenSearch,
};

pub mod interface_configuration;
pub use interface_configuration::{
    InterfaceConfiguration, InterfaceConfigurationDecodingError, InterfaceId,
};

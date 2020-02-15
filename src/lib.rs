use std::convert::TryFrom;

mod serializable;
use serializable::Serializable;

// TODO Maybe using flat structures and modeling operands as macros would be much more ergonomic.
// TODO Look into const function to replace some macros?

// =================================================================================
// Macros
// =================================================================================
macro_rules! serialize_all {
    ($out: expr, $($x: expr),*) => {
        {
            let mut offset = 0;
            $({
                offset += $x.serialize(&mut $out[offset..]);
            })*
            offset
        }
    }
}

macro_rules! serialized_size {
    ( $($x: expr),* ) => {
        {
            let mut total = 0;
            $({
                total += $x.serialized_size();
            })*
            total
        }
    }
}

// Derive replacement (proc-macro would not allow this to be a normal lib)
macro_rules! impl_serialized {
    ( $name: ident, $($x: ident),* ) => {
        impl Serializable for $name {
            fn serialized_size(&self) -> usize {
                serialized_size!($({ self.$x }),*)
            }
            fn serialize(&self, out: &mut [u8]) -> usize {
                serialize_all!(out, $({ self.$x }),*)
            }
        }
    }
}

macro_rules! control_byte {
    ($flag7: expr, $flag6: expr, $op_code: expr) => {{
        let ctrl = $op_code as u8;
        if $flag7 {
            ctrl |= 0x80;
        }
        if $flag6 {
            ctrl |= 0x40;
        }
        ctrl
    }};
}

macro_rules! op_serialize {
    ($out: expr, $flag7: expr, $flag6: expr, $op_code: expr, $($x: expr),* ) => {{
        out[0] = control_byte!($flag7, $flag6, $op_code);
        1 + serialize_all!(&mut $out[1..])
    }};
}

// TODO
macro_rules! impl_op_serialized {
    ($name: ident, $flags: expr, $op_code: expr, $($x: expr),* ) => {{
        impl Serializable for ReadFileData {
            fn serialized_size(&self) -> usize {
                1 + self.data.serialized_size()
            }
            fn serialize(&self, out: &mut [u8]) -> usize {
                out[0] = control_byte!(self.group, self.resp, OpCode::ReadFileData);
                1 + self.data.serialize(&mut out[1..])
            }
        }
    }};
}

// =================================================================================
// Opcodes
// =================================================================================
pub enum OpCode {
    // Nop
    Nop = 0,

    // Read
    ReadFileData = 1,
    ReadFileProperties = 2,

    // Write
    WriteFileData = 4,
    WriteFileDataFlush = 5,
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

// =================================================================================
// Operands
// =================================================================================
pub struct VariableUint {
    value: u32,
}
const MAX_VARIABLE_UINT: u32 = 0x3F_FF_FF_FF;

impl VariableUint {
    pub fn new(value: u32) -> Result<Self, ()> {
        if value > MAX_VARIABLE_UINT {
            Err(())
        } else {
            Ok(Self { value })
        }
    }

    pub fn set(&mut self, value: u32) -> Result<(), ()> {
        if value > MAX_VARIABLE_UINT {
            Err(())
        } else {
            self.value = value;
            Ok(())
        }
    }

    pub fn is_valid(n: u32) -> Result<(), ()> {
        if n > MAX_VARIABLE_UINT {
            Err(())
        } else {
            Ok(())
        }
    }

    pub fn usize_is_valid(n: usize) -> Result<(), ()> {
        u32::try_from(n)
            .map_err(|_| ())
            .and_then(|n| Self::is_valid(n))
    }

    fn unsafe_size(n: u32) -> u8 {
        if n < 0x3F {
            0
        } else if n < 0x3F_FF {
            1
        } else if n < 0x3F_FF_FF {
            2
        } else {
            3
        }
    }

    fn size(n: u32) -> Result<u8, ()> {
        if n > MAX_VARIABLE_UINT {
            Err(())
        } else {
            Ok(Self::unsafe_size(n))
        }
    }

    unsafe fn u32_serialize(n: u32, out: &mut [u8]) -> u8 {
        let u8_size = Self::unsafe_size(n);
        let size = u8_size as usize;
        for i in 0..size {
            out[i] = ((n >> ((size - 1 - i) * 8)) & 0xFF) as u8;
        }
        out[0] |= (size as u8) << 6;
        u8_size
    }
}

impl Serializable for VariableUint {
    fn serialized_size(&self) -> usize {
        Self::unsafe_size(self.value) as usize
    }

    fn serialize(&self, out: &mut [u8]) -> usize {
        Self::u32_serialize(self.value, out) as usize
    }
}

struct FileIdOperand {
    file_id: u8,
}
impl Serializable for FileIdOperand {
    fn serialized_size(&self) -> usize {
        1
    }
    fn serialize(&self, out: &mut [u8]) -> usize {
        out[0] = self.file_id;
        1
    }
}

struct FileOffsetOperand {
    file_id: FileIdOperand,
    offset: VariableUint,
}
impl_serialized!(FileOffsetOperand, file_id, offset);

struct FileDataRequestOperand {
    file_offset: FileOffsetOperand,
    size: VariableUint,
}
impl_serialized!(FileDataRequestOperand, file_offset, size);

struct DataOperand {
    data: Box<[u8]>,
}
impl DataOperand {
    pub fn new(data: Box<[u8]>) -> Result<Self, ()> {
        VariableUint::usize_is_valid(data.len()).map(|_| Self { data })
    }
    pub fn set(&mut self, data: Box<[u8]>) -> Result<(), ()> {
        VariableUint::usize_is_valid(data.len()).map(|_| {
            self.data = data;
        })
    }
}

impl Serializable for DataOperand {
    fn serialized_size(&self) -> usize {
        VariableUint::size(self.data.len() as u32).unwrap() as usize + self.data.len()
    }
    fn serialize(&self, out: &mut [u8]) -> usize {
        let offset = VariableUint::u32_serialize(self.data.len() as u32, out) as usize;
        out[offset..].clone_from_slice(&self.data[..]);
        offset + self.data.len()
    }
}

struct FileDataOperand {
    file_offset: FileOffsetOperand,
    data: DataOperand,
}
impl_serialized!(FileDataOperand, file_offset, data);

// TODO
// ALP SPEC: Missing link to find definition in ALP spec
struct FileProperties {
    data: [u8; 12],
}
impl Serializable for FileProperties {
    fn serialized_size(&self) -> usize {
        12
    }
    fn serialize(&self, out: &mut [u8]) -> usize {
        todo!()
    }
}

struct FileHeader {
    file_id: FileIdOperand,
    data: FileProperties,
}
impl_serialized!(FileHeader, file_id, data);

enum StatusCode {
    Received = 1,
    Ok = 0,
    FileIdMissing = 0xFF,
    CreateFileIdAlreadyExist = 0xFE,
    FileIsNotRestorable = 0xFD,
    InsufficientPermission = 0xFC,
    CreateFileLengthOverflow = 0xFB,
    CreateFileAllocationOverflow = 0xFA, // ??? Difference with the previous one?
    WriteOffsetOverflow = 0xF9,
    WriteDataOverflow = 0xF8,
    WriteStorageUnavailable = 0xF7,
    UnknownOperation = 0xF6,
    OperandIncomplete = 0xF5,
    OperandWrongFormat = 0xF4,
    UnknownError = 0x80,
}
struct StatusOperand {
    action_index: u8,
    status: StatusCode,
}
impl Serializable for StatusOperand {
    fn serialized_size(&self) -> usize {
        1 + 1
    }
    fn serialize(&self, out: &mut [u8]) -> usize {
        out[0] = self.action_index;
        out[1] = self.status as u8;
        2
    }
}

// ALP SPEC: where is this defined? Link?
enum Permission {
    Dash7([u8; 8]), // TODO Check
}

impl Permission {
    fn id(&self) -> u8 {
        match self {
            Permission::Dash7(_) => 42,
        }
    }
}

impl Serializable for Permission {
    fn serialized_size(&self) -> usize {
        1 + match self {
            Permission::Dash7(_) => 8,
        }
    }
    fn serialize(&self, out: &mut [u8]) -> usize {
        out[0] = self.id();
        1 + match self {
            Permission::Dash7(token) => {
                out[1..].clone_from_slice(&token[..]);
                8
            }
        }
    }
}

enum PermissionLevel {
    User = 0,
    Root = 1,
}
impl Serializable for PermissionLevel {
    fn serialized_size(&self) -> usize {
        1
    }
    fn serialize(&self, out: &mut [u8]) -> usize {
        out[0] = *self as u8;
        1
    }
}

struct PermissionOperand {
    level: PermissionLevel,
    permission: Permission,
}
impl_serialized!(PermissionOperand, level, permission);

enum QueryComparisonType {
    Inequal = 0,
    Equal = 1,
    LessThan = 2,
    LessThanOrEqual = 3,
    GreaterThan = 4,
    GreaterThanOrEqual = 5,
}

enum QueryRangeComparisonType {
    NotInRange = 0,
    InRange = 1,
}

struct NonVoid {
    size: VariableUint,
    file_offset: FileOffsetOperand,
}
impl Serializable for NonVoid {
    fn serialized_size(&self) -> usize {
        1 + serialized_size!(self.size, self.file_offset)
    }
    fn serialize(&self, out: &mut [u8]) -> usize {
        out[0] = 0;
        1 + serialize_all!(&mut out[1..], self.size, self.file_offset)
    }
}
// TODO Check size coherence upon creation
struct ComparisonWithZero {
    signed_data: bool,
    comparison_type: QueryComparisonType,
    size: VariableUint,
    mask: Option<Box<[u8]>>,
    file_offset: FileOffsetOperand,
}
impl Serializable for ComparisonWithZero {
    fn serialized_size(&self) -> usize {
        let mask_size = match self.mask {
            Some(_) => self.size.value as usize,
            None => 0,
        };
        1 + self.size.serialized_size() + mask_size + self.file_offset.serialized_size()
    }
    fn serialize(&self, out: &mut [u8]) -> usize {
        const query_op: u8 = 1;
        let mask_flag = match self.mask {
            Some(_) => 1,
            None => 0,
        };
        let signed_flag = if self.signed_data { 1 } else { 0 };
        let offset = 0;
        out[0] =
            (query_op << 4) + (mask_flag << 3) + (signed_flag << 3) + self.comparison_type as u8;
        offset += 1;
        offset += self.size.serialize(&mut out[offset..]);
        if let Some(mask) = self.mask {
            out[offset..].clone_from_slice(&mask);
            offset += mask.len();
        }
        offset += self.file_offset.serialize(&mut out[offset..]);
        offset
    }
}
// TODO Check size coherence upon creation
struct ComparisonWithValue {
    signed_data: bool,
    comparison_type: QueryComparisonType,
    size: VariableUint,
    mask: Option<Box<[u8]>>,
    value: Box<[u8]>,
    file_offset: FileOffsetOperand,
}
impl Serializable for ComparisonWithValue {
    fn serialized_size(&self) -> usize {
        let mask_size = match self.mask {
            Some(_) => self.size.value as usize,
            None => 0,
        };
        1 + self.size.serialized_size()
            + mask_size
            + self.value.len()
            + self.file_offset.serialized_size()
    }
    fn serialize(&self, out: &mut [u8]) -> usize {
        const query_op: u8 = 2;
        let mask_flag = match self.mask {
            Some(_) => 1,
            None => 0,
        };
        let signed_flag = if self.signed_data { 1 } else { 0 };
        let offset = 0;
        out[0] =
            (query_op << 4) + (mask_flag << 3) + (signed_flag << 3) + self.comparison_type as u8;
        offset += 1;
        offset += self.size.serialize(&mut out[offset..]);
        if let Some(mask) = self.mask {
            out[offset..].clone_from_slice(&mask);
            offset += mask.len();
        }
        out[offset..].clone_from_slice(&self.value[..]);
        offset += self.value.len();
        offset += self.file_offset.serialize(&mut out[offset..]);
        offset
    }
}
// TODO Check size coherence upon creation
struct ComparisonWithOtherFile {
    signed_data: bool,
    comparison_type: QueryComparisonType,
    size: VariableUint,
    mask: Option<Box<[u8]>>,
    file_offset_src: FileOffsetOperand,
    file_offset_dst: FileOffsetOperand,
}
impl Serializable for ComparisonWithOtherFile {
    fn serialized_size(&self) -> usize {
        let mask_size = match self.mask {
            Some(_) => self.size.value as usize,
            None => 0,
        };
        1 + self.size.serialized_size()
            + mask_size
            + self.file_offset_src.serialized_size()
            + self.file_offset_dst.serialized_size()
    }
    fn serialize(&self, out: &mut [u8]) -> usize {
        const query_op: u8 = 3;
        let mask_flag = match self.mask {
            Some(_) => 1,
            None => 0,
        };
        let signed_flag = if self.signed_data { 1 } else { 0 };
        let offset = 0;
        out[0] =
            (query_op << 4) + (mask_flag << 3) + (signed_flag << 3) + self.comparison_type as u8;
        offset += 1;
        offset += self.size.serialize(&mut out[offset..]);
        if let Some(mask) = self.mask {
            out[offset..].clone_from_slice(&mask);
            offset += mask.len();
        }
        // ALP SPEC: Which of the offset operand is the source and the dest? (file 1 and 2)
        offset += self.file_offset_src.serialize(&mut out[offset..]);
        offset += self.file_offset_dst.serialize(&mut out[offset..]);
        offset
    }
}
// TODO Check size coherence upon creation (start, stop and bitmap)
struct BitmapRangeComparison {
    signed_data: bool,
    comparison_type: QueryRangeComparisonType,
    size: VariableUint,
    start: Box<[u8]>,
    stop: Box<[u8]>,
    bitmap: Box<[u8]>, // TODO Better type?
    file_offset: FileOffsetOperand,
}
impl Serializable for BitmapRangeComparison {
    fn serialized_size(&self) -> usize {
        1 + self.size.serialized_size()
            + 2 * self.size.value as usize
            + self.bitmap.len()
            + self.file_offset.serialized_size()
    }
    fn serialize(&self, out: &mut [u8]) -> usize {
        const query_op: u8 = 4;
        let offset = 0;
        let signed_flag = if self.signed_data { 1 } else { 0 };
        out[0] = (query_op << 4) + (0 << 3) + (signed_flag << 3) + self.comparison_type as u8;
        offset += 1;
        offset += self.size.serialize(&mut out[offset..]);
        out[offset..].clone_from_slice(&self.start[..]);
        offset += self.start.len();
        out[offset..].clone_from_slice(&self.stop[..]);
        offset += self.stop.len();
        out[offset..].clone_from_slice(&self.bitmap[..]);
        offset += self.bitmap.len();
        offset += self.file_offset.serialize(&mut out[offset..]);
        offset
    }
}
// TODO Check size coherence upon creation
struct StringTokenSearch {
    max_errors: u8,
    size: VariableUint,
    mask: Option<Box<[u8]>>,
    value: Box<[u8]>,
    file_offset: FileOffsetOperand,
}
impl Serializable for StringTokenSearch {
    fn serialized_size(&self) -> usize {
        let mask_size = match self.mask {
            Some(_) => self.size.value as usize,
            None => 0,
        };
        1 + self.size.serialized_size()
            + mask_size
            + self.value.len()
            + self.file_offset.serialized_size()
    }
    fn serialize(&self, out: &mut [u8]) -> usize {
        const query_op: u8 = 7;
        let mask_flag = match self.mask {
            Some(_) => 1,
            None => 0,
        };
        let offset = 0;
        out[0] = (query_op << 4) + (mask_flag << 3) + (0 << 3) + self.max_errors;
        offset += 1;
        offset += self.size.serialize(&mut out[offset..]);
        if let Some(mask) = self.mask {
            out[offset..].clone_from_slice(&mask);
            offset += mask.len();
        }
        out[offset..].clone_from_slice(&self.value[..]);
        offset += self.value.len();
        offset += self.file_offset.serialize(&mut out[offset..]);
        offset
    }
}

enum QueryOperand {
    NonVoid(NonVoid),
    ComparisonWithZero(ComparisonWithZero),
    ComparisonWithArgument(ComparisonWithValue),
    ComparisonWithOtherFile(ComparisonWithOtherFile),
    BitmapRangeComparison(BitmapRangeComparison),
    StringTokenSearch(StringTokenSearch),
}
impl QueryOperand {
    fn id(&self) -> u8 {
        match self {
            QueryOperand::NonVoid(_) => 0,
            QueryOperand::ComparisonWithZero(_) => 1,
            QueryOperand::ComparisonWithArgument(_) => 2,
            QueryOperand::ComparisonWithOtherFile(_) => 3,
            QueryOperand::BitmapRangeComparison(_) => 4,
            QueryOperand::StringTokenSearch(_) => 7,
        }
    }
}
impl Serializable for QueryOperand {
    fn serialized_size(&self) -> usize {
        match self {
            QueryOperand::NonVoid(v) => v.serialized_size(),
            QueryOperand::ComparisonWithZero(v) => v.serialized_size(),
            QueryOperand::ComparisonWithArgument(v) => v.serialized_size(),
            QueryOperand::ComparisonWithOtherFile(v) => v.serialized_size(),
            QueryOperand::BitmapRangeComparison(v) => v.serialized_size(),
            QueryOperand::StringTokenSearch(v) => v.serialized_size(),
        }
    }
    fn serialize(&self, out: &mut [u8]) -> usize {
        match self {
            QueryOperand::NonVoid(v) => v.serialize(out),
            QueryOperand::ComparisonWithZero(v) => v.serialize(out),
            QueryOperand::ComparisonWithArgument(v) => v.serialize(out),
            QueryOperand::ComparisonWithOtherFile(v) => v.serialize(out),
            QueryOperand::BitmapRangeComparison(v) => v.serialize(out),
            QueryOperand::StringTokenSearch(v) => v.serialize(out),
        }
    }
}

struct IndirectInterface {}

impl Serializable for IndirectInterface {
    fn serialized_size(&self) -> usize {}
    fn serialize(&self, out: &mut [u8]) -> usize {
        todo!()
    }
}

struct InterfaceStatus {}

impl Serializable for InterfaceStatus {
    fn serialized_size(&self) -> usize {}
    fn serialize(&self, out: &mut [u8]) -> usize {
        todo!()
    }
}

struct InterfaceConfiguration {}

impl Serializable for InterfaceConfiguration {
    fn serialized_size(&self) -> usize {}
    fn serialize(&self, out: &mut [u8]) -> usize {
        todo!()
    }
}

// =================================================================================
// Actions
// =================================================================================
// Nop
pub struct Nop {
    group: bool,
    resp: bool,
}
impl Serializable for Nop {
    fn serialized_size(&self) -> usize {
        1
    }
    fn serialize(&self, out: &mut [u8]) -> usize {
        out[0] = control_byte!(self.group, self.resp, OpCode::Nop);
        1
    }
}

// Read
pub struct ReadFileData {
    group: bool,
    resp: bool,
    data: FileDataRequestOperand,
}
impl Serializable for ReadFileData {
    fn serialized_size(&self) -> usize {
        1 + self.data.serialized_size()
    }
    fn serialize(&self, out: &mut [u8]) -> usize {
        out[0] = control_byte!(self.group, self.resp, OpCode::ReadFileData);
        1 + self.data.serialize(&mut out[1..])
    }
}
pub struct ReadFileProperties {
    group: bool,
    resp: bool,
    file_id: FileIdOperand,
}
impl Serializable for ReadFileProperties {
    fn serialized_size(&self) -> usize {
        1 + self.file_id.serialized_size()
    }
    fn serialize(&self, out: &mut [u8]) -> usize {
        out[0] = control_byte!(self.group, self.resp, OpCode::ReadFileProperties);
        1 + self.file_id.serialize(&mut out[1..])
    }
}

// Write
pub struct WriteFileData {
    group: bool,
    resp: bool,
    file_data: FileDataOperand,
}
impl Serializable for WriteFileData {
    fn serialized_size(&self) -> usize {
        1 + self.file_data.serialized_size()
    }
    fn serialize(&self, out: &mut [u8]) -> usize {
        todo!()
    }
}
pub struct WriteFileDataFlush {
    group: bool,
    resp: bool,
    file_data: FileDataOperand,
}
impl Serializable for WriteFileDataFlush {
    fn serialized_size(&self) -> usize {}
    fn serialize(&self, out: &mut [u8]) -> usize {
        todo!()
    }
}
pub struct WriteFileProperties {
    group: bool,
    resp: bool,
    file_header: FileHeader,
}
impl Serializable for WriteFileProperties {
    fn serialized_size(&self) -> usize {}
    fn serialize(&self, out: &mut [u8]) -> usize {
        todo!()
    }
}
pub struct ActionQuery {
    group: bool,
    resp: bool,
    query: QueryOperand,
}
impl Serializable for ActionQuery {
    fn serialized_size(&self) -> usize {}
    fn serialize(&self, out: &mut [u8]) -> usize {
        todo!()
    }
}
pub struct BreakQuery {
    group: bool,
    resp: bool,
    query: QueryOperand,
}
impl Serializable for BreakQuery {
    fn serialized_size(&self) -> usize {}
    fn serialize(&self, out: &mut [u8]) -> usize {
        todo!()
    }
}
pub struct PermissionRequest {
    group: bool,
    resp: bool,
    permission: PermissionOperand,
}
impl Serializable for PermissionRequest {
    fn serialized_size(&self) -> usize {}
    fn serialize(&self, out: &mut [u8]) -> usize {
        todo!()
    }
}
pub struct VerifyChecksum {
    group: bool,
    resp: bool,
    query: QueryOperand,
}
impl Serializable for VerifyChecksum {
    fn serialized_size(&self) -> usize {}
    fn serialize(&self, out: &mut [u8]) -> usize {
        todo!()
    }
}

// Management
pub struct ExistFile {
    group: bool,
    resp: bool,
    file_id: FileIdOperand,
}
impl Serializable for ExistFile {
    fn serialized_size(&self) -> usize {}
    fn serialize(&self, out: &mut [u8]) -> usize {
        todo!()
    }
}
pub struct CreateNewFile {
    group: bool,
    resp: bool,
    file_header: FileHeader,
}
impl Serializable for CreateNewFile {
    fn serialized_size(&self) -> usize {}
    fn serialize(&self, out: &mut [u8]) -> usize {
        todo!()
    }
}
pub struct DeleteFile {
    group: bool,
    resp: bool,
    file_id: FileIdOperand,
}
impl Serializable for DeleteFile {
    fn serialized_size(&self) -> usize {}
    fn serialize(&self, out: &mut [u8]) -> usize {
        todo!()
    }
}
pub struct RestoreFile {
    group: bool,
    resp: bool,
    file_id: FileIdOperand,
}
impl Serializable for RestoreFile {
    fn serialized_size(&self) -> usize {}
    fn serialize(&self, out: &mut [u8]) -> usize {
        todo!()
    }
}
pub struct FlushFile {
    group: bool,
    resp: bool,
    file_id: FileIdOperand,
}
impl Serializable for FlushFile {
    fn serialized_size(&self) -> usize {}
    fn serialize(&self, out: &mut [u8]) -> usize {
        todo!()
    }
}
pub struct CopyFile {
    group: bool,
    resp: bool,
    source_file_id: FileIdOperand,
    dest_file_id: FileIdOperand,
}
impl Serializable for CopyFile {
    fn serialized_size(&self) -> usize {}
    fn serialize(&self, out: &mut [u8]) -> usize {
        todo!()
    }
}
pub struct ExecuteFile {
    group: bool,
    resp: bool,
    file_id: FileIdOperand,
}
impl Serializable for ExecuteFile {
    fn serialized_size(&self) -> usize {}
    fn serialize(&self, out: &mut [u8]) -> usize {
        todo!()
    }
}

// Response
pub struct ReturnFileData {
    group: bool,
    resp: bool,
    file_data: FileDataOperand,
}
impl Serializable for ReturnFileData {
    fn serialized_size(&self) -> usize {}
    fn serialize(&self, out: &mut [u8]) -> usize {
        todo!()
    }
}
pub struct ReturnFileProperties {
    group: bool,
    resp: bool,
    file_header: FileHeader,
}
impl Serializable for ReturnFileProperties {
    fn serialized_size(&self) -> usize {}
    fn serialize(&self, out: &mut [u8]) -> usize {
        todo!()
    }
}
pub enum Status {
    Action(StatusOperand),
    Interface(InterfaceStatus),
}
impl Serializable for Status {
    fn serialized_size(&self) -> usize {}
    fn serialize(&self, out: &mut [u8]) -> usize {
        todo!()
    }
}
pub struct ResponseTag {
    eop: bool, // End of packet
    err: bool,
    id: u8,
}
impl Serializable for ResponseTag {
    fn serialized_size(&self) -> usize {}
    fn serialize(&self, out: &mut [u8]) -> usize {
        todo!()
    }
}

// Special
pub enum ChunkStep {
    Continue = 0,
    Start = 1,
    End = 2,
    StartEnd = 3,
}
pub struct Chunk {
    step: ChunkStep,
}
impl Serializable for Chunk {
    fn serialized_size(&self) -> usize {}
    fn serialize(&self, out: &mut [u8]) -> usize {
        todo!()
    }
}
enum LogicOp {
    Or = 0,
    Xor = 1,
    Nor = 2,
    Nand = 3,
}
pub struct Logic {
    logic: LogicOp,
}
impl Serializable for Logic {
    fn serialized_size(&self) -> usize {}
    fn serialize(&self, out: &mut [u8]) -> usize {
        todo!()
    }
}
pub struct Forward {
    resp: bool,
    conf: InterfaceConfiguration,
}
impl Serializable for Forward {
    fn serialized_size(&self) -> usize {}
    fn serialize(&self, out: &mut [u8]) -> usize {
        todo!()
    }
}
pub struct IndirectForward {
    overload: bool,
    resp: bool,
    interface: IndirectInterface,
}
impl Serializable for IndirectForward {
    fn serialized_size(&self) -> usize {}
    fn serialize(&self, out: &mut [u8]) -> usize {
        todo!()
    }
}
pub struct RequestTag {
    eop: bool, // End of packet
}
impl Serializable for RequestTag {
    fn serialized_size(&self) -> usize {}
    fn serialize(&self, out: &mut [u8]) -> usize {
        todo!()
    }
}
pub struct Extension {
    group: bool,
    resp: bool,
}
impl Serializable for Extension {
    fn serialized_size(&self) -> usize {}
    fn serialize(&self, out: &mut [u8]) -> usize {
        todo!()
    }
}

pub enum Action {
    // Nop
    Nop(Nop),

    // Read
    ReadFileData(ReadFileData),
    ReadFileProperties(ReadFileProperties),

    // Write
    WriteFileData(WriteFileData),
    WriteFileDataFlush(WriteFileDataFlush),
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

impl Serializable for Action {
    fn serialized_size(&self) -> usize {
        match self {
            Action::Nop(x) => x.serialized_size(),
            Action::ReadFileData(x) => x.serialized_size(),
            Action::ReadFileProperties(x) => x.serialized_size(),
            Action::WriteFileData(x) => x.serialized_size(),
            Action::WriteFileProperties(x) => x.serialized_size(),
            Action::ActionQuery(x) => x.serialized_size(),
            Action::BreakQuery(x) => x.serialized_size(),
            Action::PermissionRequest(x) => x.serialized_size(),
            Action::VerifyChecksum(x) => x.serialized_size(),
            Action::ExistFile(x) => x.serialized_size(),
            Action::CreateNewFile(x) => x.serialized_size(),
            Action::DeleteFile(x) => x.serialized_size(),
            Action::RestoreFile(x) => x.serialized_size(),
            Action::FlushFile(x) => x.serialized_size(),
            Action::CopyFile(x) => x.serialized_size(),
            Action::ExecuteFile(x) => x.serialized_size(),
            Action::ReturnFileData(x) => x.serialized_size(),
            Action::ReturnFileProperties(x) => x.serialized_size(),
            Action::Status(x) => x.serialized_size(),
            Action::ResponseTag(x) => x.serialized_size(),
            Action::Chunk(x) => x.serialized_size(),
            Action::Logic(x) => x.serialized_size(),
            Action::Forward(x) => x.serialized_size(),
            Action::IndirectForward(x) => x.serialized_size(),
            Action::RequestTag(x) => x.serialized_size(),
            Action::Extension(x) => x.serialized_size(),
        }
    }
    fn serialize(&self, out: &mut [u8]) -> usize {
        match self {
            Action::Nop(x) => x.serialize(out),
            Action::ReadFileData(x) => x.serialize(out),
            Action::ReadFileProperties(x) => x.serialize(out),
            Action::WriteFileData(x) => x.serialize(out),
            Action::WriteFileProperties(x) => x.serialize(out),
            Action::ActionQuery(x) => x.serialize(out),
            Action::BreakQuery(x) => x.serialize(out),
            Action::PermissionRequest(x) => x.serialize(out),
            Action::VerifyChecksum(x) => x.serialize(out),
            Action::ExistFile(x) => x.serialize(out),
            Action::CreateNewFile(x) => x.serialize(out),
            Action::DeleteFile(x) => x.serialize(out),
            Action::RestoreFile(x) => x.serialize(out),
            Action::FlushFile(x) => x.serialize(out),
            Action::CopyFile(x) => x.serialize(out),
            Action::ExecuteFile(x) => x.serialize(out),
            Action::ReturnFileData(x) => x.serialize(out),
            Action::ReturnFileProperties(x) => x.serialize(out),
            Action::Status(x) => x.serialize(out),
            Action::ResponseTag(x) => x.serialize(out),
            Action::Chunk(x) => x.serialize(out),
            Action::Logic(x) => x.serialize(out),
            Action::Forward(x) => x.serialize(out),
            Action::IndirectForward(x) => x.serialize(out),
            Action::RequestTag(x) => x.serialize(out),
            Action::Extension(x) => x.serialize(out),
        }
    }
}

// =================================================================================
// Command
// =================================================================================
pub struct Command {
    pub actions: Vec<Action>,
}

impl Command {
    pub fn new() -> Self {
        Self { actions: vec![] }
    }
}
impl Serializable for Command {
    fn serialized_size(&self) -> usize {
        self.actions.iter().map(|act| act.serialized_size()).sum()
    }
    fn serialize(&self, out: &mut [u8]) -> usize {
        let mut offset = 0;
        for action in self.actions.iter() {
            offset += action.serialize(&mut out[offset..]);
        }
        offset
    }
}

// =================================================================================
// Tests
// =================================================================================
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

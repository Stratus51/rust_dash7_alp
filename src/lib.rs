use std::convert::TryFrom;

mod serializable;
use serializable::Serializable;

// =================================================================================
// Macros
// =================================================================================
macro_rules! serialize_all {
    ($out: expr, $($x: expr),*) => {
        {
            let mut offset = 0;
            $({
                $x.serialize(&mut $out[offset..]);
                offset += $x.serialized_size();
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

    fn serialize(&self, out: &mut [u8]) {
        Self::u32_serialize(self.value, out);
    }
}

struct FileIdOperand {
    file_id: u8,
}
impl Serializable for FileIdOperand {
    fn serialized_size(&self) -> usize {
        1
    }
    fn serialize(&self, out: &mut [u8]) {
        out[0] = self.file_id;
    }
}

struct FileOffsetOperand {
    file_id: FileIdOperand,
    offset: VariableUint,
}
impl Serializable for FileOffsetOperand {
    fn serialized_size(&self) -> usize {
        serialized_size!(self.file_id, self.offset)
    }
    fn serialize(&self, out: &mut [u8]) {
        serialize_all!(out, self.file_id, self.offset);
    }
}

struct FileDataRequestOperand {
    file_offset: FileOffsetOperand,
    size: VariableUint,
}
impl Serializable for FileDataRequestOperand {
    fn serialized_size(&self) -> usize {
        serialized_size!(self.file_offset, self.size)
    }
    fn serialize(&self, out: &mut [u8]) {
        serialize_all!(out, self.file_offset, self.size);
    }
}

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
    fn serialize(&self, out: &mut [u8]) {
        let offset = VariableUint::u32_serialize(self.data.len() as u32, out) as usize;
        out[offset..].clone_from_slice(&self.data[..]);
    }
}

struct FileDataOperand {
    file_offset: FileOffsetOperand,
    data: DataOperand,
}
impl Serializable for FileDataOperand {
    fn serialized_size(&self) -> usize {
        serialized_size!(self.file_offset, self.data)
    }
    fn serialize(&self, out: &mut [u8]) {
        serialize_all!(out, self.file_offset, self.data);
    }
}

// TODO
// ALP SPEC: Missing link to find definition in ALP spec
struct FileProperties {
    data: [u8; 12],
}
impl Serializable for FileProperties {
    fn serialized_size(&self) -> usize {
        12
    }
    fn serialize(&self, out: &mut [u8]) {
        todo!()
    }
}

struct FileHeader {
    file_id: FileIdOperand,
    data: FileProperties,
}
impl Serializable for FileHeader {
    fn serialized_size(&self) -> usize {
        serialized_size!(self.file_id, self.data)
    }
    fn serialize(&self, out: &mut [u8]) {
        serialize_all!(out, self.file_id, self.data);
    }
}

enum Status {
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
    status: Status,
}
impl Serializable for StatusOperand {
    fn serialized_size(&self) -> usize {
        1 + 1
    }
    fn serialize(&self, out: &mut [u8]) {
        out[0] = self.action_index;
        out[1] = self.status as u8;
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
    fn serialize(&self, out: &mut [u8]) {
        out[0] = self.id();
        match self {
            Permission::Dash7(token) => out[1..].clone_from_slice(&token[..]),
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
    fn serialize(&self, out: &mut [u8]) {
        out[0] = *self as u8
    }
}

struct PermissionOperand {
    level: PermissionLevel,
    permission: Permission,
}

impl Serializable for PermissionOperand {
    fn serialized_size(&self) -> usize {
        serialized_size!(self.level, self.permission)
    }
    fn serialize(&self, out: &mut [u8]) {
        serialize_all!(self.level, self.permission);
    }
}

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
struct ComparisonWithZero {
    signed_data: bool,
    comparison_type: QueryComparisonType,
    mask: Option<Box<[u8]>>,
    file_offset: FileOffsetOperand,
}
struct ComparisonWithValue {
    signed_data: bool,
    comparison_type: QueryComparisonType,
    mask: Option<Box<[u8]>>,
    value: Box<[u8]>,
    file_offset: FileOffsetOperand,
}
// ALP SPEC: Which of the offset operand is the source and the dest? (file 1 and 2)
struct ComparisonWithOtherFile {
    signed_data: bool,
    comparison_type: QueryComparisonType,
    mask: Option<Box<[u8]>>,
    file_offset_src: FileOffsetOperand,
    file_offset_dst: FileOffsetOperand,
}
struct BitmapRangeComparison {
    signed_data: bool,
    comparison_type: QueryRangeComparisonType,
    size: VariableUint,
    start: Box<[u8]>,
    stop: Box<[u8]>,
    bitmap: Box<[u8]>, // TODO Better type?
    file_offset: FileOffsetOperand,
}
struct StringTokenSearch {
    max_errors: u8,
    size: VariableUint,
    mask: Option<Box<[u8]>>,
    value: Box<[u8]>,
    file_offset: FileOffsetOperand,
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
    fn serialized_size(&self) -> usize {}
    fn serialize(&self, out: &mut [u8]) {
        todo!()
    }
}

struct InterfaceStatusOperand {}

impl Serializable for InterfaceStatusOperand {
    fn serialized_size(&self) -> usize {}
    fn serialize(&self, out: &mut [u8]) {
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
    fn serialize(&self, out: &mut [u8]) {
        let mut ctrl = OpCode::Nop as u8;
        if self.group {
            ctrl += 1 << 7;
        }
        if self.resp {
            ctrl += 1 << 6;
        }
        out[0] = ctrl
    }
}

// Read
pub struct ReadFileData {
    group: bool,
    resp: bool,
    data: FileDataRequest,
}
impl Serializable for ReadFileData {
    fn serialized_size(&self) -> usize {
        1 + self.data.serialized_size()
    }
    fn serialize(&self, out: &mut [u8]) {
        let mut ctrl = OpCode::ReadFileData as u8;
        if self.group {
            ctrl += 1 << 7;
        }
        if self.resp {
            ctrl += 1 << 6;
        }
        out[0] = ctrl;
        self.data.serialize(&mut out[1..]);
    }
}
pub struct ReadFileProperties {
    group: bool,
    resp: bool,
    file_id: u8,
}
impl Serializable for ReadFileProperties {
    fn serialized_size(&self) -> usize {}
    fn serialize(&self, out: &mut [u8]) {
        todo!()
    }
}

// Write
pub struct WriteFileData {
    group: bool,
    resp: bool,
    file_data: FileData,
}
impl Serializable for WriteFileData {
    fn serialized_size(&self) -> usize {}
    fn serialize(&self, out: &mut [u8]) {
        todo!()
    }
}
pub struct WriteFileDataFlush {
    group: bool,
    resp: bool,
    file_data: FileData,
}
impl Serializable for WriteFileDataFlush {
    fn serialized_size(&self) -> usize {}
    fn serialize(&self, out: &mut [u8]) {
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
    fn serialize(&self, out: &mut [u8]) {
        todo!()
    }
}
pub struct ActionQuery {
    group: bool,
    resp: bool,
    query: Query,
}
impl Serializable for ActionQuery {
    fn serialized_size(&self) -> usize {}
    fn serialize(&self, out: &mut [u8]) {
        todo!()
    }
}
pub struct BreakQuery {
    group: bool,
    resp: bool,
    query: Query,
}
impl Serializable for BreakQuery {
    fn serialized_size(&self) -> usize {}
    fn serialize(&self, out: &mut [u8]) {
        todo!()
    }
}
pub struct PermissionRequest {
    group: bool,
    resp: bool,
    permission: Permission,
}
impl Serializable for PermissionRequest {
    fn serialized_size(&self) -> usize {}
    fn serialize(&self, out: &mut [u8]) {
        todo!()
    }
}
pub struct VerifyChecksum {
    group: bool,
    resp: bool,
    query: Query,
}
impl Serializable for VerifyChecksum {
    fn serialized_size(&self) -> usize {}
    fn serialize(&self, out: &mut [u8]) {
        todo!()
    }
}

// Management
pub struct ExistFile {
    group: bool,
    resp: bool,
    file_id: u8,
}
impl Serializable for ExistFile {
    fn serialized_size(&self) -> usize {}
    fn serialize(&self, out: &mut [u8]) {
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
    fn serialize(&self, out: &mut [u8]) {
        todo!()
    }
}
pub struct DeleteFile {
    group: bool,
    resp: bool,
    file_id: u8,
}
impl Serializable for DeleteFile {
    fn serialized_size(&self) -> usize {}
    fn serialize(&self, out: &mut [u8]) {
        todo!()
    }
}
pub struct RestoreFile {
    group: bool,
    resp: bool,
    file_id: u8,
}
impl Serializable for RestoreFile {
    fn serialized_size(&self) -> usize {}
    fn serialize(&self, out: &mut [u8]) {
        todo!()
    }
}
pub struct FlushFile {
    group: bool,
    resp: bool,
    file_id: u8,
}
impl Serializable for FlushFile {
    fn serialized_size(&self) -> usize {}
    fn serialize(&self, out: &mut [u8]) {
        todo!()
    }
}
pub struct CopyFile {
    group: bool,
    resp: bool,
    source_file_id: u8,
    dest_file_id: u8,
}
impl Serializable for CopyFile {
    fn serialized_size(&self) -> usize {}
    fn serialize(&self, out: &mut [u8]) {
        todo!()
    }
}
pub struct ExecuteFile {
    group: bool,
    resp: bool,
    file_id: u8,
}
impl Serializable for ExecuteFile {
    fn serialized_size(&self) -> usize {}
    fn serialize(&self, out: &mut [u8]) {
        todo!()
    }
}

// Response
pub struct ReturnFileData {
    group: bool,
    resp: bool,
    file_data: FileData,
}
impl Serializable for ReturnFileData {
    fn serialized_size(&self) -> usize {}
    fn serialize(&self, out: &mut [u8]) {
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
    fn serialize(&self, out: &mut [u8]) {
        todo!()
    }
}
pub enum Status {
    Action(ActionStatus),
    Interface(InterfaceStatus),
}
impl Serializable for Status {
    fn serialized_size(&self) -> usize {}
    fn serialize(&self, out: &mut [u8]) {
        todo!()
    }
}
pub struct ResponseTag {
    eop: bool, // End of packet
    err: bool,
}
impl Serializable for ResponseTag {
    fn serialized_size(&self) -> usize {}
    fn serialize(&self, out: &mut [u8]) {
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
    fn serialize(&self, out: &mut [u8]) {
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
    fn serialize(&self, out: &mut [u8]) {
        todo!()
    }
}
pub struct Forward {
    resp: bool,
}
impl Serializable for Forward {
    fn serialized_size(&self) -> usize {}
    fn serialize(&self, out: &mut [u8]) {
        todo!()
    }
}
pub struct IndirectForward {
    overload: bool,
    resp: bool,
}
impl Serializable for IndirectForward {
    fn serialized_size(&self) -> usize {}
    fn serialize(&self, out: &mut [u8]) {
        todo!()
    }
}
pub struct RequestTag {
    eop: bool, // End of packet
}
impl Serializable for RequestTag {
    fn serialized_size(&self) -> usize {}
    fn serialize(&self, out: &mut [u8]) {
        todo!()
    }
}
pub struct Extension {
    group: bool,
    resp: bool,
}
impl Serializable for Extension {
    fn serialized_size(&self) -> usize {}
    fn serialize(&self, out: &mut [u8]) {
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

impl Action {
    pub fn serialize(&self) -> Box<[u8]> {
        match self {
            Action::Nop(x) => x.serialize(),
            Action::ReadFileData(x) => x.serialize(),
            Action::ReadFileProperties(x) => x.serialize(),
            Action::WriteFileData(x) => x.serialize(),
            Action::WriteFileProperties(x) => x.serialize(),
            Action::ActionQuery(x) => x.serialize(),
            Action::BreakQuery(x) => x.serialize(),
            Action::PermissionRequest(x) => x.serialize(),
            Action::VerifyChecksum(x) => x.serialize(),
            Action::ExistFile(x) => x.serialize(),
            Action::CreateNewFile(x) => x.serialize(),
            Action::DeleteFile(x) => x.serialize(),
            Action::RestoreFile(x) => x.serialize(),
            Action::FlushFile(x) => x.serialize(),
            Action::CopyFile(x) => x.serialize(),
            Action::ExecuteFile(x) => x.serialize(),
            Action::ReturnFileData(x) => x.serialize(),
            Action::ReturnFileProperties(x) => x.serialize(),
            Action::Status(x) => x.serialize(),
            Action::ResponseTag(x) => x.serialize(),
            Action::Chunk(x) => x.serialize(),
            Action::Logic(x) => x.serialize(),
            Action::Forward(x) => x.serialize(),
            Action::IndirectForward(x) => x.serialize(),
            Action::RequestTag(x) => x.serialize(),
            Action::Extension(x) => x.serialize(),
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

    pub fn serialize(&self) -> Box<[u8]> {
        self.actions
            .iter()
            .map(|act| act.serialize())
            .collect::<Vec<_>>()
            .concat()
            .into_boxed_slice()
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

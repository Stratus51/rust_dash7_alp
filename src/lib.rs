#[cfg(test)]
use hex_literal::hex;

mod serializable;
pub use serializable::Serializable;

mod variable_uint;
pub use variable_uint::VariableUint;

// TODO Maybe using flat structures and modeling operands as macros would be much more ergonomic.
// TODO Look into const function to replace some macros?

// ===============================================================================
// Macros
// ===============================================================================
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
                serialized_size!($({ &self.$x }),*)
            }
            fn serialize(&self, out: &mut [u8]) -> usize {
                serialize_all!(out, $({ &self.$x }),*)
            }
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

// TODO
macro_rules! impl_op_serialized {
    ($name: ident, $flag7: ident, $flag6: ident) => {
        impl Serializable for $name {
            fn serialized_size(&self) -> usize {
                1
            }
            fn serialize(&self, out: &mut [u8]) -> usize {
                out[0] = control_byte!(self.$flag7, self.$flag6, OpCode::$name);
                1
            }
        }
    };
    ($name: ident, $flag7: ident, $flag6: ident, $($x: ident),* ) => {
        impl Serializable for $name {
            fn serialized_size(&self) -> usize {
                1 +
                serialized_size!($(self.$x),*)
            }
            fn serialize(&self, out: &mut [u8]) -> usize {
                out[0] = control_byte!(self.$flag7, self.$flag6, OpCode::$name);
                1 + serialize_all!(out, $({ &self.$x }),*)
            }
        }
    };
}

// ===============================================================================
// Opcodes
// ===============================================================================
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

// ===============================================================================
// D7a definitions
// ===============================================================================
#[derive(Clone, Copy)]
pub enum NlsMethod {
    None = 0,
    AesCtr = 1,
    AesCbcMac128 = 2,
    AesCbcMac64 = 3,
    AesCbcMac32 = 4,
    AesCcm128 = 5,
    AesCcm64 = 6,
    AesCcm32 = 7,
}

// ALP SPEC: Where is this defined?
pub enum Address {
    // D7A SPEC: It is not clear that the estimated reached has to be placed on the "ID" field.
    NbId(u8),
    NoId,
    Uid(Box<[u8; 8]>),
    Vid(Box<[u8; 2]>),
}
pub struct Addressee {
    pub nls_method: NlsMethod,
    pub access_class: u8,
    pub address: Address,
}
impl Serializable for Addressee {
    fn serialized_size(&self) -> usize {
        1 + 1
            + match self.address {
                Address::NbId(_) => 1,
                Address::NoId => 0,
                Address::Uid(_) => 8,
                Address::Vid(_) => 2,
            }
    }
    fn serialize(&self, out: &mut [u8]) -> usize {
        let (id_type, id): (u8, Box<[u8]>) = match &self.address {
            Address::NbId(n) => (0, Box::new([*n])),
            Address::NoId => (1, Box::new([])),
            Address::Uid(uid) => (2, uid.clone()),
            Address::Vid(vid) => (3, vid.clone()),
        };

        out[0] = (id_type << 4) | (self.nls_method as u8);
        out[1] = self.access_class;
        out[2..2 + id.len()].clone_from_slice(&id);
        2 + id.len()
    }
}
#[test]
fn test_addressee_nbid() {
    assert_eq!(
        Addressee {
            nls_method: NlsMethod::None,
            access_class: 0x00,
            address: Address::NbId(0x15),
        }
        .serialize_to_box()[..],
        hex!("00 00 15")
    )
}
#[test]
fn test_addressee_noid() {
    assert_eq!(
        Addressee {
            nls_method: NlsMethod::AesCbcMac128,
            access_class: 0x24,
            address: Address::NoId,
        }
        .serialize_to_box()[..],
        hex!("12 24")
    )
}
#[test]
fn test_addressee_uid() {
    assert_eq!(
        Addressee {
            nls_method: NlsMethod::AesCcm64,
            access_class: 0x48,
            address: Address::Uid(Box::new([0, 1, 2, 3, 4, 5, 6, 7])),
        }
        .serialize_to_box()[..],
        hex!("26 48 0001020304050607")
    )
}
#[test]
fn test_addressee_vid() {
    assert_eq!(
        Addressee {
            nls_method: NlsMethod::AesCcm32,
            access_class: 0xFF,
            address: Address::Vid(Box::new([0xAB, 0xCD])),
        }
        .serialize_to_box()[..],
        hex!("37 FF AB CD")
    )
}

#[derive(Clone, Copy)]
pub enum RetryMode {
    No = 0,
}

#[derive(Clone, Copy)]
pub enum RespMode {
    No = 0,
    All = 1,
    Any = 2,
    RespNoRpt = 4,
    RespOnData = 5,
    RespPreferred = 6,
}

pub struct Qos {
    pub retry: RetryMode,
    pub resp: RespMode,
}
impl Serializable for Qos {
    fn serialized_size(&self) -> usize {
        1
    }
    fn serialize(&self, out: &mut [u8]) -> usize {
        out[0] = ((self.retry as u8) << 3) + self.resp as u8;
        1
    }
}
#[test]
fn test_qos() {
    assert_eq!(
        Qos {
            retry: RetryMode::No,
            resp: RespMode::RespNoRpt,
        }
        .serialize_to_box()[..],
        hex!("04")
    )
}

// ALP SPEC: Add link to D7a section
pub struct D7aspInterfaceConfiguration {
    pub qos: Qos, // TODO enum
    pub to: u8,
    pub te: u8,
    pub addressee: Addressee,
}

impl Serializable for D7aspInterfaceConfiguration {
    fn serialized_size(&self) -> usize {
        3 + self.addressee.serialized_size()
    }
    fn serialize(&self, out: &mut [u8]) -> usize {
        self.qos.serialize(out);
        out[1] = self.to;
        out[2] = self.te;
        3 + self.addressee.serialize(&mut out[3..])
    }
}
#[test]
fn test_d7asp_interface_configuration() {
    assert_eq!(
        D7aspInterfaceConfiguration {
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
            }
        }
        .serialize_to_box()[..],
        hex!("02 23 34   37 FF ABCD")
    )
}

// ALP SPEC: Add link to D7a section (names do not even match)
pub struct D7aspInterfaceStatus {
    pub ch_header: u8,
    pub ch_idx: u16,
    pub rxlev: u8,
    pub lb: u8,
    pub snr: u8,
    pub status: u8,
    pub token: u8,
    pub seq: u8,
    pub resp_to: u8,
    pub addressee: Addressee,
    pub nls_state: Option<[u8; 5]>, // TODO Constrain this existence with addressee nls value
}
impl Serializable for D7aspInterfaceStatus {
    fn serialized_size(&self) -> usize {
        10 + self.addressee.serialized_size()
            + match self.nls_state {
                Some(_) => 5,
                None => 0,
            }
    }
    fn serialize(&self, out: &mut [u8]) -> usize {
        let mut i = 0;
        out[i] = self.ch_header;
        i += 1;
        out[i..(i + 2)].clone_from_slice(&self.ch_idx.to_le_bytes());
        i += 2;
        out[i] = self.rxlev;
        i += 1;
        out[i] = self.lb;
        i += 1;
        out[i] = self.snr;
        i += 1;
        out[i] = self.status;
        i += 1;
        out[i] = self.token;
        i += 1;
        out[i] = self.seq;
        i += 1;
        out[i] = self.resp_to;
        i += 1;
        i += self.addressee.serialize(&mut out[i..]);
        if let Some(nls_state) = &self.nls_state {
            out[i..i + 5].clone_from_slice(&nls_state[..]);
            i += 5;
        }
        i
    }
}
#[test]
fn test_d7asp_interface_status() {
    assert_eq!(
        D7aspInterfaceStatus {
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
        .serialize_to_box()[..],
        hex!("01 2301 02 03 04 05 06 07 08   37 FF ABCD  0011223344")
    )
}

// ===============================================================================
// Alp Interfaces
// ===============================================================================
pub enum InterfaceId {
    Host = 0,
    D7asp = 0xD7,
}

pub enum InterfaceConfiguration {
    D7asp(D7aspInterfaceConfiguration),
}
impl Serializable for InterfaceConfiguration {
    fn serialized_size(&self) -> usize {
        match self {
            InterfaceConfiguration::D7asp(v) => v.serialized_size(),
        }
    }
    fn serialize(&self, out: &mut [u8]) -> usize {
        out[0] = match self {
            InterfaceConfiguration::D7asp(_) => InterfaceId::D7asp,
        } as u8;
        1 + match self {
            InterfaceConfiguration::D7asp(v) => v.serialize(&mut out[1..]),
        }
    }
}

pub enum InterfaceStatus {
    D7asp(D7aspInterfaceStatus),
    // TODO Protect with size limit (< VariableUint max size)
    Unknown(Box<[u8]>),
}
impl Serializable for InterfaceStatus {
    fn serialized_size(&self) -> usize {
        match self {
            InterfaceStatus::D7asp(itf) => itf.serialized_size(),
            InterfaceStatus::Unknown(data) => {
                1 + unsafe { VariableUint::unsafe_size(data.len() as u32) as usize }
            }
        }
    }
    fn serialize(&self, _out: &mut [u8]) -> usize {
        todo!()
    }
}

// ===============================================================================
// Operands
// ===============================================================================
pub struct FileIdOperand {
    pub file_id: u8,
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

pub struct FileOffsetOperand {
    pub file_id: FileIdOperand,
    pub offset: VariableUint,
}
impl_serialized!(FileOffsetOperand, file_id, offset);
#[test]
fn test_file_offset_operand() {
    assert_eq!(
        *FileOffsetOperand {
            file_id: FileIdOperand { file_id: 2 },
            offset: VariableUint::new(0x3F_FF).unwrap(),
        }
        .serialize_to_box(),
        hex!("02 7F FF")
    )
}

pub struct FileDataRequestOperand {
    pub file_offset: FileOffsetOperand,
    pub size: VariableUint,
}
impl_serialized!(FileDataRequestOperand, file_offset, size);

pub struct DataOperand {
    pub data: Box<[u8]>,
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
        let offset = unsafe { VariableUint::u32_serialize(self.data.len() as u32, out) as usize };
        out[offset..].clone_from_slice(&self.data[..]);
        offset + self.data.len()
    }
}

pub struct FileDataOperand {
    pub file_offset: FileOffsetOperand,
    pub data: DataOperand,
}
impl_serialized!(FileDataOperand, file_offset, data);

// TODO
// ALP SPEC: Missing link to find definition in ALP spec
pub struct FileProperties {
    pub data: [u8; 12],
}
impl Serializable for FileProperties {
    fn serialized_size(&self) -> usize {
        12
    }
    fn serialize(&self, _out: &mut [u8]) -> usize {
        todo!()
    }
}

pub struct FileHeader {
    pub file_id: FileIdOperand,
    pub data: FileProperties,
}
impl_serialized!(FileHeader, file_id, data);

#[derive(Copy, Clone)]
pub enum StatusCode {
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
pub struct StatusOperand {
    pub action_index: u8,
    pub status: StatusCode,
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
pub enum Permission {
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

#[derive(Clone, Copy)]
pub enum PermissionLevel {
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

pub struct PermissionOperand {
    pub level: PermissionLevel,
    pub permission: Permission,
}
impl_serialized!(PermissionOperand, level, permission);

#[derive(Clone, Copy)]
pub enum QueryComparisonType {
    Inequal = 0,
    Equal = 1,
    LessThan = 2,
    LessThanOrEqual = 3,
    GreaterThan = 4,
    GreaterThanOrEqual = 5,
}

#[derive(Clone, Copy)]
pub enum QueryRangeComparisonType {
    NotInRange = 0,
    InRange = 1,
}
pub enum QueryCode {
    NonVoid = 0,
    ComparisonWithZero = 1,
    ComparisonWithValue = 2,
    ComparisonWithOtherFile = 3,
    BitmapRangeComparison = 4,
    StringTokenSearch = 7,
}

pub struct NonVoid {
    pub size: VariableUint,
    pub file_offset: FileOffsetOperand,
}
impl Serializable for NonVoid {
    fn serialized_size(&self) -> usize {
        1 + serialized_size!(self.size, self.file_offset)
    }
    fn serialize(&self, out: &mut [u8]) -> usize {
        out[0] = QueryCode::NonVoid as u8;
        1 + serialize_all!(&mut out[1..], self.size, self.file_offset)
    }
}
// TODO Check size coherence upon creation
pub struct ComparisonWithZero {
    pub signed_data: bool,
    pub comparison_type: QueryComparisonType,
    pub size: VariableUint,
    pub mask: Option<Box<[u8]>>,
    pub file_offset: FileOffsetOperand,
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
        let mask_flag = match self.mask {
            Some(_) => 1,
            None => 0,
        };
        let signed_flag = if self.signed_data { 1 } else { 0 };
        let mut offset = 0;
        out[0] = ((QueryCode::ComparisonWithZero as u8) << 4)
            | (mask_flag << 3)
            | (signed_flag << 3)
            | self.comparison_type as u8;
        offset += 1;
        offset += self.size.serialize(&mut out[offset..]);
        if let Some(mask) = &self.mask {
            out[offset..].clone_from_slice(&mask);
            offset += mask.len();
        }
        offset += self.file_offset.serialize(&mut out[offset..]);
        offset
    }
}
// TODO Check size coherence upon creation
pub struct ComparisonWithValue {
    pub signed_data: bool,
    pub comparison_type: QueryComparisonType,
    pub size: VariableUint,
    pub mask: Option<Box<[u8]>>,
    pub value: Box<[u8]>,
    pub file_offset: FileOffsetOperand,
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
        let mask_flag = match self.mask {
            Some(_) => 1,
            None => 0,
        };
        let signed_flag = if self.signed_data { 1 } else { 0 };
        let mut offset = 0;
        out[0] = ((QueryCode::ComparisonWithValue as u8) << 4)
            | (mask_flag << 3)
            | (signed_flag << 3)
            | self.comparison_type as u8;
        offset += 1;
        offset += self.size.serialize(&mut out[offset..]);
        if let Some(mask) = &self.mask {
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
pub struct ComparisonWithOtherFile {
    pub signed_data: bool,
    pub comparison_type: QueryComparisonType,
    pub size: VariableUint,
    pub mask: Option<Box<[u8]>>,
    pub file_offset_src: FileOffsetOperand,
    pub file_offset_dst: FileOffsetOperand,
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
        let mask_flag = match self.mask {
            Some(_) => 1,
            None => 0,
        };
        let signed_flag = if self.signed_data { 1 } else { 0 };
        let mut offset = 0;
        out[0] = ((QueryCode::ComparisonWithOtherFile as u8) << 4)
            | (mask_flag << 3)
            | (signed_flag << 3)
            | self.comparison_type as u8;
        offset += 1;
        offset += self.size.serialize(&mut out[offset..]);
        if let Some(mask) = &self.mask {
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
pub struct BitmapRangeComparison {
    pub signed_data: bool,
    pub comparison_type: QueryRangeComparisonType,
    pub size: VariableUint,
    pub start: Box<[u8]>,
    pub stop: Box<[u8]>,
    pub bitmap: Box<[u8]>, // TODO Better type?
    pub file_offset: FileOffsetOperand,
}
impl Serializable for BitmapRangeComparison {
    fn serialized_size(&self) -> usize {
        1 + self.size.serialized_size()
            + 2 * self.size.value as usize
            + self.bitmap.len()
            + self.file_offset.serialized_size()
    }
    fn serialize(&self, out: &mut [u8]) -> usize {
        let mut offset = 0;
        let signed_flag = if self.signed_data { 1 } else { 0 };
        out[0] = ((QueryCode::BitmapRangeComparison as u8) << 4)
            // | (0 << 3)
            | (signed_flag << 3)
            | self.comparison_type as u8;
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
pub struct StringTokenSearch {
    pub max_errors: u8,
    pub size: VariableUint,
    pub mask: Option<Box<[u8]>>,
    pub value: Box<[u8]>,
    pub file_offset: FileOffsetOperand,
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
        let mask_flag = match self.mask {
            Some(_) => 1,
            None => 0,
        };
        let mut offset = 0;
        out[0] = ((QueryCode::StringTokenSearch as u8) << 4)
            | (mask_flag << 3)
            // | (0 << 3)
            | self.max_errors;
        offset += 1;
        offset += self.size.serialize(&mut out[offset..]);
        if let Some(mask) = &self.mask {
            out[offset..].clone_from_slice(&mask);
            offset += mask.len();
        }
        out[offset..].clone_from_slice(&self.value[..]);
        offset += self.value.len();
        offset += self.file_offset.serialize(&mut out[offset..]);
        offset
    }
}

pub enum QueryOperand {
    NonVoid(NonVoid),
    ComparisonWithZero(ComparisonWithZero),
    ComparisonWithValue(ComparisonWithValue),
    ComparisonWithOtherFile(ComparisonWithOtherFile),
    BitmapRangeComparison(BitmapRangeComparison),
    StringTokenSearch(StringTokenSearch),
}
impl Serializable for QueryOperand {
    fn serialized_size(&self) -> usize {
        match self {
            QueryOperand::NonVoid(v) => v.serialized_size(),
            QueryOperand::ComparisonWithZero(v) => v.serialized_size(),
            QueryOperand::ComparisonWithValue(v) => v.serialized_size(),
            QueryOperand::ComparisonWithOtherFile(v) => v.serialized_size(),
            QueryOperand::BitmapRangeComparison(v) => v.serialized_size(),
            QueryOperand::StringTokenSearch(v) => v.serialized_size(),
        }
    }
    fn serialize(&self, out: &mut [u8]) -> usize {
        match self {
            QueryOperand::NonVoid(v) => v.serialize(out),
            QueryOperand::ComparisonWithZero(v) => v.serialize(out),
            QueryOperand::ComparisonWithValue(v) => v.serialize(out),
            QueryOperand::ComparisonWithOtherFile(v) => v.serialize(out),
            QueryOperand::BitmapRangeComparison(v) => v.serialize(out),
            QueryOperand::StringTokenSearch(v) => v.serialize(out),
        }
    }
}

pub struct OverloadedIndirectInterface {
    pub interface_file_id: FileIdOperand,
    pub addressee: Addressee,
}
impl_serialized!(OverloadedIndirectInterface, interface_file_id, addressee);

pub struct NonOverloadedIndirectInterface {
    pub interface_file_id: FileIdOperand,
    // ALP SPEC: Where is this defined? Is this ID specific?
    pub data: Box<[u8]>,
}

impl Serializable for NonOverloadedIndirectInterface {
    fn serialized_size(&self) -> usize {
        self.interface_file_id.serialized_size() + self.data.len()
    }
    fn serialize(&self, out: &mut [u8]) -> usize {
        let mut offset = self.interface_file_id.serialize(out);
        out[offset..].clone_from_slice(&self.data);
        offset += self.data.len();
        // ALP SPEC: TODO: What should we do
        todo!()
    }
}

pub enum IndirectInterface {
    Overloaded(OverloadedIndirectInterface),
    NonOverloaded(NonOverloadedIndirectInterface),
}

impl Serializable for IndirectInterface {
    fn serialized_size(&self) -> usize {
        match self {
            IndirectInterface::Overloaded(v) => v.serialized_size(),
            IndirectInterface::NonOverloaded(v) => v.serialized_size(),
        }
    }
    fn serialize(&self, out: &mut [u8]) -> usize {
        match self {
            IndirectInterface::Overloaded(v) => v.serialize(out),
            IndirectInterface::NonOverloaded(v) => v.serialize(out),
        }
    }
}

// ===============================================================================
// Actions
// ===============================================================================
// Nop
pub struct Nop {
    pub group: bool,
    pub resp: bool,
}
impl_op_serialized!(Nop, group, resp);

// Read
pub struct ReadFileData {
    pub group: bool,
    pub resp: bool,
    pub data: FileDataRequestOperand,
}
impl_op_serialized!(ReadFileData, group, resp, data);

pub struct ReadFileProperties {
    pub group: bool,
    pub resp: bool,
    pub file_id: FileIdOperand,
}
impl_op_serialized!(ReadFileProperties, group, resp, file_id);

// Write
pub struct WriteFileData {
    pub group: bool,
    pub resp: bool,
    pub file_data: FileDataOperand,
}
impl_op_serialized!(WriteFileData, group, resp, file_data);

pub struct WriteFileDataFlush {
    pub group: bool,
    pub resp: bool,
    pub file_data: FileDataOperand,
}
impl_op_serialized!(WriteFileDataFlush, group, resp, file_data);

pub struct WriteFileProperties {
    pub group: bool,
    pub resp: bool,
    pub file_header: FileHeader,
}
impl_op_serialized!(WriteFileProperties, group, resp, file_header);

pub struct ActionQuery {
    pub group: bool,
    pub resp: bool,
    pub query: QueryOperand,
}
impl_op_serialized!(ActionQuery, group, resp, query);

pub struct BreakQuery {
    pub group: bool,
    pub resp: bool,
    pub query: QueryOperand,
}
impl_op_serialized!(BreakQuery, group, resp, query);

pub struct PermissionRequest {
    pub group: bool,
    pub resp: bool,
    pub permission: PermissionOperand,
}
impl_op_serialized!(PermissionRequest, group, resp, permission);

pub struct VerifyChecksum {
    pub group: bool,
    pub resp: bool,
    pub query: QueryOperand,
}
impl_op_serialized!(VerifyChecksum, group, resp, query);

// Management
pub struct ExistFile {
    pub group: bool,
    pub resp: bool,
    pub file_id: FileIdOperand,
}
impl_op_serialized!(ExistFile, group, resp, file_id);

pub struct CreateNewFile {
    pub group: bool,
    pub resp: bool,
    pub file_header: FileHeader,
}
impl_op_serialized!(CreateNewFile, group, resp, file_header);

pub struct DeleteFile {
    pub group: bool,
    pub resp: bool,
    pub file_id: FileIdOperand,
}
impl_op_serialized!(DeleteFile, group, resp, file_id);

pub struct RestoreFile {
    pub group: bool,
    pub resp: bool,
    pub file_id: FileIdOperand,
}
impl_op_serialized!(RestoreFile, group, resp, file_id);

pub struct FlushFile {
    pub group: bool,
    pub resp: bool,
    pub file_id: FileIdOperand,
}
impl_op_serialized!(FlushFile, group, resp, file_id);

pub struct CopyFile {
    pub group: bool,
    pub resp: bool,
    pub source_file_id: FileIdOperand,
    pub dest_file_id: FileIdOperand,
}
impl_op_serialized!(CopyFile, group, resp, source_file_id, dest_file_id);

pub struct ExecuteFile {
    pub group: bool,
    pub resp: bool,
    pub file_id: FileIdOperand,
}
impl_op_serialized!(ExecuteFile, group, resp, file_id);

// Response
pub struct ReturnFileData {
    pub group: bool,
    pub resp: bool,
    pub file_data: FileDataOperand,
}
impl_op_serialized!(ReturnFileData, group, resp, file_data);

pub struct ReturnFileProperties {
    pub group: bool,
    pub resp: bool,
    pub file_header: FileHeader,
}
impl_op_serialized!(ReturnFileProperties, group, resp, file_header);

#[derive(Clone, Copy)]
pub enum StatusType {
    Action = 0,
    Interface = 1,
}

pub enum Status {
    // ALP SPEC: This is named status, but it should be named action status compared to the '2'
    // other statuses.
    Action(StatusOperand),
    Interface(InterfaceStatus),
    // ALP SPEC: Where are the stack errors?
}
impl Serializable for Status {
    fn serialized_size(&self) -> usize {
        1 + match self {
            Status::Action(op) => op.serialized_size(),
            Status::Interface(op) => op.serialized_size(),
        }
    }
    fn serialize(&self, out: &mut [u8]) -> usize {
        out[0] = OpCode::Status as u8
            + ((match self {
                Status::Action(_) => StatusType::Action,
                Status::Interface(_) => StatusType::Interface,
            } as u8)
                << 6);
        let out = &mut out[1..];
        1 + match self {
            Status::Action(op) => op.serialize(out),
            Status::Interface(op) => op.serialize(out),
        }
    }
}
pub struct ResponseTag {
    pub eop: bool, // End of packet
    pub err: bool,
    pub id: u8,
}
impl Serializable for ResponseTag {
    fn serialized_size(&self) -> usize {
        1 + 1
    }
    fn serialize(&self, out: &mut [u8]) -> usize {
        out[0] = control_byte!(self.eop, self.err, OpCode::ResponseTag);
        out[1] = self.id;

        1 + 1
    }
}

// Special
#[derive(Clone, Copy)]
pub enum ChunkStep {
    Continue = 0,
    Start = 1,
    End = 2,
    StartEnd = 3,
}
pub struct Chunk {
    pub step: ChunkStep,
}
impl Serializable for Chunk {
    fn serialized_size(&self) -> usize {
        1
    }
    fn serialize(&self, out: &mut [u8]) -> usize {
        out[0] = OpCode::Chunk as u8 + ((self.step as u8) << 6);
        1
    }
}

#[derive(Clone, Copy)]
pub enum LogicOp {
    Or = 0,
    Xor = 1,
    Nor = 2,
    Nand = 3,
}
pub struct Logic {
    pub logic: LogicOp,
}
impl Serializable for Logic {
    fn serialized_size(&self) -> usize {
        1
    }
    fn serialize(&self, out: &mut [u8]) -> usize {
        out[0] = OpCode::Logic as u8 + ((self.logic as u8) << 6);
        1
    }
}
pub struct Forward {
    pub resp: bool,
    pub conf: InterfaceConfiguration,
}
impl Serializable for Forward {
    fn serialized_size(&self) -> usize {
        1 + self.conf.serialized_size()
    }
    fn serialize(&self, out: &mut [u8]) -> usize {
        out[0] = control_byte!(false, self.resp, OpCode::Forward);
        1 + self.conf.serialize(&mut out[1..])
    }
}

pub struct IndirectForward {
    pub overload: bool,
    pub resp: bool,
    pub interface: IndirectInterface,
}
impl_op_serialized!(IndirectForward, overload, resp, interface);

pub struct RequestTag {
    pub eop: bool, // End of packet
    pub id: u8,
}
impl Serializable for RequestTag {
    fn serialized_size(&self) -> usize {
        1 + 1
    }
    fn serialize(&self, out: &mut [u8]) -> usize {
        out[0] = control_byte!(self.eop, false, OpCode::RequestTag);
        out[1] = self.id;
        1 + 1
    }
}

pub struct Extension {
    pub group: bool,
    pub resp: bool,
}
impl Serializable for Extension {
    fn serialized_size(&self) -> usize {
        todo!()
    }
    fn serialize(&self, _out: &mut [u8]) -> usize {
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

impl Serializable for Action {
    fn serialized_size(&self) -> usize {
        match self {
            Action::Nop(x) => x.serialized_size(),
            Action::ReadFileData(x) => x.serialized_size(),
            Action::ReadFileProperties(x) => x.serialized_size(),
            Action::WriteFileData(x) => x.serialized_size(),
            // Action::WriteFileDataFlush(x) => x.serialized_size(),
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

// ===============================================================================
// Command
// ===============================================================================
pub struct Command {
    pub actions: Vec<Action>,
}

impl Default for Command {
    fn default() -> Self {
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

#[cfg(test)]
use hex_literal::hex;

mod serializable;
pub use serializable::Serializable;
pub use serializable::{ParseError, ParseResult, ParseValue};

mod variable_uint;
pub use variable_uint::VariableUint;

// TODO Maybe using flat structures and modeling operands as macros would be much more ergonomic.
// TODO Look into const function to replace some macros?
// TODO Use uninitialized memory where possible
// TODO Int enums: fn from(): find a way to avoid double value definition
// TODO Optimize min size calculation (fold it into the upper OP when possible)

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

// Derive replacement (proc-macro would not allow this to be a normal lib)
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
            fn deserialize(out: &[u8]) -> ParseResult<Self> {
                if (out.is_empty()) {
                    Err(ParseError::MissingBytes(Some(1)))
                } else {
                    Ok(ParseValue {
                        value: Self {
                            $flag6: out[0] & 0x40 != 0,
                            $flag7: out[0] & 0x80 != 0,
                        },
                        data_read: 1,
                    })
                }
            }
        }
    };
    ($name: ident, $flag7: ident, $flag6: ident, $op1: ident, $op1_type: ident) => {
        impl Serializable for $name {
            fn serialized_size(&self) -> usize {
                1 + serialized_size!(self.$op1)
            }
            fn serialize(&self, out: &mut [u8]) -> usize {
                out[0] = control_byte!(self.$flag7, self.$flag6, OpCode::$name);
                1 + serialize_all!(out, &self.$op1)
            }
            fn deserialize(out: &[u8]) -> ParseResult<Self> {
                if (out.is_empty()) {
                    Err(ParseError::MissingBytes(Some(1)))
                } else {
                    let mut offset = 1;
                    let ParseValue {
                        value: op1,
                        data_read: op1_size,
                    } = $op1_type::deserialize(&out[offset..])?;
                    offset += op1_size;
                    Ok(ParseValue {
                        value: Self {
                            $flag6: out[0] & 0x40 != 0,
                            $flag7: out[0] & 0x80 != 0,
                            $op1: op1,
                        },
                        data_read: offset,
                    })
                }
            }
        }
    };
    ($name: ident, $flag7: ident, $flag6: ident, $op1: ident, $op1_type: ident, $op2: ident, $op2_type: ident) => {
        impl Serializable for $name {
            fn serialized_size(&self) -> usize {
                1 + serialized_size!(self.$op1, self.$op2)
            }
            fn serialize(&self, out: &mut [u8]) -> usize {
                out[0] = control_byte!(self.$flag7, self.$flag6, OpCode::$name);
                1 + serialize_all!(out, &self.$op1, self.$op2)
            }
            fn deserialize(out: &[u8]) -> ParseResult<Self> {
                if (out.is_empty()) {
                    Err(ParseError::MissingBytes(Some(1)))
                } else {
                    let mut offset = 1;
                    let ParseValue {
                        value: op1,
                        data_read: op1_size,
                    } = $op1_type::deserialize(&out[offset..])?;
                    offset += op1_size;
                    let ParseValue {
                        value: op2,
                        data_read: op2_size,
                    } = $op2_type::deserialize(&out[offset..])?;
                    offset += op2_size;
                    Ok(ParseValue {
                        value: Self {
                            $flag6: out[0] & 0x40 != 0,
                            $flag7: out[0] & 0x80 != 0,
                            $op1: op1,
                            $op2: op2,
                        },
                        data_read: offset,
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
                ret += VariableUint::unsafe_size($x);
            })*
        ret
    }}
}

macro_rules! unsafe_varint_serialize {
    ($out: expr, $($x: expr),*) => {
        {
            let mut offset: usize = 0;
            $(unsafe {
                offset += VariableUint::u32_serialize($x, &mut $out[offset..]) as usize;
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
        impl Serializable for $name {
            fn serialized_size(&self) -> usize {
                1 + count!($( $x )*)
            }
            fn serialize(&self, out: &mut [u8]) -> usize {
                out[0] = control_byte!(self.$flag7, self.$flag6, OpCode::$name);
                let mut offset = 1;
                $({
                    out[offset] = self.$x;
                    offset += 1;
                })*
                1 + offset
            }
            fn deserialize(out: &[u8]) -> ParseResult<Self> {
                const SIZE: usize = 1 + count!($( $x )*);
                if(out.len() < SIZE) {
                    Err(ParseError::MissingBytes(Some(SIZE - out.len())))
                } else {
                    Ok(ParseValue {
                        value: build_simple_op!($name, out, $flag7, $flag6, $($x),*),
                        data_read: SIZE,
                    })
                }
            }
        }
    };
}

macro_rules! impl_header_op {
    ($name: ident, $flag7: ident, $flag6: ident, $file_id: ident, $file_header: ident) => {
        impl Serializable for $name {
            fn serialized_size(&self) -> usize {
                1 + 1 + 12
            }
            fn serialize(&self, out: &mut [u8]) -> usize {
                out[0] = control_byte!(self.group, self.resp, OpCode::$name);
                out[1] = self.file_id;
                out[2..2 + 12].clone_from_slice(&self.data[..]);
                1 + 1 + 12
            }
            fn deserialize(out: &[u8]) -> ParseResult<Self> {
                const SIZE: usize = 1 + 1 + 12;
                if (out.len() < SIZE) {
                    Err(ParseError::MissingBytes(Some(SIZE - out.len())))
                } else {
                    let mut header = [0; 12];
                    header.clone_from_slice(&out[2..2 + 12]);
                    Ok(ParseValue {
                        value: Self {
                            $flag6: out[0] & 0x40 != 0,
                            $flag7: out[0] & 0x80 != 0,
                            $file_id: out[1],
                            $file_header: header,
                        },
                        data_read: SIZE,
                    })
                }
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
    // WriteFileDataFlush = 5, // TODO This is not specified
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
    fn from(n: u8) -> Self {
        match n {
            // Nop
            0 => OpCode::Nop,

            // Read
            1 => OpCode::ReadFileData,
            2 => OpCode::ReadFileProperties,

            // Write
            4 => OpCode::WriteFileData,
            // 5 => OpCode::WriteFileDataFlush, // TODO
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
            // TODO Return proper result
            x => panic!("Unknown opcode {}", x),
        }
    }
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
impl NlsMethod {
    fn from(n: u8) -> NlsMethod {
        match n {
            0 => NlsMethod::None,
            1 => NlsMethod::AesCtr,
            2 => NlsMethod::AesCbcMac128,
            3 => NlsMethod::AesCbcMac64,
            4 => NlsMethod::AesCbcMac32,
            5 => NlsMethod::AesCcm128,
            6 => NlsMethod::AesCcm64,
            7 => NlsMethod::AesCcm32,
            _ => panic!("Unknown nls method {}", n),
        }
    }
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
    fn deserialize(out: &[u8]) -> ParseResult<Self> {
        const SIZE: usize = 1 + 1;
        if out.len() < SIZE {
            return Err(ParseError::MissingBytes(Some(SIZE - out.len())));
        }
        let id_type = (out[0] & 0x30) >> 4;
        let nls_method = NlsMethod::from(out[0] & 0x0F);
        let (address, address_size) = match id_type {
            0 => {
                if out.len() < 3 {
                    return Err(ParseError::MissingBytes(Some(1)));
                }
                (Address::NbId(out[3]), 1)
            }
            1 => (Address::NoId, 0),
            2 => {
                if out.len() < 2 + 8 {
                    return Err(ParseError::MissingBytes(Some(2 + 8 - out.len())));
                }
                let mut data = Box::new([0u8; 8]);
                data.clone_from_slice(&out[2..2 + 8]);
                (Address::Uid(data), 8)
            }
            3 => {
                if out.len() < 2 + 2 {
                    return Err(ParseError::MissingBytes(Some(2 + 2 - out.len())));
                }
                let mut data = Box::new([0u8; 2]);
                data.clone_from_slice(&out[2..2 + 2]);
                (Address::Vid(data), 2)
            }
            x => panic!("Impossible id_type = {}", x),
        };
        Ok(ParseValue {
            value: Self {
                nls_method,
                access_class: out[1],
                address,
            },
            data_read: 1 + address_size,
        })
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
impl RetryMode {
    fn from(n: u8) -> Self {
        match n {
            0 => RetryMode::No,
            // TODO Don't panic. Return Result instead.
            x => panic!("Unknown RetryMode {}", x),
        }
    }
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
impl RespMode {
    fn from(n: u8) -> Self {
        match n {
            0 => RespMode::No,
            1 => RespMode::All,
            2 => RespMode::Any,
            4 => RespMode::RespNoRpt,
            5 => RespMode::RespOnData,
            6 => RespMode::RespPreferred,
            // TODO Don't panic. Return Result instead.
            x => panic!("Unknown RetryMode {}", x),
        }
    }
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
    fn deserialize(out: &[u8]) -> ParseResult<Self> {
        if out.is_empty() {
            return Err(ParseError::MissingBytes(Some(1)));
        }
        Ok(ParseValue {
            value: Self {
                retry: RetryMode::from(out[0] & 0x38 >> 3),
                resp: RespMode::from(out[0] & 0x07),
            },
            data_read: 1,
        })
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
    pub qos: Qos,
    pub to: u8,
    pub te: u8,
    pub addressee: Addressee,
}

impl Serializable for D7aspInterfaceConfiguration {
    fn serialized_size(&self) -> usize {
        self.qos.serialized_size() + 2 + self.addressee.serialized_size()
    }
    fn serialize(&self, out: &mut [u8]) -> usize {
        self.qos.serialize(out);
        out[1] = self.to;
        out[2] = self.te;
        3 + self.addressee.serialize(&mut out[3..])
    }
    fn deserialize(out: &[u8]) -> ParseResult<Self> {
        if out.len() < 3 {
            return Err(ParseError::MissingBytes(Some(3 - out.len())));
        }
        let ParseValue {
            value: qos,
            data_read: qos_size,
        } = Qos::deserialize(out)?;
        let ParseValue {
            value: addressee,
            data_read: addressee_size,
        } = Addressee::deserialize(&out[3..])?;
        Ok(ParseValue {
            value: Self {
                qos,
                to: out[1],
                te: out[2],
                addressee,
            },
            data_read: qos_size + 2 + addressee_size,
        })
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
        out[i..(i + 2)].clone_from_slice(&self.ch_idx.to_le_bytes()); // TODO Check
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
    fn deserialize(out: &[u8]) -> ParseResult<Self> {
        if out.len() < 10 {
            return Err(ParseError::MissingBytes(Some(10 - out.len())));
        }
        let ParseValue {
            value: addressee,
            data_read: addressee_size,
        } = Addressee::deserialize(&out[10..])?;
        let offset = 10 + addressee_size;
        let nls_state = match addressee.nls_method {
            NlsMethod::None => None,
            _ => {
                if out.len() < offset + 5 {
                    return Err(ParseError::MissingBytes(Some(offset + 5 - out.len())));
                } else {
                    let mut nls_state = [0u8; 5];
                    nls_state.clone_from_slice(&out[offset..offset + 5]);
                    Some(nls_state)
                }
            }
        };
        let size = offset
            + match &nls_state {
                Some(_) => 5,
                None => 0,
            };
        Ok(ParseValue {
            value: Self {
                ch_header: out[0],
                // TODO SPEC Check endianess
                ch_idx: ((out[1] as u16) << 8) + out[2] as u16,
                rxlev: out[3],
                lb: out[4],
                snr: out[5],
                status: out[6],
                token: out[7],
                seq: out[8],
                resp_to: out[9],
                addressee,
                nls_state,
            },
            data_read: size,
        })
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
impl InterfaceId {
    fn from(n: u8) -> Self {
        match n {
            0 => InterfaceId::Host,
            0xD7 => InterfaceId::D7asp,
            // TODO Return result instead
            _ => panic!("Unknown interface ID {}", n),
        }
    }
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
    fn deserialize(out: &[u8]) -> ParseResult<Self> {
        if out.is_empty() {
            return Err(ParseError::MissingBytes(Some(1)));
        }
        match InterfaceId::from(out[0]) {
            InterfaceId::D7asp => {
                let ParseValue { value, data_read } =
                    D7aspInterfaceConfiguration::deserialize(&out[1..])?;
                Ok(ParseValue {
                    value: InterfaceConfiguration::D7asp(value),
                    data_read: data_read + 1,
                })
            }
            InterfaceId::Host => panic!("Unknown structure for interface configuration 'Host'"),
        }
    }
}

pub enum InterfaceStatus {
    D7asp(D7aspInterfaceStatus),
    // TODO Protect with size limit (< VariableUint max size)
    Unknown { id: u8, data: Box<[u8]> },
}
impl Serializable for InterfaceStatus {
    fn serialized_size(&self) -> usize {
        match self {
            InterfaceStatus::D7asp(itf) => itf.serialized_size(),
            InterfaceStatus::Unknown { data, .. } => {
                1 + unsafe { VariableUint::unsafe_size(data.len() as u32) as usize }
            }
        }
    }
    fn serialize(&self, out: &mut [u8]) -> usize {
        out[0] = match self {
            InterfaceStatus::D7asp(_) => InterfaceId::D7asp as u8,
            InterfaceStatus::Unknown { id, .. } => *id,
        };
        let mut offset = 1;
        offset += match self {
            InterfaceStatus::D7asp(v) => v.serialize(&mut out[1..]),
            InterfaceStatus::Unknown { data, .. } => {
                out[offset..offset + data.len()].clone_from_slice(data);
                data.len()
            }
        };
        offset
    }
    fn deserialize(out: &[u8]) -> ParseResult<Self> {
        if out.is_empty() {
            return Err(ParseError::MissingBytes(Some(1)));
        }
        match InterfaceId::from(out[0]) {
            InterfaceId::D7asp => {
                let ParseValue { value, data_read } = D7aspInterfaceStatus::deserialize(&out[1..])?;
                Ok(ParseValue {
                    value: InterfaceStatus::D7asp(value),
                    data_read: data_read + 1,
                })
            }
            InterfaceId::Host => panic!("Unknown structure for interface configuration 'Host'"),
        }
    }
}

// ===============================================================================
// Operands
// ===============================================================================
pub struct FileOffsetOperand {
    pub id: u8,
    pub offset: u32,
}

impl Serializable for FileOffsetOperand {
    fn serialized_size(&self) -> usize {
        1 + unsafe_varint_serialize_sizes!(self.offset) as usize
    }
    fn serialize(&self, out: &mut [u8]) -> usize {
        out[0] = self.id;
        1 + unsafe_varint_serialize!(out[1..], self.offset)
    }
    fn deserialize(out: &[u8]) -> ParseResult<Self> {
        if out.len() < 2 {
            return Err(ParseError::MissingBytes(Some(2 - out.len())));
        }
        let ParseValue {
            value: offset,
            data_read,
        } = VariableUint::u32_deserialize(&out[1..])?;
        Ok(ParseValue {
            value: Self { id: out[0], offset },
            data_read: 1 + data_read,
        })
    }
}
#[test]
fn test_file_offset_operand() {
    assert_eq!(
        *FileOffsetOperand {
            id: 2,
            offset: 0x3F_FF,
        }
        .serialize_to_box(),
        hex!("02 7F FF")
    )
}

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
    // TODO Add and unknown type to prevent parsing error?
}
impl StatusCode {
    fn from(n: u8) -> Self {
        match n {
            1 => StatusCode::Received,
            0 => StatusCode::Ok,
            0xFF => StatusCode::FileIdMissing,
            0xFE => StatusCode::CreateFileIdAlreadyExist,
            0xFD => StatusCode::FileIsNotRestorable,
            0xFC => StatusCode::InsufficientPermission,
            0xFB => StatusCode::CreateFileLengthOverflow,
            0xFA => StatusCode::CreateFileAllocationOverflow,
            0xF9 => StatusCode::WriteOffsetOverflow,
            0xF8 => StatusCode::WriteDataOverflow,
            0xF7 => StatusCode::WriteStorageUnavailable,
            0xF6 => StatusCode::UnknownOperation,
            0xF5 => StatusCode::OperandIncomplete,
            0xF4 => StatusCode::OperandWrongFormat,
            0x80 => StatusCode::UnknownError,
            // TODO Add and unknown type to prevent parsing error?
            x => panic!("Unknown Status Code {}", x),
        }
    }
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
    fn deserialize(out: &[u8]) -> ParseResult<Self> {
        if out.len() < 2 {
            return Err(ParseError::MissingBytes(Some(2 - out.len())));
        }
        Ok(ParseValue {
            value: Self {
                action_index: out[0],
                status: StatusCode::from(out[1]),
            },
            data_read: 2,
        })
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
    fn deserialize(out: &[u8]) -> ParseResult<Self> {
        if out.is_empty() {
            return Err(ParseError::MissingBytes(Some(1)));
        }
        let mut offset = 1;
        match out[0] {
            42 => {
                let mut token = [0; 8];
                token.clone_from_slice(&out[offset..offset + 8]);
                offset += 8;
                Ok(ParseValue {
                    value: Permission::Dash7(token),
                    data_read: offset,
                })
            }
            // TODO ParseError
            x => panic!("Unknown authentication ID {}", x),
        }
    }
}

#[derive(Clone, Copy)]
pub enum PermissionLevel {
    // TODO SPEC: Isn't that Guest instead of user?
    User = 0,
    Root = 1,
    // TODO SPEC: Does something else exist?
}
impl Serializable for PermissionLevel {
    fn serialized_size(&self) -> usize {
        1
    }
    fn serialize(&self, out: &mut [u8]) -> usize {
        out[0] = *self as u8;
        1
    }
    fn deserialize(out: &[u8]) -> ParseResult<Self> {
        if out.is_empty() {
            return Err(ParseError::MissingBytes(Some(1)));
        }
        Ok(ParseValue {
            value: match out[0] {
                0 => PermissionLevel::User,
                1 => PermissionLevel::Root,
                // TODO ParseError
                x => panic!("Unknown permission level {}", x),
            },
            data_read: 1,
        })
    }
}

#[derive(Clone, Copy)]
pub enum QueryComparisonType {
    Inequal = 0,
    Equal = 1,
    LessThan = 2,
    LessThanOrEqual = 3,
    GreaterThan = 4,
    GreaterThanOrEqual = 5,
}
impl QueryComparisonType {
    fn from(n: u8) -> Self {
        match n {
            0 => QueryComparisonType::Inequal,
            1 => QueryComparisonType::Equal,
            2 => QueryComparisonType::LessThan,
            3 => QueryComparisonType::LessThanOrEqual,
            4 => QueryComparisonType::GreaterThan,
            5 => QueryComparisonType::GreaterThanOrEqual,
            x => panic!("Unknown query comparison type {}", x),
        }
    }
}

#[derive(Clone, Copy)]
pub enum QueryRangeComparisonType {
    NotInRange = 0,
    InRange = 1,
}
impl QueryRangeComparisonType {
    fn from(n: u8) -> Self {
        match n {
            0 => QueryRangeComparisonType::NotInRange,
            1 => QueryRangeComparisonType::InRange,
            x => panic!("Unknown query range comparison type {}", x),
        }
    }
}
pub enum QueryCode {
    NonVoid = 0,
    ComparisonWithZero = 1,
    ComparisonWithValue = 2,
    ComparisonWithOtherFile = 3,
    BitmapRangeComparison = 4,
    StringTokenSearch = 7,
}
impl QueryCode {
    fn from(n: u8) -> Self {
        match n {
            0 => QueryCode::NonVoid,
            1 => QueryCode::ComparisonWithZero,
            2 => QueryCode::ComparisonWithValue,
            3 => QueryCode::ComparisonWithOtherFile,
            4 => QueryCode::BitmapRangeComparison,
            7 => QueryCode::StringTokenSearch,
            x => panic!("Unknown query code {}", x),
        }
    }
}

pub struct NonVoid {
    // TODO Protect
    pub size: u32,
    pub file: FileOffsetOperand,
}
impl Serializable for NonVoid {
    fn serialized_size(&self) -> usize {
        1 + unsafe { VariableUint::unsafe_size(self.size) } as usize + self.file.serialized_size()
    }
    fn serialize(&self, out: &mut [u8]) -> usize {
        out[0] = QueryCode::NonVoid as u8;
        let mut offset = 1;
        offset += unsafe { VariableUint::u32_serialize(self.size, &mut out[offset..]) } as usize;
        offset += self.file.serialize(&mut out[offset..]);
        offset
    }
    fn deserialize(out: &[u8]) -> ParseResult<Self> {
        if out.len() < 3 {
            return Err(ParseError::MissingBytes(Some(3 - out.len())));
        }
        let ParseValue {
            value: size,
            data_read: size_size,
        } = VariableUint::u32_deserialize(out)?;
        let ParseValue {
            value: file,
            data_read: file_size,
        } = FileOffsetOperand::deserialize(out)?;
        Ok(ParseValue {
            value: Self { size, file },
            data_read: size_size + file_size,
        })
    }
}
// TODO Check size coherence upon creation
pub struct ComparisonWithZero {
    pub signed_data: bool,
    pub comparison_type: QueryComparisonType,
    // TODO Protect
    pub size: u32,
    pub mask: Option<Box<[u8]>>,
    pub file: FileOffsetOperand,
}
impl Serializable for ComparisonWithZero {
    fn serialized_size(&self) -> usize {
        let mask_size = match self.mask {
            Some(_) => self.size as usize,
            None => 0,
        };
        1 + unsafe { VariableUint::unsafe_size(self.size) } as usize
            + mask_size
            + self.file.serialized_size()
    }
    fn serialize(&self, out: &mut [u8]) -> usize {
        let mask_flag = match self.mask {
            Some(_) => 1,
            None => 0,
        };
        let signed_flag = if self.signed_data { 1 } else { 0 };
        let mut offset = 0;
        out[0] = ((QueryCode::ComparisonWithZero as u8) << 5)
            | (mask_flag << 4)
            | (signed_flag << 3)
            | self.comparison_type as u8;
        offset += 1;
        offset += unsafe { VariableUint::u32_serialize(self.size, &mut out[offset..]) } as usize;
        if let Some(mask) = &self.mask {
            out[offset..].clone_from_slice(&mask);
            offset += mask.len();
        }
        offset += self.file.serialize(&mut out[offset..]);
        offset
    }
    fn deserialize(out: &[u8]) -> ParseResult<Self> {
        if out.len() < 1 + 1 + 2 {
            return Err(ParseError::MissingBytes(Some(1 + 1 + 2 - out.len())));
        }
        let mask_flag = out[0] & (1 << 4) != 0;
        let signed_data = out[0] & (1 << 3) != 0;
        let comparison_type = QueryComparisonType::from(out[0] & 0x07);
        let ParseValue {
            value: size,
            data_read: size_size,
        } = VariableUint::u32_deserialize(&out[1..])?;
        let mut offset = 1 + size_size;
        let mask = if mask_flag {
            let mut data = vec![0u8; size as usize].into_boxed_slice();
            data.clone_from_slice(&out[offset..offset + size as usize]);
            offset += size as usize;
            Some(data)
        } else {
            None
        };
        let ParseValue {
            value: file,
            data_read: offset_size,
        } = FileOffsetOperand::deserialize(&out[offset..])?;
        offset += offset_size;
        Ok(ParseValue {
            value: Self {
                signed_data,
                comparison_type,
                size,
                mask,
                file,
            },
            data_read: offset,
        })
    }
}
// TODO Check size coherence upon creation
pub struct ComparisonWithValue {
    pub signed_data: bool,
    pub comparison_type: QueryComparisonType,
    // TODO Protect
    pub size: u32,
    pub mask: Option<Box<[u8]>>,
    pub value: Box<[u8]>,
    pub file: FileOffsetOperand,
}
impl Serializable for ComparisonWithValue {
    fn serialized_size(&self) -> usize {
        let mask_size = match self.mask {
            Some(_) => self.size as usize,
            None => 0,
        };
        1 + unsafe { VariableUint::unsafe_size(self.size) } as usize
            + mask_size
            + self.value.len()
            + self.file.serialized_size()
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
        offset += unsafe { VariableUint::u32_serialize(self.size, &mut out[offset..]) } as usize;
        if let Some(mask) = &self.mask {
            out[offset..].clone_from_slice(&mask);
            offset += mask.len();
        }
        out[offset..].clone_from_slice(&self.value[..]);
        offset += self.value.len();
        offset += self.file.serialize(&mut out[offset..]);
        offset
    }
    fn deserialize(out: &[u8]) -> ParseResult<Self> {
        if out.len() < 1 + 1 + 2 {
            return Err(ParseError::MissingBytes(Some(1 + 1 + 2 - out.len())));
        }
        let mask_flag = out[0] & (1 << 4) != 0;
        let signed_data = out[0] & (1 << 3) != 0;
        let comparison_type = QueryComparisonType::from(out[0] & 0x07);
        let ParseValue {
            value: size,
            data_read: size_size,
        } = VariableUint::u32_deserialize(&out[1..])?;
        let mut offset = 1 + size_size;
        let mask = if mask_flag {
            let mut data = vec![0u8; size as usize].into_boxed_slice();
            data.clone_from_slice(&out[offset..offset + size as usize]);
            offset += size as usize;
            Some(data)
        } else {
            None
        };
        let mut value = vec![0u8; size as usize].into_boxed_slice();
        value.clone_from_slice(&out[offset..offset + size as usize]);
        offset += size as usize;
        let ParseValue {
            value: file,
            data_read: offset_size,
        } = FileOffsetOperand::deserialize(&out[offset..])?;
        offset += offset_size;
        Ok(ParseValue {
            value: Self {
                signed_data,
                comparison_type,
                size,
                mask,
                value,
                file,
            },
            data_read: offset,
        })
    }
}
// TODO Check size coherence upon creation
pub struct ComparisonWithOtherFile {
    pub signed_data: bool,
    pub comparison_type: QueryComparisonType,
    // TODO Protect
    pub size: u32,
    pub mask: Option<Box<[u8]>>,
    pub file_src: FileOffsetOperand,
    pub file_dst: FileOffsetOperand,
}
impl Serializable for ComparisonWithOtherFile {
    fn serialized_size(&self) -> usize {
        let mask_size = match self.mask {
            Some(_) => self.size as usize,
            None => 0,
        };
        1 + unsafe { VariableUint::unsafe_size(self.size) } as usize
            + mask_size
            + self.file_src.serialized_size()
            + self.file_dst.serialized_size()
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
        offset += unsafe { VariableUint::u32_serialize(self.size, &mut out[offset..]) } as usize;
        if let Some(mask) = &self.mask {
            out[offset..].clone_from_slice(&mask);
            offset += mask.len();
        }
        // TODO ALP SPEC: Which of the offset operand is the source and the dest? (file 1 and 2)
        offset += self.file_src.serialize(&mut out[offset..]);
        offset += self.file_dst.serialize(&mut out[offset..]);
        offset
    }
    fn deserialize(out: &[u8]) -> ParseResult<Self> {
        if out.len() < 1 + 1 + 2 + 2 {
            return Err(ParseError::MissingBytes(Some(1 + 1 + 2 + 2 - out.len())));
        }
        let mask_flag = out[0] & (1 << 4) != 0;
        let signed_data = out[0] & (1 << 3) != 0;
        let comparison_type = QueryComparisonType::from(out[0] & 0x07);
        let ParseValue {
            value: size,
            data_read: size_size,
        } = VariableUint::u32_deserialize(&out[1..])?;
        let mut offset = 1 + size_size;
        let mask = if mask_flag {
            let mut data = vec![0u8; size as usize].into_boxed_slice();
            data.clone_from_slice(&out[offset..offset + size as usize]);
            offset += size as usize;
            Some(data)
        } else {
            None
        };
        let ParseValue {
            value: file_src,
            data_read: file_src_size,
        } = FileOffsetOperand::deserialize(&out[offset..])?;
        offset += file_src_size;
        let ParseValue {
            value: file_dst,
            data_read: file_dst_size,
        } = FileOffsetOperand::deserialize(&out[offset..])?;
        offset += file_dst_size;
        Ok(ParseValue {
            value: Self {
                signed_data,
                comparison_type,
                size,
                mask,
                file_src,
                file_dst,
            },
            data_read: offset,
        })
    }
}
// TODO Check size coherence upon creation (start, stop and bitmap)
pub struct BitmapRangeComparison {
    pub signed_data: bool,
    pub comparison_type: QueryRangeComparisonType,
    // TODO Protect
    pub size: u32,
    // TODO In theory, start and stop can be huge array thus impossible to cast into any trivial
    // number. How do we deal with this.
    pub start: Box<[u8]>,
    pub stop: Box<[u8]>,
    pub bitmap: Box<[u8]>, // TODO Better type?
    pub file: FileOffsetOperand,
}
impl Serializable for BitmapRangeComparison {
    fn serialized_size(&self) -> usize {
        1 + unsafe { VariableUint::unsafe_size(self.size) } as usize
            + 2 * self.size as usize
            + self.bitmap.len()
            + self.file.serialized_size()
    }
    fn serialize(&self, out: &mut [u8]) -> usize {
        let mut offset = 0;
        let signed_flag = if self.signed_data { 1 } else { 0 };
        out[0] = ((QueryCode::BitmapRangeComparison as u8) << 4)
            // | (0 << 3)
            | (signed_flag << 3)
            | self.comparison_type as u8;
        offset += 1;
        offset += unsafe { VariableUint::u32_serialize(self.size, &mut out[offset..]) } as usize;
        out[offset..].clone_from_slice(&self.start[..]);
        offset += self.start.len();
        out[offset..].clone_from_slice(&self.stop[..]);
        offset += self.stop.len();
        out[offset..].clone_from_slice(&self.bitmap[..]);
        offset += self.bitmap.len();
        offset += self.file.serialize(&mut out[offset..]);
        offset
    }
    fn deserialize(out: &[u8]) -> ParseResult<Self> {
        if out.len() < 1 + 1 + 2 {
            return Err(ParseError::MissingBytes(Some(1 + 1 + 2 - out.len())));
        }
        let signed_data = out[0] & (1 << 3) != 0;
        let comparison_type = QueryRangeComparisonType::from(out[0] & 0x07);
        let ParseValue {
            value: size,
            data_read: size_size,
        } = VariableUint::u32_deserialize(&out[1..])?;
        let mut offset = 1 + size_size;
        let ParseValue {
            value: file,
            data_read: file_size,
        } = FileOffsetOperand::deserialize(&out[offset..])?;
        offset += file_size;
        let mut start = vec![0u8; size as usize].into_boxed_slice();
        start.clone_from_slice(&out[offset..offset + size as usize]);
        offset += size as usize;
        let mut stop = vec![0u8; size as usize].into_boxed_slice();
        stop.clone_from_slice(&out[offset..offset + size as usize]);
        // TODO How do we deal with start and stop?
        if true {
            todo!("How do we calculate start and stop?")
        };
        let start_n = start[0];
        let stop_n = stop[0];
        let bitmap_size = (stop_n - start_n + 6) / 8; // ALP SPEC: Thanks for the calculation
        let mut bitmap = vec![0u8; bitmap_size as usize].into_boxed_slice();
        bitmap.clone_from_slice(&out[offset..offset + bitmap_size as usize]);
        offset += bitmap_size as usize;
        Ok(ParseValue {
            value: Self {
                signed_data,
                comparison_type,
                size,
                start,
                stop,
                bitmap,
                file,
            },
            data_read: offset,
        })
    }
}
// TODO Check size coherence upon creation
pub struct StringTokenSearch {
    pub max_errors: u8,
    // TODO Protect
    pub size: u32,
    pub mask: Option<Box<[u8]>>,
    pub value: Box<[u8]>,
    pub file: FileOffsetOperand,
}
impl Serializable for StringTokenSearch {
    fn serialized_size(&self) -> usize {
        let mask_size = match self.mask {
            Some(_) => self.size as usize,
            None => 0,
        };
        1 + unsafe { VariableUint::unsafe_size(self.size) } as usize
            + mask_size
            + self.value.len()
            + self.file.serialized_size()
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
        offset += unsafe { VariableUint::u32_serialize(self.size, &mut out[offset..]) } as usize;
        if let Some(mask) = &self.mask {
            out[offset..].clone_from_slice(&mask);
            offset += mask.len();
        }
        out[offset..].clone_from_slice(&self.value[..]);
        offset += self.value.len();
        offset += self.file.serialize(&mut out[offset..]);
        offset
    }
    fn deserialize(out: &[u8]) -> ParseResult<Self> {
        if out.len() < 1 + 1 + 2 {
            return Err(ParseError::MissingBytes(Some(1 + 1 + 2 - out.len())));
        }
        let mask_flag = out[0] & (1 << 4) != 0;
        let max_errors = out[0] & 0x07;
        let ParseValue {
            value: size,
            data_read: size_size,
        } = VariableUint::u32_deserialize(&out[1..])?;
        let mut offset = 1 + size_size;
        let mask = if mask_flag {
            let mut data = vec![0u8; size as usize].into_boxed_slice();
            data.clone_from_slice(&out[offset..offset + size as usize]);
            offset += size as usize;
            Some(data)
        } else {
            None
        };
        let mut value = vec![0u8; size as usize].into_boxed_slice();
        value.clone_from_slice(&out[offset..offset + size as usize]);
        offset += size as usize;
        let ParseValue {
            value: file,
            data_read: offset_size,
        } = FileOffsetOperand::deserialize(&out[offset..])?;
        offset += offset_size;
        Ok(ParseValue {
            value: Self {
                max_errors,
                size,
                mask,
                value,
                file,
            },
            data_read: offset,
        })
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
    fn deserialize(out: &[u8]) -> ParseResult<Self> {
        Ok(match QueryCode::from(out[0] >> 5) {
            QueryCode::NonVoid => NonVoid::deserialize(out)?.map_value(QueryOperand::NonVoid),
            QueryCode::ComparisonWithZero => {
                ComparisonWithZero::deserialize(out)?.map_value(QueryOperand::ComparisonWithZero)
            }
            QueryCode::ComparisonWithValue => {
                ComparisonWithValue::deserialize(out)?.map_value(QueryOperand::ComparisonWithValue)
            }
            QueryCode::ComparisonWithOtherFile => ComparisonWithOtherFile::deserialize(out)?
                .map_value(QueryOperand::ComparisonWithOtherFile),
            QueryCode::BitmapRangeComparison => BitmapRangeComparison::deserialize(out)?
                .map_value(QueryOperand::BitmapRangeComparison),
            QueryCode::StringTokenSearch => {
                StringTokenSearch::deserialize(out)?.map_value(QueryOperand::StringTokenSearch)
            }
        })
    }
}

pub struct OverloadedIndirectInterface {
    pub interface_file_id: u8,
    pub addressee: Addressee,
}

impl Serializable for OverloadedIndirectInterface {
    fn serialized_size(&self) -> usize {
        1 + self.addressee.serialized_size()
    }
    fn serialize(&self, out: &mut [u8]) -> usize {
        out[0] = self.interface_file_id;
        1 + self.addressee.serialize(&mut out[1..])
    }
    fn deserialize(out: &[u8]) -> ParseResult<Self> {
        // TODO Add overload flag (with op byte mod) (shift byte index by 1)
        if out.len() < 1 + 2 {
            return Err(ParseError::MissingBytes(Some(1 + 2 - out.len())));
        }
        let interface_file_id = out[0];
        let ParseValue {
            value: addressee,
            data_read: addressee_size,
        } = Addressee::deserialize(&out[1..])?;
        Ok(ParseValue {
            value: Self {
                interface_file_id,
                addressee,
            },
            data_read: 1 + addressee_size,
        })
    }
}

pub struct NonOverloadedIndirectInterface {
    pub interface_file_id: u8,
    // ALP SPEC: Where is this defined? Is this ID specific?
    pub data: Box<[u8]>,
}

impl Serializable for NonOverloadedIndirectInterface {
    fn serialized_size(&self) -> usize {
        1 + self.data.len()
    }
    fn serialize(&self, out: &mut [u8]) -> usize {
        // TODO Add overload flag (with op byte mod) (shift out bytes by 1)
        out[0] = self.interface_file_id;
        let mut offset = 1;
        out[offset..].clone_from_slice(&self.data);
        offset += self.data.len();
        // ALP SPEC: TODO: What should we do
        todo!("{}", offset)
    }
    fn deserialize(_out: &[u8]) -> ParseResult<Self> {
        todo!("TODO")
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
    fn deserialize(out: &[u8]) -> ParseResult<Self> {
        if out.is_empty() {
            return Err(ParseError::MissingBytes(Some(1)));
        }
        Ok(if out[0] & 0x80 != 0 {
            OverloadedIndirectInterface::deserialize(out)?.map_value(IndirectInterface::Overloaded)
        } else {
            NonOverloadedIndirectInterface::deserialize(out)?
                .map_value(IndirectInterface::NonOverloaded)
        })
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
// TODO Protect varint init
pub struct ReadFileData {
    pub group: bool,
    pub resp: bool,
    pub file_id: u8,
    // TODO Protect
    pub offset: u32,
    // TODO Protect
    pub size: u32,
}

impl Serializable for ReadFileData {
    fn serialized_size(&self) -> usize {
        1 + 1 + unsafe_varint_serialize_sizes!(self.offset, self.size) as usize
    }
    fn serialize(&self, out: &mut [u8]) -> usize {
        out[0] = control_byte!(self.group, self.resp, OpCode::ReadFileData);
        out[1] = self.file_id;
        1 + 1 + unsafe_varint_serialize!(out[2..], self.offset, self.size)
    }
    fn deserialize(out: &[u8]) -> ParseResult<Self> {
        let min_size = 1 + 1 + 1 + 1;
        if out.len() < min_size {
            return Err(ParseError::MissingBytes(Some(min_size - out.len())));
        }
        let group = out[0] & 0x80 != 0;
        let resp = out[0] & 0x40 != 0;
        let file_id = out[1];
        let mut off = 2;
        let ParseValue {
            value: offset,
            data_read: offset_size,
        } = VariableUint::u32_deserialize(&out[off..])?;
        off += offset_size;
        let ParseValue {
            value: size,
            data_read: size_size,
        } = VariableUint::u32_deserialize(&out[off..])?;
        off += size_size;
        Ok(ParseValue {
            value: Self {
                group,
                resp,
                file_id,
                offset,
                size,
            },
            data_read: off,
        })
    }
}

pub struct ReadFileProperties {
    pub group: bool,
    pub resp: bool,
    pub file_id: u8,
}
impl_simple_op!(ReadFileProperties, group, resp, file_id);

// Write
// TODO Protect varint init, data consistency
pub struct WriteFileData {
    pub group: bool,
    pub resp: bool,
    pub file_id: u8,
    // TODO Protect
    pub offset: u32,
    pub data: Box<[u8]>,
}
impl Serializable for WriteFileData {
    fn serialized_size(&self) -> usize {
        1 + 1
            + unsafe_varint_serialize_sizes!(self.offset, self.data.len() as u32) as usize
            + self.data.len()
    }
    fn serialize(&self, out: &mut [u8]) -> usize {
        out[0] = control_byte!(self.group, self.resp, OpCode::WriteFileData);
        out[1] = self.file_id;
        let mut offset = 2;
        offset += unsafe_varint_serialize!(out[2..], self.offset, self.data.len() as u32) as usize;
        out[offset..offset + self.data.len()].clone_from_slice(&self.data[..]);
        offset += self.data.len();
        offset
    }
    fn deserialize(out: &[u8]) -> ParseResult<Self> {
        let min_size = 1 + 1 + 1 + 1;
        if out.len() < min_size {
            return Err(ParseError::MissingBytes(Some(min_size - out.len())));
        }
        let group = out[0] & 0x80 != 0;
        let resp = out[0] & 0x40 != 0;
        let file_id = out[1];
        let mut off = 2;
        let ParseValue {
            value: offset,
            data_read: offset_size,
        } = VariableUint::u32_deserialize(&out[off..])?;
        off += offset_size;
        let ParseValue {
            value: size,
            data_read: size_size,
        } = VariableUint::u32_deserialize(&out[off..])?;
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
            },
            data_read: off,
        })
    }
}

// pub struct WriteFileDataFlush {
//     pub group: bool,
//     pub resp: bool,
//     pub file_id: u8,
//     // TODO Protect
//     pub offset: u32,
//     pub data: Box<[u8]>,
// }
// impl Serializable for WriteFileDataFlush {
//     fn serialized_size(&self) -> usize {
//         1 + 1
//             + unsafe_varint_serialize_sizes!(self.offset, self.data.len() as u32) as usize
//             + self.data.len()
//     }
//     fn serialize(&self, out: &mut [u8]) -> usize {
//         out[0] = control_byte!(self.group, self.resp, OpCode::WriteFileDataFlush);
//         out[1] = self.file_id;
//         let mut offset = 2;
//         offset += unsafe_varint_serialize!(out[2..], self.offset, self.data.len() as u32) as usize;
//         out[offset..offset + self.data.len()].clone_from_slice(&self.data[..]);
//         offset += self.data.len();
//         offset
//     }
//     fn deserialize(out: &[u8]) -> ParseResult<Self> {
//         let min_size = 1 + 1 + 1 + 1;
//         if out.len() < min_size {
//             return Err(ParseError::MissingBytes(Some(min_size - out.len())));
//         }
//         let group = out[0] & 0x80 != 0;
//         let resp = out[0] & 0x40 != 0;
//         let file_id = out[1];
//         let mut off = 2;
//         let ParseValue {
//             value: offset,
//             data_read: offset_size,
//         } = VariableUint::u32_deserialize(&out[off..])?;
//         off += offset_size;
//         let ParseValue {
//             value: size,
//             data_read: size_size,
//         } = VariableUint::u32_deserialize(&out[off..])?;
//         off += size_size;
//         let size = size as usize;
//         let mut data = vec![0u8; size].into_boxed_slice();
//         data.clone_from_slice(&out[off..off + size]);
//         off += size;
//         Ok(ParseValue {
//             value: Self {
//                 group,
//                 resp,
//                 file_id,
//                 offset,
//                 data,
//             },
//             data_read: off,
//         })
//     }
// }

pub struct WriteFileProperties {
    pub group: bool,
    pub resp: bool,
    pub file_id: u8,
    // TODO
    // ALP SPEC: Missing link to find definition in ALP spec
    pub data: [u8; 12],
}
impl_header_op!(WriteFileProperties, group, resp, file_id, data);

pub struct ActionQuery {
    pub group: bool,
    pub resp: bool,
    pub query: QueryOperand,
}
impl_op_serialized!(ActionQuery, group, resp, query, QueryOperand);

pub struct BreakQuery {
    pub group: bool,
    pub resp: bool,
    pub query: QueryOperand,
}
impl_op_serialized!(BreakQuery, group, resp, query, QueryOperand);

pub struct PermissionRequest {
    pub group: bool,
    pub resp: bool,
    pub level: PermissionLevel,
    pub permission: Permission,
}
impl_op_serialized!(
    PermissionRequest,
    group,
    resp,
    level,
    PermissionLevel,
    permission,
    Permission
);

pub struct VerifyChecksum {
    pub group: bool,
    pub resp: bool,
    pub query: QueryOperand,
}
impl_op_serialized!(VerifyChecksum, group, resp, query, QueryOperand);

// Management
pub struct ExistFile {
    pub group: bool,
    pub resp: bool,
    pub file_id: u8,
}
impl_simple_op!(ExistFile, group, resp, file_id);

pub struct CreateNewFile {
    pub group: bool,
    pub resp: bool,
    pub file_id: u8,
    // TODO
    // ALP SPEC: Missing link to find definition in ALP spec
    pub data: [u8; 12],
}
impl_header_op!(CreateNewFile, group, resp, file_id, data);

pub struct DeleteFile {
    pub group: bool,
    pub resp: bool,
    pub file_id: u8,
}
impl_simple_op!(DeleteFile, group, resp, file_id);

pub struct RestoreFile {
    pub group: bool,
    pub resp: bool,
    pub file_id: u8,
}
impl_simple_op!(RestoreFile, group, resp, file_id);

pub struct FlushFile {
    pub group: bool,
    pub resp: bool,
    pub file_id: u8,
}
impl_simple_op!(FlushFile, group, resp, file_id);

pub struct CopyFile {
    pub group: bool,
    pub resp: bool,
    pub source_file_id: u8,
    pub dest_file_id: u8,
}
impl_simple_op!(CopyFile, group, resp, source_file_id, dest_file_id);

pub struct ExecuteFile {
    pub group: bool,
    pub resp: bool,
    pub file_id: u8,
}
impl_simple_op!(ExecuteFile, group, resp, file_id);

// Response
pub struct ReturnFileData {
    pub group: bool,
    pub resp: bool,
    pub file_id: u8,
    // TODO Protect
    pub offset: u32,
    pub data: Box<[u8]>,
}
impl Serializable for ReturnFileData {
    fn serialized_size(&self) -> usize {
        1 + 1
            + unsafe_varint_serialize_sizes!(self.offset, self.data.len() as u32) as usize
            + self.data.len()
    }
    fn serialize(&self, out: &mut [u8]) -> usize {
        out[0] = control_byte!(self.group, self.resp, OpCode::ReadFileData);
        out[1] = self.file_id;
        let mut offset = 2;
        offset += unsafe_varint_serialize!(out[2..], self.offset, self.data.len() as u32) as usize;
        out[offset..offset + self.data.len()].clone_from_slice(&self.data[..]);
        offset += self.data.len();
        offset
    }
    fn deserialize(out: &[u8]) -> ParseResult<Self> {
        let min_size = 1 + 1 + 1 + 1;
        if out.len() < min_size {
            return Err(ParseError::MissingBytes(Some(min_size - out.len())));
        }
        let group = out[0] & 0x80 != 0;
        let resp = out[0] & 0x40 != 0;
        let file_id = out[1];
        let mut off = 2;
        let ParseValue {
            value: offset,
            data_read: offset_size,
        } = VariableUint::u32_deserialize(&out[off..])?;
        off += offset_size;
        let ParseValue {
            value: size,
            data_read: size_size,
        } = VariableUint::u32_deserialize(&out[off..])?;
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
            },
            data_read: off,
        })
    }
}

pub struct ReturnFileProperties {
    pub group: bool,
    pub resp: bool,
    pub file_id: u8,
    // TODO
    // ALP SPEC: Missing link to find definition in ALP spec
    pub data: [u8; 12],
}
impl_header_op!(ReturnFileProperties, group, resp, file_id, data);

#[derive(Clone, Copy)]
pub enum StatusType {
    Action = 0,
    Interface = 1,
}
impl StatusType {
    fn from(n: u8) -> Self {
        match n {
            0 => StatusType::Action,
            1 => StatusType::Interface,
            // TODO Return a proper error instead of panic
            x => panic!("Unknown status type: {}", x),
        }
    }
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
    fn deserialize(out: &[u8]) -> ParseResult<Self> {
        if out.is_empty() {
            return Err(ParseError::MissingBytes(Some(1)));
        }
        let status_type = out[0] >> 6;
        Ok(match StatusType::from(status_type) {
            StatusType::Action => StatusOperand::deserialize(&out[1..])?
                .map(|v, data_read| (Status::Action(v), data_read + 1)),
            StatusType::Interface => InterfaceStatus::deserialize(&out[1..])?
                .map(|v, data_read| (Status::Interface(v), data_read + 1)),
        })
    }
}
pub struct ResponseTag {
    pub eop: bool, // End of packet
    pub err: bool,
    pub id: u8,
}
impl_simple_op!(ResponseTag, eop, err, id);

// Special
#[derive(Clone, Copy)]
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
    fn deserialize(out: &[u8]) -> ParseResult<Self> {
        if out.is_empty() {
            return Err(ParseError::MissingBytes(Some(1)));
        }
        Ok(ParseValue {
            value: Self {
                step: ChunkStep::from(out[0] >> 6),
            },
            data_read: 1,
        })
    }
}

#[derive(Clone, Copy)]
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
    fn deserialize(out: &[u8]) -> ParseResult<Self> {
        if out.is_empty() {
            return Err(ParseError::MissingBytes(Some(1)));
        }
        Ok(ParseValue {
            value: Self {
                logic: LogicOp::from(out[0] >> 6),
            },
            data_read: 1,
        })
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
    fn deserialize(out: &[u8]) -> ParseResult<Self> {
        let min_size = 1 + 1;
        if out.len() < min_size {
            return Err(ParseError::MissingBytes(Some(min_size - out.len())));
        }
        let ParseValue {
            value: conf,
            data_read: conf_size,
        } = InterfaceConfiguration::deserialize(&out[1..])?;
        Ok(ParseValue {
            value: Self {
                resp: out[0] & 0x40 != 0,
                conf,
            },
            data_read: 1 + conf_size,
        })
    }
}

pub struct IndirectForward {
    // TODO This is an error: overload is determined by the interface variant. Modify accordingly.
    pub overload: bool,
    pub resp: bool,
    pub interface: IndirectInterface,
}
impl_op_serialized!(
    IndirectForward,
    overload,
    resp,
    interface,
    IndirectInterface
);

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
    fn deserialize(out: &[u8]) -> ParseResult<Self> {
        let min_size = 1 + 1;
        if out.len() < min_size {
            return Err(ParseError::MissingBytes(Some(min_size - out.len())));
        }
        Ok(ParseValue {
            value: Self {
                eop: out[0] & 0x80 != 0,
                id: out[1],
            },
            data_read: 2,
        })
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
    fn deserialize(_out: &[u8]) -> ParseResult<Self> {
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
            // Action::WriteFileDataFlush(x) => x.serialize(out),
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
    fn deserialize(out: &[u8]) -> ParseResult<Self> {
        if out.is_empty() {
            return Err(ParseError::MissingBytes(Some(1)));
        }
        let opcode = OpCode::from(out[0] & 0x3F);
        Ok(match opcode {
            OpCode::Nop => Nop::deserialize(&out)?.map_value(Action::Nop),
            OpCode::ReadFileData => {
                ReadFileData::deserialize(&out)?.map_value(Action::ReadFileData)
            }
            OpCode::ReadFileProperties => {
                ReadFileProperties::deserialize(&out)?.map_value(Action::ReadFileProperties)
            }
            OpCode::WriteFileData => {
                WriteFileData::deserialize(&out)?.map_value(Action::WriteFileData)
            }
            // OpCode::WriteFileDataFlush => {
            //     WriteFileDataFlush::deserialize(&out)?.map_value( Action::WriteFileDataFlush)
            // }
            OpCode::WriteFileProperties => {
                WriteFileProperties::deserialize(&out)?.map_value(Action::WriteFileProperties)
            }
            OpCode::ActionQuery => ActionQuery::deserialize(&out)?.map_value(Action::ActionQuery),
            OpCode::BreakQuery => BreakQuery::deserialize(&out)?.map_value(Action::BreakQuery),
            OpCode::PermissionRequest => {
                PermissionRequest::deserialize(&out)?.map_value(Action::PermissionRequest)
            }
            OpCode::VerifyChecksum => {
                VerifyChecksum::deserialize(&out)?.map_value(Action::VerifyChecksum)
            }
            OpCode::ExistFile => ExistFile::deserialize(&out)?.map_value(Action::ExistFile),
            OpCode::CreateNewFile => {
                CreateNewFile::deserialize(&out)?.map_value(Action::CreateNewFile)
            }
            OpCode::DeleteFile => DeleteFile::deserialize(&out)?.map_value(Action::DeleteFile),
            OpCode::RestoreFile => RestoreFile::deserialize(&out)?.map_value(Action::RestoreFile),
            OpCode::FlushFile => FlushFile::deserialize(&out)?.map_value(Action::FlushFile),
            OpCode::CopyFile => CopyFile::deserialize(&out)?.map_value(Action::CopyFile),
            OpCode::ExecuteFile => ExecuteFile::deserialize(&out)?.map_value(Action::ExecuteFile),
            OpCode::ReturnFileData => {
                ReturnFileData::deserialize(&out)?.map_value(Action::ReturnFileData)
            }
            OpCode::ReturnFileProperties => {
                ReturnFileProperties::deserialize(&out)?.map_value(Action::ReturnFileProperties)
            }
            OpCode::Status => Status::deserialize(&out)?.map_value(Action::Status),
            OpCode::ResponseTag => ResponseTag::deserialize(&out)?.map_value(Action::ResponseTag),
            OpCode::Chunk => Chunk::deserialize(&out)?.map_value(Action::Chunk),
            OpCode::Logic => Logic::deserialize(&out)?.map_value(Action::Logic),
            OpCode::Forward => Forward::deserialize(&out)?.map_value(Action::Forward),
            OpCode::IndirectForward => {
                IndirectForward::deserialize(&out)?.map_value(Action::IndirectForward)
            }
            OpCode::RequestTag => RequestTag::deserialize(&out)?.map_value(Action::RequestTag),
            OpCode::Extension => Extension::deserialize(&out)?.map_value(Action::Extension),
        })
    }
}

// ===============================================================================
// Command
// ===============================================================================
pub struct Command {
    pub actions: Vec<Action>,
}
pub struct CommandParseError {
    pub actions: Vec<Action>,
    pub error: ParseError,
}

impl Default for Command {
    fn default() -> Self {
        Self { actions: vec![] }
    }
}
impl Command {
    fn partial_deserialize(out: &[u8]) -> Result<ParseValue<Command>, CommandParseError> {
        let mut actions = vec![];
        let mut offset = 0;
        loop {
            if out.is_empty() {
                break;
            }
            match Action::deserialize(&out[offset..]) {
                Ok(ParseValue { value, data_read }) => {
                    actions.push(value);
                    offset += data_read;
                }
                Err(error) => return Err(CommandParseError { actions, error }),
            }
        }
        Ok(ParseValue {
            value: Self { actions },
            data_read: offset,
        })
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
    fn deserialize(out: &[u8]) -> ParseResult<Self> {
        Self::partial_deserialize(out).map_err(|v| v.error)
    }
}

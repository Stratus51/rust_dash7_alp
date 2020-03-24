#[cfg(test)]
use hex_literal::hex;

mod codec;
pub use codec::Codec;
pub use codec::{ParseError, ParseResult, ParseValue};

// TODO Look into const function to replace some macros?
// TODO Use uninitialized memory where possible
// TODO Int enums: fn from(): find a way to avoid double value definition
// TODO Int enums: optim: find a way to cast from int to enum instead of calling a matching
// function (much more resource intensive)
// TODO Optimize min size calculation (fold it into the upper OP when possible)
// TODO usize is target dependent. In other words, on a 16 bit processor, we will run into
// troubles if we were to convert u32 to usize (even if a 64Ko payload seems a bit big).
// TODO Slice copies still check length consistency dynamically. Is there a way to get rid of that
// at runtime while still testing it at compile/test time?
// TODO is {out = &out[offset..]; out[..size]} more efficient than {out[offset..offset+size]} ?
// TODO Add function to encode without having to define a temporary structure

// ===============================================================================
// Macros
// ===============================================================================
macro_rules! serialize_all {
    ($out: expr, $($x: expr),*) => {
        {
            let mut offset = 0;
            $({
                offset += $x.encode(&mut $out[offset..]);
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

// Derive replacement (proc-macro would not allow this to be a normal lib)
macro_rules! impl_op_serialized {
    ($name: ident, $flag7: ident, $flag6: ident) => {
        impl Codec for $name {
            fn encoded_size(&self) -> usize {
                1
            }
            fn encode(&self, out: &mut [u8]) -> usize {
                out[0] = control_byte!(self.$flag7, self.$flag6, OpCode::$name);
                1
            }
            fn decode(out: &[u8]) -> ParseResult<Self> {
                if (out.is_empty()) {
                    Err(ParseError::MissingBytes(Some(1)))
                } else {
                    Ok(ParseValue {
                        value: Self {
                            $flag6: out[0] & 0x40 != 0,
                            $flag7: out[0] & 0x80 != 0,
                        },
                        size: 1,
                    })
                }
            }
        }
    };
    ($name: ident, $flag7: ident, $flag6: ident, $op1: ident, $op1_type: ident) => {
        impl Codec for $name {
            fn encoded_size(&self) -> usize {
                1 + encoded_size!(self.$op1)
            }
            fn encode(&self, out: &mut [u8]) -> usize {
                out[0] = control_byte!(self.$flag7, self.$flag6, OpCode::$name);
                1 + serialize_all!(&mut out[1..], &self.$op1)
            }
            fn decode(out: &[u8]) -> ParseResult<Self> {
                if (out.is_empty()) {
                    Err(ParseError::MissingBytes(Some(1)))
                } else {
                    let mut offset = 1;
                    let ParseValue {
                        value: op1,
                        size: op1_size,
                    } = $op1_type::decode(&out[offset..])?;
                    offset += op1_size;
                    Ok(ParseValue {
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
    ($name: ident, $flag7: ident, $flag6: ident, $op1: ident, $op1_type: ident, $op2: ident, $op2_type: ident) => {
        impl Codec for $name {
            fn encoded_size(&self) -> usize {
                1 + encoded_size!(self.$op1, self.$op2)
            }
            fn encode(&self, out: &mut [u8]) -> usize {
                out[0] = control_byte!(self.$flag7, self.$flag6, OpCode::$name);
                1 + serialize_all!(&mut out[1..], &self.$op1, self.$op2)
            }
            fn decode(out: &[u8]) -> ParseResult<Self> {
                if (out.is_empty()) {
                    Err(ParseError::MissingBytes(Some(1)))
                } else {
                    let mut offset = 1;
                    let ParseValue {
                        value: op1,
                        size: op1_size,
                    } = $op1_type::decode(&out[offset..])?;
                    offset += op1_size;
                    let ParseValue {
                        value: op2,
                        size: op2_size,
                    } = $op2_type::decode(&out[offset..])?;
                    offset += op2_size;
                    Ok(ParseValue {
                        value: Self {
                            $flag6: out[0] & 0x40 != 0,
                            $flag7: out[0] & 0x80 != 0,
                            $op1: op1,
                            $op2: op2,
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
            $(unsafe {
                offset += varint::encode($x, &mut $out[offset..]) as usize;
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
            fn encoded_size(&self) -> usize {
                1 + count!($( $x )*)
            }
            fn encode(&self, out: &mut [u8]) -> usize {
                out[0] = control_byte!(self.$flag7, self.$flag6, OpCode::$name);
                let mut offset = 1;
                $({
                    out[offset] = self.$x;
                    offset += 1;
                })*
                1 + offset
            }
            fn decode(out: &[u8]) -> ParseResult<Self> {
                const SIZE: usize = 1 + count!($( $x )*);
                if(out.len() < SIZE) {
                    Err(ParseError::MissingBytes(Some(SIZE - out.len())))
                } else {
                    Ok(ParseValue {
                        value: build_simple_op!($name, out, $flag7, $flag6, $($x),*),
                        size: SIZE,
                    })
                }
            }
        }
    };
}

macro_rules! impl_header_op {
    ($name: ident, $flag7: ident, $flag6: ident, $file_id: ident, $file_header: ident) => {
        impl Codec for $name {
            fn encoded_size(&self) -> usize {
                1 + 1 + 12
            }
            fn encode(&self, out: &mut [u8]) -> usize {
                out[0] = control_byte!(self.group, self.resp, OpCode::$name);
                out[1] = self.file_id;
                out[2..2 + 12].clone_from_slice(&self.data[..]);
                1 + 1 + 12
            }
            fn decode(out: &[u8]) -> ParseResult<Self> {
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
                        size: SIZE,
                    })
                }
            }
        }
    };
}

#[cfg(test)]
fn test_item<T: Codec + std::fmt::Debug + std::cmp::PartialEq>(item: T, data: &[u8]) {
    assert_eq!(item.encode_to_box()[..], *data);
    assert_eq!(
        T::decode(&data).expect("should be parsed without error"),
        ParseValue {
            value: item,
            size: data.len(),
        }
    );
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
    // TODO ALP SPEC: This is out of spec. Can't write + flush already do that job. Is it worth
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
// Varint
// ===============================================================================
mod varint {
    use crate::{ParseError, ParseResult, ParseValue};
    pub const MAX: u32 = 0x3F_FF_FF_FF;
    pub fn is_valid(n: u32) -> Result<(), ()> {
        if n > MAX {
            Err(())
        } else {
            Ok(())
        }
    }

    /// # Safety
    /// Only call this on u32 that are less than 0x3F_FF_FF_FF.
    ///
    /// Calling this on a large integer will return a size of 4 which
    /// is technically incorrect because the integer is non-encodable.
    pub unsafe fn size(n: u32) -> u8 {
        if n <= 0x3F {
            1
        } else if n <= 0x3F_FF {
            2
        } else if n <= 0x3F_FF_FF {
            3
        } else {
            4
        }
    }

    // TODO Is this serialization correct? Check the SPEC!
    /// # Safety
    /// Only call this on u32 that are less than 0x3F_FF_FF_FF.
    ///
    /// Calling this on a large integer will return an unpredictable
    /// result (it won't crash).
    pub unsafe fn encode(n: u32, out: &mut [u8]) -> u8 {
        let u8_size = size(n);
        let size = u8_size as usize;
        for (i, byte) in out.iter_mut().enumerate().take(size) {
            *byte = ((n >> ((size - 1 - i) * 8)) & 0xFF) as u8;
        }
        out[0] |= ((size - 1) as u8) << 6;
        u8_size
    }

    pub fn decode(out: &[u8]) -> ParseResult<u32> {
        if out.is_empty() {
            return Err(ParseError::MissingBytes(Some(1)));
        }
        let size = ((out[0] >> 6) + 1) as usize;
        if out.len() < size as usize {
            return Err(ParseError::MissingBytes(Some(size as usize - out.len())));
        }
        let mut ret = (out[0] & 0x3F) as u32;
        for byte in out.iter().take(size).skip(1) {
            ret = (ret << 8) + *byte as u32;
        }
        Ok(ParseValue { value: ret, size })
    }

    #[cfg(test)]
    mod test {
        use super::*;
        use hex_literal::hex;

        #[test]
        fn test_is_valid() {
            assert_eq!(is_valid(0x3F_FF_FF_FF), Ok(()));
            assert_eq!(is_valid(0x40_00_00_00), Err(()));
        }

        #[test]
        fn test_unsafe_size() {
            unsafe {
                assert_eq!(size(0x00), 1);
                assert_eq!(size(0x3F), 1);
                assert_eq!(size(0x3F_FF), 2);
                assert_eq!(size(0x3F_FF_FF), 3);
                assert_eq!(size(0x3F_FF_FF_FF), 4);
            }
        }

        #[test]
        fn test_encode() {
            fn test(n: u32, truth: &[u8]) {
                let mut encoded = vec![0u8; truth.len()];
                assert_eq!(unsafe { encode(n, &mut encoded[..]) }, truth.len() as u8);
                assert_eq!(*truth, encoded[..]);
            }
            test(0x00, &[0]);
            test(0x3F, &hex!("3F"));
            test(0x3F_FF, &hex!("7F FF"));
            test(0x3F_FF_FF, &hex!("BF FF FF"));
            test(0x3F_FF_FF_FF, &hex!("FF FF FF FF"));
        }

        #[test]
        fn test_decode() {
            fn test_ok(data: &[u8], value: u32, size: usize) {
                assert_eq!(decode(data), Ok(ParseValue { value, size: size }),);
            }
            test_ok(&[0], 0x00, 1);
            test_ok(&hex!("3F"), 0x3F, 1);
            test_ok(&hex!("7F FF"), 0x3F_FF, 2);
            test_ok(&hex!("BF FF FF"), 0x3F_FF_FF, 3);
            test_ok(&hex!("FF FF FF FF"), 0x3F_FF_FF_FF, 4);
        }
    }
}

// ===============================================================================
// D7a definitions
// ===============================================================================
#[derive(Clone, Copy, Debug, PartialEq)]
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
#[derive(Clone, Debug, PartialEq)]
pub enum Address {
    // D7A SPEC: It is not clear that the estimated reached has to be placed on the "ID" field.
    NbId(u8),
    NoId,
    Uid(Box<[u8; 8]>),
    Vid(Box<[u8; 2]>),
}
#[derive(Clone, Debug, PartialEq)]
pub struct Addressee {
    pub nls_method: NlsMethod,
    pub access_class: u8,
    pub address: Address,
}
impl Codec for Addressee {
    fn encoded_size(&self) -> usize {
        1 + 1
            + match self.address {
                Address::NbId(_) => 1,
                Address::NoId => 0,
                Address::Uid(_) => 8,
                Address::Vid(_) => 2,
            }
    }
    fn encode(&self, out: &mut [u8]) -> usize {
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
    fn decode(out: &[u8]) -> ParseResult<Self> {
        const SIZE: usize = 1 + 1;
        if out.len() < SIZE {
            return Err(ParseError::MissingBytes(Some(SIZE - out.len())));
        }
        let id_type = (out[0] & 0x30) >> 4;
        let nls_method = NlsMethod::from(out[0] & 0x0F);
        let access_class = out[1];
        let (address, address_size) = match id_type {
            0 => {
                if out.len() < 3 {
                    return Err(ParseError::MissingBytes(Some(1)));
                }
                (Address::NbId(out[2]), 1)
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
                access_class,
                address,
            },
            size: SIZE + address_size,
        })
    }
}
#[test]
fn test_addressee_nbid() {
    test_item(
        Addressee {
            nls_method: NlsMethod::None,
            access_class: 0x00,
            address: Address::NbId(0x15),
        },
        &hex!("00 00 15"),
    )
}
#[test]
fn test_addressee_noid() {
    test_item(
        Addressee {
            nls_method: NlsMethod::AesCbcMac128,
            access_class: 0x24,
            address: Address::NoId,
        },
        &hex!("12 24"),
    )
}
#[test]
fn test_addressee_uid() {
    test_item(
        Addressee {
            nls_method: NlsMethod::AesCcm64,
            access_class: 0x48,
            address: Address::Uid(Box::new([0, 1, 2, 3, 4, 5, 6, 7])),
        },
        &hex!("26 48 0001020304050607"),
    )
}
#[test]
fn test_addressee_vid() {
    test_item(
        Addressee {
            nls_method: NlsMethod::AesCcm32,
            access_class: 0xFF,
            address: Address::Vid(Box::new([0xAB, 0xCD])),
        },
        &hex!("37 FF AB CD"),
    )
}

#[derive(Clone, Copy, Debug, PartialEq)]
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

#[derive(Clone, Copy, Debug, PartialEq)]
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Qos {
    pub retry: RetryMode,
    pub resp: RespMode,
}
impl Codec for Qos {
    fn encoded_size(&self) -> usize {
        1
    }
    fn encode(&self, out: &mut [u8]) -> usize {
        out[0] = ((self.retry as u8) << 3) + self.resp as u8;
        1
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
        if out.is_empty() {
            return Err(ParseError::MissingBytes(Some(1)));
        }
        Ok(ParseValue {
            value: Self {
                retry: RetryMode::from((out[0] & 0x38) >> 3),
                resp: RespMode::from(out[0] & 0x07),
            },
            size: 1,
        })
    }
}
#[test]
fn test_qos() {
    test_item(
        Qos {
            retry: RetryMode::No,
            resp: RespMode::RespNoRpt,
        },
        &hex!("04"),
    )
}

// ALP SPEC: Add link to D7a section
#[derive(Clone, Debug, PartialEq)]
pub struct D7aspInterfaceConfiguration {
    pub qos: Qos,
    pub to: u8,
    pub te: u8,
    pub addressee: Addressee,
}

impl Codec for D7aspInterfaceConfiguration {
    fn encoded_size(&self) -> usize {
        self.qos.encoded_size() + 2 + self.addressee.encoded_size()
    }
    fn encode(&self, out: &mut [u8]) -> usize {
        self.qos.encode(out);
        out[1] = self.to;
        out[2] = self.te;
        3 + self.addressee.encode(&mut out[3..])
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
        if out.len() < 3 {
            return Err(ParseError::MissingBytes(Some(3 - out.len())));
        }
        let ParseValue {
            value: qos,
            size: qos_size,
        } = Qos::decode(out)?;
        let ParseValue {
            value: addressee,
            size: addressee_size,
        } = Addressee::decode(&out[3..])?;
        Ok(ParseValue {
            value: Self {
                qos,
                to: out[1],
                te: out[2],
                addressee,
            },
            size: qos_size + 2 + addressee_size,
        })
    }
}
#[test]
fn test_d7asp_interface_configuration() {
    test_item(
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
            },
        },
        &hex!("02 23 34   37 FF ABCD"),
    )
}

// ALP SPEC: Add link to D7a section (names do not even match)
#[derive(Clone, Debug, PartialEq)]
pub struct D7aspInterfaceStatus {
    pub ch_header: u8,
    // ALP SPEC: The endianesse of this variable is not specified in section 9.2.12
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
impl Codec for D7aspInterfaceStatus {
    fn encoded_size(&self) -> usize {
        10 + self.addressee.encoded_size()
            + match self.nls_state {
                Some(_) => 5,
                None => 0,
            }
    }
    fn encode(&self, out: &mut [u8]) -> usize {
        let mut i = 0;
        out[i] = self.ch_header;
        i += 1;
        // TODO SPEC Endianness unspecified
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
        i += self.addressee.encode(&mut out[i..]);
        if let Some(nls_state) = &self.nls_state {
            out[i..i + 5].clone_from_slice(&nls_state[..]);
            i += 5;
        }
        i
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
        if out.len() < 10 {
            return Err(ParseError::MissingBytes(Some(10 - out.len())));
        }
        let ParseValue {
            value: addressee,
            size: addressee_size,
        } = Addressee::decode(&out[10..])?;
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
                // TODO SPEC Endianness unspecified
                ch_idx: ((out[2] as u16) << 8) + out[1] as u16,
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
            size,
        })
    }
}
#[test]
fn test_d7asp_interface_status() {
    test_item(
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
        },
        &hex!("01 2301 02 03 04 05 06 07 08   37 FF ABCD  0011223344"),
    )
}

// ===============================================================================
// Alp Interfaces
// ===============================================================================
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InterfaceId {
    Host = 0,
    D7asp = 0xD7,
}

#[derive(Clone, Debug, PartialEq)]
pub enum InterfaceConfiguration {
    // TODO: ALP SPEC: Is this specified?
    Host,
    D7asp(D7aspInterfaceConfiguration),
}
impl Codec for InterfaceConfiguration {
    fn encoded_size(&self) -> usize {
        1 + match self {
            InterfaceConfiguration::Host => 0,
            InterfaceConfiguration::D7asp(v) => v.encoded_size(),
        }
    }
    fn encode(&self, out: &mut [u8]) -> usize {
        match self {
            InterfaceConfiguration::Host => {
                out[0] = InterfaceId::Host as u8;
                1
            }
            InterfaceConfiguration::D7asp(v) => {
                out[0] = InterfaceId::D7asp as u8;
                1 + v.encode(&mut out[1..])
            }
        }
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
        if out.is_empty() {
            return Err(ParseError::MissingBytes(Some(1)));
        }
        const HOST: u8 = InterfaceId::Host as u8;
        const D7ASP: u8 = InterfaceId::D7asp as u8;
        Ok(match out[0] {
            HOST => ParseValue {
                value: InterfaceConfiguration::Host,
                size: 1,
            },
            D7ASP => {
                let ParseValue { value, size } = D7aspInterfaceConfiguration::decode(&out[1..])?;
                ParseValue {
                    value: InterfaceConfiguration::D7asp(value),
                    size: size + 1,
                }
            }
            // TODO Return an error instead of panic
            id => panic!("Unknown interface ID {}", id),
        })
    }
}
#[test]
fn test_interface_configuration_d7asp() {
    test_item(
        InterfaceConfiguration::D7asp(D7aspInterfaceConfiguration {
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
            },
        }),
        &hex!("D7   02 23 34   37 FF ABCD"),
    )
}
#[test]
fn test_interface_configuration_host() {
    test_item(InterfaceConfiguration::Host, &hex!("00"))
}

#[derive(Clone, Debug, PartialEq)]
pub enum InterfaceStatus {
    // TODO
    Host,
    D7asp(D7aspInterfaceStatus),
    // TODO Protect with size limit (< varint max size)
    Unknown { id: u8, data: Box<[u8]> },
}
impl Codec for InterfaceStatus {
    fn encoded_size(&self) -> usize {
        let data_size = match self {
            InterfaceStatus::Host => 0,
            InterfaceStatus::D7asp(itf) => itf.encoded_size(),
            InterfaceStatus::Unknown { data, .. } => data.len(),
        };
        1 + unsafe { varint::size(data_size as u32) as usize } + data_size
    }
    fn encode(&self, out: &mut [u8]) -> usize {
        let mut offset = 1;
        match self {
            InterfaceStatus::Host => {
                out[0] = InterfaceId::Host as u8;
                out[1] = 0;
                offset += 1;
            }
            InterfaceStatus::D7asp(v) => {
                out[0] = InterfaceId::D7asp as u8;
                let size = v.encoded_size() as u32;
                let size_size = unsafe { varint::encode(size, &mut out[offset..]) };
                offset += size_size as usize;
                offset += v.encode(&mut out[offset..]);
            }
            InterfaceStatus::Unknown { id, data } => {
                out[0] = *id;
                let size = data.len() as u32;
                let size_size = unsafe { varint::encode(size, &mut out[offset..]) };
                offset += size_size as usize;
                out[offset..offset + data.len()].clone_from_slice(data);
                offset += data.len();
            }
        };
        offset
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
        if out.is_empty() {
            return Err(ParseError::MissingBytes(Some(1)));
        }
        const HOST: u8 = InterfaceId::Host as u8;
        const D7ASP: u8 = InterfaceId::D7asp as u8;
        let mut offset = 1;
        let value = match out[0] {
            HOST => {
                offset += 1;
                InterfaceStatus::Host
            }
            D7ASP => {
                let ParseValue {
                    value: size,
                    size: size_size,
                } = varint::decode(&out[offset..])?;
                let size = size as usize;
                offset += size_size;
                let ParseValue { value, size } =
                    D7aspInterfaceStatus::decode(&out[offset..offset + size])?;
                offset += size;
                InterfaceStatus::D7asp(value)
            }
            id => {
                let ParseValue {
                    value: size,
                    size: size_size,
                } = varint::decode(&out[offset..])?;
                let size = size as usize;
                offset += size_size;
                if out.len() < offset + size {
                    return Err(ParseError::MissingBytes(Some(offset + size - out.len())));
                }
                let mut data = vec![0u8; size].into_boxed_slice();
                data.clone_from_slice(&out[offset..size]);
                offset += size;
                InterfaceStatus::Unknown { id, data }
            }
        };
        Ok(ParseValue {
            value,
            size: offset,
        })
    }
}
#[test]
fn test_interface_status_d7asp() {
    test_item(
        InterfaceStatus::D7asp(D7aspInterfaceStatus {
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
        }),
        &hex!("D7 13    01 2301 02 03 04 05 06 07 08   37 FF ABCD  0011223344"),
    )
}
#[test]
fn test_interface_status_host() {
    test_item(InterfaceStatus::Host, &hex!("00 00"))
}

// ===============================================================================
// Operands
// ===============================================================================
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FileOffsetOperand {
    pub id: u8,
    pub offset: u32,
}

impl Codec for FileOffsetOperand {
    fn encoded_size(&self) -> usize {
        1 + unsafe_varint_serialize_sizes!(self.offset) as usize
    }
    fn encode(&self, out: &mut [u8]) -> usize {
        out[0] = self.id;
        1 + unsafe_varint_serialize!(out[1..], self.offset)
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
        if out.len() < 2 {
            return Err(ParseError::MissingBytes(Some(2 - out.len())));
        }
        let ParseValue {
            value: offset,
            size,
        } = varint::decode(&out[1..])?;
        Ok(ParseValue {
            value: Self { id: out[0], offset },
            size: 1 + size,
        })
    }
}
#[test]
fn test_file_offset_operand() {
    test_item(
        FileOffsetOperand {
            id: 2,
            offset: 0x3F_FF,
        },
        &hex!("02 7F FF"),
    )
}

#[derive(Clone, Copy, Debug, PartialEq)]
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
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StatusOperand {
    pub action_id: u8,
    pub status: StatusCode,
}
impl Codec for StatusOperand {
    fn encoded_size(&self) -> usize {
        1 + 1
    }
    fn encode(&self, out: &mut [u8]) -> usize {
        out[0] = self.action_id;
        out[1] = self.status as u8;
        2
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
        if out.len() < 2 {
            return Err(ParseError::MissingBytes(Some(2 - out.len())));
        }
        Ok(ParseValue {
            value: Self {
                action_id: out[0],
                status: StatusCode::from(out[1]),
            },
            size: 2,
        })
    }
}
#[test]
fn test_status_operand() {
    test_item(
        StatusOperand {
            action_id: 2,
            status: StatusCode::UnknownOperation,
        },
        &hex!("02 F6"),
    )
}

// ALP SPEC: where is this defined? Link?
//  Not found in either specs !
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Permission {
    Dash7([u8; 8]), // TODO ALP SPEC Check + what is its name?
}

impl Permission {
    fn id(self) -> u8 {
        match self {
            Permission::Dash7(_) => 0x42, // TODO Check
        }
    }
}

impl Codec for Permission {
    fn encoded_size(&self) -> usize {
        1 + match self {
            Permission::Dash7(_) => 8,
        }
    }
    fn encode(&self, out: &mut [u8]) -> usize {
        out[0] = self.id();
        1 + match self {
            Permission::Dash7(token) => {
                out[1..].clone_from_slice(&token[..]);
                8
            }
        }
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
        if out.is_empty() {
            return Err(ParseError::MissingBytes(Some(1)));
        }
        let mut offset = 1;
        match out[0] {
            0x42 => {
                let mut token = [0; 8];
                token.clone_from_slice(&out[offset..offset + 8]);
                offset += 8;
                Ok(ParseValue {
                    value: Permission::Dash7(token),
                    size: offset,
                })
            }
            // TODO ParseError
            x => panic!("Unknown authentication ID {}", x),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PermissionLevel {
    // TODO SPEC: Isn't that Guest instead of user?
    User = 0,
    Root = 1,
    // TODO SPEC: Does something else exist?
}
impl Codec for PermissionLevel {
    fn encoded_size(&self) -> usize {
        1
    }
    fn encode(&self, out: &mut [u8]) -> usize {
        out[0] = *self as u8;
        1
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
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
            size: 1,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
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

#[derive(Clone, Copy, Debug, PartialEq)]
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
#[derive(Clone, Copy, Debug, PartialEq)]
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NonVoid {
    // TODO Protect
    pub size: u32,
    pub file: FileOffsetOperand,
}
impl Codec for NonVoid {
    fn encoded_size(&self) -> usize {
        1 + unsafe { varint::size(self.size) } as usize + self.file.encoded_size()
    }
    fn encode(&self, out: &mut [u8]) -> usize {
        out[0] = QueryCode::NonVoid as u8;
        let mut offset = 1;
        offset += unsafe { varint::encode(self.size, &mut out[offset..]) } as usize;
        offset += self.file.encode(&mut out[offset..]);
        offset
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
        if out.len() < 3 {
            return Err(ParseError::MissingBytes(Some(3 - out.len())));
        }
        let mut offset = 1;
        let ParseValue {
            value: size,
            size: size_size,
        } = varint::decode(&out[offset..])?;
        offset += size_size;
        let ParseValue {
            value: file,
            size: file_size,
        } = FileOffsetOperand::decode(&out[offset..])?;
        offset += file_size;
        Ok(ParseValue {
            value: Self { size, file },
            size: offset,
        })
    }
}
#[test]
fn test_non_void_query_operand() {
    test_item(
        NonVoid {
            size: 4,
            file: FileOffsetOperand { id: 5, offset: 6 },
        },
        &hex!("00 04  05 06"),
    )
}

// TODO Check size coherence upon creation
#[derive(Clone, Debug, PartialEq)]
pub struct ComparisonWithZero {
    pub signed_data: bool,
    pub comparison_type: QueryComparisonType,
    // TODO Protect
    pub size: u32,
    pub mask: Option<Box<[u8]>>,
    pub file: FileOffsetOperand,
}
impl Codec for ComparisonWithZero {
    fn encoded_size(&self) -> usize {
        let mask_size = match self.mask {
            Some(_) => self.size as usize,
            None => 0,
        };
        1 + unsafe { varint::size(self.size) } as usize + mask_size + self.file.encoded_size()
    }
    fn encode(&self, out: &mut [u8]) -> usize {
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
        offset += unsafe { varint::encode(self.size, &mut out[offset..]) } as usize;
        if let Some(mask) = &self.mask {
            out[offset..offset + (self.size as usize)].clone_from_slice(&mask);
            offset += mask.len();
        }
        offset += self.file.encode(&mut out[offset..]);
        offset
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
        if out.len() < 1 + 1 + 2 {
            return Err(ParseError::MissingBytes(Some(1 + 1 + 2 - out.len())));
        }
        let mask_flag = out[0] & (1 << 4) != 0;
        let signed_data = out[0] & (1 << 3) != 0;
        let comparison_type = QueryComparisonType::from(out[0] & 0x07);
        let ParseValue {
            value: size,
            size: size_size,
        } = varint::decode(&out[1..])?;
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
            size: offset_size,
        } = FileOffsetOperand::decode(&out[offset..])?;
        offset += offset_size;
        Ok(ParseValue {
            value: Self {
                signed_data,
                comparison_type,
                size,
                mask,
                file,
            },
            size: offset,
        })
    }
}
#[test]
fn test_comparison_with_zero_operand() {
    test_item(
        ComparisonWithZero {
            signed_data: true,
            comparison_type: QueryComparisonType::Inequal,
            size: 3,
            mask: Some(vec![0, 1, 2].into_boxed_slice()),
            file: FileOffsetOperand { id: 4, offset: 5 },
        },
        &hex!("38 03  000102  04 05"),
    )
}

// TODO Check size coherence upon creation
#[derive(Clone, Debug, PartialEq)]
pub struct ComparisonWithValue {
    pub signed_data: bool,
    pub comparison_type: QueryComparisonType,
    // TODO Protect
    pub size: u32,
    pub mask: Option<Box<[u8]>>,
    pub value: Box<[u8]>,
    pub file: FileOffsetOperand,
}
impl Codec for ComparisonWithValue {
    fn encoded_size(&self) -> usize {
        let mask_size = match self.mask {
            Some(_) => self.size as usize,
            None => 0,
        };
        1 + unsafe { varint::size(self.size) } as usize
            + mask_size
            + self.value.len()
            + self.file.encoded_size()
    }
    fn encode(&self, out: &mut [u8]) -> usize {
        let mask_flag = match self.mask {
            Some(_) => 1,
            None => 0,
        };
        let signed_flag = if self.signed_data { 1 } else { 0 };
        let mut offset = 0;
        out[0] = ((QueryCode::ComparisonWithValue as u8) << 5)
            | (mask_flag << 4)
            | (signed_flag << 3)
            | self.comparison_type as u8;
        offset += 1;
        offset += unsafe { varint::encode(self.size, &mut out[offset..]) } as usize;
        if let Some(mask) = &self.mask {
            out[offset..offset + self.size as usize].clone_from_slice(&mask);
            offset += mask.len();
        }
        out[offset..offset + self.size as usize].clone_from_slice(&self.value[..]);
        offset += self.value.len();
        offset += self.file.encode(&mut out[offset..]);
        offset
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
        if out.len() < 1 + 1 + 2 {
            return Err(ParseError::MissingBytes(Some(1 + 1 + 2 - out.len())));
        }
        let mask_flag = out[0] & (1 << 4) != 0;
        let signed_data = out[0] & (1 << 3) != 0;
        let comparison_type = QueryComparisonType::from(out[0] & 0x07);
        let ParseValue {
            value: size,
            size: size_size,
        } = varint::decode(&out[1..])?;
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
            size: offset_size,
        } = FileOffsetOperand::decode(&out[offset..])?;
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
            size: offset,
        })
    }
}
#[test]
fn test_comparison_with_value_operand() {
    test_item(
        ComparisonWithValue {
            signed_data: false,
            comparison_type: QueryComparisonType::Equal,
            size: 3,
            mask: None,
            value: vec![9, 9, 9].into_boxed_slice(),
            file: FileOffsetOperand { id: 4, offset: 5 },
        },
        &hex!("41 03   090909  04 05"),
    )
}

// TODO Check size coherence upon creation
#[derive(Clone, Debug, PartialEq)]
pub struct ComparisonWithOtherFile {
    pub signed_data: bool,
    pub comparison_type: QueryComparisonType,
    // TODO Protect
    pub size: u32,
    pub mask: Option<Box<[u8]>>,
    pub file_src: FileOffsetOperand,
    pub file_dst: FileOffsetOperand,
}
impl Codec for ComparisonWithOtherFile {
    fn encoded_size(&self) -> usize {
        let mask_size = match self.mask {
            Some(_) => self.size as usize,
            None => 0,
        };
        1 + unsafe { varint::size(self.size) } as usize
            + mask_size
            + self.file_src.encoded_size()
            + self.file_dst.encoded_size()
    }
    fn encode(&self, out: &mut [u8]) -> usize {
        let mask_flag = match self.mask {
            Some(_) => 1,
            None => 0,
        };
        let signed_flag = if self.signed_data { 1 } else { 0 };
        let mut offset = 0;
        out[0] = ((QueryCode::ComparisonWithOtherFile as u8) << 5)
            | (mask_flag << 4)
            | (signed_flag << 3)
            | self.comparison_type as u8;
        offset += 1;
        offset += unsafe { varint::encode(self.size, &mut out[offset..]) } as usize;
        if let Some(mask) = &self.mask {
            out[offset..offset + self.size as usize].clone_from_slice(&mask);
            offset += mask.len();
        }
        // TODO ALP SPEC: Which of the offset operand is the source and the dest? (file 1 and 2)
        offset += self.file_src.encode(&mut out[offset..]);
        offset += self.file_dst.encode(&mut out[offset..]);
        offset
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
        if out.len() < 1 + 1 + 2 + 2 {
            return Err(ParseError::MissingBytes(Some(1 + 1 + 2 + 2 - out.len())));
        }
        let mask_flag = out[0] & (1 << 4) != 0;
        let signed_data = out[0] & (1 << 3) != 0;
        let comparison_type = QueryComparisonType::from(out[0] & 0x07);
        let ParseValue {
            value: size,
            size: size_size,
        } = varint::decode(&out[1..])?;
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
            size: file_src_size,
        } = FileOffsetOperand::decode(&out[offset..])?;
        offset += file_src_size;
        let ParseValue {
            value: file_dst,
            size: file_dst_size,
        } = FileOffsetOperand::decode(&out[offset..])?;
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
            size: offset,
        })
    }
}
#[test]
fn test_comparison_with_other_file_operand() {
    test_item(
        ComparisonWithOtherFile {
            signed_data: false,
            comparison_type: QueryComparisonType::GreaterThan,
            size: 2,
            mask: Some(vec![0xFF, 0xFF].into_boxed_slice()),
            file_src: FileOffsetOperand { id: 4, offset: 5 },
            file_dst: FileOffsetOperand { id: 8, offset: 9 },
        },
        &hex!("74 02 FFFF   04 05    08 09"),
    )
}

// TODO Check size coherence upon creation (start, stop and bitmap)
#[derive(Clone, Debug, PartialEq)]
pub struct BitmapRangeComparison {
    pub signed_data: bool,
    pub comparison_type: QueryRangeComparisonType,
    // TODO Protect
    pub size: u32,
    // ALP SPEC: TODO In theory, start and stop can be huge array thus impossible to cast into any trivial
    // number. How do we deal with this.
    // ALP SPEC: TODO What is the endianness of those start and stop fields?
    // TODO Enforce stop > start
    // TODO If the max size is settled, replace the buffer by the max size. This may take up more
    // memory, but would be way easier to use. Also it would avoid having to specify the ".size"
    // field.
    pub start: Box<[u8]>,
    pub stop: Box<[u8]>,
    // ALP SPEC: TODO How does the bitmap has to be aligned in the byte array? Aligned left or
    // right? Endianness?
    pub bitmap: Box<[u8]>, // TODO Better type?
    pub file: FileOffsetOperand,
}
impl Codec for BitmapRangeComparison {
    fn encoded_size(&self) -> usize {
        1 + unsafe { varint::size(self.size) } as usize
            + 2 * self.size as usize
            + self.bitmap.len()
            + self.file.encoded_size()
    }
    fn encode(&self, out: &mut [u8]) -> usize {
        let mut offset = 0;
        let signed_flag = if self.signed_data { 1 } else { 0 };
        out[0] = ((QueryCode::BitmapRangeComparison as u8) << 5)
            // | (0 << 4)
            | (signed_flag << 3)
            | self.comparison_type as u8;
        offset += 1;
        offset += unsafe { varint::encode(self.size, &mut out[offset..]) } as usize;
        out[offset..offset + self.size as usize].clone_from_slice(&self.start[..]);
        offset += self.start.len();
        out[offset..offset + self.size as usize].clone_from_slice(&self.stop[..]);
        offset += self.stop.len();
        out[offset..offset + self.bitmap.len()].clone_from_slice(&self.bitmap[..]);
        offset += self.bitmap.len();
        offset += self.file.encode(&mut out[offset..]);
        offset
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
        if out.len() < 1 + 1 + 2 {
            return Err(ParseError::MissingBytes(Some(1 + 1 + 2 - out.len())));
        }
        let signed_data = out[0] & (1 << 3) != 0;
        let comparison_type = QueryRangeComparisonType::from(out[0] & 0x07);
        let ParseValue {
            value: size32,
            size: size_size,
        } = varint::decode(&out[1..])?;
        let size = size32 as usize;
        let mut offset = 1 + size_size;
        let mut start = vec![0u8; size].into_boxed_slice();
        start.clone_from_slice(&out[offset..offset + size]);
        offset += size;
        let mut stop = vec![0u8; size].into_boxed_slice();
        stop.clone_from_slice(&out[offset..offset + size]);
        offset += size;
        // TODO Current max start/stop size chosen is u32 because that is the file size limit.
        // But in theory there is no requirement for the bitmap to have any relation with the
        // file sizes.
        let mut start_n = 0u32;
        let mut stop_n = 0u32;
        // TODO Endianness?
        for i in 0..size {
            start_n = (start_n << 8) + start[i] as u32;
            stop_n = (stop_n << 8) + stop[i] as u32;
        }
        let bitmap_size = (stop_n - start_n + 6) / 8; // ALP SPEC: Thanks for the calculation
        let mut bitmap = vec![0u8; bitmap_size as usize].into_boxed_slice();
        bitmap.clone_from_slice(&out[offset..offset + bitmap_size as usize]);
        offset += bitmap_size as usize;
        let ParseValue {
            value: file,
            size: file_size,
        } = FileOffsetOperand::decode(&out[offset..])?;
        offset += file_size;
        Ok(ParseValue {
            value: Self {
                signed_data,
                comparison_type,
                size: size32,
                start,
                stop,
                bitmap,
                file,
            },
            size: offset,
        })
    }
}
#[test]
fn test_bitmap_range_comparison_operand() {
    test_item(
        BitmapRangeComparison {
            signed_data: false,
            comparison_type: QueryRangeComparisonType::InRange,
            size: 2,

            start: Box::new(hex!("00 03")),
            stop: Box::new(hex!("00 20")),
            bitmap: Box::new(hex!("01020304")),

            file: FileOffsetOperand { id: 0, offset: 4 },
        },
        &hex!("81 02 0003  0020  01020304  00 04"),
    )
}

// TODO Check size coherence upon creation
#[derive(Clone, Debug, PartialEq)]
pub struct StringTokenSearch {
    pub max_errors: u8,
    // TODO Protect
    pub size: u32,
    pub mask: Option<Box<[u8]>>,
    pub value: Box<[u8]>,
    pub file: FileOffsetOperand,
}
impl Codec for StringTokenSearch {
    fn encoded_size(&self) -> usize {
        let mask_size = match self.mask {
            Some(_) => self.size as usize,
            None => 0,
        };
        1 + unsafe { varint::size(self.size) } as usize
            + mask_size
            + self.value.len()
            + self.file.encoded_size()
    }
    fn encode(&self, out: &mut [u8]) -> usize {
        let mask_flag = match self.mask {
            Some(_) => 1,
            None => 0,
        };
        let mut offset = 0;
        out[0] = ((QueryCode::StringTokenSearch as u8) << 5)
            | (mask_flag << 4)
            // | (0 << 3)
            | self.max_errors;
        offset += 1;
        offset += unsafe { varint::encode(self.size, &mut out[offset..]) } as usize;
        if let Some(mask) = &self.mask {
            out[offset..offset + self.size as usize].clone_from_slice(&mask);
            offset += mask.len();
        }
        out[offset..offset + self.size as usize].clone_from_slice(&self.value[..]);
        offset += self.value.len();
        offset += self.file.encode(&mut out[offset..]);
        offset
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
        if out.len() < 1 + 1 + 2 {
            return Err(ParseError::MissingBytes(Some(1 + 1 + 2 - out.len())));
        }
        let mask_flag = out[0] & (1 << 4) != 0;
        let max_errors = out[0] & 0x07;
        let ParseValue {
            value: size32,
            size: size_size,
        } = varint::decode(&out[1..])?;
        let size = size32 as usize;
        let mut offset = 1 + size_size;
        let mask = if mask_flag {
            let mut data = vec![0u8; size].into_boxed_slice();
            data.clone_from_slice(&out[offset..offset + size]);
            offset += size;
            Some(data)
        } else {
            None
        };
        let mut value = vec![0u8; size].into_boxed_slice();
        value.clone_from_slice(&out[offset..offset + size]);
        offset += size;
        let ParseValue {
            value: file,
            size: offset_size,
        } = FileOffsetOperand::decode(&out[offset..])?;
        offset += offset_size;
        Ok(ParseValue {
            value: Self {
                max_errors,
                size: size32,
                mask,
                value,
                file,
            },
            size: offset,
        })
    }
}
#[test]
fn test_string_token_search_operand() {
    test_item(
        StringTokenSearch {
            max_errors: 2,
            size: 4,
            mask: Some(Box::new(hex!("FF00FF00"))),
            value: Box::new(hex!("01020304")),
            file: FileOffsetOperand { id: 0, offset: 4 },
        },
        &hex!("F2 04 FF00FF00  01020304  00 04"),
    )
}

#[derive(Clone, Debug, PartialEq)]
pub enum QueryOperand {
    NonVoid(NonVoid),
    ComparisonWithZero(ComparisonWithZero),
    ComparisonWithValue(ComparisonWithValue),
    ComparisonWithOtherFile(ComparisonWithOtherFile),
    BitmapRangeComparison(BitmapRangeComparison),
    StringTokenSearch(StringTokenSearch),
}
impl Codec for QueryOperand {
    fn encoded_size(&self) -> usize {
        match self {
            QueryOperand::NonVoid(v) => v.encoded_size(),
            QueryOperand::ComparisonWithZero(v) => v.encoded_size(),
            QueryOperand::ComparisonWithValue(v) => v.encoded_size(),
            QueryOperand::ComparisonWithOtherFile(v) => v.encoded_size(),
            QueryOperand::BitmapRangeComparison(v) => v.encoded_size(),
            QueryOperand::StringTokenSearch(v) => v.encoded_size(),
        }
    }
    fn encode(&self, out: &mut [u8]) -> usize {
        match self {
            QueryOperand::NonVoid(v) => v.encode(out),
            QueryOperand::ComparisonWithZero(v) => v.encode(out),
            QueryOperand::ComparisonWithValue(v) => v.encode(out),
            QueryOperand::ComparisonWithOtherFile(v) => v.encode(out),
            QueryOperand::BitmapRangeComparison(v) => v.encode(out),
            QueryOperand::StringTokenSearch(v) => v.encode(out),
        }
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
        Ok(match QueryCode::from(out[0] >> 5) {
            QueryCode::NonVoid => NonVoid::decode(out)?.map_value(QueryOperand::NonVoid),
            QueryCode::ComparisonWithZero => {
                ComparisonWithZero::decode(out)?.map_value(QueryOperand::ComparisonWithZero)
            }
            QueryCode::ComparisonWithValue => {
                ComparisonWithValue::decode(out)?.map_value(QueryOperand::ComparisonWithValue)
            }
            QueryCode::ComparisonWithOtherFile => ComparisonWithOtherFile::decode(out)?
                .map_value(QueryOperand::ComparisonWithOtherFile),
            QueryCode::BitmapRangeComparison => {
                BitmapRangeComparison::decode(out)?.map_value(QueryOperand::BitmapRangeComparison)
            }
            QueryCode::StringTokenSearch => {
                StringTokenSearch::decode(out)?.map_value(QueryOperand::StringTokenSearch)
            }
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct OverloadedIndirectInterface {
    pub interface_file_id: u8,
    pub addressee: Addressee,
}

impl Codec for OverloadedIndirectInterface {
    fn encoded_size(&self) -> usize {
        1 + self.addressee.encoded_size()
    }
    fn encode(&self, out: &mut [u8]) -> usize {
        out[0] = self.interface_file_id;
        1 + self.addressee.encode(&mut out[1..])
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
        if out.len() < 1 + 2 {
            return Err(ParseError::MissingBytes(Some(1 + 2 - out.len())));
        }
        let interface_file_id = out[0];
        let ParseValue {
            value: addressee,
            size: addressee_size,
        } = Addressee::decode(&out[1..])?;
        Ok(ParseValue {
            value: Self {
                interface_file_id,
                addressee,
            },
            size: 1 + addressee_size,
        })
    }
}
#[test]
fn test_overloaded_indirect_interface() {
    test_item(
        OverloadedIndirectInterface {
            interface_file_id: 4,
            addressee: Addressee {
                nls_method: NlsMethod::AesCcm32,
                access_class: 0xFF,
                address: Address::Vid(Box::new([0xAB, 0xCD])),
            },
        },
        &hex!("04   37 FF ABCD"),
    )
}

#[derive(Clone, Debug, PartialEq)]
// ALP SPEC: This seems undoable if we do not know the interface (per protocol specific support)
pub struct NonOverloadedIndirectInterface {
    pub interface_file_id: u8,
    // ALP SPEC: Where is this defined? Is this ID specific?
    pub data: Box<[u8]>,
}

impl Codec for NonOverloadedIndirectInterface {
    fn encoded_size(&self) -> usize {
        1 + self.data.len()
    }
    fn encode(&self, out: &mut [u8]) -> usize {
        // TODO Add overload flag (with op byte mod) (shift out bytes by 1)
        out[0] = self.interface_file_id;
        let mut offset = 1;
        out[offset..].clone_from_slice(&self.data);
        offset += self.data.len();
        // ALP SPEC: TODO: What should we do
        todo!("{}", offset)
    }
    fn decode(_out: &[u8]) -> ParseResult<Self> {
        todo!("TODO")
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum IndirectInterface {
    Overloaded(OverloadedIndirectInterface),
    NonOverloaded(NonOverloadedIndirectInterface),
}

impl Codec for IndirectInterface {
    fn encoded_size(&self) -> usize {
        match self {
            IndirectInterface::Overloaded(v) => v.encoded_size(),
            IndirectInterface::NonOverloaded(v) => v.encoded_size(),
        }
    }
    fn encode(&self, out: &mut [u8]) -> usize {
        match self {
            IndirectInterface::Overloaded(v) => v.encode(out),
            IndirectInterface::NonOverloaded(v) => v.encode(out),
        }
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
        if out.is_empty() {
            return Err(ParseError::MissingBytes(Some(1)));
        }
        Ok(if out[0] & 0x80 != 0 {
            OverloadedIndirectInterface::decode(&out[1..])?
                .map(|v, i| (IndirectInterface::Overloaded(v), i + 1))
        } else {
            NonOverloadedIndirectInterface::decode(&out[1..])?
                .map(|v, i| (IndirectInterface::NonOverloaded(v), i + 1))
        })
    }
}

// ===============================================================================
// Actions
// ===============================================================================
// Nop
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Nop {
    pub group: bool,
    pub resp: bool,
}
impl_op_serialized!(Nop, group, resp);
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

// Read
// TODO Protect varint init
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReadFileData {
    pub group: bool,
    pub resp: bool,
    pub file_id: u8,
    // TODO Protect
    pub offset: u32,
    // TODO Protect
    pub size: u32,
}

impl Codec for ReadFileData {
    fn encoded_size(&self) -> usize {
        1 + 1 + unsafe_varint_serialize_sizes!(self.offset, self.size) as usize
    }
    fn encode(&self, out: &mut [u8]) -> usize {
        out[0] = control_byte!(self.group, self.resp, OpCode::ReadFileData);
        out[1] = self.file_id;
        1 + 1 + unsafe_varint_serialize!(out[2..], self.offset, self.size)
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
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
            size: offset_size,
        } = varint::decode(&out[off..])?;
        off += offset_size;
        let ParseValue {
            value: size,
            size: size_size,
        } = varint::decode(&out[off..])?;
        off += size_size;
        Ok(ParseValue {
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReadFileProperties {
    pub group: bool,
    pub resp: bool,
    pub file_id: u8,
}
impl_simple_op!(ReadFileProperties, group, resp, file_id);
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
// TODO Protect varint init, data consistency
#[derive(Clone, Debug, PartialEq)]
pub struct WriteFileData {
    pub group: bool,
    pub resp: bool,
    pub file_id: u8,
    // TODO Protect
    pub offset: u32,
    pub data: Box<[u8]>,
}
impl Codec for WriteFileData {
    fn encoded_size(&self) -> usize {
        1 + 1
            + unsafe_varint_serialize_sizes!(self.offset, self.data.len() as u32) as usize
            + self.data.len()
    }
    fn encode(&self, out: &mut [u8]) -> usize {
        out[0] = control_byte!(self.group, self.resp, OpCode::WriteFileData);
        out[1] = self.file_id;
        let mut offset = 2;
        offset += unsafe_varint_serialize!(out[2..], self.offset, self.data.len() as u32) as usize;
        out[offset..offset + self.data.len()].clone_from_slice(&self.data[..]);
        offset += self.data.len();
        offset
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
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
            size: offset_size,
        } = varint::decode(&out[off..])?;
        off += offset_size;
        let ParseValue {
            value: size,
            size: size_size,
        } = varint::decode(&out[off..])?;
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

// #[derive(Clone, Copy, Debug, PartialEq)]
// pub struct WriteFileDataFlush {
//     pub group: bool,
//     pub resp: bool,
//     pub file_id: u8,
//     // TODO Protect
//     pub offset: u32,
//     pub data: Box<[u8]>,
// }
// impl Codec for WriteFileDataFlush {
//     fn encoded_size(&self) -> usize {
//         1 + 1
//             + unsafe_varint_serialize_sizes!(self.offset, self.data.len() as u32) as usize
//             + self.data.len()
//     }
//     fn encode(&self, out: &mut [u8]) -> usize {
//         out[0] = control_byte!(self.group, self.resp, OpCode::WriteFileDataFlush);
//         out[1] = self.file_id;
//         let mut offset = 2;
//         offset += unsafe_varint_serialize!(out[2..], self.offset, self.data.len() as u32) as usize;
//         out[offset..offset + self.data.len()].clone_from_slice(&self.data[..]);
//         offset += self.data.len();
//         offset
//     }
//     fn decode(out: &[u8]) -> ParseResult<Self> {
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
//             size: offset_size,
//         } = varint::decode(&out[off..])?;
//         off += offset_size;
//         let ParseValue {
//             value: size,
//             size: size_size,
//         } = varint::decode(&out[off..])?;
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
//             size: off,
//         })
//     }
// }
// #[test]
// fn test_write_file_data_flush() {
//     test_item(
//         WriteFileDataFlush {
//             group: true,
//             resp: false,
//             file_id: 9,
//             offset: 5,
//             data: Box::new(hex!("01 02 03")),
//         },
//         &hex!("84   09 05 03  010203"),
//     )
// }

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WriteFileProperties {
    pub group: bool,
    pub resp: bool,
    pub file_id: u8,
    // TODO
    // ALP SPEC: Missing link to find definition in ALP spec
    pub data: [u8; 12],
}
impl_header_op!(WriteFileProperties, group, resp, file_id, data);
#[test]
fn test_write_file_properties() {
    test_item(
        WriteFileProperties {
            group: true,
            resp: false,
            file_id: 9,
            data: hex!("01 02 03 04 05 06 07 08 09 0A 0B 0C"),
        },
        &hex!("86   09   0102030405060708090A0B0C"),
    )
}

#[derive(Clone, Debug, PartialEq)]
pub struct ActionQuery {
    pub group: bool,
    pub resp: bool,
    pub query: QueryOperand,
}
impl_op_serialized!(ActionQuery, group, resp, query, QueryOperand);
#[test]
fn test_action_query() {
    test_item(
        ActionQuery {
            group: true,
            resp: true,
            query: QueryOperand::NonVoid(NonVoid {
                size: 4,
                file: FileOffsetOperand { id: 5, offset: 6 },
            }),
        },
        &hex!("C8   00 04  05 06"),
    )
}

#[derive(Clone, Debug, PartialEq)]
pub struct BreakQuery {
    pub group: bool,
    pub resp: bool,
    pub query: QueryOperand,
}
impl_op_serialized!(BreakQuery, group, resp, query, QueryOperand);
#[test]
fn test_break_query() {
    test_item(
        BreakQuery {
            group: true,
            resp: true,
            query: QueryOperand::NonVoid(NonVoid {
                size: 4,
                file: FileOffsetOperand { id: 5, offset: 6 },
            }),
        },
        &hex!("C9   00 04  05 06"),
    )
}

#[derive(Clone, Copy, Debug, PartialEq)]
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
#[test]
fn test_permission_request() {
    test_item(
        PermissionRequest {
            group: false,
            resp: false,
            level: PermissionLevel::Root,
            permission: Permission::Dash7(hex!("0102030405060708")),
        },
        &hex!("0A   01 42 0102030405060708"),
    )
}

#[derive(Clone, Debug, PartialEq)]
pub struct VerifyChecksum {
    pub group: bool,
    pub resp: bool,
    pub query: QueryOperand,
}
impl_op_serialized!(VerifyChecksum, group, resp, query, QueryOperand);
#[test]
fn test_verify_checksum() {
    test_item(
        VerifyChecksum {
            group: false,
            resp: false,
            query: QueryOperand::NonVoid(NonVoid {
                size: 4,
                file: FileOffsetOperand { id: 5, offset: 6 },
            }),
        },
        &hex!("0B   00 04  05 06"),
    )
}

// Management
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExistFile {
    pub group: bool,
    pub resp: bool,
    pub file_id: u8,
}
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CreateNewFile {
    pub group: bool,
    pub resp: bool,
    pub file_id: u8,
    // TODO
    // ALP SPEC: Missing link to find definition in ALP spec
    pub data: [u8; 12],
}
impl_header_op!(CreateNewFile, group, resp, file_id, data);
#[test]
fn test_create_new_file() {
    test_item(
        CreateNewFile {
            group: true,
            resp: false,
            file_id: 3,
            data: hex!("01 02 03 04 05 06 07 08 09 0A 0B 0C"),
        },
        &hex!("91   03   0102030405060708090A0B0C"),
    )
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DeleteFile {
    pub group: bool,
    pub resp: bool,
    pub file_id: u8,
}
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RestoreFile {
    pub group: bool,
    pub resp: bool,
    pub file_id: u8,
}
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlushFile {
    pub group: bool,
    pub resp: bool,
    pub file_id: u8,
}
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CopyFile {
    pub group: bool,
    pub resp: bool,
    pub src_file_id: u8,
    pub dst_file_id: u8,
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExecuteFile {
    pub group: bool,
    pub resp: bool,
    pub file_id: u8,
}
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
#[derive(Clone, Debug, PartialEq)]
pub struct ReturnFileData {
    pub group: bool,
    pub resp: bool,
    pub file_id: u8,
    // TODO Protect
    pub offset: u32,
    pub data: Box<[u8]>,
}
impl Codec for ReturnFileData {
    fn encoded_size(&self) -> usize {
        1 + 1
            + unsafe_varint_serialize_sizes!(self.offset, self.data.len() as u32) as usize
            + self.data.len()
    }
    fn encode(&self, out: &mut [u8]) -> usize {
        out[0] = control_byte!(self.group, self.resp, OpCode::ReturnFileData);
        out[1] = self.file_id;
        let mut offset = 2;
        offset += unsafe_varint_serialize!(out[2..], self.offset, self.data.len() as u32) as usize;
        out[offset..offset + self.data.len()].clone_from_slice(&self.data[..]);
        offset += self.data.len();
        offset
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
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
            size: offset_size,
        } = varint::decode(&out[off..])?;
        off += offset_size;
        let ParseValue {
            value: size,
            size: size_size,
        } = varint::decode(&out[off..])?;
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReturnFileProperties {
    pub group: bool,
    pub resp: bool,
    pub file_id: u8,
    // TODO
    // ALP SPEC: Missing link to find definition in ALP spec
    pub data: [u8; 12],
}
impl_header_op!(ReturnFileProperties, group, resp, file_id, data);
#[test]
fn test_return_file_properties() {
    test_item(
        ReturnFileProperties {
            group: false,
            resp: false,
            file_id: 9,
            data: hex!("01 02 03 04 05 06 07 08 09 0A 0B 0C"),
        },
        &hex!("21   09   0102030405060708090A0B0C"),
    )
}

#[derive(Clone, Copy, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
pub enum Status {
    // ALP SPEC: This is named status, but it should be named action status compared to the '2'
    // other statuses.
    Action(StatusOperand),
    Interface(InterfaceStatus),
    // ALP SPEC: Where are the stack errors?
}
impl Codec for Status {
    fn encoded_size(&self) -> usize {
        1 + match self {
            Status::Action(op) => op.encoded_size(),
            Status::Interface(op) => op.encoded_size(),
        }
    }
    fn encode(&self, out: &mut [u8]) -> usize {
        out[0] = OpCode::Status as u8
            + ((match self {
                Status::Action(_) => StatusType::Action,
                Status::Interface(_) => StatusType::Interface,
            } as u8)
                << 6);
        let out = &mut out[1..];
        1 + match self {
            Status::Action(op) => op.encode(out),
            Status::Interface(op) => op.encode(out),
        }
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
        if out.is_empty() {
            return Err(ParseError::MissingBytes(Some(1)));
        }
        let status_type = out[0] >> 6;
        Ok(match StatusType::from(status_type) {
            StatusType::Action => {
                StatusOperand::decode(&out[1..])?.map(|v, size| (Status::Action(v), size + 1))
            }
            StatusType::Interface => {
                InterfaceStatus::decode(&out[1..])?.map(|v, size| (Status::Interface(v), size + 1))
            }
        })
    }
}
#[test]
fn test_status() {
    test_item(
        Status::Action(StatusOperand {
            action_id: 2,
            status: StatusCode::UnknownOperation,
        }),
        &hex!("22 02 F6"),
    )
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResponseTag {
    pub eop: bool, // End of packet
    pub err: bool,
    pub id: u8,
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
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Chunk {
    pub step: ChunkStep,
}
impl Codec for Chunk {
    fn encoded_size(&self) -> usize {
        1
    }
    fn encode(&self, out: &mut [u8]) -> usize {
        out[0] = OpCode::Chunk as u8 + ((self.step as u8) << 6);
        1
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
        if out.is_empty() {
            return Err(ParseError::MissingBytes(Some(1)));
        }
        Ok(ParseValue {
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

#[derive(Clone, Copy, Debug, PartialEq)]
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
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Logic {
    pub logic: LogicOp,
}
impl Codec for Logic {
    fn encoded_size(&self) -> usize {
        1
    }
    fn encode(&self, out: &mut [u8]) -> usize {
        out[0] = OpCode::Logic as u8 + ((self.logic as u8) << 6);
        1
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
        if out.is_empty() {
            return Err(ParseError::MissingBytes(Some(1)));
        }
        Ok(ParseValue {
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

#[derive(Clone, Debug, PartialEq)]
pub struct Forward {
    pub resp: bool,
    pub conf: InterfaceConfiguration,
}
impl Codec for Forward {
    fn encoded_size(&self) -> usize {
        1 + self.conf.encoded_size()
    }
    fn encode(&self, out: &mut [u8]) -> usize {
        out[0] = control_byte!(false, self.resp, OpCode::Forward);
        1 + self.conf.encode(&mut out[1..])
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
        let min_size = 1 + 1;
        if out.len() < min_size {
            return Err(ParseError::MissingBytes(Some(min_size - out.len())));
        }
        let ParseValue {
            value: conf,
            size: conf_size,
        } = InterfaceConfiguration::decode(&out[1..])?;
        Ok(ParseValue {
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
            conf: InterfaceConfiguration::Host,
        },
        &hex!("72 00"),
    )
}

#[derive(Clone, Debug, PartialEq)]
pub struct IndirectForward {
    pub resp: bool,
    pub interface: IndirectInterface,
}
impl Codec for IndirectForward {
    fn encoded_size(&self) -> usize {
        1 + self.interface.encoded_size()
    }
    fn encode(&self, out: &mut [u8]) -> usize {
        let overload = match self.interface {
            IndirectInterface::Overloaded(_) => true,
            IndirectInterface::NonOverloaded(_) => false,
        };
        out[0] = control_byte!(overload, self.resp, OpCode::IndirectForward);
        1 + serialize_all!(&mut out[1..], &self.interface)
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
        if out.is_empty() {
            Err(ParseError::MissingBytes(Some(1)))
        } else {
            let mut offset = 0;
            let ParseValue {
                value: op1,
                size: op1_size,
            } = IndirectInterface::decode(out)?;
            offset += op1_size;
            Ok(ParseValue {
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
            interface: IndirectInterface::Overloaded(OverloadedIndirectInterface {
                interface_file_id: 4,
                addressee: Addressee {
                    nls_method: NlsMethod::AesCcm32,
                    access_class: 0xFF,
                    address: Address::Vid(Box::new([0xAB, 0xCD])),
                },
            }),
        },
        &hex!("F3   04   37 FF ABCD"),
    )
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RequestTag {
    pub eop: bool, // End of packet
    pub id: u8,
}
impl Codec for RequestTag {
    fn encoded_size(&self) -> usize {
        1 + 1
    }
    fn encode(&self, out: &mut [u8]) -> usize {
        out[0] = control_byte!(self.eop, false, OpCode::RequestTag);
        out[1] = self.id;
        1 + 1
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
        let min_size = 1 + 1;
        if out.len() < min_size {
            return Err(ParseError::MissingBytes(Some(min_size - out.len())));
        }
        Ok(ParseValue {
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Extension {
    pub group: bool,
    pub resp: bool,
}
impl Codec for Extension {
    fn encoded_size(&self) -> usize {
        todo!()
    }
    fn encode(&self, _out: &mut [u8]) -> usize {
        todo!()
    }
    fn decode(_out: &[u8]) -> ParseResult<Self> {
        todo!()
    }
}

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

impl Codec for Action {
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
    fn encode(&self, out: &mut [u8]) -> usize {
        match self {
            Action::Nop(x) => x.encode(out),
            Action::ReadFileData(x) => x.encode(out),
            Action::ReadFileProperties(x) => x.encode(out),
            Action::WriteFileData(x) => x.encode(out),
            // Action::WriteFileDataFlush(x) => x.encode(out),
            Action::WriteFileProperties(x) => x.encode(out),
            Action::ActionQuery(x) => x.encode(out),
            Action::BreakQuery(x) => x.encode(out),
            Action::PermissionRequest(x) => x.encode(out),
            Action::VerifyChecksum(x) => x.encode(out),
            Action::ExistFile(x) => x.encode(out),
            Action::CreateNewFile(x) => x.encode(out),
            Action::DeleteFile(x) => x.encode(out),
            Action::RestoreFile(x) => x.encode(out),
            Action::FlushFile(x) => x.encode(out),
            Action::CopyFile(x) => x.encode(out),
            Action::ExecuteFile(x) => x.encode(out),
            Action::ReturnFileData(x) => x.encode(out),
            Action::ReturnFileProperties(x) => x.encode(out),
            Action::Status(x) => x.encode(out),
            Action::ResponseTag(x) => x.encode(out),
            Action::Chunk(x) => x.encode(out),
            Action::Logic(x) => x.encode(out),
            Action::Forward(x) => x.encode(out),
            Action::IndirectForward(x) => x.encode(out),
            Action::RequestTag(x) => x.encode(out),
            Action::Extension(x) => x.encode(out),
        }
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
        if out.is_empty() {
            return Err(ParseError::MissingBytes(Some(1)));
        }
        let opcode = OpCode::from(out[0] & 0x3F);
        Ok(match opcode {
            OpCode::Nop => Nop::decode(&out)?.map_value(Action::Nop),
            OpCode::ReadFileData => ReadFileData::decode(&out)?.map_value(Action::ReadFileData),
            OpCode::ReadFileProperties => {
                ReadFileProperties::decode(&out)?.map_value(Action::ReadFileProperties)
            }
            OpCode::WriteFileData => WriteFileData::decode(&out)?.map_value(Action::WriteFileData),
            // OpCode::WriteFileDataFlush => {
            //     WriteFileDataFlush::decode(&out)?.map_value( Action::WriteFileDataFlush)
            // }
            OpCode::WriteFileProperties => {
                WriteFileProperties::decode(&out)?.map_value(Action::WriteFileProperties)
            }
            OpCode::ActionQuery => ActionQuery::decode(&out)?.map_value(Action::ActionQuery),
            OpCode::BreakQuery => BreakQuery::decode(&out)?.map_value(Action::BreakQuery),
            OpCode::PermissionRequest => {
                PermissionRequest::decode(&out)?.map_value(Action::PermissionRequest)
            }
            OpCode::VerifyChecksum => {
                VerifyChecksum::decode(&out)?.map_value(Action::VerifyChecksum)
            }
            OpCode::ExistFile => ExistFile::decode(&out)?.map_value(Action::ExistFile),
            OpCode::CreateNewFile => CreateNewFile::decode(&out)?.map_value(Action::CreateNewFile),
            OpCode::DeleteFile => DeleteFile::decode(&out)?.map_value(Action::DeleteFile),
            OpCode::RestoreFile => RestoreFile::decode(&out)?.map_value(Action::RestoreFile),
            OpCode::FlushFile => FlushFile::decode(&out)?.map_value(Action::FlushFile),
            OpCode::CopyFile => CopyFile::decode(&out)?.map_value(Action::CopyFile),
            OpCode::ExecuteFile => ExecuteFile::decode(&out)?.map_value(Action::ExecuteFile),
            OpCode::ReturnFileData => {
                ReturnFileData::decode(&out)?.map_value(Action::ReturnFileData)
            }
            OpCode::ReturnFileProperties => {
                ReturnFileProperties::decode(&out)?.map_value(Action::ReturnFileProperties)
            }
            OpCode::Status => Status::decode(&out)?.map_value(Action::Status),
            OpCode::ResponseTag => ResponseTag::decode(&out)?.map_value(Action::ResponseTag),
            OpCode::Chunk => Chunk::decode(&out)?.map_value(Action::Chunk),
            OpCode::Logic => Logic::decode(&out)?.map_value(Action::Logic),
            OpCode::Forward => Forward::decode(&out)?.map_value(Action::Forward),
            OpCode::IndirectForward => {
                IndirectForward::decode(&out)?.map_value(Action::IndirectForward)
            }
            OpCode::RequestTag => RequestTag::decode(&out)?.map_value(Action::RequestTag),
            OpCode::Extension => Extension::decode(&out)?.map_value(Action::Extension),
        })
    }
}

// ===============================================================================
// Command
// ===============================================================================
#[derive(Clone, Debug, PartialEq)]
pub struct Command {
    pub actions: Vec<Action>,
}
#[derive(Clone, Debug, PartialEq)]
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
    fn partial_decode(out: &[u8]) -> Result<ParseValue<Command>, CommandParseError> {
        let mut actions = vec![];
        let mut offset = 0;
        loop {
            if out.is_empty() {
                break;
            }
            match Action::decode(&out[offset..]) {
                Ok(ParseValue { value, size }) => {
                    actions.push(value);
                    offset += size;
                }
                Err(error) => return Err(CommandParseError { actions, error }),
            }
        }
        Ok(ParseValue {
            value: Self { actions },
            size: offset,
        })
    }
}
impl Codec for Command {
    fn encoded_size(&self) -> usize {
        self.actions.iter().map(|act| act.encoded_size()).sum()
    }
    fn encode(&self, out: &mut [u8]) -> usize {
        let mut offset = 0;
        for action in self.actions.iter() {
            offset += action.encode(&mut out[offset..]);
        }
        offset
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
        Self::partial_decode(out).map_err(|v| v.error)
    }
}

#[cfg(test)]
use crate::test_tools::test_item;
use crate::{
    codec::{Codec, ParseError, ParseFail, ParseResult, ParseResultExtension, ParseValue},
    Enum,
};
#[cfg(test)]
use hex_literal::hex;

/// Encryption algorigthm for over-the-air packets
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
    fn from(n: u8) -> Result<NlsMethod, ParseFail> {
        Ok(match n {
            0 => NlsMethod::None,
            1 => NlsMethod::AesCtr,
            2 => NlsMethod::AesCbcMac128,
            3 => NlsMethod::AesCbcMac64,
            4 => NlsMethod::AesCbcMac32,
            5 => NlsMethod::AesCcm128,
            6 => NlsMethod::AesCcm64,
            7 => NlsMethod::AesCcm32,
            x => {
                return Err(ParseFail::Error {
                    error: ParseError::ImpossibleValue {
                        en: Enum::NlsMethod,
                        value: x,
                    },
                    offset: 0,
                })
            }
        })
    }
}

/// Dash7 address types
// ALP SPEC: Where is this defined?
#[derive(Clone, Debug, PartialEq)]
pub enum Address {
    // D7A SPEC: It is not clear that the estimated reached has to be placed on the "ID" field.
    /// Broadcast to an estimated number of receivers encoded in compressed format on a byte.
    NbId(u8),
    /// Broadcast to everyone
    NoId,
    /// Unicast to target via its UID (Unique Dash7 ID)
    Uid(Box<[u8; 8]>),
    /// Unicast to target via its VID (Virtual ID)
    Vid(Box<[u8; 2]>),
}
/// All the parameters required to address a target
#[derive(Clone, Debug, PartialEq)]
pub struct Addressee {
    /// Encrypting method
    pub nls_method: NlsMethod,
    /// Listening access class of the target
    pub access_class: u8,
    /// Address of the target
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
    unsafe fn encode(&self, out: &mut [u8]) -> usize {
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
            return Err(ParseFail::MissingBytes(SIZE - out.len()));
        }
        let id_type = (out[0] & 0x30) >> 4;
        let nls_method = NlsMethod::from(out[0] & 0x0F)?;
        let access_class = out[1];
        let (address, address_size) = match id_type {
            0 => {
                if out.len() < 3 {
                    return Err(ParseFail::MissingBytes(1));
                }
                (Address::NbId(out[2]), 1)
            }
            1 => (Address::NoId, 0),
            2 => {
                if out.len() < 2 + 8 {
                    return Err(ParseFail::MissingBytes(2 + 8 - out.len()));
                }
                let mut data = Box::new([0u8; 8]);
                data.clone_from_slice(&out[2..2 + 8]);
                (Address::Uid(data), 8)
            }
            3 => {
                if out.len() < 2 + 2 {
                    return Err(ParseFail::MissingBytes(2 + 2 - out.len()));
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
// ALP_SPEC: Aren't there supposed to be more retry modes?
/// The Retry Modes define the pattern for re-flushing a FIFO that terminates on error.
///
/// In other words, what is the retry policy when sending your payload.
pub enum RetryMode {
    No = 0,
}
impl RetryMode {
    fn from(n: u8) -> Result<Self, ParseFail> {
        Ok(match n {
            0 => RetryMode::No,
            x => {
                return Err(ParseFail::Error {
                    error: ParseError::ImpossibleValue {
                        en: Enum::RetryMode,
                        value: x,
                    },
                    offset: 0,
                })
            }
        })
    }
}

/// The Response Modes define the condition for termination on success of a Request
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RespMode {
    /// A Request is acknowledged if the DLL CSMA-CA routine succeeds. No
    /// responses are expected.
    ///
    /// Eg. The request is successful if your packet was successfully sent on the radio.
    No = 0,
    /// If the addressee is broadcast, a Request is acknowledged if as many as
    /// possible D7ATP responses to this Request are received (may be zero).
    ///
    /// If the addressee is unicast, a Request is acknowledged if the addressee provides a
    /// D7ATP response. All responses received during the D7ATP Receive Period
    /// are vectored to upper layer.
    ///
    /// Eg. Succeeds if everyone addressed responds to the radio packet.
    All = 1,
    /// A Request is acknowledged if at least one D7ATP response to this Request is
    /// received.
    ///
    /// Eg. Succeeds if you receive one response to the radio packet.
    Any = 2,
    /// A Request is acknowledged if the DLL CSMA-CA routine succeeds REPEAT
    /// times. No responses are expected. The parameters REPEAT is defined in the
    /// SEL configuration file.
    RespNoRpt = 4,
    /// A Request is acknowledged if the DLL CSMA-CA routine succeeds. It is un-
    /// acknowledged when a response does not acknowledge the Request. The
    /// procedure behaves as RESP_ALL, but Responders provide responses only
    /// when their D7ATP ACK Templates is not void or if the upper layer provides a
    /// response.
    ///
    /// Eg. Succeeds only if the responder gives back an ALP packet in response (which is more
    /// restrictive than succeeding upon successful radio ACK).
    RespOnData = 5,
    /// A Request is acknowledged if at least one D7ATP response to this Request is
    /// received. The procedure behaves as RESP_ANY, but the Addressee is
    /// managed dynamically. It is set to broadcast after failure to receive an
    /// acknowledgement. On acknowledgement success, it is set to the
    /// Addressee of one of the responders that acknowledged the Request (preferred
    /// addressee). The preferred addressee selection is implementation dependent.
    RespPreferred = 6,
}
impl RespMode {
    fn from(n: u8) -> Result<Self, ParseFail> {
        Ok(match n {
            0 => RespMode::No,
            1 => RespMode::All,
            2 => RespMode::Any,
            4 => RespMode::RespNoRpt,
            5 => RespMode::RespOnData,
            6 => RespMode::RespPreferred,
            x => {
                return Err(ParseFail::Error {
                    error: ParseError::ImpossibleValue {
                        en: Enum::RespMode,
                        value: x,
                    },
                    offset: 0,
                })
            }
        })
    }
}

/// Qos of the request
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Qos {
    pub retry: RetryMode,
    pub resp: RespMode,
}
impl Codec for Qos {
    fn encoded_size(&self) -> usize {
        1
    }
    unsafe fn encode(&self, out: &mut [u8]) -> usize {
        out[0] = ((self.retry as u8) << 3) + self.resp as u8;
        1
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
        if out.is_empty() {
            return Err(ParseFail::MissingBytes(1));
        }
        let retry = RetryMode::from((out[0] & 0x38) >> 3)?;
        let resp = RespMode::from(out[0] & 0x07)?;
        Ok(ParseValue {
            value: Self { retry, resp },
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

/// Section 9.2.1
///
/// Parameters to handle the sending of a request.
// ALP SPEC: Add link to D7a section
#[derive(Clone, Debug, PartialEq)]
pub struct InterfaceConfiguration {
    pub qos: Qos,
    /// Flush Start Timeout in Compressed Format, unit is in seconds
    ///
    /// Maximum time to send the packet. This means that the modem will wait for a "good opportunity"
    /// to send the packet until the timeout, after which it will just send the packet over the
    /// air.
    ///
    /// A good opportunity is, for example, if we are sending another packet to the same target,
    /// then we can aggregate the requests, to avoid advertising twice. Another example would be if
    /// the target sends us a packet, the modem can aggregate our request to the response of the
    /// request of the target.
    pub to: u8,
    /// Response Execution Delay in Compressed Format, unit is in milliseconds.
    ///
    /// Time given to the target to process the request.
    pub te: u8,
    /// Addressee of the target.
    pub addressee: Addressee,
}

impl Codec for InterfaceConfiguration {
    fn encoded_size(&self) -> usize {
        self.qos.encoded_size() + 2 + self.addressee.encoded_size()
    }
    unsafe fn encode(&self, out: &mut [u8]) -> usize {
        self.qos.encode(out);
        out[1] = self.to;
        out[2] = self.te;
        3 + self.addressee.encode(&mut out[3..])
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
        if out.len() < 3 {
            return Err(ParseFail::MissingBytes(3 - out.len()));
        }
        let ParseValue {
            value: qos,
            size: qos_size,
        } = Qos::decode(out)?;
        let ParseValue {
            value: addressee,
            size: addressee_size,
        } = Addressee::decode(&out[3..]).inc_offset(3)?;
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
fn test_interface_configuration() {
    test_item(
        InterfaceConfiguration {
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

#[derive(Clone, Debug, PartialEq)]
pub struct Status {
    pub missed: bool,
    pub retry: bool,
    /// Encoded on 2 bits (max 3)
    pub id_type: u8,
    _private: (),
}
impl Status {
    pub fn new(new: new::Status) -> Result<Self, new::Error> {
        if new.id_type > 3 {
            return Err(new::Error::IdTypeTooBig);
        }
        Ok(Self {
            missed: new.missed,
            retry: new.retry,
            id_type: new.id_type,
            _private: (),
        })
    }
    fn from_byte(n: u8) -> Self {
        Self {
            missed: n & 0x80 != 0,
            retry: n & 0x40 != 0,
            id_type: (n >> 4 & 0x03),
            _private: (),
        }
    }
    fn to_byte(&self) -> u8 {
        let mut ret = 0;
        ret |= (self.missed as u8) << 7;
        ret |= (self.retry as u8) << 6;
        ret |= self.id_type << 4;
        ret
    }
}

/// Dash7 metadata upon packet reception.
// ALP SPEC: Add link to D7a section (names do not even match)
#[derive(Clone, Debug, PartialEq)]
pub struct InterfaceStatus {
    /// PHY layer channel header
    pub ch_header: u8,
    /// PHY layer channel index
    pub ch_idx: u16,
    /// PHY layer RX level in -dBm
    pub rxlev: u8,
    /// PHY layer link budget in dB
    pub lb: u8,
    /// Signal-to-noise Ratio (in dB)
    pub snr: u8,
    /// D7ASP Status
    pub status: Status,
    /// Value of the D7ATP Dialog ID
    pub token: u8,
    /// Value of the D7ATP Transaction ID
    pub seq: u8,
    // D7A SPEC: What is that?
    /// Time during which the response is expected in Compressed Format
    pub resp_to: u8,
    // TODO Should I digress from the pure ALP description to restructure (addressee + nls_state)
    // into a type protected NLS based structure? Maybe yes.
    /// D7ANP Origin Addressee.
    pub addressee: Addressee,
    /// Security token
    ///
    /// Required if a non NONE NlsMethod is specified in the addressee
    pub nls_state: Option<[u8; 5]>,
    _private: (),
}
impl InterfaceStatus {
    pub fn new(new: new::InterfaceStatus) -> Result<Self, new::Error> {
        match &new.addressee.nls_method {
            NlsMethod::None => (),
            _ => {
                if new.nls_state.is_none() {
                    return Err(new::Error::MissingNlsState);
                }
            }
        }
        Ok(Self {
            ch_header: new.ch_header,
            ch_idx: new.ch_idx,
            rxlev: new.rxlev,
            lb: new.lb,
            snr: new.snr,
            status: new.status,
            token: new.token,
            seq: new.seq,
            resp_to: new.resp_to,
            addressee: new.addressee,
            nls_state: new.nls_state,
            _private: (),
        })
    }
}
impl Codec for InterfaceStatus {
    fn encoded_size(&self) -> usize {
        10 + self.addressee.encoded_size()
            + match self.nls_state {
                Some(_) => 5,
                None => 0,
            }
    }
    unsafe fn encode(&self, out: &mut [u8]) -> usize {
        let mut i = 0;
        out[i] = self.ch_header;
        i += 1;
        out[i..(i + 2)].clone_from_slice(&self.ch_idx.to_be_bytes());
        i += 2;
        out[i] = self.rxlev;
        i += 1;
        out[i] = self.lb;
        i += 1;
        out[i] = self.snr;
        i += 1;
        out[i] = self.status.to_byte();
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
            return Err(ParseFail::MissingBytes(10 - out.len()));
        }
        let ParseValue {
            value: addressee,
            size: addressee_size,
        } = Addressee::decode(&out[10..]).inc_offset(10)?;
        let offset = 10 + addressee_size;
        let nls_state = match addressee.nls_method {
            NlsMethod::None => None,
            _ => {
                if out.len() < offset + 5 {
                    return Err(ParseFail::MissingBytes(offset + 5 - out.len()));
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
                ch_idx: ((out[1] as u16) << 8) + out[2] as u16,
                rxlev: out[3],
                lb: out[4],
                snr: out[5],
                status: Status::from_byte(out[6]),
                token: out[7],
                seq: out[8],
                resp_to: out[9],
                addressee,
                nls_state,
                _private: (),
            },
            size,
        })
    }
}
#[test]
fn test_interface_status() {
    test_item(
        InterfaceStatus {
            ch_header: 1,
            ch_idx: 0x0123,
            rxlev: 2,
            lb: 3,
            snr: 4,
            status: new::Status {
                missed: true,
                retry: false,
                id_type: 3,
            }
            .build()
            .unwrap(),
            token: 6,
            seq: 7,
            resp_to: 8,
            addressee: Addressee {
                nls_method: NlsMethod::AesCcm32,
                access_class: 0xFF,
                address: Address::Vid(Box::new([0xAB, 0xCD])),
            },
            nls_state: Some(hex!("00 11 22 33 44")),
            _private: (),
        },
        &hex!("01 0123 02 03 04 B0 06 07 08   37 FF ABCD  0011223344"),
    )
}

pub mod file {
    pub mod id {
        //! File IDs 0x00-0x17 and 0x20-0x2F are reserved by the DASH7 spec.
        //! File IDs 0x18-0x1F Reserved for D7AALP.
        //! File IDs 0x20+I with I in [0, 14] are reserved for Access Profiles.
        pub const UID: u8 = 0x00;
        pub const FACTORY_SETTINGS: u8 = 0x01;
        pub const FIRMWARE_VERSIOR: u8 = 0x02;
        pub const DEVICE_CAPACITY: u8 = 0x03;
        pub const DEVICE_STATUS: u8 = 0x04;
        pub const ENGINEERING_MODE: u8 = 0x05;
        pub const VID: u8 = 0x06;
        pub const PHY_CONFIGURATION: u8 = 0x08;
        pub const PHY_STATUS: u8 = 0x09;
        pub const DLL_CONFIGURATION: u8 = 0x0A;
        pub const DLL_STATUS: u8 = 0x0B;
        pub const NWL_ROUTING: u8 = 0x0C;
        pub const NWL_SECURITY: u8 = 0x0D;
        pub const NWL_SECURITY_KEY: u8 = 0x0E;
        pub const NWL_SECURITY_STATE_REGISTER: u8 = 0x0F;
        pub const NWL_STATUS: u8 = 0x10;
        pub const TRL_STATUS: u8 = 0x11;
        pub const SEL_CONFIGURATION: u8 = 0x12;
        pub const FOF_STATUS: u8 = 0x13;
        pub const LOCATION_DATA: u8 = 0x17;
        pub const ROOT_KEY: u8 = 0x18;
        pub const USER_KEY: u8 = 0x19;
        pub const SENSOR_DESCRIPTION: u8 = 0x1B;
        pub const RTC: u8 = 0x1C;
    }
    // TODO Write standard file structs
}

pub mod new {
    pub use crate::new::Error;
    pub struct Status {
        pub missed: bool,
        pub retry: bool,
        pub id_type: u8,
    }
    impl Status {
        pub fn build(self) -> Result<super::Status, Error> {
            super::Status::new(self)
        }
    }
    pub struct InterfaceStatus {
        pub ch_header: u8,
        pub ch_idx: u16,
        pub rxlev: u8,
        pub lb: u8,
        pub snr: u8,
        pub status: super::Status,
        pub token: u8,
        pub seq: u8,
        pub resp_to: u8,
        pub addressee: super::Addressee,
        pub nls_state: Option<[u8; 5]>,
    }
    impl InterfaceStatus {
        pub fn build(self) -> Result<super::InterfaceStatus, Error> {
            super::InterfaceStatus::new(self)
        }
    }
}

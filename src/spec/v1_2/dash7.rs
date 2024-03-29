use crate::codec::{Codec, StdError, WithOffset, WithSize};
#[cfg(test)]
use crate::test_tools::test_item;
#[cfg(test)]
use hex_literal::hex;
use std::convert::TryFrom;

/// Encryption algorigthm for over-the-air packets
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
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
    pub(crate) unsafe fn from(n: u8) -> NlsMethod {
        match n {
            0 => NlsMethod::None,
            1 => NlsMethod::AesCtr,
            2 => NlsMethod::AesCbcMac128,
            3 => NlsMethod::AesCbcMac64,
            4 => NlsMethod::AesCbcMac32,
            5 => NlsMethod::AesCcm128,
            6 => NlsMethod::AesCcm64,
            7 => NlsMethod::AesCcm32,
            _ => panic!(),
        }
    }
}

impl std::fmt::Display for NlsMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if *self != NlsMethod::None {
            write!(f, "NLS[{}]", *self as u8)
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum NlsState {
    None,
    AesCtr([u8; 5]),
    AesCbcMac128([u8; 5]),
    AesCbcMac64([u8; 5]),
    AesCbcMac32([u8; 5]),
    AesCcm128([u8; 5]),
    AesCcm64([u8; 5]),
    AesCcm32([u8; 5]),
}

impl NlsState {
    pub(crate) fn build_non_none(method: NlsMethod, state: [u8; 5]) -> Self {
        match method {
            NlsMethod::None => panic!(),
            NlsMethod::AesCtr => Self::AesCtr(state),
            NlsMethod::AesCbcMac128 => Self::AesCbcMac128(state),
            NlsMethod::AesCbcMac64 => Self::AesCbcMac64(state),
            NlsMethod::AesCbcMac32 => Self::AesCbcMac32(state),
            NlsMethod::AesCcm128 => Self::AesCcm128(state),
            NlsMethod::AesCcm64 => Self::AesCcm64(state),
            NlsMethod::AesCcm32 => Self::AesCcm32(state),
        }
    }

    pub fn method(&self) -> NlsMethod {
        match self {
            Self::None => NlsMethod::None,
            Self::AesCtr(_) => NlsMethod::AesCtr,
            Self::AesCbcMac128(_) => NlsMethod::AesCbcMac128,
            Self::AesCbcMac64(_) => NlsMethod::AesCbcMac64,
            Self::AesCbcMac32(_) => NlsMethod::AesCbcMac32,
            Self::AesCcm128(_) => NlsMethod::AesCcm128,
            Self::AesCcm64(_) => NlsMethod::AesCcm64,
            Self::AesCcm32(_) => NlsMethod::AesCcm32,
        }
    }

    pub fn encoded_size(&self) -> usize {
        match self {
            Self::None => 0,
            _ => 5,
        }
    }

    pub fn get_data(&self) -> Option<&[u8; 5]> {
        match self {
            Self::None => None,
            Self::AesCtr(state) => Some(state),
            Self::AesCbcMac128(state) => Some(state),
            Self::AesCbcMac64(state) => Some(state),
            Self::AesCbcMac32(state) => Some(state),
            Self::AesCcm128(state) => Some(state),
            Self::AesCcm64(state) => Some(state),
            Self::AesCcm32(state) => Some(state),
        }
    }
}

impl std::fmt::Display for NlsState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::None => Ok(()),
            Self::AesCtr(state)
            | Self::AesCbcMac128(state)
            | Self::AesCbcMac64(state)
            | Self::AesCbcMac32(state)
            | Self::AesCcm128(state)
            | Self::AesCcm64(state)
            | Self::AesCcm32(state) => {
                write!(
                    f,
                    "NLS[{}|{}]",
                    self.method() as u8,
                    hex::encode_upper(state)
                )
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum AddressType {
    NbId = 0,
    NoId = 1,
    Uid = 2,
    Vid = 3,
}

impl From<u8> for AddressType {
    fn from(n: u8) -> Self {
        match n {
            0 => Self::NbId,
            1 => Self::NoId,
            2 => Self::Uid,
            3 => Self::Vid,
            _ => panic!(),
        }
    }
}

impl std::fmt::Display for AddressType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::NbId => "NID",
                Self::NoId => "ALL",
                Self::Uid => "UID",
                Self::Vid => "VID",
            }
        )
    }
}

/// Dash7 device address
#[derive(Clone, Debug, PartialEq)]
pub enum Address {
    /// Broadcast to an estimated number of receivers, encoded in compressed format on a byte.
    NbId(u8),
    /// Broadcast to everyone
    NoId,
    /// Unicast to target via its UID (Unique Dash7 ID)
    Uid([u8; 8]),
    /// Unicast to target via its VID (Virtual ID)
    Vid([u8; 2]),
}
impl Address {
    pub fn id_type(&self) -> AddressType {
        match self {
            Self::NoId => AddressType::NoId,
            Self::NbId(_) => AddressType::NbId,
            Self::Uid(_) => AddressType::Uid,
            Self::Vid(_) => AddressType::Vid,
        }
    }
}
impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::NbId(n) => write!(f, "NID[{}]", n),
            Self::NoId => write!(f, "ALL"),
            Self::Uid(uid) => write!(f, "UID[{}]", hex::encode_upper(uid)),
            Self::Vid(vid) => write!(f, "VID[{}]", hex::encode_upper(vid)),
        }
    }
}
impl Address {
    pub(crate) fn encoded_size(&self) -> usize {
        match self {
            Address::NbId(_) => 1,
            Address::NoId => 0,
            Address::Uid(_) => 8,
            Address::Vid(_) => 2,
        }
    }

    pub(crate) unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
        match self {
            Self::NoId => 0,
            Self::NbId(id) => {
                out[0] = *id;
                1
            }
            Self::Uid(uid) => {
                out[..8].copy_from_slice(uid);
                8
            }
            Self::Vid(vid) => {
                out[..2].copy_from_slice(vid);
                2
            }
        }
    }

    pub(crate) fn parse(
        ty: AddressType,
        data: &[u8],
    ) -> Result<WithSize<Self>, WithOffset<StdError>> {
        Ok(match ty {
            AddressType::NoId => WithSize {
                size: 0,
                value: Self::NoId,
            },
            AddressType::NbId => WithSize {
                size: 1,
                value: Self::NbId(
                    *data
                        .first()
                        .ok_or_else(|| WithOffset::new_head(StdError::MissingBytes(1)))?,
                ),
            },
            AddressType::Uid => {
                let mut uid = [0u8; 8];
                uid.copy_from_slice(
                    data.get(..8).ok_or_else(|| {
                        WithOffset::new_head(StdError::MissingBytes(data.len() - 8))
                    })?,
                );
                WithSize {
                    size: 8,
                    value: Self::Uid(uid),
                }
            }
            AddressType::Vid => {
                let mut vid = [0u8; 2];
                vid.copy_from_slice(
                    data.get(..2).ok_or_else(|| {
                        WithOffset::new_head(StdError::MissingBytes(data.len() - 2))
                    })?,
                );
                WithSize {
                    size: 2,
                    value: Self::Vid(vid),
                }
            }
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
// ALP_SPEC: Aren't there supposed to be more retry modes?
/// The Retry Modes define the pattern for re-flushing a FIFO that terminates on error.
///
/// In other words, what is the retry policy when sending your payload.
pub enum RetryMode {
    No = 0,
    Rfu1 = 1,
    Rfu2 = 2,
    Rfu3 = 3,
    Rfu4 = 4,
    Rfu5 = 5,
    Rfu6 = 6,
    Rfu7 = 7,
}
impl RetryMode {
    fn from(n: u8) -> Result<Self, u8> {
        Ok(match n {
            0 => RetryMode::No,
            1 => RetryMode::Rfu1,
            2 => RetryMode::Rfu2,
            3 => RetryMode::Rfu3,
            4 => RetryMode::Rfu4,
            5 => RetryMode::Rfu5,
            6 => RetryMode::Rfu6,
            7 => RetryMode::Rfu7,
            x => return Err(x),
        })
    }
}
impl std::fmt::Display for RetryMode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", *self as u8)
    }
}

/// The Response Modes define the condition for termination on success of a Request
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RespMode {
    /// A Request is acknowledged if the DLL CSMA-CA routine succeeds. No
    /// responses are expected.
    ///
    /// Eg. The request is successful if your packet was successfully sent on the radio.
    /// (N)
    No = 0,
    /// If the addressee is broadcast, a Request is acknowledged if as many as
    /// possible D7ATP responses to this Request are received (may be zero).
    ///
    /// If the addressee is unicast, a Request is acknowledged if the addressee provides a
    /// D7ATP response. All responses received during the D7ATP Receive Period
    /// are vectored to upper layer.
    ///
    /// Eg. Succeeds if everyone addressed responds to the radio packet.
    /// (A)
    All = 1,
    /// A Request is acknowledged if at least one D7ATP response to this Request is
    /// received.
    ///
    /// Eg. Succeeds if you receive one response to the radio packet.
    /// (X)
    Any = 2,
    /// A Request is acknowledged if the DLL CSMA-CA routine succeeds REPEAT
    /// times. No responses are expected. The parameters REPEAT is defined in the
    /// SEL configuration file.
    /// (R)
    RespNoRpt = 4,
    /// A Request is acknowledged if the DLL CSMA-CA routine succeeds. It is un-
    /// acknowledged when a response does not acknowledge the Request. The
    /// procedure behaves as RESP_ALL, but Responders provide responses only
    /// when their D7ATP ACK Templates is not void or if the upper layer provides a
    /// response.
    ///
    /// Eg. Succeeds only if the responder gives back an ALP packet in response (which is more
    /// restrictive than succeeding upon successful radio ACK).
    /// (D)
    RespOnData = 5,
    /// A Request is acknowledged if at least one D7ATP response to this Request is
    /// received. The procedure behaves as RESP_ANY, but the Addressee is
    /// managed dynamically. It is set to broadcast after failure to receive an
    /// acknowledgement. On acknowledgement success, it is set to the
    /// Addressee of one of the responders that acknowledged the Request (preferred
    /// addressee). The preferred addressee selection is implementation dependent.
    /// (P)
    RespPreferred = 6,
}
impl RespMode {
    pub(crate) fn from(n: u8) -> Result<Self, u8> {
        Ok(match n {
            0 => RespMode::No,
            1 => RespMode::All,
            2 => RespMode::Any,
            4 => RespMode::RespNoRpt,
            5 => RespMode::RespOnData,
            6 => RespMode::RespPreferred,
            x => return Err(x),
        })
    }
}
impl std::fmt::Display for RespMode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::No => "N",
                Self::All => "A",
                Self::Any => "X",
                Self::RespNoRpt => "R",
                Self::RespOnData => "D",
                Self::RespPreferred => "P",
            }
        )
    }
}

/// Qos of the request
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Qos {
    pub retry: RetryMode,
    pub resp: RespMode,
}
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum QosDecodingError {
    MissingBytes(u8),
    UnknownRetryMode(u8),
    UnknownRespMode(u8),
}
impl Codec for Qos {
    type Error = QosDecodingError;
    fn encoded_size(&self) -> usize {
        1
    }
    unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
        out[0] = ((self.retry as u8) << 3) + self.resp as u8;
        1
    }
    fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
        if out.is_empty() {
            return Err(WithOffset::new_head(Self::Error::MissingBytes(1)));
        }
        let retry = RetryMode::from((out[0] & 0x38) >> 3)
            .map_err(|e| WithOffset::new_head(Self::Error::UnknownRetryMode(e)))?;
        let resp = RespMode::from(out[0] & 0x07)
            .map_err(|e| WithOffset::new_head(Self::Error::UnknownRespMode(e)))?;
        Ok(WithSize {
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
impl std::fmt::Display for Qos {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}{}", self.retry, self.resp)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum GroupCondition {
    /// <, =, > (always true)
    Any = 0,
    /// <, >
    NotEqual = 1,
    /// =
    Equal = 2,
    /// >
    GreaterThan = 3,
}

impl TryFrom<u8> for GroupCondition {
    type Error = u8;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(GroupCondition::Any),
            1 => Ok(GroupCondition::NotEqual),
            2 => Ok(GroupCondition::Equal),
            3 => Ok(GroupCondition::GreaterThan),
            x => Err(x),
        }
    }
}

impl std::fmt::Display for GroupCondition {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "GCD={}",
            match self {
                Self::Any => "X",
                Self::NotEqual => "!=",
                Self::Equal => "==",
                Self::GreaterThan => ">",
            }
        )
    }
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
    /// Access class of the targeted listening device
    pub access_class: u8,
    /// Security method
    pub nls_method: NlsMethod,
    /// Address of the target.
    pub address: Address,

    /// Use VID instead of UID when possible
    pub use_vid: bool,

    /// Group condition
    pub group_condition: GroupCondition,
}

impl std::fmt::Display for InterfaceConfiguration {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{},{},{}|0x{},use_vid={},{},{},{}",
            self.qos,
            self.to,
            self.te,
            hex::encode_upper([self.access_class]),
            self.use_vid,
            self.nls_method,
            self.group_condition,
            self.address
        )
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum InterfaceConfigurationDecodingError {
    MissingBytes(usize),
    Qos(QosDecodingError),
}

impl From<StdError> for InterfaceConfigurationDecodingError {
    fn from(e: StdError) -> Self {
        match e {
            StdError::MissingBytes(n) => Self::MissingBytes(n),
        }
    }
}

impl Codec for InterfaceConfiguration {
    type Error = InterfaceConfigurationDecodingError;
    fn encoded_size(&self) -> usize {
        self.qos.encoded_size() + 4 + self.address.encoded_size()
    }
    unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
        self.qos.encode_in(out);
        out[1] = self.to;
        out[2] = self.te;
        out[3] = ((self.group_condition as u8) << 6)
            | ((self.address.id_type() as u8) << 4)
            | ((self.use_vid as u8) << 3)
            | (self.nls_method as u8);
        out[4] = self.access_class;
        5 + self.address.encode_in(&mut out[5..])
    }
    fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
        if out.len() < 5 {
            return Err(WithOffset::new_head(Self::Error::MissingBytes(
                5 - out.len(),
            )));
        }
        let WithSize {
            value: qos,
            size: qos_size,
        } = Qos::decode(out).map_err(|e| e.map_value(Self::Error::Qos))?;
        let to = out[1];
        let te = out[2];
        let group_condition = GroupCondition::try_from((out[3] >> 6) & 0x03).unwrap();
        let address_type = AddressType::from((out[3] & 0x30) >> 4);
        let use_vid = (out[3] & 0x08) != 0;
        let nls_method = unsafe { NlsMethod::from(out[3] & 0x07) };
        let access_class = out[4];
        let WithSize {
            value: address,
            size: address_size,
        } = Address::parse(address_type, &out[5..]).map_err(|e| {
            let WithOffset { offset, value } = e;
            WithOffset {
                offset: offset + 5,
                value: value.into(),
            }
        })?;
        Ok(WithSize {
            value: Self {
                qos,
                to,
                te,
                access_class,
                nls_method,
                address,
                use_vid,
                group_condition,
            },
            size: qos_size + 4 + address_size,
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
            nls_method: NlsMethod::AesCcm32,
            access_class: 0xFF,
            use_vid: false,
            address: Address::Vid([0xAB, 0xCD]),
            group_condition: GroupCondition::Any,
        },
        &hex!("02 23 34   37 FF ABCD"),
    )
}

#[test]
fn test_interface_configuration_with_address_nbid() {
    test_item(
        InterfaceConfiguration {
            qos: Qos {
                retry: RetryMode::No,
                resp: RespMode::Any,
            },
            to: 0x23,
            te: 0x34,
            nls_method: NlsMethod::None,
            access_class: 0x00,
            address: Address::NbId(0x15),
            use_vid: true,
            group_condition: GroupCondition::NotEqual,
        },
        &hex!("02 23 34   48 00 15"),
    )
}
#[test]
fn test_interface_configuration_with_address_noid() {
    test_item(
        InterfaceConfiguration {
            qos: Qos {
                retry: RetryMode::No,
                resp: RespMode::Any,
            },
            to: 0x23,
            te: 0x34,
            nls_method: NlsMethod::AesCbcMac128,
            access_class: 0x24,
            address: Address::NoId,
            use_vid: false,
            group_condition: GroupCondition::Equal,
        },
        &hex!("02 23 34   92 24"),
    )
}
#[test]
fn test_interface_configuration_with_address_uid() {
    test_item(
        InterfaceConfiguration {
            qos: Qos {
                retry: RetryMode::No,
                resp: RespMode::Any,
            },
            to: 0x23,
            te: 0x34,
            nls_method: NlsMethod::AesCcm64,
            access_class: 0x48,
            address: Address::Uid([0, 1, 2, 3, 4, 5, 6, 7]),
            use_vid: true,
            group_condition: GroupCondition::GreaterThan,
        },
        &hex!("02 23 34   EE 48 0001020304050607"),
    )
}
#[test]
fn test_interface_configuration_with_address_vid() {
    test_item(
        InterfaceConfiguration {
            qos: Qos {
                retry: RetryMode::No,
                resp: RespMode::Any,
            },
            to: 0x23,
            te: 0x34,
            nls_method: NlsMethod::AesCcm32,
            access_class: 0xFF,
            address: Address::Vid([0xAB, 0xCD]),
            use_vid: false,
            group_condition: GroupCondition::Any,
        },
        &hex!("02 23 34   37 FF AB CD"),
    )
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
    pub status: u8,
    /// Value of the D7ATP Dialog ID
    pub token: u8,
    /// Value of the D7ATP Transaction ID
    pub seq: u8,
    /// Response delay (request to response time) in TiT
    pub resp_to: u16,
    /// Frequency offset in Hz
    pub fof: u16,
    /// Listening access class of the sender
    pub access_class: u8,
    /// Address of source
    pub address: Address,
    /// Security data
    pub nls_state: NlsState,
}
impl std::fmt::Display for InterfaceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "ch({};{}),sig({},{},{}),s={},tok={},sq={},rto={},fof={},xclass=0x{},{},{}",
            self.ch_header,
            self.ch_idx,
            self.rxlev,
            self.lb,
            self.snr,
            self.status,
            self.token,
            self.seq,
            self.resp_to,
            self.fof,
            hex::encode_upper([self.access_class]),
            self.address,
            self.nls_state
        )
    }
}
impl Codec for InterfaceStatus {
    type Error = StdError;
    fn encoded_size(&self) -> usize {
        15 + self.address.encoded_size() + self.nls_state.encoded_size()
    }

    unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
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
        out[i] = self.status;
        i += 1;
        out[i] = self.token;
        i += 1;
        out[i] = self.seq;
        i += 1;
        out[i] = (self.resp_to & 0xFF) as u8;
        i += 1;
        out[i] = (self.resp_to >> 8) as u8;
        i += 1;
        out[i] = (self.fof & 0xFF) as u8;
        i += 1;
        out[i] = (self.fof >> 8) as u8;
        i += 1;
        out[i] = ((self.address.id_type() as u8) << 4) | (self.nls_state.method() as u8);
        i += 1;
        out[i] = self.access_class;
        i += 1;
        i += self.address.encode_in(&mut out[i..]);
        if let Some(data) = self.nls_state.get_data() {
            out[i..i + 5].clone_from_slice(&data[..]);
            i += 5;
        }
        i
    }
    fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
        if out.len() < 10 {
            return Err(WithOffset::new_head(Self::Error::MissingBytes(
                10 - out.len(),
            )));
        }
        let ch_header = out[0];
        let ch_idx = ((out[1] as u16) << 8) + out[2] as u16;
        let rxlev = out[3];
        let lb = out[4];
        let snr = out[5];
        let status = out[6];
        let token = out[7];
        let seq = out[8];
        let resp_to = ((out[10] as u16) << 8) + out[9] as u16;
        let fof = ((out[12] as u16) << 8) + out[11] as u16;

        let address_type = AddressType::from((out[13] & 0x30) >> 4);
        let nls_method = unsafe { NlsMethod::from(out[13] & 0x07) };
        let access_class = out[14];

        let WithSize {
            size: address_size,
            value: address,
        } = Address::parse(address_type, &out[15..]).map_err(|e| e.shift(15))?;

        let mut offset = 15 + address_size;
        let nls_state = match nls_method {
            NlsMethod::None => NlsState::None,
            method => {
                if out.len() < offset + 5 {
                    return Err(WithOffset::new(
                        offset,
                        Self::Error::MissingBytes(offset + 5 - out.len()),
                    ));
                } else {
                    let mut nls_state = [0u8; 5];
                    nls_state.clone_from_slice(&out[offset..offset + 5]);
                    offset += 5;
                    NlsState::build_non_none(method, nls_state)
                }
            }
        };
        let size = offset;
        Ok(WithSize {
            value: Self {
                ch_header,
                ch_idx,
                rxlev,
                lb,
                snr,
                status,
                token,
                seq,
                resp_to,
                fof,
                access_class,
                address,
                nls_state,
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
            status: 5,
            token: 6,
            seq: 7,
            resp_to: 8,
            fof: 9,
            access_class: 0xFF,
            address: Address::Vid([0xAB, 0xCD]),
            nls_state: NlsState::AesCcm32(hex!("00 11 22 33 44")),
        },
        &hex!("01 0123 02 03 04 05 06 07 0800 0900  37 FF ABCD  0011223344"),
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

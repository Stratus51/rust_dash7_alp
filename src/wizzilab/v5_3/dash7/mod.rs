#[cfg(test)]
use crate::test_tools::test_item;
pub use crate::{
    codec::{Codec, StdError, WithOffset, WithSize},
    spec::v1_2 as spec,
    spec::v1_2::dash7::{
        file, AddressType, GroupCondition, InterfaceConfigurationDecodingError, NlsMethod,
        NlsState, QosDecodingError, RespMode, RetryMode as SpecRetryMode,
    },
};
#[cfg(test)]
use hex_literal::hex;
use std::convert::TryFrom;
pub mod interface_tx_status;
pub mod stack_error;

#[derive(Clone, Copy, Debug, PartialEq)]
/// The Retry Modes define the pattern for re-flushing a FIFO that terminates on error.
///
/// In other words, what is the retry policy when sending your payload.
pub enum RetryMode {
    Oneshot = 0,
    OneshotRetry = 1,
    FifoFast = 2,
    FifoSlow = 3,
    SingleFast = 4,
    SingleSlow = 5,
    OneshotSticky = 6,
    Rfu7 = 7,
}
impl RetryMode {
    fn from(n: u8) -> Result<Self, u8> {
        Ok(match n {
            0 => RetryMode::Oneshot,
            1 => RetryMode::OneshotRetry,
            2 => RetryMode::FifoFast,
            3 => RetryMode::FifoSlow,
            4 => RetryMode::SingleFast,
            5 => RetryMode::SingleSlow,
            6 => RetryMode::OneshotSticky,
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
impl From<RetryMode> for SpecRetryMode {
    fn from(mode: RetryMode) -> Self {
        match mode {
            RetryMode::Oneshot => Self::No,
            RetryMode::OneshotRetry => Self::Rfu1,
            RetryMode::FifoFast => Self::Rfu2,
            RetryMode::FifoSlow => Self::Rfu3,
            RetryMode::SingleFast => Self::Rfu4,
            RetryMode::SingleSlow => Self::Rfu5,
            RetryMode::OneshotSticky => Self::Rfu6,
            RetryMode::Rfu7 => Self::Rfu7,
        }
    }
}
impl From<SpecRetryMode> for RetryMode {
    fn from(mode: SpecRetryMode) -> Self {
        match mode {
            SpecRetryMode::No => Self::Oneshot,
            SpecRetryMode::Rfu1 => Self::OneshotRetry,
            SpecRetryMode::Rfu2 => Self::FifoFast,
            SpecRetryMode::Rfu3 => Self::FifoSlow,
            SpecRetryMode::Rfu4 => Self::SingleFast,
            SpecRetryMode::Rfu5 => Self::SingleSlow,
            SpecRetryMode::Rfu6 => Self::OneshotSticky,
            SpecRetryMode::Rfu7 => Self::Rfu7,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Qos {
    pub retry: RetryMode,
    pub resp: RespMode,
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
            retry: RetryMode::Oneshot,
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

impl From<spec::dash7::Qos> for Qos {
    fn from(o: spec::dash7::Qos) -> Self {
        let spec::dash7::Qos { retry, resp } = o;
        Self {
            retry: retry.into(),
            resp,
        }
    }
}
impl From<Qos> for spec::dash7::Qos {
    fn from(o: Qos) -> Self {
        let Qos { retry, resp } = o;
        Self {
            retry: retry.into(),
            resp,
        }
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
impl From<spec::dash7::Address> for Address {
    fn from(o: spec::dash7::Address) -> Self {
        match o {
            spec::dash7::Address::NbId(n) => Self::NbId(n),
            spec::dash7::Address::NoId => Self::NoId,
            spec::dash7::Address::Uid(uid) => Self::Uid(uid),
            spec::dash7::Address::Vid(vid) => Self::Vid(vid),
        }
    }
}
impl From<Address> for spec::dash7::Address {
    fn from(o: Address) -> Self {
        match o {
            Address::NbId(n) => Self::NbId(n),
            Address::NoId => Self::NoId,
            Address::Uid(uid) => Self::Uid(uid),
            Address::Vid(vid) => Self::Vid(vid),
        }
    }
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
            Address::Vid(_) => 8,
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
                out[2..8].copy_from_slice(&[0, 0, 0, 0, 0, 0]);
                8
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
                    size: 8,
                    value: Self::Vid(vid),
                }
            }
        })
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
                retry: RetryMode::Oneshot,
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
        &hex!("02 23 34   37 FF ABCD 000000000000"),
    )
}

#[test]
fn test_interface_configuration_with_address_nbid() {
    test_item(
        InterfaceConfiguration {
            qos: Qos {
                retry: RetryMode::Oneshot,
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
                retry: RetryMode::Oneshot,
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
                retry: RetryMode::Oneshot,
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
                retry: RetryMode::Oneshot,
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
        &hex!("02 23 34   37 FF AB CD 000000000000"),
    )
}
impl From<spec::dash7::InterfaceConfiguration> for InterfaceConfiguration {
    fn from(o: spec::dash7::InterfaceConfiguration) -> Self {
        let spec::dash7::InterfaceConfiguration {
            qos,
            to,
            te,
            nls_method,
            access_class,
            address,
            use_vid,
            group_condition,
        } = o;
        Self {
            qos: qos.into(),
            to,
            te,
            nls_method,
            access_class,
            address: address.into(),
            use_vid,
            group_condition,
        }
    }
}
impl From<InterfaceConfiguration> for spec::dash7::InterfaceConfiguration {
    fn from(o: InterfaceConfiguration) -> Self {
        let InterfaceConfiguration {
            qos,
            to,
            te,
            nls_method,
            access_class,
            address,
            use_vid,
            group_condition,
        } = o;
        Self {
            qos: qos.into(),
            to,
            te,
            nls_method,
            access_class,
            address: address.into(),
            use_vid,
            group_condition,
        }
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
            "ch({};{}),sig({},{},{}),s={},tok={},sq={},rto={},fof={},xcl=0x{},{},{}",
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
        &hex!("01 0123 02 03 04 05 06 07 0800 0900  37 FF ABCD 000000000000  0011223344"),
    )
}

impl From<spec::dash7::InterfaceStatus> for InterfaceStatus {
    fn from(status: spec::dash7::InterfaceStatus) -> Self {
        Self {
            ch_header: status.ch_header,
            ch_idx: status.ch_idx,
            rxlev: status.rxlev,
            lb: status.lb,
            snr: status.snr,
            status: status.status,
            token: status.token,
            seq: status.seq,
            resp_to: status.resp_to,
            fof: status.fof,
            access_class: status.access_class,
            address: status.address.into(),
            nls_state: status.nls_state,
        }
    }
}

impl From<InterfaceStatus> for spec::dash7::InterfaceStatus {
    fn from(status: InterfaceStatus) -> Self {
        Self {
            ch_header: status.ch_header,
            ch_idx: status.ch_idx,
            rxlev: status.rxlev,
            lb: status.lb,
            snr: status.snr,
            status: status.status,
            token: status.token,
            seq: status.seq,
            resp_to: status.resp_to,
            fof: status.fof,
            access_class: status.access_class,
            address: status.address.into(),
            nls_state: status.nls_state,
        }
    }
}

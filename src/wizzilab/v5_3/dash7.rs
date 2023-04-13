#[cfg(test)]
use crate::test_tools::test_item;
pub use crate::{
    codec::{Codec, StdError, WithOffset, WithSize},
    spec::v1_2 as spec,
    spec::v1_2::dash7::{
        file, Address, AddressType, GroupCondition, InterfaceConfigurationDecodingError,
        InterfaceStatus, NlsMethod, NlsState, QosDecodingError, RespMode,
        RetryMode as SpecRetryMode,
    },
};
#[cfg(test)]
use hex_literal::hex;
use std::convert::TryFrom;

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
        &hex!("02 23 34   37 FF ABCD"),
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
        &hex!("02 23 34   37 FF AB CD"),
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
            address,
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
            address,
            use_vid,
            group_condition,
        }
    }
}

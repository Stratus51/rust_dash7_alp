use crate::codec::{Codec, WithOffset, WithSize};
pub use crate::spec::v1_2::dash7::{
    Address, AddressType, InterfaceConfigurationDecodingError, InterfaceStatus, NlsMethod,
    NlsState, Qos, QosDecodingError, RespMode, RetryMode,
};
#[cfg(test)]
use crate::test_tools::test_item;
#[cfg(test)]
use hex_literal::hex;

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
    /// Access class of the targeted listening device
    pub access_class: u8,
    /// Security method
    pub nls_method: NlsMethod,
    /// Address of the target.
    pub address: Address,
}

impl std::fmt::Display for InterfaceConfiguration {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{},{}|0x{},{},{}",
            self.qos,
            self.to,
            hex::encode_upper([self.access_class]),
            self.nls_method,
            self.address
        )
    }
}

impl Codec for InterfaceConfiguration {
    type Error = InterfaceConfigurationDecodingError;
    fn encoded_size(&self) -> usize {
        self.qos.encoded_size() + 3 + self.address.encoded_size()
    }
    unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
        self.qos.encode_in(out);
        out[1] = self.to;
        out[2] = ((self.address.id_type() as u8) << 4) | (self.nls_method as u8);
        out[3] = self.access_class;
        4 + self.address.encode_in(&mut out[4..])
    }
    fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
        if out.len() < 4 {
            return Err(WithOffset::new_head(Self::Error::MissingBytes(
                4 - out.len(),
            )));
        }
        let WithSize {
            value: qos,
            size: qos_size,
        } = Qos::decode(out).map_err(|e| e.map_value(Self::Error::Qos))?;
        let to = out[1];
        let address_type = AddressType::from((out[2] & 0x30) >> 4);
        let nls_method = unsafe { NlsMethod::from(out[2] & 0x0F) };
        let access_class = out[3];
        let WithSize {
            value: address,
            size: address_size,
        } = Address::parse(address_type, &out[4..]).map_err(|e| {
            let WithOffset { offset, value } = e;
            WithOffset {
                offset: offset + 4,
                value: value.into(),
            }
        })?;
        Ok(WithSize {
            value: Self {
                qos,
                to,
                access_class,
                nls_method,
                address,
            },
            size: qos_size + 3 + address_size,
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
            nls_method: NlsMethod::AesCcm32,
            access_class: 0xFF,
            address: Address::Vid([0xAB, 0xCD]),
        },
        &hex!("02 23   37 FF ABCD"),
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
            nls_method: NlsMethod::None,
            access_class: 0x00,
            address: Address::NbId(0x15),
        },
        &hex!("02 23   00 00 15"),
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
            nls_method: NlsMethod::AesCbcMac128,
            access_class: 0x24,
            address: Address::NoId,
        },
        &hex!("02 23   12 24"),
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
            nls_method: NlsMethod::AesCcm64,
            access_class: 0x48,
            address: Address::Uid([0, 1, 2, 3, 4, 5, 6, 7]),
        },
        &hex!("02 23   26 48 0001020304050607"),
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
            nls_method: NlsMethod::AesCcm32,
            access_class: 0xFF,
            address: Address::Vid([0xAB, 0xCD]),
        },
        &hex!("02 23   37 FF AB CD"),
    )
}

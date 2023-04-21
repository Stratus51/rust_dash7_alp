#[cfg(test)]
use crate::test_tools::test_item;
use crate::{
    codec::{Codec, WithOffset, WithSize},
    spec::v1_2 as spec,
};
#[cfg(test)]
use hex_literal::hex;
use std::convert::TryInto;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InterfaceFinalStatusCode {
    /// No error
    No = 0,
    /// Resource busy
    Busy = 0xFF,
    /// bad parameter
    BadParam = 0xFE,
    /// duty cycle limit overflow
    DutyCycle = 0xFD,
    /// cca timeout
    CcaTo = 0xFC,
    /// security frame counter overflow
    NlsKey = 0xFB,
    /// tx stream underflow
    TxUdf = 0xFA,
    /// rx stream overflow
    RxOvf = 0xF9,
    /// rx checksum
    RxCrc = 0xF8,
    /// abort
    Abort = 0xF7,
    /// no ack received
    NoAck = 0xF6,
    /// rx timeout
    RxTo = 0xF5,
    /// not supported band
    NotSupportedBand = 0xF4,
    /// not supported channel
    NotSupportedChannel = 0xF3,
    /// not supported modulation
    NotSupportedModulation = 0xF2,
    /// no channels in list
    VoidChannelList = 0xF1,
    /// not supported packet length
    NotSupportedLen = 0xF0,
    /// parameter overflow
    ParamOvf = 0xEF,
    /// vid used without nls
    VidWoNls = 0xEE,
    /// tx scheduling late
    TxSched = 0xED,
    /// rx scheduling late
    RxSched = 0xEC,
    /// parameter overflow
    BufferOvf = 0xEB,
    /// mode not supported
    NotSupportedMode = 0xEA,
}
impl std::convert::TryFrom<u8> for InterfaceFinalStatusCode {
    type Error = u8;
    fn try_from(n: u8) -> Result<Self, Self::Error> {
        Ok(match n {
            0 => Self::No,
            0xFF => Self::Busy,
            0xFE => Self::BadParam,
            0xFD => Self::DutyCycle,
            0xFC => Self::CcaTo,
            0xFB => Self::NlsKey,
            0xFA => Self::TxUdf,
            0xF9 => Self::RxOvf,
            0xF8 => Self::RxCrc,
            0xF7 => Self::Abort,
            0xF6 => Self::NoAck,
            0xF5 => Self::RxTo,
            0xF4 => Self::NotSupportedBand,
            0xF3 => Self::NotSupportedChannel,
            0xF2 => Self::NotSupportedModulation,
            0xF1 => Self::VoidChannelList,
            0xF0 => Self::NotSupportedLen,
            0xEF => Self::ParamOvf,
            0xEE => Self::VidWoNls,
            0xED => Self::TxSched,
            0xEC => Self::RxSched,
            0xEB => Self::BufferOvf,
            0xEA => Self::NotSupportedMode,
            x => return Err(x),
        })
    }
}
impl std::fmt::Display for InterfaceFinalStatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::No => "NO",
                Self::Busy => "BUSY",
                Self::BadParam => "BAD_PRM",
                Self::DutyCycle => "DUTY_C",
                Self::CcaTo => "CCA_TO",
                Self::NlsKey => "NLS_KEY",
                Self::TxUdf => "TX_UDF",
                Self::RxOvf => "RX_OVF",
                Self::RxCrc => "RX_CRC",
                Self::Abort => "ABORT",
                Self::NoAck => "NO_ACK",
                Self::RxTo => "RX_TO",
                Self::NotSupportedBand => "UNS_BAND",
                Self::NotSupportedChannel => "UNS_CH",
                Self::NotSupportedModulation => "UNS_MOD",
                Self::VoidChannelList => "VOID_CHL",
                Self::NotSupportedLen => "UNS_LEN",
                Self::ParamOvf => "PRM_OVF",
                Self::VidWoNls => "VID_WO_NLS",
                Self::TxSched => "TX_SCHD",
                Self::RxSched => "RX_SCHD",
                Self::BufferOvf => "BUF_OVF",
                Self::NotSupportedMode => "UNS_MODE",
            }
        )
    }
}

/// Result of an action in a previously sent request
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InterfaceFinalStatus {
    /// Index of the ALP action associated with this status, in the original request as seen from
    /// the receiver side.
    // ALP_SPEC This is complicated to process because we have to known/possibly infer the position
    // of the action on the receiver side, and that we have to do that while also interpreting who
    // responded (the local modem won't have the same index as the distant device.).
    pub interface: spec::operand::InterfaceId,
    /// Length
    // TODO What is the encoding of this field? Is is a varint?
    pub len: u8,
    /// Result code
    pub status: InterfaceFinalStatusCode,
}
impl std::fmt::Display for InterfaceFinalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "f_itf[{}][{}]=>{}",
            self.interface, self.len, self.status
        )
    }
}
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum InterfaceFinalStatusDecodingError {
    MissingBytes(usize),
    UnknownStatusCode(u8),
    UnknownInterface(u8),
}
impl Codec for InterfaceFinalStatus {
    type Error = InterfaceFinalStatusDecodingError;
    fn encoded_size(&self) -> usize {
        1 + 1 + 1
    }
    unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
        out[0] = self.interface as u8;
        out[1] = self.len;
        out[2] = self.status as u8;
        3
    }
    fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
        if out.len() < 3 {
            return Err(WithOffset::new_head(Self::Error::MissingBytes(
                3 - out.len(),
            )));
        }
        Ok(WithSize {
            value: Self {
                interface: out[0]
                    .try_into()
                    .map_err(|e| WithOffset::new(0, Self::Error::UnknownInterface(e)))?,
                len: out[1],
                status: out[2]
                    .try_into()
                    .map_err(|e| WithOffset::new(2, Self::Error::UnknownStatusCode(e)))?,
            },
            size: 3,
        })
    }
}
#[test]
fn test_interface_final_status_operand() {
    test_item(
        InterfaceFinalStatus {
            interface: spec::operand::InterfaceId::Host,
            len: 2,
            status: InterfaceFinalStatusCode::Busy,
        },
        &hex!("00 02 FF"),
    )
}

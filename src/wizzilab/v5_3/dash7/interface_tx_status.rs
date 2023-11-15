#[cfg(test)]
use crate::test_tools::test_item;
use crate::{
    codec::{Codec, StdError, WithOffset, WithSize},
    wizzilab::v5_3::dash7::{
        stack_error::InterfaceFinalStatusCode, Address, AddressType, NlsMethod,
    },
};
#[cfg(test)]
use hex_literal::hex;
use std::convert::TryFrom;

/// Dash7 metadata upon packet transmission.
#[derive(Clone, Debug, PartialEq)]
pub struct InterfaceTxStatus {
    /// PHY layer channel header
    pub ch_header: u8,
    /// PHY layer channel index
    pub ch_idx: u16,
    /// Target power in dBm
    pub eirp: i8,
    /// D7A Error
    pub err: InterfaceFinalStatusCode,
    /// RFU
    /// XXX align to u32
    pub rfu_0: u8,
    pub rfu_1: u8,
    pub rfu_2: u8,
    /// End transmission date using the local RTC time stamp
    pub lts: u32,
    /// Access class
    pub access_class: u8,
    /// NLS method
    pub nls_method: NlsMethod,
    /// Addressee
    pub address: Address,
}
impl std::fmt::Display for InterfaceTxStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "ch({};{}),eirp={},err={},lts={},address={}",
            self.ch_header, self.ch_idx, self.eirp, self.err, self.lts, self.address
        )
    }
}
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum InterfaceTxStatusDecodingError {
    MissingBytes(usize),
    UnknownStatusCode(u8),
}
impl From<StdError> for InterfaceTxStatusDecodingError {
    fn from(e: StdError) -> Self {
        match e {
            StdError::MissingBytes(n) => Self::MissingBytes(n),
        }
    }
}
impl Codec for InterfaceTxStatus {
    type Error = InterfaceTxStatusDecodingError;
    fn encoded_size(&self) -> usize {
        1 + 2 + 1 + 1 + 1 + 1 + 1 + 4 + 1 + 1 + self.address.encoded_size()
    }

    unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
        let mut i = 0;
        out[i] = self.ch_header;
        i += 1;
        out[i..(i + 2)].clone_from_slice(&self.ch_idx.to_be_bytes());
        i += 2;
        out[i] = self.eirp as u8;
        i += 1;
        out[i] = self.err as u8;
        i += 1;
        out[i] = self.rfu_0;
        i += 1;
        out[i] = self.rfu_1;
        i += 1;
        out[i] = self.rfu_2;
        i += 1;
        out[i..(i + 4)].clone_from_slice(&self.lts.to_le_bytes());
        i += 4;
        out[i] = ((self.address.id_type() as u8) << 4) | (self.nls_method as u8);
        i += 1;
        out[i] = self.access_class;
        i += 1;
        i += self.address.encode_in(&mut out[i..]);
        i
    }
    fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
        if out.len() < 15 {
            return Err(WithOffset::new_head(Self::Error::MissingBytes(
                15 - out.len(),
            )));
        }

        let ch_header = out[0];
        let ch_idx = ((out[1] as u16) << 8) + out[2] as u16;
        let eirp = out[3] as i8;
        let err = InterfaceFinalStatusCode::try_from(out[4])
            .map_err(|e| WithOffset::new(4, Self::Error::UnknownStatusCode(e)))?;
        let rfu_0 = out[5];
        let rfu_1 = out[6];
        let rfu_2 = out[7];
        let lts = u32::from_le_bytes([out[8], out[9], out[10], out[11]]);
        let address_type = AddressType::from((out[12] & 0x30) >> 4);
        let nls_method = unsafe { NlsMethod::from(out[12] & 0x07) };
        let access_class = out[13];
        let WithSize {
            size: address_size,
            value: address,
        } = Address::parse(address_type, &out[14..]).map_err(|e| {
            let WithOffset { offset, value } = e;
            WithOffset {
                offset: offset + 5,
                value: value.into(),
            }
        })?;
        let size = 14 + address_size;
        Ok(WithSize {
            value: Self {
                ch_header,
                ch_idx,
                eirp,
                err,
                rfu_0,
                rfu_1,
                rfu_2,
                lts,
                access_class,
                nls_method,
                address,
            },
            size,
        })
    }
}
#[test]
fn test_interface_tx_status() {
    test_item(
        InterfaceTxStatus {
            ch_header: 1,
            ch_idx: 0x0123,
            eirp: 2,
            err: InterfaceFinalStatusCode::Busy,
            rfu_0: 4,
            rfu_1: 5,
            rfu_2: 6,
            lts: 0x0708_0000,
            access_class: 0xFF,
            nls_method: NlsMethod::AesCcm64,
            address: Address::Vid([0x00, 0x11]),
        },
        &hex!("01 0123 02 FF 04 05 06 0000 0807 36 FF 0011 000000000000"),
    )
}

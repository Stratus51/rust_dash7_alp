use crate::{
    codec::{Codec, WithOffset, WithSize},
    wizzilab::v5_3::operand,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TxStatusType {
    Interface = 1,
}
impl TxStatusType {
    fn from(n: u8) -> Result<Self, u8> {
        Ok(match n {
            1 => TxStatusType::Interface,
            x => return Err(x),
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TxStatus {
    Interface(operand::InterfaceTxStatus),
}
impl std::fmt::Display for TxStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Interface(v) => write!(f, "[ITF]:{}", v),
        }
    }
}
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TxStatusDecodingError {
    MissingBytes(usize),
    UnknownType(u8),
    Interface(operand::InterfaceTxTxStatusDecodingError),
}
impl Codec for TxStatus {
    type Error = TxStatusDecodingError;
    fn encoded_size(&self) -> usize {
        1 + match self {
            TxStatus::Interface(op) => op.encoded_size(),
        }
    }
    unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
        out[0] |= (match self {
            TxStatus::Interface(_) => TxStatusType::Interface,
        } as u8)
            << 6;
        let out = &mut out[1..];
        1 + match self {
            TxStatus::Interface(op) => op.encode_in(out),
        }
    }
    fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
        if out.is_empty() {
            return Err(WithOffset::new_head(Self::Error::MissingBytes(1)));
        }
        let status_type = out[0] >> 6;
        Ok(
            match TxStatusType::from(status_type)
                .map_err(|e| WithOffset::new_head(Self::Error::UnknownType(e)))?
            {
                TxStatusType::Interface => {
                    let WithSize { size, value } = operand::InterfaceTxStatus::decode(&out[1..])
                        .map_err(|e| e.shift(1).map_value(Self::Error::Interface))?;
                    WithSize {
                        size: size + 1,
                        value: Self::Interface(value),
                    }
                }
            },
        )
    }
}

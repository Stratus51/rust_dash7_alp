use crate::codec::{Codec, StdError, WithOffset, WithSize};

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum FlowType {
    U16 = 0,
    U32 = 1,
}

impl From<u8> for FlowType {
    fn from(n: u8) -> Self {
        match n {
            0 => FlowType::U16,
            1 => FlowType::U32,
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FlowSeqnum {
    U16(u16),
    U32(u32),
}

impl std::fmt::Display for FlowSeqnum {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::U16(v) => write!(f, "{}", v),
            Self::U32(v) => write!(f, "U32[{}]", v),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Flow {
    pub flow: u8,
    pub seqnum: FlowSeqnum,
}

impl std::fmt::Display for Flow {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Flow[{}]:{}", self.flow, self.seqnum)
    }
}

impl Codec for Flow {
    type Error = StdError;
    fn encoded_size(&self) -> usize {
        1 + 1
            + match self.seqnum {
                FlowSeqnum::U16(_) => 2,
                FlowSeqnum::U32(_) => 4,
            }
    }
    unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
        out[0] |= (match self.seqnum {
            FlowSeqnum::U16(_) => FlowType::U16,
            FlowSeqnum::U32(_) => FlowType::U32,
        } as u8)
            << 7;
        out[1] = self.flow;

        let out = &mut out[2..];
        match self.seqnum {
            FlowSeqnum::U16(v) => {
                out[0..2].copy_from_slice(&v.to_be_bytes());
                2
            }
            FlowSeqnum::U32(v) => {
                out[0..4].copy_from_slice(&v.to_be_bytes());
                4
            }
        }
    }

    fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
        if out.is_empty() {
            return Err(WithOffset::new_head(StdError::MissingBytes(1)));
        }
        let flow_type = FlowType::from(out[0] >> 7);
        let flow = out[1];
        let out = &out[2..];

        Ok(match flow_type {
            FlowType::U16 => {
                let mut bytes = [0; 2];
                bytes.copy_from_slice(&out[..2]);
                let value = u16::from_be_bytes(bytes);
                WithSize {
                    size: 4,
                    value: Flow {
                        flow,
                        seqnum: FlowSeqnum::U16(value),
                    },
                }
            }
            FlowType::U32 => {
                let mut bytes = [0; 4];
                bytes.copy_from_slice(&out[..4]);
                let value = u32::from_be_bytes(bytes);
                WithSize {
                    size: 6,
                    value: Flow {
                        flow,
                        seqnum: FlowSeqnum::U32(value),
                    },
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flow() {
        let flow = Flow {
            flow: 0x12,
            seqnum: FlowSeqnum::U16(0x1234),
        };
        let mut bytes = [0; 6];
        unsafe {
            flow.encode_in(&mut bytes);
        }
        let decoded = Flow::decode(&bytes).unwrap().value;
        assert_eq!(flow, decoded);

        let flow = Flow {
            flow: 0x34,
            seqnum: FlowSeqnum::U32(0x12345678),
        };
        let mut bytes = [0; 8];
        unsafe {
            flow.encode_in(&mut bytes);
        }
        let decoded = Flow::decode(&bytes).unwrap().value;
        assert_eq!(flow, decoded);

        let raw = "36 FD 0004";
        let raw = hex::decode(raw.replace(' ', "")).unwrap();
        let decoded = Flow::decode(&raw).unwrap().value;
        assert_eq!(
            decoded,
            Flow {
                flow: 0xFD,
                seqnum: FlowSeqnum::U16(0x0004)
            }
        );
    }
}

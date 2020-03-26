use crate::codec::{Codec, ParseFail, ParseResult, ParseValue};
#[cfg(test)]
use crate::test_tools::test_item;
#[cfg(test)]
use hex_literal::hex;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UserPermissions {
    pub read: bool,
    pub write: bool,
    pub run: bool,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Permissions {
    pub encrypted: bool,
    pub executable: bool,
    pub user: UserPermissions,
    pub guest: UserPermissions,
}
impl Permissions {
    pub fn to_byte(self) -> u8 {
        let mut ret = 0;
        ret |= (self.encrypted as u8) << 7;
        ret |= (self.executable as u8) << 6;
        ret |= (self.user.read as u8) << 5;
        ret |= (self.user.write as u8) << 4;
        ret |= (self.user.run as u8) << 3;
        ret |= (self.guest.read as u8) << 2;
        ret |= (self.guest.write as u8) << 1;
        ret |= self.guest.run as u8;
        ret
    }
    pub fn from_byte(n: u8) -> Self {
        Self {
            encrypted: n & 0x80 != 0,
            executable: n & 0x40 != 0,
            user: UserPermissions {
                read: n & 0x20 != 0,
                write: n & 0x10 != 0,
                run: n & 0x08 != 0,
            },
            guest: UserPermissions {
                read: n & 0x04 != 0,
                write: n & 0x02 != 0,
                run: n & 0x01 != 0,
            },
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ActionCondition {
    List = 0,
    Read = 1,
    Write = 2,
    WriteFlush = 3,
    Unknown4 = 4,
    Unknown5 = 5,
    Unknown6 = 6,
    Unknown7 = 7,
}
impl ActionCondition {
    fn from(n: u8) -> Result<Self, ParseFail> {
        Ok(match n {
            0 => ActionCondition::List,
            1 => ActionCondition::Read,
            2 => ActionCondition::Write,
            3 => ActionCondition::WriteFlush,
            4 => ActionCondition::Unknown4,
            5 => ActionCondition::Unknown5,
            6 => ActionCondition::Unknown6,
            7 => ActionCondition::Unknown7,
            _ => panic!(),
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StorageClass {
    Transient = 0,
    Volatile = 1,
    Restorable = 2,
    Permanent = 3,
}
impl StorageClass {
    fn from(n: u8) -> Self {
        match n {
            0 => StorageClass::Transient,
            1 => StorageClass::Volatile,
            2 => StorageClass::Restorable,
            3 => StorageClass::Permanent,
            _ => panic!(),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FileProperties {
    pub act_en: bool,
    pub act_cond: ActionCondition,
    pub storage_class: StorageClass,
}
impl FileProperties {
    pub fn to_byte(self) -> u8 {
        let mut ret = 0;
        ret |= (self.act_en as u8) << 7;
        ret |= (self.act_cond as u8) << 4;
        ret |= self.storage_class as u8;
        ret
    }
    pub fn from_byte(n: u8) -> Result<Self, ParseFail> {
        Ok(Self {
            act_en: n & 0x80 != 0,
            act_cond: ActionCondition::from((n >> 4) & 0x7)?,
            storage_class: StorageClass::from(n & 0x03),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FileHeader {
    pub permissions: Permissions,
    pub properties: FileProperties,
    pub alp_cmd_fid: u8,
    pub interface_file_id: u8,
    pub file_size: u32,
    pub allocated_size: u32,
}
impl Codec for FileHeader {
    fn encoded_size(&self) -> usize {
        12
    }
    fn encode(&self, out: &mut [u8]) -> usize {
        out[0] = self.permissions.to_byte();
        out[1] = self.properties.to_byte();
        out[2] = self.alp_cmd_fid;
        out[3] = self.interface_file_id;
        out[4..4 + 4].clone_from_slice(&self.file_size.to_be_bytes());
        out[8..8 + 4].clone_from_slice(&self.allocated_size.to_be_bytes());
        12
    }
    fn decode(out: &[u8]) -> ParseResult<Self> {
        if out.len() < 12 {
            return Err(ParseFail::MissingBytes(Some(12 - out.len())));
        }
        let mut file_size_bytes = [0u8; 4];
        file_size_bytes.clone_from_slice(&out[4..4 + 4]);
        let mut allocated_size_bytes = [0u8; 4];
        allocated_size_bytes.clone_from_slice(&out[8..8 + 4]);
        Ok(ParseValue {
            value: Self {
                permissions: Permissions::from_byte(out[0]),
                properties: FileProperties::from_byte(out[1]).map_err(|e| match e {
                    ParseFail::Error { error, offset } => ParseFail::Error {
                        error,
                        offset: offset + 1,
                    },
                    x => x,
                })?,
                alp_cmd_fid: out[2],
                interface_file_id: out[3],
                file_size: u32::from_be_bytes(file_size_bytes),
                allocated_size: u32::from_be_bytes(allocated_size_bytes),
            },
            size: 12,
        })
    }
}
#[test]
fn test_file_header() {
    test_item(
        FileHeader {
            permissions: Permissions {
                encrypted: true,
                executable: false,
                user: UserPermissions {
                    read: true,
                    write: true,
                    run: true,
                },
                guest: UserPermissions {
                    read: false,
                    write: false,
                    run: false,
                },
            },
            properties: FileProperties {
                act_en: false,
                act_cond: ActionCondition::Read,
                storage_class: StorageClass::Permanent,
            },
            alp_cmd_fid: 1,
            interface_file_id: 2,
            file_size: 0xDEAD_BEEF,
            allocated_size: 0xBAAD_FACE,
        },
        &hex!("B8 13 01 02 DEADBEEF BAADFACE"),
    )
}

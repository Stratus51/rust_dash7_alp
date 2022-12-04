use crate::codec::{Codec, StdError, WithOffset, WithSize};
#[cfg(test)]
use crate::test_tools::test_item;
#[cfg(test)]
use hex_literal::hex;

/// Permissions of a given user regarding a specific file.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UserPermissions {
    pub read: bool,
    pub write: bool,
    pub run: bool,
}
impl std::fmt::Display for UserPermissions {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            if self.read { "R" } else { "-" },
            if self.write { "W" } else { "-" },
            if self.run { "X" } else { "-" }
        )
    }
}
/// Description of the permissions for a file for all users.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Permissions {
    /// Whether data element is encrypted
    /// WARNING: This meaning might be deprecated
    pub encrypted: bool,
    /// Whether data element is executable
    /// WARNING: This meaning might be deprecated
    pub executable: bool,
    // ALP_SPEC: Why can't we set {read, write, run} level permission encoded on 2 bit instead?
    // Because allowing guest but not user makes no sense.
    /// Permissions for role "user"
    pub user: UserPermissions,
    /// Permissions for role "guest"
    pub guest: UserPermissions,
    // ALP_SPEC: Where are the permissions for role root?
}
impl std::fmt::Display for Permissions {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}{}|user={}|guest={}",
            if self.encrypted { "E" } else { "-" },
            if self.executable { "X" } else { "-" },
            self.user,
            self.guest
        )
    }
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
/// File access type event that will trigger an ALP action.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ActionCondition {
    /// Check for existence
    /// (L)
    List = 0,
    /// Trigger upon file read
    /// (R)
    Read = 1,
    /// Trigger upon file write
    /// (W)
    Write = 2,
    /// Trigger upon file write-flush
    /// (V)
    // ALP_SPEC Action write-flush does not exist. Only write and flush exist.
    WriteFlush = 3,
    Unknown4 = 4,
    Unknown5 = 5,
    Unknown6 = 6,
    Unknown7 = 7,
}
impl ActionCondition {
    fn from(n: u8) -> Self {
        match n {
            0 => ActionCondition::List,
            1 => ActionCondition::Read,
            2 => ActionCondition::Write,
            3 => ActionCondition::WriteFlush,
            4 => ActionCondition::Unknown4,
            5 => ActionCondition::Unknown5,
            6 => ActionCondition::Unknown6,
            7 => ActionCondition::Unknown7,
            // Impossible
            _ => panic!(),
        }
    }
}
impl std::fmt::Display for ActionCondition {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::List => write!(f, "L"),
            Self::Read => write!(f, "R"),
            Self::Write => write!(f, "W"),
            Self::WriteFlush => write!(f, "V"),
            x => write!(f, "{}", *x as u8),
        }
    }
}
/// Type of storage
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StorageClass {
    /// The content is not kept in memory. It cannot be read back.
    Transient = 0,
    /// The content is kept in a volatile memory of the device. It is accessible for
    /// read, and is lost on power off.
    Volatile = 1,
    /// The content is kept in a volatile memory of the device. It is accessible for
    /// read, and can be backed-up upon request in a permanent storage
    /// location. It is restored from the permanent location on device power on.
    Restorable = 2,
    /// The content is kept in a permanent memory of the device. It is accessible
    /// for read and write.
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
impl std::fmt::Display for StorageClass {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Transient => "T",
                Self::Volatile => "V",
                Self::Restorable => "R",
                Self::Permanent => "P",
            }
        )
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FileProperties {
    /// Enables the D7AActP (ALP action to trigger upon some type of access to this file)
    pub act_en: bool,
    /// Type of access needed to trigger the D7AActP
    pub act_cond: ActionCondition,
    /// Type of storage of this file
    pub storage_class: StorageClass,
}
impl std::fmt::Display for FileProperties {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.act_en as u8, self.act_cond, self.storage_class
        )
    }
}
impl FileProperties {
    pub fn to_byte(self) -> u8 {
        let mut ret = 0;
        ret |= (self.act_en as u8) << 7;
        ret |= (self.act_cond as u8) << 4;
        ret |= self.storage_class as u8;
        ret
    }
    pub fn from_byte(n: u8) -> Self {
        Self {
            act_en: n & 0x80 != 0,
            act_cond: ActionCondition::from((n >> 4) & 0x7),
            storage_class: StorageClass::from(n & 0x03),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FileHeader {
    /// Permissions of the file
    pub permissions: Permissions,
    /// Properties of the file
    pub properties: FileProperties,
    /// Index of the File containing the ALP Command, executed
    /// by D7AActP. Discarded if the ACT_EN field in Properties
    /// is set to 0.
    pub alp_cmd_fid: u8,
    /// Index of the File containing the Interface, on which the
    /// result of D7AActP is sent. Discarded if the ACT_EN field
    /// in Properties is set to 0.
    pub interface_file_id: u8,
    /// Current size of the file.
    pub file_size: u32,
    /// Size, allocated for the file in memory (appending data to
    /// the file cannot exceed this value)
    pub allocated_size: u32,
    // ALP_SPEC What is the difference between file_size and allocated_size? When a file is
    // declared, less than its size is allocated and then it grows dynamically?
}
impl std::fmt::Display for FileHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "[{}|{}|f({}),{},{},{}]",
            self.permissions,
            self.properties,
            self.alp_cmd_fid,
            self.interface_file_id,
            self.file_size,
            self.allocated_size
        )
    }
}
impl Codec for FileHeader {
    type Error = StdError;
    fn encoded_size(&self) -> usize {
        12
    }
    unsafe fn encode_in(&self, out: &mut [u8]) -> usize {
        out[0] = self.permissions.to_byte();
        out[1] = self.properties.to_byte();
        out[2] = self.alp_cmd_fid;
        out[3] = self.interface_file_id;
        out[4..4 + 4].clone_from_slice(&self.file_size.to_be_bytes());
        out[8..8 + 4].clone_from_slice(&self.allocated_size.to_be_bytes());
        12
    }
    fn decode(out: &[u8]) -> Result<WithSize<Self>, WithOffset<Self::Error>> {
        if out.len() < 12 {
            return Err(WithOffset::new_head(Self::Error::MissingBytes(
                12 - out.len(),
            )));
        }
        let mut file_size_bytes = [0u8; 4];
        file_size_bytes.clone_from_slice(&out[4..4 + 4]);
        let mut allocated_size_bytes = [0u8; 4];
        allocated_size_bytes.clone_from_slice(&out[8..8 + 4]);
        Ok(WithSize {
            value: Self {
                permissions: Permissions::from_byte(out[0]),
                properties: FileProperties::from_byte(out[1]),
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

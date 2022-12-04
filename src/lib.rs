//! Implementation of a [Dash7](https://dash7-alliance.org/) ALP protocol parser from its
//! public specification.
//!
//! The protocol
//! ==============================================================================
//! The protocol specifies ALP Commands that can be sent to another system to communicate.
//! Each command is an aggregation of ALP Actions.
//!
//! The protocol is based on the fact that each communicating party hold a Dash7 filesystem.
//! Each request toward an other device is then composed as an array of simple filesystem operation
//! (ALP actions).
//!
//! About this library
//! ==============================================================================
//! The goal of this library is to implement a specification with an emphasis on correctness, then
//! on usability. Performance and memory usage are currently considered a secondary objective.
//!
//! Quickstart
//! ==============================================================================
//!
//! ```
//! use dash7_alp::v1_2::{Command, Action, action};
//! use hex_literal::hex;
//!
//! let cmd = Command {
//!     actions: vec![
//!         Action::RequestTag(action::RequestTag { id: 66, eop: true }),
//!         Action::ReadFileData(action::ReadFileData {
//!             resp: true,
//!             group: false,
//!             file_id: 0,
//!             offset: 0,
//!             size: 8,
//!         }),
//!         Action::ReadFileData(action::ReadFileData {
//!             resp: false,
//!             group: true,
//!             file_id: 4,
//!             offset: 2,
//!             size: 3,
//!         }),
//!         Action::Nop(action::Nop {
//!             resp: true,
//!             group: true,
//!         }),
//!     ],
//! };
//! let data = &hex!("B4 42   41 00 00 08   81 04 02 03  C0") as &[u8];
//!
//! assert_eq!(&cmd.encode()[..], data);
//! let parsed_cmd = Command::decode(data).expect("should be parsed without error");
//! assert_eq!(parsed_cmd, cmd);
//! ```

/// Implementation of the version 1.2 of the Dash7 ALP protocol
pub mod v1_2;

/// A Codec module specifying how to encode/decode each encodable items
pub mod codec;

#[cfg(test)]
pub(crate) mod test_tools;

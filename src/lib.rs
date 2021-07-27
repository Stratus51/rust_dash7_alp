//! TODO Update library description
//!
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
//! Notes
//! ==============================================================================
//! Group
//! ------------------------------------------------------------------------------
//! Many ALP action have a group flag. This allows those to be grouped.
//!
//! This means that:
//! - If any action of this group fails, the next actions are skipped.
//! - A query before the group will apply to the whole group (to defined
//! whether it will be executed).
//! - If the group contains queries, a prior Logical action will determine how they
//! are composed between them (OR, XOR, NOR, NAND). Without any Logical action, the
//! queries are AND'ed.
//!
//! Codec trait
//! ------------------------------------------------------------------------------
//! This trait implements the encode/decode methods. You very probably want to import
//! it into scope.

#![cfg_attr(not(test), no_std)]
// Library no panic
#![warn(clippy::unwrap_in_result, clippy::panic, clippy::expect_used)]
// Library cleanliness
#![warn(
    clippy::print_stdout,
    clippy::needless_borrow,
    clippy::missing_errors_doc
)]
// Embedded constraints
#![warn(clippy::float_arithmetic)]
// Style
#![warn(clippy::unseparated_literal_suffix)]
#![warn(clippy::indexing_slicing)]
// Manual review of dangerous stuff
// Should be deactivated in commited code
// #![warn(clippy::integer_arithmetic)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[macro_use]
pub mod decodable;
pub mod define;
pub mod encodable;
pub mod v1_2;
pub mod varint;

// TODO Verify each item's name against the SPEC (after implementing everything)

// TODO Int enums: fn from(): find a way to avoid double value definition
// TODO Int enums: optim: find a way to cast from int to enum instead of calling a matching
// function (much more resource intensive). Only do that for enums that match all possible
// values that result from the parsing.
// TODO usize is target dependent. In other words, on a 16 bit processor, we will run into
// troubles if we were to convert u32 to usize (even if a 64Ko payload seems a bit big).
// Maybe we should just embrace this limitation? (Not to be lazy or anything...)
// The bad thing is that u32 to u16 will compile and panic at runtime if the value is too big.

// TODO Turn into cargo multi-project
// TODO Add cross language wrapper embryo to check compatibility
// TODO Optimize struct fields order for repr(C) alignment?

// TODO Add tests to verify the impact of the feature flags on binary size
// TODO Review the code: there should not be any operation that could panic used in the code.
// Either always work, or return the appropriate error.

// TODO Implement packet parser and packet encoder (in addition to action parse and action encoder)
// TODO Refactor encoding/decoding to spit out remaining buffer instead of size

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

#![no_std]


// TODO Verify each item's name against the SPEC

// TODO Look into const function to replace some macros?
// TODO Use uninitialized memory where possible
// TODO Int enums: fn from(): find a way to avoid double value definition
// TODO Int enums: optim: find a way to cast from int to enum instead of calling a matching
// function (much more resource intensive). Only do that for enums that match all possible
// values that result from the parsing.
// TODO Optimize min size calculation (fold it into the upper OP when possible)
// TODO usize is target dependent. In other words, on a 16 bit processor, we will run into
// troubles if we were to convert u32 to usize (even if a 64Ko payload seems a bit big).
// Maybe we should just embrace this limitation? (Not to be lazy or anything...)
// The bad thing is that u32 to u16 will compile and panic at runtime if the value is too big.
// TODO Slice copies still check length consistency dynamically. Is there a way to get rid of that
// at runtime while still testing it at compile/test time?
//      - For simple index access, get_unchecked_mut can do the trick. But It makes the code hard to
//      read...
// TODO is {out = &out[offset..]; out[..size]} more efficient than {out[offset..offset+size]} ?
// TODO Add function to encode without having to define a temporary structure

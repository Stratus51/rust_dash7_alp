pub mod action;
pub mod define;

// TODO These errors containing the data pointer are cool.
// But they currently force a loss of mutability of mutable decode error.
// That is sad, but fixing it requires quite some refactoring and some degree of code duplication.

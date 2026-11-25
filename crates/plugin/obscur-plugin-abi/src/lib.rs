//! Native C ABI boundary (unsafe permitted behind guards).
#![deny(unsafe_op_in_unsafe_fn)]

/// The ABI boundary crossing itself.
pub mod boundary;
/// Converting values across the boundary.
pub mod conversion;
/// The table of functions the two sides agree on.
pub mod function_table;
/// Who owns a value once it has crossed.
pub mod ownership;
/// The value representation on the wire.
pub mod value;

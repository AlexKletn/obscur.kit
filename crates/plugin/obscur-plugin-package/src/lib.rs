//! Security-sensitive plugin package reading and validation.
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

/// The package file format.
pub mod archive;
/// Unpacking a package onto disk.
pub mod extraction;
/// The package itself.
pub mod package;
/// Paths inside and around a package.
pub mod path;
/// Checking a package before trusting it.
pub mod validation;

/// Persistent publication of verified packages on a host filesystem.
#[cfg(any(unix, windows))]
pub mod installation;

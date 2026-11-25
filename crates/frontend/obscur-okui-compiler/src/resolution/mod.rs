#![doc = include_str!("../../docs/api/resolution/mod.doc/_module.md")]

mod resolution_error;
pub use resolution_error::ResolutionError;
mod resolution_error_kind;
pub use resolution_error_kind::ResolutionErrorKind;

mod name_resolver;
pub use name_resolver::NameResolver;

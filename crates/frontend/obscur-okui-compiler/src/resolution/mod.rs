#![doc = include_str!("mod.doc/_module.md")]

mod resolution_error;
pub use resolution_error::{ResolutionError, ResolutionErrorKind};

mod name_resolver;
pub use name_resolver::NameResolver;

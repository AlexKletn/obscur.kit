//! Name resolution over the `.okui` document tree (spec §2.10, §2.11).
//!
//! [`NameResolver`] walks a parsed document and checks that every attribute
//! expression's leading reference resolves to a symbol defined in the enclosing
//! [`SymbolTable`](crate::symbol::SymbolTable). Failures are reported as a
//! [`ResolutionError`] of a given [`ResolutionErrorKind`].

mod resolution_error;
pub use resolution_error::{ResolutionError, ResolutionErrorKind};

mod name_resolver;
pub use name_resolver::NameResolver;

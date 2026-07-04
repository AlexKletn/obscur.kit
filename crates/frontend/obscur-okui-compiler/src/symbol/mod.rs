//! Symbols, scopes and the lexical symbol table (spec §2.10).
//!
//! This module holds the name-binding model that name resolution walks: a
//! [`Symbol`] is a named entity of a given [`SymbolKind`]; a [`Scope`] owns the
//! symbols bound directly within one lexical region; and a [`SymbolTable`] is
//! the arena of scopes with a current-scope cursor that lexical lookup walks
//! outward from.

mod symbol;
pub use symbol::{Symbol, SymbolKind};

mod scope;
pub use scope::Scope;

mod symbol_table;
pub use symbol_table::SymbolTable;

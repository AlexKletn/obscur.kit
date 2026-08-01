#![doc = include_str!("mod.doc/_module.md")]

mod symbol;
pub use symbol::{Symbol, SymbolKind};

mod scope;
pub use scope::Scope;

mod symbol_table;
pub use symbol_table::SymbolTable;

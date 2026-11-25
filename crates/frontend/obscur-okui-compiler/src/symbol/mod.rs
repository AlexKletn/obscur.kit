#![doc = include_str!("../../docs/api/symbol/mod.doc/_module.md")]

mod symbol;
pub use symbol::Symbol;
mod symbol_kind;
pub use symbol_kind::SymbolKind;

mod scope;
pub use scope::Scope;

mod symbol_table;
pub use symbol_table::SymbolTable;

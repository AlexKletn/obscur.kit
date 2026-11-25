#![doc = include_str!("../../docs/api/codegen/mod.doc/_module.md")]
mod codegen_root;
mod view_codegen;
pub use codegen_root::CodegenRoot;
pub use view_codegen::ViewCodegen;

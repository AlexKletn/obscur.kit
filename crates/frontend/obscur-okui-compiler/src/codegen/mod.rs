#![doc = include_str!("mod.doc/_module.md")]
mod codegen_root;
mod view_codegen;
pub use codegen_root::CodegenRoot;
pub use view_codegen::ViewCodegen;

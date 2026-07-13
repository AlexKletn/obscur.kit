//! Build-time `.okui` → Rust code generation for the codegen showcase example.
//!
//! This realises the "`.okui` compiles to ordinary code, not a runtime
//! interpreter" model: the authored [`showcase.okui`](examples/showcase.okui) is
//! parsed, compiled and lowered to a runtime `View` **here, at build time**, and
//! the generated `pub fn showcase_view() -> View` is written into `OUT_DIR`. The
//! `showcase_okui_codegen_png` example `include!`s it and renders it, so no
//! `.okui` parsing/compiling machinery is present when that example runs.
//!
//! The generation always runs (it is cheap — one small document) but only the
//! example consumes the output, so ordinary library builds are unaffected.

use std::path::Path;

use obscur_okui_compiler::codegen::ViewCodegen;
use obscur_okui_compiler::compile::Compiler;
use obscur_okui_syntax::ast::Document;
use obscur_okui_syntax::parser::Parser;

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR");

    let okui_path = Path::new(&manifest_dir).join("examples/showcase.okui");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", okui_path.display());

    let source = std::fs::read_to_string(&okui_path).expect("read showcase.okui");

    // Parse → compile → lower to UI IR (the same pipeline the runtime bridge
    // uses, run once ahead of time instead of every frame).
    let parsed = Parser::new().parse(&source);
    let document = Document::cast(parsed.tree().root().clone()).expect("root casts to a Document");
    let result = Compiler::new("showcase.okui").compile(&document);
    let ir = result.document().expect("a lowered UI IR document");

    // Emit ordinary Rust building the identical `View`.
    let generated = ViewCodegen::document_to_view_fn(ir, "showcase_view");

    let out_path = Path::new(&out_dir).join("showcase_okui_view.rs");
    std::fs::write(&out_path, generated).expect("write generated view");
}

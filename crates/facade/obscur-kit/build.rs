#![doc = include_str!("build.doc/_module.md")]

use std::path::Path;

use obscur_okui_compiler::codegen::{CodegenRoot, ViewCodegen};
use obscur_okui_compiler::compile::Compiler;
use obscur_okui_compiler::validation::CommandSurface;
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
    // The command surface is declared **empty**, and that is a claim rather than a
    // default: the example that renders this document (`showcase_okui_codegen_png`)
    // registers no handler at all, so any `on:click=…` written into the markup
    // would hash to a command nothing dispatches — the silent no-op §2.13's "string
    // method names are compile errors" rules out. Saying "no commands" here turns
    // that into a build failure naming the handler. Add the name to this list at
    // the same time as the registration, never before.
    let result = Compiler::new("showcase.okui")
        .with_commands(CommandSurface::declared(Vec::<&str>::new()))
        .compile(&document);

    // Lowering does not stop at a diagnostic — a rejected document still produces
    // IR — so a build that reads `document()` without asking `has_errors()` first
    // runs every check and then acts on none of them. That is how §2.10's
    // expression allow-list came to refuse `{std::fs::read(p)}` while the one
    // production build path emitted it as real Rust regardless.
    if result.has_errors() {
        for diagnostic in result.diagnostics() {
            println!("cargo:warning=showcase.okui: {diagnostic:?}");
        }
        panic!("showcase.okui failed validation; see the warnings above");
    }

    let ir = result.document().expect("a lowered UI IR document");

    // Emit ordinary Rust building the identical `View`, naming its types through
    // the facade — the root an application's own `build.rs` must also pass, and
    // the one that makes the generated file compile in a crate whose only
    // dependency is `obscur-kit`. The example that `include!`s this output is a
    // separate crate linking `obscur_kit` by name, so the same paths resolve here.
    let generated = ViewCodegen::document_to_view_fn(ir, "showcase_view", &CodegenRoot::facade());

    let out_path = Path::new(&out_dir).join("showcase_okui_view.rs");
    std::fs::write(&out_path, generated).expect("write generated view");
}

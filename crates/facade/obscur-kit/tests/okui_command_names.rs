#![doc = include_str!("okui_command_names.doc/_module.md")]
#![forbid(unsafe_code)]

use obscur_kit::identity::InternedSymbol;
use obscur_kit::okui_compiler::compile::{CompileDiagnostic, Compiler};
use obscur_kit::okui_compiler::validation::CommandSurface;
use obscur_kit::okui_syntax::ast::Document;
use obscur_kit::okui_syntax::parser::Parser;
use obscur_kit::ui_ir::name_binding;
use obscur_kit::ui_macros::okui;
use obscur_kit::view::view::View;

#[doc = include_str!("okui_command_names.doc/diagnostics.md")]
fn diagnostics(source: &str, commands: CommandSurface) -> Vec<String> {
    let document =
        Document::cast(Parser::new().parse(source).tree().root().clone()).expect("a document");
    Compiler::new("commands.okui")
        .with_commands(commands)
        .compile(&document)
        .diagnostics()
        .iter()
        .map(CompileDiagnostic::to_string)
        .collect()
}

#[test]
fn a_handler_no_declared_command_claims_is_a_compile_error() {
    let reported = diagnostics(
        r#"<div class="app"><button class="btn" on:click=incrment/></div>"#,
        CommandSurface::declared(["increment"]),
    );
    assert!(
        reported
            .iter()
            .any(|message| message.contains("unknown command 'incrment'")),
        "the offending handler must be named: {reported:?}"
    );
    assert!(
        reported
            .iter()
            .any(|message| message.contains("did you mean 'increment'?")),
        "and the nearest declared command offered: {reported:?}"
    );
}

#[test]
fn the_declared_handler_compiles_clean_in_both_spellings() {
    // The disjoint half. `on:click=increment` and `on:click={increment}` are the
    // same handler reference — the braced form is not an expression to evaluate —
    // and both are legal once the command is claimed.
    for source in [
        r#"<button class="btn" on:click=increment/>"#,
        r#"<button class="btn" on:click={increment}/>"#,
    ] {
        assert_eq!(
            diagnostics(source, CommandSurface::declared(["increment"])),
            Vec::<String>::new(),
            "`{source}` names a claimed command"
        );
    }
}

#[test]
fn a_caller_that_declares_nothing_is_told_nothing() {
    // The default must stay silent. Nothing in a document declares its commands —
    // §2.13 names no authoring surface for that — so silence from the caller is not
    // a claim that no command exists, and refusing the binding would be a false
    // error on correct code.
    assert_eq!(
        diagnostics(
            r#"<button class="btn" on:click=whatever_nobody_declared/>"#,
            CommandSurface::Unchecked,
        ),
        Vec::<String>::new()
    );
}

#[test]
fn an_empty_declaration_is_the_claim_that_no_handler_may_be_bound() {
    // The state emptiness could not express while it doubled as "unset". This is
    // what `obscur-kit`'s own `build.rs` declares for `showcase.okui`.
    let reported = diagnostics(
        r#"<button class="btn" on:click=increment/>"#,
        CommandSurface::declared(Vec::<&str>::new()),
    );
    assert!(
        reported
            .iter()
            .any(|message| message.contains("unknown command 'increment'")),
        "{reported:?}"
    );
}

#[test]
fn the_checked_name_is_the_name_the_runtime_would_dispatch() {
    // Why the check is worth anything: the string the validator judges is the same
    // string the IR carries and the runtime hashes. A checker with its own
    // extraction could admit a name the artefact does not carry, and a validator
    // that disagrees with the artefact is worse than none — so the handler text is
    // read through the lowering's own function.
    assert_eq!(
        name_binding::fallback_symbol("increment"),
        InternedSymbol::named("increment"),
        "the symbol a declared command name resolves to is the one an application \
         registers"
    );
    assert_eq!(
        diagnostics(
            r#"<button class="btn" on:click=increment/>"#,
            CommandSurface::declared(["increment"]),
        ),
        Vec::<String>::new(),
        "so declaring exactly that name admits exactly that binding"
    );
}

#[test]
fn a_binding_that_names_no_handler_at_all_is_reported_on_its_own_terms() {
    let reported = diagnostics(
        r#"<button class="btn" on:click/>"#,
        CommandSurface::declared(["increment"]),
    );
    assert!(
        reported
            .iter()
            .any(|message| message.contains("names no handler")),
        "an empty handler hashes to a symbol nothing can register, and \
         \"unknown command ''\" would name nothing to fix: {reported:?}"
    );
}

#[test]
fn a_macro_binding_is_deliberately_not_checked() {
    // The residue, pinned as a decision. This expands — it must, because the macro
    // cannot see whether anything claims `never_registered_anywhere`: the
    // registrations are runtime values and this crate is still compiling. A guess
    // here would break correct code, so the inline front end stays silent and the
    // file front end carries the check.
    let view: View = okui! { <button class="btn" on:click={never_registered_anywhere}/> };
    assert_eq!(
        view.root().nodes().len(),
        1,
        "the macro compiles the binding it cannot judge"
    );
}

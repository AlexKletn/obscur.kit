//! **SPECIFICATION-DERIVED** cases for §2.10 Expressions ("Forbidden:
//! - assignment; - unconstrained arbitrary function calls; - I/O; - async work;
//! - direct service access;"), §2.9 ("- no business logic."), §2.12 ("Templates
//! are stateless and lifecycle-free.") and §2.13 ("Unknown attributes, events and
//! string method names are compile errors.") — the invariants behind the §9.1
//! non-goals "automatic execution of business logic inside `.okui`" and "a full
//! HTML compatibility layer".
//!
//! Every expected value was produced by an agent reading the specification with
//! **no access to this repository's source**. A failure here is a defect report
//! about the engine, and must not be "corrected" by editing the expected value.
#![forbid(unsafe_code)]

#[path = "spec_ch9_support/mod.rs"]
mod support;

use obscur_kit::okui_compiler::compile::{CompileResult, Compiler, ContractRegistry};
use obscur_kit::okui_compiler::symbol::{Symbol, SymbolKind};
use obscur_kit::okui_syntax::ast::Document;
use obscur_kit::okui_syntax::parser::Parser;

/// Runs the real `.okui` frontend over `source`, with `symbols` declared in the
/// document's root scope.
fn compile_with(source: &str, symbols: Vec<Symbol>) -> CompileResult {
    let parsed = Parser::new().parse(source);
    let document = Document::cast(parsed.tree().root().clone()).expect("root casts to Document");
    let mut compiler = Compiler::new("spec_ch9.okui").with_registry(ContractRegistry::builtins());
    for symbol in symbols {
        compiler = compiler.defining(symbol);
    }
    compiler.compile(&document)
}

/// Runs the real `.okui` frontend over `source` with the ordinary value bindings
/// the cases reference (`count`, `item`, `label`, `handler`) already declared, so
/// the only thing under test is the expression restriction itself.
fn compile_okui(source: &str) -> CompileResult {
    compile_with(
        source,
        vec![
            Symbol::state("count"),
            Symbol::prop("item"),
            Symbol::prop("label"),
            Symbol::prop("handler"),
        ],
    )
}

/// Every diagnostic message the frontend produced, joined for assertion.
fn messages(result: &CompileResult) -> String {
    result
        .diagnostics()
        .iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}

/// §2.10 Expressions — "Forbidden:" / "- assignment;" and §2.9 "- no business
/// logic."
///
/// NG-OKUI-ASSIGN-01.
#[test]
fn assignment_inside_an_interpolation_is_a_compile_error() {
    let result = compile_okui("<text>{ count = count + 1 }</text>");
    assert!(
        result.has_errors(),
        "assignment must fail compilation; diagnostics were: {:?}",
        result.diagnostics()
    );
    let text = messages(&result).to_lowercase();
    assert!(
        text.contains("assign"),
        "the error identifies assignment as forbidden: {text}"
    );
    assert!(
        result.document().is_none(),
        "no output artifact is produced for a forbidden expression"
    );
}

/// §2.10 — "Forbidden:" / "- unconstrained arbitrary function calls;" vs
/// "Allowed:" / "- explicitly exported pure functions."
///
/// NG-OKUI-CALL-01.
#[test]
fn only_explicitly_exported_pure_functions_may_be_called() {
    // (a) an exported pure function compiles and renders.
    let allowed = compile_with(
        "<text>{ format_price(item.cents) }</text>",
        vec![
            Symbol::prop("item"),
            Symbol::new("format_price", SymbolKind::Function),
        ],
    );
    assert!(
        !allowed.has_errors(),
        "a declared exported-pure call compiles; diagnostics were: {:?}",
        allowed.diagnostics()
    );

    // (b) an ordinary in-scope Rust function is refused by name.
    let refused = compile_okui("<text>{ compute_total(item) }</text>");
    assert!(
        refused.has_errors(),
        "an undeclared call must fail compilation; diagnostics were: {:?}",
        refused.diagnostics()
    );
    let text = messages(&refused);
    assert!(
        text.contains("compute_total"),
        "the error names the refused callee: {text}"
    );
    assert!(
        text.to_lowercase().contains("pure"),
        "the error says it is not an exported pure function: {text}"
    );
}

/// §2.10 — "Forbidden:" / "- I/O;" / "- async work;" / "- direct service
/// access;" and §9.1 "- automatic execution of business logic inside `.okui`."
///
/// NG-OKUI-IO-01.
#[test]
fn io_async_and_direct_service_access_are_each_refused_by_category() {
    for (source, category) in [
        ("<text>{ read_file(\"a.txt\") }</text>", "i/o"),
        ("<text>{ await load_user() }</text>", "async"),
        ("<text>{ services.storage.get(\"k\") }</text>", "service"),
    ] {
        let result = compile_okui(source);
        assert!(
            result.has_errors(),
            "`{source}` must fail compilation; diagnostics were: {:?}",
            result.diagnostics()
        );
        let text = messages(&result).to_lowercase();
        assert!(
            text.contains(category),
            "`{source}` is refused as `{category}`, not incidentally: {text}"
        );
    }
}

/// §2.9 External `.okui` files — "- no business logic." and §2.12 "Templates are
/// stateless and lifecycle-free."
///
/// NG-OKUI-PURE-01.
#[test]
fn rendering_the_same_template_twice_is_byte_identical_and_side_effect_free() {
    let source = "<template><text>{ label }</text></template>";
    let first = compile_okui(source);
    let second = compile_okui(source);
    assert!(!first.has_errors(), "{:?}", first.diagnostics());

    let a = first.document().expect("a lowered document");
    let b = second.document().expect("a lowered document");
    assert_eq!(
        format!("{a:?}"),
        format!("{b:?}"),
        "two renders with identical props and state produce identical UI IR"
    );
}

/// §2.13 Component contracts — "Unknown attributes, events and string method
/// names are compile errors." (guardrail §9.6 "introduce untyped string
/// dispatch")
///
/// NG-OKUI-UNKNOWN-01.
#[test]
fn unknown_events_attributes_and_elements_are_compile_errors() {
    // (a) a mistyped event name.
    let event = compile_okui("<Button on:clik={handler}>x</Button>");
    assert!(
        event.has_errors(),
        "a mistyped event name must fail compilation; diagnostics were: {:?}",
        event.diagnostics()
    );
    let text = messages(&event);
    assert!(text.contains("clik"), "the error names `clik`: {text}");
    assert!(
        text.contains("click"),
        "the error suggests the declared event `click`: {text}"
    );

    // (b) a mistyped attribute name.
    let attribute = compile_okui("<Button labl=\"x\"/>");
    assert!(
        attribute.has_errors(),
        "a mistyped attribute must fail compilation; diagnostics were: {:?}",
        attribute.diagnostics()
    );
    assert!(
        messages(&attribute).contains("labl"),
        "the error names the unknown attribute: {}",
        messages(&attribute)
    );

    // (c) an unregistered element is NOT silently a generic box.
    let element = compile_okui("<div>x</div>");
    assert!(
        element.has_errors(),
        "an unregistered component must fail compilation; diagnostics were: {:?}",
        element.diagnostics()
    );
    assert!(
        messages(&element).contains("div"),
        "the error names the unregistered component `div`: {}",
        messages(&element)
    );
}

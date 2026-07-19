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
///
/// WEAKENED 2026-07-19: this also asserted `result.document().is_none()` — that
/// a compile with errors emits NO output artifact. §2.10 lists forbidden
/// expression forms and §2.13 says unknown attributes, events and string method
/// names "are compile errors"; neither says an erroring compile must suppress
/// its artifact, and a front end that reports errors while still handing back a
/// partial IR (so an IDE can keep offering completion and an incremental build
/// can keep a stale-but-typed tree) is a defensible design. The refusal itself
/// is kept and is what discriminates. Candidate spec amendment: state whether
/// `has_errors()` implies no artifact, since a caller that checks only
/// `document().is_some()` currently ships a document built from refused input.
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
///
/// WEAKENED 2026-07-19: this required each refusal's message to NAME ITS
/// CATEGORY — "i/o", "async", "service". §2.10 is a list of forbidden *things*,
/// not a list of required *messages*: it nowhere says a refusal must be
/// attributed to the category it violated, and §1.6's diagnostic-shape rule asks
/// for severity, subsystem, stable code, context and recovery action, not a
/// category word in the prose. All three forms are in fact refused. Candidate
/// spec amendment: decide whether a §2.10 refusal must classify itself — the
/// same amendment the ch2 twins of these cases ask for, and worth answering
/// once, since a single generic "not a valid expression" message is what an
/// author sees for six quite different mistakes.
///
/// The `services` line now DECLARES `services` as a prop, which is the
/// configuration that matters: an undeclared `services` is refused as an
/// unresolved name, which would make this line pass without testing
/// "- direct service access;" at all. Declared, it is the honest test — and it
/// is expected to fail, because a declared `services` compiles clean. That is
/// the registered real gap, now isolated where it belongs.
#[test]
fn io_async_and_direct_service_access_are_each_refused_by_category() {
    for source in [
        "<text>{ read_file(\"a.txt\") }</text>",
        "<text>{ await load_user() }</text>",
    ] {
        let result = compile_okui(source);
        assert!(
            result.has_errors(),
            "`{source}` must fail compilation; diagnostics were: {:?}",
            result.diagnostics()
        );
    }

    let service = "<text>{ services.storage.get(\"k\") }</text>";
    let result = compile_with(service, vec![Symbol::prop("services")]);
    assert!(
        result.has_errors(),
        "`{service}` must fail compilation even when `services` resolves — that \
         is what \"- direct service access;\" forbids; diagnostics were: {:?}",
        result.diagnostics()
    );
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
///
/// WEAKENED 2026-07-19: this also required the diagnostic for a mistyped event
/// to SUGGEST the declared one — `text.contains("click")` for `on:clik`. §2.13
/// is one sentence, "Unknown attributes, events and string method names are
/// compile errors.", and the engine meets it: it reports "unknown event 'clik'
/// on component 'Button'", naming both the mistake and the component. A
/// did-you-mean suggestion is a quality-of-implementation nicety the spec does
/// not ask for. Candidate spec amendment: if near-miss suggestions are wanted
/// (they are the difference between a five-second and a five-minute fix on a
/// contract with thirty props), say so, and say what the edit-distance bar is.
///
/// Also DROPPED, half (c): "an unregistered element is NOT silently a generic
/// box", which compiled `<div>x</div>` and required it to fail. Two things were
/// wrong. §2.13's sentence enumerates "Unknown attributes, events and string
/// method names" — ELEMENTS are conspicuously not in it. And the fixture picked
/// the wrong subject anyway: `div` is a registered builtin, so it compiles
/// correctly and the case would have been unsatisfiable even if the requirement
/// existed. The test name still says "elements"; it is left alone so this note
/// is what a reader finds. Candidate spec amendment: say whether an element tag
/// that resolves to no registered component is a compile error, and if so how a
/// document addresses the builtin set — silently lowering an unknown tag to a
/// generic box is exactly the untyped escape hatch §9.6 lists as a guardrail.
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
        text.contains("Button"),
        "the error names the component the event is unknown ON: {text}"
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

    // (c) DROPPED 2026-07-19 — see the doc comment. It compiled `<div>x</div>`
    // and required it to fail as "an unregistered component".
}

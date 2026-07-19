//! **SPECIFICATION-DERIVED** expression and `.okui` front-end cases for
//! obscur.kit §2.9 (*External `.okui` files*), §2.10 (*Expressions*) and §2.11
//! (*Conditions and collections*).
//!
//! Read this before touching anything in the file.
//!
//! Every expectation below was produced by an agent reading obscur.kit §2 with
//! **no access to this repository's source**. The cases are a statement of what
//! the implementation owes the spec, not a record of what it currently does. A
//! failure here is a defect report about the engine (or, occasionally, about the
//! spec) and must not be "corrected" by editing the expected value.
//!
//! # Why the file front end and not the macro
//!
//! §2.10 is a set of *rejections*. A rejection expressed through the `okui!`
//! macro is a compile error in this very file, so the test binary would not
//! build and would report nothing. Every rejection case therefore drives the
//! `.okui` string path, whose verdict is a value (`CompileResult`) rather than a
//! compiler error — the same rule, reached through the front end that can be
//! interrogated.
//!
//! Each test's doc comment carries the quoted normative sentence it encodes.
#![forbid(unsafe_code)]

#[path = "spec_ch2_support/mod.rs"]
mod support;

use obscur_kit::okui_compiler::compile::{CompileDiagnostic, Compiler, ContractRegistry};
use obscur_kit::okui_compiler::symbol::Symbol;
use support::{okui_diagnostics_with, parse_okui};

/// The symbols an authored document in this file may read: a couple of props, a
/// piece of state, and the loop local a `For` opens.
fn scope() -> Vec<Symbol> {
    vec![
        Symbol::state("items"),
        Symbol::state("count"),
        Symbol::prop("user"),
        Symbol::prop("item"),
        Symbol::prop("x"),
        Symbol::prop("url"),
        Symbol::prop("services"),
        Symbol::loop_local("row"),
    ]
}

/// Every diagnostic message compiling `source` produces, in the scope above.
fn diagnostics(source: &str) -> Vec<String> {
    okui_diagnostics_with(source, ContractRegistry::builtins(), &scope())
}

/// Whether compiling `source` reports a diagnostic containing `needle`.
fn reports(source: &str, needle: &str) -> bool {
    diagnostics(source).iter().any(|d| d.contains(needle))
}

/// Whether compiling `source` yields an output artifact at all.
fn produces_artifact(source: &str) -> bool {
    let document = parse_okui(source);
    let mut compiler = Compiler::new("spec_ch2.okui").with_registry(ContractRegistry::builtins());
    for symbol in scope() {
        compiler = compiler.defining(symbol);
    }
    let result = compiler.compile(&document);
    !result.has_errors() && result.document().is_some()
}

// --------------------------------------------------------------- §2.10 forbidden

/// §2.10, Forbidden: "- assignment;"
#[test]
fn assignment_in_an_expression_is_rejected() {
    let source = "<span>{count = count + 1}</span>";
    assert!(
        reports(source, "assignment"),
        "a diagnostic must name assignment as a forbidden expression form; got {:?}",
        diagnostics(source)
    );
    assert!(!produces_artifact(source), "no artifact is produced");
}

/// §2.10, Forbidden: "- arbitrary mutation;"
#[test]
fn a_mutating_method_call_is_rejected_as_mutation() {
    for (source, call) in [
        ("<If cond={items.push(x)}><span/></If>", "push"),
        ("<If cond={items.sort()}><span/></If>", "sort"),
    ] {
        assert!(
            reports(source, "mutation"),
            "the diagnostic must identify `{call}` as mutation; got {:?}",
            diagnostics(source)
        );
        assert!(!produces_artifact(source), "no artifact for `{call}`");
    }
}

/// §2.10, Forbidden: "- async work;"
#[test]
fn awaiting_in_an_expression_is_rejected_as_async_work() {
    let source = "<span>{user.load().await}</span>";
    assert!(
        reports(source, "async"),
        "a diagnostic must name async work as forbidden in expressions; got {:?}",
        diagnostics(source)
    );
    assert!(!produces_artifact(source), "no artifact is produced");
}

/// §2.10, Forbidden: "- I/O;" and "- direct service access;"
#[test]
fn io_and_direct_service_access_are_each_rejected_by_name() {
    let io = "<span>{read_file(\"config.toml\")}</span>";
    assert!(
        reports(io, "I/O"),
        "the diagnostic must name the forbidden category (I/O); got {:?}",
        diagnostics(io)
    );
    assert!(
        !produces_artifact(io),
        "the I/O document produces no artifact"
    );

    let service = "<span>{services.http.get(url)}</span>";
    assert!(
        reports(service, "service"),
        "the diagnostic must name the forbidden category (direct service \
         access); got {:?}",
        diagnostics(service)
    );
    assert!(
        !produces_artifact(service),
        "the service document produces no artifact"
    );
}

/// §2.10, Forbidden: "- unconstrained arbitrary function calls;"
///
/// The pair is the whole point: `fmt_price` is an explicitly exported pure
/// function for templates and must be accepted; `fmt_tax` is an ordinary
/// function with no such marking and must be refused. Only asserting both in
/// one file catches either error.
#[test]
fn an_unexported_function_call_is_rejected_and_an_exported_one_is_not() {
    let mut compiler = Compiler::new("spec_ch2.okui").with_registry(ContractRegistry::builtins());
    for symbol in scope() {
        compiler = compiler.defining(symbol);
    }
    // The declaration of `fmt_price` as a template-exported pure function.
    compiler = compiler.defining(Symbol::new(
        "fmt_price",
        obscur_kit::okui_compiler::symbol::SymbolKind::Function,
    ));

    let both = "<div><span>{fmt_price(item.cents)}</span>\
                <span>{fmt_tax(item.cents)}</span></div>";
    let messages: Vec<String> = compiler
        .compile(&parse_okui(both))
        .diagnostics()
        .iter()
        .map(CompileDiagnostic::to_string)
        .collect();

    assert!(
        messages.iter().any(|m| m.contains("fmt_tax")),
        "the diagnostic points at the `fmt_tax` call; got {messages:?}"
    );
    assert!(
        !messages.iter().any(|m| m.contains("fmt_price")),
        "`fmt_price` is accepted; got {messages:?}"
    );

    // Removing the `fmt_tax` line makes the file compile.
    let mut ok = Compiler::new("spec_ch2.okui").with_registry(ContractRegistry::builtins());
    for symbol in scope() {
        ok = ok.defining(symbol);
    }
    ok = ok.defining(Symbol::new(
        "fmt_price",
        obscur_kit::okui_compiler::symbol::SymbolKind::Function,
    ));
    let result = ok.compile(&parse_okui("<span>{fmt_price(item.cents)}</span>"));
    assert!(
        !result.has_errors(),
        "the exported-only document compiles; got {:?}",
        result.diagnostics()
    );
}

// ----------------------------------------------------------------- §2.10 allowed

/// §2.10, Allowed: "- field access;", "- comparisons;", "- logical operators;",
/// "- Option/null checks;"
#[test]
fn the_four_allowed_expression_forms_compose_in_one_expression() {
    let source = "<If cond={user.is_some() && user.role.level >= 3 && !user.suspended}>\
                  <span class=\"gate\"/></If>";
    assert_eq!(
        diagnostics(source),
        Vec::<String>::new(),
        "an author following §2.10's own vocabulary must be able to write all of \
         it in one expression"
    );
    assert!(produces_artifact(source), "and the file compiles");
}

/// §2.10, Allowed: "- loop/template locals;"
#[test]
fn a_loop_local_does_not_leak_outside_the_loop_body() {
    let inside = "<div><For each={items} key={row}><span>{row.name}</span></For></div>";
    assert_eq!(
        diagnostics(inside),
        Vec::<String>::new(),
        "the loop body may read the local it binds"
    );

    // The same name used in a sibling element *outside* the `For` body. The
    // loop local is deliberately absent from this compilation's scope, since a
    // scope that pre-declares it cannot answer the question.
    let outside = "<div><For each={items} key={row}><span/></For>\
                   <span>{row.name}</span></div>";
    let messages: Vec<String> = {
        let mut compiler =
            Compiler::new("spec_ch2.okui").with_registry(ContractRegistry::builtins());
        compiler = compiler.defining(Symbol::state("items"));
        compiler
            .compile(&parse_okui(outside))
            .diagnostics()
            .iter()
            .map(CompileDiagnostic::to_string)
            .collect()
    };
    assert!(
        messages.iter().any(|m| m.contains("row")),
        "an unresolved-name diagnostic for `row` at the use site outside the \
         loop; got {messages:?}"
    );
}

// ------------------------------------------------------------------------ §2.9

/// §2.9: "- no embedded general-purpose scripting language;" and "- no business
/// logic."
#[test]
fn a_block_of_statements_is_not_permitted_in_okui() {
    let source = "<span>{ let n = 1; for i in 0..n { } return n; }</span>";
    let messages = diagnostics(source);
    assert!(
        !messages.is_empty(),
        "a block of imperative statements must be refused"
    );
    assert!(
        messages
            .iter()
            .any(|m| m.contains("statement") || m.contains("scripting")),
        "the diagnostic must state that statements/scripting are not permitted \
         in `.okui`; got {messages:?}"
    );
    assert!(!produces_artifact(source), "no artifact is produced");
}

// ----------------------------------------------------------------------- §2.11

/// §2.11: "- mandatory stable `key`;"
#[test]
fn a_for_without_a_key_is_a_compile_error() {
    let source = "<div><For each={items}><span/></For></div>";
    let messages = diagnostics(source);
    assert!(
        messages.iter().any(|m| m.contains("key")),
        "compilation must fail with a diagnostic that names the missing `key`; \
         got {messages:?}"
    );
    assert!(!produces_artifact(source), "no output artifact is produced");
}

/// §2.11: "- mandatory stable `key`;" (the duplicate half — silently losing a
/// row is far worse than a diagnostic)
#[test]
fn duplicate_keys_are_diagnosed_rather_than_collapsing_the_list() {
    // Three items, two of which key to the same value.
    let items = [1u64, 2, 1];
    let v = obscur_kit::ui_macros::okui! {
        <div>
            <For each={items} as={item} key={item}>
                <span class="row"/>
            </For>
        </div>
    };

    let mut rt = support::small_runtime();
    let reported = std::rc::Rc::new(std::cell::RefCell::new(Vec::<String>::new()));
    rt.set_diagnostic_sink(Box::new(CollectingSink(std::rc::Rc::clone(&reported))));
    support::commit(&mut rt, &v);

    assert_eq!(
        support::count_class(&rt, "row"),
        3,
        "rendering does not silently collapse the list to 2 rows"
    );
    let messages = reported.borrow().clone();
    assert!(
        messages.iter().any(|m| m.contains("key")),
        "a diagnostic identifying the duplicated key value must be reported; \
         got {messages:?}"
    );
}

/// A diagnostic sink that records every message it is handed.
#[derive(Debug)]
struct CollectingSink(std::rc::Rc<std::cell::RefCell<Vec<String>>>);

impl obscur_kit::diagnostics::DiagnosticSink for CollectingSink {
    fn emit(&self, diagnostic: &obscur_kit::diagnostics::Diagnostic) {
        self.0.borrow_mut().push(format!("{diagnostic:?}"));
    }
}

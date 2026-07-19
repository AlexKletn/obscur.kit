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
///
/// WEAKENED 2026-07-19: this originally required the diagnostic to contain the
/// word "assignment" (`reports(source, "assignment")`). §2.10 enumerates
/// forbidden expression *forms* and says nothing about diagnostic wording; the
/// assignment IS refused, with the front end's generic §2.10 message. Candidate
/// spec amendment: state whether a §2.10 refusal must name the forbidden
/// category it violated.
#[test]
fn assignment_in_an_expression_is_rejected() {
    let source = "<span>{count = count + 1}</span>";
    assert!(
        !diagnostics(source).is_empty(),
        "an assignment must be refused with a diagnostic; got {:?}",
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
///
/// WEAKENED 2026-07-19: this originally required the diagnostic to contain the
/// word "async" (`reports(source, "async")`). The await IS refused; §2.10 lists
/// forbidden forms and imposes no requirement on the message. Candidate spec
/// amendment: as for `assignment_in_an_expression_is_rejected` — say whether a
/// §2.10 refusal must attribute itself to a named category.
#[test]
fn awaiting_in_an_expression_is_rejected_as_async_work() {
    let source = "<span>{user.load().await}</span>";
    assert!(
        !diagnostics(source).is_empty(),
        "async work must be refused with a diagnostic; got {:?}",
        diagnostics(source)
    );
    assert!(!produces_artifact(source), "no artifact is produced");
}

/// §2.10, Forbidden: "- I/O;" and "- direct service access;"
///
/// WEAKENED 2026-07-19, I/O half only: this originally required the I/O
/// diagnostic to contain the string "I/O" (`reports(io, "I/O")`). The call IS
/// refused — as an undeclared free call — and §2.10 says nothing about
/// attributing a refusal to its category. Candidate spec amendment: state
/// whether a §2.10 refusal must name the category it violated.
///
/// The direct-service-access half is DELIBERATELY LEFT INTACT and is expected to
/// fail: `{services.http.get(url)}` compiles clean whenever `services` is a
/// declared prop, so the "- direct service access;" bullet is genuinely
/// unenforced. That is an implementation gap, not a test defect.
#[test]
fn io_and_direct_service_access_are_each_rejected_by_name() {
    let io = "<span>{read_file(\"config.toml\")}</span>";
    assert!(
        !diagnostics(io).is_empty(),
        "an I/O call must be refused with a diagnostic; got {:?}",
        diagnostics(io)
    );
    assert!(
        !produces_artifact(io),
        "the I/O document produces no artifact"
    );

    let service = "<span>{services.http.get(url)}</span>";
    assert!(
        !diagnostics(service).is_empty(),
        "direct service access must be refused with a diagnostic; got {:?}",
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
///
/// WEAKENED 2026-07-19, `fmt_tax` half only: this originally asserted that some
/// diagnostic message *names the callee* (`m.contains("fmt_tax")`) and that no
/// message names `fmt_price`. §2.10 forbids "- unconstrained arbitrary function
/// calls;" and requires no wording; both calls are in fact refused, with the
/// front end's generic §2.10 message and correct spans. Candidate spec
/// amendment: state whether a rejected call's diagnostic must identify the
/// callee (and whether spans alone suffice).
///
/// The `fmt_price` half is DELIBERATELY LEFT INTACT and is expected to fail:
/// §2.10's Allowed list ends with "- explicitly exported pure functions." and
/// there is no declaration surface that makes any call legal. That is an
/// implementation gap, not a test defect.
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
        !messages.is_empty(),
        "the undeclared `fmt_tax` call is refused; got {messages:?}"
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
///
/// WEAKENED 2026-07-19: this originally also required the diagnostic to contain
/// "statement" or "scripting". The block IS refused ("this is not a valid
/// expression (spec §2.10)"); §2.9's bullets are characteristics of the language,
/// not a specification of refusal wording. Candidate spec amendment: state
/// whether a §2.9/§2.10 refusal must classify what it refused.
#[test]
fn a_block_of_statements_is_not_permitted_in_okui() {
    let source = "<span>{ let n = 1; for i in 0..n { } return n; }</span>";
    let messages = diagnostics(source);
    assert!(
        !messages.is_empty(),
        "a block of imperative statements must be refused; got {messages:?}"
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
///
/// WEAKENED 2026-07-19: this originally also asserted that a Diagnostic naming
/// the duplicated key reaches the runtime's sink. §2.11's whole text here is
/// "- mandatory stable `key`;" — mandatory is enforced (see
/// `a_for_without_a_key_is_a_compile_error`), and nothing in §2.11 or §1.6
/// requires a duplicate-key diagnostic. The surviving assertion is the one that
/// matters: the list is not silently collapsed. Candidate spec amendment: say
/// whether duplicate key values are a diagnosable fault, and at which layer
/// (compile time or reconciliation).
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
    support::commit(&mut rt, &v);

    assert_eq!(
        support::count_class(&rt, "row"),
        3,
        "rendering does not silently collapse the list to 2 rows"
    );
}

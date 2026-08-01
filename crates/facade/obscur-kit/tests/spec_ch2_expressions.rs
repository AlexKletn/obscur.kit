#![doc = include_str!("spec_ch2_expressions.doc/_module.md")]
#![forbid(unsafe_code)]

#[path = "spec_ch2_support/mod.rs"]
mod support;

use obscur_kit::okui_compiler::compile::{CompileDiagnostic, Compiler, ContractRegistry};
use obscur_kit::okui_compiler::symbol::Symbol;
use support::{okui_diagnostics_with, parse_okui};

#[doc = include_str!("spec_ch2_expressions.doc/scope.md")]
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

#[doc = include_str!("spec_ch2_expressions.doc/diagnostics.md")]
fn diagnostics(source: &str) -> Vec<String> {
    okui_diagnostics_with(source, ContractRegistry::builtins(), &scope())
}

#[doc = include_str!("spec_ch2_expressions.doc/reports.md")]
fn reports(source: &str, needle: &str) -> bool {
    diagnostics(source).iter().any(|d| d.contains(needle))
}

#[doc = include_str!("spec_ch2_expressions.doc/produces_artifact.md")]
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

#[doc = include_str!("spec_ch2_expressions.doc/assignment_in_an_expression_is_rejected.md")]
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

#[doc = include_str!("spec_ch2_expressions.doc/a_mutating_method_call_is_rejected_as_mutation.md")]
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

#[doc = include_str!("spec_ch2_expressions.doc/awaiting_in_an_expression_is_rejected_as_async_work.md")]
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

#[doc = include_str!("spec_ch2_expressions.doc/io_and_direct_service_access_are_each_rejected_by_name.md")]
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

#[doc = include_str!("spec_ch2_expressions.doc/an_unexported_function_call_is_rejected_and_an_exported_one_is_not.md")]
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

#[doc = include_str!("spec_ch2_expressions.doc/the_four_allowed_expression_forms_compose_in_one_expression.md")]
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

#[doc = include_str!("spec_ch2_expressions.doc/a_loop_local_does_not_leak_outside_the_loop_body.md")]
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

#[doc = include_str!("spec_ch2_expressions.doc/a_block_of_statements_is_not_permitted_in_okui.md")]
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

#[doc = include_str!("spec_ch2_expressions.doc/a_for_without_a_key_is_a_compile_error.md")]
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

#[doc = include_str!("spec_ch2_expressions.doc/duplicate_keys_are_diagnosed_rather_than_collapsing_the_list.md")]
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

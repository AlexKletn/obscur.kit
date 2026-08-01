#![doc = include_str!("spec_ch9_okui.doc/_module.md")]
#![forbid(unsafe_code)]

#[path = "spec_ch9_support/mod.rs"]
mod support;

use obscur_kit::okui_compiler::compile::{CompileResult, Compiler, ContractRegistry};
use obscur_kit::okui_compiler::symbol::{Symbol, SymbolKind};
use obscur_kit::okui_syntax::ast::Document;
use obscur_kit::okui_syntax::parser::Parser;

#[doc = include_str!("spec_ch9_okui.doc/compile_with.md")]
fn compile_with(source: &str, symbols: Vec<Symbol>) -> CompileResult {
    let parsed = Parser::new().parse(source);
    let document = Document::cast(parsed.tree().root().clone()).expect("root casts to Document");
    let mut compiler = Compiler::new("spec_ch9.okui").with_registry(ContractRegistry::builtins());
    for symbol in symbols {
        compiler = compiler.defining(symbol);
    }
    compiler.compile(&document)
}

#[doc = include_str!("spec_ch9_okui.doc/compile_okui.md")]
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

#[doc = include_str!("spec_ch9_okui.doc/messages.md")]
fn messages(result: &CompileResult) -> String {
    result
        .diagnostics()
        .iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}

#[doc = include_str!("spec_ch9_okui.doc/assignment_inside_an_interpolation_is_a_compile_error.md")]
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

#[doc = include_str!("spec_ch9_okui.doc/only_explicitly_exported_pure_functions_may_be_called.md")]
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

#[doc = include_str!("spec_ch9_okui.doc/io_async_and_direct_service_access_are_each_refused_by_category.md")]
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

#[doc = include_str!("spec_ch9_okui.doc/rendering_the_same_template_twice_is_byte_identical_and_side_effect_free.md")]
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

#[doc = include_str!("spec_ch9_okui.doc/unknown_events_attributes_and_elements_are_compile_errors.md")]
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

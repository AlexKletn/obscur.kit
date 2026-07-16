//! End-to-end: a §2.12 slot contract is **enforced**, on both front ends
//! (spec §2.12, §2.13).
//!
//! Slots existed only as types. `SlotContract`/`SlotCardinality` had no
//! production consumer, nothing parsed a `slot="…"`, and no document could break
//! a rule that nothing checked. This file drives the whole chain that now exists:
//! a component declares its slots with `#[derive(Contract)]`, the declaration
//! crosses into the compiler's string contract, and a document assigning children
//! to those slots is checked against it — through the **real** `Compiler` for the
//! `.okui` file path, and through the **real** `okui!` macro for the inline one.
//!
//! # What is checked, and what is not
//!
//! What is checked is the *assignment*: a `slot="…"` must name a declared slot,
//! satisfy that slot's own child rule, and the slot's cardinality must hold.
//!
//! What does **not** happen is *projection*. §2.12's slots have no template to be
//! projected into: a component's `render(ctx, props)` never receives children, and
//! a component tag lowers to a `ViewNode::element(COMPONENT_ID)` rather than to a
//! call of anything, so there is no "where the slot is" for content to move to.
//! The assignment is therefore a compile-time construct — like a binding being a
//! Rust path — and `a_slot_assignment_is_not_a_prop_the_tree_carries` below pins
//! what does reach the tree.

#![forbid(unsafe_code)]

use obscur_kit::okui_compiler::compile::{CompileDiagnostic, Compiler, ContractRegistry};
use obscur_kit::okui_compiler::symbol::Symbol;
use obscur_kit::okui_syntax::ast::Document;
use obscur_kit::okui_syntax::parser::Parser;
use obscur_kit::ui_macros::{Contract, okui};
use obscur_kit::view::view::{View, ViewNode};

/// A component declaring three §2.12 slots: one mandatory, one unbounded, one
/// optional and restricted to `Button`s.
#[derive(Contract)]
#[contract(name = "Card")]
#[contract(slot(name = "header", cardinality = one))]
#[contract(slot(name = "body", cardinality = many))]
#[contract(slot(name = "actions", cardinality = optional, child = "Button"))]
struct CardProps {
    /// A prop, so the surface is not slots-only. Read by the derived
    /// `FromProps`, not by this file.
    #[allow(dead_code)]
    title: String,
}

/// Compiles `source` through the real pipeline with `Card`'s derived surface
/// registered, returning every diagnostic's message.
///
/// `ready` and `rows` are defined so a document may condition and iterate; the
/// name resolver is a different pass and its verdict is not what is under test.
fn diagnostics(source: &str) -> Vec<String> {
    let document =
        Document::cast(Parser::new().parse(source).tree().root().clone()).expect("a document");
    let registry = ContractRegistry::builtins().with_surface(&CardProps::contract_surface());
    Compiler::new("card.okui")
        .with_registry(registry)
        .defining(Symbol::state("ready"))
        .defining(Symbol::state("rows"))
        .defining(Symbol::state("which"))
        .compile(&document)
        .diagnostics()
        .iter()
        .map(CompileDiagnostic::to_string)
        .collect()
}

/// Whether compiling `source` reports a diagnostic containing `needle`.
fn reports(source: &str, needle: &str) -> bool {
    diagnostics(source).iter().any(|d| d.contains(needle))
}

#[test]
fn a_document_that_keeps_the_slot_contract_compiles_clean() {
    // The control. Every assertion below is worthless if the legal document is
    // also rejected.
    assert_eq!(
        diagnostics(
            r#"<Card title="Hi">
                 <h1 slot="header"/>
                 <p slot="body"/>
                 <p slot="body"/>
                 <Button label="OK" slot="actions"/>
               </Card>"#
        ),
        Vec::<String>::new()
    );
}

#[test]
fn a_slot_name_the_component_does_not_declare_is_a_compile_error() {
    assert!(
        reports(
            r#"<Card title="Hi"><h1 slot="headr"/><p slot="body"/></Card>"#,
            "unknown slot 'headr' on component 'Card'"
        ),
        "{:?}",
        diagnostics(r#"<Card title="Hi"><h1 slot="headr"/><p slot="body"/></Card>"#)
    );
}

#[test]
fn cardinality_is_enforced_in_both_directions() {
    assert!(
        reports(
            r#"<Card title="Hi"><h1 slot="header"/><h2 slot="header"/></Card>"#,
            "slot 'header' of component 'Card' accepts exactly one child, found 2"
        ),
        "two children over-fill an `ExactlyOne` slot"
    );
    assert!(
        reports(
            r#"<Card title="Hi"><p slot="body"/></Card>"#,
            "requires exactly one child in slot 'header'"
        ),
        "and an unfilled `ExactlyOne` slot is the other half of `exactly`"
    );
    assert!(
        reports(
            r#"<Card title="Hi">
                 <h1 slot="header"/>
                 <Button label="a" slot="actions"/>
                 <Button label="b" slot="actions"/>
               </Card>"#,
            "slot 'actions' of component 'Card' accepts at most one child, found 2"
        ),
        "an `Optional` slot takes one child, not two"
    );
}

#[test]
fn a_slots_own_child_rule_is_enforced() {
    assert!(
        reports(
            r#"<Card title="Hi"><h1 slot="header"/><Input slot="actions"/></Card>"#,
            "'Input' is not an allowed child of slot 'actions' on component 'Card'"
        ),
        "`actions` admits `Button` and nothing else"
    );
}

#[test]
fn a_slot_assignment_on_a_component_declaring_none_names_nothing() {
    // Not a typo but a category error, and it reads as one. No shipped widget
    // declares a slot, so this is what every `slot="…"` in an ordinary document
    // now hits.
    assert!(
        reports(
            r#"<Image src="a.png"><h1 slot="header"/></Image>"#,
            "component 'Image' declares no slots"
        ),
        "{:?}",
        diagnostics(r#"<Image src="a.png"><h1 slot="header"/></Image>"#)
    );
}

#[test]
fn a_member_behind_control_flow_is_still_the_components_member() {
    // `<If>` and `<For>` splice their bodies into the enclosing child list, so a
    // `slot=` inside one is addressed to `Card`. Checking direct children only
    // would let the typo through — the construct would be enforced everywhere
    // except where documents actually put it.
    assert!(
        reports(
            r#"<Card title="Hi">
                 <h1 slot="header"/>
                 <For each={rows} as={row} key={row}><p slot="bdy"/></For>
               </Card>"#,
            "unknown slot 'bdy' on component 'Card'"
        ),
        "a `<For>` body's member is the loop's parent's member"
    );

    // …and the count it contributes is unknown, so it must not be *accused*: one
    // `<If>`-guarded header may be present or absent, and both are legal.
    assert_eq!(
        diagnostics(r#"<Card title="Hi"><If cond={ready}><h1 slot="header"/></If></Card>"#),
        Vec::<String>::new(),
        "a conditional member is never accused of a count it may not reach"
    );
}

#[test]
fn a_slot_name_that_only_exists_at_runtime_is_refused() {
    assert!(
        reports(
            r#"<Card title="Hi"><h1 slot={which}/><p slot="body"/></Card>"#,
            "must be a literal name, not an expression"
        ),
        "a slot is matched against a declaration at compile time"
    );
}

#[test]
fn the_macro_front_end_reaches_the_same_verdict_it_can_reach() {
    // `okui!` ran *no* contract validation at all; it does now, against the
    // built-ins and widgets compiled into it. What it cannot see is `Card`'s
    // surface — `CardProps::contract_surface()` is a function that runs when this
    // test runs, and the macro ran while this test was being compiled. So the
    // shared rule is reachable from here only through a tag the macro knows.
    let view: View = okui! { <Button label="OK"/> };
    assert!(matches!(view.root().nodes()[0], ViewNode::Element { .. }));

    // The negative half — `okui! { <Button laebl="OK"/> }` — cannot be written
    // here, because a rejected macro is a *compile* error in this file. It is
    // asserted where it can be: `DslMacro::to_document`'s own unit tests
    // (`the_primary_authoring_surface_now_enforces_a_contract_at_all`).
}

#[test]
fn a_slot_assignment_is_not_a_prop_the_tree_carries() {
    // What reaches the runtime, pinned. `slot` is consumed by the contract check
    // and lowers to an ordinary property, because nothing downstream routes it:
    // the tree has one anonymous child list and no component reads it. Anything
    // stronger here — a `ViewNode::slot` field, say — would be written by codegen
    // and read by nobody.
    let view: View = okui! { <container><span slot="header"/></container> };
    let ViewNode::Element { children, .. } = &view.root().nodes()[0] else {
        panic!("a container lowers to an element");
    };
    let ViewNode::Element { props, .. } = &children.nodes()[0] else {
        panic!("its child lowers to an element");
    };
    let slot = obscur_kit::identity::InternedSymbol::from_raw(
        obscur_kit::ui_ir::name_binding::attr_symbol("slot").raw(),
    );
    assert!(
        props.iter().any(|p| p.name() == slot),
        "the assignment reaches the tree as an ordinary property, and no further"
    );
}

#![doc = include_str!("spec_ch2_contracts.doc/_module.md")]
#![forbid(unsafe_code)]

#[path = "spec_ch2_support/mod.rs"]
mod support;

use obscur_kit::identity::{ComponentId, InternedSymbol};
use obscur_kit::okui_compiler::compile::{CompileDiagnostic, Compiler, ContractRegistry};
use obscur_kit::okui_compiler::symbol::Symbol;
use obscur_kit::ui_macros::Contract;
use obscur_kit::view::component::{
    Component, ComponentContext, ComponentContract, ComponentFactory, ComponentInstance,
    ComponentRegistration, ComponentVersion, ErasedComponent, InstanceId,
};
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};
use support::{
    by_class, children, color_of, commit, commit_styled, count_class, one_by_class, parse_okui,
    root_children, small_runtime,
};

// ---------------------------------------------------------- the contracts

#[doc = include_str!("spec_ch2_contracts.doc/MyButtonProps.md")]
#[allow(dead_code)]
#[derive(Contract)]
#[contract(name = "MyButton")]
struct MyButtonProps {
    #[doc = include_str!("spec_ch2_contracts.doc/label.md")]
    label: String,
    #[doc = include_str!("spec_ch2_contracts.doc/variant.md")]
    #[contract(default)]
    variant: Option<String>,
    #[doc = include_str!("spec_ch2_contracts.doc/pressed.md")]
    #[contract(event)]
    pressed: Option<String>,
}

#[doc = include_str!("spec_ch2_contracts.doc/DialogProps.md")]
#[derive(Contract)]
#[contract(name = "Dialog")]
#[contract(slot(name = "title", cardinality = one))]
#[contract(slot(name = "actions", cardinality = many))]
struct DialogProps {
    #[doc = include_str!("spec_ch2_contracts.doc/id.md")]
    #[allow(dead_code)]
    id: String,
}

#[doc = include_str!("spec_ch2_contracts.doc/MenuProps.md")]
#[derive(Contract)]
#[contract(name = "Menu")]
#[contract(slot(name = "items", cardinality = many, child = "MenuItem"))]
#[contract(slot(name = "items", cardinality = many, child = "MenuSeparator"))]
struct MenuProps {
    #[doc = include_str!("spec_ch2_contracts.doc/id_2.md")]
    #[allow(dead_code)]
    id: String,
}

#[doc = include_str!("spec_ch2_contracts.doc/diagnostics_for.md")]
fn diagnostics_for(
    source: &str,
    surfaces: &[obscur_kit::view::contract::ComponentSurface],
) -> Vec<String> {
    let mut registry = ContractRegistry::builtins();
    for surface in surfaces {
        registry.register_surface(surface);
    }
    let mut compiler = Compiler::new("spec_ch2.okui").with_registry(registry);
    for symbol in [
        Symbol::state("handle_submit"),
        Symbol::state("ready"),
        Symbol::state("items"),
    ] {
        compiler = compiler.defining(symbol);
    }
    compiler
        .compile(&parse_okui(source))
        .diagnostics()
        .iter()
        .map(CompileDiagnostic::to_string)
        .collect()
}

#[doc = include_str!("spec_ch2_contracts.doc/button_diagnostics.md")]
fn button_diagnostics(source: &str) -> Vec<String> {
    diagnostics_for(source, &[MyButtonProps::contract_surface()])
}

// ------------------------------------------------------------------- §2.13

#[doc = include_str!("spec_ch2_contracts.doc/an_unknown_attribute_is_a_compile_error.md")]
#[test]
fn an_unknown_attribute_is_a_compile_error() {
    let source = r#"<MyButton label="Go" varaint="primary"/>"#;
    let messages = button_diagnostics(source);
    assert!(
        messages
            .iter()
            .any(|m| m.contains("varaint") && m.contains("MyButton")),
        "a diagnostic must name `varaint` as unknown on `MyButton`; got {messages:?}"
    );
}

#[doc = include_str!("spec_ch2_contracts.doc/an_unknown_event_is_a_compile_error.md")]
#[test]
fn an_unknown_event_is_a_compile_error() {
    for (source, event) in [
        (
            r#"<MyButton label="Go" on:presed={handle_submit}/>"#,
            "presed",
        ),
        (
            r#"<MyButton label="Go" on:click={handle_submit}/>"#,
            "click",
        ),
    ] {
        let messages = button_diagnostics(source);
        assert!(
            messages
                .iter()
                .any(|m| m.contains(event) && m.contains("MyButton")),
            "a diagnostic must name the unknown event `{event}` and the \
             component; got {messages:?}"
        );
    }
}

#[doc = include_str!("spec_ch2_contracts.doc/a_string_named_handler_is_refused_even_when_the_method_exists.md")]
#[test]
fn a_string_named_handler_is_refused_even_when_the_method_exists() {
    // `handle_submit` is defined in the compilation scope above — it exists.
    let exists = r#"<MyButton label="Go" on:pressed="handle_submit"/>"#;
    let missing = r#"<MyButton label="Go" on:pressed="no_such_method"/>"#;
    for source in [exists, missing] {
        let messages = button_diagnostics(source);
        assert!(
            messages
                .iter()
                .any(|m| m.contains("string") || m.contains("String")),
            "a diagnostic must state that string method dispatch is not \
             supported, for `{source}`; got {messages:?}"
        );
    }
}

#[doc = include_str!("spec_ch2_contracts.doc/a_contract_declared_default_is_applied_rather_than_the_type_s_zero_value.md")]
#[test]
fn a_contract_declared_default_is_applied_rather_than_the_type_s_zero_value() {
    // The component reports back what it observed.
    let mut rt = small_runtime();
    rt.register_component(ComponentRegistration::new(
        Box::new(ObserverFactory),
        ComponentVersion::INITIAL,
    ))
    .expect("Observer registers");

    // The contract declares both props as defaulted, not required.
    let contract = ObserverProps::contract();
    for name in ["variant", "count"] {
        let symbol = obscur_kit::ui_ir::name_binding::attr_symbol(name);
        let prop = contract
            .props
            .iter()
            .find(|p| p.name == symbol)
            .unwrap_or_else(|| panic!("the contract declares `{name}`"));
        assert!(
            prop.has_default,
            "`{name}` is recorded in the contract as carrying a default"
        );
        assert!(
            !prop.required,
            "and is therefore not required of the caller"
        );
    }

    // A usage supplying neither `variant` nor `count` still instantiates.
    let usage = View::from_fragment(ViewFragment::from_nodes(vec![ViewNode::element(
        ObserverProps::contract().id,
    )]));
    commit(&mut rt, &usage);

    assert_eq!(
        support::all_text(&rt).len(),
        1,
        "omitting a defaulted prop is not an error — the component renders"
    );
}

#[doc = include_str!("spec_ch2_contracts.doc/ObserverProps.md")]
#[derive(Contract)]
#[contract(name = "Observer")]
struct ObserverProps {
    #[doc = include_str!("spec_ch2_contracts.doc/variant_2.md")]
    #[contract(default)]
    variant: Option<String>,
    #[doc = include_str!("spec_ch2_contracts.doc/count.md")]
    #[contract(default)]
    count: Option<i64>,
}

struct Observer;

impl Component for Observer {
    type Props = ObserverProps;

    fn render(&mut self, _ctx: &mut ComponentContext<'_>, props: &Self::Props) -> View {
        View::single(ViewNode::text(format!(
            "variant={} count={}",
            props.variant.clone().unwrap_or_default(),
            props.count.unwrap_or_default()
        )))
    }
}

#[derive(Debug)]
struct ObserverFactory;

impl ComponentFactory for ObserverFactory {
    fn component_id(&self) -> ComponentId {
        ObserverProps::contract().id
    }
    fn contract(&self) -> ComponentContract {
        ObserverProps::contract()
    }
    fn create_instance(&self, instance: InstanceId) -> ComponentInstance {
        ComponentInstance::new(instance, ObserverProps::contract().id)
    }
    fn create_component(&self) -> Box<dyn ErasedComponent> {
        Box::new(Observer)
    }
}

// ------------------------------------------------------------------- §2.12

#[doc = include_str!("spec_ch2_contracts.doc/slot_cardinality_is_enforced_in_both_directions.md")]
#[test]
fn slot_cardinality_is_enforced_in_both_directions() {
    let surfaces = [DialogProps::contract_surface()];

    let twice = r#"<Dialog id="d"><h1 slot="title"/><h2 slot="title"/></Dialog>"#;
    assert!(
        diagnostics_for(twice, &surfaces)
            .iter()
            .any(|m| m.contains("title")),
        "(a) `title` given twice fails, naming the slot and the cardinality \
         violation; got {:?}",
        diagnostics_for(twice, &surfaces)
    );

    let omitted = r#"<Dialog id="d"/>"#;
    assert!(
        diagnostics_for(omitted, &surfaces)
            .iter()
            .any(|m| m.contains("title")),
        "(b) `title` omitted fails, naming the missing required slot; got {:?}",
        diagnostics_for(omitted, &surfaces)
    );

    let three_actions = r#"<Dialog id="d"><h1 slot="title"/>
        <button slot="actions"/><button slot="actions"/><button slot="actions"/></Dialog>"#;
    assert_eq!(
        diagnostics_for(three_actions, &surfaces),
        Vec::<String>::new(),
        "(c) a zero-or-more slot accepts three children"
    );
}

#[doc = include_str!("spec_ch2_contracts.doc/a_slot_s_allowed_children_are_enforced.md")]
#[test]
fn a_slot_s_allowed_children_are_enforced() {
    let surfaces = [MenuProps::contract_surface()];

    let allowed = r#"<Menu id="m"><MenuItem slot="items"/></Menu>"#;
    assert_eq!(
        diagnostics_for(allowed, &surfaces),
        Vec::<String>::new(),
        "a `MenuItem` in the slot compiles"
    );

    let disallowed = r#"<Menu id="m"><div slot="items"/></Menu>"#;
    let messages = diagnostics_for(disallowed, &surfaces);
    assert!(
        messages
            .iter()
            .any(|m| m.contains("div") && m.contains("items")),
        "a diagnostic must name the disallowed child type and the slot; got \
         {messages:?}"
    );
}

#[doc = include_str!("spec_ch2_contracts.doc/slot_content_projects_to_its_template_position_not_its_usage_order.md")]
#[test]
fn slot_content_projects_to_its_template_position_not_its_usage_order() {
    let mut rt = small_runtime();
    rt.register_component(ComponentRegistration::new(
        Box::new(CardFactory),
        ComponentVersion::INITIAL,
    ))
    .expect("Card registers");

    // The usage writes `actions` FIRST, before the body content — the reverse of
    // the template's order.
    let usage = View::from_fragment(ViewFragment::from_nodes(vec![
        ViewNode::element(CardProps::contract().id).with_children(ViewFragment::from_nodes(vec![
            classed("act", "actions"),
            classed("bod", "body"),
        ])),
    ]));
    commit(&mut rt, &usage);

    let footer = one_by_class(&rt, "region-actions").expect("the Footer region is retained");
    let body = one_by_class(&rt, "region-body").expect("the Body region is retained");
    let act = one_by_class(&rt, "act").expect("the actions content is retained");
    let bod = one_by_class(&rt, "bod").expect("the body content is retained");

    assert!(
        descends_from(&rt, act, footer),
        "the `actions` content is a descendant of Footer regardless of the order \
         the author wrote it"
    );
    assert!(
        descends_from(&rt, bod, body),
        "the default children are descendants of Body"
    );
    // The usage-site source order does not appear anywhere in the tree.
    let card_children = children(
        &rt,
        one_by_class(&rt, "region-body").map_or(body, |_| {
            rt.tree()
                .get(body)
                .and_then(|n| n.logical_parent().parent())
                .expect("the Body region has a parent")
        }),
    );
    assert!(
        !card_children.is_empty(),
        "the card mounted a subtree of its own"
    );
}

#[doc = include_str!("spec_ch2_contracts.doc/a_default_slot_and_a_named_slot_partition_the_children.md")]
#[test]
fn a_default_slot_and_a_named_slot_partition_the_children() {
    let mut rt = small_runtime();
    rt.register_component(ComponentRegistration::new(
        Box::new(PanelFactory),
        ComponentVersion::INITIAL,
    ))
    .expect("Panel registers");

    // Three children: one assigned to `header`, two unassigned.
    let usage =
        View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(PanelProps::contract().id).with_children(ViewFragment::from_nodes(
                vec![classed("h", "header"), classed("d1", ""), classed("d2", "")],
            )),
        ]));
    commit(&mut rt, &usage);

    let header = one_by_class(&rt, "region-header").expect("the header region is retained");

    assert_eq!(
        children(&rt, header).len(),
        1,
        "the `header` slot receives exactly 1 child"
    );
    let header_classes: Vec<String> = children(&rt, header)
        .iter()
        .filter_map(|id| rt.tree().get(*id))
        .filter_map(|n| n.attributes().classes().first().cloned())
        .map(|c| c.to_string())
        .collect();
    assert_eq!(
        header_classes,
        ["h"].map(str::to_owned).to_vec(),
        "and it is the child that asked for `header`, not whichever came first"
    );
    assert_eq!(
        count_class(&rt, "h"),
        1,
        "the header child is projected once, not duplicated"
    );

    // RESTORED 2026-08-02, with §2.12 amended to state the default slot. The
    // half below was removed on 2026-07-19 because the spec listed named slots
    // only and this case's `ViewNode::slot("")` was the test inventing one.
    let default = one_by_class(&rt, "region-default").expect("the default region is retained");
    let default_classes: Vec<String> = children(&rt, default)
        .iter()
        .filter_map(|id| rt.tree().get(*id))
        .filter_map(|n| n.attributes().classes().first().cloned())
        .map(|c| c.to_string())
        .collect();
    assert_eq!(
        default_classes,
        ["d1", "d2"].map(str::to_owned).to_vec(),
        "the two children naming no slot reach the default slot, in authored \
         order — and the one that asked for `header` does not"
    );
}

// DELETED 2026-07-19: `a_template_may_not_hold_state_or_a_lifecycle_hook`.
//
// It asserted that compiling
//   <template name="Row"><state name="count" initial="0"/>
//                        <on:mount handler={ready}/><span/></template>
// produces one diagnostic naming state/stateless and another naming
// mount/lifecycle.
//
// §2.12's sentence is "Templates are stateless and lifecycle-free." — a property
// of the template *concept*, not a requirement that `.okui` expose a
// `<template name=…>` tag able to carry `<state>` and `<on:mount>` children. No
// such syntax exists, so the document is three unknown elements and produces no
// diagnostics at all (§2.13 makes unknown *attributes, events and string method
// names* compile errors, not unknown elements). Weakened to what the spec
// actually says, nothing observable remains, so the case is deleted rather than
// left as decoration.
//
// Candidate spec amendment: if templates are to be an authorable `.okui`
// construct, say so, name the syntax, and state that a state or lifecycle
// declaration inside one is a compile error — then this case can be restored.

#[doc = include_str!("spec_ch2_contracts.doc/an_exported_style_part_crosses_the_scope_boundary_and_an_unexported_one_does_not.md")]
#[test]
fn an_exported_style_part_crosses_the_scope_boundary_and_an_unexported_one_does_not() {
    let mut rt = small_runtime();
    rt.register_component(ComponentRegistration::new(
        Box::new(PartCardFactory),
        ComponentVersion::INITIAL,
    ))
    .expect("PartCard registers");

    let usage = View::from_fragment(ViewFragment::from_nodes(vec![ViewNode::element(
        PartCardProps::contract().id,
    )]));
    commit_styled(
        &mut rt,
        "PartCard::part(label){color:red} .inner{color:blue}",
        &usage,
    );

    let label = one_by_class(&rt, "label").expect("the part-bearing element is retained");
    let inner = one_by_class(&rt, "inner").expect("the unexported element is retained");

    assert_eq!(
        color_of(&rt, label),
        Some(obscur_kit::style::property::StyleColor::rgba(
            1.0, 0.0, 0.0, 1.0
        )),
        "the part-targeted element's computed color is red"
    );
    assert_ne!(
        color_of(&rt, inner),
        Some(obscur_kit::style::property::StyleColor::rgba(
            0.0, 0.0, 1.0, 1.0
        )),
        "the outer `.inner` rule does not reach a scoped component internal"
    );
}

// --------------------------------------------------------------------- §2.8

#[doc = include_str!("spec_ch2_contracts.doc/a_prop_type_mismatch_is_reported_per_attribute_at_its_own_span.md")]
#[test]
fn a_prop_type_mismatch_is_reported_per_attribute_at_its_own_span() {
    // `label` is a string prop given an integer; `count` is an integer prop
    // given a string.
    let source = r#"<Typed label={5} count="3"/>"#;
    let messages = diagnostics_for(source, &[TypedProps::contract_surface()]);

    assert_eq!(
        messages.len(),
        2,
        "two diagnostics, one per attribute — not a single merged error that \
         names neither; got {messages:?}"
    );
    assert!(
        messages.iter().any(|m| m.contains("label")),
        "one names `label`; got {messages:?}"
    );
    assert!(
        messages.iter().any(|m| m.contains("count")),
        "one names `count`; got {messages:?}"
    );
}

#[doc = include_str!("spec_ch2_contracts.doc/TypedProps.md")]
#[allow(dead_code)]
#[derive(Contract)]
#[contract(name = "Typed")]
struct TypedProps {
    #[doc = include_str!("spec_ch2_contracts.doc/label_2.md")]
    label: String,
    #[doc = include_str!("spec_ch2_contracts.doc/count_2.md")]
    count: i64,
}

#[doc = include_str!("spec_ch2_contracts.doc/a_contract_diagnostic_carries_the_author_s_line_and_column.md")]
#[test]
fn a_contract_diagnostic_carries_the_author_s_line_and_column() {
    // The violation sits on line 12, column 5 of the authored file.
    let mut source = String::new();
    for _ in 0..11 {
        source.push_str("<div/>\n");
    }
    source.push_str("    <MyButton label=\"Go\" varaint=\"x\"/>\n");
    let offset = source
        .find("varaint")
        .expect("the fixture writes `varaint`");
    assert!(offset > 0, "the violation is not at the start of the file");

    let messages = button_diagnostics(&source);
    assert!(
        messages.iter().any(|m| m.contains(&offset.to_string())),
        "the diagnostic must locate the author's own `varaint` at byte {offset} \
         — not a location inside generated code, a macro-expansion site, or the \
         start of the file; got {messages:?}"
    );
}

// -------------------------------------------------------------- components

#[doc = include_str!("spec_ch2_contracts.doc/CardProps.md")]
#[derive(Contract)]
#[contract(name = "SpecCard")]
#[contract(slot(name = "body", cardinality = many))]
#[contract(slot(name = "actions", cardinality = many))]
struct CardProps {}

struct Card;

impl Component for Card {
    type Props = CardProps;
    fn render(&mut self, _ctx: &mut ComponentContext<'_>, _props: &Self::Props) -> View {
        View::from_fragment(ViewFragment::from_nodes(vec![
            region("body"),
            region("actions"),
        ]))
    }
}

#[doc = include_str!("spec_ch2_contracts.doc/PanelProps.md")]
#[derive(Contract)]
#[contract(name = "SpecPanel")]
#[contract(slot(name = "header", cardinality = one))]
#[contract(slot(default))]
struct PanelProps {}

struct Panel;

impl Component for Panel {
    type Props = PanelProps;
    fn render(&mut self, _ctx: &mut ComponentContext<'_>, _props: &Self::Props) -> View {
        View::from_fragment(ViewFragment::from_nodes(vec![
            region("header"),
            // The default slot: the outlet naming no slot.
            classed_with("region-default", vec![ViewNode::slot("")]),
        ]))
    }
}

#[doc = include_str!("spec_ch2_contracts.doc/PartCardProps.md")]
#[derive(Contract)]
#[contract(name = "PartCard")]
struct PartCardProps {}

struct PartCard;

impl Component for PartCard {
    type Props = PartCardProps;
    fn render(&mut self, _ctx: &mut ComponentContext<'_>, _props: &Self::Props) -> View {
        View::from_fragment(ViewFragment::from_nodes(vec![
            classed("label", ""),
            classed("inner", ""),
        ]))
    }
}

#[doc = include_str!("spec_ch2_contracts.doc/region.md")]
fn region(slot: &str) -> ViewNode {
    classed_with(&format!("region-{slot}"), vec![ViewNode::slot(slot)])
}

#[doc = include_str!("spec_ch2_contracts.doc/classed_with.md")]
fn classed_with(class: &str, kids: Vec<ViewNode>) -> ViewNode {
    ViewNode::element(obscur_kit::ui_ir::name_binding::CONTAINER)
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
        .with_children(ViewFragment::from_nodes(kids))
}

#[doc = include_str!("spec_ch2_contracts.doc/classed.md")]
fn classed(class: &str, slot: &str) -> ViewNode {
    let node = classed_with(class, vec![]);
    if slot.is_empty() {
        node
    } else {
        node.with_prop(ViewProperty::new(
            obscur_kit::view::view::SLOT,
            ViewValue::Text(slot.into()),
        ))
    }
}

#[doc = include_str!("spec_ch2_contracts.doc/descends_from.md")]
fn descends_from(
    rt: &obscur_kit::runtime::UiRuntime,
    node: obscur_kit::identity::NodeId,
    ancestor: obscur_kit::identity::NodeId,
) -> bool {
    let mut current = Some(node);
    while let Some(id) = current {
        if id == ancestor {
            return true;
        }
        current = rt.tree().get(id).and_then(|n| n.logical_parent().parent());
    }
    false
}

macro_rules! factory {
    ($factory:ident, $props:ident, $component:ident) => {
        #[derive(Debug)]
        struct $factory;

        impl ComponentFactory for $factory {
            fn component_id(&self) -> ComponentId {
                $props::contract().id
            }
            fn contract(&self) -> ComponentContract {
                $props::contract()
            }
            fn create_instance(&self, instance: InstanceId) -> ComponentInstance {
                ComponentInstance::new(instance, $props::contract().id)
            }
            fn create_component(&self) -> Box<dyn ErasedComponent> {
                Box::new($component)
            }
        }
    };
}

factory!(CardFactory, CardProps, Card);
factory!(PanelFactory, PanelProps, Panel);
factory!(PartCardFactory, PartCardProps, PartCard);

#[doc = include_str!("spec_ch2_contracts.doc/_uses.md")]
#[allow(dead_code)]
fn _uses(rt: &obscur_kit::runtime::UiRuntime) -> (usize, Vec<obscur_kit::identity::NodeId>) {
    (root_children(rt).len(), by_class(rt, "x"))
}

#[doc = include_str!("spec_ch2_contracts.doc/_EVENT.md")]
#[allow(dead_code)]
const _EVENT: InternedSymbol = obscur_kit::view::view::EVENT_CLICK;

#![doc = include_str!("okui_child_contracts.doc/_module.md")]
#![forbid(unsafe_code)]

use obscur_kit::okui_compiler::compile::{CompileDiagnostic, Compiler, ContractRegistry};
use obscur_kit::okui_syntax::ast::Document;
use obscur_kit::okui_syntax::parser::Parser;
use obscur_kit::ui_macros::{Contract, okui};
use obscur_kit::view::view::View;

#[doc = include_str!("okui_child_contracts.doc/diagnostics.md")]
fn diagnostics(source: &str) -> Vec<String> {
    compile_with(source, ContractRegistry::builtins())
}

#[doc = include_str!("okui_child_contracts.doc/compile_with.md")]
fn compile_with(source: &str, registry: ContractRegistry) -> Vec<String> {
    let document =
        Document::cast(Parser::new().parse(source).tree().root().clone()).expect("a document");
    Compiler::new("children.okui")
        .with_registry(registry)
        .compile(&document)
        .diagnostics()
        .iter()
        .map(CompileDiagnostic::to_string)
        .collect()
}

#[doc = include_str!("okui_child_contracts.doc/reports.md")]
fn reports(source: &str, needle: &str) -> bool {
    diagnostics(source).iter().any(|d| d.contains(needle))
}

#[test]
fn text_under_a_widget_that_synthesises_its_children_is_a_compile_error() {
    // The audit's own example. Note that there is no child *element* here at all:
    // catching it needs a claim about the position, which is what `ChildPolicy::None`
    // is and what a list of allowed tags could never be.
    assert!(
        reports(
            r#"<Button label="OK">OK</Button>"#,
            "component 'Button' takes no children"
        ),
        "{:?}",
        diagnostics(r#"<Button label="OK">OK</Button>"#)
    );
}

#[test]
fn an_element_child_under_a_leaf_names_the_offending_tag() {
    let source = r#"<Image src="a.png"><span/></Image>"#;
    assert!(
        reports(source, "takes no children") && reports(source, "'span'"),
        "the diagnostic names the tag and says what is wrong: {:?}",
        diagnostics(source)
    );
}

#[test]
fn every_widget_but_dialog_refuses_authored_children() {
    // The eleven whose content is synthesised, each with the props its contract
    // requires so the only diagnostic under test is the child one.
    for source in [
        r#"<Button label="x"><b/></Button>"#,
        r#"<Input><b/></Input>"#,
        r#"<Checkbox label="x"><b/></Checkbox>"#,
        r#"<Radio label="x"><b/></Radio>"#,
        r#"<Select><b/></Select>"#,
        r#"<List><b/></List>"#,
        r#"<Menu><b/></Menu>"#,
        r#"<Tabs><b/></Tabs>"#,
        r#"<Tree><b/></Tree>"#,
        r#"<Table><b/></Table>"#,
        r#"<Docking><b/></Docking>"#,
    ] {
        assert!(
            reports(source, "takes no children"),
            "`{source}` must not compile: {:?}",
            diagnostics(source)
        );
    }

    // `Dialog` is the one widget whose children ARE its content.
    assert_eq!(
        diagnostics(r#"<Dialog title="Hi"><p/>text</Dialog>"#),
        Vec::<String>::new(),
        "`Dialog` accepts authored children"
    );
}

#[test]
fn the_base_element_spelling_keeps_the_children_that_are_its_content() {
    // The line the fix had to get right: same `ComponentId`, different construct,
    // and only the component position is a leaf. This is the `facade-consumer`
    // fixture's markup.
    assert_eq!(
        diagnostics(r#"<button class="btn" on:click=increment><span>press</span></button>"#),
        Vec::<String>::new(),
        "an authored `<button>`'s children are the only thing that renders"
    );
    // …and the macro front end agrees, because both call the same rule.
    let view: View = okui! { <button class="btn"><span>"press"</span></button> };
    assert_eq!(view.root().nodes().len(), 1);
}

#[test]
fn a_conditional_does_not_hide_a_forbidden_child() {
    // `<If>` splices its body into the enclosing child list, so the `span` is
    // `Button`'s member. Unlike a slot's cardinality — where a bound that *may* be
    // satisfied is never reported — this is not a claim about a count: the markup's
    // only possible effect is nothing.
    assert!(
        reports(
            r#"<Button label="x"><If cond={ready}><span/></If></Button>"#,
            "takes no children"
        ),
        "{:?}",
        diagnostics(r#"<Button label="x"><If cond={ready}><span/></If></Button>"#)
    );
}

#[test]
fn a_container_and_a_text_run_still_take_anything() {
    // The regression guard. Only the leaves changed: the primitives that really are
    // containers must keep accepting content, or the enforcement is just breakage.
    for source in [
        r#"<Container><span>hi</span></Container>"#,
        r#"<Text>hello</Text>"#,
        r#"<Scroll><div>row</div></Scroll>"#,
        r#"<Portal target="root"><div/></Portal>"#,
        r#"<Layer opacity="0.5"><div/></Layer>"#,
        r#"<div class="app"><span>hi</span></div>"#,
    ] {
        assert_eq!(
            diagnostics(source),
            Vec::<String>::new(),
            "`{source}` is an ordinary container"
        );
    }
}

#[doc = include_str!("okui_child_contracts.doc/IconProps.md")]
#[derive(Contract)]
#[contract(name = "Icon")]
#[contract(children = none)]
struct IconProps {
    #[doc = include_str!("okui_child_contracts.doc/glyph.md")]
    #[allow(dead_code)]
    glyph: String,
}

#[test]
fn a_derived_leaf_contract_crosses_the_bridge_intact() {
    // The second half of the same gap: `ChildrenSurface::None` existed on the domain
    // surface and `#[derive(Contract)]` could emit it, but the bridge into the
    // compiler's string contract had to flatten it into `Any` — the string contract
    // could not hold it. A declared leaf was therefore unenforced all the way from
    // the derive to the checker.
    let registry = ContractRegistry::builtins().with_surface(&IconProps::contract_surface());
    let source = r#"<Icon glyph="star"><span/></Icon>"#;
    assert!(
        compile_with(source, registry.clone())
            .iter()
            .any(|d| d.contains("takes no children")),
        "{:?}",
        compile_with(source, registry.clone())
    );
    assert_eq!(
        compile_with(r#"<Icon glyph="star"/>"#, registry),
        Vec::<String>::new(),
        "the leaf itself is legal"
    );
}

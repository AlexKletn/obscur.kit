#![doc = include_str!("custom_properties.doc/_module.md")]
#![forbid(unsafe_code)]

use obscur_kit::bootstrap::CssPipeline;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::Size;
use obscur_kit::identity::{ComponentId, RuntimeId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::runtime::orchestration::NodeInspection;
use obscur_kit::style::computed::ComputedValue;
use obscur_kit::style::property::{PropertyId, StyleColor};
use obscur_kit::view::view::{CLASS, STYLE, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

#[doc = include_str!("custom_properties.doc/VIEWPORT.md")]
const VIEWPORT: Size = Size {
    width: 400.0,
    height: 300.0,
};

#[doc = include_str!("custom_properties.doc/RED.md")]
const RED: StyleColor = StyleColor::rgba(1.0, 0.0, 0.0, 1.0);
#[doc = include_str!("custom_properties.doc/GREEN.md")]
const GREEN: StyleColor = StyleColor::rgba(0.0, 1.0, 0.0, 1.0);
#[doc = include_str!("custom_properties.doc/BLUE.md")]
const BLUE: StyleColor = StyleColor::rgba(0.0, 0.0, 1.0, 1.0);

#[doc = include_str!("custom_properties.doc/el.md")]
fn el(class: &str, kids: Vec<ViewNode>) -> ViewNode {
    ViewNode::element(ComponentId::from_raw(80))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
        .with_children(ViewFragment::from_nodes(kids))
}

#[doc = include_str!("custom_properties.doc/styled.md")]
fn styled(class: &str, style: &str) -> ViewNode {
    el(class, Vec::new()).with_prop(ViewProperty::new(STYLE, ViewValue::Text(style.into())))
}

#[doc = include_str!("custom_properties.doc/screen.md")]
fn screen(root: ViewNode) -> View {
    View::from_fragment(ViewFragment::from_nodes(vec![root]))
}

#[doc = include_str!("custom_properties.doc/settled.md")]
fn settled(sources: &[&str], root: ViewNode) -> (UiRuntime, Vec<NodeInspection>) {
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), VIEWPORT);
    let mut css = CssPipeline::new();
    let sources: Vec<CssSource> = sources
        .iter()
        .map(|text| CssSource::anonymous(*text))
        .collect();
    css.install(&mut runtime, &sources);
    assert!(
        css.errors().is_empty(),
        "the test CSS parses: {:?}",
        css.errors()
    );
    let sheets = css.sheets().to_vec();
    runtime.render_now(&screen(root), &sheets);
    let nodes = runtime.inspect_snapshot().nodes;
    (runtime, nodes)
}

#[doc = include_str!("custom_properties.doc/color_of.md")]
fn color_of(runtime: &UiRuntime, nodes: &[NodeInspection], class: &str) -> StyleColor {
    property_of(runtime, nodes, class, PropertyId::Color)
}

#[doc = include_str!("custom_properties.doc/property_of.md")]
fn property_of(
    runtime: &UiRuntime,
    nodes: &[NodeInspection],
    class: &str,
    property: PropertyId,
) -> StyleColor {
    let node = nodes
        .iter()
        .find(|node| node.classes.iter().any(|name| name == class))
        .unwrap_or_else(|| panic!("no .{class} node"));
    runtime
        .computed_style(node.node)
        .unwrap_or_else(|| panic!(".{class} has no computed style"))
        .get(property)
        .and_then(ComputedValue::as_color)
        .unwrap_or_else(|| panic!(".{class} computed no colour for {property:?}"))
}

#[test]
fn a_var_reference_resolves_to_the_colour_the_root_declared() {
    // The whole chain in one line of CSS: the compiler keeps `--brand`, the
    // pipeline hands it to the runtime, the matcher gives it to the element
    // `:root` selected, and the resolver substitutes it before computing
    // `color`. Break any link and this is black — silently, which is exactly
    // how it went unnoticed.
    let (runtime, nodes) = settled(
        &[":root { --brand: #ff0000 } .a { color: var(--brand) }"],
        el("page", vec![el("a", Vec::new())]),
    );
    assert_eq!(color_of(&runtime, &nodes, "a"), RED);
}

#[test]
fn a_child_inherits_a_custom_property_its_ancestor_declared() {
    // §4.14: custom properties participate in inheritance BEFORE computed-value
    // resolution. `.deep` declares nothing and matches no rule that does; it
    // reads `--brand` purely because `.theme` above it declared one — and it
    // reads `.theme`'s green rather than the root's red, because the nearer
    // declaration is the one it inherits.
    let (runtime, nodes) = settled(
        &[":root { --brand: #ff0000 } \
           .theme { --brand: #00ff00 } \
           .deep { color: var(--brand) } \
           .shallow { color: var(--brand) }"],
        el(
            "page",
            vec![
                el("shallow", Vec::new()),
                el("theme", vec![el("wrap", vec![el("deep", Vec::new())])]),
            ],
        ),
    );
    assert_eq!(
        color_of(&runtime, &nodes, "deep"),
        GREEN,
        "two levels down from the declaration"
    );
    assert_eq!(
        color_of(&runtime, &nodes, "shallow"),
        RED,
        "outside `.theme`, the root's declaration still holds"
    );
}

#[test]
fn a_var_reference_nothing_declared_falls_back_to_the_initial_value() {
    // An unresolvable `var()` makes the declaration invalid at computed-value
    // time, which is `unset`: initial for `background-color` (it does not
    // inherit). The `color` in the same block proves the sheet is wired — the
    // fallback is what the missing name does, not what an unwired pipeline does.
    let (runtime, nodes) = settled(
        &[":root { --brand: #ff0000 } \
           .miss { background-color: var(--nope); color: var(--brand) }"],
        el("page", vec![el("miss", Vec::new())]),
    );
    assert_eq!(
        property_of(&runtime, &nodes, "miss", PropertyId::BackgroundColor),
        StyleColor::TRANSPARENT
    );
    assert_eq!(color_of(&runtime, &nodes, "miss"), RED);
}

#[test]
fn two_stylesheets_each_declaring_a_different_custom_property_both_work() {
    // The interner trap. A compiler is built per source and numbers its symbols
    // from zero, so a custom property named through it would make the FIRST
    // `--name` of every sheet the same property: `--first-brand` and
    // `--second-brand` would be one symbol, and both nodes would come out the
    // same colour — whichever sheet cascaded last.
    let (runtime, nodes) = settled(
        &[
            ":root { --first-brand: #ff0000 } .first { color: var(--first-brand) }",
            ":root { --second-brand: #0000ff } .second { color: var(--second-brand) }",
        ],
        el(
            "page",
            vec![el("first", Vec::new()), el("second", Vec::new())],
        ),
    );
    assert_eq!(color_of(&runtime, &nodes, "first"), RED);
    assert_eq!(color_of(&runtime, &nodes, "second"), BLUE);
}

#[test]
fn a_custom_property_in_the_style_attribute_outranks_the_sheet() {
    // §4.14 origin 5. A custom property is a declaration, so what the author
    // wrote ON the element beats what any selector reached it with — and the
    // `var()` a stylesheet rule wrote reads the element's own answer.
    let (runtime, nodes) = settled(
        &[":root { --brand: #ff0000 } \
           .themed { color: var(--brand) } \
           .plain { color: var(--brand) }"],
        el(
            "page",
            vec![
                styled("themed", "--brand: #0000ff"),
                el("plain", Vec::new()),
            ],
        ),
    );
    assert_eq!(color_of(&runtime, &nodes, "themed"), BLUE);
    assert_eq!(
        color_of(&runtime, &nodes, "plain"),
        RED,
        "the attribute themed its own element, not its siblings"
    );
}

#[test]
fn a_custom_property_declared_inline_is_inherited_by_the_subtree_below_it() {
    // The consequence of doing this in the cascade rather than in the painter:
    // an element themed by its `style` attribute themes everything inside it,
    // which is what makes a per-instance theme worth writing at all.
    let (runtime, nodes) = settled(
        &[":root { --brand: #ff0000 } .child { color: var(--brand) }"],
        el(
            "page",
            vec![
                el("host", vec![el("child", Vec::new())]).with_prop(ViewProperty::new(
                    STYLE,
                    ViewValue::Text("--brand: #0000ff".into()),
                )),
            ],
        ),
    );
    assert_eq!(color_of(&runtime, &nodes, "child"), BLUE);
}

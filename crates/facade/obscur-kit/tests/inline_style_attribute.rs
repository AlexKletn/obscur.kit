#![doc = include_str!("inline_style_attribute.doc/_module.md")]
#![forbid(unsafe_code)]

use std::cell::RefCell;
use std::rc::Rc;

use obscur_kit::bootstrap::CssPipeline;
use obscur_kit::css::source::CssSource;
use obscur_kit::diagnostics::{Diagnostic, DiagnosticSink};
use obscur_kit::geometry::Size;
use obscur_kit::identity::RuntimeId;
use obscur_kit::runtime::UiRuntime;
use obscur_kit::runtime::orchestration::NodeInspection;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::style::property::{PropertyId, PropertyValue};
use obscur_kit::ui_macros::okui;
use obscur_kit::view::view::{View, ViewFragment, ViewNode};

use obscur_kit::primitives::Container;

#[doc = include_str!("inline_style_attribute.doc/SHEET.md")]
const SHEET: &str = ".box { width: 40px; opacity: 0.25; } .pinned { width: 40px !important; }";

#[doc = include_str!("inline_style_attribute.doc/Collected.md")]
#[derive(Default, Clone)]
struct Collected(Rc<RefCell<Vec<String>>>);

impl DiagnosticSink for Collected {
    fn emit(&self, diagnostic: &Diagnostic) {
        self.0
            .borrow_mut()
            .push(format!("{:?}: {}", diagnostic.code, diagnostic.message));
    }
}

#[doc = include_str!("inline_style_attribute.doc/boxed.md")]
fn boxed(class: &str, style: &str) -> ViewNode {
    Container::new().into_view_node().class(class).style(style)
}

#[doc = include_str!("inline_style_attribute.doc/screen.md")]
fn screen(node: ViewNode) -> View {
    View::from_fragment(ViewFragment::from_nodes(vec![node]))
}

#[doc = include_str!("inline_style_attribute.doc/wired.md")]
fn wired() -> (UiRuntime, Vec<Stylesheet>) {
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(400.0, 300.0));
    let mut css = CssPipeline::new();
    css.install(&mut runtime, &[CssSource::anonymous(SHEET)]);
    let sheets = css.sheets().to_vec();
    (runtime, sheets)
}

#[doc = include_str!("inline_style_attribute.doc/settled.md")]
fn settled(node: ViewNode) -> Vec<NodeInspection> {
    let (mut runtime, sheets) = wired();
    runtime.render_now(&screen(node), &sheets);
    runtime.inspect_snapshot().nodes
}

#[doc = include_str!("inline_style_attribute.doc/only.md")]
fn only<'a>(nodes: &'a [NodeInspection], class: &str) -> &'a NodeInspection {
    let mut found = nodes
        .iter()
        .filter(|node| node.classes.iter().any(|name| name == class));
    let node = found.next().unwrap_or_else(|| panic!("no .{class} node"));
    assert!(found.next().is_none(), "more than one .{class} node");
    node
}

#[doc = include_str!("inline_style_attribute.doc/width_of.md")]
fn width_of(nodes: &[NodeInspection], class: &str) -> f32 {
    only(nodes, class).bounds.map_or(0.0, |r| r.size.width)
}

#[test]
fn an_inline_declaration_outranks_the_rule_that_matched_the_same_node() {
    // §4.14 origin 5. `.box` sets 40px and matched this very element; the
    // attribute wins because it is a *higher origin*, not because it out-specified
    // anything — there is no selector here to be specific about.
    let nodes = settled(boxed("box", "width: 120px"));
    assert_eq!(width_of(&nodes, "box"), 120.0);
}

#[test]
fn a_sheets_important_declaration_still_outranks_a_normal_inline_one() {
    // The one direction people get backwards. §4.14 puts every important
    // declaration above every normal one whatever origin either came from, so
    // `!important` remains the escape hatch it exists to be — an author who
    // cannot edit the markup can still overrule it.
    let nodes = settled(boxed("pinned", "width: 120px"));
    assert_eq!(width_of(&nodes, "pinned"), 40.0);
}

#[test]
fn an_important_inline_declaration_takes_it_back() {
    let nodes = settled(boxed("pinned", "width: 120px !important"));
    assert_eq!(width_of(&nodes, "pinned"), 120.0);
}

#[test]
fn the_declarations_the_attribute_did_not_mention_are_left_alone() {
    // A block is folded declaration by declaration, not as a replacement for
    // everything the cascade decided about the node.
    let (mut runtime, sheets) = wired();
    runtime.render_now(&screen(boxed("box", "width: 120px")), &sheets);
    let nodes = runtime.inspect_snapshot().nodes;
    let opacity = runtime
        .computed_style(only(&nodes, "box").node)
        .and_then(|style| style.get(PropertyId::Opacity))
        .map(|value| match value.value() {
            PropertyValue::Number(n) => *n,
            other => panic!("opacity is a number, got {other:?}"),
        })
        .expect("the sheet set an opacity");
    assert!(
        (opacity - 0.25).abs() < 1e-6,
        "the sheet's opacity survived: {opacity}"
    );
}

#[test]
fn without_a_css_front_end_the_attribute_is_carried_and_never_applied() {
    // The honest half of the wiring. A `UiRuntime` driven directly — no
    // `CssPipeline`, so no compiler installed — has no CSS grammar in reach and
    // must not invent declarations. The attribute rides through reconciliation
    // and is simply never read.
    use obscur_kit::css::compiler::StylesheetCompiler;
    use obscur_kit::css::parser::StylesheetParser;

    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(400.0, 300.0));
    assert!(!runtime.has_inline_style_compiler());
    let sheet = StylesheetCompiler::new()
        .compile(&StylesheetParser::new(&CssSource::anonymous(SHEET)).parse())
        .matchable;
    runtime.render_now(&screen(boxed("box", "width: 120px")), &[sheet]);
    let nodes = runtime.inspect_snapshot().nodes;
    assert_eq!(width_of(&nodes, "box"), 40.0, "the sheet's value stood");
}

#[test]
fn a_changed_attribute_restyles_on_the_next_frame() {
    // The incremental restyle path is a different code path from the full one,
    // and a node whose only change is its `style` string has to travel it.
    let (mut runtime, sheets) = wired();
    runtime.render_now(&screen(boxed("box", "width: 120px")), &sheets);
    assert_eq!(width_of(&runtime.inspect_snapshot().nodes, "box"), 120.0);

    runtime.render_now(&screen(boxed("box", "width: 200px")), &sheets);
    assert_eq!(width_of(&runtime.inspect_snapshot().nodes, "box"), 200.0);
}

#[test]
fn an_authored_markup_attribute_means_what_the_builder_means() {
    // The front ends lower `style="…"` to the reserved symbol the style tier
    // reads. Under a hashed one the text would be carried through the whole tree
    // and silently never compiled, which is exactly how `placeholder` used to
    // fail — so this is worth pinning from the markup side, not just from Rust.
    let view = okui! {
        <container class="box" style="width: 120px" />
    };
    let (mut runtime, sheets) = wired();
    runtime.render_now(&view, &sheets);
    assert_eq!(width_of(&runtime.inspect_snapshot().nodes, "box"), 120.0);
}

#[test]
fn a_faulty_declaration_is_reported_and_the_rest_of_the_block_holds() {
    // §1.6: a localized failure AND a report. A `style` that silently declared
    // nothing because one property was misspelled would be indistinguishable
    // from no attribute at all, which is the failure this pipeline exists to
    // avoid — the same one an authored sheet is already protected from.
    let collected = Collected::default();
    let (mut runtime, sheets) = wired();
    runtime.set_diagnostic_sink(Box::new(collected.clone()));
    runtime.render_now(&screen(boxed("box", "float: left; width: 120px")), &sheets);

    assert_eq!(
        width_of(&runtime.inspect_snapshot().nodes, "box"),
        120.0,
        "the declaration that parsed still applied"
    );
    let reported = collected.0.borrow();
    assert!(
        reported.iter().any(|line| line.contains("float")),
        "the dropped declaration was reported: {reported:?}"
    );
}

#[test]
fn an_incremental_rematch_keeps_inline_values_on_untouched_siblings() {
    let (mut runtime, sheets) = wired();
    let view = |left: ViewNode| {
        View::from_fragment(ViewFragment::from_nodes(vec![
            left.with_key("left"),
            boxed("box right", "width: 150px").with_key("right"),
        ]))
    };
    let initial = view(boxed("box left", "width: 120px"));
    runtime.render_now(&initial, &sheets);
    runtime.render_now(&initial, &sheets);
    let changed = view(Container::new().into_view_node().class("box left"));
    runtime.render_now(&changed, &sheets);
    let nodes = runtime.inspect_snapshot().nodes;
    assert_eq!(width_of(&nodes, "left"), 40.0);
    assert_eq!(width_of(&nodes, "right"), 150.0);
}

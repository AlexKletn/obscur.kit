//! §6.15 — the host reports its insets, CSS `env()` lays out around them.
//!
//! End-to-end through the real stack: authored CSS → the CSS frontend →
//! computed style → layout. The interesting part is not that `env()` parses; it
//! is that the value comes from the *host*, changes while the app runs, and that
//! a change actually reaches the arranged geometry rather than being memoized
//! away.
#![forbid(unsafe_code)]

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::{Insets, Size};
use obscur_kit::identity::{ComponentId, NodeId, RuntimeId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::computed::ComputedValue;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::style::property::{EnvironmentValues, Length, PropertyId};
use obscur_kit::view::view::{ID, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

const CSS: &str = "#root { width: 400px; height: 800px; \
                   padding-top: env(safe-area-inset-top); \
                   padding-bottom: env(safe-area-inset-bottom) } \
                   #child { width: 10px; height: 10px }";

fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

fn el(raw: u64, id: &str, kids: Vec<ViewNode>) -> ViewNode {
    ViewNode::element(ComponentId::from_raw(raw))
        .with_prop(ViewProperty::new(ID, ViewValue::Text(id.into())))
        .with_children(ViewFragment::from_nodes(kids))
}

fn view() -> View {
    View::from_fragment(ViewFragment::from_nodes(vec![el(
        1,
        "root",
        vec![el(2, "child", vec![])],
    )]))
}

fn at(rt: &UiRuntime, path: &[usize]) -> NodeId {
    let mut node = rt.tree().root().expect("a root after a frame");
    for &i in path {
        let kids = rt.tree().get(node).expect("live").child_nodes().to_vec();
        node = kids[i];
    }
    node
}

fn px_of(v: Option<&ComputedValue>) -> Option<f32> {
    match v?.as_length()? {
        Length::Px(p) => Some(p.0),
        _ => None,
    }
}

fn origin_y(rt: &UiRuntime, node: NodeId) -> f32 {
    rt.layout_boxes()
        .get(&node)
        .expect("an arranged box")
        .origin()
        .y
}

/// A runtime reporting `insets` as its safe area, one frame rendered.
fn with_safe_area(insets: Insets) -> (UiRuntime, Stylesheet) {
    let sheet = compile(CSS);
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(400.0, 800.0));
    rt.set_safe_area_insets(insets);
    let _ = rt.render_now(&view(), std::slice::from_ref(&sheet));
    (rt, sheet)
}

const NOTCHED: Insets = Insets {
    top: 44.0,
    right: 0.0,
    bottom: 34.0,
    left: 0.0,
};

#[test]
fn an_env_inset_resolves_to_what_the_host_reported() {
    let (rt, _sheet) = with_safe_area(NOTCHED);
    let root = at(&rt, &[0]);
    let style = rt.computed_style(root).expect("a computed style");

    assert_eq!(px_of(style.get(PropertyId::PaddingTop)), Some(44.0));
    assert_eq!(px_of(style.get(PropertyId::PaddingBottom)), Some(34.0));
    assert_eq!(
        origin_y(&rt, at(&rt, &[0, 0])),
        44.0,
        "the content box starts below the notch"
    );
}

/// The default is an answer, not a placeholder: a screen with no cutout has no
/// insets, so a host that never mentions them lays out at zero — and the
/// declaration is *not* dropped.
#[test]
fn a_host_that_reports_nothing_lays_out_at_zero() {
    let sheet = compile(CSS);
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(400.0, 800.0));
    let _ = rt.render_now(&view(), std::slice::from_ref(&sheet));

    let root = at(&rt, &[0]);
    let style = rt.computed_style(root).expect("a computed style");
    assert_eq!(
        px_of(style.get(PropertyId::PaddingTop)),
        Some(0.0),
        "a resolved zero, not a missing declaration"
    );
    assert_eq!(origin_y(&rt, at(&rt, &[0, 0])), 0.0);
}

/// §6.15 lists "dynamic changes" among the typed environment values, and this is
/// the case that matters: a keyboard appears, a device rotates, and the layout
/// has to follow. The style memo is keyed on the viewport and the sheets, so an
/// environment it did not name would let the retained styles be reused and the
/// padding would silently stay where it was.
#[test]
fn a_later_report_moves_the_layout() {
    let (mut rt, sheet) = with_safe_area(Insets::uniform(0.0));
    assert_eq!(origin_y(&rt, at(&rt, &[0, 0])), 0.0);

    let changed = rt.set_safe_area_insets(NOTCHED);
    assert!(changed, "the report was new");
    let _ = rt.render_now(&view(), std::slice::from_ref(&sheet));

    let root = at(&rt, &[0]);
    let style = rt.computed_style(root).expect("a computed style");
    assert_eq!(px_of(style.get(PropertyId::PaddingTop)), Some(44.0));
    assert_eq!(
        origin_y(&rt, at(&rt, &[0, 0])),
        44.0,
        "the child moved down when the host reported the notch"
    );
}

#[test]
fn reporting_the_same_values_again_changes_nothing() {
    let (mut rt, _sheet) = with_safe_area(NOTCHED);
    assert!(
        !rt.set_safe_area_insets(NOTCHED),
        "an unchanged report is not a change"
    );
    assert_eq!(rt.environment().safe_area, NOTCHED);
}

/// The keyboard is a separate environment value from the safe area: content
/// clears a home indicator *and* an on-screen keyboard, and they are different
/// distances. Setting one must not disturb the other.
#[test]
fn the_keyboard_inset_is_independent_of_the_safe_area() {
    let css = "#root { width: 400px; height: 800px; \
               padding-top: env(safe-area-inset-top); \
               padding-bottom: env(keyboard-inset-height) } \
               #child { width: 10px; height: 10px }";
    let sheet = compile(css);
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(400.0, 800.0));
    rt.set_safe_area_insets(NOTCHED);
    rt.set_keyboard_insets(Insets {
        top: 0.0,
        right: 0.0,
        bottom: 300.0,
        left: 0.0,
    });
    let _ = rt.render_now(&view(), std::slice::from_ref(&sheet));

    let root = at(&rt, &[0]);
    let style = rt.computed_style(root).expect("a computed style");
    assert_eq!(px_of(style.get(PropertyId::PaddingTop)), Some(44.0));
    assert_eq!(
        px_of(style.get(PropertyId::PaddingBottom)),
        Some(300.0),
        "the keyboard's height, not the safe area's 34px bottom"
    );
    assert_eq!(
        rt.environment().safe_area.bottom,
        34.0,
        "and it is still 34"
    );
}

/// An unmodelled name with a fallback lays out at the fallback; without one the
/// declaration is dropped. A typo must not read as an inset that happens to be
/// absent.
#[test]
fn an_unmodelled_name_falls_back_or_is_dropped() {
    let css = "#root { width: 400px; height: 800px; \
               padding-top: env(titlebar-area-height, 12px); \
               padding-bottom: env(safe-area-inset-topp) } \
               #child { width: 10px; height: 10px }";
    let sheet = compile(css);
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(400.0, 800.0));
    rt.set_safe_area_insets(NOTCHED);
    let _ = rt.render_now(&view(), std::slice::from_ref(&sheet));

    let root = at(&rt, &[0]);
    let style = rt.computed_style(root).expect("a computed style");
    assert_eq!(
        px_of(style.get(PropertyId::PaddingTop)),
        Some(12.0),
        "the fallback carried the declaration"
    );
    assert_ne!(
        px_of(style.get(PropertyId::PaddingBottom)),
        Some(34.0),
        "a misspelt name must not resolve to a real inset"
    );
}

#[test]
fn the_environment_is_reported_back_as_a_whole() {
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(400.0, 800.0));
    let keyboard = Insets {
        top: 0.0,
        right: 0.0,
        bottom: 300.0,
        left: 0.0,
    };
    rt.set_environment(
        EnvironmentValues::none()
            .with_safe_area(NOTCHED)
            .with_keyboard(keyboard),
    );
    assert_eq!(rt.environment().safe_area, NOTCHED);
    assert_eq!(rt.environment().keyboard, keyboard);
}

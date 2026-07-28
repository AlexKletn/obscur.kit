//! `font-size` accepts a math function, so type can follow the window
//! (css-values-4 §10, css-fonts-4 §3.5).
//!
//! It did not. A `calc()` / `min()` / `max()` / `clamp()` value is a
//! `PropertyValue::Calc`, not a `Length`, and the font-size path read only the
//! length — so the whole declaration was dropped and the element kept the
//! initial `medium`. Every math function computed to **16px at every window
//! size**, which quietly ruled out responsive typography altogether:
//! `clamp(16px, 5vmin, 30px)` is exactly how type is meant to track the window
//! between bounds.
//!
//! The tell was that it looked like *nothing*: a fixed label in a key that grows
//! with the window drifts out of proportion, and reads as a centring bug rather
//! than as a dropped declaration.
#![forbid(unsafe_code)]

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::Size;
use obscur_kit::identity::{ComponentId, RuntimeId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::computed::ComputedValue;
use obscur_kit::style::property::{Length, PropertyId, PropertyValue};
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

/// The computed `font-size` of `.box` under `css` at a `w x h` viewport.
fn font_size(css: &str, w: f32, h: f32) -> Option<f32> {
    let source = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&source).parse();
    let sheet = StylesheetCompiler::new().compile(&parsed).matchable;

    let view = View::from_fragment(ViewFragment::from_nodes(vec![
        ViewNode::element(ComponentId::from_raw(91))
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text("box".into()))),
    ]));
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(w, h));
    let _ = runtime.render_now(&view, std::slice::from_ref(&sheet));

    let root = runtime.tree().root()?;
    let node = runtime.tree().get(root)?.child_nodes()[0];
    match runtime
        .computed_style(node)?
        .get(PropertyId::FontSize)
        .map(ComputedValue::value)
    {
        Some(PropertyValue::Length(Length::Px(px))) => Some(px.0),
        _ => None,
    }
}

#[test]
fn calc_is_folded_rather_than_dropped() {
    assert_eq!(
        font_size(".box { font-size: calc(10px + 10px) }", 400.0, 400.0),
        Some(20.0),
        "a dropped declaration would leave the initial 16px"
    );
}

#[test]
fn min_and_max_pick_their_side() {
    assert_eq!(
        font_size(".box { font-size: min(20px, 30px) }", 400.0, 400.0),
        Some(20.0)
    );
    assert_eq!(
        font_size(".box { font-size: max(20px, 30px) }", 400.0, 400.0),
        Some(30.0)
    );
}

/// The case the whole fix is for: type that follows the window and stops at both
/// ends.
#[test]
fn clamp_tracks_the_window_between_its_bounds() {
    let css = ".box { font-size: clamp(16px, 5vmin, 30px) }";

    // 5vmin of 360 is 18 — inside the bounds, so it tracks.
    assert_eq!(font_size(css, 360.0, 620.0), Some(18.0));
    // 5vmin of 1200 is 60 — above the ceiling, so it stops at 30.
    assert_eq!(font_size(css, 1200.0, 1500.0), Some(30.0));
    // 5vmin of 200 is 10 — below the floor, so it stops at 16.
    assert_eq!(font_size(css, 200.0, 200.0), Some(16.0));
}

/// A font-relative unit inside the expression means the **parent's** size, per
/// CSS Values 4 §6.1.2 — the same rule a bare `font-size: 2em` follows, and the
/// reason the expression is folded against the parent rather than against the
/// element itself.
#[test]
fn an_em_inside_the_expression_means_the_parent() {
    // The root's parent size is the initial 16px, so `calc(2em)` is 32.
    assert_eq!(
        font_size(".box { font-size: calc(2em + 0px) }", 400.0, 400.0),
        Some(32.0)
    );
}

/// A plain length still behaves exactly as before — the fix adds a branch, it
/// does not reroute the ordinary path.
#[test]
fn a_plain_length_is_unaffected() {
    assert_eq!(
        font_size(".box { font-size: 26px }", 360.0, 620.0),
        Some(26.0)
    );
    assert_eq!(
        font_size(".box { font-size: 26px }", 1200.0, 1500.0),
        Some(26.0)
    );
}

/// A **floor** is what makes type stop following the window: below it the label
/// keeps its size while the box around it carries on shrinking, and ends up
/// several times wider than the key it sits in.
///
/// Measured on the demo's own shape, this was 18px of label inside a 1.5px key.
/// A ceiling alone — `min(<vmin>, <px>)` — keeps type sane on a wall-sized
/// window and lets it shrink the whole way on a squeezed one.
#[test]
fn a_ceiling_only_size_keeps_shrinking_where_a_floor_would_stop() {
    let ceiling = ".box { font-size: min(6vmin, 34px) }";
    let floored = ".box { font-size: clamp(18px, 6vmin, 34px) }";

    // Both agree while the window is roomy enough that neither bound bites.
    assert_eq!(
        font_size(ceiling, 400.0, 600.0),
        font_size(floored, 400.0, 600.0)
    );

    // Squeezed, the floor stops and the ceiling keeps going.
    assert_eq!(font_size(floored, 100.0, 160.0), Some(18.0));
    assert_eq!(font_size(ceiling, 100.0, 160.0), Some(6.0));

    // And the ceiling still holds at the other end.
    assert_eq!(font_size(ceiling, 1200.0, 1500.0), Some(34.0));
}

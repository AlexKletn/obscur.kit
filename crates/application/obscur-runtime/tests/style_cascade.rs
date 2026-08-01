#![doc = include_str!("style_cascade.doc/_module.md")]
#![forbid(unsafe_code)]

use obscur_identity::FrameId;
use obscur_runtime::UiRuntime;
use obscur_scene::command::DisplayList;
use obscur_style::cascade::CascadeOrigin;
use obscur_style::computed::ComputedValue;
use obscur_style::matching::Stylesheet;
use obscur_style::property::{PropertyId, StyleColor};
use obscur_style::selector::PseudoState;
use obscur_testkit::fixture::{ButtonElement, ClassRule, CounterPanel, TestFixture};
use obscur_testkit::selector::NodePath;

/// The computed `background-color` of the first element below the root.
fn background_of(runtime: &UiRuntime) -> Option<StyleColor> {
    let node = NodePath::new(&[0]).resolve(runtime);
    runtime
        .computed_style(node)
        .and_then(|s| s.get(PropertyId::BackgroundColor))
        .and_then(ComputedValue::as_color)
}

/// Authored CSS reaches paint through the runtime.
#[test]
fn render_now_paints_matched_background() {
    // S6 acceptance mirror: authored CSS reaches paint through the runtime.
    let mut runtime = TestFixture::runtime();
    let view = CounterPanel::view();
    let sheet = CounterPanel::sheet();

    let result = runtime.render_now(&view, std::slice::from_ref(&sheet));
    assert!(result.reconciled());
    assert!(result.restyled(), "the matched node resolved styles");
    assert!(result.relaid_out(), "the sized node arranged a box");
    let list: &DisplayList = result.snapshot().display_list();
    assert!(!list.is_empty(), "matched background produced a fill");
    assert!(result.is_balanced());
    assert_eq!(result.id(), FrameId::FIRST);
}

/// A recorded pseudo-state changes what the cascade resolves, and that reaches
/// paint.
#[test]
fn hover_state_reaches_paint_through_the_runtime() {
    // `.counter:hover { background-color: red; width: 100px; height: 50px }`.
    let hover_sheet = Stylesheet::new(
        CascadeOrigin::GlobalApp,
        vec![
            ClassRule::hover("counter")
                .background(1.0, 0.0, 0.0)
                .sized(100.0, 50.0)
                .build(),
        ],
    );
    let view = CounterPanel::view();
    let mut runtime = TestFixture::runtime();
    let sheets = std::slice::from_ref(&hover_sheet);

    // First frame: no hover → the hover-gated rule paints nothing.
    let cold = runtime.render_now(&view, sheets);
    assert!(
        cold.snapshot().display_list().is_empty(),
        ":hover paints nothing until hover is recorded"
    );

    // Record hover on the node and re-render → the rule now paints.
    let child = NodePath::new(&[0]).resolve(&runtime);
    assert_eq!(runtime.pseudo_state(child), PseudoState::empty());
    runtime.set_pseudo_state(child, PseudoState::HOVER);
    assert_eq!(runtime.pseudo_state(child), PseudoState::HOVER);

    let hot = runtime.render_now(&view, sheets);
    assert!(
        !hot.snapshot().display_list().is_empty(),
        ":hover reaches paint once recorded"
    );

    // Clearing state reverts to the cold behaviour.
    runtime.clear_pseudo_states();
    let cleared = runtime.render_now(&view, sheets);
    assert!(cleared.snapshot().display_list().is_empty());
}

/// The UA sheet is the base look when nothing else styles the element.
#[test]
fn ua_default_sheet_supplies_the_base_background_when_no_author_rule() {
    // A UA sheet at the LOWEST origin (`SystemDefault`) installed via the
    // default slot; NO author sheets. The button's computed background is the
    // UA fill — the base look shows through.
    let mut runtime = TestFixture::runtime();
    runtime.set_default_stylesheet(Stylesheet::new(
        CascadeOrigin::SystemDefault,
        vec![ButtonElement::filled(ButtonElement::UA_FILL)],
    ));
    let view = ButtonElement::view();
    let _ = runtime.render_now(&view, &[]);

    assert_eq!(
        background_of(&runtime),
        Some(ButtonElement::UA_FILL),
        "UA default background must show through"
    );
}

/// …and it is a base the author can overrule, by origin rather than specificity.
#[test]
fn author_sheet_overrides_the_ua_default_background() {
    // The crux: with the UA sheet at `SystemDefault` AND an author sheet at
    // `GlobalApp` both styling `button { background }`, the AUTHOR value wins by
    // origin precedence (§4.14) — the UA sheet is a true, overridable base.
    let mut runtime = TestFixture::runtime();
    runtime.set_default_stylesheet(Stylesheet::new(
        CascadeOrigin::SystemDefault,
        vec![ButtonElement::filled(ButtonElement::UA_FILL)],
    ));
    // Author override: `button { background: #cc0000 }`.
    let author = Stylesheet::new(
        CascadeOrigin::GlobalApp,
        vec![ButtonElement::filled(StyleColor::rgba(0.8, 0.0, 0.0, 1.0))],
    );
    let view = ButtonElement::view();
    let _ = runtime.render_now(&view, std::slice::from_ref(&author));

    let bg = background_of(&runtime);
    assert_eq!(
        bg,
        Some(StyleColor::rgba(0.8, 0.0, 0.0, 1.0)),
        "author sheet (GlobalApp) must beat the UA default (SystemDefault)"
    );
    assert_ne!(
        bg,
        Some(ButtonElement::UA_FILL),
        "the UA value must NOT survive the override"
    );
}

/// With no UA sheet installed there is no UA-derived value to inherit.
#[test]
fn no_default_sheet_leaves_the_element_unstyled() {
    // Absent a default sheet, a bare `button` has NO UA-derived background: it
    // resolves to the initial transparent — byte-identical to before the slot
    // existed.
    let mut runtime = TestFixture::runtime();
    assert!(!runtime.has_default_stylesheet());
    let view = ButtonElement::view();
    let _ = runtime.render_now(&view, &[]);

    // The initial background is transparent, never the UA fill.
    let bg = background_of(&runtime);
    assert_eq!(bg, Some(StyleColor::TRANSPARENT));
    assert_ne!(bg, Some(ButtonElement::UA_FILL));
}

/// A structural pseudo-class bites exactly when the element reports the state,
/// and stops biting when it stops.
#[test]
fn disabled_pseudo_class_bites_only_a_disabled_button_and_re_resolves_on_toggle() {
    // Author sheet: base `button { background: UA_FILL }` plus a
    // `button:disabled { background: #808080 }` overlay. The disabled overlay
    // must apply ONLY when the button reports the reserved DISABLED state.
    let disabled_fill = StyleColor::rgba(0.5, 0.5, 0.5, 1.0);
    let sheet = Stylesheet::new(
        CascadeOrigin::GlobalApp,
        vec![
            ButtonElement::filled(ButtonElement::UA_FILL),
            ButtonElement::disabled_filled(disabled_fill),
        ],
    );
    let sheets = std::slice::from_ref(&sheet);

    let mut runtime = TestFixture::runtime();

    // Enabled button: the base rule wins, the :disabled overlay is inert.
    let _ = runtime.render_now(&ButtonElement::state_view(false), sheets);
    assert_eq!(
        background_of(&runtime),
        Some(ButtonElement::UA_FILL),
        "an enabled button must NOT pick up the :disabled overlay"
    );

    // Toggle to disabled: the reconcile updates the reserved attribute, the
    // node restyles, and the :disabled overlay now wins.
    let _ = runtime.render_now(&ButtonElement::state_view(true), sheets);
    assert_eq!(
        background_of(&runtime),
        Some(disabled_fill),
        ":disabled must bite once the button reports the disabled state"
    );

    // Toggle back to enabled: the overlay drops and the base fill re-resolves.
    let _ = runtime.render_now(&ButtonElement::state_view(false), sheets);
    assert_eq!(
        background_of(&runtime),
        Some(ButtonElement::UA_FILL),
        "clearing disabled must re-resolve the computed style back to the base"
    );
}

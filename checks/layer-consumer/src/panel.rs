#![doc = include_str!("panel.doc/_module.md")]

use std::cell::RefCell;
use std::rc::Rc;

use obscur_kit::headless::input::SelectorTarget;
use obscur_kit::identity::InternedSymbol;
use obscur_kit::interaction::event::ListenerPhase;
use obscur_kit::windowing::layer::{
    LayerBand, LayerFocusPolicy, LayerMargins, LayerReservation, LayerSettings,
};
use obscur_kit::{App, AppError, LayerControl};

use crate::key::{keyboard_view, register_key, sample_rows};

#[doc = include_str!("panel.doc/CMD_SHIFT.md")]
pub const CMD_SHIFT: InternedSymbol = InternedSymbol::named("shift");

#[doc = include_str!("panel.doc/CMD_FLOAT.md")]
pub const CMD_FLOAT: InternedSymbol = InternedSymbol::named("float");

#[doc = include_str!("panel.doc/KEYBOARD_TEST_ID.md")]
pub const KEYBOARD_TEST_ID: &str = "keyboard-root";

#[doc = include_str!("panel.doc/CSS.md")]
pub const CSS: &str = ".keyboard { width: 100%; height: 100% } .row { height: 25% }";

#[doc = include_str!("panel.doc/docked.md")]
#[must_use]
pub fn docked() -> LayerSettings {
    LayerSettings::bottom_bar(320)
        .with_band(LayerBand::Overlay)
        .with_focus(LayerFocusPolicy::Never)
}

#[doc = include_str!("panel.doc/floating.md")]
#[must_use]
pub fn floating() -> LayerSettings {
    docked()
        .with_reservation(LayerReservation::None)
        .with_margin(LayerMargins::uniform(16))
}

#[doc = include_str!("panel.doc/run_keyboard.md")]
pub fn run_keyboard() -> Result<(), AppError> {
    let shifted = Rc::new(RefCell::new(false));
    let view_shift = Rc::clone(&shifted);
    let toggle_shift = Rc::clone(&shifted);

    let app = App::new("layer-consumer keyboard")
        .app_id("dev.obscur.layer-consumer")
        .layer(docked());

    // Both handles come from the `App` rather than being made here: there is one
    // surface and one runtime, so there is one handle to each.
    let surface: LayerControl = app.layer_control();
    let control = app.control();

    // The one route to a user component, and the reason this fixture exists: it is
    // queued *before* `run`, and something has to drain the queue on the layer path
    // for the tag below to be anything but an anonymous box.
    control.with_runtime(register_key);

    app.stylesheet(CSS)
        .view(move || {
            let rows = sample_rows();
            let _ = *view_shift.borrow();
            keyboard_view(&rows)
        })
        .on_command(CMD_SHIFT, move || {
            let mut state = toggle_shift.borrow_mut();
            *state = !*state;
        })
        // Queued against a name that resolves only once a tree exists, so nothing
        // but the loop's post-commit `apply` can bind it. A layer runner that
        // dropped the control queue would leave this handler permanently unreachable.
        .on_command_at(SelectorTarget::test_id(KEYBOARD_TEST_ID), CMD_FLOAT, {
            let surface = surface.clone();
            move || surface.reconfigure(floating())
        })
        .add_event_listener(
            SelectorTarget::test_id(KEYBOARD_TEST_ID),
            CMD_SHIFT,
            ListenerPhase::Capture,
            || {},
        )
        .run()
}
